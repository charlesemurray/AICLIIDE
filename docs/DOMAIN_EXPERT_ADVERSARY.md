# Domain Expert Adversarial Prompt

**Purpose**: Challenge implementation with deep technical knowledge  
**Principle**: An expert would spot these issues immediately

---

## The Expert Questions

### SQLite Expert

**"Show me your schema. I'll tell you what's wrong."**

Issues to check:
- Missing indexes on frequently queried columns
- No foreign key constraints
- Wrong column types (TEXT for booleans?)
- No CHECK constraints
- Missing UNIQUE constraints
- No default values
- Nullable columns that shouldn't be

**"Show me your queries. I'll tell you what's slow."**

Issues to check:
- Full table scans
- Missing WHERE clauses
- No LIMIT on unbounded queries
- String concatenation in queries (SQL injection)
- No prepared statements
- No query plan analysis

**"Show me your error handling. I'll tell you what breaks."**

Issues to check:
- SQLITE_BUSY not handled
- SQLITE_LOCKED not handled
- Disk full not handled
- Corruption not detected
- No retry logic
- No timeout configuration

---

### Rust Expert

**"Show me your types. I'll tell you what's unsafe."**

Issues to check:
- unwrap() in production code
- expect() without context
- as conversions that can truncate
- String allocations in hot paths
- Clone where reference would work
- Box where stack would work
- Arc without need for sharing

**"Show me your error types. I'll tell you what's lost."**

Issues to check:
- String errors (no context)
- Swallowed errors
- Error conversion loses information
- No error chain
- No source() implementation
- Can't distinguish error types

**"Show me your tests. I'll tell you what's missing."**

Issues to check:
- No property-based tests
- No edge case tests (empty, max, overflow)
- No error path tests
- No integration tests
- No benchmark tests
- Tests that don't actually test anything

---

### Database Expert

**"Show me your transaction isolation. I'll tell you what races."**

Issues to check:
- Read-modify-write without transaction
- Lost updates
- Dirty reads
- Non-repeatable reads
- Phantom reads
- No isolation level specified

**"Show me your concurrency control. I'll tell you what deadlocks."**

Issues to check:
- Lock ordering violations
- No timeout on locks
- Holding locks across I/O
- No deadlock detection
- No retry on SQLITE_BUSY

**"Show me your data integrity. I'll tell you what corrupts."**

Issues to check:
- No foreign key enforcement
- No CHECK constraints
- No NOT NULL constraints
- No referential integrity
- Orphaned records possible
- Inconsistent state possible

---

### Systems Expert

**"Show me your resource management. I'll tell you what leaks."**

Issues to check:
- File descriptors not closed
- Database connections not closed
- Temporary files not cleaned up
- Memory not freed
- No RAII pattern
- Manual resource management

**"Show me your error recovery. I'll tell you what fails."**

Issues to check:
- No graceful degradation
- No fallback behavior
- No retry logic
- No circuit breaker
- Cascading failures possible
- No health checks

---

## Apply to Feedback System

### SQLite Expert Review

**Schema:**
```sql
CREATE TABLE IF NOT EXISTS memory_feedback (
    memory_id TEXT PRIMARY KEY,
    helpful INTEGER NOT NULL,
    timestamp INTEGER NOT NULL
)
```

**Issues Found:**

1. ❌ **No index on timestamp** - Can't efficiently query recent feedback
2. ❌ **No foreign key to memory table** - Can have feedback for non-existent memories
3. ❌ **INTEGER for boolean** - Should use CHECK constraint
4. ❌ **No default for timestamp** - Should use DEFAULT CURRENT_TIMESTAMP
5. ❌ **TEXT for memory_id** - No validation at schema level

**Correct Schema:**
```sql
CREATE TABLE IF NOT EXISTS memory_feedback (
    memory_id TEXT PRIMARY KEY CHECK(length(memory_id) > 0 AND length(memory_id) <= 255),
    helpful INTEGER NOT NULL CHECK(helpful IN (0, 1)),
    timestamp INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    FOREIGN KEY (memory_id) REFERENCES memory_notes(id) ON DELETE CASCADE
) STRICT;

CREATE INDEX IF NOT EXISTS idx_feedback_timestamp ON memory_feedback(timestamp DESC);
```

---

### Rust Expert Review

**Code:**
```rust
let timestamp = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()  // ❌ PANIC IN PRODUCTION
    .as_secs() as i64;  // ❌ TRUNCATION
```

**Issues Found:**

