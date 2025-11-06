# Merge Conflict Resolution System

## Overview
Implemented a complete merge conflict resolution workflow with dedicated chat assistance for worktree sessions.

## Features Implemented

### 1. Merge State Tracking
- Added `MergeState` enum to `WorktreeInfo`:
  - `None` - No merge in progress
  - `Conflicted { files }` - Merge has conflicts
  - `Resolving` - User is resolving conflicts

### 2. Merge Command
```bash
/sessions merge <session_id>           # Attempt merge, detect conflicts
/sessions merge <session_id> --force   # Proceed with merge despite conflicts
/sessions merge <session_id> --continue # Complete merge after resolution
```

### 3. Conflict Detection
- Pre-merge conflict detection using `git merge-tree`
- Real-time conflict file listing
- Unresolved conflict verification

### 4. Conflict Resolution Chat
When conflicts are detected with `--force`:
- Launches dedicated chat session with context
- Provides list of conflicted files
- Offers guidance on resolution steps
- User resolves conflicts in worktree
- Completes merge with `--continue`

## Workflow

### Happy Path (No Conflicts)
```
1. User: /sessions merge abc123
2. System: ✓ Merge completed successfully
3. System: ✓ Worktree cleaned up
```

### Conflict Path
```
1. User: /sessions merge abc123
2. System: ⚠️  Conflicts detected. Use --force to proceed.

3. User: /sessions merge abc123 --force
4. System: ⚠️  Merge has conflicts. Launching resolution assistant...
   
   I need help resolving merge conflicts when merging branch 'feature-x' into 'main'.
   
   Conflicted files:
     - src/main.rs
     - README.md
   
   Please help me:
   1. Understand what changes conflict
   2. Decide how to resolve each conflict
   3. Verify the resolution is correct
   
   I'm in the worktree and ready to edit files.
   
   💡 After resolving conflicts, run: /sessions merge abc123 --continue

5. User: [Edits files, resolves conflicts]

6. User: /sessions merge abc123 --continue
7. System: ✓ Merge completed successfully
8. System: ✓ Worktree cleaned up
```

## Implementation Details

### New Functions in merge_workflow.rs
- `launch_conflict_resolution_chat()` - Generates helpful prompt for conflict resolution
- `has_unresolved_conflicts()` - Checks for unresolved conflicts
- `get_conflicted_files()` - Lists files with conflicts
- `complete_merge()` - Stages and commits after resolution
- Updated `merge_branch()` - Returns conflict info instead of bailing

### Modified Files
- `crates/chat-cli/src/session/metadata.rs` - Added MergeState enum
- `crates/chat-cli/src/cli/chat/merge_workflow.rs` - Conflict resolution functions
- `crates/chat-cli/src/cli/chat/cli/session_mgmt.rs` - Merge command handler
- `crates/chat-cli/src/cli/chat/cli/mod.rs` - Command name mapping
- `crates/chat-cli/src/cli/chat/mod.rs` - WorktreeInfo initializations
- `crates/chat-cli/src/git/worktree.rs` - WorktreeInfo conversion

## Benefits

1. **Guided Resolution** - Chat provides context-aware help
2. **Safe Workflow** - Requires explicit --force for conflicts
3. **State Tracking** - System knows merge status
4. **Clean Completion** - Automatic cleanup after successful merge
5. **Error Recovery** - Can abort and retry at any step

## Future Enhancements

- Interactive conflict resolution UI
- Diff visualization in chat
- Automatic conflict resolution suggestions
- Merge conflict history tracking
- Integration with code review tools
