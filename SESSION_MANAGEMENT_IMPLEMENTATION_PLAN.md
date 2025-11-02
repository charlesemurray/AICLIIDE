# Session Management - TDD Implementation Plan

## Principles

- **TDD**: Write tests first, then implementation
- **Small steps**: Each step is independently testable and compilable
- **No placeholders**: Every implementation is complete and functional
- **Git commits**: Commit after each completed step
- **User signoff**: Analysis and validation before proceeding
- **Always compiling**: Code must compile at every step

## Phase 1: Metadata Infrastructure

### Step 1.1: Define Metadata Schema
**Goal:** Create the data structures for session metadata

**Test First:**
```rust
// crates/chat-cli/src/util/session_metadata.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_creation() {
        let metadata = SessionMetadata::new("test-id", "First message");
        assert_eq!(metadata.id, "test-id");
        assert_eq!(metadata.status, SessionStatus::Active);
        assert_eq!(metadata.first_message, "First message");
    }

    #[test]
    fn test_metadata_serialization() {
        let metadata = SessionMetadata::new("test-id", "First message");
        let json = serde_json::to_string(&metadata).unwrap();
        let deserialized: SessionMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(metadata.id, deserialized.id);
    }

    #[test]
    fn test_status_transitions() {
        let mut metadata = SessionMetadata::new("test-id", "First message");
        metadata.archive();
        assert_eq!(metadata.status, SessionStatus::Archived);
    }
}
```

**Implementation:**
```rust
// crates/chat-cli/src/util/session_metadata.rs
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionStatus {
    Active,
    Background,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub id: String,
    pub status: SessionStatus,
    #[serde(with = "time::serde::rfc3339")]
    pub created: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub last_active: OffsetDateTime,
    pub first_message: String,
    pub name: Option<String>,
    pub file_count: usize,
    pub message_count: usize,
}

impl SessionMetadata {
    pub fn new(id: impl Into<String>, first_message: impl Into<String>) -> Self {
        let now = OffsetDateTime::now_utc();
        Self {
            id: id.into(),
            status: SessionStatus::Active,
            created: now,
            last_active: now,
            first_message: first_message.into(),
            name: None,
            file_count: 0,
            message_count: 0,
        }
    }

    pub fn archive(&mut self) {
        self.status = SessionStatus::Archived;
        self.last_active = OffsetDateTime::now_utc();
    }

    pub fn update_activity(&mut self) {
        self.last_active = OffsetDateTime::now_utc();
    }
}
```

**Validation:**
- [ ] Tests pass: `cargo test session_metadata`
- [ ] Code compiles: `cargo check`
- [ ] No warnings in new code

**Git Commit:**
```bash
git add crates/chat-cli/src/util/session_metadata.rs
git commit -m "feat: add session metadata data structures with tests"
```

**User Signoff Required:** ✋
- Review metadata schema
- Confirm fields are sufficient
- Approve before proceeding

---

### Step 1.2: Metadata File I/O
**Goal:** Read and write metadata files

**Test First:**
```rust
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_save_metadata() {
        let temp_dir = tempfile::tempdir().unwrap();
        let session_dir = temp_dir.path().join("test-session");
        
        let metadata = SessionMetadata::new("test-id", "First message");
        save_metadata(&session_dir, &metadata).await.unwrap();
        
        let metadata_path = session_dir.join("metadata.json");
        assert!(metadata_path.exists());
    }

    #[tokio::test]
    async fn test_load_metadata() {
        let temp_dir = tempfile::tempdir().unwrap();
        let session_dir = temp_dir.path().join("test-session");
        
        let original = SessionMetadata::new("test-id", "First message");
        save_metadata(&session_dir, &original).await.unwrap();
        
        let loaded = load_metadata(&session_dir).await.unwrap();
        assert_eq!(original.id, loaded.id);
        assert_eq!(original.first_message, loaded.first_message);
    }

    #[tokio::test]
    async fn test_load_missing_metadata() {
        let temp_dir = tempfile::tempdir().unwrap();
        let session_dir = temp_dir.path().join("nonexistent");
        
        let result = load_metadata(&session_dir).await;
        assert!(result.is_err());
    }
}
```

