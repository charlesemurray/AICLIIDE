# Session Management - Gap Closure Implementation Plan

## Executive Summary

**Current Grade:** B (83/100)
**Target Grade:** A (95/100)
**Estimated Effort:** 3-4 days
**Priority:** High (required for production deployment)

## Critical Gaps (Must Fix)

### Gap 1: No Observability ❌ CRITICAL
**Current:** No logging, no metrics, no debugging support
**Impact:** Cannot debug production issues, no visibility into system health
**Priority:** P0 (blocking production)

### Gap 2: Race Conditions ❌ CRITICAL
**Current:** Concurrent writes can corrupt metadata
**Impact:** Data corruption in multi-process scenarios
**Priority:** P0 (data integrity)

### Gap 3: Repository Pattern Not Used ⚠️ IMPORTANT
**Current:** SessionManager uses concrete Os type
**Impact:** Reduced testability, tight coupling
**Priority:** P1 (architectural debt)

### Gap 4: No Performance Optimization ⚠️ IMPORTANT
**Current:** No caching, loads all sessions every time
**Impact:** Degrades with scale (1000+ sessions)
**Priority:** P1 (scalability)

---

## Phase 1: Observability (Day 1) - P0

### 1.1 Add Structured Logging

**Goal:** Enable debugging and monitoring

**Implementation:**

```rust
// session/manager.rs
use tracing::{debug, info, warn, error, instrument};

impl<'a> SessionManager<'a> {
    #[instrument(skip(self), fields(session_count))]
    pub async fn list_sessions(&self) -> Result<Vec<SessionMetadata>, SessionError> {
        debug!("Listing sessions from filesystem");
        let sessions_dir = self.os.env.current_dir()?.join(".amazonq/sessions");

        if !sessions_dir.exists() {
            debug!("Sessions directory does not exist");
            return Ok(Vec::new());
        }

        let mut sessions = Vec::new();
        let mut entries = tokio::fs::read_dir(&sessions_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            if entry.file_type().await?.is_dir() {
                match load_metadata(&entry.path()).await {
                    Ok(metadata) => {
                        debug!(session_id = %metadata.id, "Loaded session metadata");
                        sessions.push(metadata);
                    }
                    Err(e) => {
                        warn!(path = ?entry.path(), error = %e, "Failed to load session metadata");
                    }
                }
            }
        }

        sessions.sort_by(|a, b| b.last_active.cmp(&a.last_active));
        
        info!(count = sessions.len(), "Listed sessions successfully");
        tracing::Span::current().record("session_count", sessions.len());
        
        Ok(sessions)
    }

    #[instrument(skip(self), fields(session_id))]
    pub async fn archive_session(&self, session_id: &str) -> Result<(), SessionError> {
        info!(session_id, "Archiving session");
        
        let session_dir = self.session_dir(session_id)?;
        let mut metadata = load_metadata(&session_dir).await
            .map_err(|e| {
                error!(session_id, error = %e, "Failed to load metadata for archiving");
                e
            })?;
        
        metadata.archive();
        save_metadata(&session_dir, &metadata).await
            .map_err(|e| {
                error!(session_id, error = %e, "Failed to save archived metadata");
                e
            })?;
        
        info!(session_id, "Session archived successfully");
        Ok(())
    }
}
```

**Files to modify:**
- `session/manager.rs` - Add logging to all public methods
- `session/io.rs` - Add logging to save/load operations
- `session/metadata.rs` - Add logging to validation

**Dependencies:**
```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = "0.3"
```

**Tests:**
```rust
#[tokio::test]
async fn test_logging_on_list_sessions() {
    let subscriber = tracing_subscriber::fmt()
        .with_test_writer()
        .finish();
    
    tracing::subscriber::with_default(subscriber, || async {
        let manager = SessionManager::new(&os);
        manager.list_sessions().await.unwrap();
        // Verify logs were emitted
    });
}
```

