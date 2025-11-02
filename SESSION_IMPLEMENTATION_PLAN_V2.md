# Session Management - Senior Engineer Implementation Plan

## Overview

Production-grade implementation with TDD, proper error handling, concurrency safety, performance optimization, and comprehensive testing.

## Core Principles

### No Placeholder Implementations
Every step must be **production-ready and complete**. Absolutely forbidden:
- `TODO` comments in implementation code
- `unimplemented!()` macros
- `panic!()` for unhandled cases
- Stub functions that just return `Ok(())`
- Match arms with empty implementations
- "Will implement later" comments

### Definition of "Done" for Each Step
A step is complete ONLY when:
1. All tests pass (including edge cases and error paths)
2. Implementation is production-ready (no placeholders)
3. All error cases are handled properly
4. Code is documented with rustdoc comments
5. Manual testing validates expected behavior
6. Code compiles without warnings
7. Code review checklist passes

### Small, Complete Steps
- Each step is independently deployable
- Each step adds complete functionality
- No partial implementations
- No "skeleton" code waiting to be filled in

## Pre-Implementation Phase

### Architecture Review
- [ ] Review design document with team
- [ ] Identify integration points with existing code
- [ ] Define success criteria and metrics
- [ ] Establish performance benchmarks
- [ ] Create ADRs for key decisions

### Risk Assessment
- [ ] Identify potential breaking changes
- [ ] Plan migration strategy for existing users
- [ ] Define rollback procedures
- [ ] Assess performance impact
- [ ] Security review

---

## Phase 0: Foundation & Abstractions

### Step 0.1: Define Repository Trait
**Goal:** Abstract storage layer for testability and future extensibility

**Test First:**
```rust
// crates/chat-cli/src/session/repository.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_repo_save_and_get() {
        let repo = InMemoryRepository::new();
        let metadata = SessionMetadata::new("test-1", "First message");
        
        repo.save(&metadata).await.unwrap();
        let loaded = repo.get("test-1").await.unwrap();
        
        assert_eq!(loaded.id, "test-1");
        assert_eq!(loaded.first_message, "First message");
    }

    #[tokio::test]
    async fn test_get_nonexistent_session() {
        let repo = InMemoryRepository::new();
        let result = repo.get("nonexistent").await;
        
        assert!(matches!(result, Err(SessionError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_list_empty() {
        let repo = InMemoryRepository::new();
        let sessions = repo.list(SessionFilter::default()).await.unwrap();
        
        assert!(sessions.is_empty());
    }

    #[tokio::test]
    async fn test_list_with_filter() {
        let repo = InMemoryRepository::new();
        
        let mut active = SessionMetadata::new("active-1", "Active");
        repo.save(&active).await.unwrap();
        
        let mut archived = SessionMetadata::new("archived-1", "Archived");
        archived.archive();
        repo.save(&archived).await.unwrap();
        
        let filter = SessionFilter {
            status: Some(SessionStatus::Active),
            ..Default::default()
        };
        let results = repo.list(filter).await.unwrap();
        
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "active-1");
    }

    #[tokio::test]
    async fn test_delete_session() {
        let repo = InMemoryRepository::new();
        let metadata = SessionMetadata::new("test-1", "First");
        
        repo.save(&metadata).await.unwrap();
        repo.delete("test-1").await.unwrap();
        
        let result = repo.get("test-1").await;
        assert!(matches!(result, Err(SessionError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_concurrent_save() {
        let repo = Arc::new(InMemoryRepository::new());
        let mut handles = vec![];
        
        for i in 0..10 {
            let repo_clone = Arc::clone(&repo);
            let handle = tokio::spawn(async move {
                let metadata = SessionMetadata::new(
                    format!("session-{}", i),
                    format!("Message {}", i)
                );
                repo_clone.save(&metadata).await
            });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.await.unwrap().unwrap();
        }
        
        let sessions = repo.list(SessionFilter::default()).await.unwrap();
        assert_eq!(sessions.len(), 10);
    }
}
```