**Implementation:**
```rust
use std::path::Path;
use eyre::Result;

pub async fn save_metadata(session_dir: &Path, metadata: &SessionMetadata) -> Result<()> {
    tokio::fs::create_dir_all(session_dir).await?;
    let metadata_path = session_dir.join("metadata.json");
    let json = serde_json::to_string_pretty(metadata)?;
    tokio::fs::write(metadata_path, json).await?;
    Ok(())
}

pub async fn load_metadata(session_dir: &Path) -> Result<SessionMetadata> {
    let metadata_path = session_dir.join("metadata.json");
    let json = tokio::fs::read_to_string(metadata_path).await?;
    let metadata = serde_json::from_str(&json)?;
    Ok(metadata)
}
```

**Validation:**
- [ ] Tests pass: `cargo test session_metadata`
- [ ] Code compiles: `cargo check`
- [ ] Manual test: Create and read metadata file

**Git Commit:**
```bash
git add crates/chat-cli/src/util/session_metadata.rs
git commit -m "feat: add metadata file I/O with tests"
```

**User Signoff Required:** ✋
- Verify metadata files are created correctly
- Check JSON format is readable
- Approve before proceeding

---

### Step 1.3: Integrate Metadata Creation
**Goal:** Create metadata when starting a new conversation

**Test First:**
```rust
// In conversation.rs tests
#[tokio::test]
async fn test_conversation_creates_metadata() {
    let temp_dir = tempfile::tempdir().unwrap();
    let os = Os::test_with_root(temp_dir.path());
    
    let conversation_id = "test-conv-id";
    let conversation = ConversationState::new(
        conversation_id,
        Agents::default(),
        HashMap::new(),
        ToolManager::default(),
        None,
    ).await;
    
    // Simulate first message
    conversation.create_session_metadata("First test message", &os).await.unwrap();
    
    let session_dir = temp_dir.path().join(".amazonq/sessions").join(conversation_id);
    let metadata = load_metadata(&session_dir).await.unwrap();
    assert_eq!(metadata.id, conversation_id);
    assert_eq!(metadata.first_message, "First test message");
}
```

**Implementation:**
```rust
// In conversation.rs
use crate::util::session_metadata::{SessionMetadata, save_metadata};

impl ConversationState {
    pub async fn create_session_metadata(&self, first_message: &str, os: &Os) -> Result<()> {
        let session_dir = os.env.current_dir()?
            .join(".amazonq/sessions")
            .join(&self.conversation_id);
        
        let metadata = SessionMetadata::new(&self.conversation_id, first_message);
        save_metadata(&session_dir, &metadata).await?;
        Ok(())
    }
    
    pub async fn update_session_metadata(&self, os: &Os) -> Result<()> {
        let session_dir = os.env.current_dir()?
            .join(".amazonq/sessions")
            .join(&self.conversation_id);
        
        if let Ok(mut metadata) = load_metadata(&session_dir).await {
            metadata.update_activity();
            metadata.message_count += 1;
            save_metadata(&session_dir, &metadata).await?;
        }
        Ok(())
    }
}
```

**Validation:**
- [ ] Tests pass: `cargo test conversation`
- [ ] Code compiles: `cargo check`
- [ ] Integration test: Start Q, send message, check metadata file

**Git Commit:**
```bash
git add crates/chat-cli/src/cli/chat/conversation.rs
git commit -m "feat: integrate metadata creation in conversations"
```

**User Signoff Required:** ✋
- Test manually: Start Q CLI, send message
- Verify metadata.json is created in `.amazonq/sessions/{id}/`
- Check metadata content is correct
- Approve before proceeding

---

## Phase 2: Session Listing

### Step 2.1: List All Sessions
**Goal:** Read all session directories and their metadata

**Test First:**
```rust
// crates/chat-cli/src/util/session_manager.rs
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_list_sessions_empty() {
        let temp_dir = tempfile::tempdir().unwrap();
        let os = Os::test_with_root(temp_dir.path());
        
        let sessions = list_sessions(&os).await.unwrap();
        assert!(sessions.is_empty());
    }

    #[tokio::test]
    async fn test_list_sessions_with_data() {
        let temp_dir = tempfile::tempdir().unwrap();
        let os = Os::test_with_root(temp_dir.path());
        
        // Create test sessions
        let session1_dir = temp_dir.path().join(".amazonq/sessions/session-1");
        let metadata1 = SessionMetadata::new("session-1", "First session");
        save_metadata(&session1_dir, &metadata1).await.unwrap();
        
        let session2_dir = temp_dir.path().join(".amazonq/sessions/session-2");
        let metadata2 = SessionMetadata::new("session-2", "Second session");
        save_metadata(&session2_dir, &metadata2).await.unwrap();
        
        let sessions = list_sessions(&os).await.unwrap();
        assert_eq!(sessions.len(), 2);
    }

    #[tokio::test]
    async fn test_filter_sessions_by_status() {
        let temp_dir = tempfile::tempdir().unwrap();
        let os = Os::test_with_root(temp_dir.path());
        
        // Create active and archived sessions
        let active_dir = temp_dir.path().join(".amazonq/sessions/active-1");
        let active_meta = SessionMetadata::new("active-1", "Active");
        save_metadata(&active_dir, &active_meta).await.unwrap();
        
        let archived_dir = temp_dir.path().join(".amazonq/sessions/archived-1");
        let mut archived_meta = SessionMetadata::new("archived-1", "Archived");
        archived_meta.archive();
        save_metadata(&archived_dir, &archived_meta).await.unwrap();
        
        let active_sessions = list_sessions_by_status(&os, SessionStatus::Active).await.unwrap();
        assert_eq!(active_sessions.len(), 1);
        assert_eq!(active_sessions[0].id, "active-1");
    }
}
```