**Acceptance Criteria:**
- [ ] All public methods have `#[instrument]`
- [ ] Debug logs for operations
- [ ] Info logs for success
- [ ] Warn logs for recoverable errors
- [ ] Error logs for failures
- [ ] Structured fields (session_id, count, etc.)

**Effort:** 4 hours

---

### 1.2 Add Metrics

**Goal:** Monitor system health and performance

**Implementation:**

```rust
// session/metrics.rs
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Clone)]
pub struct SessionMetrics {
    pub list_calls: Arc<AtomicU64>,
    pub list_duration_ms: Arc<AtomicU64>,
    pub archive_calls: Arc<AtomicU64>,
    pub errors: Arc<AtomicU64>,
    pub active_sessions: Arc<AtomicU64>,
}

impl SessionMetrics {
    pub fn new() -> Self {
        Self {
            list_calls: Arc::new(AtomicU64::new(0)),
            list_duration_ms: Arc::new(AtomicU64::new(0)),
            archive_calls: Arc::new(AtomicU64::new(0)),
            errors: Arc::new(AtomicU64::new(0)),
            active_sessions: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn record_list(&self, duration_ms: u64, count: usize) {
        self.list_calls.fetch_add(1, Ordering::Relaxed);
        self.list_duration_ms.fetch_add(duration_ms, Ordering::Relaxed);
        self.active_sessions.store(count as u64, Ordering::Relaxed);
    }

    pub fn record_archive(&self) {
        self.archive_calls.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_error(&self) {
        self.errors.fetch_add(1, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            list_calls: self.list_calls.load(Ordering::Relaxed),
            avg_list_duration_ms: self.list_duration_ms.load(Ordering::Relaxed) 
                / self.list_calls.load(Ordering::Relaxed).max(1),
            archive_calls: self.archive_calls.load(Ordering::Relaxed),
            errors: self.errors.load(Ordering::Relaxed),
            active_sessions: self.active_sessions.load(Ordering::Relaxed),
        }
    }
}

pub struct MetricsSnapshot {
    pub list_calls: u64,
    pub avg_list_duration_ms: u64,
    pub archive_calls: u64,
    pub errors: u64,
    pub active_sessions: u64,
}
```

**Usage:**

```rust
pub struct SessionManager<'a> {
    os: &'a Os,
    metrics: SessionMetrics,
}

impl<'a> SessionManager<'a> {
    pub async fn list_sessions(&self) -> Result<Vec<SessionMetadata>, SessionError> {
        let start = std::time::Instant::now();
        
        let result = self.list_sessions_impl().await;
        
        let duration_ms = start.elapsed().as_millis() as u64;
        match &result {
            Ok(sessions) => {
                self.metrics.record_list(duration_ms, sessions.len());
            }
            Err(_) => {
                self.metrics.record_error();
            }
        }
        
        result
    }
}
```

**Acceptance Criteria:**
- [ ] Metrics for all operations
- [ ] Duration tracking
- [ ] Error counting
- [ ] Session count tracking
- [ ] Metrics accessible via API

**Effort:** 3 hours

---

## Phase 2: Concurrency Safety (Day 2) - P0

### 2.1 Add File Locking

**Goal:** Prevent concurrent write corruption

**Implementation:**

```rust
// session/lock.rs
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

pub struct FileLock {
    lock_file: File,
    path: std::path::PathBuf,
}

impl FileLock {
    pub async fn acquire(session_dir: &Path) -> Result<Self, SessionError> {
        let lock_path = session_dir.join(".lock");
        
        // Try to create lock file exclusively
        let lock_file = tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&lock_path)
            .await
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::AlreadyExists {
                    SessionError::ConcurrentModification
                } else {
                    SessionError::Storage(e)
                }
            })?;
        
        Ok(Self {
            lock_file,
            path: lock_path,
        })
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        // Best effort cleanup
        let _ = std::fs::remove_file(&self.path);
    }
}
```

**Usage:**

