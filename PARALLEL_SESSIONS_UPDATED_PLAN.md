# Parallel Sessions with Worktrees - Updated Implementation Plan

**Development Branch**: `main`  
**Status**: Phase 3 Complete ✅ (38/142 hours = 27%)

## Context & Integration

This plan integrates with existing Q CLI development efforts:

### Existing Systems to Integrate With

1. **Session Management V2** (In Progress)
   - Production-grade session metadata storage
   - Repository trait abstraction
   - File-based persistence with corruption recovery
   - **Integration Point**: Use SessionRepository for worktree session metadata

2. **Skills System** (Complete)
   - Skill trait and registry
   - Built-in skills (calculator, etc.)
   - @skill_name syntax in chat
   - **Integration Point**: Skills can declare `requiresWorktree` in metadata

3. **Creation System** (Complete)
   - Unified creation flows for skills/commands/agents
   - Terminal-native UX
   - Context intelligence
   - **Integration Point**: Creation flows can trigger worktree creation

4. **Multi-Session Design** (Planned)
   - TUI for multiple concurrent sessions
   - Session switching and indicators
   - **Integration Point**: Worktree sessions appear in TUI

5. **Development Sessions** (Planned)
   - Isolated dev environments for skills/commands/agents
   - Testing frameworks
   - **Integration Point**: Dev sessions can use worktrees for isolation

---

## Revised Architecture

### Layered Integration

```
┌─────────────────────────────────────────────────────┐
│              CLI Commands & Chat                     │
│         (q chat, /sessions, @skills)                 │
└──────────────────────┬──────────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────────┐
│         WorktreeSessionManager (NEW)                 │
│  - Worktree-aware session operations                │
│  - Integrates with SessionManager V2                │
└──────────────────────┬──────────────────────────────┘
                       │
        ┌──────────────┴──────────────┐
        │                             │
┌───────▼────────┐          ┌─────────▼────────┐
│ SessionManager │          │  GitWorktree     │
│ (V2 - Exists)  │          │  (NEW)           │
└────────────────┘          └──────────────────┘
```

### Data Model Extension

Extend existing `SessionMetadata` from Session Management V2:

```rust
// Extend existing SessionMetadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    // ... existing fields from V2 ...
    
    // NEW: Worktree-specific fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worktree_info: Option<WorktreeInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeInfo {
    pub path: PathBuf,
    pub branch: String,
    pub repo_root: PathBuf,
    pub is_temporary: bool,
    pub merge_target: String, // Usually "main"
}
```

---

## Updated Phase Breakdown

### Phase 1: Git Detection & Worktree Management (Week 1)
**Status**: Build from scratch
**Effort**: 22 hours

**No changes** - This is foundational infrastructure

**Deliverables**:
- Git context detection
- Worktree creation/removal
- Worktree listing
- Error handling

---

### Phase 2: Conversation Storage Integration (Week 2)
**Status**: Integrate with Session Management V2
**Effort**: Reduced from 16 to **10 hours**

**Changes**:
- Use existing `SessionRepository` trait instead of building new storage
- Extend `SessionMetadata` with `WorktreeInfo`
- Use existing file-based persistence
- Leverage existing corruption recovery

**Tasks**:

#### Task 2.1: Extend SessionMetadata ✅ COMPLETE (2 hours)
**Completed**:
- ✅ Added `WorktreeInfo` struct to `metadata.rs`
- ✅ Extended `SessionMetadata` with `worktree_info: Option<WorktreeInfo>`
- ✅ Implemented `with_worktree()`, `is_worktree_session()`, `worktree_path()` methods
- ✅ Added 4 integration tests (all passing)
- ✅ Proper serialization with `skip_serializing_if` for None values

**Files Modified**:
- `crates/chat-cli/src/session/metadata.rs`
- `crates/chat-cli/src/session/mod.rs`

**Files Created**:
- `crates/chat-cli/tests/session_worktree_tests.rs`

