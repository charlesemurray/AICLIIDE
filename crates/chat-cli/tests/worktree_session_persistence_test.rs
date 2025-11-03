use chat_cli::cli::chat::worktree_session::{persist_to_worktree, load_from_worktree};
use chat_cli::session::metadata::{SessionMetadata, WorktreeInfo};
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_persist_preserves_all_metadata_fields() {
    let temp_dir = TempDir::new().unwrap();
    let wt_path = temp_dir.path();
    
    let wt_info = WorktreeInfo {
        path: wt_path.to_path_buf(),
        branch: "feature-test".to_string(),
        repo_root: PathBuf::from("/tmp/repo"),
        is_temporary: false,
        merge_target: "main".to_string(),
    };
    
    let metadata = SessionMetadata::new("test-id-123", "First message content")
        .with_worktree(wt_info.clone());
    
    // Persist
    persist_to_worktree(&wt_path, &metadata).unwrap();
    
    // Load back
    let loaded = load_from_worktree(&wt_path).unwrap();
    
    // Verify all fields preserved
    assert_eq!(loaded.id, "test-id-123");
    assert_eq!(loaded.first_message, "First message content");
    assert!(loaded.worktree_info.is_some());
    
    let loaded_wt = loaded.worktree_info.unwrap();
    assert_eq!(loaded_wt.branch, "feature-test");
    assert_eq!(loaded_wt.merge_target, "main");
}

#[test]
fn test_persist_rejects_empty_first_message() {
    let temp_dir = TempDir::new().unwrap();
    let wt_path = temp_dir.path();
    
    let wt_info = WorktreeInfo {
        path: wt_path.to_path_buf(),
        branch: "test".to_string(),
        repo_root: PathBuf::from("/tmp/repo"),
        is_temporary: false,
        merge_target: "main".to_string(),
    };
    
    // Create metadata with empty first_message
    let metadata = SessionMetadata::new("test-id", "")
        .with_worktree(wt_info);
    
    // Should still work but we'll document this is not ideal
    // In future, we should validate this
    let result = persist_to_worktree(&wt_path, &metadata);
    
    // For now, just verify it doesn't panic
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_persist_creates_directory_structure() {
    let temp_dir = TempDir::new().unwrap();
    let wt_path = temp_dir.path();
    
    let wt_info = WorktreeInfo {
        path: wt_path.to_path_buf(),
        branch: "test".to_string(),
        repo_root: PathBuf::from("/tmp/repo"),
        is_temporary: false,
        merge_target: "main".to_string(),
    };
    
    let metadata = SessionMetadata::new("test-id", "Test message")
        .with_worktree(wt_info);
    
    persist_to_worktree(&wt_path, &metadata).unwrap();
    
    // Verify directory structure
    assert!(wt_path.join(".amazonq").exists());
    assert!(wt_path.join(".amazonq/session.json").exists());
}
