# Quit System Fix

## Problem
The `/quit` command was not properly signaling the coordinator to exit the application. The coordinator loop was comparing session IDs and couldn't distinguish between a quit and other exit scenarios.

## Solution
Added an explicit `should_quit` flag to the coordinator that is set when the user issues `/quit`.

## Changes Made

### 1. Added Quit Flag to SessionState
**File**: `crates/chat-cli/src/cli/chat/coordinator.rs`

```rust
pub(crate) struct SessionState {
    // ... existing fields ...
    /// Flag to signal application should quit
    pub(crate) should_quit: bool,
}
```

### 2. Added quit() Method
**File**: `crates/chat-cli/src/cli/chat/coordinator.rs`

```rust
/// Signal the coordinator to quit the application
pub async fn quit(&self) {
    let mut state = self.state.lock().await;
    state.should_quit = true;
}
```

### 3. Updated /quit Command
**File**: `crates/chat-cli/src/cli/chat/cli/mod.rs`

```rust
Self::Quit => {
    // Signal coordinator to quit
    if let Some(ref coord) = session.coordinator {
        coord.lock().await.quit().await;
    }
    Ok(ChatState::Exit)
},
```

### 4. Updated Coordinator Loop
**File**: `crates/chat-cli/src/cli/chat/coordinator.rs`

The run loop now checks `should_quit` flag first:

```rust
let should_quit = state.should_quit;

if should_quit {
    eprintln!("[DEBUG] should_quit flag set - exiting");
    break;
}
```

## Behavior

### Before
- `/quit` would exit session but coordinator couldn't tell it was a quit
- Loop logic relied on comparing session IDs (ambiguous)

### After
- `/quit` explicitly sets `should_quit = true` in coordinator
- Coordinator loop checks flag and exits immediately
- Clear, unambiguous signal for application exit

## Benefits

1. **Explicit Intent**: Quit is a clear, dedicated signal
2. **No Ambiguity**: Doesn't rely on session ID comparisons
3. **Clean Separation**: Quit logic separate from session switching
4. **Future-Proof**: Easy to add other quit scenarios (timeout, error, etc.)

## Testing

```bash
# Start application
q chat

# Create multiple sessions
/sessions new session1
/sessions new session2

# Quit should exit immediately
/quit
# Application exits (not just session)
```
