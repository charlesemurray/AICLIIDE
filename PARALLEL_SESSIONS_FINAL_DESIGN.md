# Parallel Sessions with Git Worktrees - Final Design

## Overview

Enable users to run multiple interactive Q CLI sessions simultaneously on the same codebase, with automatic worktree management to prevent file conflicts and maintain conversation isolation.

## Goals

1. Allow parallel development without file conflicts
2. Maintain independent conversation history per session
3. Automatic worktree creation with minimal user friction
4. Clean merge workflow back to main branch
5. Work with or without git (graceful degradation)

## Core Concepts

### Session Types

| Type | Description | Worktree Behavior | User Interaction |
|------|-------------|-------------------|------------------|
| **Exploration** | Read-only, asking questions | Never create | Interactive |
| **Feature** | New functionality | Always create | Interactive |
| **Hotfix** | Quick fixes | Ask user | Interactive |
| **Refactor** | Large code changes | Always create | Interactive |
| **Experiment** | Temporary/throwaway work | Create (temp) | Interactive |
| **Agent** | Background task | Never create | Non-interactive |
| **Delegate** | Async agent task | Always create | Non-interactive |

### Worktree Strategy

```rust
enum WorktreeStrategy {
    Create(String),      // Create with specified name
    CreateTemp,          // Create temporary worktree
    UseExisting,         // Use current worktree/branch
    Never,               // Don't use worktrees
    Ask,                 // Prompt user for decision
}
```

## Decision Logic (Layered)

### Layer 1: Explicit User Input (Highest Priority)

```bash
# User specifies worktree name
q chat --worktree feature-name
# → Create worktree "feature-name" (must provide name)

# User specifies session type
q chat --type feature
# → Create worktree (auto-generate name)

q chat --type exploration
# → Never create worktree

# User explicitly disables
q chat --no-worktree
# → Never create worktree
```

**Rules**:
- `--worktree` requires a name, checks for conflicts
- `--type` determines worktree behavior per session type
- `--no-worktree` overrides all other logic

### Layer 2: Skill/Agent Configuration

Skills and agents declare their requirements:

```json
// Skill config
{
  "name": "feature-scaffolder",
  "sessionType": "feature",
  "requiresWorktree": true,
  "modifiesFiles": true
}

// Agent config
{
  "name": "test-runner-agent",
  "type": "background",
  "requiresWorktree": false
}
```

**Rules**:
- If `requiresWorktree: true` → Create worktree
- If `requiresWorktree: false` → Never create
- If `type: background` → Never create
- If `type: task` → Create worktree

### Layer 3: Git State Detection

```rust
fn check_git_state() -> WorktreeStrategy {
    match detect_git_context() {
        // Already in a worktree
        Ok(info) if info.is_worktree => UseExisting,
        
        // In main/master branch
        Ok(info) if info.is_main_branch() => Ask,
        
        // In feature branch (not worktree)
        Ok(info) => UseExisting,
        
        // Not a git repo
        Err(_) => Never,
    }
}
```

**Rules**:
- Already in worktree → Use it
- In main branch → Ask user (Layer 4)
- In feature branch → Use current branch
- Not git repo → Disable worktrees

### Layer 4: Ask User (Fallback)

Triggered when:
- In main branch and about to modify files
- Layer 3 returns `Ask`

```
Q: "This will modify files. Create a worktree to isolate changes?
    [y] Yes, create worktree
    [n] No, work in current directory"
```

## Worktree Naming

### User-Requested (--worktree flag)

```bash
q chat --worktree my-feature
# User must provide name
# Check for conflicts, error if exists
```

**Rules**:
- Name is required
- Check if worktree exists → Error
- Check if branch exists → Error
- No LLM involvement

### Auto-Generated (Interactive Sessions)

```bash
q chat --type feature
> "Add readonly bash tool"

# LLM generates name from conversation
Q: "I'll create worktree 'feature/readonly-bash-tool' for this work.
    
    Press Enter to accept, or type a new name:"
```

**Rules**:
- LLM analyzes conversation, suggests name
- Format: `<type>/<description>` (e.g., `feature/readonly-tool`)
- If LLM returns "GENERIC" → Ask user for name
- Check for conflicts → Ask user for different name
- User can accept (Enter) or provide custom name

### Auto-Generated (Non-Interactive Sessions)

```bash
# Agent/delegate task
Q: "Creating worktree 'task/mcp-refactor' for agent..."
```

**Rules**:
- LLM generates name, no user prompt
- If conflict → Append short UUID: `task/mcp-refactor-a3f9`
- If LLM returns "GENERIC" → Use `task/<uuid>`

## Conversation Storage

### Location

Store conversations **in the worktree** (not central database):

```
/workspace/q-cli-feature-a/
  .q/
    conversation.json       # Conversation history
    checkpoints/            # Checkpoint data (if enabled)
  src/
  ...
```

### Conversation Key

Use git-aware key format:

```
<repo-name>/<branch-name>
```