**Implementation:**
```rust
// crates/chat-cli/src/util/session_manager.rs
use std::path::PathBuf;
use eyre::Result;
use crate::os::Os;
use crate::util::session_metadata::{SessionMetadata, SessionStatus, load_metadata};

pub async fn list_sessions(os: &Os) -> Result<Vec<SessionMetadata>> {
    let sessions_dir = os.env.current_dir()?.join(".amazonq/sessions");
    
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
    
    // Sort by last_active, most recent first
    sessions.sort_by(|a, b| b.last_active.cmp(&a.last_active));
    
    Ok(sessions)
}

pub async fn list_sessions_by_status(os: &Os, status: SessionStatus) -> Result<Vec<SessionMetadata>> {
    let all_sessions = list_sessions(os).await?;
    Ok(all_sessions.into_iter().filter(|s| s.status == status).collect())
}
```

**Validation:**
- [ ] Tests pass: `cargo test session_manager`
- [ ] Code compiles: `cargo check`
- [ ] Manual test: Create multiple sessions, list them

**Git Commit:**
```bash
git add crates/chat-cli/src/util/session_manager.rs crates/chat-cli/src/util/mod.rs
git commit -m "feat: add session listing functionality with tests"
```

**User Signoff Required:** ✋
- Review session listing logic
- Test with multiple sessions
- Verify sorting and filtering
- Approve before proceeding

---

### Step 2.2: Update `/sessions list` Command
**Goal:** Replace HashMap with filesystem-based listing

**Test First:**
```rust
// In sessions.rs
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_sessions_list_command() {
        let temp_dir = tempfile::tempdir().unwrap();
        let os = Os::test_with_root(temp_dir.path());
        
        // Create test sessions
        let session_dir = temp_dir.path().join(".amazonq/sessions/test-1");
        let metadata = SessionMetadata::new("test-1", "Test session");
        save_metadata(&session_dir, &metadata).await.unwrap();
        
        let mut session = create_test_chat_session(&os);
        let result = SessionsSubcommand::List.execute(&mut session, &os).await;
        
        assert!(result.is_ok());
        // Verify output contains session info
    }
}
```

**Implementation:**
```rust
// In sessions.rs
use crate::util::session_manager::{list_sessions_by_status, SessionStatus};

impl SessionsSubcommand {
    pub async fn execute(&self, chat_session: &mut ChatSession, os: &Os) -> Result<ChatState, ChatError> {
        match self {
            SessionsSubcommand::List => {
                let active_sessions = list_sessions_by_status(os, SessionStatus::Active).await
                    .map_err(|e| ChatError::Other(e.to_string()))?;
                
                println!("📋 Active Sessions:");
                if active_sessions.is_empty() {
                    println!("  • No active sessions");
                } else {
                    for (idx, session) in active_sessions.iter().enumerate() {
                        let name = session.name.as_deref().unwrap_or(&session.id[..8]);
                        let age = format_duration(session.last_active);
                        let is_current = session.id == chat_session.conversation.conversation_id();
                        let marker = if is_current { " (current)" } else { "" };
                        println!("  {}. {}{} - \"{}\" ({} ago, {} messages)", 
                            idx + 1, name, marker, session.first_message, age, session.message_count);
                    }
                }
                
                Ok(ChatState::PromptUser { skip_printing_tools: true })
            },
            // ... other commands
        }
    }
}

fn format_duration(timestamp: OffsetDateTime) -> String {
    let now = OffsetDateTime::now_utc();
    let duration = now - timestamp;
    
    if duration.whole_days() > 0 {
        format!("{} days", duration.whole_days())
    } else if duration.whole_hours() > 0 {
        format!("{} hours", duration.whole_hours())
    } else {
        format!("{} minutes", duration.whole_minutes())
    }
}
```

