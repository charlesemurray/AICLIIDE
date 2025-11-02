# Parallel Sessions with Worktrees - Updated Implementation Plan V2

## Changes from V1

### Key Updates
1. **Workspace-Aware**: Support multiple git repos in current directory
2. **Lazy Creation**: Only create worktrees when Q modifies files
3. **Base Branch Tracking**: Merge back to branch repo started on
4. **Scope Flexibility**: Workspace-level or repo-level conversations

### Design Simplifications
- Workspace = current directory (no complex detection)
- Create worktrees on-demand (not upfront)
- Base branch = current branch (no guessing)

---

## Updated Phase Breakdown

### Phase 1: Git Detection & Worktree Management (Week 1)

**Status**: ✅ Core complete, needs multi-repo support

#### Task 1.1-1.4: ✅ COMPLETE
- Git module structure
- Context detection
- Worktree management
- Error handling

#### Task 1.5: Add Multi-Repo Detection (4 hours) - NEW
**File**: `crates/chat-cli/src/git/workspace.rs` (NEW)

```rust
pub struct WorkspaceContext {
    pub root: PathBuf,
    pub repos: Vec<RepoInfo>,
    pub scope: ConversationScope,
    pub session_name: Option<String>,
}

pub struct RepoInfo {
    pub path: PathBuf,
    pub name: String,
    pub base_branch: String,
    pub worktree: Option<PathBuf>,
    pub has_changes: bool,
}

pub enum ConversationScope {
    Workspace,
    Repo(PathBuf),
}

pub fn detect_workspace_repos(root: &Path) -> Result<Vec<RepoInfo>>;
pub fn find_repo_for_file(file: &Path, repos: &[RepoInfo]) -> Option<&RepoInfo>;
```

**Tests**:
- Detect multiple repos in workspace
- Handle nested repos
- Handle non-git directories

**Estimated time**: 4 hours

**Phase 1 Total**: 26 hours (22 original + 4 new)

---

### Phase 2: Workspace & Conversation Storage (Week 2)

**Status**: Updated with workspace awareness

**Effort**: 12 hours (reduced from 16, simplified by workspace model)

#### Task 2.1: Workspace Detection (3 hours)
**File**: `crates/chat-cli/src/git/workspace.rs`

```rust
pub fn detect_workspace_context(path: &Path) -> Result<WorkspaceContext> {
    let repos = detect_workspace_repos(path)?;
    
    Ok(WorkspaceContext {
        root: path.to_path_buf(),
        repos,
        scope: ConversationScope::Workspace,
        session_name: None,
    })
}
```

#### Task 2.2: Conversation Scope Resolution (3 hours)
**File**: `crates/chat-cli/src/cli/chat/conversation_scope.rs` (NEW)

```rust
pub fn resolve_conversation_path(
    workspace: &WorkspaceContext,
    scope: ConversationScope
) -> PathBuf {
    match scope {
        ConversationScope::Workspace => {
            workspace.root.join(".amazonq/conversation.json")
        }
        ConversationScope::Repo(repo_path) => {
            repo_path.join(".amazonq/conversation.json")
        }
    }
}
```

#### Task 2.3: Extend SessionMetadata (2 hours)
**File**: Modify existing Session Management V2 metadata

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    // ... existing fields ...
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_info: Option<WorkspaceInfo>,
}

