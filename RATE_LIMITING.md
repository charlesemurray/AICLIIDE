# Background Processing Rate Limiting

## Problem

Multiple sessions processing in background could overwhelm the LLM backend API, causing throttling and failures.

## Solution

Semaphore-based rate limiting with multiple workers processing messages in parallel within controlled limits.

## Architecture

```
                    ┌─────────────────────────────────┐
                    │     Priority Queue              │
                    │  (High Priority / Low Priority) │
                    └─────────────────────────────────┘
                                  │
                    ┌─────────────┴─────────────┐
                    │                           │
                    ▼                           ▼
            ┌───────────────┐          ┌───────────────┐
            │   Worker 0    │          │   Worker 1    │
            └───────────────┘          └───────────────┘
                    │                           │
                    ▼                           ▼
            ┌───────────────┐          ┌───────────────┐
            │ Acquire Permit│          │ Acquire Permit│
            │  (Semaphore)  │          │  (Semaphore)  │
            └───────────────┘          └───────────────┘
                    │                           │
                    ▼                           ▼
            ┌───────────────┐          ┌───────────────┐
            │  LLM Stream 1 │          │  LLM Stream 2 │
            └───────────────┘          └───────────────┘
                    │                           │
                    ▼                           ▼
            ┌───────────────┐          ┌───────────────┐
            │Release Permit │          │Release Permit │
            └───────────────┘          └───────────────┘
```

## Implementation

### Rate Limiter

```rust
pub struct QueueManager {
    queue: Arc<Mutex<MessageQueue>>,
    response_channels: Arc<Mutex<HashMap<String, mpsc::UnboundedSender<LLMResponse>>>>,
    api_client: Option<ApiClient>,
    rate_limiter: Arc<Semaphore>,  // ← Controls concurrency
    max_workers: usize,             // ← Number of parallel workers
}
```

### Configuration

```rust
const DEFAULT_MAX_CONCURRENT_CALLS: usize = 3;

// Create with default rate limit
QueueManager::with_api_client(client)

// Create with custom rate limit
QueueManager::with_rate_limit(client, 5)  // Max 5 concurrent calls
```

### Worker Flow

```rust
// Each worker runs this loop
loop {
    // 1. Dequeue message from priority queue
    let msg = queue.dequeue();
    
    if let Some(queued_msg) = msg {
        // 2. Acquire permit (blocks if at limit)
        let permit = rate_limiter.acquire().await;
        eprintln!("[WORKER-{}] Permit acquired (available: {})", 
            worker_id, rate_limiter.available_permits());
        
        // 3. Make LLM API call
        let stream = SendMessageStream::send_message(...).await?;
        
        // 4. Stream responses
        while let Some(event) = stream.recv().await {
            // Handle events
        }
        
        // 5. Release permit (automatic via Drop)
        drop(permit);
        eprintln!("[WORKER-{}] Permit released (available: {})", 
            worker_id, rate_limiter.available_permits());
    }
}
```

## Behavior

### Normal Operation

```
[WORKER] Starting 3 background worker threads with rate limit 3
[WORKER-0] Started
[WORKER-1] Started
[WORKER-2] Started

// Session 1 message arrives
[WORKER-0] Acquiring rate limit permit (available: 3)
[WORKER-0] Permit acquired, processing session abc123
[WORKER-0] Using real LLM API for session abc123

// Session 2 message arrives (concurrent)
[WORKER-1] Acquiring rate limit permit (available: 2)
[WORKER-1] Permit acquired, processing session def456
[WORKER-1] Using real LLM API for session def456

// Session 3 message arrives (concurrent)
[WORKER-2] Acquiring rate limit permit (available: 1)
[WORKER-2] Permit acquired, processing session ghi789
[WORKER-2] Using real LLM API for session ghi789

// Session 4 message arrives (BLOCKED - at limit)
[WORKER-0] Acquiring rate limit permit (available: 0)
// Worker-0 waits here until a permit is released

// Session 1 completes
[WORKER-0] Completed real LLM processing for session abc123
[WORKER-0] Permit released (available: 1)

// Now Session 4 can proceed
[WORKER-0] Permit acquired, processing session jkl012
```

### Priority Handling

High priority messages (active sessions) are dequeued first:

```
Queue State:
  High Priority: [session-A, session-B]
  Low Priority:  [session-X, session-Y, session-Z]

Worker dequeue order:
  1. session-A (high priority)
  2. session-B (high priority)
  3. session-X (low priority)
  4. session-Y (low priority)
  5. session-Z (low priority)
```

### Rate Limit Reached

```
// All 3 permits in use
[WORKER-0] Processing session-1 (permit 1/3)
[WORKER-1] Processing session-2 (permit 2/3)
[WORKER-2] Processing session-3 (permit 3/3)

// New message arrives
[WORKER-0] Acquiring rate limit permit (available: 0)
// Blocks here until session-1, session-2, or session-3 completes

// Session-1 completes
[WORKER-0] Permit released (available: 1)

// Worker-0 immediately acquires for next message
[WORKER-0] Permit acquired, processing session-4
```

## Benefits

1. **Prevents Throttling**: Never exceed backend API rate limits
2. **Maximizes Throughput**: Process up to N sessions concurrently
3. **Fair Scheduling**: Priority queue ensures important work goes first
4. **Graceful Degradation**: Automatically queues excess requests
5. **Observable**: Logs show permit usage at all times

## Configuration Tuning

### Conservative (Low Rate Limit)
```rust
QueueManager::with_rate_limit(client, 1)
```
- Safest for strict rate limits
- Sequential processing
- Lowest throughput

### Balanced (Default)
```rust
QueueManager::with_rate_limit(client, 3)
```
- Good balance of safety and throughput
- Handles 3 concurrent sessions
- Recommended for most use cases

### Aggressive (High Rate Limit)
```rust
QueueManager::with_rate_limit(client, 10)
```
- Maximum throughput
- Risk of hitting rate limits
- Only if backend supports high concurrency

## Monitoring

### Log Messages

```
[WORKER-0] Acquiring rate limit permit (available: 3)
  → Worker is requesting a permit

[WORKER-0] Permit acquired, processing session abc123
  → Permit granted, API call starting

[WORKER-0] Permit released (available: 3)
  → API call complete, permit returned to pool
```

### Key Metrics

- **Available Permits**: How many more concurrent calls can be made
- **Worker ID**: Which worker is processing
- **Session ID**: Which session is being processed
- **Queue Stats**: High/low priority message counts

## Comparison to Alternatives

### Single Worker (Previous Implementation)
```
✗ Sequential processing only
✗ One LLM call at a time
✗ Underutilizes API capacity
✓ Simple implementation
```

### Unlimited Workers
```
✗ Can overwhelm backend
✗ Gets throttled by API
✗ Wastes resources on failed calls
✓ Maximum theoretical throughput
```

### Rate-Limited Workers (Current)
```
✓ Controlled concurrency
✓ Prevents throttling
✓ Maximizes safe throughput
✓ Observable and tunable
✓ Graceful under load
```

## Future Enhancements

1. **Dynamic Rate Limiting**: Adjust based on API response headers
2. **Backoff Strategy**: Reduce concurrency on throttle errors
3. **Per-User Limits**: Different limits for different users
4. **Metrics Dashboard**: Real-time visualization of throughput
5. **Adaptive Workers**: Spawn more workers when queue grows

## Conclusion

The rate-limited multi-worker architecture ensures:
- ✅ Backend never gets overwhelmed
- ✅ Multiple sessions process concurrently
- ✅ Priority queue ensures important work first
- ✅ Observable and tunable for different backends

**This is production-ready rate limiting.**