**Implementation:**
```rust
// crates/chat-cli/src/session/repository.rs
use async_trait::async_trait;
use std::sync::Arc;

#[async_trait]
pub trait SessionRepository: Send + Sync {
    async fn get(&self, id: &str) -> Result<SessionMetadata, SessionError>;
    async fn save(&self, metadata: &SessionMetadata) -> Result<(), SessionError>;
    async fn delete(&self, id: &str) -> Result<(), SessionError>;
    async fn list(&self, filter: SessionFilter) -> Result<Vec<SessionMetadata>, SessionError>;
    async fn exists(&self, id: &str) -> Result<bool, SessionError>;
}

#[derive(Default)]
pub struct SessionFilter {
    pub status: Option<SessionStatus>,
    pub limit: Option<usize>,
    pub search: Option<String>,
}

// In-memory implementation for testing
pub struct InMemoryRepository {
    sessions: Arc<tokio::sync::RwLock<HashMap<String, SessionMetadata>>>,
}

impl InMemoryRepository {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl SessionRepository for InMemoryRepository {
    async fn get(&self, id: &str) -> Result<SessionMetadata, SessionError> {
        let sessions = self.sessions.read().await;
        sessions.get(id)
            .cloned()
            .ok_or_else(|| SessionError::NotFound(id.to_string()))
    }

    async fn save(&self, metadata: &SessionMetadata) -> Result<(), SessionError> {
        let mut sessions = self.sessions.write().await;
        sessions.insert(metadata.id.clone(), metadata.clone());
        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<(), SessionError> {
        let mut sessions = self.sessions.write().await;
        sessions.remove(id)
            .ok_or_else(|| SessionError::NotFound(id.to_string()))?;
        Ok(())
    }

    async fn list(&self, filter: SessionFilter) -> Result<Vec<SessionMetadata>, SessionError> {
        let sessions = self.sessions.read().await;
        let mut results: Vec<_> = sessions.values().cloned().collect();
        
        // Apply status filter
        if let Some(status) = filter.status {
            results.retain(|s| s.status == status);
        }
        
        // Apply search filter
        if let Some(search) = filter.search {
            let search_lower = search.to_lowercase();
            results.retain(|s| {
                s.first_message.to_lowercase().contains(&search_lower) ||
                s.name.as_ref().map_or(false, |n| n.to_lowercase().contains(&search_lower))
            });
        }
        
        // Sort by last_active descending
        results.sort_by(|a, b| b.last_active.cmp(&a.last_active));
        
        // Apply limit
        if let Some(limit) = filter.limit {
            results.truncate(limit);
        }
        
        Ok(results)
    }

    async fn exists(&self, id: &str) -> Result<bool, SessionError> {
        let sessions = self.sessions.read().await;
        Ok(sessions.contains_key(id))
    }
}
```

**Validation:**
- [ ] All tests pass: `cargo test repository`
- [ ] Code compiles: `cargo check`
- [ ] No warnings
- [ ] Concurrent test passes reliably

**No Placeholder Check:**
- [ ] No `TODO` comments in implementation
- [ ] No `unimplemented!()` macros
- [ ] No `panic!()` for error cases
- [ ] All match arms fully implemented
- [ ] All trait methods have real implementations
- [ ] No stub functions returning default values
- [ ] All error paths handled properly

**Code Quality:**
- [ ] Rustdoc comments on public items
- [ ] No clippy warnings: `cargo clippy`
- [ ] Code formatted: `cargo fmt --check`
- [ ] No dead code warnings

**Manual Testing:**
- [ ] Create in-memory repo
- [ ] Save multiple sessions
- [ ] Retrieve sessions
- [ ] Test filtering
- [ ] Verify concurrent access works

**Git Commit:**
```bash
git add crates/chat-cli/src/session/repository.rs
git commit -m "feat(session): add repository trait with in-memory implementation

- Define SessionRepository trait for storage abstraction
- Implement InMemoryRepository for testing
- Add comprehensive tests including concurrency
- Support filtering and searching
"
```

**User Signoff Required:** ✋
- Review trait design
- Confirm abstraction is sufficient
- Approve before proceeding

---

### Step 0.2: Error Types & Handling
**Goal:** Define comprehensive error types with user-friendly messages

