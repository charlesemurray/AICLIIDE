# Multi-Session Implementation Plan

## Goal
Make coordinator own ChatSessions, enable true session switching with background sessions continuing to run.

## Current Status (2025-11-05)

### What Works
- ✅ Coordinator created at startup
- ✅ Sessions created with nice random names (e.g., "swift-fox")
- ✅ `/switch` command updates coordinator's active_session_id
- ✅ Conversations stored in coordinator's ManagedSession
- ✅ OutputBuffer infrastructure exists for background sessions
- ✅ Session indicator shows active sessions

### What's Broken
- ❌ Prompt shows wrong session name after switch
- ❌ Messages may go to wrong conversation after switch
- ❌ ChatSession created outside coordinator
- ❌ Only one ChatSession exists (should be one per session)
- ❌ Background sessions don't actually run

### Root Cause
ChatSession is created independently and just references the coordinator. It should be owned BY the coordinator.

## Phase 1: Minimal Working Switch (Priority: HIGH)

**Goal**: Get switching working correctly with current architecture.

### Step 1.1: Store active ChatSession reference in coordinator
**File**: `crates/chat-cli/src/cli/chat/coordinator.rs`

```rust
pub struct MultiSessionCoordinator {
    // ... existing fields ...
    
    /// Reference to the currently active ChatSession
    active_chat_session: Option<Arc<Mutex<ChatSession>>>,
}
```

**Changes**:
- Add field to struct
- Initialize as None in new()
- Add method: `set_active_chat_session(&mut self, session: Arc<Mutex<ChatSession>>)`

### Step 1.2: Register ChatSession with coordinator
**File**: `crates/chat-cli/src/cli/chat/mod.rs` (ChatArgs::execute)

**Current** (line ~830):
```rust
if let Some(coord) = coordinator {
    let coord_arc = Arc::new(Mutex::new(coord));
    session.coordinator = Some(coord_arc.clone());
    // ... create initial session ...
}
```

**Change to**:
```rust
if let Some(coord) = coordinator {
    let coord_arc = Arc::new(Mutex::new(coord));
    session.coordinator = Some(coord_arc.clone());
    
    // Register this ChatSession as active
    let session_arc = Arc::new(Mutex::new(session));
    coord_arc.lock().await.set_active_chat_session(session_arc.clone());
    
    // ... create initial session ...
    
    // Continue with the registered session
    let mut session = session_arc.lock().await;
}
```

### Step 1.3: Implement proper conversation switching
**File**: `crates/chat-cli/src/cli/chat/mod.rs` (ChatState::SwitchSession handler)

**Current** (line ~1526):
```rust
ChatState::SwitchSession { target_id } => {
    // Get target session and swap it in
    if let Some(ref coord) = self.coordinator {
        let coord_lock = coord.lock().await;
        if let Some(managed_session) = coord_lock.get_managed_session(&target_id).await {
            drop(coord_lock);
            self.switch_conversation(managed_session.conversation, managed_session.display.name);
            Ok(ChatState::PromptUser { skip_printing_tools: false })
        } else {
            Err(ChatError::Custom(format!("Session not found: {}", target_id).into()))
        }
    } else {
        Err(ChatError::Custom("No coordinator available".into()))
    }
},
```

**Change to**:
```rust
ChatState::SwitchSession { target_id } => {
    if let Some(ref coord) = self.coordinator {
        // Save current conversation back to coordinator
        let current_id = self.conversation.conversation_id().to_string();
        {
            let mut coord_lock = coord.lock().await;
            if let Some(current_session) = coord_lock.get_managed_session_mut(&current_id).await {
                current_session.conversation = self.conversation.clone();
            }
        }
        
        // Load target conversation
        let coord_lock = coord.lock().await;
        if let Some(target_session) = coord_lock.get_managed_session(&target_id).await {
            let new_conversation = target_session.conversation.clone();
            let new_name = target_session.display.name.clone();
            drop(coord_lock);
            
            self.switch_conversation(new_conversation, new_name);
            Ok(ChatState::PromptUser { skip_printing_tools: false })
        } else {
            Err(ChatError::Custom(format!("Session not found: {}", target_id).into()))
        }
    } else {
        Err(ChatError::Custom("No coordinator available".into()))
    }
},
```

### Step 1.4: Add get_managed_session_mut method
**File**: `crates/chat-cli/src/cli/chat/coordinator.rs`

```rust
/// Get a mutable reference to a managed session
pub async fn get_managed_session_mut(&mut self, conversation_id: &str) -> Option<&mut ManagedSession> {
    let mut state = self.state.lock().await;
    state.sessions.get_mut(conversation_id)
}
```

### Step 1.5: Ensure active_session_name is set on initial session
**File**: `crates/chat-cli/src/cli/chat/mod.rs` (ChatArgs::execute)

After creating initial session in coordinator:
```rust
// Set the active session name for prompt display
session.active_session_name = Some(session_name.clone());
```

**Result**: Switching works correctly, prompt shows right name, conversations preserved.

**Estimated Time**: 2-3 hours

---

## Phase 2: Multiple ChatSession Support (Priority: MEDIUM)

**Goal**: Each session has its own ChatSession, background sessions can run.

### Step 2.1: Add ChatSession to ManagedSession
**File**: `crates/chat-cli/src/cli/chat/managed_session.rs`

```rust
pub struct ManagedSession {
    // ... existing fields ...
    
    /// The ChatSession running this conversation
    /// None for sessions that haven't been started yet
    pub chat_session: Option<Arc<Mutex<ChatSession>>>,
}
```

### Step 2.2: Create ChatSession when creating session
**File**: `crates/chat-cli/src/cli/chat/coordinator.rs` (create_session method)

