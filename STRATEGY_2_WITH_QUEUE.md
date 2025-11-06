# Strategy 2 Implementation Plan - With Message Queue
## Background LLM Processing + Priority Queue

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                  Message Queue                          │
│  ┌──────────────────────────────────────────┐          │
│  │  Priority Queue                           │          │
│  │  - Active session messages (priority 1)  │          │
│  │  - Background session messages (priority 2)│        │
│  └──────────────────────────────────────────┘          │
└─────────────────────────────────────────────────────────┘
           │
           ▼
    ┌──────────────────────────────────────────┐
    │         LLM Processor                     │
    │  - Processes one message at a time       │
    │  - Checks for higher priority messages   │
    │  - Can be interrupted for active session │
    └──────────────────────────────────────────┘
           │
           ▼
    ┌──────────────────────────────────────────┐
    │         Session Workers                   │
    │  - Receive responses via channels        │
    │  - Display to user when active           │
    │  - Buffer when inactive                  │
    └──────────────────────────────────────────┘
```

## Message Queue Design

### Priority Levels
1. **Priority 1 (High)**: Active session messages
2. **Priority 2 (Low)**: Background session messages

### Queue Behavior
- Active session messages jump to front
- Background messages process when queue is empty
- Can interrupt background processing for active session

## Implementation

### Phase 1: Message Queue (Day 1-2, 16 hours)

#### 1.1 Message Queue Structure

```rust
// crates/chat-cli/src/cli/chat/message_queue.rs (NEW FILE)

use std::collections::VecDeque;
use tokio::sync::{Mutex, mpsc};

#[derive(Debug, Clone)]
pub struct QueuedMessage {
    pub session_id: String,
    pub message: String,
    pub priority: MessagePriority,
    pub timestamp: std::time::Instant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MessagePriority {
    High = 1,  // Active session
    Low = 2,   // Background session
}

pub struct MessageQueue {
    high_priority: VecDeque<QueuedMessage>,
    low_priority: VecDeque<QueuedMessage>,
    current_processing: Option<QueuedMessage>,
}

impl MessageQueue {
    pub fn new() -> Self {
        Self {
            high_priority: VecDeque::new(),
            low_priority: VecDeque::new(),
            current_processing: None,
        }
    }
    
    /// Add message to queue
    pub fn enqueue(&mut self, message: QueuedMessage) {
        match message.priority {
            MessagePriority::High => {
                self.high_priority.push_back(message);
            }
            MessagePriority::Low => {
                self.low_priority.push_back(message);
            }
        }
    }
    
    /// Get next message to process (high priority first)
    pub fn dequeue(&mut self) -> Option<QueuedMessage> {
        if let Some(msg) = self.high_priority.pop_front() {
            self.current_processing = Some(msg.clone());
            Some(msg)
        } else if let Some(msg) = self.low_priority.pop_front() {
            self.current_processing = Some(msg.clone());
            Some(msg)
        } else {
            None
        }
    }
    
    /// Check if should interrupt current processing
    pub fn should_interrupt(&self) -> bool {
        // Interrupt if high priority message arrives while processing low priority
        if let Some(current) = &self.current_processing {
            if current.priority == MessagePriority::Low && !self.high_priority.is_empty() {
                return true;
            }
        }
        false
    }
    
    /// Mark current message as complete
    pub fn complete_current(&mut self) {
        self.current_processing = None;
    }
    
    /// Get current processing message
    pub fn current(&self) -> Option<&QueuedMessage> {
        self.current_processing.as_ref()
    }
    
    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.high_priority.is_empty() && self.low_priority.is_empty()
    }
    
