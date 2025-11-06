# /close Command Implementation

## Overview
Implemented a new top-level `/close` slash command that closes the current session. When the last session is closed, a new default session is automatically created instead of exiting the application.

## Changes Made

### 1. Added `/close` Slash Command
**File**: `crates/chat-cli/src/cli/chat/cli/mod.rs`

- Added `Close` variant to `SlashCommand` enum (line ~282)
- Added handler in `execute()` method that delegates to `SessionsSubcommand::Close`
- Added "close" to `command_name()` method

### 2. Modified Coordinator to Auto-Create Session
**File**: `crates/chat-cli/src/cli/chat/coordinator.rs`

Modified `close_session()` method:
- Changed signature to accept optional `SessionContext`: `pub async fn close_session(&mut self, name: &str, context: Option<SessionContext>)`
- When closing the last session and context is provided, automatically creates a new "default" session with `SessionType::Development`
- Sets the new session as active

### 3. Updated Session Integration
**File**: `crates/chat-cli/src/cli/chat/session_integration.rs`

- Updated `SessionCommand::Close` handler to pass context to `close_session()`
- Context is obtained from the `context_factory` closure

## Behavior

### Before
- `/sessions close <name>` - Close a specific session
- `/quit` - Exit the application
- Closing the last session would leave no active session, causing exit

### After
- `/close` - Close the current session (new top-level command)
- `/sessions close <name>` - Close a specific session (still works)
- `/quit` - Exit the application (unchanged)
- **Closing the last session automatically creates a new "default" session**

## Usage Examples

```bash
# Close current session
/close

# If this was the last session, a new "default" session is created automatically
# User stays in the chat interface instead of exiting

# Close a specific session by name
/sessions close my-session

# Quit the application entirely
/quit
```

## Technical Details

### Session Creation on Close
When the last session is closed:
1. `close_session()` detects no remaining sessions
2. If `SessionContext` is provided, creates new session with:
   - Name: "default"
   - Type: `SessionType::Development`
3. Sets new session as active
4. User seamlessly continues in the new session

### Context Propagation
The `SessionContext` contains:
- `conversation_id`: Unique ID for the conversation
- `agents`: Available agents
- `model`: Model configuration
- Other session-specific data

This context is passed through the command chain:
1. `SlashCommand::Close` → `SessionsSubcommand::Close`
2. `SessionsSubcommand::execute()` → `handle_session_command()`
3. `handle_session_command()` → `coordinator.close_session()`

## Testing Recommendations

1. **Single session close**: Start with one session, use `/close`, verify new session created
2. **Multiple session close**: Have 3 sessions, close current, verify switch to another
3. **Named close**: Use `/sessions close <name>` to close non-active session
4. **Quit still works**: Verify `/quit` exits regardless of session count

## Related Files
- `crates/chat-cli/src/cli/chat/cli/mod.rs` - Command definitions
- `crates/chat-cli/src/cli/chat/coordinator.rs` - Session lifecycle management
- `crates/chat-cli/src/cli/chat/session_integration.rs` - Command routing
- `crates/chat-cli/src/theme/session.rs` - SessionType enum