#### Task 2.2: Worktree-Aware Repository (4 hours)
**Status**: Next - Ready to start
```rust
// crates/chat-cli/src/session/worktree_repo.rs (NEW)
pub struct WorktreeSessionRepository {
    inner: Box<dyn SessionRepository>,
    git_context: GitContext,
}

impl WorktreeSessionRepository {
    pub async fn save_in_worktree(
        &self,
        metadata: &SessionMetadata,
        worktree_path: &Path
    ) -> Result<()> {
        // Save to .q/session.json in worktree
        // Also register in central repo for discovery
    }
    
    pub async fn discover_worktree_sessions(
        &self,
        repo_root: &Path
    ) -> Result<Vec<SessionMetadata>> {
        // Scan worktrees, load metadata
    }
}
```

#### Task 2.3: Conversation Key Resolver (2 hours)
```rust
// Use existing session ID format, extend with git context
pub fn resolve_session_id(
    path: &Path,
    override_id: Option<&str>
) -> String {
    if let Some(id) = override_id {
        return id.to_string();
    }
    
    if let Ok(git) = detect_git_context(path) {
        return format!("{}/{}", git.repo_name, git.branch_name);
    }
    
    // Fallback to existing path-based ID
    path.to_string_lossy().to_string()
}
```

#### Task 2.4: Integration Tests (2 hours)
- Test with existing SessionRepository implementations
- Test worktree-specific storage
- Test discovery

---

### Phase 3: Decision Logic & Session Types (Week 3)
**Status**: Extend existing SessionType
**Effort**: Reduced from 18 to **12 hours**

**Changes**:
- Extend existing `SessionType` enum (Debug, Planning, Development, CodeReview)
- Add new types: Feature, Hotfix, Refactor, Experiment
- Integrate with existing SessionManager
- Add CLI flags to existing ChatArgs

**Tasks**:

#### Task 3.1: Extend SessionType (2 hours)
```rust
// crates/chat-cli/src/theme/session.rs (modify existing)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SessionType {
    // Existing
    Debug,
    Planning,
    Development,
    CodeReview,
    
    // NEW: Worktree-specific
    Feature,
    Hotfix,
    Refactor,
    Experiment,
}

impl SessionType {
    pub fn requires_worktree(&self) -> bool {
        matches!(self, 
            SessionType::Feature | 
            SessionType::Refactor | 
            SessionType::Experiment
        )
    }
    
    pub fn is_interactive(&self) -> bool {
        !matches!(self, SessionType::Development) // Development can be background
    }
}
```

#### Task 3.2: Add CLI Flags (1 hour)
```rust
// crates/chat-cli/src/cli/chat/mod.rs (modify ChatArgs)
#[derive(Parser)]
pub struct ChatArgs {
    // ... existing args ...
    
    /// Create or use a worktree with specified name
    #[arg(long)]
    pub worktree: Option<String>,
    
    /// Disable worktree creation
    #[arg(long)]
    pub no_worktree: bool,
    
    /// Session type
    #[arg(long, value_enum)]
    pub session_type: Option<SessionType>,
}
```

#### Task 3.3: WorktreeStrategy Resolver (5 hours)
```rust
// crates/chat-cli/src/cli/chat/worktree_strategy.rs (NEW)
pub enum WorktreeStrategy {
    Create(String),
    CreateTemp,
    UseExisting,
    Never,
    Ask,
}

pub async fn resolve_worktree_strategy(
    args: &ChatArgs,
    skill: Option<&SkillMetadata>, // From existing skills system
    git_state: &GitContext,
) -> Result<WorktreeStrategy> {
    // Layer 1: Explicit flags
    if let Some(name) = &args.worktree {
        return Ok(WorktreeStrategy::Create(name.clone()));
    }
    
    if args.no_worktree {
        return Ok(WorktreeStrategy::Never);
    }
    
    // Layer 2: Session type
    if let Some(session_type) = args.session_type {
        if session_type.requires_worktree() {
            return Ok(WorktreeStrategy::Create(generate_name().await?));
        }
    }
    
    // Layer 3: Skill config
    if let Some(skill) = skill {
        if skill.requires_worktree() {
            return Ok(WorktreeStrategy::Create(generate_name().await?));
        }
    }
    
    // Layer 4: Git state
    if git_state.is_main_branch() {
        return Ok(WorktreeStrategy::Ask);
    }
    
    Ok(WorktreeStrategy::UseExisting)
}
```