1. ❌ **unwrap() can panic** - System time before UNIX epoch
2. ❌ **as i64 can truncate** - u64 to i64 conversion
3. ❌ **No error handling** - Silent failure possible

**Correct Code:**
```rust
let timestamp = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?
    .as_secs()
    .try_into()
    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
```

---

**Code:**
```rust
self.conn.execute(
    "INSERT OR REPLACE INTO memory_feedback (memory_id, helpful, timestamp) VALUES (?1, ?2, ?3)",
    [memory_id, &(helpful as i64).to_string(), &timestamp.to_string()],
)?;
```

**Issues Found:**

1. ❌ **String conversion for integers** - Inefficient, wrong type
2. ❌ **No prepared statement caching** - Recompiles query every time
3. ❌ **No SQLITE_BUSY handling** - Fails on concurrent access

**Correct Code:**
```rust
self.conn.execute(
    "INSERT OR REPLACE INTO memory_feedback (memory_id, helpful, timestamp) VALUES (?1, ?2, ?3)",
    params![memory_id, helpful as i64, timestamp],
)?;
```

---

### Database Expert Review

**Transaction Isolation:**
```rust
pub fn record_feedback(&self, memory_id: &str, helpful: bool) -> Result<()> {
    // Single statement - no transaction
    self.conn.execute(...)?;
}
```

**Issues Found:**

1. ✅ **Single statement is atomic** - No transaction needed
2. ❌ **No SQLITE_BUSY retry** - Fails immediately on lock
3. ❌ **No timeout configuration** - Uses default (5 seconds)

**Correct Code:**
```rust
pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
    let conn = Connection::open(db_path)?;
    
    // Set busy timeout
    conn.busy_timeout(Duration::from_secs(30))?;
    
    // Enable WAL mode for better concurrency
    conn.pragma_update(None, "journal_mode", "WAL")?;
    
    // Enable foreign keys
    conn.pragma_update(None, "foreign_keys", "ON")?;
    
    // Create schema...
}
```

---

### Systems Expert Review

**Resource Management:**
```rust
pub struct FeedbackManager {
    conn: Connection,  // ✅ RAII - closes on drop
}
```

**Issues Found:**

1. ✅ **Connection closes on drop** - Good
2. ❌ **No connection pooling** - But not needed (single-threaded)
3. ❌ **No health check** - Can't detect corruption
4. ❌ **No metrics** - Can't monitor performance

**Correct Code:**
```rust
impl FeedbackManager {
    /// Check database health
    pub fn health_check(&self) -> Result<()> {
        self.conn.execute("PRAGMA integrity_check", [])?;
        Ok(())
    }
    
    /// Get database metrics
    pub fn metrics(&self) -> Result<DatabaseMetrics> {
        let page_count: i64 = self.conn.pragma_query_value(None, "page_count", |row| row.get(0))?;
        let page_size: i64 = self.conn.pragma_query_value(None, "page_size", |row| row.get(0))?;
        
        Ok(DatabaseMetrics {
            size_bytes: page_count * page_size,
            entry_count: self.get_stats()?.0 + self.get_stats()?.1,
        })
    }
}
```

---

## Summary of Issues Found

### Critical (Must Fix)
1. ❌ **unwrap() in production code** - Can panic
2. ❌ **as i64 truncation** - Data loss possible
3. ❌ **Wrong parameter types** - String instead of integer

### High Priority (Should Fix)
4. ❌ **No SQLITE_BUSY handling** - Fails on concurrent access
5. ❌ **No foreign key constraint** - Orphaned feedback possible
6. ❌ **No CHECK constraints** - Invalid data possible
7. ❌ **No index on timestamp** - Slow queries

### Medium Priority (Nice to Have)
8. ❌ **No prepared statement caching** - Performance issue
9. ❌ **No WAL mode** - Poor concurrency
10. ❌ **No health check** - Can't detect corruption
11. ❌ **No metrics** - Can't monitor

### Low Priority (Future)
12. ❌ **No connection pooling** - Not needed (single-threaded)
13. ❌ **No retry logic** - Could improve reliability

---

## The Verdict

**Current Grade: C-**

- Works in happy path
- Fails under stress
- Has production bugs (unwrap, truncation)
- Missing database best practices
- No monitoring or health checks

**After Fixes: B+**

- Safe error handling
- Proper database configuration
- Better concurrency handling
- Still missing some nice-to-haves

---

**This is what a domain expert would find in 5 minutes.**
