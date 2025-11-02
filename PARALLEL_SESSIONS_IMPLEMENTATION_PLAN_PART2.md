# Parallel Sessions Implementation Plan - Part 2

## Continuation of Phase Breakdown

### Phase 4: Worktree Naming

#### Task 4.1: LLM Name Generation Prompt
**File**: `crates/chat-cli/src/cli/chat/worktree_naming.rs`

**Function to implement**:
```rust
pub async fn llm_suggest_worktree_name(
    conversation: &Conversation,
    session_type: SessionType,
) -> Result<String>;
```

**Prompt template**:
```
Based on this user request: "{user_message}"
And session type: {session_type}

Suggest a concise git branch/worktree name following these rules:
- Format: <type>/<description>
- Type: feature, bugfix, refactor, experiment
- Description: 2-4 words, kebab-case, descriptive
- Examples: "feature/readonly-tool", "bugfix/parser-leak"

If the request is too vague or generic, respond with exactly: "GENERIC"

Suggested name:
```

**Implementation approach**:
- Use existing LLM client
- Parse response
- Validate format
- Handle "GENERIC" response

**Tests to write**:
- Clear feature request → good name
- Vague request → "GENERIC"
- Various session types
- Edge cases (very long names, special chars)

**Estimated time**: 4 hours

#### Task 4.2: Name Validation
**File**: `crates/chat-cli/src/cli/chat/worktree_naming.rs`

**Functions to implement**:
```rust
pub fn validate_worktree_name(name: &str) -> Result<(), ValidationError>;
pub fn sanitize_worktree_name(name: &str) -> String;
pub fn is_valid_branch_name(name: &str) -> bool;
```

**Validation rules**:
- No spaces
- No special chars except `-`, `/`, `_`
- Not empty
- Max length 100 chars
- Valid git branch name

**Estimated time**: 2 hours

#### Task 4.3: Conflict Detection
**File**: `crates/chat-cli/src/cli/chat/worktree_naming.rs`

**Functions to implement**:
```rust
pub fn check_name_conflicts(
    repo_root: &Path,
    name: &str
) -> Result<Vec<ConflictType>>;

pub enum ConflictType {
    WorktreeExists,
    BranchExists,
    DirectoryExists,
}
```

**Implementation approach**:
- Check git worktree list
- Check git branch list
- Check filesystem

**Estimated time**: 3 hours

#### Task 4.4: Interactive Name Confirmation
**File**: `crates/chat-cli/src/cli/chat/worktree_naming.rs`

**Function to implement**:
```rust
pub async fn confirm_or_change_name(
    suggested: &str,
    repo_root: &Path
) -> Result<String>;
```

**UI flow**:
```
Suggested worktree name: 'feature/readonly-tool'
Press Enter to accept, or type a new name: _
```

**Implementation approach**:
- Display suggestion
- Read user input
- If empty, use suggestion
- If provided, validate and check conflicts
- Retry on conflict

**Estimated time**: 3 hours

#### Task 4.5: Non-Interactive Fallback
**File**: `crates/chat-cli/src/cli/chat/worktree_naming.rs`

**Function to implement**:
```rust
pub fn generate_fallback_name(
    session_type: SessionType,
    base_name: Option<&str>
) -> String;

fn short_uuid() -> String;
```

**Implementation approach**:
- Use base name if provided
- Append short UUID (8 chars)
- Format: `task/<base>-<uuid>` or `task/<uuid>`

**Estimated time**: 2 hours

#### Task 4.6: Main Name Generator
**File**: `crates/chat-cli/src/cli/chat/worktree_naming.rs`

**Function to implement**:
```rust
pub async fn generate_worktree_name(
    conversation: &Conversation,
    session_type: SessionType,
    is_interactive: bool,
    repo_root: &Path,
) -> Result<String>;
```

**Implementation approach**:
- Call LLM for suggestion
- Handle "GENERIC" response
- Check conflicts
- Confirm with user (if interactive)
- Use fallback (if non-interactive)

**Tests to write**:
- Interactive flow
- Non-interactive flow
- Conflict resolution
- Generic name handling
- Validation errors

**Estimated time**: 4 hours

**Phase 4 Total**: ~18 hours (2-3 days)

---

### Phase 5: Session Discovery & Management

#### Task 5.1: Session Scanner
**File**: `crates/chat-cli/src/cli/chat/session_scanner.rs`

