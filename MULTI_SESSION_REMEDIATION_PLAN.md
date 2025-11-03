# Multi-Session Remediation Plan

## Overview

**Goal**: Address critical issues and improve code quality to production standards  
**Timeline**: 3 sprints (6 weeks)  
**Current Grade**: B+ → **Target Grade**: A

---

## Sprint 1: Critical Fixes (2 weeks)

### Priority: 🔴 CRITICAL - Must fix before production

#### Task 1.1: Fix Race Conditions (3 days)
**Issue**: Multiple locks without deadlock prevention

**Solution**:
```rust
// Before: Two separate locks
let sessions = self.sessions.lock().await;
let active_id = self.active_session_id.lock().await;

// After: Single lock for related data
struct SessionState {
    sessions: HashMap<String, ManagedSession>,
    active_id: Option<String>,
}

pub struct MultiSessionCoordinator {
    state: Arc<Mutex<SessionState>>,
    // ...
}
```

**Steps**:
1. Create `SessionState` struct
2. Combine `sessions` and `active_session_id` into single lock
3. Update all methods to use single lock
4. Add tests for concurrent access
5. Run stress tests with multiple threads

**Acceptance Criteria**:
- [ ] All session operations use single lock
- [ ] No deadlocks under concurrent load
- [ ] Tests pass with 100 concurrent operations

---

#### Task 1.2: Implement Automatic Cleanup (2 days)
**Issue**: Sessions never cleaned up, potential memory leak

**Solution**:
```rust
pub struct SessionMetadata {
    last_active: Instant,
    created_at: Instant,
    message_count: usize,
}

impl MultiSessionCoordinator {
    pub async fn cleanup_inactive_sessions(&mut self, max_age: Duration) -> Result<usize> {
        let mut state = self.state.lock().await;
        let now = Instant::now();
        
        let to_remove: Vec<_> = state.sessions
            .iter()
            .filter(|(_, session)| {
                now.duration_since(session.metadata.last_active) > max_age
            })
            .map(|(id, _)| id.clone())
            .collect();
        
        for id in &to_remove {
            state.sessions.remove(id);
            if let Some(p) = &self.persistence {
                let _ = p.delete_session(id);
            }
        }
        
        Ok(to_remove.len())
    }
    
    // Background cleanup task
    pub fn start_cleanup_task(&self, interval: Duration, max_age: Duration) {
        let coordinator = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(interval);
            loop {
                interval.tick().await;
                if let Err(e) = coordinator.cleanup_inactive_sessions(max_age).await {
                    tracing::warn!("Cleanup failed: {}", e);
                }
            }
        });
    }
}
```

**Steps**:
1. Add `SessionMetadata` with timestamps
2. Implement `cleanup_inactive_sessions()`
3. Add background cleanup task
4. Add configuration for cleanup intervals
5. Add tests for cleanup logic

**Acceptance Criteria**:
- [ ] Inactive sessions cleaned up after configurable timeout
- [ ] Background task runs without blocking
- [ ] Memory usage stays bounded under load

---

#### Task 1.3: Use Bounded Channels (1 day)
**Issue**: Unbounded channels can grow without limit

**Solution**:
```rust
// Before
let (state_tx, state_rx) = mpsc::unbounded_channel();

// After
const STATE_CHANNEL_CAPACITY: usize = 100;
let (state_tx, state_rx) = mpsc::channel(STATE_CHANNEL_CAPACITY);

// Handle backpressure
match state_tx.try_send(event) {
    Ok(_) => {},
    Err(mpsc::error::TrySendError::Full(_)) => {
        tracing::warn!("State channel full, dropping event");
        metrics::counter!("session.state_channel.dropped").increment(1);
    }
    Err(e) => return Err(e.into()),
}
```

**Steps**:
1. Replace unbounded channels with bounded
2. Add backpressure handling
3. Add metrics for dropped events
4. Add configuration for channel sizes
5. Test under high load

**Acceptance Criteria**:
- [ ] All channels have bounded capacity
- [ ] Backpressure handled gracefully
- [ ] Metrics track dropped events

---

#### Task 1.4: Add Input Validation (2 days)
**Issue**: No validation of user input

