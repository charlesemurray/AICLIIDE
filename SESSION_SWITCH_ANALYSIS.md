# Session Switch Flow Analysis

## User Action: `/switch hello`

### Step 1: Command Parsing
- User types `/switch hello` in session "noble-seal"
- SlashCommand parses it as `SessionsSubcommand::Switch { name: "hello" }`

### Step 2: SessionsSubcommand::execute()
Location: `crates/chat-cli/src/cli/chat/cli/mod.rs`

```rust
// Calls session_integration::handle_session_command()
// Which calls coordinator.switch_session("hello")
// This updates coordinator.active_session_id to "hello"'s conversation_id

// Then returns:
if is_switch {
    if let Some(active_id) = coord_lock.active_session_id().await {
        return Ok(ChatState::SwitchSession { target_id: active_id });
    }
}
```

**Result**: Returns `ChatState::SwitchSession { target_id: "d5790330..." }`

### Step 3: ChatSession::next() handles SwitchSession
Location: `crates/chat-cli/src/cli/chat/mod.rs:1542`

```rust
ChatState::SwitchSession { target_id: _ } => {
    // Exit this ChatSession - coordinator will switch to the target session
    Ok(ChatState::Exit)
}
```

**Result**: Returns `ChatState::Exit`

### Step 4: ChatSession::spawn() loop exits
Location: `crates/chat-cli/src/cli/chat/mod.rs:2186`

```rust
while !matches!(self.inner, Some(ChatState::Exit)) {
    self.next(os).await?;
}
```

**Result**: Loop exits, spawn() returns Ok(())

### Step 5: Coordinator::run() detects switch
Location: `crates/chat-cli/src/cli/chat/coordinator.rs`

```rust
match session.spawn(os).await {
    Ok(_) => {
        drop(session);
        let new_active = coord.state.active_session_id.clone();
        
        if new_active.as_ref() != Some(&session_id) {
            continue; // Run new active session
        } else {
            break; // Exit
        }
    }
}
```

**Expected**: `new_active` should be "d5790330..." (hello's ID)
**Expected**: `session_id` should be "noble-seal"'s ID
**Expected**: They don't match, so continue loop

### Step 6: Loop continues with new session
```rust
let active_id = coord.state.active_session_id.clone();
// Should be "d5790330..." (hello)

// Get or create ChatSession
if session.chat_session.is_none() {
    // Create new ChatSession
    chat_session.coordinator = Some(coord_arc.clone());
}
```

**Expected**: Creates ChatSession for "hello" with coordinator reference

### Step 7: New ChatSession runs
```rust
session.spawn(os).await
```

**Expected**: Prompt generation calls `generate_tool_trust_prompt()`

### Step 8: Prompt Generation
Location: `crates/chat-cli/src/cli/chat/mod.rs:4904`

```rust
let session_name = if let Some(ref coord) = self.coordinator {
    let coord_lock = coord.lock().await;
    if let Some(active_id) = coord_lock.active_session_id().await {
        coord_lock.get_session_name(&active_id)
    } else {
        None
    }
} else {
    self.active_session_name.clone()
};
```

**Expected**: 
- `self.coordinator` should be Some(Arc<Mutex<Coordinator>>)
- `active_id` should be "d5790330..." (hello)
- `get_session_name("d5790330...")` should return Some("hello")

## PROBLEM HYPOTHESIS

The issue is likely one of:

1. **Coordinator reference not set**: The dynamically created ChatSession doesn't have coordinator set
2. **Wrong active_session_id**: The coordinator's active_session_id is not actually "hello"'s ID
3. **Session name lookup fails**: get_session_name() can't find the session by conversation_id
4. **Timing issue**: The prompt is generated BEFORE the switch completes

## Let me check each hypothesis...
