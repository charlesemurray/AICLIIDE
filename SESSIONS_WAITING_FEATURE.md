# /sessions --waiting Feature

## Overview
Implemented the `/sessions --waiting` command to list sessions that are waiting for user input, providing visibility into which sessions need attention.

## Implementation

### Changes Made

1. **SessionSwitcher** (`session_switcher.rs`)
   - Added `list_waiting_sessions()` method
   - Filters sessions using coordinator's `get_waiting_sessions()`
   - Shows "No sessions waiting for input" when list is empty

2. **Session Integration** (`session_integration.rs`)
   - Updated command handler to check `waiting` flag
   - Routes to `list_waiting_sessions()` when flag is true
   - Falls back to `list_sessions()` for normal listing

3. **Cleanup** (`session_commands.rs`)
   - Removed unused `SessionStatus` import

## Usage

```bash
# List all active sessions
/sessions

# List only sessions waiting for input
/sessions --waiting

# List all sessions including completed
/sessions --all
```

## Technical Details

### Leveraged Existing Infrastructure
- Uses `MultiSessionCoordinator::get_waiting_sessions()` (already implemented)
- Filters by `SessionState::WaitingForInput` state
- No changes needed to core coordinator logic

### Session States
Sessions can be in the following states:
- `Active` - Currently active session
- `WaitingForInput` - Paused, waiting for user input
- `Processing` - Executing a command or tool
- `Paused` - Manually paused
- `Completed` - Finished

## Benefits

1. **Visibility** - Users can quickly see which sessions need attention
2. **Workflow** - Helps manage multiple concurrent sessions
3. **Minimal** - Only 24 lines of code added
4. **Reusable** - Leverages existing coordinator infrastructure

## Testing

Build verified successfully:
```bash
cargo build --lib
# Finished `dev` profile [unoptimized + debuginfo] target(s) in 7.22s
```

## Commit
- Commit: `9ae930e2`
- Branch: `main`
- Status: Merged and pushed to origin