Examples:
- `q-cli/main`
- `q-cli/feature/readonly-tool`
- `q-cli/bugfix/parser-leak`

### Fallback (Non-Git)

If not in git repo, use absolute path (current behavior):
```
/workspace/my-project/
```

## Session Discovery

### /sessions Command

Enhanced to show worktree-based sessions:

```bash
> "/sessions"

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
```

**Implementation**:
- Scan git worktrees in current repo
- Check for `.q/conversation.json` in each
- Show status, last activity, file changes
- Highlight current session

## Merge Workflow

### User-Initiated Merge

```bash
# In feature worktree
q chat
> "Merge this to main"

Q: "I'll merge 'feature/readonly-tool' to main. This will:
    1. Commit current changes
    2. Switch to main worktree
    3. Merge feature/readonly-tool
    4. Remove this worktree
    5. Delete conversation
    
    Proceed? [y/n]"

# User: y

Q: "✓ Changes committed
    ✓ Merged to main (no conflicts)
    ✓ Removed worktree
    ✓ Cleaned up conversation
    
    You are now in main worktree."
```

### Merge with Conflicts

```bash
Q: "⚠️  Merge conflict in execute/mod.rs
    
    Options:
    1. Let me resolve it
    2. Show me the conflict
    3. Abort merge
    
    Choose [1/2/3]:"

# User: 1
Q: "Analyzing conflict...
    The conflict is in the ExecuteCommand struct.
    Main branch added 'timeout' field.
    Your branch added 'is_readonly' field.
    
    I'll keep both fields. Continue? [y/n]"
```

### Automatic Cleanup

After successful merge:
1. Commit any uncommitted changes
2. Switch to main worktree
3. Run `git merge <branch>`
4. If successful:
   - Remove worktree: `git worktree remove <path>`
   - Delete conversation: `rm -rf <path>/.q/`
   - Delete branch (optional): `git branch -d <branch>`

## Implementation Components

### 1. Git Detection Module

```rust
struct GitContext {
    repo_root: PathBuf,
    repo_name: String,
    branch_name: String,
    is_worktree: bool,
    is_main_branch: bool,
    worktree_path: Option<PathBuf>,
}

fn detect_git_context(path: &Path) -> Result<GitContext>;
fn list_worktrees(repo_root: &Path) -> Result<Vec<WorktreeInfo>>;
fn create_worktree(repo_root: &Path, name: &str, base_branch: &str) -> Result<PathBuf>;
fn remove_worktree(path: &Path) -> Result<()>;
```

### 2. Conversation Key Resolver

```rust
fn resolve_conversation_key(
    path: &Path,
    override_context: Option<&str>
) -> String {
    // 1. Manual override
    if let Some(ctx) = override_context {
        return ctx.to_string();
    }
    
    // 2. Git context
    if let Ok(git) = detect_git_context(path) {
        return format!("{}/{}", git.repo_name, git.branch_name);
    }
    
    // 3. Fallback to path
    path.to_string_lossy().to_string()
}
```

### 3. Worktree Strategy Resolver

```rust
async fn resolve_worktree_strategy(
    args: &ChatArgs,
    skill: Option<&SkillConfig>,
    agent: Option<&AgentConfig>,
) -> Result<WorktreeStrategy> {
    // Layer 1: Explicit flags
    if let Some(strategy) = check_explicit_flags(args) {
        return Ok(strategy);
    }
    
    // Layer 2: Skill/Agent config
    if let Some(strategy) = check_skill_agent_config(skill, agent) {
        return Ok(strategy);
    }
    
    // Layer 3: Git state
    let strategy = check_git_state()?;
    
    // Layer 4: Ask user if needed
    if matches!(strategy, WorktreeStrategy::Ask) {
        return ask_user_for_worktree().await;
    }
    
    Ok(strategy)
}
```

### 4. Worktree Name Generator

```rust
async fn generate_worktree_name(
    conversation: &Conversation,
    session_type: SessionType,
    is_interactive: bool,
) -> Result<String> {
    // Ask LLM for name suggestion
    let suggested = llm_suggest_name(conversation, session_type).await?;
    
    // Handle generic names
    if suggested == "GENERIC" {
        if is_interactive {
            return ask_user_for_name().await;
        } else {
            return Ok(format!("task/{}", short_uuid()));
        }
    }
    
    // Check conflicts
    if worktree_exists(&suggested) {
        if is_interactive {
            return resolve_conflict_interactive(&suggested).await;
        } else {
            return Ok(format!("{}-{}", suggested, short_uuid()));
        }
    }
    
    // Confirm with user (interactive only)
    if is_interactive {
        return confirm_or_change_name(&suggested).await;
    }
    
    Ok(suggested)
}
```

### 5. Session Manager Integration