**Test First:**
```rust
// crates/chat-cli/src/session/error.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_user_messages() {
        let err = SessionError::NotFound("test-123".to_string());
        assert!(err.user_message().contains("test-123"));
        assert!(err.user_message().contains("/sessions list"));
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let session_err = SessionError::from(io_err);
        
        assert!(matches!(session_err, SessionError::Storage(_)));
        assert!(session_err.user_message().contains("Permission denied"));
    }

    #[test]
    fn test_error_display() {
        let err = SessionError::Corrupted("test-123".to_string());
        let display = format!("{}", err);
        assert!(display.contains("Corrupted"));
        assert!(display.contains("test-123"));
    }

    #[test]
    fn test_error_debug() {
        let err = SessionError::ConcurrentModification;
        let debug = format!("{:?}", err);
        assert!(debug.contains("ConcurrentModification"));
    }
}
```

**Implementation:**
```rust
// crates/chat-cli/src/session/error.rs
use thiserror::Error;

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
    
    #[error("Invalid session name: {0}")]
    InvalidName(String),
}

impl SessionError {
    /// Get user-friendly error message with actionable guidance
    pub fn user_message(&self) -> String {
        match self {
            SessionError::NotFound(id) => {
                format!(
                    "Session '{}' not found.\n\
                     Use '/sessions list' to see available sessions.",
                    id
                )
            }
            SessionError::AlreadyExists(id) => {
                format!(
                    "Session '{}' already exists.\n\
                     Use '/sessions list' to see existing sessions.",
                    id
                )
            }
            SessionError::InvalidMetadata(msg) => {
                format!("Invalid session data: {}", msg)
            }
            SessionError::Corrupted(id) => {
                format!(
                    "Session '{}' data is corrupted.\n\
                     Attempting automatic recovery...",
                    id
                )
            }
            SessionError::ConcurrentModification => {
                "Another process is modifying this session.\n\
                 Please try again in a moment."
                    .to_string()
            }
            SessionError::PermissionDenied(path) => {
                format!(
                    "Permission denied accessing: {}\n\
                     Check file permissions in .amazonq/sessions/",
                    path
                )
            }
            SessionError::Storage(e) => {
                format!("Storage error: {}\nPlease check disk space and permissions.", e)
            }
            SessionError::Serialization(e) => {
                format!("Data format error: {}", e)
            }
            SessionError::InvalidName(msg) => {
                format!(
                    "Invalid session name: {}\n\
                     Names must be 1-100 characters, alphanumeric with dash/underscore only.",
                    msg
                )
            }
        }
    }

    /// Check if error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            SessionError::ConcurrentModification | SessionError::Corrupted(_)
        )
    }
}
```

**Validation:**
- [ ] Tests pass
- [ ] Error messages are clear and actionable
- [ ] All error types covered

**No Placeholder Check:**
- [ ] All error variants have user_message() implementation
- [ ] No generic "TODO" error messages
- [ ] All From implementations complete
- [ ] is_recoverable() logic implemented for all variants

**Code Quality:**
- [ ] Error messages tested
- [ ] Display and Debug traits work correctly
- [ ] Documentation explains when each error occurs

**Git Commit:**
```bash
git add crates/chat-cli/src/session/error.rs
git commit -m "feat(session): add comprehensive error types

- Define SessionError enum with all error cases
- Add user-friendly error messages
- Include recovery hints
- Test error display and conversion
"
```

**User Signoff Required:** ✋

---

## Phase 1: Core Metadata Implementation

### Step 1.1: Metadata Data Structures
**Goal:** Define session metadata with versioning and validation

