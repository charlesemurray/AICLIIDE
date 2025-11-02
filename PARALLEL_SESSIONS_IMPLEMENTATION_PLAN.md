# Parallel Sessions Implementation Plan

## Overview

This document outlines the step-by-step implementation plan for adding parallel session support with git worktrees to Q CLI.

**Based on**: `PARALLEL_SESSIONS_FINAL_DESIGN.md`

**Estimated Timeline**: 4-6 weeks

**Dependencies**: Git must be available on system (optional, graceful fallback)

---

## Implementation Phases

### Phase 1: Git Detection & Worktree Management (Week 1)
**Goal**: Core infrastructure for detecting and managing git worktrees

**Deliverables**:
- Git context detection module
- Worktree creation/removal functions
- Worktree listing and validation
- Unit tests for git operations

**Success Criteria**:
- Can detect if directory is in git repo
- Can detect if directory is a worktree
- Can create/remove worktrees programmatically
- Gracefully handles git not installed

---

### Phase 2: Conversation Storage Refactoring (Week 2)
**Goal**: Move conversations from central DB to worktree-local storage

**Deliverables**:
- Conversation key resolver (repo/branch format)
- Save/load conversations from `.q/` directory
- Migration path for existing conversations
- Backward compatibility layer

**Success Criteria**:
- Conversations persist in worktree directory
- Can load conversations from both old and new locations
- Existing conversations still work
- Non-git projects use path-based keys

---

### Phase 3: Decision Logic & Session Types (Week 3)
**Goal**: Implement layered decision logic for when to create worktrees

**Deliverables**:
- WorktreeStrategy enum and resolver
- Session type detection
- CLI argument parsing (--worktree, --type, --no-worktree)
- Skill/agent config integration
- User prompts for confirmation

**Success Criteria**:
- Explicit flags override all other logic
- Skill configs control worktree behavior
- Git state detection works correctly
- User is prompted when appropriate

---

### Phase 4: Worktree Naming (Week 4)
**Goal**: Intelligent worktree name generation with LLM

**Deliverables**:
- LLM prompt for name generation
- Conflict detection and resolution
- Interactive name confirmation
- Non-interactive fallback (UUID)

**Success Criteria**:
- LLM generates sensible names from conversation
- Conflicts are detected and resolved
- User can accept or override suggested names
- Background tasks get automatic names

---

### Phase 5: Session Discovery & Management (Week 5)
**Goal**: Enhanced /sessions command with worktree awareness

**Deliverables**:
- Scan worktrees for active sessions
- Display session status and metadata
- Session switching helpers
- Session cleanup commands

**Success Criteria**:
- /sessions shows all worktree-based sessions
- Can see which sessions need attention
- Can identify current session
- Can clean up orphaned sessions

---

### Phase 6: Merge Workflow (Week 6)
**Goal**: Automated merge and cleanup workflow

**Deliverables**:
- Merge command implementation
- Conflict detection and resolution
- Automatic worktree cleanup
- Conversation archival/deletion

**Success Criteria**:
- Can merge feature branch to main
- Handles merge conflicts gracefully
- Cleans up worktree after merge
- Deletes conversation after merge

---

## Detailed Task Breakdown

### Phase 1: Git Detection & Worktree Management

#### Task 1.1: Create Git Module Structure
**Files to create**:
- `crates/chat-cli/src/git/mod.rs`
- `crates/chat-cli/src/git/context.rs`
- `crates/chat-cli/src/git/worktree.rs`
- `crates/chat-cli/src/git/error.rs`

**Estimated time**: 2 hours

#### Task 1.2: Implement Git Context Detection
**File**: `crates/chat-cli/src/git/context.rs`