#### Task 3.4: Skill Integration (2 hours)
```rust
// crates/chat-cli/src/cli/skills/mod.rs (modify existing)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMetadata {
    // ... existing fields ...
    
    /// Whether this skill requires a worktree
    #[serde(default)]
    pub requires_worktree: bool,
}

impl SkillMetadata {
    pub fn requires_worktree(&self) -> bool {
        self.requires_worktree
    }
}
```

#### Task 3.5: Tests (2 hours)

---

### Phase 4: Worktree Naming (Week 4)
**Status**: Build from scratch (no existing system)
**Effort**: 18 hours

**No changes** - This is new functionality

**Integration Points**:
- Use existing LLM client from chat system
- Follow existing naming patterns from creation system

---

### Phase 5: Session Discovery & Management (Week 5)
**Status**: Enhance existing /sessions command
**Effort**: Reduced from 19 to **14 hours**

**Changes**:
- Extend existing `/sessions` command
- Use existing SessionManager
- Add worktree scanning
- Integrate with Session Management V2

**Tasks**:

#### Task 5.1: Extend SessionsSubcommand (2 hours)
```rust
// crates/chat-cli/src/cli/chat/cli/sessions.rs (modify existing)
#[derive(Debug, PartialEq, Subcommand)]
pub enum SessionsSubcommand {
    // ... existing subcommands ...
    
    /// Scan for worktree-based sessions
    Scan,
    
    /// Show worktree sessions
    Worktrees,
}
```

#### Task 5.2: Worktree Scanner (6 hours)
```rust
// crates/chat-cli/src/cli/chat/session_scanner.rs (NEW)
pub async fn scan_worktree_sessions(
    repo_root: &Path,
    session_repo: &dyn SessionRepository
) -> Result<Vec<SessionMetadata>> {
    let worktrees = list_worktrees(repo_root)?;
    let mut sessions = Vec::new();
    
    for wt in worktrees {
        if let Ok(metadata) = load_session_from_worktree(&wt.path, session_repo).await {
            sessions.push(metadata);
        }
    }
    
    Ok(sessions)
}
```

#### Task 5.3: Enhanced List Display (3 hours)
```rust
// Modify existing list command to show worktree info
impl SessionsSubcommand {
    pub async fn execute(&self, session: &mut ChatSession, os: &Os) -> Result<ChatState> {
        match self {
            SessionsSubcommand::List => {
                // Use existing SessionManager
                let manager = &session.session_manager;
                let sessions = manager.list_sessions();
                
                // NEW: Also scan worktrees
                if let Ok(git) = detect_git_context(&current_dir()) {
                    let wt_sessions = scan_worktree_sessions(&git.repo_root, &session.repo).await?;
                    // Merge and display
                }
                
                // Display with existing formatting
            }
            // ... other commands ...
        }
    }
}
```

#### Task 5.4: Cleanup Command (3 hours)
```rust
// Add to existing SessionsSubcommand
SessionsSubcommand::Cleanup {
    #[arg(long)]
    merged: bool,
    
    #[arg(long)]
    orphaned: bool,
}
```

---

### Phase 6: Merge Workflow (Week 6)
**Status**: Build from scratch
**Effort**: 30 hours

**No changes** - This is new functionality

**Integration Points**:
- Use existing LLM client for conflict resolution
- Integrate with Session Management V2 for cleanup
- Follow existing error handling patterns

---

## Integration Tasks (Cross-Phase)

### Integration 1: Creation System (4 hours)
**When**: After Phase 3

```rust
// crates/chat-cli/src/cli/creation/mod.rs (modify)
impl CreationFlow {
    async fn execute(&mut self, args: &CreateArgs) -> Result<()> {
        // NEW: Check if worktree needed
        if self.should_use_worktree() {
            let strategy = resolve_worktree_strategy(
                &args.to_chat_args(),
                None,
                &detect_git_context(&current_dir())?
            ).await?;
            
            if let WorktreeStrategy::Create(name) = strategy {
                create_worktree_for_creation(&name).await?;
            }
        }
        
        // Continue with existing creation flow
    }
}
```