**Solution**:
```rust
const MAX_SESSION_NAME_LENGTH: usize = 64;
const MIN_SESSION_NAME_LENGTH: usize = 1;
const VALID_NAME_REGEX: &str = r"^[a-zA-Z0-9_-]+$";

pub fn validate_session_name(name: &str) -> Result<()> {
    if name.is_empty() {
        bail!("Session name cannot be empty");
    }
    if name.len() > MAX_SESSION_NAME_LENGTH {
        bail!("Session name too long (max {})", MAX_SESSION_NAME_LENGTH);
    }
    if name.len() < MIN_SESSION_NAME_LENGTH {
        bail!("Session name too short (min {})", MIN_SESSION_NAME_LENGTH);
    }
    
    let re = regex::Regex::new(VALID_NAME_REGEX).unwrap();
    if !re.is_match(name) {
        bail!("Session name contains invalid characters (use a-z, A-Z, 0-9, _, -)");
    }
    
    Ok(())
}

pub async fn switch_session(&mut self, name: &str) -> Result<()> {
    validate_session_name(name)?;
    // ... rest of implementation
}
```

**Steps**:
1. Define validation rules and constants
2. Create validation functions
3. Add validation to all public methods
4. Add tests for edge cases
5. Update error messages

**Acceptance Criteria**:
- [ ] All inputs validated before use
- [ ] Clear error messages for invalid input
- [ ] Tests cover all validation rules

---

## Sprint 2: Refactoring (2 weeks)

### Priority: ⚠️ HIGH - Improves maintainability

#### Task 2.1: Refactor God Object (5 days)
**Issue**: `MultiSessionCoordinator` has too many responsibilities

**Solution**:
```rust
// Split into focused components

pub struct SessionRegistry {
    state: Arc<Mutex<SessionState>>,
    persistence: Option<SessionPersistence>,
    lock_manager: SessionLockManager,
}

impl SessionRegistry {
    pub async fn add(&mut self, id: String, session: ManagedSession) -> Result<()> { }
    pub async fn remove(&mut self, id: &str) -> Result<()> { }
    pub async fn get(&self, id: &str) -> Option<ManagedSession> { }
    pub async fn list(&self) -> Vec<String> { }
}

pub struct SessionResourceManager {
    rate_limiter: ApiRateLimiter,
    memory_monitor: MemoryMonitor,
    cleanup_manager: ResourceCleanupManager,
}

impl SessionResourceManager {
    pub async fn check_resources(&self) -> ResourceStatus { }
    pub async fn cleanup(&self) -> Result<()> { }
}

pub struct MultiSessionCoordinator {
    registry: SessionRegistry,
    resources: SessionResourceManager,
    config: CoordinatorConfig,
}
```

**Steps**:
1. Create `SessionRegistry` for session storage
2. Create `SessionResourceManager` for resources
3. Update `MultiSessionCoordinator` to delegate
4. Update all call sites
5. Update tests

**Acceptance Criteria**:
- [ ] Each component has single responsibility
- [ ] No component > 300 lines
- [ ] All tests pass

---

#### Task 2.2: Fix Parameter Explosion (3 days)
**Issue**: `create_session()` takes 8 parameters

**Solution**:
```rust
#[derive(Debug, Clone)]
pub struct SessionConfig {
    pub conversation_id: String,
    pub session_type: SessionType,
    pub name: Option<String>,
    pub model_id: Option<String>,
}

pub struct SessionContext<'a> {
    pub os: &'a Os,
    pub agents: Agents,
    pub tool_config: HashMap<String, ToolSpec>,
    pub tool_manager: ToolManager,
}

impl MultiSessionCoordinator {
    pub async fn create_session(
        &mut self,
        config: SessionConfig,
        context: SessionContext<'_>,
    ) -> Result<String> {
        // Implementation
    }
}

// Usage
let config = SessionConfig {
    conversation_id: "abc123".to_string(),
    session_type: SessionType::Development,
    name: Some("my-session".to_string()),
    model_id: None,
};

let context = SessionContext {
    os: &os,
    agents,
    tool_config,
    tool_manager,
};

coordinator.create_session(config, context).await?;
```

**Steps**:
1. Create `SessionConfig` struct
2. Create `SessionContext` struct
3. Update `create_session()` signature
4. Update all call sites
5. Update tests with new signature

**Acceptance Criteria**:
- [ ] No method has > 4 parameters
- [ ] Config structs have builders
- [ ] All tests pass

---

#### Task 2.3: Add Structured Errors (2 days)
**Issue**: Generic error messages

**Solution**:
```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SessionError {
    #[error("Session '{0}' not found")]
    NotFound(String),
    
    #[error("Maximum sessions ({0}) reached")]
    LimitReached(usize),
    
    #[error("Session '{0}' already exists")]
    AlreadyExists(String),
    
    #[error("Invalid session name: {0}")]
    InvalidName(String),
    
    #[error("Session '{0}' is locked by {1}")]
    Locked(String, String),
    
    #[error("Persistence error: {0}")]
    Persistence(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, SessionError>;
```

