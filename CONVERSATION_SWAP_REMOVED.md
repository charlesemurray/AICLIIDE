# Removed Conversation-Swapping Approach

## What Was Removed

### 1. SwitchSession Handler
**Before**: Swapped conversations within the same ChatSession
```rust
ChatState::SwitchSession { target_id } => {
    // Load target conversation
    let new_conversation = target_session.conversation.clone();
    let new_name = target_session.display.name.clone();
    self.switch_conversation(new_conversation, new_name);
    Ok(ChatState::PromptUser { skip_printing_tools: false })
}
```

**After**: Exit to coordinator
```rust
ChatState::SwitchSession { target_id: _ } => {
    // Exit this ChatSession - coordinator will switch to the target session
    Ok(ChatState::Exit)
}
```

### 2. switch_conversation() Method
Removed entirely - no longer swapping conversations within a ChatSession.

### 3. Coordinator Main Loop
**Before**: Ran single session to completion
```rust
pub async fn run(&mut self, os: &mut Os) -> Result<()> {
    let session = get_active_session();
    session.spawn(os).await
}
```

**After**: Loop that switches between ChatSession instances
```rust
pub async fn run(&mut self, os: &mut Os) -> Result<()> {
    loop {
        let session = get_active_session();
        session.spawn(os).await?;
        
        // Check if active session changed (switch) or truly exited
        if active_session_changed() {
            continue; // Run new active session
        } else {
            break; // Exit
        }
    }
}
```

## New Architecture

### Session Switching Flow
1. User types `/switch hello`
2. SlashCommand updates `coordinator.active_session_id` to "hello"
3. Returns `ChatState::SwitchSession`
4. Current ChatSession exits with `ChatState::Exit`
5. Coordinator's run() loop detects active session changed
6. Loop continues with "hello" ChatSession

### Key Principles
- Each session has its own ChatSession instance
- No conversation swapping within a ChatSession
- Coordinator manages which ChatSession is active
- Switching = exit current + run target ChatSession

## Current Limitation

New sessions created with `/sessions new` don't have ChatSessions yet. The coordinator will error with:
```
Session has no ChatSession - dynamic session creation not yet implemented
```

This needs to be implemented next - creating ChatSession instances for dynamically created sessions.

## Files Modified
1. `crates/chat-cli/src/cli/chat/mod.rs` - Removed switch_conversation, simplified SwitchSession handler
2. `crates/chat-cli/src/cli/chat/coordinator.rs` - Implemented proper run() loop with session switching