```rust
pub async fn save_metadata(session_dir: &Path, metadata: &SessionMetadata) -> Result<(), SessionError> {
    tokio::fs::create_dir_all(session_dir).await?;
    
    // Acquire lock
    let _lock = FileLock::acquire(session_dir).await?;
    
    // Write atomically
    let metadata_path = session_dir.join("metadata.json");
    let temp_path = session_dir.join(".metadata.json.tmp");
    
    let json = serde_json::to_string_pretty(metadata)?;
    tokio::fs::write(&temp_path, json).await?;
    tokio::fs::rename(&temp_path, &metadata_path).await?;
    
    // Lock released on drop
    Ok(())
}
```

**Acceptance Criteria:**
- [ ] Lock acquired before writes
- [ ] Lock released on success
- [ ] Lock released on error
- [ ] Concurrent writes blocked
- [ ] Stale locks cleaned up (timeout)

**Effort:** 4 hours

---

### 2.2 Add Lock Timeout and Recovery

**Goal:** Handle stale locks

**Implementation:**

```rust
impl FileLock {
    const LOCK_TIMEOUT_SECS: u64 = 30;
    
    pub async fn acquire_with_timeout(session_dir: &Path) -> Result<Self, SessionError> {
        let lock_path = session_dir.join(".lock");
        
        // Check if lock exists and is stale
        if lock_path.exists() {
            if let Ok(metadata) = tokio::fs::metadata(&lock_path).await {
                if let Ok(modified) = metadata.modified() {
                    let age = std::time::SystemTime::now()
                        .duration_since(modified)
                        .unwrap_or_default();
                    
                    if age.as_secs() > Self::LOCK_TIMEOUT_SECS {
                        warn!("Removing stale lock file (age: {}s)", age.as_secs());
                        let _ = tokio::fs::remove_file(&lock_path).await;
                    }
                }
            }
        }
        
        Self::acquire(session_dir).await
    }
}
```

**Acceptance Criteria:**
- [ ] Stale locks detected
- [ ] Stale locks removed
- [ ] Logged when removing stale locks
- [ ] Configurable timeout

**Effort:** 2 hours

---

## Phase 3: Architecture Fix (Day 2-3) - P1

### 3.1 Use Repository Trait in SessionManager

**Goal:** Proper dependency injection, better testability

**Implementation:**

```rust
// session/manager.rs
pub struct SessionManager<R: SessionRepository> {
    repository: R,
}

impl<R: SessionRepository> SessionManager<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn list_sessions(&self) -> Result<Vec<SessionMetadata>, SessionError> {
        let filter = SessionFilter::default();
        self.repository.list(filter).await
    }

    pub async fn list_by_status(&self, status: SessionStatus) -> Result<Vec<SessionMetadata>, SessionError> {
        let filter = SessionFilter {
            status: Some(status),
            ..Default::default()
        };
        self.repository.list(filter).await
    }

    pub async fn get_session(&self, session_id: &str) -> Result<SessionMetadata, SessionError> {
        self.repository.get(session_id).await
    }

    pub async fn archive_session(&self, session_id: &str) -> Result<(), SessionError> {
        let mut metadata = self.repository.get(session_id).await?;
        metadata.archive();
        self.repository.save(&metadata).await
    }

    pub async fn name_session(&self, session_id: &str, name: impl Into<String>) -> Result<(), SessionError> {
        let mut metadata = self.repository.get(session_id).await?;
        metadata.set_name(name)?;
        self.repository.save(&metadata).await
    }
}
```

**Create FileSystemRepository:**