Instead of just creating ConversationState, also create ChatSession:
```rust
pub async fn create_session(
    &mut self,
    config: SessionConfig,
    context: SessionContext,
) -> Result<String> {
    // ... existing validation ...
    
    // Create ConversationState
    let conversation = ConversationState::new(...).await;
    
    // Create ChatSession for this conversation
    let chat_session = ChatSession::new(
        &context.os,
        &context.conversation_id,
        context.agents,
        None, // no initial input
        // ... other params ...
    ).await?;
    
    let session = ManagedSession {
        display,
        conversation,
        conversation_id: context.conversation_id.clone(),
        state: SessionState::Active,
        output_buffer: buffer,
        chat_session: Some(Arc::new(Mutex::new(chat_session))),
        task_handle: None,
        last_error: None,
        metadata: SessionMetadata { ... },
    };
    
    // ... rest of method ...
}
```

### Step 2.3: Implement coordinator.run() main loop
**File**: `crates/chat-cli/src/cli/chat/coordinator.rs`

```rust
impl MultiSessionCoordinator {
    /// Main execution loop - runs active session, processes background sessions
    pub async fn run(&mut self, os: &mut Os) -> Result<()> {
        loop {
            // Get active session
            let active_id = self.active_session_id().await
                .ok_or_else(|| eyre::eyre!("No active session"))?;
            
            let mut state = self.state.lock().await;
            let active_session = state.sessions.get_mut(&active_id)
                .ok_or_else(|| eyre::eyre!("Active session not found"))?;
            
            // Run one tick of active session
            if let Some(ref chat_session) = active_session.chat_session {
                let mut session = chat_session.lock().await;
                
                match session.next(os).await {
                    Ok(()) => continue,
                    Err(ChatError::Custom(msg)) if msg.starts_with("SWITCH_SESSION:") => {
                        // Handle switch
                        let target_id = msg.strip_prefix("SWITCH_SESSION:").unwrap();
                        self.switch_session(target_id).await?;
                        continue;
                    },
                    Err(e) => return Err(e.into()),
                }
            }
            
            // TODO: Process background sessions
        }
    }
}
```

### Step 2.4: Refactor ChatArgs::execute to use coordinator.run()
**File**: `crates/chat-cli/src/cli/chat/mod.rs`

**Current**:
```rust
session.spawn(os).await.map(|_| ExitCode::SUCCESS)
```

**Change to**:
```rust
if let Some(ref mut coord) = coordinator {
    coord.run(os).await.map(|_| ExitCode::SUCCESS)
} else {
    // Fallback to single session mode
    session.spawn(os).await.map(|_| ExitCode::SUCCESS)
}
```

### Step 2.5: Implement background session processing
**File**: `crates/chat-cli/src/cli/chat/coordinator.rs`

```rust
/// Process background sessions (non-blocking)
async fn tick_background_sessions(&mut self) {
    let mut state = self.state.lock().await;
    
    for (id, session) in state.sessions.iter_mut() {
        // Skip active session
        if Some(id.clone()) == state.active_session_id {
            continue;
        }
        
        // Process background session
        if let Some(ref chat_session) = session.chat_session {
            if session.state == SessionState::Processing {
                // Run one step, buffer output
                // TODO: Implement non-blocking tick
            }
        }
    }
}
```

**Result**: True multi-session with background execution.

**Estimated Time**: 1-2 days

---

## Phase 3: Polish & Optimization (Priority: LOW)

### Step 3.1: Implement session pause/resume
- Pause background sessions when not needed
- Resume when switched to

### Step 3.2: Add session limits and cleanup
- Max active sessions
- Auto-archive old sessions
- Memory management

### Step 3.3: Improve session indicator
- Show background session activity
- Display buffered output count

**Estimated Time**: 1 day

---

## Testing Plan

### Phase 1 Tests
1. Create session, switch to it, verify prompt shows correct name
2. Send message in session A, switch to B, send message, switch back to A, verify history
3. Create 3 sessions, switch between them randomly, verify no data loss

### Phase 2 Tests
1. Create session, let it run in background while using another
2. Switch back, verify buffered output is displayed
3. Multiple sessions processing simultaneously

### Phase 3 Tests
1. Create max sessions, verify limit enforced
2. Archive old sessions, verify they're saved
3. Load archived session, verify history restored

---

## Migration Path

### For Users
- Phase 1: Transparent, just works better
- Phase 2: New capability (background sessions)
- Phase 3: Better performance and management

### For Developers
- Phase 1: Minimal changes, mostly in coordinator
- Phase 2: Larger refactor but builds on Phase 1
- Phase 3: Additive features

---

## Rollback Plan

If Phase 2 causes issues:
- Keep Phase 1 changes (they're stable)
- Disable background execution
- Fall back to single active session

---

## Success Criteria

### Phase 1
- [ ] Prompt shows correct session name after switch
- [ ] Messages go to correct conversation
- [ ] No crashes when switching
- [ ] Session history preserved across switches

### Phase 2
- [ ] Multiple sessions can exist simultaneously
- [ ] Background sessions buffer output
- [ ] Switching displays buffered output
- [ ] No performance degradation

### Phase 3
- [ ] Session limits enforced
- [ ] Old sessions archived automatically
- [ ] Memory usage stays reasonable
- [ ] Session indicator shows useful info

---

## Notes

- The OutputBuffer infrastructure already exists and is well-designed
- ManagedSession already has SessionState enum for Active/WaitingForInput/Processing
- The architecture was designed for this - we just need to use it properly
- Phase 1 is the critical path - get it working first