**Functions to implement**:
```rust
pub struct SessionInfo {
    pub name: String,
    pub path: PathBuf,
    pub branch: String,
    pub last_activity: SystemTime,
    pub status: SessionStatus,
    pub files_modified: usize,
}

pub enum SessionStatus {
    Active,
    Idle,
    WaitingForApproval,
    Error(String),
}

pub async fn scan_worktree_sessions(
    repo_root: &Path
) -> Result<Vec<SessionInfo>>;

pub async fn get_session_status(
    worktree_path: &Path
) -> Result<SessionStatus>;
```

**Implementation approach**:
- List all worktrees
- Check for `.q/conversation.json` in each
- Load conversation metadata
- Determine status from conversation state
- Count modified files (git status)

**Estimated time**: 6 hours

#### Task 5.2: Enhanced /sessions Command
**File**: `crates/chat-cli/src/cli/chat/cli/sessions.rs` (modify existing)

**Functions to add**:
```rust
impl SessionsSubcommand {
    async fn list_with_worktrees(&self, os: &Os) -> Result<ChatState>;
    async fn show_session_details(&self, name: &str, os: &Os) -> Result<ChatState>;
}
```

**Output format**:
```
📋 Active Sessions:

  • q-cli/main              /workspace/q-cli/main
    Status: Idle
    Last activity: 2 hours ago
    
  • q-cli/feature/readonly-tool    /workspace/q-cli-feature-readonly-tool  ← current
    Status: Working
    Last activity: 10 seconds ago
    Files modified: 3
    
  • q-cli/bugfix/parser-leak       /workspace/q-cli-bugfix-parser
    Status: Waiting for approval
    Last activity: 2 minutes ago
    Files modified: 1
```

**Estimated time**: 4 hours

#### Task 5.3: Session Cleanup Command
**File**: `crates/chat-cli/src/cli/chat/cli/sessions.rs`

**New subcommand**:
```rust
SessionsSubcommand::Cleanup {
    /// Remove merged sessions
    #[arg(long)]
    merged: bool,
    
    /// Remove orphaned sessions (worktree deleted)
    #[arg(long)]
    orphaned: bool,
    
    /// Remove all inactive sessions
    #[arg(long)]
    all: bool,
}
```

**Implementation approach**:
- Scan for sessions
- Check if worktree still exists
- Check if branch is merged
- Prompt for confirmation
- Remove worktrees and conversations

**Estimated time**: 4 hours

#### Task 5.4: Session Switching Helper
**File**: `crates/chat-cli/src/cli/chat/cli/sessions.rs`

**New subcommand**:
```rust
SessionsSubcommand::Switch {
    /// Session name to switch to
    name: String,
}
```

**Implementation approach**:
- Find session by name
- Print `cd` command for user
- Or integrate with shell (advanced)

**Output**:
```
To switch to session 'feature/readonly-tool':
  cd /workspace/q-cli-feature-readonly-tool
```

**Estimated time**: 2 hours

#### Task 5.5: Session Status Indicators
**File**: `crates/chat-cli/src/cli/chat/session_status.rs`

**Functions to implement**:
```rust
pub fn detect_session_status(
    conversation: &ConversationState
) -> SessionStatus;

pub fn format_status_indicator(status: &SessionStatus) -> String;
```

**Status detection logic**:
- Active: Recent activity (<5 min)
- Idle: No recent activity
- Waiting: Tool use pending approval
- Error: Last operation failed

**Estimated time**: 3 hours

**Phase 5 Total**: ~19 hours (2-3 days)

---

### Phase 6: Merge Workflow

#### Task 6.1: Merge Command
**File**: `crates/chat-cli/src/cli/chat/cli/merge.rs`

**New command**:
```rust
#[derive(Debug, PartialEq, Subcommand)]
pub enum MergeSubcommand {
    /// Merge current worktree to main
    ToMain {
        /// Skip confirmation
        #[arg(long)]
        yes: bool,
    },
    
    /// Merge specific session
    Session {
        /// Session name
        name: String,
        
        /// Skip confirmation
        #[arg(long)]
        yes: bool,
    },
    
    /// Show merge preview
    Preview {
        /// Session name (defaults to current)
        name: Option<String>,
    },
}
```

**Estimated time**: 2 hours

#### Task 6.2: Pre-Merge Validation
**File**: `crates/chat-cli/src/cli/chat/merge/validation.rs`

**Functions to implement**:
```rust
pub struct MergeValidation {
    pub uncommitted_changes: Vec<PathBuf>,
    pub conflicts_expected: Vec<PathBuf>,
    pub files_to_merge: Vec<PathBuf>,
    pub can_fast_forward: bool,
}

pub async fn validate_merge(
    worktree_path: &Path,
    target_branch: &str
) -> Result<MergeValidation>;

pub fn check_uncommitted_changes(path: &Path) -> Result<Vec<PathBuf>>;
pub fn predict_conflicts(
    source_branch: &str,
    target_branch: &str
) -> Result<Vec<PathBuf>>;
```