```rust
// session/fs_repository.rs
pub struct FileSystemRepository {
    os: Os,
}

impl FileSystemRepository {
    pub fn new(os: Os) -> Self {
        Self { os }
    }
    
    fn sessions_dir(&self) -> Result<PathBuf, SessionError> {
        Ok(self.os.env.current_dir()?.join(".amazonq/sessions"))
    }
}

#[async_trait]
impl SessionRepository for FileSystemRepository {
    async fn get(&self, id: &str) -> Result<SessionMetadata, SessionError> {
        let session_dir = self.sessions_dir()?.join(id);
        load_metadata(&session_dir).await
    }

    async fn save(&self, metadata: &SessionMetadata) -> Result<(), SessionError> {
        let session_dir = self.sessions_dir()?.join(&metadata.id);
        save_metadata(&session_dir, metadata).await
    }

    async fn delete(&self, id: &str) -> Result<(), SessionError> {
        let session_dir = self.sessions_dir()?.join(id);
        tokio::fs::remove_dir_all(session_dir).await?;
        Ok(())
    }

    async fn list(&self, filter: SessionFilter) -> Result<Vec<SessionMetadata>, SessionError> {
        let sessions_dir = self.sessions_dir()?;
        
        if !sessions_dir.exists() {
            return Ok(Vec::new());
        }

        let mut sessions = Vec::new();
        let mut entries = tokio::fs::read_dir(&sessions_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            if entry.file_type().await?.is_dir() {
                if let Ok(metadata) = load_metadata(&entry.path()).await {
                    sessions.push(metadata);
                }
            }
        }

        // Apply filters
        if let Some(status) = filter.status {
            sessions.retain(|s| s.status == status);
        }

        if let Some(search) = filter.search {
            let search_lower = search.to_lowercase();
            sessions.retain(|s| {
                s.first_message.to_lowercase().contains(&search_lower)
                    || s.name.as_ref().map_or(false, |n| n.to_lowercase().contains(&search_lower))
            });
        }

        sessions.sort_by(|a, b| b.last_active.cmp(&a.last_active));

        if let Some(limit) = filter.limit {
            sessions.truncate(limit);
        }

        Ok(sessions)
    }

    async fn exists(&self, id: &str) -> Result<bool, SessionError> {
        let session_dir = self.sessions_dir()?.join(id);
        Ok(session_dir.exists())
    }
}
```

**Update call sites:**

```rust
// cli/chat/cli/session_mgmt.rs
impl SessionMgmtArgs {
    pub async fn execute(self, _session: &mut ChatSession, os: &Os) -> Result<ChatState, ChatError> {
        let repository = FileSystemRepository::new(os.clone());
        let manager = SessionManager::new(repository);
        
        // ... rest of implementation
    }
}
```

**Acceptance Criteria:**
- [ ] SessionManager uses trait
- [ ] FileSystemRepository implements trait
- [ ] All tests pass
- [ ] Can inject InMemoryRepository for tests
- [ ] No breaking changes to public API

**Effort:** 6 hours

---

## Phase 4: Performance (Day 3-4) - P1

### 4.1 Add Caching Layer

**Goal:** Reduce filesystem reads

**Implementation:**

```rust
// session/cache.rs
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};

pub struct SessionCache {
    cache: Arc<RwLock<HashMap<String, CachedEntry>>>,
    ttl: Duration,
}

struct CachedEntry {
    metadata: SessionMetadata,
    cached_at: Instant,
}

impl SessionCache {
    pub fn new(ttl: Duration) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            ttl,
        }
    }

    pub async fn get(&self, id: &str) -> Option<SessionMetadata> {
        let cache = self.cache.read().await;
        cache.get(id).and_then(|entry| {
            if entry.cached_at.elapsed() < self.ttl {
                Some(entry.metadata.clone())
            } else {
                None
            }
        })
    }

    pub async fn put(&self, metadata: SessionMetadata) {
        let mut cache = self.cache.write().await;
        cache.insert(metadata.id.clone(), CachedEntry {
            metadata,
            cached_at: Instant::now(),
        });
    }

    pub async fn invalidate(&self, id: &str) {
        let mut cache = self.cache.write().await;
        cache.remove(id);
    }

    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }
}
```

**Cached Repository:**

