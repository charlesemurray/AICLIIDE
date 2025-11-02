# Cortex Memory - Session Integration

## Q CLI Session Infrastructure

### Existing Components

**Session Repository** (`crates/chat-cli/src/session/repository.rs`):
- `SessionRepository` trait for storage operations
- `get()`, `save()`, `delete()`, `list()`, `exists()`
- `SessionFilter` for filtering by status, search, limit
- Already implemented with in-memory and persistent storage

**Session Metadata** (`crates/chat-cli/src/session/metadata.rs`):
```rust
pub struct SessionMetadata {
    pub id: String,
    pub status: SessionStatus,  // Active, Background, Archived
    pub created: OffsetDateTime,
    pub last_active: OffsetDateTime,
    pub first_message: String,
    pub name: Option<String>,
    pub message_count: usize,
    pub custom_fields: HashMap<String, serde_json::Value>,
}
```

**Session Commands** (`crates/chat-cli/src/cli/chat/cli/sessions.rs`):
- `/sessions list` - List active sessions
- `/sessions create` - Create development session
- `/sessions close` - Close session
- `/sessions cleanup` - Clean old sessions

---

## Integration with Cortex Memory

### 1. Link Memory to Sessions

**Store session_id in memory metadata**:
```rust
// When storing interaction
let mut metadata = HashMap::new();
metadata.insert("session_id".to_string(), Value::String(session.id.clone()));
metadata.insert("session_name".to_string(), Value::String(session.name.clone()));
metadata.insert("created_at".to_string(), Value::String(session.created.to_string()));

let note = MemoryNote::new(id, content, metadata);
cortex.manager.add(note, embedding)?;
```

**Filter by session in recall**:
```rust
// Current session only (default)
let filter = HashMap::from([
    ("session_id".to_string(), Value::String(current_session_id))
]);
let results = cortex.ltm.search(&query_embedding, k, Some(&filter))?;

// All sessions (global)
let results = cortex.ltm.search(&query_embedding, k, None)?;
```

### 2. Enhanced `/recall` Command

**Integrate with session discovery**:
```bash
# List sessions with memory
You: /recall --list-sessions
Q: Sessions with memories:
   1. session-abc123 (Today, 15 memories) - "AWS Lambda deployment"
   2. session-xyz789 (Yesterday, 8 memories) - "React component patterns"
   3. session-def456 (3 days ago, 12 memories) - "Database optimization"

# Recall from specific session by number or ID
You: /recall --session 1 Lambda configuration
You: /recall --session abc123 Lambda configuration

# Recall with session name
You: /recall --session "AWS Lambda deployment" configuration
```

**Auto-complete session names**:
```bash
You: /recall --session <TAB>
→ Shows: session-abc123 (AWS Lambda deployment)
         session-xyz789 (React component patterns)
         ...
```

### 3. Session-Aware Memory Commands

**Enhanced `q memory` commands**:
```bash
# List memories by session
$ q memory list --session abc123
$ q memory list --session "AWS Lambda deployment"

# Search within session
$ q memory search "Lambda" --session abc123

# Show session memory stats
$ q memory stats --by-session
Sessions with memories:
- session-abc123: 15 memories, 2.3 MB
- session-xyz789: 8 memories, 1.1 MB
- session-def456: 12 memories, 1.8 MB
Total: 35 memories across 3 sessions
```

### 4. Session Lifecycle Integration

**Automatic memory management**:
```rust
impl ChatSession {
    // When session starts
    pub async fn start(&mut self) -> Result<()> {
        // Initialize Cortex for this session
        self.cortex = Some(CortexMemory::new(&self.memory_db_path)?);
        Ok(())
    }
    
    // When session ends
    pub async fn end(&mut self) -> Result<()> {
        // Promote important memories to LTM
        if let Some(cortex) = &mut self.cortex {
            cortex.promote_session_memories(&self.session_id).await?;
        }
        Ok(())
    }
    
    // When session archived
    pub async fn archive(&mut self) -> Result<()> {
        // Keep memories but mark session as archived
        if let Some(cortex) = &mut self.cortex {
            cortex.archive_session_memories(&self.session_id).await?;
        }
        Ok(())
    }
}
```

---

## Updated `/recall` Command Design

### Syntax
```bash
/recall [options] <query>

Options:
  --global, -g              Search all sessions
  --session <id|name>, -s   Search specific session
  --recent <time>           Search recent time period
  --list-sessions, -ls      List sessions with memories
  --limit <n>, -l           Max results (default: 5)
```

### Examples

**Current session** (default):
```bash
You: /recall Lambda deployment
Q: [searches current session only]
```

**Specific session by ID**:
```bash
You: /recall --session abc123 Lambda deployment
Q: [searches session abc123]
```

**Specific session by name** (fuzzy match):
```bash
You: /recall --session "AWS Lambda" deployment
Q: Found session: "AWS Lambda deployment" (session-abc123)
   [searches that session]
```

**List sessions**:
```bash
You: /recall --list-sessions
Q: Sessions with memories:
   1. session-abc123 (Today) - "AWS Lambda deployment" (15 memories)
   2. session-xyz789 (Yesterday) - "React patterns" (8 memories)
   3. session-def456 (3 days ago) - "Database optimization" (12 memories)
   
   Use: /recall --session <id|number> <query>
```

**Global search**:
```bash
You: /recall --global Lambda
Q: Searching all sessions...
   Found in 3 sessions:
   
   📌 session-abc123 (Today, 95% relevant)
   "AWS Lambda deployment with Python..."
   
   📌 session-xyz789 (1 week ago, 87% relevant)
   "Lambda environment variables..."
```

**Recent time period**:
```bash
You: /recall --recent 7d deployment issues
Q: Searching last 7 days across all sessions...
```

---

## Implementation Tasks

### Phase 1: Basic Integration
1. ✅ Store session_id in memory metadata
2. ✅ Filter by session_id in recall
3. ✅ Default to current session

### Phase 2: Session Discovery
1. Add `--list-sessions` to `/recall` command
2. Query SessionRepository for session list
3. Show sessions with memory counts
4. Enable recall by session ID

### Phase 3: Enhanced UX
1. Add session name fuzzy matching
2. Add tab completion for session names
3. Add `--recent` time filtering
4. Show session context in results

### Phase 4: Lifecycle Integration
1. Auto-promote memories on session end
2. Archive memories with session
3. Clean up old session memories
4. Session memory statistics

---

## Database Schema Update

**Add session index to memories table**:
```sql
-- Already have metadata as JSON, just add index
CREATE INDEX IF NOT EXISTS idx_session_id 
ON memories(json_extract(metadata, '$.session_id'));

-- For faster session listing
CREATE INDEX IF NOT EXISTS idx_session_created 
ON memories(json_extract(metadata, '$.created_at'));
```

---

## Decision Summary

**✅ Question 2: Memory Scope Default**

**Decision**: 
- Default: Session-only (current conversation)
- In-chat: `/recall --session <id|name>` for specific sessions
- In-chat: `/recall --global` for all sessions
- In-chat: `/recall --list-sessions` to discover sessions
- Integration: Use Q CLI's existing SessionRepository

**Benefits**:
- ✅ Leverages existing session infrastructure
- ✅ Consistent with Q CLI's session model
- ✅ Natural discovery via `/recall --list-sessions`
- ✅ No need to leave chat prompt
- ✅ Session names for better UX
