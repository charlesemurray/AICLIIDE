# Background Processing - Logging Guide

## Overview

Comprehensive logging has been added to identify and debug issues in the background processing system.

## Log Prefixes

All logs use prefixes to identify the component:

- `[WORKER]` - Background worker thread
- `[QUEUE]` - Queue manager operations
- `[ROUTING]` - Message routing decisions
- `[SESSION]` - Session state checks
- `[SPAWN]` - Session startup and notification display
- `[STORE]` - Response storage operations
- `[RETRIEVE]` - Response retrieval operations
- `[NOTIFY]` - Notification posting
- `[COORDINATOR]` - Coordinator operations

## Key Log Points

### 1. Worker Startup
```
[WORKER] Starting background worker thread
[COORDINATOR] Background worker started
```
**What to check**: Worker should start when coordinator is created

### 2. Message Submission
```
[ROUTING] Session abc123 is inactive, routing to background
[ROUTING] Submitting message to background queue (msg_len: 42)
[QUEUE] Submitting message from session abc123 (priority: Low, msg_len: 42)
[QUEUE] Registered response channel for session abc123 (total channels: 1)
[QUEUE] Enqueued message for session abc123 (queue size: high=0, low=1)
[ROUTING] Successfully submitted to background, returning to prompt
```
**What to check**: 
- Session correctly identified as inactive
- Message enqueued successfully
- Channel registered

### 3. Worker Processing
```
[WORKER] Iteration 1 - Queue stats: high=0, low=1
[WORKER] Processing message from session abc123 (waited: 123ms, priority: Low)
[WORKER] Simulating LLM processing for session abc123
[WORKER] Sending response to session abc123 (456 bytes)
[WORKER] Completed processing for session abc123 (sent 12 chunks)
```
**What to check**:
- Worker picks up message
- Wait time reasonable
- Response sent successfully
- Chunk count matches expected

### 4. Response Storage
```
[STORE] Stored background response for session abc123 (123 bytes, 1 total responses)
```
**What to check**:
- Responses being stored
- Byte count reasonable
- Total count incrementing

### 5. Notification
```
[NOTIFY] Background work complete for session abc123
```
**What to check**:
- Notification posted after completion

### 6. Session Switch
```
[SPAWN] Starting session abc123
[SPAWN] Checking for notifications for session abc123
[SPAWN] Found notification for session abc123: Background work complete
[RETRIEVE] Retrieved 1 background responses for session abc123
[SPAWN] Displaying response 1 (456 bytes)
```
**What to check**:
- Notification found
- Responses retrieved
- Responses displayed

### 7. Session State Checks
```
[SESSION] is_active_session for abc123: false (active_id: Some("xyz789"))
[ROUTING] should_process_in_background: true (has_coordinator: true, is_active: false)
```
**What to check**:
- Active session ID correct
- Routing decision correct

## Common Issues and Log Patterns

### Issue: Messages Not Processing

**Look for:**
```
[QUEUE] Enqueued message for session abc123 (queue size: high=0, low=1)
[WORKER] Iteration 100 - Queue stats: high=0, low=1
```
**Problem**: Message enqueued but worker not processing
**Check**: Worker thread still running? Lock contention?

### Issue: No Response Channel

**Look for:**
```
[WORKER] ERROR: No response channel for session abc123
```
**Problem**: Channel not registered or removed
**Check**: Was channel registered in submit_message?

### Issue: Failed to Send Response

**Look for:**
```
[WORKER] ERROR: Failed to send chunk 5 to session abc123
[WORKER] ERROR: Failed to send completion to session abc123
```
**Problem**: Receiver dropped or channel closed
**Check**: Is session still alive? Did it exit?

### Issue: Session Not Found

**Look for:**
```
[STORE] ERROR: Session abc123 not found, cannot store response
[RETRIEVE] ERROR: Session abc123 not found
```
**Problem**: Session doesn't exist in coordinator
**Check**: Was session created? Session ID correct?

### Issue: No Notification

**Look for:**
```
[SPAWN] No notifications for session abc123
```
**Problem**: Notification not posted or already consumed
**Check**: Was notify_background_complete called?

### Issue: Lock Contention

