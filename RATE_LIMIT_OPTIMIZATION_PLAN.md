# Rate Limiting Optimization Plan

## Goal
Reduce API request volume to avoid rate limiting on Q Developer Pro while maintaining functionality.

## Approaches

### 1. Request Caching Layer
**Location**: `crates/chat-cli/src/api_client/cache.rs` (new)

**Implementation**:
- Cache responses by conversation context hash
- Store in SQLite database alongside existing data
- TTL-based expiration (configurable, default 1 hour)
- Cache key: hash of (prompt + conversation history + model)

**Database Schema**:
```sql
CREATE TABLE response_cache (
    cache_key TEXT PRIMARY KEY,
    request_hash TEXT NOT NULL,
    response_json TEXT NOT NULL,
    model TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    expires_at INTEGER NOT NULL,
    hit_count INTEGER DEFAULT 0
);
CREATE INDEX idx_expires_at ON response_cache(expires_at);
```

**Settings**:
- `cache.enabled` (boolean, default: true)
- `cache.ttl_seconds` (number, default: 3600)
- `cache.max_entries` (number, default: 1000)

---

### 2. Client-Side Rate Limiting
**Location**: `crates/chat-cli/src/api_client/rate_limiter.rs` (new)

**Implementation**:
- Token bucket algorithm
- Track requests per minute/hour
- Queue requests when approaching limits
- Exponential backoff on 429 responses

**Settings**:
- `rateLimit.requestsPerMinute` (number, default: 50)
- `rateLimit.requestsPerHour` (number, default: 500)
- `rateLimit.enableQueueing` (boolean, default: true)

---

### 3. Tiered Model Routing
**Location**: `crates/agent/src/agent/model_router.rs` (new)

**Implementation**:
- Classify queries by complexity
- Route simple queries to faster/cheaper models
- Use heuristics: query length, tool usage, conversation depth

**Classification Rules**:
```rust
enum QueryComplexity {
    Simple,   // < 100 tokens, no tools, factual
    Medium,   // 100-500 tokens, basic tools
    Complex,  // > 500 tokens, multiple tools, reasoning
}
```

**Model Mapping**:
- Simple → Claude Haiku (if available)
- Medium → Default model
- Complex → Claude Sonnet

**Settings**:
- `modelRouter.enabled` (boolean, default: false)
- `modelRouter.simpleThreshold` (number, default: 100)
- `modelRouter.complexThreshold` (number, default: 500)
- `modelRouter.simpleModel` (string, default: "haiku")

---

### 4. Context Optimization
**Location**: `crates/agent/src/agent/context_optimizer.rs` (new)

**Implementation**:
- Aggressive context pruning
- Summarize old conversation turns
- Remove redundant tool outputs
- Smart context window management

**Strategies**:
- Keep last N turns in full
- Summarize older turns
- Remove successful tool outputs (keep only failures)
- Compress file contents in context

**Settings**:
- `context.maxFullTurns` (number, default: 5)
- `context.autoCompactThreshold` (number, default: 0.7) // 70% of context
- `context.removeSuccessfulTools` (boolean, default: false)

---

### 5. Local Preprocessing
**Location**: `crates/agent/src/agent/local_handler.rs` (new)

**Implementation**:
- Handle simple queries locally without API calls
- Pattern matching for common requests
- Local file operations
- Basic code analysis

**Local Handlers**:
```rust
enum LocalHandler {
    FileRead,           // Read file contents
    DirectoryList,      // List directory
    SimpleGrep,         // Search in files
    BasicInfo,          // System info, time, etc.
    HelpText,           // CLI help
}
```

**Settings**:
- `local.enabled` (boolean, default: true)
- `local.maxFileSize` (number, default: 1MB)
- `local.patterns` (array, configurable patterns)

---

## Implementation Phases

### Phase 1: Foundation (Week 1)
- [ ] Add cache database schema
- [ ] Implement basic caching layer
- [ ] Add cache settings
- [ ] Test cache hit/miss scenarios

### Phase 2: Rate Limiting (Week 1)
- [ ] Implement token bucket rate limiter
- [ ] Add request queueing
- [ ] Integrate with API client
- [ ] Add rate limit settings

### Phase 3: Smart Routing (Week 2)
- [ ] Implement query classifier
- [ ] Add model router
- [ ] Test with different query types
- [ ] Add router settings

### Phase 4: Context Optimization (Week 2)
- [ ] Implement context pruning
- [ ] Add auto-compaction triggers
- [ ] Optimize tool output storage
- [ ] Add context settings

### Phase 5: Local Handlers (Week 3)
- [ ] Implement pattern matching
- [ ] Add local file handlers
- [ ] Add basic info handlers
- [ ] Add local handler settings

### Phase 6: Integration & Testing (Week 3)
- [ ] Integration testing
- [ ] Performance benchmarking
- [ ] Documentation
- [ ] User testing

---

## File Structure

```
crates/chat-cli/src/api_client/
├── cache.rs              # Response caching
├── rate_limiter.rs       # Client-side rate limiting
└── mod.rs                # Updated with new modules

crates/agent/src/agent/
├── model_router.rs       # Tiered model routing
├── context_optimizer.rs  # Context pruning
├── local_handler.rs      # Local preprocessing
└── mod.rs                # Updated with new modules

crates/chat-cli/src/database/
└── sqlite_migrations/
    └── 00X_cache_tables.sql  # Cache schema
```

---

## Metrics to Track

1. **Cache Performance**
   - Hit rate
   - Miss rate
   - Average response time (cached vs uncached)

2. **Rate Limiting**
   - Requests queued
   - Average queue time
   - 429 errors avoided

3. **Model Routing**
   - Queries by complexity
   - Model usage distribution
   - Cost savings estimate

4. **Context Optimization**
   - Average context size
   - Tokens saved per request
   - Compaction frequency

5. **Local Handling**
   - Queries handled locally
   - API calls avoided
   - Response time improvement

---

## Configuration Example

```toml
# ~/.aws/amazonq/config.toml

[cache]
enabled = true
ttl_seconds = 3600
max_entries = 1000

[rateLimit]
requestsPerMinute = 50
requestsPerHour = 500
enableQueueing = true

[modelRouter]
enabled = true
simpleThreshold = 100
complexThreshold = 500
simpleModel = "haiku"

[context]
maxFullTurns = 5
autoCompactThreshold = 0.7
removeSuccessfulTools = false

[local]
enabled = true
maxFileSize = 1048576  # 1MB
patterns = [
    "^list files",
    "^show directory",
    "^what time",
    "^help"
]
```

---

## Testing Strategy

1. **Unit Tests**
   - Cache operations
   - Rate limiter logic
   - Query classification
   - Context pruning

2. **Integration Tests**
   - End-to-end with caching
   - Rate limit enforcement
   - Model routing decisions
   - Local handler fallback

3. **Performance Tests**
   - Cache hit rate under load
   - Queue performance
   - Context size reduction
   - Local handler speed

4. **User Acceptance**
   - Real-world usage patterns
   - Rate limit avoidance
   - Response quality
   - User experience

---

## Success Criteria

- [ ] 30%+ reduction in API requests
- [ ] 50%+ cache hit rate for repeated queries
- [ ] Zero rate limit errors during normal usage
- [ ] No degradation in response quality
- [ ] < 100ms overhead for optimizations