**Validation:**
- [ ] Tests pass: `cargo test sessions`
- [ ] Code compiles: `cargo check`
- [ ] Manual test: Run `/sessions list` in Q CLI
- [ ] Verify output format is readable

**Git Commit:**
```bash
git add crates/chat-cli/src/cli/chat/cli/sessions.rs
git commit -m "feat: update /sessions list to use filesystem-based listing"
```

**User Signoff Required:** ✋
- Test `/sessions list` command manually
- Verify output is clear and useful
- Check performance with many sessions
- Approve before proceeding

---

### Step 2.3: Add `/sessions history` Command
**Goal:** List archived sessions

**Test First:**
```rust
#[tokio::test]
async fn test_sessions_history_command() {
    let temp_dir = tempfile::tempdir().unwrap();
    let os = Os::test_with_root(temp_dir.path());
    
    // Create archived session
    let session_dir = temp_dir.path().join(".amazonq/sessions/archived-1");
    let mut metadata = SessionMetadata::new("archived-1", "Old session");
    metadata.archive();
    save_metadata(&session_dir, &metadata).await.unwrap();
    
    let mut session = create_test_chat_session(&os);
    let result = SessionsSubcommand::History { limit: None, search: None }
        .execute(&mut session, &os).await;
    
    assert!(result.is_ok());
}
```

**Implementation:**
```rust
#[derive(Debug, PartialEq, Subcommand)]
pub enum SessionsSubcommand {
    // ... existing commands
    
    /// Show historical (archived) sessions
    History {
        /// Limit number of results
        #[arg(long, default_value = "10")]
        limit: Option<usize>,
        /// Search term to filter sessions
        #[arg(long)]
        search: Option<String>,
    },
}

impl SessionsSubcommand {
    pub async fn execute(&self, chat_session: &mut ChatSession, os: &Os) -> Result<ChatState, ChatError> {
        match self {
            // ... existing commands
            
            SessionsSubcommand::History { limit, search } => {
                let mut archived_sessions = list_sessions_by_status(os, SessionStatus::Archived).await
                    .map_err(|e| ChatError::Other(e.to_string()))?;
                
                // Apply search filter
                if let Some(term) = search {
                    archived_sessions.retain(|s| 
                        s.first_message.to_lowercase().contains(&term.to_lowercase()) ||
                        s.name.as_ref().map_or(false, |n| n.to_lowercase().contains(&term.to_lowercase()))
                    );
                }
                
                // Apply limit
                if let Some(n) = limit {
                    archived_sessions.truncate(*n);
                }
                
                println!("📚 Session History:");
                if archived_sessions.is_empty() {
                    println!("  No archived sessions found");
                } else {
                    for (idx, session) in archived_sessions.iter().enumerate() {
                        let name = session.name.as_deref().unwrap_or(&session.id[..8]);
                        let age = format_duration(session.last_active);
                        println!("  {}. {} - \"{}\" ({} ago, {} files)", 
                            idx + 1, name, session.first_message, age, session.file_count);
                    }
                }
                
                Ok(ChatState::PromptUser { skip_printing_tools: true })
            },
        }
    }
}
```

**Validation:**
- [ ] Tests pass: `cargo test sessions`
- [ ] Code compiles: `cargo check`
- [ ] Manual test: Archive a session, run `/sessions history`
- [ ] Test search functionality

**Git Commit:**
```bash
git add crates/chat-cli/src/cli/chat/cli/sessions.rs
git commit -m "feat: add /sessions history command with search"
```

**User Signoff Required:** ✋
- Test history listing
- Verify search works correctly
- Check limit parameter
- Approve before proceeding

---

## Phase 3: Session Operations

### Step 3.1: Archive Session
**Goal:** Move active session to archived status

**Test First:**
```rust
#[tokio::test]
async fn test_archive_session() {
    let temp_dir = tempfile::tempdir().unwrap();
    let os = Os::test_with_root(temp_dir.path());
    
    // Create active session
    let session_dir = temp_dir.path().join(".amazonq/sessions/test-1");
    let metadata = SessionMetadata::new("test-1", "Test");
    save_metadata(&session_dir, &metadata).await.unwrap();
    
    // Archive it
    archive_session(&os, "test-1").await.unwrap();
    
    // Verify status changed
    let updated = load_metadata(&session_dir).await.unwrap();
    assert_eq!(updated.status, SessionStatus::Archived);
}
```

