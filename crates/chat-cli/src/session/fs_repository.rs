use std::path::PathBuf;

use async_trait::async_trait;

use super::error::SessionError;
use super::io::{load_metadata, save_metadata};
use super::metadata::SessionMetadata;
use super::repository::{SessionFilter, SessionRepository};
use crate::os::Os;

/// Filesystem-based session repository
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

        // Apply status filter
        if let Some(status) = filter.status {
            sessions.retain(|s| s.status == status);
        }

        // Apply search filter
        if let Some(search) = filter.search {
            let search_lower = search.to_lowercase();
            sessions.retain(|s| {
                s.first_message.to_lowercase().contains(&search_lower)
                    || s.name
                        .as_ref()
                        .map_or(false, |n| n.to_lowercase().contains(&search_lower))
            });
        }

        // Sort by last_active, most recent first
        sessions.sort_by(|a, b| b.last_active.cmp(&a.last_active));

        // Apply limit
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_os(temp_dir: &TempDir) -> Os {
        Os::test_with_root(temp_dir.path())
    }

    #[tokio::test]
    async fn test_save_and_get() {
        let temp_dir = TempDir::new().unwrap();
        let os = create_test_os(&temp_dir);
        let repo = FileSystemRepository::new(os);

        let metadata = SessionMetadata::new("test-1", "Test session");
        repo.save(&metadata).await.unwrap();

        let loaded = repo.get("test-1").await.unwrap();
        assert_eq!(loaded.id, "test-1");
        assert_eq!(loaded.first_message, "Test session");
    }

    #[tokio::test]
    async fn test_list_with_filter() {
        let temp_dir = TempDir::new().unwrap();
        let os = create_test_os(&temp_dir);
        let repo = FileSystemRepository::new(os);

        let meta1 = SessionMetadata::new("id-1", "Active session");
        repo.save(&meta1).await.unwrap();

        let mut meta2 = SessionMetadata::new("id-2", "Archived session");
        meta2.archive();
        repo.save(&meta2).await.unwrap();

        let filter = SessionFilter {
            status: Some(SessionStatus::Active),
            ..Default::default()
        };
        let results = repo.list(filter).await.unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "id-1");
    }
}