**Implementation approach**:
- Run `git status` for uncommitted changes
- Run `git diff --name-only` for changes
- Run `git merge-tree` for conflict prediction
- Check if fast-forward possible

**Estimated time**: 6 hours

#### Task 6.3: Commit Automation
**File**: `crates/chat-cli/src/cli/chat/merge/commit.rs`

**Functions to implement**:
```rust
pub async fn commit_worktree_changes(
    worktree_path: &Path,
    message: Option<&str>
) -> Result<String>; // Returns commit hash

pub fn generate_commit_message(
    conversation: &ConversationState
) -> String;
```

**Implementation approach**:
- Stage all changes: `git add .`
- Generate message from conversation summary
- Commit: `git commit -m <message>`
- Return commit hash

**Estimated time**: 3 hours

#### Task 6.4: Merge Execution
**File**: `crates/chat-cli/src/cli/chat/merge/execute.rs`

**Functions to implement**:
```rust
pub struct MergeResult {
    pub success: bool,
    pub conflicts: Vec<PathBuf>,
    pub merged_files: Vec<PathBuf>,
    pub commit_hash: Option<String>,
}

pub async fn execute_merge(
    worktree_path: &Path,
    source_branch: &str,
    target_branch: &str,
) -> Result<MergeResult>;
```

**Implementation approach**:
- Switch to target branch
- Run `git merge <source>`
- Parse output for conflicts
- Return result

**Estimated time**: 4 hours

#### Task 6.5: Conflict Resolution
**File**: `crates/chat-cli/src/cli/chat/merge/conflicts.rs`

**Functions to implement**:
```rust
pub struct Conflict {
    pub file: PathBuf,
    pub ours: String,
    pub theirs: String,
    pub base: Option<String>,
}

pub async fn detect_conflicts(repo_root: &Path) -> Result<Vec<Conflict>>;

pub async fn resolve_conflict_with_llm(
    conflict: &Conflict,
    conversation: &ConversationState
) -> Result<String>;

pub async fn apply_resolution(
    file: &Path,
    resolution: &str
) -> Result<()>;
```

**Implementation approach**:
- Parse conflict markers
- Extract ours/theirs/base
- Ask LLM for resolution
- Apply resolution
- Stage file

**Estimated time**: 8 hours

#### Task 6.6: Cleanup After Merge
**File**: `crates/chat-cli/src/cli/chat/merge/cleanup.rs`

**Functions to implement**:
```rust
pub async fn cleanup_after_merge(
    worktree_path: &Path,
    branch_name: &str,
    delete_branch: bool
) -> Result<()>;
```

**Implementation approach**:
- Remove worktree: `git worktree remove`
- Delete branch (optional): `git branch -d`
- Delete conversation directory
- Update session registry

**Estimated time**: 3 hours

#### Task 6.7: Merge UI Flow
**File**: `crates/chat-cli/src/cli/chat/merge/ui.rs`

**Functions to implement**:
```rust
pub async fn show_merge_preview(
    validation: &MergeValidation
) -> Result<()>;

pub async fn confirm_merge(
    validation: &MergeValidation
) -> Result<bool>;

pub async fn show_merge_progress(
    stage: MergeStage
) -> Result<()>;

pub enum MergeStage {
    Validating,
    Committing,
    Merging,
    ResolvingConflicts,
    Cleaning,
    Complete,
}
```

**UI flow**:
```
Merge Preview:
  Source: feature/readonly-tool
  Target: main
  Files to merge: 3
    - execute/mod.rs
    - tool_index.json
    - tools/mod.rs
  Potential conflicts: 0
  
Proceed with merge? [y/n]: y

✓ Committed changes (abc123)
✓ Merged to main (no conflicts)
✓ Removed worktree
✓ Cleaned up conversation

You are now in main worktree.
```

**Estimated time**: 4 hours

**Phase 6 Total**: ~30 hours (4 days)

---

## Integration Points

### 1. ChatSession Initialization
**File**: `crates/chat-cli/src/cli/chat/mod.rs`

**Changes needed**:
```rust
impl ChatSession {
    pub async fn new(args: ChatArgs, os: &mut Os) -> Result<Self> {
        // Existing initialization...
        
        // NEW: Resolve worktree strategy
        let strategy = resolve_worktree_strategy(
            &args,
            None, // skill
            None, // agent
            &current_dir
        ).await?;
        
        // NEW: Create worktree if needed
        if let WorktreeStrategy::Create(name) = strategy {
            let worktree_path = create_worktree_for_session(
                &current_dir,
                &name,
                &args
            ).await?;
            
            // Switch to worktree
            std::env::set_current_dir(&worktree_path)?;
        }
        
        // Continue with existing initialization...
    }
}
```