```rust
impl SessionManager {
    // Discover worktree-based sessions
    pub fn discover_worktree_sessions(&mut self, repo_root: &Path) -> Result<()> {
        let worktrees = list_worktrees(repo_root)?;
        
        for wt in worktrees {
            let conv_path = wt.path.join(".q/conversation.json");
            if conv_path.exists() {
                let session = load_session_from_worktree(&wt)?;
                self.register_session(session);
            }
        }
        
        Ok(())
    }
    
    // Merge and cleanup
    pub async fn merge_session(&mut self, session_name: &str) -> Result<()> {
        let session = self.get_session(session_name)?;
        
        // Commit changes
        commit_worktree_changes(&session.worktree_path)?;
        
        // Merge to main
        merge_to_main(&session.branch_name)?;
        
        // Cleanup
        remove_worktree(&session.worktree_path)?;
        self.close_session(session_name)?;
        
        Ok(())
    }
}
```

### 6. Conversation Persistence

```rust
impl ConversationState {
    // Save to worktree
    pub async fn save_to_worktree(&self, worktree_path: &Path) -> Result<()> {
        let q_dir = worktree_path.join(".q");
        fs::create_dir_all(&q_dir).await?;
        
        let conv_path = q_dir.join("conversation.json");
        let json = serde_json::to_string_pretty(self)?;
        fs::write(conv_path, json).await?;
        
        Ok(())
    }
    
    // Load from worktree
    pub async fn load_from_worktree(worktree_path: &Path) -> Result<Self> {
        let conv_path = worktree_path.join(".q/conversation.json");
        let json = fs::read_to_string(conv_path).await?;
        let state = serde_json::from_str(&json)?;
        Ok(state)
    }
}
```

## CLI Arguments

### New Flags

```rust
#[derive(Parser)]
pub struct ChatArgs {
    // Existing args...
    
    /// Create or use a worktree with specified name
    #[arg(long)]
    pub worktree: Option<String>,
    
    /// Disable worktree creation
    #[arg(long)]
    pub no_worktree: bool,
    
    /// Session type (exploration, feature, hotfix, refactor, experiment)
    #[arg(long)]
    pub session_type: Option<SessionType>,
}
```

## Edge Cases

### 1. Worktree Deleted Externally

```bash
# User manually deletes worktree
rm -rf /workspace/q-cli-feature-a/

# Q detects missing worktree
> "/sessions"
Q: "⚠️  Worktree 'feature-a' not found. Clean up? [y/n]"
```

### 2. Branch Already Merged

```bash
# Branch merged outside Q CLI
> "Merge this to main"
Q: "Branch 'feature/readonly-tool' is already merged to main.
    Remove worktree? [y/n]"
```

### 3. Uncommitted Changes During Merge

```bash
> "Merge to main"
Q: "You have uncommitted changes:
    - execute/mod.rs (modified)
    - tool_index.json (modified)
    
    Commit these changes? [y/n]"
```

### 4. Multiple Worktrees Ready to Merge

```bash
> "/sessions merge-all"
Q: "Multiple sessions ready to merge:
    1. feature/readonly-tool (3 files)
    2. bugfix/parser-leak (1 file)
    
    Merge order: [1,2] or specify: "
```

### 5. Detached HEAD State

```bash
# In detached HEAD
q chat
Q: "You're in detached HEAD state. 
    Create a branch for this work? [y/n]"
```

## Backward Compatibility

### Existing Conversations

- Conversations stored in central DB remain accessible
- New conversations use worktree storage
- Migration tool (optional): `/sessions migrate-to-worktrees`

### Non-Git Projects

- Worktree features disabled
- Falls back to current behavior (path-based conversations)
- No errors, graceful degradation

### Git Not Installed

- Detect at startup: `which git`
- Disable worktree features
- Show info message once: "Git not found. Worktree features disabled."

## Success Metrics

1. **User can run 3+ parallel sessions** without file conflicts
2. **Worktree creation takes <2 seconds** (including name generation)
3. **Merge success rate >95%** (conflicts handled gracefully)
4. **Zero data loss** (conversations persist through crashes)
5. **Works without git** (graceful fallback)

## Future Enhancements

- Cross-session context sharing (optional)
- Conflict prediction before merge
- Team collaboration (shared worktrees)
- Integration with tmux/terminal multiplexers
- Visual dashboard for session management
- Automatic rebase on main changes

## Implementation Phases

### Phase 1: Core Infrastructure
- Git detection module
- Worktree creation/removal
- Conversation storage in worktrees
- Basic session discovery

### Phase 2: Decision Logic
- Layered strategy resolver
- Session type detection
- Skill/agent config integration

### Phase 3: Naming & UX
- LLM-based name generation
- Interactive confirmation
- Conflict resolution

### Phase 4: Merge Workflow
- Merge command
- Conflict handling
- Cleanup automation

### Phase 5: Polish
- /sessions enhancements
- Edge case handling
- Documentation
- Testing

## Open Questions

1. Should we support nested worktrees?
2. How to handle submodules?
3. Should conversations be git-ignored by default?
4. What about .qconfig in worktrees vs main?
5. Should we support worktree templates?
