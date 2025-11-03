use std::path::PathBuf;

use chat_cli::session::{
    InMemoryRepository,
    SessionMetadata,
    WorktreeInfo,
    WorktreeSessionRepository,
    resolve_session_id,
};
use tempfile::TempDir;

#[tokio::test]
async fn test_worktree_session_full_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let worktree_path = temp_dir.path().to_path_buf();

    let worktree_info = WorktreeInfo {
        path: worktree_path.clone(),
        branch: "feature/test".to_string(),
        repo_root: PathBuf::from("/tmp/repo"),
        is_temporary: true,
        merge_target: "main".to_string(),
    };

    let metadata = SessionMetadata::new("test-session", "First message").with_worktree(worktree_info);

    let inner = Box::new(InMemoryRepository::new());
    let repo = WorktreeSessionRepository::new(inner);

    // Save
    repo.save(&metadata).await.unwrap();

    // Load from worktree
    let loaded = repo.load_from_worktree(&worktree_path).await.unwrap();
    assert_eq!(loaded.id, "test-session");
    assert!(loaded.is_worktree_session());

    // Load from main repo
    let from_repo = repo.get("test-session").await.unwrap();
    assert_eq!(from_repo.id, "test-session");
}

#[test]
fn test_session_id_resolution() {
    let path = PathBuf::from("/tmp/my-project");

    // With override
    let id = resolve_session_id(&path, Some("custom"));
    assert_eq!(id, "custom");

    // Without override (fallback to path)
    let id = resolve_session_id(&path, None);
    assert_eq!(id, "my-project");
}

#[tokio::test]
async fn test_non_worktree_session_workflow() {
    let inner = Box::new(InMemoryRepository::new());
    let repo = WorktreeSessionRepository::new(inner);

    let metadata = SessionMetadata::new("regular-session", "Message");

    repo.save(&metadata).await.unwrap();

    let loaded = repo.get("regular-session").await.unwrap();
    assert_eq!(loaded.id, "regular-session");
    assert!(!loaded.is_worktree_session());
}