**Implementation:**
```rust
// In session_manager.rs
pub async fn archive_session(os: &Os, session_id: &str) -> Result<()> {
    let session_dir = os.env.current_dir()?
        .join(".amazonq/sessions")
        .join(session_id);
    
    let mut metadata = load_metadata(&session_dir).await?;
    metadata.archive();
    save_metadata(&session_dir, &metadata).await?;
    
    Ok(())
}
```

**Validation:**
- [ ] Tests pass
- [ ] Code compiles
- [ ] Manual test: Archive a session

**Git Commit:**
```bash
git add crates/chat-cli/src/util/session_manager.rs
git commit -m "feat: add session archiving functionality"
```

**User Signoff Required:** ✋

---

### Step 3.2: Name Session
**Goal:** Allow users to name sessions

**Test First:**
```rust
#[tokio::test]
async fn test_name_session() {
    let temp_dir = tempfile::tempdir().unwrap();
    let os = Os::test_with_root(temp_dir.path());
    
    let session_dir = temp_dir.path().join(".amazonq/sessions/test-1");
    let metadata = SessionMetadata::new("test-1", "Test");
    save_metadata(&session_dir, &metadata).await.unwrap();
    
    name_session(&os, "test-1", "my-feature").await.unwrap();
    
    let updated = load_metadata(&session_dir).await.unwrap();
    assert_eq!(updated.name, Some("my-feature".to_string()));
}
```

**Implementation:**
```rust
pub async fn name_session(os: &Os, session_id: &str, name: &str) -> Result<()> {
    let session_dir = os.env.current_dir()?
        .join(".amazonq/sessions")
        .join(session_id);
    
    let mut metadata = load_metadata(&session_dir).await?;
    metadata.name = Some(name.to_string());
    metadata.update_activity();
    save_metadata(&session_dir, &metadata).await?;
    
    Ok(())
}
```

**Validation:**
- [ ] Tests pass
- [ ] Code compiles
- [ ] Manual test: Name a session, verify in list

**Git Commit:**
```bash
git add crates/chat-cli/src/util/session_manager.rs
git commit -m "feat: add session naming functionality"
```

**User Signoff Required:** ✋

---

## Phase 4: Background Sessions (Future)

**Note:** Background sessions require more design work around:
- Process management
- State persistence across restarts
- Notification system
- Resource management

**Defer to separate implementation plan after Phase 3 is complete and validated.**

---

## Testing Strategy

### Unit Tests
- Every function has tests
- Test happy path and error cases
- Use `tempfile` for filesystem tests
- Mock `Os` for isolation

### Integration Tests
- Test full command workflows
- Verify file system state
- Test cross-module interactions

### Manual Testing Checklist
After each phase:
- [ ] Start Q CLI
- [ ] Create multiple sessions
- [ ] Run all new commands
- [ ] Verify metadata files
- [ ] Check error handling
- [ ] Test edge cases (empty dirs, missing files, etc.)

---

## Validation Checklist (Per Step)

Before committing:
- [ ] All tests pass: `cargo test`
- [ ] Code compiles: `cargo check`
- [ ] No new warnings: `cargo clippy`
- [ ] Code formatted: `cargo fmt`
- [ ] Manual testing complete
- [ ] User signoff obtained

---

## Rollback Plan

If any step fails:
1. Revert the last commit: `git revert HEAD`
2. Analyze the failure
3. Fix the issue
4. Re-run tests
5. Commit the fix

---

## Success Criteria

### Phase 1 Complete
- [ ] Metadata files created for all new sessions
- [ ] Metadata persists across Q CLI restarts
- [ ] All tests passing

### Phase 2 Complete
- [ ] `/sessions list` shows active sessions from filesystem
- [ ] `/sessions history` shows archived sessions
- [ ] Search and filtering work correctly
- [ ] All tests passing

### Phase 3 Complete
- [ ] Can archive sessions
- [ ] Can name sessions
- [ ] Session operations reflected in listings
- [ ] All tests passing

---

## Timeline Estimate

- **Phase 1:** 4-6 hours (3 steps × 1.5-2 hours each)
- **Phase 2:** 4-6 hours (3 steps × 1.5-2 hours each)
- **Phase 3:** 2-3 hours (2 steps × 1-1.5 hours each)

**Total:** 10-15 hours for Phases 1-3

---

## Next Steps

1. Review this plan
2. Get user approval
3. Start with Step 1.1
4. Follow TDD process strictly
5. Get signoff at each checkpoint
6. Commit frequently
7. Validate continuously
