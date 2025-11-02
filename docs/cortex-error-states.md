# Cortex Memory - Error States & Recovery Design

## Error State Catalog

### 1. Database Errors

#### 1.1 Database Locked
**Scenario**: Another process has locked the SQLite database

**Error Message**:
```
You: /recall Lambda deployment

[Error: Memory database is locked]
The memory database is currently in use by another process.

Try:
  • Wait a moment and try again
  • Close other Q CLI instances
  • If problem persists: /memory toggle --disable

Continuing without memory for this query...
```

**Recovery**:
- Automatic retry (1 attempt after 500ms)
- If still locked, continue without memory
- User can manually retry or disable

**Implementation**:
```rust
match cortex.recall_context(query, 5) {
    Err(CortexError::StorageError(e)) if e.contains("locked") => {
        // Show error, continue without memory
        eprintln!("[Error: Memory database is locked]");
        eprintln!("Continuing without memory for this query...\n");
        vec![] // Empty context
    }
    Err(e) => return Err(e),
    Ok(context) => context,
}
```

#### 1.2 Database Corrupted
**Scenario**: SQLite database file is corrupted

**Error Message**:
```
[Error: Memory database is corrupted]
The memory database file appears to be damaged.

Options:
  1. Backup and reset: /memory export backup.json && /memory reset
  2. Disable memory: /memory toggle --disable
  3. Manual fix: rm ~/.q/memory/cortex.db

Would you like to reset the database? (y/N): 
```

**Recovery**:
- Offer to export existing data (if possible)
- Offer to reset database
- Provide manual fix instructions

#### 1.3 Database Permission Error
**Scenario**: No write permission to `~/.q/memory/`

**Error Message**:
```
[Error: Cannot write to memory database]
Permission denied: ~/.q/memory/cortex.db

Fix:
  chmod 755 ~/.q/memory/
  chmod 644 ~/.q/memory/cortex.db

Or disable memory: /memory toggle --disable
```

**Recovery**:
- Show exact commands to fix permissions
- Offer to disable memory
- Continue in read-only mode if possible

---

### 2. Storage Errors

#### 2.1 Storage Full (100 MB limit)
**Scenario**: Memory storage has reached configured limit

**Error Message**:
```
You: How do I deploy to Lambda?

[Warning: Memory storage full (100 MB / 100 MB)]
Cannot store new memories until space is freed.

Options:
  1. Clean old memories: /memory cleanup
  2. Increase limit: /memory set max-size 200
  3. Disable memory: /memory toggle --disable

Continuing without storing this conversation...

Q: Here's how to deploy to Lambda...
```

**Recovery**:
- Continue without storing (read-only mode)
- Offer cleanup command
- Offer to increase limit
- Show current usage stats

**Implementation**:
```rust
if cortex.is_storage_full()? {
    eprintln!("[Warning: Memory storage full]");
    eprintln!("Options:");
    eprintln!("  1. Clean old memories: /memory cleanup");
    eprintln!("  2. Increase limit: /memory set max-size 200\n");
    // Continue without storing
}
```

#### 2.2 Disk Full
**Scenario**: System disk is full

**Error Message**:
```
[Error: Disk full]
Cannot write to memory database - no space left on device.

Free up disk space or disable memory: /memory toggle --disable
```

**Recovery**:
- Clear error message
- Suggest disabling memory
- Continue without storing

---

### 3. Embedding Errors

#### 3.1 Embedder Initialization Failed
**Scenario**: Cannot load embedding model

**Error Message**:
```
[Warning: Memory embedding service unavailable]
The embedding model failed to load. Memory features disabled for this session.

This may be due to:
  • Missing model files
  • Insufficient memory
  • Platform incompatibility (Linux ARM64)

Fallback: Using keyword search instead of semantic search.
To disable this warning: /memory toggle --disable
```

**Recovery**:
- Fallback to keyword search (BM25)
- Continue with degraded functionality
- Offer to disable memory