**Test First:**
```rust
// crates/chat-cli/src/session/metadata.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_creation() {
        let metadata = SessionMetadata::new("test-id", "First message");
        
        assert_eq!(metadata.id, "test-id");
        assert_eq!(metadata.version, METADATA_VERSION);
        assert_eq!(metadata.status, SessionStatus::Active);
        assert_eq!(metadata.first_message, "First message");
        assert_eq!(metadata.message_count, 0);
        assert_eq!(metadata.file_count, 0);
        assert!(metadata.name.is_none());
    }

    #[test]
    fn test_metadata_serialization() {
        let metadata = SessionMetadata::new("test-id", "First message");
        let json = serde_json::to_string(&metadata).unwrap();
        let deserialized: SessionMetadata = serde_json::from_str(&json).unwrap();
        
        assert_eq!(metadata.id, deserialized.id);
        assert_eq!(metadata.version, deserialized.version);
        assert_eq!(metadata.first_message, deserialized.first_message);
    }

    #[test]
    fn test_status_transitions() {
        let mut metadata = SessionMetadata::new("test-id", "First");
        
        assert_eq!(metadata.status, SessionStatus::Active);
        
        metadata.archive();
        assert_eq!(metadata.status, SessionStatus::Archived);
    }

    #[test]
    fn test_update_activity() {
        let mut metadata = SessionMetadata::new("test-id", "First");
        let original_time = metadata.last_active;
        
        std::thread::sleep(std::time::Duration::from_millis(10));
        metadata.update_activity();
        
        assert!(metadata.last_active > original_time);
    }

    #[test]
    fn test_validate_name_valid() {
        assert!(validate_session_name("my-feature").is_ok());
        assert!(validate_session_name("feature_123").is_ok());
        assert!(validate_session_name("ABC-def_123").is_ok());
    }

    #[test]
    fn test_validate_name_invalid() {
        assert!(validate_session_name("").is_err());
        assert!(validate_session_name("a".repeat(101).as_str()).is_err());
        assert!(validate_session_name("my feature").is_err()); // space
        assert!(validate_session_name("my/feature").is_err()); // slash
        assert!(validate_session_name("my.feature").is_err()); // dot
    }

    #[test]
    fn test_set_name_validation() {
        let mut metadata = SessionMetadata::new("test", "First");
        
        assert!(metadata.set_name("valid-name").is_ok());
        assert_eq!(metadata.name, Some("valid-name".to_string()));
        
        assert!(metadata.set_name("invalid name").is_err());
        assert_eq!(metadata.name, Some("valid-name".to_string())); // unchanged
    }

    #[test]
    fn test_schema_migration_v0_to_v1() {
        // Simulate V0 metadata (no custom_fields)
        let json = r#"{
            "version": 0,
            "id": "test",
            "status": "active",
            "created": "2025-01-01T00:00:00Z",
            "last_active": "2025-01-01T00:00:00Z",
            "first_message": "Test",
            "name": null,
            "file_count": 0,
            "message_count": 0
        }"#;
        
        let mut metadata: SessionMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(metadata.version, 0);
        
        metadata = metadata.migrate().unwrap();
        assert_eq!(metadata.version, 1);
        assert!(metadata.custom_fields.is_empty());
    }

    #[test]
    fn test_schema_migration_unknown_version() {
        let mut metadata = SessionMetadata::new("test", "First");
        metadata.version = 999;
        
        let result = metadata.migrate();
        assert!(result.is_err());
    }
}
```

**Implementation:** (See next message due to length)

**Validation:**
- [ ] All tests pass
- [ ] Serialization round-trips correctly
- [ ] Validation catches all invalid inputs
- [ ] Migration logic works

**No Placeholder Check:**
- [ ] All SessionMetadata methods fully implemented
- [ ] validate_session_name() handles all edge cases
- [ ] migrate() handles all version transitions
- [ ] No "will add later" fields
- [ ] All status transitions implemented

**Code Quality:**
- [ ] Public API documented
- [ ] Migration path tested for all versions
- [ ] Validation error messages are helpful

**Git Commit:**
```bash
git add crates/chat-cli/src/session/metadata.rs
git commit -m "feat(session): add metadata structures with validation

- Define SessionMetadata with versioning
- Add validation for session names
- Implement schema migration logic
- Comprehensive tests for all operations
"
```

**User Signoff Required:** ✋

---

(Continued in next section due to length...)

---

## Universal Validation Checklist

Apply this checklist to **EVERY STEP** before committing:

### Compilation & Tests
- [ ] `cargo test` - All tests pass
- [ ] `cargo check` - Code compiles
- [ ] `cargo clippy` - No clippy warnings
- [ ] `cargo fmt --check` - Code is formatted
- [ ] `cargo test --doc` - Doc tests pass

### No Placeholders
- [ ] Search codebase for `TODO` - none in implementation
- [ ] Search codebase for `FIXME` - none in implementation
- [ ] Search codebase for `unimplemented!` - none
- [ ] Search codebase for `todo!` - none
- [ ] Search codebase for `panic!` in error paths - none
- [ ] All match arms have real implementations
- [ ] All functions have complete logic, not stubs

