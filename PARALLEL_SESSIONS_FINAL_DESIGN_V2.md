# Parallel Sessions with Git Worktrees - Final Design V2

## Overview

Enable users to run multiple interactive Q CLI sessions simultaneously on the same workspace, with automatic worktree management for git repositories. Supports workspaces with multiple git repos and lazy worktree creation.

## Core Principles

1. **Workspace = Current Directory** - No complex workspace detection
2. **Lazy Worktree Creation** - Only create worktrees when needed
3. **Base Branch = Current Branch** - Merge back to whatever branch repo started on
4. **Workspace-Level by Default** - Conversations span all repos unless scoped
5. **Multi-Repo Support** - Handle workspaces with multiple git repositories

## Workspace Model

### Workspace Structure

```
/workspace/my-project/              # Workspace root (current directory)
├── .amazonq/
│   └── conversation.json           # Workspace-level conversation
├── frontend/                       # Git repo 1 (on main)
│   ├── .git/
│   └── src/
├── backend/                        # Git repo 2 (on develop)
│   ├── .git/
│   └── src/
└── docs/                           # Not a git repo
    └── README.md
```

### Worktree Structure (Created on Demand)

```
/workspace/
├── my-project/                     # Main workspace
│   ├── .amazonq/
│   │   └── conversation.json
│   ├── frontend/
│   ├── backend/
│   └── docs/
│
└── my-project-feature-auth/        # Workspace worktree (lazy created)
    ├── .amazonq/
    │   └── conversation.json       # Feature conversation
    ├── frontend/                   # Worktree (created when Q modifies frontend)
    ├── backend/                    # Worktree (created when Q modifies backend)
    └── docs/                       # Symlink or copy (not a git repo)
```

## Data Model

### WorkspaceContext

```rust
pub struct WorkspaceContext {
    /// Workspace root (current directory)
    pub root: PathBuf,
    
    /// Git repositories found in workspace
    pub repos: Vec<RepoInfo>,
    
    /// Conversation scope
    pub scope: ConversationScope,
    
    /// Session name (if in a worktree session)
    pub session_name: Option<String>,
}

pub struct RepoInfo {
    /// Path to repository
    pub path: PathBuf,
    
    /// Repository name
    pub name: String,
    
    /// Base branch (branch at session start)
    pub base_branch: String,
    
    /// Created worktree path (if any)
    pub worktree: Option<PathBuf>,
    
    /// Whether this repo has uncommitted changes
    pub has_changes: bool,
}

pub enum ConversationScope {
    /// Conversation covers entire workspace (default)
    Workspace,
    
    /// Conversation limited to specific repo
    Repo(PathBuf),
}
```

### Session Metadata Extension

Extend existing SessionMetadata from Session Management V2:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    // ... existing fields from V2 ...
    
    /// Workspace-specific information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_info: Option<WorkspaceInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceInfo {
    /// Workspace root directory
    pub root: PathBuf,
    
    /// Repositories with worktrees
    pub repos_with_worktrees: Vec<RepoWorktreeInfo>,
    
    /// Conversation scope
    pub scope: ConversationScope,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoWorktreeInfo {
    pub repo_path: PathBuf,
    pub repo_name: String,
    pub base_branch: String,
    pub worktree_path: PathBuf,
    pub created_at: DateTime<Utc>,
}
```

## Lazy Worktree Creation

### When Worktrees Are Created

Worktrees are created **only when Q needs to modify a file** in a git repository:

```rust
// Q is about to call fs_write
if in_worktree_session() && file_in_git_repo(file_path) {
    let worktree_path = ensure_worktree_for_file(file_path)?;
    // Modify file in worktree instead
}
```

### Creation Flow

```
1. Q plans to modify file: frontend/src/auth.rs
   ↓
2. Check: Is this a worktree session?
   ↓
3. Check: Is file in a git repo?
   ↓
4. Check: Does worktree exist for this repo?
   ↓
5. If not: Create worktree for frontend/ repo
   ↓
