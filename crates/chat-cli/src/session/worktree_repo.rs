use std::path::{
    Path,
    PathBuf,
};

use async_trait::async_trait;

use super::error::SessionError;
use super::io::{
    load_metadata,
    save_metadata,
};
use super::metadata::SessionMetadata;
use super::repository::{
    SessionFilter,
    SessionRepository,
};
use crate::git::detect_git_context;

/// Worktree-aware session repository
/// Handles saving/loading session metadata in worktree directories
pub struct WorktreeSessionRepository {
    /// Base repository for non-worktree operations
    inner: Box<dyn SessionRepository>,
}

impl WorktreeSessionRepository {
    pub fn new(inner: Box<dyn SessionRepository>) -> Self {
        Self { inner }
    }

    /// Save session metadata in a worktree directory
    pub async fn save_in_worktree(&self, metadata: &SessionMetadata, worktree_path: &Path) -> Result<(), SessionError> {
        let session_file = worktree_path.join(".amazonq").join("session.json");

        // Ensure directory exists
        if let Some(parent) = session_file.parent() {
            std::fs::create_dir_all(parent)?;
        }

        save_metadata(metadata, &session_file)
            .await
            ?;

        // Also save to main repository
        self.inner.save(metadata).await
    }

    /// Load session metadata from a worktree directory
    pub async fn load_from_worktree(&self, worktree_path: &Path) -> Result<SessionMetadata, SessionError> {
        let session_file = worktree_path.join(".amazonq").join("session.json");

        if !session_file.exists() {
            return Err(SessionError::NotFound);
        }

        load_metadata(&session_file)
            .await
            
    }

    /// Detect if current directory is in a worktree and load session
    pub async fn load_current_worktree(&self) -> Result<Option<SessionMetadata>, SessionError> {
        let current_dir = std::env::current_dir()?;

        // Check if we're in a worktree by detecting git context
        match detect_git_context(&current_dir) {
            Ok(Some(_)) => match self.load_from_worktree(&current_dir).await {
                Ok(metadata) => Ok(Some(metadata)),
                Err(SessionError::NotFound(_)) => Ok(None),
                Err(e) => Err(e),
            },
            Ok(None) => Ok(None),
            Err(_) => Ok(None), // Not a git repo, that's fine
        }
    }
}

#[async_trait]
impl SessionRepository for WorktreeSessionRepository {
    async fn get(&self, id: &str) -> Result<SessionMetadata, SessionError> {
        self.inner.get(id).await
    }

    async fn save(&self, metadata: &SessionMetadata) -> Result<(), SessionError> {
        // If this is a worktree session, save to worktree too
        if let Some(worktree_path) = metadata.worktree_path() {
            self.save_in_worktree(metadata, worktree_path).await
        } else {
            self.inner.save(metadata).await
        }
    }

    async fn delete(&self, id: &str) -> Result<(), SessionError> {
        self.inner.delete(id).await
    }

    async fn list(&self, filter: SessionFilter) -> Result<Vec<SessionMetadata>, SessionError> {
        self.inner.list(filter).await
    }

    async fn exists(&self, id: &str) -> Result<bool, SessionError> {
        self.inner.exists(id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::{
        InMemoryRepository,
        WorktreeInfo,
    };

    #[tokio::test]
    async fn test_worktree_repository_creation() {
        let inner = Box::new(InMemoryRepository::new());
        let repo = WorktreeSessionRepository::new(inner);

        // Should be able to list (empty)
        let sessions = repo.list(SessionFilter::default()).await.unwrap();
        assert_eq!(sessions.len(), 0);
    }

    #[tokio::test]
    async fn test_non_worktree_session_passthrough() {
        let inner = Box::new(InMemoryRepository::new());
        let repo = WorktreeSessionRepository::new(inner);

        let metadata = SessionMetadata::new("test-id", "First message");
        repo.save(&metadata).await.unwrap();

        let loaded = repo.get("test-id").await.unwrap();
        assert_eq!(loaded.id, "test-id");
        assert!(!loaded.is_worktree_session());
    }
}