### Error Handling
- [ ] All Result types handled (no unwrap in production code)
- [ ] All error cases have tests
- [ ] Error messages are user-friendly
- [ ] Recovery paths implemented where applicable

### Code Quality
- [ ] Public items have rustdoc comments
- [ ] Complex logic has inline comments
- [ ] No dead code
- [ ] No unused imports
- [ ] No compiler warnings

### Testing
- [ ] Happy path tested
- [ ] Error paths tested
- [ ] Edge cases tested
- [ ] Concurrent access tested (if applicable)
- [ ] Manual testing completed

### Git Hygiene
- [ ] Commit message follows convention
- [ ] Only related changes in commit
- [ ] No debug code committed
- [ ] No commented-out code

### User Signoff
- [ ] Demonstrate functionality to user
- [ ] Show test results
- [ ] Explain implementation decisions
- [ ] Get explicit approval before proceeding

---

## Code Review Checklist

Before marking a step complete, review for:

### Implementation Completeness
- [ ] Does this solve the stated goal completely?
- [ ] Are there any "we'll add this later" comments?
- [ ] Are all code paths reachable and tested?
- [ ] Is error handling comprehensive?

### Production Readiness
- [ ] Would I deploy this to production?
- [ ] Are there any shortcuts or hacks?
- [ ] Is performance acceptable?
- [ ] Is it secure?

### Maintainability
- [ ] Can another engineer understand this?
- [ ] Is it well-documented?
- [ ] Are abstractions clear?
- [ ] Is it testable?

### Integration
- [ ] Does it work with existing code?
- [ ] Are there breaking changes?
- [ ] Is backwards compatibility maintained?

---

## Rollback Procedure

If any step fails validation:

1. **Stop immediately** - Do not proceed to next step
2. **Identify the issue** - What failed? Why?
3. **Decide: Fix or Revert**
   - If quick fix (< 30 min): Fix and re-validate
   - If complex: Revert commit and redesign
4. **Revert if needed:**
   ```bash
   git revert HEAD
   git push
   ```
5. **Document the issue** - Add to lessons learned
6. **Redesign if necessary** - Update plan before retry
7. **Re-validate completely** - Don't skip checks

---

## Anti-Patterns to Avoid

### ❌ Placeholder Code
```rust
// BAD
pub fn save_session(&self, metadata: &SessionMetadata) -> Result<()> {
    // TODO: implement this
    Ok(())
}
```

### ✅ Complete Implementation
```rust
// GOOD
pub async fn save_session(&self, metadata: &SessionMetadata) -> Result<()> {
    let session_dir = self.get_session_dir(&metadata.id)?;
    let _lock = SessionLock::acquire(&session_dir, Duration::from_secs(5)).await?;
    save_metadata_atomic(&session_dir, metadata).await?;
    Ok(())
}
```

### ❌ Incomplete Error Handling
```rust
// BAD
match result {
    Ok(data) => process(data),
    Err(_) => panic!("error occurred"), // Don't panic!
}
```

### ✅ Proper Error Handling
```rust
// GOOD
match result {
    Ok(data) => process(data),
    Err(e) => {
        error!("Failed to process: {}", e);
        return Err(SessionError::from(e));
    }
}
```

### ❌ Stub Functions
```rust
// BAD
pub fn validate_name(name: &str) -> Result<()> {
    Ok(()) // TODO: add validation
}
```

### ✅ Complete Validation
```rust
// GOOD
pub fn validate_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(SessionError::InvalidName("Name cannot be empty".into()));
    }
    if name.len() > 100 {
        return Err(SessionError::InvalidName("Name too long".into()));
    }
    if !name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err(SessionError::InvalidName("Invalid characters".into()));
    }
    Ok(())
}
```

---

## Success Criteria Summary

A step is successful when:
1. ✅ All tests pass
2. ✅ Code compiles without warnings
3. ✅ No placeholder implementations
4. ✅ Error handling is complete
5. ✅ Manual testing validates behavior
6. ✅ Code review checklist passes
7. ✅ User signoff obtained
8. ✅ Git commit is clean and descriptive

**If ANY criterion fails, the step is NOT complete.**
