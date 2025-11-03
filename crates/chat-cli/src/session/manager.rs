use std::path::PathBuf;

use tracing::{
    debug,
    info,
    instrument,
    warn,
};

use super::error::SessionError;
use super::io::{
    load_metadata,
    save_metadata,
};
use super::metadata::{
    SessionMetadata,
    SessionStatus,
};
use crate::os::Os;

/// Session manager for high-level session operations
pub struct SessionManager<'a> {
    os: &'a Os,
}

impl<'a> SessionManager<'a> {
    pub fn new(os: &'a Os) -> Self {
        Self { os }
    }

    /// List all sessions from the filesystem
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
                    },
                    Err(e) => {
                        warn!(path = ?entry.path(), error = %e, "Failed to load session metadata");
                    },
                }
            }
        }

        // Sort by last_active, most recent first
        sessions.sort_by(|a, b| b.last_active.cmp(&a.last_active));

        info!(count = sessions.len(), "Listed sessions successfully");
        tracing::Span::current().record("session_count", sessions.len());

        Ok(sessions)
    }

    /// List sessions filtered by status
    #[instrument(skip(self))]
    pub async fn list_by_status(&self, status: SessionStatus) -> Result<Vec<SessionMetadata>, SessionError> {
        debug!(?status, "Listing sessions by status");
        let all_sessions = self.list_sessions().await?;
        let filtered: Vec<_> = all_sessions.into_iter().filter(|s| s.status == status).collect();
        info!(status = ?status, count = filtered.len(), "Filtered sessions by status");
        Ok(filtered)
    }

    /// Get a specific session by ID
    #[instrument(skip(self))]
    pub async fn get_session(&self, session_id: &str) -> Result<SessionMetadata, SessionError> {
        debug!(session_id, "Getting session");
        let session_dir = self.session_dir(session_id)?;
        let metadata = load_metadata(&session_dir).await?;
        info!(session_id, "Retrieved session successfully");
        Ok(metadata)
    }

    /// Archive a session
    #[instrument(skip(self))]
    pub async fn archive_session(&self, session_id: &str) -> Result<(), SessionError> {
        info!(session_id, "Archiving session");
        let session_dir = self.session_dir(session_id)?;
        let mut metadata = load_metadata(&session_dir).await?;
        metadata.archive();
        save_metadata(&session_dir, &metadata).await?;
        info!(session_id, "Session archived successfully");
        Ok(())
    }

    /// Name a session
    #[instrument(skip(self))]
    pub async fn name_session(&self, session_id: &str, name: impl Into<String>) -> Result<(), SessionError> {
        let name = name.into();
        info!(session_id, name = %name, "Naming session");
        let session_dir = self.session_dir(session_id)?;
        let mut metadata = load_metadata(&session_dir).await?;
        metadata.set_name(name)?;
        save_metadata(&session_dir, &metadata).await?;
        info!(session_id, "Session named successfully");
        Ok(())
    }

    /// Get the directory path for a session
    fn session_dir(&self, session_id: &str) -> Result<PathBuf, SessionError> {
        Ok(self.os.env.current_dir()?.join(".amazonq/sessions").join(session_id))
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    fn create_test_os(temp_dir: &TempDir) -> Os {
        Os::test_with_root(temp_dir.path())
    }

    #[tokio::test]
    async fn test_list_sessions_empty() {
        let temp_dir = TempDir::new().unwrap();
        let os = create_test_os(&temp_dir);
        let manager = SessionManager::new(&os);

        let sessions = manager.list_sessions().await.unwrap();
        assert!(sessions.is_empty());
    }

    #[tokio::test]
    async fn test_list_sessions_with_data() {
        let temp_dir = TempDir::new().unwrap();
        let os = create_test_os(&temp_dir);

        // Create test sessions
        let session1_dir = temp_dir.path().join(".amazonq/sessions/session-1");
        let metadata1 = SessionMetadata::new("session-1", "First session");
        save_metadata(&session1_dir, &metadata1).await.unwrap();

        let session2_dir = temp_dir.path().join(".amazonq/sessions/session-2");
        let metadata2 = SessionMetadata::new("session-2", "Second session");
        save_metadata(&session2_dir, &metadata2).await.unwrap();

        let manager = SessionManager::new(&os);
        let sessions = manager.list_sessions().await.unwrap();

        assert_eq!(sessions.len(), 2);
    }

    #[tokio::test]
    async fn test_list_sessions_sorted_by_last_active() {
        let temp_dir = TempDir::new().unwrap();
        let os = create_test_os(&temp_dir);

        // Create old session
        let old_dir = temp_dir.path().join(".amazonq/sessions/old");
        let mut old_meta = SessionMetadata::new("old", "Old session");
        old_meta.last_active = time::OffsetDateTime::now_utc() - time::Duration::hours(2);
        save_metadata(&old_dir, &old_meta).await.unwrap();

        // Create new session
        let new_dir = temp_dir.path().join(".amazonq/sessions/new");
        let new_meta = SessionMetadata::new("new", "New session");
        save_metadata(&new_dir, &new_meta).await.unwrap();

        let manager = SessionManager::new(&os);
        let sessions = manager.list_sessions().await.unwrap();

        assert_eq!(sessions.len(), 2);
        assert_eq!(sessions[0].id, "new"); // Most recent first
        assert_eq!(sessions[1].id, "old");
    }

    #[tokio::test]
    async fn test_list_by_status() {
        let temp_dir = TempDir::new().unwrap();
        let os = create_test_os(&temp_dir);

        // Create active session
        let active_dir = temp_dir.path().join(".amazonq/sessions/active-1");
        let active_meta = SessionMetadata::new("active-1", "Active");
        save_metadata(&active_dir, &active_meta).await.unwrap();

        // Create archived session
        let archived_dir = temp_dir.path().join(".amazonq/sessions/archived-1");
        let mut archived_meta = SessionMetadata::new("archived-1", "Archived");
        archived_meta.archive();
        save_metadata(&archived_dir, &archived_meta).await.unwrap();

        let manager = SessionManager::new(&os);
        let active_sessions = manager.list_by_status(SessionStatus::Active).await.unwrap();

        assert_eq!(active_sessions.len(), 1);
        assert_eq!(active_sessions[0].id, "active-1");
    }

    #[tokio::test]
    async fn test_get_session() {
        let temp_dir = TempDir::new().unwrap();
        let os = create_test_os(&temp_dir);

        let session_dir = temp_dir.path().join(".amazonq/sessions/test-1");
        let metadata = SessionMetadata::new("test-1", "Test session");
        save_metadata(&session_dir, &metadata).await.unwrap();

        let manager = SessionManager::new(&os);
        let loaded = manager.get_session("test-1").await.unwrap();

        assert_eq!(loaded.id, "test-1");
        assert_eq!(loaded.first_message, "Test session");
    }

    #[tokio::test]
    async fn test_get_nonexistent_session() {
        let temp_dir = TempDir::new().unwrap();
        let os = create_test_os(&temp_dir);
        let manager = SessionManager::new(&os);

        let result = manager.get_session("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_archive_session() {
        let temp_dir = TempDir::new().unwrap();
        let os = create_test_os(&temp_dir);

        let session_dir = temp_dir.path().join(".amazonq/sessions/test-1");
        let metadata = SessionMetadata::new("test-1", "Test");
        save_metadata(&session_dir, &metadata).await.unwrap();

        let manager = SessionManager::new(&os);
        manager.archive_session("test-1").await.unwrap();

        let updated = load_metadata(&session_dir).await.unwrap();
        assert_eq!(updated.status, SessionStatus::Archived);
    }

    #[tokio::test]
    async fn test_name_session() {
        let temp_dir = TempDir::new().unwrap();
        let os = create_test_os(&temp_dir);

        let session_dir = temp_dir.path().join(".amazonq/sessions/test-1");
        let metadata = SessionMetadata::new("test-1", "Test");
        save_metadata(&session_dir, &metadata).await.unwrap();

        let manager = SessionManager::new(&os);
        manager.name_session("test-1", "My Session").await.unwrap();

        let updated = load_metadata(&session_dir).await.unwrap();
        assert_eq!(updated.name, Some("My Session".to_string()));
    }

    #[tokio::test]
    async fn test_name_session_validation() {
        let temp_dir = TempDir::new().unwrap();
        let os = create_test_os(&temp_dir);

        let session_dir = temp_dir.path().join(".amazonq/sessions/test-1");
        let metadata = SessionMetadata::new("test-1", "Test");
        save_metadata(&session_dir, &metadata).await.unwrap();

        let manager = SessionManager::new(&os);

        // Invalid name should fail
        let result = manager.name_session("test-1", "invalid name with spaces").await;
        assert!(result.is_err());

        // Original name should be unchanged
        let unchanged = load_metadata(&session_dir).await.unwrap();
        assert_eq!(unchanged.name, None);
    }

    #[tokio::test]
    async fn test_list_ignores_corrupted_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let os = create_test_os(&temp_dir);

        // Create valid session
        let valid_dir = temp_dir.path().join(".amazonq/sessions/valid");
        let valid_meta = SessionMetadata::new("valid", "Valid");
        save_metadata(&valid_dir, &valid_meta).await.unwrap();

        // Create corrupted session
        let corrupted_dir = temp_dir.path().join(".amazonq/sessions/corrupted");
        tokio::fs::create_dir_all(&corrupted_dir).await.unwrap();
        tokio::fs::write(corrupted_dir.join("metadata.json"), "corrupted{{{")
            .await
            .unwrap();

        let manager = SessionManager::new(&os);
        let sessions = manager.list_sessions().await.unwrap();

        // Should only return valid session
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id, "valid");
    }
}
