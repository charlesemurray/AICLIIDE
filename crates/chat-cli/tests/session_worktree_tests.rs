use std::path::PathBuf;

use chat_cli::session::{
    SessionMetadata,
    WorktreeInfo,
};

#[test]
fn test_worktree_info_creation() {
    let worktree_info = WorktreeInfo {
        path: PathBuf::from("/tmp/worktree"),
        branch: "feature/test".to_string(),
        repo_root: PathBuf::from("/tmp/repo"),
        is_temporary: true,
        merge_target: "main".to_string(),
    };

    let metadata = SessionMetadata::new("test-session", "First message").with_worktree(worktree_info.clone());

    assert!(metadata.is_worktree_session());
    assert_eq!(metadata.worktree_path(), Some(&PathBuf::from("/tmp/worktree")));
    assert_eq!(metadata.worktree_info.as_ref().unwrap().branch, "feature/test");
    assert_eq!(metadata.worktree_info.as_ref().unwrap().merge_target, "main");
    assert!(metadata.worktree_info.as_ref().unwrap().is_temporary);
}

#[test]
fn test_non_worktree_session() {
    let metadata = SessionMetadata::new("test-session", "First message");

    assert!(!metadata.is_worktree_session());
    assert_eq!(metadata.worktree_path(), None);
    assert!(metadata.worktree_info.is_none());
}

#[test]
fn test_worktree_serialization() {
    let worktree_info = WorktreeInfo {
        path: PathBuf::from("/tmp/worktree"),
        branch: "feature/parallel".to_string(),
        repo_root: PathBuf::from("/tmp/repo"),
        is_temporary: false,
        merge_target: "main".to_string(),
    };

    let metadata = SessionMetadata::new("test-session", "First message").with_worktree(worktree_info);

    // Serialize to JSON
    let json = serde_json::to_string(&metadata).unwrap();
    assert!(json.contains("worktree_info"));
    assert!(json.contains("feature/parallel"));

    // Deserialize back
    let deserialized: SessionMetadata = serde_json::from_str(&json).unwrap();
    assert!(deserialized.is_worktree_session());
    assert_eq!(deserialized.worktree_info.as_ref().unwrap().branch, "feature/parallel");
}

#[test]
fn test_non_worktree_serialization_omits_field() {
    let metadata = SessionMetadata::new("test-session", "First message");

    let json = serde_json::to_string(&metadata).unwrap();
    // worktree_info should be omitted when None
    assert!(!json.contains("worktree_info"));
}

#[test]
fn test_persist_session_to_worktree() {
    use chat_cli::cli::chat::worktree_session::persist_to_worktree;
    use tempfile::TempDir;
    
    let temp_dir = TempDir::new().unwrap();
    let worktree_path = temp_dir.path();
    
    let worktree_info = WorktreeInfo {
        path: worktree_path.to_path_buf(),
        branch: "feature/test".to_string(),
        repo_root: PathBuf::from("/repo"),
        is_temporary: false,
        merge_target: "main".to_string(),
    };
    
    let result = persist_to_worktree(worktree_path, "test-session-123", &worktree_info);
    assert!(result.is_ok(), "Failed to persist session: {:?}", result.err());
    
    // Verify file was created
    let session_file = worktree_path.join(".amazonq/session.json");
    assert!(session_file.exists(), "Session file was not created");
    
    // Verify content
    let content = std::fs::read_to_string(&session_file).unwrap();
    assert!(content.contains("test-session-123"));
    assert!(content.contains("feature/test"));
    assert!(content.contains("worktree_info"));
}

#[test]
fn test_persist_creates_directory() {
    use chat_cli::cli::chat::worktree_session::persist_to_worktree;
    use tempfile::TempDir;
    
    let temp_dir = TempDir::new().unwrap();
    let worktree_path = temp_dir.path();
    
    let worktree_info = WorktreeInfo {
        path: worktree_path.to_path_buf(),
        branch: "feature/new".to_string(),
        repo_root: PathBuf::from("/repo"),
        is_temporary: true,
        merge_target: "develop".to_string(),
    };
    
    persist_to_worktree(worktree_path, "session-456", &worktree_info).unwrap();
    
    let amazonq_dir = worktree_path.join(".amazonq");
    assert!(amazonq_dir.exists());
    assert!(amazonq_dir.is_dir());
}