**Steps**:
1. Add `thiserror` dependency
2. Define `SessionError` enum
3. Replace `eyre::Result` with `SessionError`
4. Update error handling
5. Update tests

**Acceptance Criteria**:
- [ ] All errors are typed
- [ ] Error messages are clear
- [ ] Errors can be matched programmatically

---

## Sprint 3: Quality & Observability (2 weeks)

### Priority: ⚠️ MEDIUM - Production polish

#### Task 3.1: Fix Test Coverage (4 days)
**Issue**: Many tests disabled, ~60% coverage

**Solution**:
```rust
#[cfg(test)]
mod test_helpers {
    use super::*;
    
    pub fn mock_os() -> Os {
        Os::new_mock()
    }
    
    pub fn mock_agents() -> Agents {
        Agents::default()
    }
    
    pub fn mock_tools() -> HashMap<String, ToolSpec> {
        HashMap::new()
    }
    
    pub fn mock_tool_manager() -> ToolManager {
        ToolManager::default()
    }
    
    pub fn create_test_session_config(id: &str) -> SessionConfig {
        SessionConfig {
            conversation_id: id.to_string(),
            session_type: SessionType::Development,
            name: Some(format!("test-{}", id)),
            model_id: None,
        }
    }
    
    pub fn create_test_context() -> SessionContext<'static> {
        SessionContext {
            os: Box::leak(Box::new(mock_os())),
            agents: mock_agents(),
            tool_config: mock_tools(),
            tool_manager: mock_tool_manager(),
        }
    }
}

#[tokio::test]
async fn test_create_session() {
    let mut coordinator = MultiSessionCoordinator::new(CoordinatorConfig::default());
    let config = test_helpers::create_test_session_config("test-1");
    let context = test_helpers::create_test_context();
    
    let id = coordinator.create_session(config, context).await.unwrap();
    assert_eq!(id, "test-1");
}
```

**Steps**:
1. Create test helper module
2. Implement mock factories
3. Re-enable all tests
4. Add missing test cases
5. Achieve 80%+ coverage

**Acceptance Criteria**:
- [ ] All tests enabled and passing
- [ ] Test coverage > 80%
- [ ] Integration tests cover main flows

---

#### Task 3.2: Add Observability (3 days)
**Issue**: No metrics or tracing

**Solution**:
```rust
use tracing::{info, warn, error, instrument};

impl MultiSessionCoordinator {
    #[instrument(skip(self, context))]
    pub async fn create_session(
        &mut self,
        config: SessionConfig,
        context: SessionContext<'_>,
    ) -> Result<String> {
        info!("Creating session: {:?}", config);
        
        let start = Instant::now();
        let result = self.create_session_impl(config, context).await;
        let duration = start.elapsed();
        
        match &result {
            Ok(id) => {
                info!("Session created: {}", id);
                metrics::counter!("session.create.success").increment(1);
                metrics::histogram!("session.create.duration").record(duration);
            }
            Err(e) => {
                error!("Session creation failed: {}", e);
                metrics::counter!("session.create.failure").increment(1);
            }
        }
        
        result
    }
}
```

**Steps**:
1. Add `tracing` and `metrics` dependencies
2. Add instrumentation to all public methods
3. Add metrics for key operations
4. Add structured logging
5. Document metrics

**Acceptance Criteria**:
- [ ] All public methods instrumented
- [ ] Key metrics tracked
- [ ] Logs are structured and searchable

---

#### Task 3.3: Add Named Constants (1 day)
**Issue**: Magic numbers throughout code

**Solution**:
```rust
// Configuration constants
pub mod config {
    use std::time::Duration;
    
    pub const DEFAULT_MAX_SESSIONS: usize = 10;
    pub const DEFAULT_BUFFER_SIZE_MB: usize = 10;
    pub const DEFAULT_MAX_API_CALLS: usize = 5;
    
    pub const MAX_SESSION_NAME_LENGTH: usize = 64;
    pub const MIN_SESSION_NAME_LENGTH: usize = 1;
    
    pub const CLEANUP_INTERVAL: Duration = Duration::from_secs(300); // 5 min
    pub const SESSION_TIMEOUT: Duration = Duration::from_secs(3600); // 1 hour
    
    pub const STATE_CHANNEL_CAPACITY: usize = 100;
    pub const LOCK_TIMEOUT: Duration = Duration::from_secs(30);
}
```