6. Modify file in worktree: my-project-feature-auth/frontend/src/auth.rs
```

### Implementation

```rust
pub fn ensure_worktree_for_file(
    file: &Path,
    session_name: &str,
    workspace: &mut WorkspaceContext
) -> Result<PathBuf> {
    // Find which repo contains this file
    let repo = workspace.repos.iter_mut()
        .find(|r| file.starts_with(&r.path))
        .ok_or(GitError::NotInRepo)?;
    
    // Create worktree if not exists
    if repo.worktree.is_none() {
        let worktree_root = workspace.root
            .parent()
            .unwrap()
            .join(format!("{}-{}", 
                workspace.root.file_name().unwrap().to_str().unwrap(),
                session_name
            ));
        
        let worktree_path = worktree_root.join(&repo.name);
        
        create_worktree(
            &repo.path,
            session_name,
            &repo.base_branch,
            Some(worktree_path.clone())
        )?;
        
        repo.worktree = Some(worktree_path);
    }
    
    // Return path in worktree
    let relative = file.strip_prefix(&repo.path)?;
    Ok(repo.worktree.as_ref().unwrap().join(relative))
}
```

## Base Branch Detection

The branch each repo is on when the session starts becomes the merge target:

```rust
pub fn detect_base_branches(workspace_root: &Path) -> Result<Vec<RepoInfo>> {
    let mut repos = Vec::new();
    
    for entry in fs::read_dir(workspace_root)? {
        let path = entry?.path();
        
        if path.join(".git").exists() {
            let branch = get_current_branch(&path)?;
            
            repos.push(RepoInfo {
                path: path.clone(),
                name: path.file_name().unwrap().to_str().unwrap().to_string(),
                base_branch: branch,
                worktree: None,
                has_changes: false,
            });
        }
    }
    
    Ok(repos)
}
```

## Conversation Scope

### Workspace-Level (Default)

Conversation stored at workspace root, covers all repos:

```
/workspace/my-project/.amazonq/conversation.json
```

Used for:
- Interactive chat sessions
- Features spanning multiple repos
- General development work

### Repo-Level (Optional)

Conversation stored in specific repo, limited to that repo:

```
/workspace/my-project/frontend/.amazonq/conversation.json
```

Used for:
- Agents focused on one repo
- Skills that only touch one repo
- Isolated development

### Configuration

**In Agent Config**:
```json
{
  "name": "frontend-agent",
  "conversationScope": "repo",
  "repoPath": "frontend/",
  "tools": ["fs_read", "fs_write"]
}
```

**In Skill Metadata**:
```json
{
  "name": "deploy-backend",
  "conversationScope": "repo",
  "repoPath": "backend/",
  "requiresWorktree": true
}
```

**At Runtime**:
```bash
q chat --scope repo --repo frontend/
```

## Build System Integration

### Separate Target Directories

Each worktree gets its own build directory:

```rust
pub fn configure_worktree_build(worktree_path: &Path) -> Result<()> {
    let cargo_config = worktree_path.join(".cargo");
    fs::create_dir_all(&cargo_config)?;
    
    fs::write(
        cargo_config.join("config.toml"),
        "[build]\ntarget-dir = \"target\"\n"
    )?;
    
    // Add to .gitignore
    let gitignore = worktree_path.join(".gitignore");
    let mut content = if gitignore.exists() {
        fs::read_to_string(&gitignore)?
    } else {
        String::new()
    };
    
    if !content.contains(".cargo/config.toml") {
        content.push_str("\n.cargo/config.toml\n");
        fs::write(gitignore, content)?;
    }
    
    Ok(())
}
```

### Non-Git Files

For files not in git repos (like `docs/`):
- **Option 1**: Symlink to original
- **Option 2**: Copy to worktree
- **Option 3**: Access from original location

Recommendation: **Symlink** for simplicity.

## Merge Workflow

### Merge All Repos

```rust
pub fn merge_session(
    workspace: &WorkspaceContext,
    session_name: &str
) -> Result<MergeResult> {
    let mut results = Vec::new();
    
    for repo in &workspace.repos {
        if let Some(worktree) = &repo.worktree {
            let result = merge_repo_worktree(
                worktree,
                &repo.path,
                &repo.base_branch
            )?;
            results.push(result);
        }
    }
    
    Ok(MergeResult { repos: results })
}

fn merge_repo_worktree(
    worktree: &Path,
    repo: &Path,
    base_branch: &str
) -> Result<RepoMergeResult> {
    // 1. Commit changes in worktree
    commit_worktree_changes(worktree)?;
    
    // 2. Switch to base branch in main repo
    checkout_branch(repo, base_branch)?;
    
    // 3. Merge worktree branch
    let branch_name = get_current_branch(worktree)?;
    merge_branch(repo, &branch_name)?;
    
    // 4. Remove worktree
    remove_worktree(worktree)?;
    
    Ok(RepoMergeResult {
        repo_name: repo.file_name().unwrap().to_str().unwrap().to_string(),
        base_branch: base_branch.to_string(),
        success: true,
        conflicts: vec![],
    })
}
```

## Session Types

Same as before, but with workspace awareness:

| Type | Worktree Behavior | Scope |
|------|-------------------|-------|
| **Exploration** | Never create | Workspace |
| **Feature** | Create on demand | Workspace |
| **Hotfix** | Ask user | Workspace |
| **Refactor** | Create on demand | Workspace |
| **Experiment** | Create on demand | Workspace |
| **Agent** | Never (background) | Configurable |
| **Delegate** | Create on demand | Configurable |

## Decision Logic (Updated)

### Layer 1: Explicit Flags
```bash
q chat --worktree feature-name --scope workspace
```

### Layer 2: Skill/Agent Config
```json
{
  "requiresWorktree": true,
  "conversationScope": "repo",
  "repoPath": "frontend/"
}
```

### Layer 3: Workspace State
- In main workspace → May need worktree
- Already in worktree → Use existing
- Not in git repo → No worktrees

### Layer 4: Ask User
```
Q: "This will modify files in frontend/ and backend/.
    Create worktrees to isolate changes? [y/n]"