### Integration 2: Skills System (2 hours)
**When**: After Phase 3

```rust
// crates/chat-cli/src/cli/skills/registry.rs (modify)
impl SkillRegistry {
    pub async fn execute_skill(&self, name: &str, params: &Value) -> Result<SkillResult> {
        let skill = self.get(name)?;
        
        // NEW: Check if skill requires worktree
        if skill.metadata().requires_worktree {
            ensure_worktree_for_skill(skill).await?;
        }
        
        // Continue with existing execution
        skill.execute(params).await
    }
}
```

### Integration 3: Multi-Session TUI (6 hours)
**When**: After Phase 5

```rust
// Future integration with multi-session TUI
// Worktree sessions appear in TUI with special indicator
// Can switch between worktree sessions in TUI
```

### Integration 4: Development Sessions (4 hours)
**When**: After Phase 6

```rust
// Development sessions automatically use worktrees
// Testing happens in isolated worktree
// Merge back to main after testing passes
```

---

## Updated Timeline

| Phase | Original | Updated | Savings | Notes |
|-------|----------|---------|---------|-------|
| Phase 1 | 22h | 22h | 0h | No existing system |
| Phase 2 | 16h | 10h | 6h | Use Session Management V2 |
| Phase 3 | 18h | 12h | 6h | Extend existing SessionType |
| Phase 4 | 18h | 18h | 0h | No existing system |
| Phase 5 | 19h | 14h | 5h | Enhance existing /sessions |
| Phase 6 | 30h | 30h | 0h | No existing system |
| Integration | 15h | 16h | -1h | More integration points |
| **Total** | **138h** | **122h** | **16h** | ~2 weeks saved |

**With 30% buffer**: 159 hours / **20 days / 4 weeks**

---

## Dependencies & Coordination

### Must Complete Before Starting
- ✅ Session Management V2 Phase 0 (Foundation & Abstractions)
- ✅ Skills System (Complete)
- ✅ Creation System (Complete)

### Can Work in Parallel
- Multi-Session TUI (different code paths)
- Development Sessions (will integrate later)

### Coordination Points
1. **Week 2**: Coordinate with Session Management V2 team on metadata extension
2. **Week 3**: Coordinate with Skills team on `requiresWorktree` field
3. **Week 5**: Coordinate with Multi-Session TUI team on display integration

---

## Risk Mitigation

### Risk 1: Session Management V2 Changes
**Mitigation**: Use repository trait abstraction, minimal coupling

### Risk 2: Skills System Evolution
**Mitigation**: Use metadata extension, backward compatible

### Risk 3: Multi-Session TUI Conflicts
**Mitigation**: Clear interface boundaries, integration layer

### Risk 4: Creation System Changes
**Mitigation**: Hook into existing flow, optional feature

---

## Testing Strategy

### Unit Tests
- Each module independently
- Mock git operations
- Mock SessionRepository

### Integration Tests
- With Session Management V2
- With Skills System
- With Creation System
- Real git operations

### End-to-End Tests
- Complete workflows
- Multiple worktrees
- Merge scenarios

---

## Success Criteria

1. **Functional**:
   - Can create worktrees automatically
   - Sessions isolated per worktree
   - Merge workflow works
   - Integrates with existing systems

2. **Performance**:
   - Worktree creation <2s
   - Session discovery <500ms
   - No regression in existing features

3. **Quality**:
   - >80% test coverage
   - No data loss
   - Graceful error handling
   - Clear documentation

---

## Next Steps

1. ✅ Review this plan with team
2. ✅ Confirm Session Management V2 integration points
3. ✅ Confirm Skills System integration points
4. ⏳ Begin Phase 1 implementation
5. ⏳ Set up coordination meetings for Week 2, 3, 5

---

## Open Questions

1. Should worktree sessions appear in Multi-Session TUI immediately or wait for TUI completion?
2. Should Development Sessions always use worktrees or make it optional?
3. How to handle worktree sessions when Session Management V2 adds new features?
4. Should we add worktree support to existing sessions retroactively?