pub struct WorkspaceInfo {
    pub root: PathBuf,
    pub repos_with_worktrees: Vec<RepoWorktreeInfo>,
    pub scope: ConversationScope,
}
```

#### Task 2.4: Integration with Session Repository (4 hours)
- Use existing SessionRepository trait
- Add workspace context to save/load
- Handle both workspace and repo-level storage

**Phase 2 Total**: 12 hours

---

### Phase 3: Lazy Worktree Creation (Week 3)

**Status**: New approach - create on-demand

**Effort**: 16 hours (reduced from 18, simpler logic)

#### Task 3.1: File Modification Interceptor (4 hours)
**File**: `crates/chat-cli/src/cli/chat/tools/fs_write.rs` (modify)

```rust
impl FsWrite {
    pub async fn invoke(&self, os: &Os, workspace: &WorkspaceContext) -> Result<InvokeOutput> {
        // Check if we need a worktree
        if workspace.session_name.is_some() {
            let translated_path = ensure_worktree_for_file(
                &self.path,
                workspace
            ).await?;
            
            // Use translated path
            self.write_to_path(&translated_path, os).await
        } else {
            // Normal write
            self.write_to_path(&self.path, os).await
        }
    }
}
```

#### Task 3.2: Lazy Worktree Creator (6 hours)
**File**: `crates/chat-cli/src/git/lazy_worktree.rs` (NEW)

```rust
pub async fn ensure_worktree_for_file(
    file: &Path,
    workspace: &mut WorkspaceContext
) -> Result<PathBuf> {
    // Find repo containing file
    let repo = workspace.repos.iter_mut()
        .find(|r| file.starts_with(&r.path))
        .ok_or(GitError::NotInRepo)?;
    
    // Create worktree if needed
    if repo.worktree.is_none() {
        let worktree_path = create_repo_worktree(
            &repo,
            workspace.session_name.as_ref().unwrap(),
            &workspace.root
        ).await?;
        
        repo.worktree = Some(worktree_path);
    }
    
    // Translate path
    translate_path_to_worktree(file, repo)
}

async fn create_repo_worktree(
    repo: &RepoInfo,
    session_name: &str,
    workspace_root: &Path
) -> Result<PathBuf> {
    let worktree_root = workspace_root
        .parent()
        .unwrap()
        .join(format!("{}-{}", 
            workspace_root.file_name().unwrap().to_str().unwrap(),
            session_name
        ));
    
    let worktree_path = worktree_root.join(&repo.name);
    
    create_worktree(
        &repo.path,
        session_name,
        &repo.base_branch,
        Some(worktree_path.clone())
    )?;
    
    // Configure separate build directory
    configure_worktree_build(&worktree_path).await?;
    
    Ok(worktree_path)
}
```

#### Task 3.3: Path Translation (3 hours)
**File**: `crates/chat-cli/src/git/path_translation.rs` (NEW)

```rust
pub fn translate_path_to_worktree(
    path: &Path,
    repo: &RepoInfo
) -> Result<PathBuf> {
    let relative = path.strip_prefix(&repo.path)?;
    Ok(repo.worktree.as_ref().unwrap().join(relative))
}

pub fn translate_path_from_worktree(
    path: &Path,
    repo: &RepoInfo
) -> Result<PathBuf> {
    let relative = path.strip_prefix(repo.worktree.as_ref().unwrap())?;
    Ok(repo.path.join(relative))
}
```

#### Task 3.4: Build Configuration (3 hours)
**File**: `crates/chat-cli/src/git/build_config.rs` (NEW)

```rust
pub async fn configure_worktree_build(worktree_path: &Path) -> Result<()> {
    // Create .cargo/config.toml
    let cargo_dir = worktree_path.join(".cargo");
    fs::create_dir_all(&cargo_dir).await?;
    
    fs::write(
        cargo_dir.join("config.toml"),
        "[build]\ntarget-dir = \"target\"\n"
    ).await?;
    
    // Add to .gitignore
    add_to_gitignore(worktree_path, ".cargo/config.toml").await?;
    
    Ok(())
}
```

**Phase 3 Total**: 16 hours

---

### Phase 4: Worktree Naming (Week 4)

**Status**: No changes from V1

**Effort**: 18 hours

Same as before - LLM-based name generation, conflict detection, interactive confirmation.

---

### Phase 5: Session Discovery & Management (Week 5)

**Status**: Enhanced with workspace info

**Effort**: 16 hours (increased from 14, more info to display)

#### Task 5.1: Workspace-Aware Session Scanner (6 hours)
**File**: `crates/chat-cli/src/cli/chat/session_scanner.rs`

```rust
pub async fn scan_workspace_sessions(
    workspace: &WorkspaceContext,
    session_repo: &dyn SessionRepository
) -> Result<Vec<SessionInfo>> {
    let mut sessions = Vec::new();
    
    // Scan for workspace-level sessions
    let workspace_sessions = scan_workspace_level(workspace).await?;
    sessions.extend(workspace_sessions);
    
    // Scan for repo-level sessions
    for repo in &workspace.repos {
        let repo_sessions = scan_repo_level(repo).await?;
        sessions.extend(repo_sessions);
    }
    
    Ok(sessions)
}

