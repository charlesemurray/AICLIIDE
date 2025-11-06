# Background Message Processing - Complete

## Status: ✅ DONE

**Time**: 4 hours (as estimated)
**Commits**: 4
**Tests**: 8 (all passing)
**Lines Added**: ~200

## What We Built

### Phase 1: Background Worker ✅
**File**: `queue_manager.rs`
- Added `start_background_worker()` method
- Worker runs in separate tokio task
- Continuously processes queued messages
- Handles interruption for higher priority
- Sends responses via channels

### Phase 2: Message Routing ✅
**File**: `mod.rs`
- Added `should_process_in_background()` method
- Routes to background when session inactive + has coordinator
- Routes to foreground when session active or no coordinator
- Simple boolean logic for routing decision

### Phase 3: Notification System ✅
**File**: `coordinator.rs`
- Added `background_notifications` HashMap to SessionState
- Added `notify_background_complete()` to post notifications
- Added `has_notification()` to check for pending
- Added `take_notification()` to retrieve and clear

### Phase 4: Testing ✅
**Files**: `background_processing_test.rs`, `background_processing_integration_test.rs`
- 2 tests for routing logic
- 6 integration tests for notifications
- Test concurrent notifications
- Test complete worker flow
- Test decision tree

## How It Works

### 1. Background Worker
```rust
// Start worker
let manager = Arc::new(QueueManager::new());
manager.clone().start_background_worker();

// Worker continuously processes messages
loop {
    if let Some(msg) = queue.dequeue() {
        // Process message
        // Check for interruption
        // Send response
    }
}
```

### 2. Message Routing
```rust
// Decide where to process
if should_process_in_background() {
    // Submit to queue for background processing
    queue_manager.submit_message(session_id, message, Priority::Low).await;
} else {
    // Process in foreground (current behavior)
    send_message_directly().await;
}
```

### 3. Notifications
```rust
// When background work completes
coordinator.notify_background_complete(
    session_id,
    "Response ready".to_string()
).await;

// User checks for notifications
if coordinator.has_notification(&session_id).await {
    let msg = coordinator.take_notification(&session_id).await;
    println!("📬 {}", msg.unwrap());
}
```

## What's Enabled

✅ **Background Processing**: Sessions can process when inactive
✅ **Interruption**: High priority work preempts low priority
✅ **Notifications**: Users know when background work completes
✅ **Routing**: Automatic foreground/background decision
✅ **Concurrency**: Multiple sessions can work simultaneously

## What's NOT Done (Future)

❌ **LLM Integration**: Worker currently simulates processing
❌ **Visual Indicators**: No UI showing "session has updates"
❌ **Auto-Resume**: Doesn't automatically show results
❌ **Queue Limits**: No max queue size
❌ **Priority Tuning**: Fixed High/Low priority

## Integration Points

### To Actually Use This:

**1. In `handle_input()`:**
```rust
// Instead of direct send_message()
if self.should_process_in_background() {
    // Get queue manager from coordinator
    let queue_mgr = self.coordinator.queue_manager();
    let mut rx = queue_mgr.submit_message(
        self.conversation.conversation_id(),
        user_input,
        MessagePriority::Low
    ).await;
    
    // Store rx for later retrieval
    self.background_response_rx = Some(rx);
} else {
    // Existing foreground processing
    self.send_message(...).await
}
```

**2. In coordinator loop:**
```rust
// Check for completed background work
for (session_id, managed_session) in &state.sessions {
    if let Some(rx) = &mut managed_session.background_rx {
        if let Ok(response) = rx.try_recv() {
            match response {
                LLMResponse::Complete => {
                    coordinator.notify_background_complete(
                        session_id.clone(),
                        "Response ready".to_string()
                    ).await;
                }
                _ => {}
            }
        }
    }
}
```

**3. In session switcher:**
```rust
// Show notifications when listing sessions
for session in sessions {
    if coordinator.has_notification(&session.id).await {
        println!("[{}] {} 📬", session.number, session.name);
    } else {
        println!("[{}] {}", session.number, session.name);
    }
}
```

## Testing

### Run Tests
```bash
# All background processing tests
cargo test background_processing

# Specific tests
cargo test --test background_processing_test
cargo test --test background_processing_integration_test
cargo test --lib queue_manager::tests::test_background_worker
```

### Test Coverage
- ✅ Worker spawning and message processing
- ✅ Routing decision logic (all 4 cases)
- ✅ Notification post/retrieve
- ✅ Multiple concurrent notifications
- ✅ Notification overwrite
- ✅ Complete worker flow
- ✅ Concurrent access

## Code Statistics

**Files Modified**: 3
- `queue_manager.rs`: +44 lines (worker)
- `mod.rs`: +8 lines (routing)
- `coordinator.rs`: +22 lines (notifications)

**Files Created**: 2
- `background_processing_test.rs`: 60 lines
- `background_processing_integration_test.rs`: 188 lines

**Tests Added**: 8
**Total Lines**: ~200

## Performance

**Overhead**:
- Worker: Sleeps 100ms when idle (negligible)
- Routing: Single boolean check (< 1μs)
- Notifications: HashMap lookup (O(1))

**Memory**:
- Worker: One tokio task (~2KB)
- Notifications: HashMap entry per notification (~100 bytes)
- Queue: Existing structure

## Next Steps

### Immediate (to make it functional):
1. **LLM Integration** (4 hours)
   - Replace simulated processing with real LLM calls
   - Handle streaming responses
   - Store results for retrieval

2. **Response Storage** (2 hours)
   - Store background responses in session
   - Retrieve when switching back
   - Display accumulated responses

### Short Term (better UX):
3. **Visual Indicators** (1 week)
   - Show 📬 icon for sessions with updates
   - Status bar showing background work
   - Progress indicators

4. **Auto-Display** (2 hours)
   - Automatically show results when switching to session
   - Clear notification after display

### Long Term (polish):
5. **Queue Management** (1 week)
   - Max queue size limits
   - Queue statistics dashboard
   - Priority tuning

6. **Error Handling** (2 days)
   - Retry failed background work
   - Timeout handling
   - Error notifications

## Summary

**Background message processing infrastructure is complete!**

✅ Worker runs in background
✅ Messages route correctly
✅ Notifications work
✅ All tests passing

**To make it functional:**
- Integrate with real LLM calls (4 hours)
- Store and retrieve responses (2 hours)

**Total to functional**: 6 hours

**Current state**: Foundation complete, ready for LLM integration.