**Steps**:
1. Create `config` module
2. Extract all magic numbers
3. Replace with named constants
4. Document each constant
5. Update tests

**Acceptance Criteria**:
- [ ] No magic numbers in code
- [ ] All constants documented
- [ ] Constants grouped logically

---

#### Task 3.4: Documentation & Examples (2 days)
**Issue**: Missing examples and API docs

**Solution**:
```rust
/// Multi-session coordinator for managing concurrent chat sessions.
///
/// # Examples
///
/// ```
/// use chat_cli::coordinator::{MultiSessionCoordinator, CoordinatorConfig};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let mut coordinator = MultiSessionCoordinator::new(
///         CoordinatorConfig::default()
///     );
///     
///     // Create a session
///     let config = SessionConfig {
///         conversation_id: "abc123".to_string(),
///         session_type: SessionType::Development,
///         name: Some("my-session".to_string()),
///         model_id: None,
///     };
///     
///     let id = coordinator.create_session(config, context).await?;
///     
///     // Switch sessions
///     coordinator.switch_session("my-session").await?;
///     
///     Ok(())
/// }
/// ```
pub struct MultiSessionCoordinator { }
```

**Steps**:
1. Add examples to all public APIs
2. Create examples/ directory
3. Write integration examples
4. Update user guide
5. Generate and review docs

**Acceptance Criteria**:
- [ ] All public APIs have examples
- [ ] 3+ integration examples
- [ ] Docs build without warnings

---

## Timeline Summary

```
Sprint 1 (Weeks 1-2): Critical Fixes
├─ Week 1
│  ├─ Task 1.1: Fix race conditions (3d)
│  └─ Task 1.2: Automatic cleanup (2d)
└─ Week 2
   ├─ Task 1.3: Bounded channels (1d)
   ├─ Task 1.4: Input validation (2d)
   └─ Buffer (2d)

Sprint 2 (Weeks 3-4): Refactoring
├─ Week 3
│  └─ Task 2.1: Refactor God Object (5d)
└─ Week 4
   ├─ Task 2.2: Fix parameters (3d)
   └─ Task 2.3: Structured errors (2d)

Sprint 3 (Weeks 5-6): Quality
├─ Week 5
│  └─ Task 3.1: Test coverage (4d)
└─ Week 6
   ├─ Task 3.2: Observability (3d)
   ├─ Task 3.3: Constants (1d)
   └─ Task 3.4: Documentation (2d)
```

---

## Success Metrics

### Sprint 1 (Critical)
- [ ] No deadlocks under load
- [ ] Memory usage bounded
- [ ] All inputs validated
- [ ] Channels bounded

### Sprint 2 (Refactoring)
- [ ] No component > 300 lines
- [ ] No method > 4 parameters
- [ ] All errors typed
- [ ] All tests pass

### Sprint 3 (Quality)
- [ ] Test coverage > 80%
- [ ] All methods instrumented
- [ ] No magic numbers
- [ ] Docs complete

### Final Grade: A
- Code quality: Excellent
- Test coverage: > 80%
- Documentation: Complete
- Production ready: Yes

---

## Risk Mitigation

### Risk 1: Breaking Changes
**Mitigation**: 
- Feature flag for new code
- Parallel implementation
- Gradual migration

### Risk 2: Performance Regression
**Mitigation**:
- Benchmark before/after
- Load testing
- Rollback plan

### Risk 3: Timeline Slip
**Mitigation**:
- Focus on Sprint 1 (critical)
- Sprint 2/3 can be deferred
- Incremental delivery

---

## Approval Gates

### Gate 1: After Sprint 1
- [ ] All critical issues fixed
- [ ] Load tests pass
- [ ] Security review complete
- **Decision**: Go/No-go for production

### Gate 2: After Sprint 2
- [ ] Refactoring complete
- [ ] All tests pass
- [ ] Code review approved
- **Decision**: Merge to main

### Gate 3: After Sprint 3
- [ ] Quality metrics met
- [ ] Documentation complete
- [ ] Final review approved
- **Decision**: Release to production

---

## Resources Required

- **Engineering**: 1 senior engineer (full-time, 6 weeks)
- **Testing**: QA support (part-time, weeks 2, 4, 6)
- **Review**: Tech lead review (2 hours/week)
- **Total Effort**: ~30 engineering days

---

## Conclusion

This plan addresses all identified issues systematically:
- **Sprint 1**: Fixes critical bugs (production blockers)
- **Sprint 2**: Improves code quality (maintainability)
- **Sprint 3**: Adds polish (production excellence)

After completion, the code will be production-ready with grade A quality.