**Functions to implement**:
```rust
pub struct GitContext {
    pub repo_root: PathBuf,
    pub repo_name: String,
    pub branch_name: String,
    pub is_worktree: bool,
    pub is_main_branch: bool,
    pub worktree_path: Option<PathBuf>,
}

pub fn detect_git_context(path: &Path) -> Result<GitContext, GitError>;
pub fn is_git_installed() -> bool;
pub fn get_repo_root(path: &Path) -> Result<PathBuf, GitError>;
pub fn get_current_branch(path: &Path) -> Result<String, GitError>;
pub fn is_worktree(path: &Path) -> Result<bool, GitError>;
pub fn is_main_branch(branch: &str) -> bool;
```

**Implementation approach**:
- Use `Command::new("git")` to shell out to git
- Parse git command output
- Handle errors gracefully (git not installed, not a repo, etc.)

**Tests to write**:
- Test in git repo
- Test in worktree
- Test in non-git directory
- Test with git not installed
- Test main vs feature branch detection

**Estimated time**: 6 hours

#### Task 1.3: Implement Worktree Management
**File**: `crates/chat-cli/src/git/worktree.rs`

**Functions to implement**:
```rust
pub struct WorktreeInfo {
    pub path: PathBuf,
    pub branch: String,
    pub commit: String,
}

pub fn list_worktrees(repo_root: &Path) -> Result<Vec<WorktreeInfo>, GitError>;
pub fn create_worktree(
    repo_root: &Path,
    name: &str,
    base_branch: &str,
    path: Option<PathBuf>
) -> Result<PathBuf, GitError>;
pub fn remove_worktree(path: &Path) -> Result<(), GitError>;
pub fn worktree_exists(repo_root: &Path, name: &str) -> bool;
pub fn branch_exists(repo_root: &Path, name: &str) -> bool;
```

**Implementation approach**:
- `git worktree list --porcelain` for listing
- `git worktree add <path> -b <branch>` for creation
- `git worktree remove <path>` for removal
- Parse output, handle errors

**Tests to write**:
- Create worktree
- List worktrees
- Remove worktree
- Detect conflicts
- Handle invalid paths

**Estimated time**: 8 hours

#### Task 1.4: Error Handling
**File**: `crates/chat-cli/src/git/error.rs`

**Error types to define**:
```rust
pub enum GitError {
    NotInstalled,
    NotARepository,
    WorktreeExists(String),
    BranchExists(String),
    CommandFailed(String),
    ParseError(String),
    IoError(std::io::Error),
}
```

**Estimated time**: 2 hours

#### Task 1.5: Integration Tests
**File**: `crates/chat-cli/tests/git_integration_tests.rs`

**Test scenarios**:
- End-to-end worktree creation and removal
- Multiple worktrees in same repo
- Conflict detection
- Graceful degradation without git

**Estimated time**: 4 hours

**Phase 1 Total**: ~22 hours (3 days)

---

### Phase 2: Conversation Storage Refactoring

#### Task 2.1: Conversation Key Resolver
**File**: `crates/chat-cli/src/cli/chat/conversation_key.rs`

**Functions to implement**:
```rust
pub fn resolve_conversation_key(
    path: &Path,
    override_context: Option<&str>
) -> String;

pub fn get_conversation_storage_path(
    path: &Path,
    key: &str
) -> PathBuf;

pub fn is_worktree_storage(path: &Path) -> bool;
```

**Implementation approach**:
- Check for manual override first
- Try git context detection
- Fallback to absolute path
- Return format: `<repo>/<branch>` or path

**Tests to write**:
- Git repo with worktree
- Git repo without worktree
- Non-git directory
- Manual override

**Estimated time**: 4 hours

#### Task 2.2: Worktree-Local Storage
**File**: `crates/chat-cli/src/cli/chat/conversation.rs` (modify existing)

**Functions to add/modify**:
```rust
impl ConversationState {
    pub async fn save_to_worktree(&self, worktree_path: &Path) -> Result<()>;
    pub async fn load_from_worktree(worktree_path: &Path) -> Result<Self>;
    pub async fn save(&self, os: &Os) -> Result<()>; // Modify to use new logic
    pub async fn load(os: &Os, path: &Path) -> Result<Option<Self>>; // Modify
}
```

