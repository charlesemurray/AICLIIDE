use tracing::{debug, info, instrument, warn};

use super::error::SessionError;
use super::metadata::{SessionMetadata, SessionStatus};
use super::metrics::SessionMetrics;
use super::preview::SessionPreview;
use super::repository::{SessionFilter, SessionRepository};

/// Session manager for high-level session operations
pub struct SessionManager<R: SessionRepository> {
    repository: R,
    metrics: SessionMetrics,
}

impl<R: SessionRepository> SessionManager<R> {
    pub fn new(repository: R) -> Self {
        Self {
            repository,
            metrics: SessionMetrics::new(),
        }
    }

    pub fn metrics(&self) -> &SessionMetrics {
        &self.metrics
    }

    /// List sessions with smart precomputation
    #[instrument(skip(self), fields(session_count))]
    pub async fn list_sessions(&self) -> Result<Vec<SessionPreview>, SessionError> {
        let start = std::time::Instant::now();
        debug!("Listing sessions with smart precomputation");

        let filter = SessionFilter::default();
        let sessions = self.repository.list(filter).await?;

        // Convert to smart previews - O(1) per session but with approximations
        let mut previews = Vec::new();
        for metadata in sessions {
            let session_path = std::path::PathBuf::from(".amazonq/sessions").join(&metadata.id);
            match SessionPreview::new(metadata, session_path) {
                Ok(preview) => previews.push(preview),
                Err(e) => {
                    warn!("Failed to create preview: {}", e);
                    // Continue with other sessions
                }
            }
        }

        let duration_ms = start.elapsed().as_millis() as u64;
        self.metrics.record_list(duration_ms, previews.len());
        info!(count = previews.len(), duration_ms, "Listed sessions with approximations");
        tracing::Span::current().record("session_count", previews.len());

        Ok(previews)
    }

    /// Fast sort by approximate size - O(n log n) but with fast comparisons
    pub async fn list_sessions_by_size(&self) -> Result<Vec<SessionPreview>, SessionError> {
        let mut previews = self.list_sessions().await?;
        
        // Sort by approximation first (fast)
        previews.sort_by_key(|p| p.approximation.estimated_size);
        
        Ok(previews)
    }

    /// Fast search in conversation previews
    pub async fn search_sessions(&self, query: &str) -> Result<Vec<SessionPreview>, SessionError> {
        let previews = self.list_sessions().await?;
        let mut results = Vec::new();
        
        for preview in previews {
            if preview.approximation.has_conversation {
                let content = preview.searchable_preview().await?;
                if content.to_lowercase().contains(&query.to_lowercase()) {
                    results.push(preview);
                }
            }
        }
        
        Ok(results)
    }

    /// List sessions filtered by status
    #[instrument(skip(self))]
    pub async fn list_by_status(&self, status: SessionStatus) -> Result<Vec<SessionMetadata>, SessionError> {
        debug!(?status, "Listing sessions by status");
        let filter = SessionFilter {
            status: Some(status.clone()),
            ..Default::default()
        };
        let filtered = self.repository.list(filter).await?;
        info!(status = ?status, count = filtered.len(), "Filtered sessions by status");
        Ok(filtered)
    }

    /// Get a specific session by ID
    #[instrument(skip(self))]
    pub async fn get_session(&self, session_id: &str) -> Result<SessionMetadata, SessionError> {
        debug!(session_id, "Getting session");
        let metadata = self.repository.get(session_id).await?;
        info!(session_id, "Retrieved session successfully");
        Ok(metadata)
    }

    /// Archive an active session
    #[instrument(skip(self))]
    pub async fn archive_active_session(&self, session_id: &str) -> Result<(), SessionError> {
        info!(session_id, "Archiving active session");
        
        let mut metadata = SessionMetadata::new(session_id, "");
        metadata.archive();
        
        self.repository.save(&metadata).await?;
        self.metrics.record_archive();
        info!(session_id, "Active session archived successfully");
        Ok(())
    }

    /// Optimize frequently accessed sessions in background (optional)
    pub async fn optimize_hot_sessions(&self, session_ids: &[String]) -> Result<(), SessionError> {
        // Only optimize a small number of "hot" sessions to avoid resource usage
        let limit = session_ids.len().min(10);
        
        for session_id in &session_ids[..limit] {
            if let Ok(preview) = self.get_session_preview(session_id).await {
                // Pre-warm the cache for hot sessions
                let _ = preview.precise_workspace_size().await;
                debug!("Pre-warmed cache for hot session: {}", session_id);
            }
        }
        
        Ok(())
    }