pub struct SessionInfo {
    pub name: String,
    pub scope: ConversationScope,
    pub repos_with_worktrees: Vec<String>,
    pub base_branches: HashMap<String, String>,
    pub last_activity: SystemTime,
    pub status: SessionStatus,
}
```

#### Task 5.2: Enhanced /sessions Display (5 hours)
```
📋 Active Sessions:

Workspace: /workspace/my-project/

  • main (workspace-level)
    Repos: frontend (main), backend (develop)
    Status: Idle
    Last activity: 2 hours ago
    
  • feature/auth (workspace-level)  ← current
    Repos: frontend (worktree), backend (worktree)
    Base branches: frontend→main, backend→develop
    Status: Working
    Last activity: 10 seconds ago
    Files modified: 5
    
  • bugfix/parser (repo-level: backend)
    Repo: backend (worktree)
    Base branch: develop
    Status: Waiting for approval
    Last activity: 2 minutes ago
```

#### Task 5.3: Cleanup Commands (3 hours)
- Clean up workspace-level sessions
- Clean up repo-level sessions
- Remove orphaned worktrees

#### Task 5.4: Disk Usage Display (2 hours)
```
> "/sessions disk-usage"

Session Disk Usage:

  • feature/auth
    frontend/target: 2.3 GB
    backend/target: 1.8 GB
    Total: 4.1 GB
    
  • bugfix/parser
    backend/target: 1.8 GB
    Total: 1.8 GB
    
Overall: 5.9 GB
```

**Phase 5 Total**: 16 hours

---

### Phase 6: Merge Workflow (Week 6)

**Status**: Enhanced for multi-repo

**Effort**: 34 hours (increased from 30, multi-repo complexity)

#### Task 6.1: Multi-Repo Merge Command (4 hours)
**File**: `crates/chat-cli/src/cli/chat/cli/merge.rs`

```rust
pub async fn merge_workspace_session(
    workspace: &WorkspaceContext,
    session_name: &str
) -> Result<WorkspaceMergeResult> {
    let mut results = Vec::new();
    
    for repo in &workspace.repos {
        if let Some(worktree) = &repo.worktree {
            let result = merge_repo_worktree(
                worktree,
                &repo.path,
                &repo.base_branch
            ).await?;
            results.push(result);
        }
    }
    
    Ok(WorkspaceMergeResult { repos: results })
}
```

#### Task 6.2: Per-Repo Validation (4 hours)
- Validate each repo independently
- Check for conflicts per repo
- Predict merge issues

#### Task 6.3: Per-Repo Merge Execution (6 hours)
- Merge each repo to its base branch
- Handle conflicts per repo
- Rollback on failure

#### Task 6.4: Multi-Repo Conflict Resolution (10 hours)
- Detect conflicts in each repo
- LLM-assisted resolution per repo
- Apply resolutions

#### Task 6.5: Coordinated Cleanup (4 hours)
- Remove all worktrees
- Clean up workspace conversation
- Update session metadata

#### Task 6.6: Merge UI (6 hours)
```
Merge Preview:

Workspace: /workspace/my-project/
Session: feature/auth

Repos to merge:
  1. frontend/
     Base: main
     Files: 3 modified
     Conflicts: 0 expected
     
  2. backend/
     Base: develop
     Files: 2 modified
     Conflicts: 1 expected (auth.rs)