**Estimated time**: 4 hours

### 2. Conversation Loading
**File**: `crates/chat-cli/src/cli/chat/conversation.rs`

**Changes needed**:
```rust
impl ConversationState {
    pub async fn load_or_create(
        os: &Os,
        conversation_id: &str,
        path: &Path
    ) -> Result<Self> {
        // NEW: Try worktree-local storage first
        if let Some(conv) = Self::load_from_worktree(path).await? {
            return Ok(conv);
        }
        
        // Fallback to central DB
        if let Some(conv) = os.database.get_conversation_by_path(path)? {
            return Ok(conv);
        }
        
        // Create new
        Self::new(conversation_id, path)
    }
}
```

**Estimated time**: 2 hours

### 3. Conversation Saving
**File**: `crates/chat-cli/src/cli/chat/conversation.rs`

**Changes needed**:
```rust
impl ConversationState {
    pub async fn save(&self, os: &Os) -> Result<()> {
        // NEW: Determine storage location
        let storage_path = if is_worktree_storage(&self.path) {
            self.path.join(".q/conversation.json")
        } else {
            // Use central DB
            return os.database.set_conversation_by_path(&self.path, self);
        };
        
        // Save to worktree
        self.save_to_worktree(&self.path).await
    }
}
```

**Estimated time**: 2 hours

### 4. Tool Execution Context
**File**: `crates/chat-cli/src/cli/chat/tools/mod.rs`

**Changes needed**:
- Ensure tools use correct working directory
- Path sanitization (hide `.q/` directory from LLM)
- Relative path resolution

**Estimated time**: 3 hours

### 5. Checkpoint Integration
**File**: `crates/chat-cli/src/cli/chat/checkpoint.rs`

**Changes needed**:
- Checkpoints should work within worktrees
- Shadow repo per worktree or shared?
- Update checkpoint paths

**Estimated time**: 4 hours

**Total Integration**: ~15 hours (2 days)

---

## Total Effort Estimate

| Phase | Hours | Days |
|-------|-------|------|
| Phase 1: Git Detection | 22 | 3 |
| Phase 2: Conversation Storage | 16 | 2 |
| Phase 3: Decision Logic | 18 | 2-3 |
| Phase 4: Worktree Naming | 18 | 2-3 |
| Phase 5: Session Discovery | 19 | 2-3 |
| Phase 6: Merge Workflow | 30 | 4 |
| Integration | 15 | 2 |
| **Total** | **138** | **17-19** |

**With buffer (30%)**: ~180 hours / 23 days / **4-5 weeks**

---

## Milestones

### Milestone 1: Core Infrastructure (End of Week 2)
- Git detection working
- Worktree creation/removal
- Conversation storage refactored
- **Demo**: Create worktree manually, Q detects it

### Milestone 2: Automatic Worktrees (End of Week 3)
- Decision logic complete
- Session types working
- CLI arguments functional
- **Demo**: Q auto-creates worktree for feature work

### Milestone 3: Smart Naming (End of Week 4)
- LLM name generation
- Conflict resolution
- Interactive confirmation
- **Demo**: Q suggests good worktree names

### Milestone 4: Session Management (End of Week 5)
- /sessions shows worktrees
- Session status detection
- Cleanup commands
- **Demo**: Manage multiple parallel sessions

### Milestone 5: Merge Workflow (End of Week 6)
- Merge command working
- Conflict resolution
- Automatic cleanup
- **Demo**: Complete parallel workflow end-to-end

---

## Definition of Done

Each task is complete when:
- [ ] Code implemented and reviewed
- [ ] Unit tests written and passing
- [ ] Integration tests passing
- [ ] Documentation updated
- [ ] Manual testing completed
- [ ] No known bugs

Each phase is complete when:
- [ ] All tasks done
- [ ] Phase milestone demo successful
- [ ] Performance acceptable
- [ ] User feedback positive (if applicable)

---

## Tracking

Use GitHub issues with labels:
- `worktree-feature` - All related issues
- `phase-1` through `phase-6` - Phase labels
- `priority-high/medium/low` - Prioritization
- `blocked` - Waiting on dependencies
- `in-progress` - Currently being worked on
- `review` - Ready for code review
- `testing` - In testing phase

---

## Questions for Team

1. Should we implement all phases or prioritize subset?
2. What's the target release timeline?
3. Do we need Windows-specific testing?
4. Should merge workflow be in Phase 1 or can it wait?
5. What's the minimum viable feature set?