**Implementation approach**:
- Create `.q/` directory in worktree
- Save conversation as JSON
- Load from `.q/conversation.json`
- Fallback to central DB if not found

**Tests to write**:
- Save to worktree
- Load from worktree
- Fallback to central DB
- Handle missing .q directory

**Estimated time**: 6 hours

#### Task 2.3: Database Migration
**File**: `crates/chat-cli/src/database/migrations/008_worktree_conversations.sql`

**Migration tasks**:
- Add metadata column for storage location
- Mark existing conversations as "central"
- Add index for faster lookups

**Estimated time**: 2 hours

#### Task 2.4: Backward Compatibility Layer
**File**: `crates/chat-cli/src/cli/chat/conversation_compat.rs`

**Functions to implement**:
```rust
pub async fn load_conversation_any_location(
    os: &Os,
    path: &Path
) -> Result<Option<ConversationState>>;

pub async fn migrate_conversation_to_worktree(
    os: &Os,
    old_key: &str,
    worktree_path: &Path
) -> Result<()>;
```

**Implementation approach**:
- Try worktree location first
- Fallback to central DB
- Optional migration command

**Estimated time**: 4 hours

**Phase 2 Total**: ~16 hours (2 days)

---

### Phase 3: Decision Logic & Session Types

#### Task 3.1: WorktreeStrategy Enum
**File**: `crates/chat-cli/src/cli/chat/worktree_strategy.rs`

**Types to define**:
```rust
pub enum WorktreeStrategy {
    Create(String),
    CreateTemp,
    UseExisting,
    Never,
    Ask,
}

pub enum SessionType {
    Exploration,
    Feature,
    Hotfix,
    Refactor,
    Experiment,
    Agent,
    Delegate,
}

impl SessionType {
    pub fn default_worktree_behavior(&self) -> WorktreeStrategy;
    pub fn is_interactive(&self) -> bool;
}
```

**Estimated time**: 2 hours

#### Task 3.2: CLI Arguments
**File**: `crates/chat-cli/src/cli/chat/mod.rs` (modify ChatArgs)

**Arguments to add**:
```rust
#[derive(Parser)]
pub struct ChatArgs {
    // ... existing args
    
    #[arg(long)]
    pub worktree: Option<String>,
    
    #[arg(long)]
    pub no_worktree: bool,
    
    #[arg(long, value_enum)]
    pub session_type: Option<SessionType>,
}
```

**Estimated time**: 1 hour

#### Task 3.3: Layer 1 - Explicit Flags
**File**: `crates/chat-cli/src/cli/chat/worktree_resolver.rs`

**Function to implement**:
```rust
pub fn check_explicit_flags(args: &ChatArgs) -> Option<WorktreeStrategy>;
```

**Implementation approach**:
- Check `--worktree` flag (validate name provided)
- Check `--no-worktree` flag
- Check `--session-type` flag
- Return strategy or None

**Tests to write**:
- Each flag combination
- Validation errors

**Estimated time**: 3 hours

#### Task 3.4: Layer 2 - Skill/Agent Config
**File**: `crates/chat-cli/src/cli/chat/worktree_resolver.rs`

**Function to implement**:
```rust
pub fn check_skill_agent_config(
    skill: Option<&SkillConfig>,
    agent: Option<&AgentConfig>
) -> Option<WorktreeStrategy>;
```

**Implementation approach**:
- Read `requiresWorktree` from skill config
- Read `type` from agent config
- Return strategy based on config

**Skill/Agent config changes**:
- Add `requiresWorktree` field to skill schema
- Add to agent schema

**Estimated time**: 4 hours

#### Task 3.5: Layer 3 - Git State
**File**: `crates/chat-cli/src/cli/chat/worktree_resolver.rs`

**Function to implement**:
```rust
pub fn check_git_state(path: &Path) -> WorktreeStrategy;
```

