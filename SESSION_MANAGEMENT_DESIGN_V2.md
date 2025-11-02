# Session Management Design V2

## Overview

Unified session management system for Q CLI that handles active work, background tasks, and historical reference with production-grade reliability, performance, and maintainability.

## Session Types

### 1. Active Sessions
Interactive work contexts the user switches between

### 2. Background Sessions
Long-running autonomous tasks/agents

### 3. Historical Sessions
Completed conversations for reference

## Architecture

### Core Principles

1. **Abstraction** - Storage layer abstracted for testability and future extensibility
2. **Resilience** - Graceful degradation, corruption recovery, partial failure handling
3. **Performance** - Sub-100ms operations, lazy loading, efficient indexing
4. **Concurrency** - Safe multi-instance operation, file locking, atomic operations
5. **Observability** - Structured logging, metrics, debugging support
6. **Backwards Compatibility** - Schema versioning, migration paths

### Component Architecture

```
┌─────────────────────────────────────────────────────┐
│                  CLI Commands                        │
│         (/sessions list, archive, etc.)              │
└──────────────────────┬──────────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────────┐
│              SessionManager (Facade)                 │
│  - High-level operations                             │
│  - Error translation                                 │
│  - Metrics collection                                │
└──────────────────────┬──────────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────────┐
│           SessionRepository (Trait)                  │
│  - list_sessions()                                   │
│  - get_session()                                     │
│  - save_session()                                    │
│  - delete_session()                                  │
└──────────────────────┬──────────────────────────────┘
                       │
        ┌──────────────┴──────────────┐
        │                             │
┌───────▼────────┐          ┌─────────▼────────┐
│ FileSystemRepo │          │  InMemoryRepo    │
│ (Production)   │          │  (Testing)       │
└────────────────┘          └──────────────────┘
```

### Data Model

```rust
// Schema version for migrations
const METADATA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    // Schema version for backwards compatibility
    #[serde(default = "default_version")]
    pub version: u32,
    
    // Core identity
    pub id: String,
    pub status: SessionStatus,
    
    // Timestamps (RFC3339 for human readability)
    #[serde(with = "time::serde::rfc3339")]
    pub created: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub last_active: OffsetDateTime,
    
    // User-facing info
    pub first_message: String,
    pub name: Option<String>,
    
    // Statistics
    pub file_count: usize,
    pub message_count: usize,
    
    // Background task info (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_task: Option<BackgroundTaskInfo>,
    
    // Extensibility - custom fields for future use
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub custom_fields: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundTaskInfo {
    pub status: TaskStatus,
    #[serde(with = "time::serde::rfc3339")]
    pub started: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub last_update: OffsetDateTime,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
```

## Error Handling Strategy

### Error Types

```rust
#[derive(Debug, Error)]
pub enum SessionError {
    #[error("Session not found: {0}")]
    NotFound(String),
    
    #[error("Session already exists: {0}")]
    AlreadyExists(String),
    
    #[error("Invalid session metadata: {0}")]
    InvalidMetadata(String),
    
    #[error("Corrupted session data: {0}")]
    Corrupted(String),
    
    #[error("Concurrent modification detected")]
    ConcurrentModification,
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Storage error: {0}")]
    Storage(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
```

### Error Recovery

- **Corrupted metadata**: Attempt to recover from backup, fallback to minimal metadata
- **Missing files**: Create with defaults, log warning
- **Permission denied**: Clear error message with resolution steps
- **Concurrent modification**: Retry with exponential backoff
- **Disk full**: Graceful degradation, suggest cleanup

### User-Facing Messages

```rust
impl SessionError {
    pub fn user_message(&self) -> String {
        match self {
            SessionError::NotFound(id) => 
                format!("Session '{}' not found. Use '/sessions list' to see available sessions.", id),
            SessionError::PermissionDenied(_) => 
                "Permission denied. Check file permissions in .amazonq/sessions/".to_string(),
            SessionError::Corrupted(id) => 
                format!("Session '{}' data is corrupted. Attempting recovery...", id),
            _ => format!("Session operation failed: {}", self),
        }
    }
}
```

## Concurrency & Safety

### File Locking Strategy

