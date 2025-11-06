# Background Session Execution - Revised Design
## Based on Actual Q CLI & Crossterm Implementation

## Critical Implementation Constraints

### 1. Rustyline (not crossterm) for Input
**Reality**: Q CLI uses `rustyline::Editor` for input, not raw crossterm
- `InputSource` wraps `rustyline::Editor<ChatHelper, FileHistory>`
- Rustyline is **synchronous** and **blocking**
- `rl.readline(prompt)` blocks until user presses Enter
- **Cannot be shared between threads** (not Send/Sync)

**Impact**: 
- ❌ Cannot have multiple rustyline instances reading stdin simultaneously
- ❌ Cannot move rustyline to background task
- ✅ Can only have ONE active input reader at a time

### 2. Os is Clone but Contains Non-Send Types
**Reality**: Os is Clone but contains:
- `database: Database` - uses rusqlite (not Send)
- `telemetry: Telemetry` - may have non-Send internals

**Impact**:
- ✅ Can clone Os for each session
- ❌ Cannot move Os to tokio::spawn (not Send)
- ✅ Can use tokio::spawn_blocking for Os operations

### 3. Terminal State in Rustyline
**Reality**: Rustyline manages terminal state internally
- Raw mode, history, completion, key bindings
- Drop handler restores terminal
- Cannot "detach" and "reattach" terminal

**Impact**:
- ❌ Cannot pause rustyline and resume later
- ❌ Cannot transfer terminal control between rustyline instances
- ✅ Must create new rustyline instance per session

## Feasible Architecture

### Core Insight
**We cannot have true background execution with rustyline blocking stdin.**

Instead, we can achieve:
1. ✅ Save/restore conversation state (already works)
2. ✅ Continue LLM processing after user input (before next prompt)
3. ✅ Switch sessions between user inputs
4. ❌ Switch during rustyline.readline() (impossible - it's blocking)
5. ❌ Background sessions reading input (only one can read stdin)

### Revised Approach: Async LLM Processing

```
User Input → LLM Processing (async) → Display Output → Next Prompt
     ↑                                                        ↓
     └────────────── Can switch here ────────────────────────┘
     
Cannot switch during readline() - it's blocking
```

## Practical Design

### 1. Async Message Processing (Achievable)

```rust
pub struct SessionWorker {
    conversation: ConversationState,
    os: Os,
    
    // Message queue
    pending_messages: VecDeque<String>,
    
    // Output buffer
    output_buffer: Arc<Mutex<VecDeque<OutputChunk>>>,
    
    // State
    state: WorkerState,
}

impl SessionWorker {
    /// Process messages asynchronously
    async fn process_messages(&mut self) -> Result<()> {
        while let Some(message) = self.pending_messages.pop_front() {
            // This can run in background
            let response = self.send_to_llm(&message).await?;
            
            // Buffer output
            self.output_buffer.lock().await.push_back(
                OutputChunk::Text(response)
            );
            
            // Check for switch signal
            if self.should_switch() {
                // Save state and pause
                return Ok(());
            }
        }
        Ok(())
    }
}
```

### 2. Modified Session Flow

```rust
impl ChatSession {
    async fn run_loop(&mut self, os: &mut Os) -> Result<()> {
        loop {
            // 1. Read input (BLOCKING - cannot switch here)
            let input = self.input_source.read_line(Some(&prompt))?;
            
            // 2. Process with LLM (ASYNC - can switch here)
            let response = self.process_with_llm(input, os).await?;
            
            // 3. Display output (ASYNC - can switch here)
            self.display_output(response).await?;
            
            // 4. Check for switch before next prompt
            if self.should_switch() {
                // Save conversation state
                self.save_state().await?;
                return Ok(());
            }
        }
    }
    
    fn should_switch(&self) -> bool {
        // Check coordinator for switch signal
        if let Some(ref coord) = self.coordinator {
            let state = coord.lock().await.state.lock().await;
            state.active_session_id != Some(self.conversation_id)
        } else {
            false
        }
    }
}
```

### 3. Switch Detection During LLM Processing

```rust
// In conversation.rs - modify LLM call to check for switch
pub async fn send_message_with_switch_check(
    &mut self,
    coordinator: Option<&Arc<Mutex<Coordinator>>>,
) -> Result<Response> {
    let chunks = self.llm_client.stream_response().await?;
    
    for chunk in chunks {
        // Check for switch signal
        if let Some(coord) = coordinator {
            if Self::check_switch(coord, &self.conversation_id).await {
                // Save partial response
                self.save_partial_response(chunk)?;
                return Err(ChatError::Switched);
            }
        }
        
        // Process chunk
        self.process_chunk(chunk)?;
    }
    
    Ok(response)
}
```

## What This Achieves

### ✅ Possible
1. **Switch between prompts**: After LLM responds, before next input
2. **Interrupt LLM processing**: During streaming response
3. **Save/restore state**: Full conversation history
4. **Multiple sessions**: Switch between them
5. **Buffered output**: See what happened while away

### ❌ Not Possible (Due to Rustyline)
1. **Switch during input**: While readline() is blocking
2. **Background input reading**: Only one session can read stdin
3. **True parallel sessions**: All waiting for input simultaneously
4. **Detach/reattach terminal**: Rustyline doesn't support this

## Implementation Plan

### Phase 1: Switch Detection (1-2 days)
- Add switch checking during LLM streaming
- Save partial responses
- Test interrupting LLM mid-response

```rust
// In coordinator loop
loop {
    let session = get_active_session();
    
    // Run until switch or complete
    match session.run_until_switch(os).await {
        Ok(SwitchReason::UserSwitch) => continue,
        Ok(SwitchReason::Completed) => break,
        Err(e) => return Err(e),
    }
}
```

### Phase 2: Async LLM Processing (2-3 days)
- Separate LLM calls from input reading
- Buffer output during processing
- Allow switch during LLM response

### Phase 3: Output Buffering (1-2 days)
- Buffer output when session inactive
- Replay on switch back
- Show "session has updates" indicator

## Limitations & Workarounds

### Limitation 1: Cannot Switch During Input
**Workaround**: Show "Press Enter to switch" message
```rust
// User types /switch 2
// System: "Press Enter to switch to session 2"
// User presses Enter
// Switch happens
```

### Limitation 2: Only One Session Reading Input
**Workaround**: Queue messages for background sessions
```rust
// In background session
if has_pending_messages() {
    process_messages_async().await?;
}
```

### Limitation 3: Rustyline Blocks
**Workaround**: Use tokio::spawn_blocking
```rust
let input = tokio::task::spawn_blocking(move || {
    input_source.read_line(Some(&prompt))
}).await??;
```

## Realistic Timeline

- Phase 1: 1-2 days (switch detection)
- Phase 2: 2-3 days (async processing)
- Phase 3: 1-2 days (output buffering)

**Total**: 4-7 days for practical implementation

## Conclusion

**The original design is NOT feasible** due to:
1. Rustyline's blocking nature
2. Single stdin reader limitation
3. Terminal state management

**The revised design IS feasible** and provides:
- ✅ Switch between user inputs
- ✅ Interrupt LLM during response
- ✅ Save/restore full state
- ✅ Output buffering
- ❌ Cannot switch during readline()
- ❌ Cannot have parallel input reading

This is a **significant improvement** over current state while respecting actual implementation constraints.