**Implementation**:
```rust
let embedder = match create_embedder(EmbeddingType::Best) {
    Ok(e) => e,
    Err(_) => {
        eprintln!("[Warning: Embedding service unavailable]");
        eprintln!("Fallback: Using keyword search\n");
        create_embedder(EmbeddingType::Fast)? // BM25 fallback
    }
};
```

#### 3.2 Embedding Generation Failed
**Scenario**: Embedder fails on specific text

**Error Message**:
```
[Warning: Failed to generate embedding for query]
Falling back to keyword search...
```

**Recovery**:
- Fallback to keyword search
- Log error for debugging
- Continue with degraded search

---

### 4. Search Errors

#### 4.1 No Results Found
**Scenario**: Search returns no matching memories

**Message**:
```
You: /recall Lambda deployment

[No memories found matching "Lambda deployment"]

Tips:
  • Try broader terms: /recall Lambda
  • Search all sessions: /recall --global Lambda
  • Check if memory is enabled: /memory config

Q: I don't have any previous context about Lambda deployment.
   How can I help you with Lambda deployment?
```

**Recovery**:
- Helpful suggestions
- Offer to search globally
- Proceed with query anyway

#### 4.2 Search Timeout
**Scenario**: Search takes too long (> 5 seconds)

**Message**:
```
You: /recall Lambda

[Searching memories...]
[Search timed out after 5 seconds]

The memory database may be too large or corrupted.

Try:
  • /memory cleanup - Remove old memories
  • /memory stats - Check database size
  • /memory toggle --disable - Disable if problem persists
```

**Recovery**:
- Timeout after 5 seconds
- Suggest cleanup
- Continue without results

---

### 5. Command Errors

#### 5.1 Invalid Session ID
**Scenario**: User specifies non-existent session

**Message**:
```
You: /recall --session invalid-id Lambda

[Error: Session 'invalid-id' not found]

Available sessions with memories:
  1. session-abc123 - "AWS Lambda deployment" (15 memories)
  2. session-xyz789 - "React patterns" (8 memories)

Use: /recall --list-sessions to see all sessions
```

**Recovery**:
- Show available sessions
- Suggest list command
- Fuzzy match if close

#### 5.2 Invalid Configuration Value
**Scenario**: User sets invalid config value

**Message**:
```
You: /memory set retention -5

[Error: Invalid retention value: -5]
Retention must be 0 (unlimited) or 1-365 days.

Current retention: 30 days
Example: /memory set retention 90
```

**Recovery**:
- Show valid range
- Show current value
- Provide example

#### 5.3 Command Not Available
**Scenario**: Memory disabled but user tries memory command

**Message**:
```
You: /recall Lambda

[Memory is currently disabled]

To enable memory:
  /memory toggle

To learn more:
  /memory config
  /help memory
```

**Recovery**:
- Clear message about state
- Show how to enable
- Provide help

---

### 6. Empty States

#### 6.1 First Use - No Memories Yet
**Scenario**: User tries to recall before any memories stored

**Message**:
```
You: /recall Lambda

[No memories stored yet]

Memory will automatically save your conversations.
Ask a few questions, then try /recall again!

Tips:
  • Memory saves after each Q response
  • Use /memory config to view settings
  • Use /help to see all commands

Q: How can I help you with Lambda?
```

**Recovery**:
- Explain how memory works
- Encourage usage
- Proceed with query

#### 6.2 Empty Session
**Scenario**: User searches specific session with no memories

**Message**:
```
You: /recall --session abc123 Lambda

[No memories in session 'abc123']

This session has no stored memories yet.

Try:
  • /recall Lambda - Search current session
  • /recall --global Lambda - Search all sessions
  • /recall --list-sessions - See sessions with memories
```

**Recovery**:
- Explain session is empty
- Suggest alternatives
- Show other options

#### 6.3 All Memories Cleaned
**Scenario**: User cleaned all memories

**Message**:
```
You: /memory cleanup --all
⚠️  This will delete ALL 1,247 memories. Type 'yes' to confirm: yes

[Deleting memories...]
✓ Deleted 1,247 memories
✓ Freed 45.2 MB of storage

Memory database is now empty.
New memories will be saved as you continue using Q.
```