```rust
// Use advisory file locks for metadata updates
// Lock file: .amazonq/sessions/{id}/.lock

pub struct SessionLock {
    file: File,
    path: PathBuf,
}

impl SessionLock {
    pub async fn acquire(session_dir: &Path, timeout: Duration) -> Result<Self> {
        let lock_path = session_dir.join(".lock");
        let start = Instant::now();
        
        loop {
            match OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&lock_path) 
            {
                Ok(file) => return Ok(Self { file, path: lock_path }),
                Err(e) if e.kind() == ErrorKind::AlreadyExists => {
                    if start.elapsed() > timeout {
                        return Err(SessionError::ConcurrentModification);
                    }
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
                Err(e) => return Err(e.into()),
            }
        }
    }
}

impl Drop for SessionLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}
```

### Atomic Operations

- Use temp file + rename for atomic writes
- Validate before overwriting
- Keep backup of previous version

```rust
pub async fn save_metadata_atomic(
    session_dir: &Path, 
    metadata: &SessionMetadata
) -> Result<()> {
    // Acquire lock
    let _lock = SessionLock::acquire(session_dir, Duration::from_secs(5)).await?;
    
    // Write to temp file
    let temp_path = session_dir.join(".metadata.json.tmp");
    let json = serde_json::to_string_pretty(metadata)?;
    tokio::fs::write(&temp_path, json).await?;
    
    // Backup existing
    let metadata_path = session_dir.join("metadata.json");
    if metadata_path.exists() {
        let backup_path = session_dir.join("metadata.json.bak");
        tokio::fs::copy(&metadata_path, &backup_path).await?;
    }
    
    // Atomic rename
    tokio::fs::rename(&temp_path, &metadata_path).await?;
    
    Ok(())
}
```

## Performance Considerations

### Benchmarks & Targets

| Operation | Target Latency | Max Latency |
|-----------|---------------|-------------|
| List sessions (100) | < 50ms | < 200ms |
| Get session | < 10ms | < 50ms |
| Save session | < 20ms | < 100ms |
| Archive session | < 30ms | < 150ms |

### Optimization Strategies

1. **Lazy Loading**: Don't load full conversation history, only metadata
2. **Caching**: Cache session list with TTL (5 seconds)
3. **Indexing**: Maintain index file for fast lookups
4. **Pagination**: Limit results, support cursor-based pagination
5. **Parallel I/O**: Load multiple sessions concurrently

### Index File

```rust
// .amazonq/sessions/.index.json
#[derive(Serialize, Deserialize)]
pub struct SessionIndex {
    pub version: u32,
    pub last_updated: OffsetDateTime,
    pub sessions: Vec<SessionIndexEntry>,
}

#[derive(Serialize, Deserialize)]
pub struct SessionIndexEntry {
    pub id: String,
    pub status: SessionStatus,
    pub last_active: OffsetDateTime,
    pub name: Option<String>,
    pub first_message_preview: String, // First 100 chars
}

// Rebuild index if:
// - Missing or corrupted
// - Older than 1 hour
// - Session count mismatch
```

### Memory Management

- Stream large lists instead of loading all into memory
- Use `Vec` with capacity hints
- Drop unused data aggressively
- Monitor memory usage in tests

## Backwards Compatibility

### Schema Versioning

```rust
impl SessionMetadata {
    pub fn migrate(mut self) -> Result<Self> {
        match self.version {
            0 => {
                // V0 -> V1: Add custom_fields
                self.custom_fields = HashMap::new();
                self.version = 1;
            }
            1 => {
                // Current version
            }
            v => return Err(SessionError::InvalidMetadata(
                format!("Unknown schema version: {}", v)
            )),
        }
        Ok(self)
    }
}
```

### Migration Strategy

1. **Read old format**: Support reading V0 metadata
2. **Write new format**: Always write latest version
3. **Lazy migration**: Migrate on first write
4. **Backup before migration**: Keep old version
5. **Rollback support**: Can downgrade if needed

### Compatibility Matrix

| Q CLI Version | Metadata Version | Read | Write |
|---------------|------------------|------|-------|
| 1.19.x | V0 | ✓ | ✓ |
| 1.20.x | V1 | ✓ (V0+V1) | ✓ (V1) |
| 1.21.x | V1 | ✓ (V0+V1) | ✓ (V1) |

## Security Considerations

### File Permissions

