#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use tempfile::TempDir;
    use time::OffsetDateTime;

    use crate::os::Os;
    use crate::session::{
        SessionManager,
        SessionMetadata,
        SessionStatus,
    };

    fn create_test_os(temp_dir: &TempDir) -> Os {
        let mut os = Os::default();
        os.home_dir = Some(temp_dir.path().to_path_buf());
        os
    }

    #[tokio::test]
    async fn test_full_session_lifecycle() {
        let temp_dir = TempDir::new().unwrap();
        let os = create_test_os(&temp_dir);
        let manager = SessionManager::new(&os);

        // Create session metadata
        let session_id = "test-session-123";
        let metadata = SessionMetadata {
            version: 1,
            id: session_id.to_string(),
            status: SessionStatus::Active,
            created: OffsetDateTime::now_utc(),
            last_active: OffsetDateTime::now_utc(),
            first_message: "Test session".to_string(),
            name: None,
            file_count: 0,
            message_count: 1,
        };

        // Save metadata
        manager.repository.save(&metadata).await.unwrap();

        // List active sessions
        let active = manager.list_by_status(SessionStatus::Active).await.unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].id, session_id);

        // Name the session
        manager.name_session(session_id, "My Test Session").await.unwrap();
        let named = manager.get_session(session_id).await.unwrap();
        assert_eq!(named.name, Some("My Test Session".to_string()));

        // Archive the session
        manager.archive_session(session_id).await.unwrap();
        let archived = manager.get_session(session_id).await.unwrap();
        assert_eq!(archived.status, SessionStatus::Archived);

        // Verify it's in archived list
        let archived_list = manager.list_by_status(SessionStatus::Archived).await.unwrap();
        assert_eq!(archived_list.len(), 1);
        assert_eq!(archived_list[0].id, session_id);

        // Verify it's not in active list
        let active_list = manager.list_by_status(SessionStatus::Active).await.unwrap();
        assert_eq!(active_list.len(), 0);
    }

    #[tokio::test]
    async fn test_multiple_sessions_sorting() {
        let temp_dir = TempDir::new().unwrap();
        let os = create_test_os(&temp_dir);
        let manager = SessionManager::new(&os);

        let now = OffsetDateTime::now_utc();

        // Create three sessions with different timestamps
        for i in 0..3 {
            let metadata = SessionMetadata {
                version: 1,
                id: format!("session-{}", i),
                status: SessionStatus::Active,
                created: now - time::Duration::hours(i as i64),
                last_active: now - time::Duration::hours(i as i64),
                first_message: format!("Session {}", i),
                name: None,
                file_count: 0,
                message_count: 1,
            };
            manager.repository.save(&metadata).await.unwrap();
        }

        // List should be sorted by last_active (most recent first)
        let sessions = manager.list_sessions().await.unwrap();
        assert_eq!(sessions.len(), 3);
        assert_eq!(sessions[0].id, "session-0");
        assert_eq!(sessions[1].id, "session-1");
        assert_eq!(sessions[2].id, "session-2");
    }

    #[tokio::test]
    async fn test_session_status_filtering() {
        let temp_dir = TempDir::new().unwrap();
        let os = create_test_os(&temp_dir);
        let manager = SessionManager::new(&os);

        let now = OffsetDateTime::now_utc();

        // Create sessions with different statuses
        let statuses = vec![
            SessionStatus::Active,
            SessionStatus::Background,
            SessionStatus::Archived,
        ];

        for (i, status) in statuses.iter().enumerate() {
            let metadata = SessionMetadata {
                version: 1,
                id: format!("session-{}", i),
                status: status.clone(),
                created: now,
                last_active: now,
                first_message: format!("Session {}", i),
                name: None,
                file_count: 0,
                message_count: 1,
            };
            manager.repository.save(&metadata).await.unwrap();
        }

        // Test each status filter
        let active = manager.list_by_status(SessionStatus::Active).await.unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].status, SessionStatus::Active);

        let background = manager.list_by_status(SessionStatus::Background).await.unwrap();
        assert_eq!(background.len(), 1);
        assert_eq!(background[0].status, SessionStatus::Background);

        let archived = manager.list_by_status(SessionStatus::Archived).await.unwrap();
        assert_eq!(archived.len(), 1);
        assert_eq!(archived[0].status, SessionStatus::Archived);
    }

    #[tokio::test]
    async fn test_session_name_validation() {
        let temp_dir = TempDir::new().unwrap();
        let os = create_test_os(&temp_dir);
        let manager = SessionManager::new(&os);

        let metadata = SessionMetadata {
            version: 1,
            id: "test-session".to_string(),
            status: SessionStatus::Active,
            created: OffsetDateTime::now_utc(),
            last_active: OffsetDateTime::now_utc(),
            first_message: "Test".to_string(),
            name: None,
            file_count: 0,
            message_count: 1,
        };
        manager.repository.save(&metadata).await.unwrap();

        // Valid name
        assert!(manager.name_session("test-session", "Valid Name").await.is_ok());

        // Empty name should fail
        assert!(manager.name_session("test-session", "").await.is_err());

        // Too long name should fail
        let long_name = "a".repeat(300);
        assert!(manager.name_session("test-session", &long_name).await.is_err());
    }

    #[tokio::test]
    async fn test_archive_nonexistent_session() {
        let temp_dir = TempDir::new().unwrap();
        let os = create_test_os(&temp_dir);
        let manager = SessionManager::new(&os);

        let result = manager.archive_session("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_session_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let os = create_test_os(&temp_dir);

        let session_id = "persistent-session";

        // Create and save in first manager instance
        {
            let manager = SessionManager::new(&os);
            let metadata = SessionMetadata {
                version: 1,
                id: session_id.to_string(),
                status: SessionStatus::Active,
                created: OffsetDateTime::now_utc(),
                last_active: OffsetDateTime::now_utc(),
                first_message: "Persistent test".to_string(),
                name: Some("Test Name".to_string()),
                file_count: 5,
                message_count: 10,
            };
            manager.repository.save(&metadata).await.unwrap();
        }

        // Load in new manager instance
        {
            let manager = SessionManager::new(&os);
            let loaded = manager.get_session(session_id).await.unwrap();
            assert_eq!(loaded.id, session_id);
            assert_eq!(loaded.name, Some("Test Name".to_string()));
            assert_eq!(loaded.file_count, 5);
            assert_eq!(loaded.message_count, 10);
        }
    }
}
