use std::path::PathBuf;
use tempfile::TempDir;

use chat_cli::session::{
    InMemoryRepository, SessionMetadata, WorktreeInfo, WorktreeSessionRepository,
};

#[tokio::test]
async fn test_worktree_repository_creation() {
    let inner = Box::new(InMemoryRepository::new());
    let repo = WorktreeSessionRepository::new(inner);

    let sessions = repo.list(Default::default()).await.unwrap();
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

#[tokio::test]
async fn test_worktree_session_with_temp_dir() {
    let inner = Box::new(InMemoryRepository::new());
    let repo = WorktreeSessionRepository::new(inner);

    let temp_dir = TempDir::new().unwrap();
    let worktree_path = temp_dir.path().to_path_buf();

    let worktree_info = WorktreeInfo {
        path: worktree_path.clone(),
        branch: "feature/test".to_string(),
        repo_root: PathBuf::from("/tmp/repo"),
        is_temporary: true,
        merge_target: "main".to_string(),
    };

    let metadata = SessionMetadata::new("worktree-session", "First message")
        .with_worktree(worktree_info);

    repo.save(&metadata).await.unwrap();

    let loaded = repo.load_from_worktree(&worktree_path).await.unwrap();
    assert_eq!(loaded.id, "worktree-session");
    assert!(loaded.is_worktree_session());
    assert_eq!(loaded.worktree_info.as_ref().unwrap().branch, "feature/test");
}

#[tokio::test]
async fn test_load_from_nonexistent_worktree() {
    let inner = Box::new(InMemoryRepository::new());
    let repo = WorktreeSessionRepository::new(inner);

    let result = repo
        .load_from_worktree(&PathBuf::from("/nonexistent/path"))
        .await;

    assert!(result.is_err());
}