```rust
#[cfg(unix)]
pub async fn create_session_dir(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    
    tokio::fs::create_dir_all(path).await?;
    
    // Set to 0700 (owner only)
    let mut perms = tokio::fs::metadata(path).await?.permissions();
    perms.set_mode(0o700);
    tokio::fs::set_permissions(path, perms).await?;
    
    Ok(())
}
```

### Sensitive Data

- **No credentials in metadata**: Use references to secure storage
- **Sanitize paths**: Prevent directory traversal
- **Validate input**: Prevent injection attacks
- **Audit logging**: Log security-relevant operations

### Input Validation

```rust
pub fn validate_session_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(SessionError::InvalidMetadata("Name cannot be empty".into()));
    }
    
    if name.len() > 100 {
        return Err(SessionError::InvalidMetadata("Name too long (max 100 chars)".into()));
    }
    
    // Only alphanumeric, dash, underscore
    if !name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err(SessionError::InvalidMetadata(
            "Name can only contain letters, numbers, dash, and underscore".into()
        ));
    }
    
    Ok(())
}
```

## Observability

### Structured Logging

```rust
use tracing::{info, warn, error, debug, instrument};

#[instrument(skip(metadata), fields(session_id = %metadata.id))]
pub async fn save_session(metadata: &SessionMetadata) -> Result<()> {
    debug!("Saving session metadata");
    
    match save_metadata_atomic(session_dir, metadata).await {
        Ok(_) => {
            info!(
                session_id = %metadata.id,
                status = ?metadata.status,
                message_count = metadata.message_count,
                "Session saved successfully"
            );
            Ok(())
        }
        Err(e) => {
            error!(
                session_id = %metadata.id,
                error = %e,
                "Failed to save session"
            );
            Err(e)
        }
    }
}
```

### Metrics

```rust
pub struct SessionMetrics {
    pub list_duration: Histogram,
    pub save_duration: Histogram,
    pub active_sessions: Gauge,
    pub background_sessions: Gauge,
    pub errors: Counter,
}

// Collect metrics
metrics.list_duration.observe(duration.as_secs_f64());
metrics.active_sessions.set(active_count as f64);
```

### Debug Support

```rust
// Debug command: /sessions debug <id>
pub async fn debug_session(id: &str) -> String {
    format!(
        "Session Debug Info:\n\
         ID: {}\n\
         Metadata file: {}\n\
         Metadata size: {} bytes\n\
         Lock file exists: {}\n\
         File count: {}\n\
         Last modified: {}\n",
        id,
        metadata_path.display(),
        metadata_size,
        lock_exists,
        file_count,
        last_modified,
    )
}
```

## Testing Strategy

### Unit Tests
- Every function tested in isolation
- Mock filesystem with `InMemoryRepo`
- Test error conditions
- Test edge cases

### Integration Tests
- Full command workflows
- Real filesystem operations
- Cross-module interactions
- Concurrency scenarios

### Performance Tests
```rust
#[tokio::test]
async fn bench_list_sessions_100() {
    let repo = create_test_repo_with_sessions(100).await;
    
    let start = Instant::now();
    let sessions = repo.list_sessions().await.unwrap();
    let duration = start.elapsed();
    
    assert_eq!(sessions.len(), 100);
    assert!(duration < Duration::from_millis(200), 
        "List took {:?}, expected < 200ms", duration);
}
```

### Failure Injection Tests
```rust
#[tokio::test]
async fn test_corrupted_metadata_recovery() {
    let repo = create_test_repo().await;
    
    // Create session
    let metadata = SessionMetadata::new("test", "First");
    repo.save_session(&metadata).await.unwrap();
    
    // Corrupt metadata file
    let path = repo.session_path("test").join("metadata.json");
    tokio::fs::write(&path, "corrupted json{{{").await.unwrap();
    
    // Should recover or fail gracefully
    let result = repo.get_session("test").await;
    assert!(result.is_err());
    
    // Should still be able to list other sessions
    let sessions = repo.list_sessions().await.unwrap();
    assert!(sessions.is_empty() || !sessions.iter().any(|s| s.id == "test"));
}
```

### Cross-Platform Tests
- Test on Linux, macOS, Windows
- Test path handling differences
- Test filesystem behavior differences

## Integration Points

### Existing Features

1. **Context Manager**: Sessions should track context files
2. **Checkpoints**: Integrate with checkpoint system
3. **Tool Manager**: Track tool usage per session
4. **Agents**: Background sessions for agents
5. **Telemetry**: Report session metrics

