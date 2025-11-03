# Memory System Developer Guide

Technical documentation for the Cortex memory system in Amazon Q CLI.

## Architecture

The memory system consists of three main components:

1. **cortex-memory crate**: Core memory functionality
2. **ChatSession integration**: Automatic storage and recall
3. **CLI commands**: User-facing memory management

## Core Components

### CortexMemory API

Located in `crates/cortex-memory/src/qcli_api.rs`:

```rust
pub struct CortexMemory {
    stm: ShortTermMemory,
    embedder: CortexEmbedder,
    enabled: bool,
}

impl CortexMemory {
    pub fn new(db_path: PathBuf, config: MemoryConfig) -> Result<Self>;
    pub fn store_interaction(&mut self, user: &str, assistant: &str, metadata: InteractionMetadata) -> Result<()>;
    pub fn recall_context(&mut self, query: &str, limit: usize) -> Result<Vec<ContextItem>>;
    pub fn recall_by_session(&mut self, query: &str, session_id: &str, limit: usize) -> Result<Vec<ContextItem>>;
    pub fn stats(&self) -> MemoryStats;
    pub fn list_recent(&self, limit: usize) -> Result<Vec<ContextItem>>;
    pub fn list_by_session(&self, session_id: &str, limit: usize) -> Result<Vec<ContextItem>>;
    pub fn clear(&mut self) -> Result<usize>;
    pub fn set_enabled(&mut self, enabled: bool);
    pub fn is_enabled(&self) -> bool;
}
```

### Configuration

Located in `crates/cortex-memory/src/config.rs`:

```rust
pub struct MemoryConfig {
    pub enabled: bool,
    pub retention_days: u32,
    pub max_size_mb: u32,
    pub cross_session: bool,
    pub auto_promote: bool,
    pub warn_threshold: u32,
}
```

Builder methods available:
- `with_enabled(bool)`
- `with_retention_days(u32)`
- `with_max_size_mb(u32)`
- `with_cross_session(bool)`
- `with_auto_promote(bool)`
- `with_warn_threshold(u32)`

### Embedder

Located in `crates/cortex-memory/src/embedder.rs`:

Wraps `semantic_search_client::CandleTextEmbedder` for generating embeddings.

```rust
pub struct CortexEmbedder {
    embedder: CandleTextEmbedder,
}

impl CortexEmbedder {
    pub fn new() -> Result<Self>;
    pub fn embed(&self, text: &str) -> Result<Vec<f32>>;
}
```

## ChatSession Integration

Memory is initialized in `ChatSession::new()`:

```rust
let cortex = if !no_memory && memory_enabled {
    let db_path = memory_dir.join("cortex.db");
    let config = MemoryConfig::default()
        .with_enabled(true)
        .with_retention_days(retention_days)
        .with_max_size_mb(max_size_mb);
    
    CortexMemory::new(db_path, config).ok()
} else {
    None
};
```

### Automatic Recall

Before sending messages to the LLM:

```rust
if let Some(ref mut cortex) = self.cortex {
    match cortex.recall_context(&user_input, 3) {
        Ok(items) if !items.is_empty() => {
            // Context recalled successfully
        },
        _ => { /* Continue without context */ }
    }
}
```

### Automatic Storage

After receiving assistant responses (to be implemented):

```rust
if let Some(ref mut cortex) = self.cortex {
    let metadata = InteractionMetadata {
        session_id: conversation_id.to_string(),
        timestamp: SystemTime::now(),
    };
    let _ = cortex.store_interaction(&user_prompt, &assistant_response, metadata);
}
```

## CLI Commands

Located in `crates/chat-cli/src/cli/chat/cli/`:

### Command Structure

```rust
pub enum MemorySubcommand {
    Config,
    Set(SetArgs),
    List(ListArgs),
    Search(SearchArgs),
    Stats,
    Cleanup(CleanupArgs),
    Toggle(ToggleArgs),
}
```

### Recall Command

```rust
pub struct RecallArgs {
    pub query: String,
    pub global: bool,
    pub session: Option<String>,
    pub limit: usize,
}
```

## Settings

Memory settings in `crates/chat-cli/src/database/settings.rs`:

```rust
pub enum Setting {
    MemoryEnabled,           // memory.enabled
    MemoryRetentionDays,     // memory.retentionDays
    MemoryMaxSizeMb,         // memory.maxSizeMb
    MemoryCrossSession,      // memory.crossSession
    MemoryAutoPromote,       // memory.autoPromote
    MemoryWarnThreshold,     // memory.warnThreshold
    MemoryWelcomeShown,      // memory.welcomeShown
    MemoryVerbose,           // memory.verbose
}
```

## Testing

### Unit Tests

Run cortex-memory tests:

```bash
cargo test -p cortex-memory
```

### Integration Tests

Run chat-cli tests:

```bash
cargo test -p chat_cli --lib
```

## Database Schema

SQLite database at `~/.q/memory/cortex.db`:

- **memories table**: Stores interaction content and embeddings
- **metadata table**: Session IDs, timestamps, importance scores
- **embeddings table**: Vector embeddings for semantic search

## Performance Considerations

- **Recall latency**: Target < 100ms for typical queries
- **Storage**: Default 100MB limit, configurable
- **Embeddings**: Generated on-demand, cached in database
- **Session filtering**: Uses indexed session_id for fast queries

## Error Handling

All memory operations are non-blocking:
- Failures log warnings but don't interrupt chat flow
- Graceful degradation when memory unavailable
- User-friendly error messages in CLI commands

## Future Enhancements

Potential improvements:
- Long-term memory with automatic promotion
- Memory importance scoring
- Cross-session memory with privacy controls
- Memory export/import
- Memory visualization tools