    /// Get queue stats
    pub fn stats(&self) -> QueueStats {
        QueueStats {
            high_priority_count: self.high_priority.len(),
            low_priority_count: self.low_priority.len(),
            is_processing: self.current_processing.is_some(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct QueueStats {
    pub high_priority_count: usize,
    pub low_priority_count: usize,
    pub is_processing: bool,
}
```

**File**: `message_queue.rs` (NEW)
**Lines**: ~100

#### 1.2 Queue Manager

```rust
// crates/chat-cli/src/cli/chat/queue_manager.rs (NEW FILE)

use tokio::sync::{Mutex, mpsc};
use std::sync::Arc;

pub struct QueueManager {
    queue: Arc<Mutex<MessageQueue>>,
    response_channels: Arc<Mutex<HashMap<String, mpsc::UnboundedSender<LLMResponse>>>>,
}

#[derive(Debug, Clone)]
pub enum LLMResponse {
    Chunk(String),
    ToolUse(ToolUseInfo),
    Complete,
    Error(String),
    Interrupted,
}

impl QueueManager {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(MessageQueue::new())),
            response_channels: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Submit message to queue
    pub async fn submit_message(
        &self,
        session_id: String,
        message: String,
        priority: MessagePriority,
    ) -> mpsc::UnboundedReceiver<LLMResponse> {
        let (tx, rx) = mpsc::unbounded_channel();
        
        // Register response channel
        {
            let mut channels = self.response_channels.lock().await;
            channels.insert(session_id.clone(), tx);
        }
        
        // Enqueue message
        {
            let mut queue = self.queue.lock().await;
            queue.enqueue(QueuedMessage {
                session_id,
                message,
                priority,
                timestamp: std::time::Instant::now(),
            });
        }
        
        rx
    }
    
    /// Process queue (run in background task)
    pub async fn process_queue(
        &self,
        coordinator: Arc<Mutex<MultiSessionCoordinator>>,
        os: Arc<Mutex<Os>>,
    ) {
        loop {
            // Get next message
            let message = {
                let mut queue = self.queue.lock().await;
                queue.dequeue()
            };
            
            if let Some(msg) = message {
                eprintln!("[QUEUE] Processing message from session {} (priority: {:?})", 
                    msg.session_id, msg.priority);
                
                // Process message
                self.process_message(msg, coordinator.clone(), os.clone()).await;
            } else {
                // Queue empty, wait a bit
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        }
    }
    
    async fn process_message(
        &self,
        message: QueuedMessage,
        coordinator: Arc<Mutex<MultiSessionCoordinator>>,
        os: Arc<Mutex<Os>>,
    ) {
        // Get session
        let coord = coordinator.lock().await;
        let state = coord.state.lock().await;
        let session = match state.sessions.get(&message.session_id) {
            Some(s) => s,
            None => {
                eprintln!("[QUEUE] Session {} not found", message.session_id);
                return;
            }
        };
        
        // Get response channel
        let response_tx = {
            let channels = self.response_channels.lock().await;
            channels.get(&message.session_id).cloned()
        };
        
        let Some(tx) = response_tx else {
            eprintln!("[QUEUE] No response channel for session {}", message.session_id);
            return;
        };
        
        // Send message to LLM
        let mut conversation = session.conversation.clone();
        conversation.set_next_user_message(message.message);
        
        let mut os_guard = os.lock().await;
        let conv_state = conversation.as_sendable_conversation_state(&mut *os_guard).await;
        drop(os_guard);
        
        // Stream response
        match self.stream_response(conv_state, tx.clone()).await {
            Ok(_) => {
                eprintln!("[QUEUE] Completed message from session {}", message.session_id);
            }
            Err(e) => {
                eprintln!("[QUEUE] Error processing message: {}", e);
                let _ = tx.send(LLMResponse::Error(e.to_string()));
            }
        }
        
        // Mark complete
        {
            let mut queue = self.queue.lock().await;
            queue.complete_current();
        }
    }
    
    async fn stream_response(
        &self,
        conv_state: ConversationState,
        tx: mpsc::UnboundedSender<LLMResponse>,
    ) -> Result<()> {
        // Create stream
        let mut rx = SendMessageStream::send_message(
            &client,
            conv_state,
            Arc::new(Mutex::new(None)),
            None,
        ).await?;
        
        // Stream chunks
        loop {
            // Check for interruption
            {
                let queue = self.queue.lock().await;
                if queue.should_interrupt() {
                    eprintln!("[QUEUE] Interrupting for high priority message");
                    let _ = tx.send(LLMResponse::Interrupted);
                    return Ok(());
                }
            }
            
            // Receive chunk
            match rx.recv().await {
                Some(Ok(event)) => {
                    match event {
                        ResponseEvent::AssistantText(text) => {
                            let _ = tx.send(LLMResponse::Chunk(text));
                        }
                        ResponseEvent::ToolUse(tool) => {
                            let _ = tx.send(LLMResponse::ToolUse(tool));
                        }
                        _ => {}
                    }
                }
                Some(Err(e)) => {
                    let _ = tx.send(LLMResponse::Error(e.to_string()));
                    return Err(e.into());
                }
                None => {
                    let _ = tx.send(LLMResponse::Complete);
                    return Ok(());
                }
            }
        }
    }
}
```

**File**: `queue_manager.rs` (NEW)
**Lines**: ~150

### Phase 2: Integration (Day 3, 8 hours)

#### 2.1 Add Queue to Coordinator

```rust
// crates/chat-cli/src/cli/chat/coordinator.rs

pub struct MultiSessionCoordinator {
    // ... existing fields ...
    
    /// Message queue manager
    queue_manager: Arc<QueueManager>,
}

impl MultiSessionCoordinator {
    pub fn new(config: CoordinatorConfig) -> Self {
        let queue_manager = Arc::new(QueueManager::new());
        
        // Spawn queue processor
        let queue_clone = queue_manager.clone();
        tokio::spawn(async move {
            queue_clone.process_queue(coord_arc, os_arc).await;
        });
        
        Self {
            // ... existing fields ...
            queue_manager,
        }
    }
}
```

**File**: `coordinator.rs`
**Lines**: ~20 modified

#### 2.2 Modify ChatSession to Use Queue

```rust
// crates/chat-cli/src/cli/chat/mod.rs

impl ChatSession {
    async fn handle_input(&mut self, input: String) -> Result<ChatState> {
        // Determine priority based on active session
        let priority = if self.is_active_session() {
            MessagePriority::High
        } else {
            MessagePriority::Low
        };
        
        // Submit to queue
        let mut response_rx = self.coordinator
            .lock().await
            .queue_manager
            .submit_message(
                self.conversation.conversation_id().to_string(),
                input,
                priority,
            ).await;
        
        // Stream response
        while let Some(response) = response_rx.recv().await {
            match response {
                LLMResponse::Chunk(text) => {
                    write!(self.stdout, "{}", text)?;
                }
                LLMResponse::Complete => {
                    break;
                }
                LLMResponse::Interrupted => {
                    // Save partial response
                    self.conversation.save_partial_response(&buffer)?;
                    return Ok(ChatState::PromptUser);
                }
                LLMResponse::Error(e) => {
                    return Err(ChatError::Custom(e.into()));
                }
                _ => {}
            }
        }
        
        Ok(ChatState::PromptUser)
    }
    
    fn is_active_session(&self) -> bool {
        if let Some(ref coord) = self.coordinator {
            let coord = coord.try_lock().ok()?;
            let state = coord.state.try_lock().ok()?;
            state.active_session_id.as_ref() == Some(&self.conversation.conversation_id().to_string())
        } else {
            true // Default to high priority if no coordinator
        }
    }
}
```

**File**: `mod.rs`
**Lines**: ~40 modified

### Phase 3: Testing (Day 4, 8 hours)

#### Test Scenarios
1. **Priority Test**: Submit background message, then active message
   - Verify active message processes first
2. **Interruption Test**: Background processing interrupted by active message
3. **Queue Stats**: Monitor queue depth and processing
4. **Multiple Sessions**: All sessions can submit messages

## Updated Timeline

### Day 1-2: Message Queue (16 hours)
- Implement MessageQueue
- Implement QueueManager
- Unit tests

### Day 3: Integration (8 hours)
- Add to Coordinator
- Modify ChatSession
- Integration tests

### Day 4: Testing & Polish (8 hours)
- Test priority handling
- Test interruption
- Performance testing
- Documentation

**Total: 4 days (32 hours)**

## Code Changes Summary

### New Files
- `message_queue.rs`: ~100 lines
- `queue_manager.rs`: ~150 lines

### Modified Files
- `coordinator.rs`: ~20 lines
- `mod.rs`: ~40 lines
- `conversation.rs`: ~15 lines (partial response)

**Total: ~325 lines**

## Benefits

### With Queue System
✅ Active session always prioritized
✅ Background sessions can queue messages
✅ Interruption for high priority
✅ Fair processing when idle
✅ Queue stats for monitoring

### Queue Behavior Examples

**Scenario 1: Active Session Priority**
```
Queue: [Background-1, Background-2]
User switches to Session-1, sends message
Queue: [Active-1, Background-1, Background-2]
Processing: Active-1 (jumps to front)
```

**Scenario 2: Interruption**
```
Processing: Background-1 (streaming)
User sends message in active session
Action: Interrupt Background-1, process Active-1
Background-1: Queued for retry
```

**Scenario 3: Fair Processing**
```
Queue: [Background-1, Background-2, Background-3]
No active messages
Processing: Background-1, then Background-2, then Background-3
```

## Risk Assessment

### Low Risk
- Queue is isolated component
- Easy to test independently
- Can disable if issues arise

### Potential Issues
1. **Queue starvation**: Background messages never process
   - **Mitigation**: Process background when active idle
2. **Memory growth**: Queue grows unbounded
   - **Mitigation**: Add max queue size, drop old messages
3. **Interruption handling**: Partial state corruption
   - **Mitigation**: Save state before interruption

## Success Criteria

- [ ] Active session messages process immediately
- [ ] Background messages queue correctly
- [ ] Interruption works without corruption
- [ ] Queue stats accurate
- [ ] No memory leaks
- [ ] Performance acceptable (<100ms overhead)

## Conclusion

Adding the queue system:
- **Adds 2 days** to timeline (4 days total)
- **Adds ~250 lines** of code
- **Provides proper prioritization**
- **Enables true background processing**

**This is the right approach. The queue ensures active sessions are always responsive while allowing background work.**