**Implementation approach**:
- Use git detection from Phase 1
- Return strategy based on current state

**Estimated time**: 2 hours

#### Task 3.6: Layer 4 - Ask User
**File**: `crates/chat-cli/src/cli/chat/worktree_resolver.rs`

**Function to implement**:
```rust
pub async fn ask_user_for_worktree() -> Result<WorktreeStrategy>;
```

**Implementation approach**:
- Display prompt with options
- Read user input
- Return strategy

**Estimated time**: 2 hours

#### Task 3.7: Main Resolver
**File**: `crates/chat-cli/src/cli/chat/worktree_resolver.rs`

**Function to implement**:
```rust
pub async fn resolve_worktree_strategy(
    args: &ChatArgs,
    skill: Option<&SkillConfig>,
    agent: Option<&AgentConfig>,
    path: &Path,
) -> Result<WorktreeStrategy>;
```

**Implementation approach**:
- Call each layer in order
- Return first non-None result
- Handle Ask strategy

**Tests to write**:
- Each layer independently
- Layer priority
- Full integration

**Estimated time**: 4 hours

**Phase 3 Total**: ~18 hours (2-3 days)

---

## Risk Assessment

### High Risk
1. **Git command parsing** - Different git versions may have different output formats
   - Mitigation: Test with multiple git versions, use porcelain formats

2. **Conversation data loss** - Moving from central DB to worktree storage
   - Mitigation: Keep backward compatibility, thorough testing, migration tool

3. **Merge conflicts** - Complex conflict resolution logic
   - Mitigation: Start with manual resolution, add automation later

### Medium Risk
1. **LLM name generation** - May produce invalid or inappropriate names
   - Mitigation: Validation layer, user confirmation, fallback to UUID

2. **Performance** - Git operations may be slow on large repos
   - Mitigation: Cache git context, async operations, progress indicators

3. **Cross-platform** - Git behavior differs on Windows
   - Mitigation: Test on all platforms, use git porcelain commands

### Low Risk
1. **Session discovery** - Scanning many worktrees may be slow
   - Mitigation: Cache results, lazy loading, background scanning

2. **Cleanup** - Orphaned worktrees if Q crashes
   - Mitigation: Add cleanup command, detect orphans on startup

---

## Testing Strategy

### Unit Tests
- Each module independently
- Mock git commands
- Edge cases and error conditions

### Integration Tests
- End-to-end workflows
- Real git operations
- Multiple worktrees

### Manual Testing
- Different git configurations
- Various project sizes
- Error scenarios

### Performance Tests
- Large repos (1000+ files)
- Many worktrees (10+)
- Concurrent operations

---

## Rollout Plan

### Phase 1: Internal Testing
- Enable for Q CLI developers
- Test on Q CLI codebase itself
- Gather feedback

### Phase 2: Opt-In Beta
- Add `--enable-worktrees` flag
- Document in experimental features
- Monitor for issues

### Phase 3: Opt-Out
- Enable by default
- Add `--disable-worktrees` flag
- Provide migration guide

### Phase 4: Full Release
- Remove flags
- Update documentation
- Announce feature

---

## Success Metrics

1. **Adoption**: 30% of users create worktrees within first month
2. **Reliability**: <1% error rate in worktree operations
3. **Performance**: Worktree creation <2 seconds
4. **Satisfaction**: Positive feedback from beta testers
5. **Stability**: No data loss incidents

---

## Dependencies

### External
- Git 2.15+ (for worktree support)
- Rust 1.70+ (for async/await)

### Internal
- Existing conversation system
- Session manager
- Database layer
- CLI argument parsing

---

## Documentation Needs

1. User guide for parallel sessions
2. Git worktree primer
3. Troubleshooting guide
4. Migration guide for existing users
5. API documentation for developers

---

## Next Steps

1. Review this plan with team
2. Prioritize phases based on user needs
3. Set up development environment
4. Begin Phase 1 implementation
5. Create tracking issues for each task
