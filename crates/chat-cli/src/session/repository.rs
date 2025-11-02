use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use time::OffsetDateTime;

// Forward declarations for types we'll define in other modules
use super::error::SessionError;
use super::metadata::{SessionMetadata, SessionStatus};

/// Filter criteria for listing sessions
#[derive(Default, Clone)]
pub struct SessionFilter {
    pub status: Option<SessionStatus>,
    pub limit: Option<usize>,
    pub search: Option<String>,
}

/// Repository trait for session storage operations
#[async_trait]
pub trait SessionRepository: Send + Sync {
    /// Get a session by ID
    async fn get(&self, id: &str) -> Result<SessionMetadata, SessionError>;
    
    /// Save or update a session
    async fn save(&self, metadata: &SessionMetadata) -> Result<(), SessionError>;
    
    /// Delete a session
    async fn delete(&self, id: &str) -> Result<(), SessionError>;
    
    /// List sessions with optional filtering
    async fn list(&self, filter: SessionFilter) -> Result<Vec<SessionMetadata>, SessionError>;
    
    /// Check if a session exists
    async fn exists(&self, id: &str) -> Result<bool, SessionError>;
}

/// In-memory implementation for testing
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
        sessions
            .get(id)
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
        sessions
            .remove(id)
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
                s.first_message.to_lowercase().contains(&search_lower)
                    || s.name
                        .as_ref()
                        .map_or(false, |n| n.to_lowercase().contains(&search_lower))
            });
        }

        // Sort by last_active descending (most recent first)
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

        let active = SessionMetadata::new("active-1", "Active");
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
                let metadata = SessionMetadata::new(format!("session-{}", i), format!("Message {}", i));
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

    #[tokio::test]
    async fn test_exists() {
        let repo = InMemoryRepository::new();
        let metadata = SessionMetadata::new("test-1", "First");

        assert!(!repo.exists("test-1").await.unwrap());

        repo.save(&metadata).await.unwrap();
        assert!(repo.exists("test-1").await.unwrap());
    }

    #[tokio::test]
    async fn test_list_with_search() {
        let repo = InMemoryRepository::new();

        let meta1 = SessionMetadata::new("id-1", "Implement authentication");
        repo.save(&meta1).await.unwrap();

        let meta2 = SessionMetadata::new("id-2", "Fix login bug");
        repo.save(&meta2).await.unwrap();

        let filter = SessionFilter {
            search: Some("auth".to_string()),
            ..Default::default()
        };
        let results = repo.list(filter).await.unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "id-1");
    }

    #[tokio::test]
    async fn test_list_with_limit() {
        let repo = InMemoryRepository::new();

        for i in 0..5 {
            let metadata = SessionMetadata::new(format!("id-{}", i), format!("Message {}", i));
            repo.save(&metadata).await.unwrap();
        }

        let filter = SessionFilter {
            limit: Some(3),
            ..Default::default()
        };
        let results = repo.list(filter).await.unwrap();

        assert_eq!(results.len(), 3);
    }

    #[tokio::test]
    async fn test_list_sorted_by_last_active() {
        let repo = InMemoryRepository::new();

        // Create sessions with different timestamps
        let mut old = SessionMetadata::new("old", "Old session");
        old.last_active = OffsetDateTime::now_utc() - time::Duration::hours(2);
        repo.save(&old).await.unwrap();

        let new = SessionMetadata::new("new", "New session");
        repo.save(&new).await.unwrap();

        let results = repo.list(SessionFilter::default()).await.unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, "new"); // Most recent first
        assert_eq!(results[1].id, "old");
    }
}