### API Compatibility

```rust
// Ensure ConversationState can access session operations
impl ConversationState {
    pub fn session_id(&self) -> &str {
        &self.conversation_id
    }
    
    pub async fn update_session_metadata(&self, os: &Os) -> Result<()> {
        let manager = SessionManager::new(os);
        manager.update_activity(self.session_id()).await
    }
}
```

## Rollback & Recovery

### Rollback Scenarios

1. **Metadata corruption**: Restore from `.bak` file
2. **Schema incompatibility**: Downgrade to previous version
3. **Performance regression**: Disable indexing, fall back to direct scan
4. **Concurrency issues**: Increase lock timeout, add retry logic

### Recovery Procedures

```rust
pub async fn recover_session(session_id: &str) -> Result<SessionMetadata> {
    // Try primary metadata
    if let Ok(metadata) = load_metadata(&session_dir).await {
        return Ok(metadata);
    }
    
    // Try backup
    if let Ok(metadata) = load_metadata_backup(&session_dir).await {
        warn!("Recovered session from backup: {}", session_id);
        return Ok(metadata);
    }
    
    // Reconstruct from directory contents
    warn!("Reconstructing session metadata: {}", session_id);
    reconstruct_metadata(&session_dir).await
}

async fn reconstruct_metadata(session_dir: &Path) -> Result<SessionMetadata> {
    let id = session_dir.file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| SessionError::InvalidMetadata("Invalid session dir".into()))?;
    
    let created = tokio::fs::metadata(session_dir).await?
        .created()?
        .into();
    
    let file_count = count_files(session_dir).await?;
    
    Ok(SessionMetadata {
        version: METADATA_VERSION,
        id: id.to_string(),
        status: SessionStatus::Active,
        created,
        last_active: OffsetDateTime::now_utc(),
        first_message: "[Recovered session]".to_string(),
        name: None,
        file_count,
        message_count: 0,
        background_task: None,
        custom_fields: HashMap::new(),
    })
}
```

## Documentation Requirements

### User Documentation
- Getting started guide
- Command reference
- Common workflows
- Troubleshooting guide

### Developer Documentation
- Architecture overview
- API reference
- Extension guide
- Testing guide

### Architecture Decision Records (ADRs)

**ADR-001: Filesystem-based storage**
- Decision: Use filesystem for session storage
- Rationale: Simple, debuggable, no external dependencies
- Alternatives considered: SQLite, embedded DB
- Consequences: Need file locking, limited query capabilities

**ADR-002: Metadata versioning**
- Decision: Include version field in metadata
- Rationale: Enable schema evolution
- Consequences: Need migration logic

**ADR-003: Advisory file locking**
- Decision: Use lock files for concurrency control
- Rationale: Cross-platform, simple
- Alternatives: flock (Unix-only), database locks
- Consequences: Not foolproof, but good enough

## Open Questions & Future Work

### Questions for Discussion

1. **Session limit**: Should we enforce max active sessions? (Suggest: 20)
2. **Auto-archive**: Auto-archive inactive sessions after N days? (Suggest: 30)
3. **Background execution**: Separate process or in-process? (Suggest: defer to Phase 4)
4. **Cloud sync**: Future support for syncing sessions? (Suggest: design for it, implement later)
5. **Session sharing**: Export/import format? (Suggest: tarball with metadata)

### Future Enhancements

- **Session templates**: Pre-configured session types
- **Session analytics**: Time tracking, productivity metrics
- **Smart search**: Full-text search across artifacts
- **Session recommendations**: "Similar to session X"
- **Collaborative sessions**: Multi-user sessions
- **Cloud backup**: Automatic backup to S3/cloud storage

## Success Metrics

### Functional
- [ ] All session operations work correctly
- [ ] No data loss under normal operation
- [ ] Graceful degradation on errors
- [ ] Backwards compatible with existing sessions

### Performance
- [ ] List 100 sessions in < 200ms
- [ ] Save session in < 100ms
- [ ] No memory leaks
- [ ] Handles 1000+ sessions

### Quality
- [ ] 90%+ test coverage
- [ ] Zero critical bugs in first month
- [ ] < 5 user-reported issues
- [ ] Positive user feedback

### Operational
- [ ] Clear error messages
- [ ] Debuggable with logs
- [ ] Recoverable from failures
- [ ] Documented for users and developers