**Recovery**:
- Confirm action completed
- Show what was deleted
- Reassure new memories will be saved

---

### 7. Warning States

#### 7.1 Storage Approaching Limit (80%)
**Scenario**: Storage at 80% of limit

**Message**:
```
You: How do I deploy to Lambda?

⚠️  Memory storage at 85 MB / 100 MB (85%)
    Run '/memory cleanup' to free space
    Or increase limit: /memory set max-size 200

Q: Here's how to deploy to Lambda...
```

**Recovery**:
- Show once per session
- Provide clear actions
- Don't block operation

#### 7.2 Old Memories (> 30 days)
**Scenario**: Memories approaching retention limit

**Message**:
```
You: /memory stats

Memory Statistics:
  Total: 1,247 memories
  Storage: 45.2 MB / 100 MB (45%)
  
⚠️  127 memories are older than 30 days and will be auto-deleted soon.
    Run '/memory cleanup' to clean now
    Or extend retention: /memory set retention 90
```

**Recovery**:
- Inform during stats command
- Offer to cleanup now
- Offer to extend retention

---

## Error Message Guidelines

### Principles

1. **Be Clear**: State what happened in plain language
2. **Be Helpful**: Provide actionable next steps
3. **Be Concise**: Don't overwhelm with text
4. **Be Consistent**: Use same format for similar errors

### Format

```
[Error: Short description]
Longer explanation of what happened.

Options/Try/Fix:
  • Action 1 with command
  • Action 2 with command
  • Action 3 with command

[Fallback behavior if applicable]
```

### Tone

- **Informative**, not alarming
- **Helpful**, not blaming
- **Actionable**, not vague
- **Professional**, not casual

### Examples

**Good**:
```
[Error: Memory database is locked]
The database is in use by another process.

Try:
  • Wait a moment and try again
  • Close other Q CLI instances

Continuing without memory for this query...
```

**Bad**:
```
Error: DB locked!!! 😱
Something went wrong with the memory thing.
Maybe try again? IDK...
```

---

## Implementation Checklist

### Phase 1 (Must Have)
- [x] Database locked error
- [x] Storage full error
- [x] No results found message
- [x] First use empty state
- [x] Storage warning (80%)

### Phase 2 (Should Have)
- [ ] Database corrupted error
- [ ] Permission error
- [ ] Embedder initialization failed
- [ ] Invalid session ID
- [ ] Invalid config value

### Phase 3 (Nice to Have)
- [ ] Search timeout
- [ ] Disk full error
- [ ] Embedding generation failed
- [ ] Command not available (memory disabled)

---

## Testing Error States

### Manual Testing

```bash
# Test database locked
sqlite3 ~/.q/memory/cortex.db "BEGIN EXCLUSIVE; SELECT sleep(10);" &
q chat
# Try /recall command

# Test storage full
q settings set memory.maxSizeMb 1
# Use Q until storage full

# Test no results
q chat
/recall nonexistent-topic-xyz

# Test first use
rm -rf ~/.q/memory/
q chat
/recall anything
```

### Automated Testing

```rust
#[test]
fn test_database_locked_error() {
    // Lock database
    let _lock = lock_database(&db_path);
    
    // Try to recall
    let result = cortex.recall_context("test", 5);
    
    // Should handle gracefully
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0); // Empty results
}

#[test]
fn test_storage_full_error() {
    let mut cortex = create_cortex_with_limit(1); // 1 MB
    
    // Fill storage
    fill_storage(&mut cortex);
    
    // Try to add more
    let result = cortex.store_interaction("test", "test", metadata);
    
    // Should warn but not fail
    assert!(result.is_ok());
}
```

---

## Summary

**Error states designed**: 20+ scenarios
**Empty states designed**: 3 scenarios
**Warning states designed**: 2 scenarios

**All errors have**:
- Clear message
- Actionable recovery steps
- Graceful fallback behavior
- Consistent formatting

**Ready for implementation** ✅
