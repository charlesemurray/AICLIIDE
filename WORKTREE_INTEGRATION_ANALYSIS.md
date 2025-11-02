# Git Worktree Integration Analysis

## Current State

### How Q CLI Handles Projects

1. **Working Directory**: Q CLI operates in `std::env::current_dir()` - wherever you run it
2. **Conversation Storage**: Conversations are stored by path in SQLite database (`~/.aws/amazonq/database.db`)
   - Key: absolute path to working directory
   - Value: serialized `ConversationState`
3. **Checkpoint System**: Uses git for tracking changes
   - Creates shadow bare repos in `~/.aws/amazonq/shadow_repos/<conversation_id>/`
   - Requires git to be installed and working directory to be in a git repo
   - Uses `git worktree` commands internally for checkpoints

### Key Observations

- Q CLI is **path-based**, not **git-repo-based**
- Each directory path gets its own conversation
- Checkpoints already use git worktrees under the hood
- No explicit "project" or "workspace" concept

## Problem Statement

**User wants to**: Work on multiple branches simultaneously, each with independent Q CLI sessions and conversation history.

**Current behavior**: 
- Running Q CLI in the same directory on different branches shares the same conversation
- Switching branches loses context unless using checkpoints

## Proposed Solutions

### Option 1: Automatic Worktree Detection (Recommended)

**Concept**: Make Q CLI aware of git worktrees and treat each worktree as a separate context.

**Implementation**:
```
Conversation Key: <repo-root>/<worktree-branch> instead of <absolute-path>
```

**Pros**:
- Transparent to users
- Each worktree gets its own conversation automatically
- Works with existing worktree workflows
- Minimal UX changes

**Cons**:
- Requires git to be installed
- Needs fallback for non-git directories

**User Experience**:
```bash
# Terminal 1
cd /workspace/q-cli/amazon-q-developer-cli
git checkout main
q chat
# Gets conversation for "q-cli/main"

# Terminal 2  
cd /workspace/q-cli-feature
q chat
# Gets conversation for "q-cli/feature-branch"
```

### Option 2: Explicit Worktree Management Command

**Concept**: Add `q worktree` subcommand to manage Q-aware worktrees.

**Implementation**:
```bash
q worktree create <branch-name>
q worktree list
q worktree switch <branch-name>
q worktree remove <branch-name>
```

**Pros**:
- Explicit control
- Can add Q-specific features (auto-start chat, copy agent config, etc.)
- Clear mental model

**Cons**:
- More commands to learn
- Duplicates git functionality
- Requires users to use Q's worktree commands instead of git's

**User Experience**:
```bash
q worktree create feature-readonly-tool
# Creates worktree, sets up Q context, optionally starts chat

q worktree list
# Shows all Q-managed worktrees with conversation status
```

### Option 3: Context Switching (No Worktrees)

**Concept**: Add context/session management without requiring worktrees.

**Implementation**:
```bash
q context create <name>
q context switch <name>
q context list
```

**Pros**:
- Works without git
- Simpler than worktrees
- Can work across any directory structure

**Cons**:
- Doesn't solve the "multiple branches" problem
- Still need to manually switch git branches
- Adds complexity without solving the core issue

### Option 4: Hybrid Approach (Best UX)

**Concept**: Automatic detection + manual override.

**Implementation**:
1. **Automatic**: Detect worktrees and use branch-aware conversation keys
2. **Manual**: Allow `--context <name>` flag to override
3. **Fallback**: Use path-based keys for non-git directories

**Pros**:
- Best of all worlds
- Works automatically for 90% of cases
- Power users can override when needed
- Graceful degradation

**Cons**:
- Most complex to implement
- Need to handle edge cases

**User Experience**:
```bash
# Automatic (in git worktree)
cd /workspace/q-cli-main
q chat
# Uses context: "q-cli/main"

# Manual override
cd /workspace/q-cli-main
q chat --context my-experiment
# Uses context: "my-experiment"

# Non-git directory
cd /tmp/random-project
q chat
# Uses context: "/tmp/random-project" (current behavior)
```

## Recommended Approach: Option 4 (Hybrid)

### Implementation Plan

#### Phase 1: Detection Layer
1. Add `GitContext` module to detect:
   - Is directory in a git repo?
   - Is it a worktree?
   - What's the branch name?
   - What's the repo root?

2. Create conversation key resolver:
   ```rust
   fn resolve_conversation_key(path: &Path, override_context: Option<&str>) -> String {
       if let Some(ctx) = override_context {
           return ctx.to_string();
       }
       
       if let Ok(git_info) = detect_git_context(path) {
           return format!("{}/{}", git_info.repo_name, git_info.branch);
       }
       
       // Fallback to path
       path.to_string_lossy().to_string()
   }
   ```

#### Phase 2: Database Migration
1. Add migration to handle existing conversations
2. Optionally migrate path-based keys to git-based keys
3. Maintain backward compatibility

#### Phase 3: UX Enhancements
1. Add `--context` flag to `q chat`
2. Show current context in prompt/status
3. Add `q context list` to show all contexts
4. Add `q context switch` for quick switching

#### Phase 4: Documentation
1. Document worktree workflows
2. Add examples for parallel development
3. Explain context resolution logic

### Fallback Strategy

**When git is not available**:
- Fall back to path-based conversation keys (current behavior)
- Show warning: "Git not detected, using path-based context"
- Everything still works, just no automatic worktree detection

**When not in a git repo**:
- Use path-based keys
- No warnings (expected behavior)

**When git command fails**:
- Log error, fall back to path-based
- Don't break the user experience

### Edge Cases to Handle

1. **Detached HEAD**: Use commit SHA as branch name
2. **Submodules**: Use submodule path + branch
3. **Bare repos**: Not supported (can't run Q CLI in bare repo anyway)
4. **Symlinks**: Resolve to real path first
5. **Multiple worktrees same branch**: Add worktree path suffix

### Benefits

✅ **Transparent**: Works automatically for most users
✅ **Flexible**: Power users can override with `--context`
✅ **Backward compatible**: Existing conversations still work
✅ **Graceful degradation**: Falls back when git unavailable
✅ **Solves the problem**: Each worktree gets independent conversation
✅ **No new commands**: Uses existing `q chat` with optional flag

### Risks

⚠️ **Complexity**: More logic in conversation key resolution
⚠️ **Migration**: Need to handle existing conversations carefully
⚠️ **Testing**: Many edge cases to test
⚠️ **Performance**: Git detection adds overhead (can cache)

## Next Steps

1. **Prototype** git detection logic
2. **Test** with various git configurations
3. **Design** migration strategy for existing conversations
4. **Implement** conversation key resolver
5. **Add** `--context` flag
6. **Document** new behavior
7. **Test** extensively with real worktree workflows

## Open Questions

1. Should we show the context in the Q CLI prompt?
2. How to handle conversation history when switching contexts?
3. Should checkpoints be per-context or global?
4. What happens if user renames a branch?
5. Should we support copying conversation between contexts?
