# Dynamic ChatSession Creation

## Implementation

Added ability to create ChatSession instances on-demand when switching to sessions that don't have one yet.

### New Method: ChatSession::from_conversation()

Creates a ChatSession from an existing ConversationState:

```rust
pub async fn from_conversation(
    os: &mut Os,
    conversation: ConversationState,
    input_source: InputSource,
) -> Result<Self>
```

**Features:**
- Minimal setup - only requires conversation and input source
- Creates conduits for stdout/stderr
- Sets up ctrl-c handling
- Uses default settings (interactive, no auto-approve, etc.)

### Coordinator.run() Enhancement

Now creates ChatSessions on-demand:

```rust
// Get or create ChatSession for active session
if session.chat_session.is_none() {
    let input_source = InputSource::new(os, ...)?;
    let chat_session = ChatSession::from_conversation(
        os,
        session.conversation.clone(),
        input_source,
    ).await?;
    session.chat_session = Some(Arc::new(Mutex::new(chat_session)));
}
```

## Testing Flow

Now you can:

1. Start chat: `q chat`
2. Create new session: `/sessions new hello`
3. Switch to it: `/switch hello`
4. ChatSession is created automatically
5. Prompt shows: `(hello) >`

## Current Limitations

1. **No coordinator reference**: Dynamically created sessions don't have access to the coordinator, so nested session commands won't work
2. **Default settings**: New sessions use default settings (interactive mode, no auto-approve, etc.)
3. **No analytics**: Dynamically created sessions don't have analytics enabled

## Files Modified

1. `crates/chat-cli/src/cli/chat/mod.rs` - Added `from_conversation()` method
2. `crates/chat-cli/src/cli/chat/coordinator.rs` - Added dynamic ChatSession creation in `run()`

## Build Status

✅ Build successful