**Look for:**
```
[SESSION] Could not lock coordinator for abc123, defaulting to active
[SESSION] Could not lock state for abc123, defaulting to active
[SPAWN] ERROR: Could not lock coordinator for session abc123
```
**Problem**: Coordinator or state locked by another thread
**Check**: Deadlock? Long-running operation holding lock?

## Debugging Workflow

### 1. Verify Worker Started
```bash
# Look for worker startup
grep "\[WORKER\] Starting" stderr.log
grep "\[COORDINATOR\] Background worker started" stderr.log
```

### 2. Trace Message Flow
```bash
# Follow a specific session
grep "session-abc123" stderr.log | grep -E "\[ROUTING\]|\[QUEUE\]|\[WORKER\]"
```

### 3. Check Queue State
```bash
# Look for queue statistics
grep "\[WORKER\] Iteration.*Queue stats" stderr.log | tail -10
```

### 4. Find Errors
```bash
# All errors
grep "ERROR:" stderr.log

# By component
grep "\[WORKER\] ERROR:" stderr.log
grep "\[STORE\] ERROR:" stderr.log
```

### 5. Trace Session Lifecycle
```bash
# Session creation to completion
grep "session-abc123" stderr.log | grep -E "\[SPAWN\]|\[ROUTING\]|\[WORKER\]|\[NOTIFY\]"
```

## Performance Monitoring

### Queue Depth
```bash
# Monitor queue growth
grep "queue size:" stderr.log | tail -20
```

### Processing Time
```bash
# Check wait times
grep "waited:" stderr.log
```

### Response Sizes
```bash
# Check response sizes
grep "bytes" stderr.log | grep -E "\[WORKER\]|\[STORE\]"
```

## Log Levels

Current implementation uses `eprintln!` for all logs (stderr).

**To reduce verbosity**, comment out:
- Worker iteration logs (every 100 iterations)
- Detailed routing logs
- Session state check logs

**To increase verbosity**, add:
- Lock acquisition/release logs
- Channel send/receive logs
- Queue operation details

## Example: Full Successful Flow

```
[COORDINATOR] Background worker started
[WORKER] Starting background worker thread

[ROUTING] Session abc123 is inactive, routing to background
[ROUTING] Submitting message to background queue (msg_len: 42)
[QUEUE] Submitting message from session abc123 (priority: Low, msg_len: 42)
[QUEUE] Registered response channel for session abc123 (total channels: 1)
[QUEUE] Enqueued message for session abc123 (queue size: high=0, low=1)
[ROUTING] Successfully submitted to background, returning to prompt

[WORKER] Processing message from session abc123 (waited: 50ms, priority: Low)
[WORKER] Simulating LLM processing for session abc123
[WORKER] Sending response to session abc123 (456 bytes)
[WORKER] Completed processing for session abc123 (sent 12 chunks)

[NOTIFY] Background work complete for session abc123
[STORE] Stored background response for session abc123 (123 bytes, 1 total responses)

[SPAWN] Starting session abc123
[SPAWN] Checking for notifications for session abc123
[SPAWN] Found notification for session abc123: Background work complete
[RETRIEVE] Retrieved 1 background responses for session abc123
[SPAWN] Displaying response 1 (456 bytes)
```

## Troubleshooting Checklist

- [ ] Worker started? (Check for `[WORKER] Starting`)
- [ ] Message enqueued? (Check for `[QUEUE] Enqueued`)
- [ ] Channel registered? (Check for `[QUEUE] Registered`)
- [ ] Worker processing? (Check for `[WORKER] Processing`)
- [ ] Response sent? (Check for `[WORKER] Completed`)
- [ ] Notification posted? (Check for `[NOTIFY]`)
- [ ] Response stored? (Check for `[STORE]`)
- [ ] Notification displayed? (Check for `[SPAWN] Found`)
- [ ] Response displayed? (Check for `[SPAWN] Displaying`)

## Summary

With this logging:
- Every decision point is logged
- Every error is logged with context
- Every state transition is logged
- Full message lifecycle is traceable
- Performance metrics are available

**All logs go to stderr, so redirect to file for analysis:**
```bash
q chat 2> debug.log
```