Proceed with merge? [y/n]:
```

**Phase 6 Total**: 34 hours

---

## Integration Tasks (Updated)

### Integration 1: ChatSession Initialization (6 hours)
**When**: After Phase 2

```rust
impl ChatSession {
    pub async fn new(args: ChatArgs, os: &mut Os) -> Result<Self> {
        // Detect workspace
        let workspace = detect_workspace_context(&current_dir())?;
        
        // Resolve worktree strategy
        let strategy = resolve_worktree_strategy(&args, &workspace).await?;
        
        // Create session name if needed
        if let WorktreeStrategy::Create(name) = strategy {
            workspace.session_name = Some(name);
        }
        
        // Continue with existing initialization
    }
}
```

### Integration 2: Tool Execution Context (8 hours)
**When**: After Phase 3

- Modify fs_write to use lazy worktree creation
- Modify fs_read to translate paths
- Modify execute_bash to run in correct directory

### Integration 3: Skills System (3 hours)
**When**: After Phase 3

```rust
// Add to SkillMetadata
pub struct SkillMetadata {
    // ... existing fields ...
    
    pub conversation_scope: Option<ConversationScope>,
    pub repo_path: Option<PathBuf>,
}
```

### Integration 4: Creation System (3 hours)
**When**: After Phase 3

- Creation flows can specify scope
- Auto-detect if creation is repo-specific

**Total Integration**: 20 hours

---

## Updated Timeline

| Phase | Original | V2 | Change | Notes |
|-------|----------|-----|--------|-------|
| Phase 1 | 22h | 26h | +4h | Multi-repo detection |
| Phase 2 | 10h | 12h | +2h | Workspace awareness |
| Phase 3 | 12h | 16h | +4h | Lazy creation logic |
| Phase 4 | 18h | 18h | 0h | No changes |
| Phase 5 | 14h | 16h | +2h | Workspace display |
| Phase 6 | 30h | 34h | +4h | Multi-repo merge |
| Integration | 16h | 20h | +4h | More integration points |
| **Total** | **122h** | **142h** | **+20h** | ~4.5 weeks |

**With 30% buffer**: 185 hours / **23 days / 4.5-5 weeks**

---

## New Files to Create

```
crates/chat-cli/src/git/
├── workspace.rs              # NEW - Workspace detection
├── lazy_worktree.rs          # NEW - Lazy worktree creation
├── path_translation.rs       # NEW - Path translation
└── build_config.rs           # NEW - Build configuration

crates/chat-cli/src/cli/chat/
└── conversation_scope.rs     # NEW - Scope resolution
```

---

## Success Criteria (Updated)

1. **Multi-Repo Support**: ✅ Can work across multiple git repos
2. **Lazy Creation**: ✅ Only creates worktrees when needed
3. **Base Branch Tracking**: ✅ Merges to correct branch per repo
4. **Scope Flexibility**: ✅ Workspace and repo-level conversations
5. **Build Isolation**: ✅ Separate build artifacts per worktree
6. **Clean Merge**: ✅ All repos merge to their base branches
7. **Disk Efficiency**: ✅ Only creates worktrees for modified repos

---

## Risk Assessment (Updated)

### New Risks

1. **Multi-Repo Complexity**: More repos = more edge cases
   - Mitigation: Thorough testing with 1, 2, 5, 10 repos

2. **Lazy Creation Timing**: When exactly to create worktrees?
   - Mitigation: Clear rules, user confirmation

3. **Path Translation Bugs**: Easy to mix up paths
   - Mitigation: Comprehensive path translation tests

4. **Partial Worktree State**: Some repos have worktrees, some don't
   - Mitigation: Clear UI showing which repos have worktrees

### Existing Risks (Still Apply)

- Git command parsing
- Conversation data loss
- Merge conflicts
- LLM name generation
- Performance with large repos

---

## Next Steps

1. ✅ Complete Phase 1 (git module) - DONE
2. ⏳ Add multi-repo detection (Task 1.5)
3. ⏳ Begin Phase 2 (workspace detection)
4. ⏳ Implement lazy worktree creation (Phase 3)