```rust
// session/cached_repository.rs
pub struct CachedRepository<R: SessionRepository> {
    inner: R,
    cache: SessionCache,
}

impl<R: SessionRepository> CachedRepository<R> {
    pub fn new(inner: R, ttl: Duration) -> Self {
        Self {
            inner,
            cache: SessionCache::new(ttl),
        }
    }
}

#[async_trait]
impl<R: SessionRepository> SessionRepository for CachedRepository<R> {
    async fn get(&self, id: &str) -> Result<SessionMetadata, SessionError> {
        // Try cache first
        if let Some(metadata) = self.cache.get(id).await {
            return Ok(metadata);
        }

        // Cache miss - load from inner
        let metadata = self.inner.get(id).await?;
        self.cache.put(metadata.clone()).await;
        Ok(metadata)
    }

    async fn save(&self, metadata: &SessionMetadata) -> Result<(), SessionError> {
        self.inner.save(metadata).await?;
        self.cache.put(metadata.clone()).await;
        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<(), SessionError> {
        self.inner.delete(id).await?;
        self.cache.invalidate(id).await;
        Ok(())
    }

    async fn list(&self, filter: SessionFilter) -> Result<Vec<SessionMetadata>, SessionError> {
        // Don't cache list results (too complex to invalidate)
        self.inner.list(filter).await
    }

    async fn exists(&self, id: &str) -> Result<bool, SessionError> {
        if self.cache.get(id).await.is_some() {
            return Ok(true);
        }
        self.inner.exists(id).await
    }
}
```

**Usage:**

```rust
let fs_repo = FileSystemRepository::new(os.clone());
let cached_repo = CachedRepository::new(fs_repo, Duration::from_secs(60));
let manager = SessionManager::new(cached_repo);
```

**Acceptance Criteria:**
- [ ] Cache hit reduces filesystem reads
- [ ] Cache invalidated on writes
- [ ] Configurable TTL
- [ ] Cache statistics available
- [ ] Thread-safe

**Effort:** 5 hours

---

### 4.2 Add Performance Tests

**Goal:** Measure and track performance

**Implementation:**

```rust
// session/benches/session_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_list_sessions(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    c.bench_function("list_sessions_10", |b| {
        b.to_async(&rt).iter(|| async {
            let manager = setup_manager_with_sessions(10).await;
            black_box(manager.list_sessions().await.unwrap())
        });
    });

    c.bench_function("list_sessions_100", |b| {
        b.to_async(&rt).iter(|| async {
            let manager = setup_manager_with_sessions(100).await;
            black_box(manager.list_sessions().await.unwrap())
        });
    });

    c.bench_function("list_sessions_1000", |b| {
        b.to_async(&rt).iter(|| async {
            let manager = setup_manager_with_sessions(1000).await;
            black_box(manager.list_sessions().await.unwrap())
        });
    });
}

criterion_group!(benches, bench_list_sessions);
criterion_main!(benches);
```

**Acceptance Criteria:**
- [ ] Benchmarks for all operations
- [ ] Test with 10, 100, 1000 sessions
- [ ] Baseline established
- [ ] Performance regression detection

**Effort:** 3 hours

---

## Phase 5: Additional Improvements (Day 4) - P2

### 5.1 Add Operational Runbook

**Goal:** Support on-call engineers

**Create:** `docs/SESSION_RUNBOOK.md`

**Contents:**
- Common issues and solutions
- How to check session health
- How to recover corrupted sessions
- How to clear stale locks
- Metrics to monitor
- Alert thresholds

**Effort:** 2 hours

---

### 5.2 Add Health Check Endpoint

**Goal:** Monitor system health

**Implementation:**

```rust
pub struct SessionHealth {
    pub total_sessions: usize,
    pub active_sessions: usize,
    pub archived_sessions: usize,
    pub corrupted_sessions: usize,
    pub stale_locks: usize,
    pub cache_hit_rate: f64,
}

impl SessionManager<R> {
    pub async fn health_check(&self) -> Result<SessionHealth, SessionError> {
        let all_sessions = self.repository.list(SessionFilter::default()).await?;
        
        Ok(SessionHealth {
            total_sessions: all_sessions.len(),
            active_sessions: all_sessions.iter().filter(|s| s.status == SessionStatus::Active).count(),
            archived_sessions: all_sessions.iter().filter(|s| s.status == SessionStatus::Archived).count(),
            corrupted_sessions: 0, // TODO: detect corrupted
            stale_locks: 0, // TODO: count stale locks
            cache_hit_rate: 0.0, // TODO: from metrics
        })
    }
}
```

