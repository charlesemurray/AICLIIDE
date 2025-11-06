# Session Persistence on Quit

## Overview
Sessions are now automatically saved when quitting the application and restored on next launch. Closed sessions are marked as archived and not restored.

## Behavior

### Quit vs Close

**`/quit` - Exit Application**:
- Saves all active sessions
- Sessions remain "active" status
- Restored on next launch

**`/close` - Close Session**:
- Marks session as "Completed" (archived)
- NOT restored on next launch
- Conversation history preserved in database

## Implementation

### 1. Save on Quit
**File**: `crates/chat-cli/src/cli/chat/coordinator.rs`

When `should_quit` flag is detected:
```rust
// Save all active sessions before exit
for id in session_ids {
    coord.save_session(&id).await;
}
std::process::exit(0);
```

### 2. Archive on Close
**File**: `crates/chat-cli/src/cli/chat/coordinator.rs`

When closing a session:
```rust
// Mark as Completed (archived)
let persisted = PersistedSession {
    status: SessionStatus::Completed,
    // ... other fields
};
persistence.save_session(&persisted);
```

### 3. Load Active Sessions
**File**: `crates/chat-cli/src/cli/chat/coordinator.rs`

On application start:
```rust
// Filter for active sessions only
let active_sessions = persisted_sessions
    .into_iter()
    .filter(|s| !matches!(s.status, SessionStatus::Completed))
    .collect();

// Restore each active session
for persisted in active_sessions {
    // Load conversation from database
    // Create session with restored state
}
```

## Usage Examples

### Scenario 1: Quit and Resume
```bash
# Start app, create sessions
q chat
> /sessions new feature-work
> /sessions new debugging

# Work in sessions...
> /quit

# Later: Restart app
q chat
# ✓ Loaded 2 saved session(s)
# Both sessions restored automatically
```

### Scenario 2: Close Session
```bash
# Close a session
> /close debugging

# Quit app
> /quit

# Later: Restart app
q chat
# ✓ Loaded 1 saved session(s)
# Only feature-work restored (debugging was archived)
```

### Scenario 3: Worktree Sessions
```bash
# In worktree A
cd /path/to/worktree-a
q chat
> /sessions new worktree-a-session

# In worktree B
cd /path/to/worktree-b
q chat
> /sessions new worktree-b-session

# Each worktree has its own sessions
# Sessions filtered by current directory
```

## Session States

- **Active**: Running session, will be restored
- **WaitingForInput**: Paused session, will be restored
- **Processing**: Working session, will be restored
- **Completed**: Archived/closed, NOT restored
- **Paused**: Suspended session, will be restored

## What Gets Saved

**PersistedSession** (metadata):
- `conversation_id`: Unique identifier
- `name`: Session name
- `session_type`: Development, Feature, etc.
- `status`: Active, Completed, etc.
- `created_at`: Creation timestamp
- `last_active`: Last activity timestamp

**ConversationState** (in database):
- Full message history
- Context files
- Tool state
- Model selection
- All conversation data

## Filtering Logic

Sessions are filtered by:
1. **Status**: Only non-Completed sessions
2. **Directory**: Matches current working directory
3. **Conversation ID**: Must exist in database

## Benefits

1. **Seamless Resume**: Pick up where you left off
2. **No Data Loss**: All active work preserved
3. **Clean Workspace**: Closed sessions don't clutter
4. **Worktree Isolation**: Each worktree has own sessions
5. **Automatic**: No manual save/load needed

## Edge Cases

1. **No database entry**: Session skipped (conversation deleted)
2. **Corrupted metadata**: Session skipped with warning
3. **Different directory**: Sessions not loaded (directory mismatch)
4. **All sessions closed**: Fresh start on next launch
5. **Persistence disabled**: No save/load (in-memory only)

## Future Enhancements

1. **Cross-directory sessions**: Option to restore all sessions
2. **Session migration**: Move sessions between directories
3. **Selective restore**: Choose which sessions to restore
4. **Session export/import**: Share sessions between machines
5. **Auto-archive**: Archive old inactive sessions

## Testing

```bash
# Test quit/resume
1. Create 3 sessions
2. Work in each
3. /quit
4. Restart app
5. Verify all 3 restored

# Test close/archive
1. Create 3 sessions
2. /close session-2
3. /quit
4. Restart app
5. Verify only 2 restored (session-2 archived)

# Test worktree isolation
1. Create session in worktree-a
2. cd to worktree-b
3. Start app
4. Verify worktree-a session NOT loaded
```

## Related Files
- `crates/chat-cli/src/cli/chat/coordinator.rs` - Save/load logic
- `crates/chat-cli/src/cli/chat/session_persistence.rs` - Persistence layer
- `crates/chat-cli/src/database/mod.rs` - Conversation storage