    /// Get single session preview by ID
    async fn get_session_preview(&self, session_id: &str) -> Result<SessionPreview, SessionError> {
        let metadata = self.repository.get(session_id).await?;
        let session_path = std::path::PathBuf::from(".amazonq/sessions").join(session_id);
        SessionPreview::new(metadata, session_path)
    }
    }

    /// Archive a session
    #[instrument(skip(self))]
    pub async fn archive_session(&self, session_id: &str) -> Result<(), SessionError> {
        info!(session_id, "Archiving session");
        let mut metadata = self.repository.get(session_id).await?;
        metadata.archive();
        self.repository.save(&metadata).await?;
        self.metrics.record_archive();
        info!(session_id, "Session archived successfully");
        Ok(())
    }

    /// Name a session
    #[instrument(skip(self, name))]
    pub async fn name_session(&self, session_id: &str, name: impl Into<String>) -> Result<(), SessionError> {
        let name = name.into();
        info!(session_id, name = %name, "Naming session");
        let mut metadata = self.repository.get(session_id).await?;
        metadata.set_name(name)?;
        self.repository.save(&metadata).await?;
        self.metrics.record_name();
        info!(session_id, "Session named successfully");
        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::InMemoryRepository;

    #[tokio::test]
    async fn test_list_sessions_empty() {
        let repo = InMemoryRepository::new();
        let manager = SessionManager::new(repo);

        let previews = manager.list_sessions().await.unwrap();
        assert!(previews.is_empty());
    }

    #[tokio::test]
    async fn test_list_sessions_with_data() {
        let repo = InMemoryRepository::new();
        let metadata1 = SessionMetadata::new("session-1", "First session");
        let metadata2 = SessionMetadata::new("session-2", "Second session");
        repo.save(&metadata1).await.unwrap();
        repo.save(&metadata2).await.unwrap();

        let manager = SessionManager::new(repo);
        let previews = manager.list_sessions().await.unwrap();

        assert_eq!(previews.len(), 2);
        assert_eq!(previews[0].metadata.id, "session-1");
        assert_eq!(previews[1].metadata.id, "session-2");
    }

    #[tokio::test]
    async fn test_list_sessions_sorted_by_last_active() {
        let repo = InMemoryRepository::new();
        
        let mut old_meta = SessionMetadata::new("old", "Old session");
        old_meta.last_active = time::OffsetDateTime::now_utc() - time::Duration::hours(2);
        repo.save(&old_meta).await.unwrap();

        let new_meta = SessionMetadata::new("new", "New session");
        repo.save(&new_meta).await.unwrap();

        let manager = SessionManager::new(repo);
        let previews = manager.list_sessions().await.unwrap();

        assert_eq!(previews.len(), 2);
        assert_eq!(previews[0].metadata.id, "new");
        assert_eq!(previews[1].metadata.id, "old");
    }

    #[tokio::test]
    async fn test_list_by_status() {
        let repo = InMemoryRepository::new();
        
        let active_meta = SessionMetadata::new("active-1", "Active");
        repo.save(&active_meta).await.unwrap();

        let mut archived_meta = SessionMetadata::new("archived-1", "Archived");
        archived_meta.archive();
        repo.save(&archived_meta).await.unwrap();

        let manager = SessionManager::new(repo);
        let active_sessions = manager.list_by_status(SessionStatus::Active).await.unwrap();

        assert_eq!(active_sessions.len(), 1);
        assert_eq!(active_sessions[0].id, "active-1");
    }

    #[tokio::test]
    async fn test_get_session() {
        let repo = InMemoryRepository::new();
        let metadata = SessionMetadata::new("test-1", "Test session");
        repo.save(&metadata).await.unwrap();

        let manager = SessionManager::new(repo);
        let loaded = manager.get_session("test-1").await.unwrap();

        assert_eq!(loaded.id, "test-1");
        assert_eq!(loaded.first_message, "Test session");
    }

    #[tokio::test]
    async fn test_get_nonexistent_session() {
        let repo = InMemoryRepository::new();
        let manager = SessionManager::new(repo);

        let result = manager.get_session("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_archive_session() {
        let repo = InMemoryRepository::new();
        let metadata = SessionMetadata::new("test-1", "Test");
        repo.save(&metadata).await.unwrap();

        let manager = SessionManager::new(repo);
        manager.archive_session("test-1").await.unwrap();

        let updated = manager.get_session("test-1").await.unwrap();
        assert_eq!(updated.status, SessionStatus::Archived);
    }

    #[tokio::test]
    async fn test_archive_active_session() {
        let repo = InMemoryRepository::new();
        let manager = SessionManager::new(repo);
        
        manager.archive_active_session("test-1").await.unwrap();
        
        let metadata = manager.get_session("test-1").await.unwrap();
        assert_eq!(metadata.status, SessionStatus::Archived);
        assert_eq!(metadata.id, "test-1");
    }

    #[tokio::test]
    async fn test_name_session() {
        let repo = InMemoryRepository::new();
        let metadata = SessionMetadata::new("test-1", "Test");
        repo.save(&metadata).await.unwrap();

        let manager = SessionManager::new(repo);
        manager.name_session("test-1", "My Session").await.unwrap();

        let updated = manager.get_session("test-1").await.unwrap();
        assert_eq!(updated.name, Some("My Session".to_string()));
    }

    #[tokio::test]
    async fn test_name_session_validation() {
        let repo = InMemoryRepository::new();
        let metadata = SessionMetadata::new("test-1", "Test");
        repo.save(&metadata).await.unwrap();

        let manager = SessionManager::new(repo);
        let result = manager.name_session("test-1", "invalid name with spaces").await;
        assert!(result.is_err());

        let unchanged = manager.get_session("test-1").await.unwrap();
        assert_eq!(unchanged.name, None);
    }
}