**Effort:** 2 hours

---

## Implementation Schedule

### Day 1: Observability (8 hours)
- [ ] 09:00-13:00: Add structured logging (4h)
- [ ] 14:00-17:00: Add metrics (3h)
- [ ] 17:00-18:00: Testing and documentation (1h)

### Day 2: Concurrency + Architecture Start (8 hours)
- [ ] 09:00-13:00: File locking (4h)
- [ ] 14:00-16:00: Lock timeout/recovery (2h)
- [ ] 16:00-18:00: Start Repository refactor (2h)

### Day 3: Architecture + Performance Start (8 hours)
- [ ] 09:00-13:00: Complete Repository refactor (4h)
- [ ] 14:00-18:00: Caching layer (4h)

### Day 4: Performance + Operations (8 hours)
- [ ] 09:00-12:00: Performance tests (3h)
- [ ] 13:00-15:00: Runbook (2h)
- [ ] 15:00-17:00: Health check (2h)
- [ ] 17:00-18:00: Final testing and documentation (1h)

**Total Effort:** 32 hours (4 days)

---

## Success Criteria

### Must Have (P0)
- [ ] All operations logged with tracing
- [ ] Metrics for operations, duration, errors
- [ ] File locking prevents corruption
- [ ] Stale lock recovery
- [ ] All tests pass

### Should Have (P1)
- [ ] SessionManager uses Repository trait
- [ ] FileSystemRepository implementation
- [ ] Caching layer reduces reads by 80%
- [ ] Performance benchmarks established

### Nice to Have (P2)
- [ ] Operational runbook
- [ ] Health check endpoint
- [ ] Monitoring dashboard

---

## Testing Strategy

### Unit Tests
- [ ] Test logging output
- [ ] Test metrics recording
- [ ] Test file locking
- [ ] Test cache hit/miss
- [ ] Test stale lock recovery

### Integration Tests
- [ ] Test concurrent writes blocked
- [ ] Test cache invalidation
- [ ] Test end-to-end with caching

### Performance Tests
- [ ] Benchmark without cache
- [ ] Benchmark with cache
- [ ] Verify 80% improvement

### Load Tests
- [ ] 1000 sessions
- [ ] 100 concurrent operations
- [ ] Verify no corruption

---

## Rollout Plan

### Phase 1: Internal Testing
- Deploy with logging/metrics only
- Monitor for 1 week
- Verify no issues

### Phase 2: Add Safety Features
- Deploy file locking
- Monitor for lock contention
- Verify no corruption

### Phase 3: Add Performance
- Deploy caching
- Monitor cache hit rate
- Verify performance improvement

### Phase 4: Full Production
- All features enabled
- Full monitoring
- On-call runbook ready

---

## Risk Mitigation

### Risk 1: File Locking Breaks Existing Code
**Mitigation:** Feature flag, gradual rollout
**Rollback:** Disable locking, revert to previous version

### Risk 2: Cache Causes Stale Data
**Mitigation:** Short TTL (60s), invalidate on writes
**Rollback:** Disable cache, direct filesystem access

### Risk 3: Performance Regression
**Mitigation:** Benchmarks before/after, load testing
**Rollback:** Revert to previous version

---

## Post-Implementation

### Monitoring
- Session operation latency (p50, p95, p99)
- Error rate
- Cache hit rate
- Lock contention
- Active session count

### Alerts
- Error rate > 1%
- Latency p95 > 100ms
- Cache hit rate < 70%
- Stale locks > 5

### Documentation
- Update user guide
- Update architecture doc
- Create runbook
- Update README

---

## Expected Outcome

**Before:**
- Grade: B (83/100)
- No observability
- Race conditions possible
- No caching
- Architectural debt

**After:**
- Grade: A (95/100)
- Full observability
- Concurrency safe
- 80% faster with cache
- Clean architecture

**Production Ready:** ✅ YES