```

## File Path Translation

When in a worktree session, translate paths:

```rust
pub fn translate_path(
    path: &Path,
    workspace: &WorkspaceContext
) -> PathBuf {
    // If in worktree session
    if let Some(session_name) = &workspace.session_name {
        // Find which repo this file belongs to
        for repo in &workspace.repos {
            if path.starts_with(&repo.path) {
                if let Some(worktree) = &repo.worktree {
                    // Translate to worktree path
                    let relative = path.strip_prefix(&repo.path).unwrap();
                    return worktree.join(relative);
                }
            }
        }
    }
    
    // No translation needed
    path.to_path_buf()
}
```

## Folder Structure

### Q CLI Conventions

```
~/.aws/amazonq/                     # Global Q data
├── cli-agents/
├── cli-checkouts/                  # Checkpoint shadow repos
├── knowledge_bases/
└── database.db

/workspace/my-project/              # Workspace
└── .amazonq/                       # Workspace-level Q data
    ├── conversation.json
    ├── sessions/
    ├── rules/
    └── prompts/
```

### With Worktrees

```
/workspace/
├── my-project/                     # Main workspace
│   ├── .amazonq/
│   │   └── conversation.json       # Main conversation
│   ├── frontend/
│   │   ├── .git/
│   │   ├── .cargo/                 # Build config
│   │   │   └── config.toml
│   │   └── target/                 # Build artifacts
│   └── backend/
│       ├── .git/
│       └── target/
│
└── my-project-feature-auth/        # Worktree workspace
    ├── .amazonq/
    │   └── conversation.json       # Feature conversation
    ├── frontend/                   # Worktree (lazy created)
    │   ├── .cargo/
    │   │   └── config.toml         # Separate build
    │   └── target/                 # Separate artifacts
    └── backend/                    # Worktree (lazy created)
        ├── .cargo/
        └── target/
```

## Edge Cases

### 1. Nested Git Repos (Submodules)
- Detect submodules separately
- Create worktrees for submodules if needed
- Track parent-child relationships

### 2. Repos on Different Branches
```
frontend/ on main
backend/ on develop
shared/ on v2
```
- Each merges back to its own base branch
- No conflicts between repos

### 3. Non-Git Files in Workspace
```
docs/ (not a git repo)
```
- Symlink to worktree workspace
- Or access from original location

### 4. Partial Worktree Creation
```
# Q only modifies frontend/, not backend/
my-project-feature-auth/
├── frontend/     # Worktree created
└── backend/      # Symlink to original
```

### 5. Worktree Name Conflicts
```
my-project-feature-auth/  # Already exists
```
- Append UUID: `my-project-feature-auth-a3f9/`
- Or ask user for different name

## Success Criteria

1. **Multi-Repo Support**: Can work across multiple git repos in workspace
2. **Lazy Creation**: Only creates worktrees when needed
3. **Base Branch Tracking**: Merges back to correct branch per repo
4. **Scope Flexibility**: Supports both workspace and repo-level conversations
5. **Build Isolation**: Each worktree has separate build artifacts
6. **Clean Merge**: All repos merge cleanly back to their base branches

## Implementation Phases (Updated)

### Phase 1: Git Detection (Complete)
- ✅ Basic git operations
- Add: Multi-repo detection
- Add: Base branch tracking

### Phase 2: Workspace Detection (New)
- Detect all git repos in current directory
- Track base branch for each repo
- Determine conversation scope

### Phase 3: Lazy Worktree Creation
- Create worktrees on-demand
- Path translation
- Build configuration

### Phase 4: Conversation Storage
- Workspace-level storage
- Repo-level storage (optional)
- Scope configuration

### Phase 5: Session Management
- Enhanced /sessions with workspace info
- Show which repos have worktrees
- Display base branches

### Phase 6: Merge Workflow
- Merge all repos
- Handle per-repo conflicts
- Clean up worktrees

## Open Questions

1. ~~Should we support nested git repos (submodules)?~~ → Yes, detect and handle
2. ~~What if repos are on different branches?~~ → Each merges to its own base
3. ~~How to handle non-git files?~~ → Symlink to worktree
4. Should we limit number of repos in workspace? → No limit, but warn if >10
5. Should we support monorepos differently? → No, treat as single repo
