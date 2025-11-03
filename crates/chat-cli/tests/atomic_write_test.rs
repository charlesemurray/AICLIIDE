use chat_cli::cli::chat::worktree_session::{persist_to_worktree, load_from_worktree};
use chat_cli::session::metadata::{SessionMetadata, WorktreeInfo};
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_persist_creates_temp_file_first() {
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
    
    // Persist
    persist_to_worktree(&wt_path, &metadata).unwrap();
    
    // Final file should exist
    assert!(wt_path.join(".amazonq/session.json").exists());
    
    // Temp file should NOT exist (was renamed)
    assert!(!wt_path.join(".amazonq/.session.json.tmp").exists());
    assert!(!wt_path.join(".amazonq/session.json.tmp").exists());
}

#[test]
fn test_persist_is_atomic() {
    let temp_dir = TempDir::new().unwrap();
    let wt_path = temp_dir.path();
    
    let wt_info = WorktreeInfo {
        path: wt_path.to_path_buf(),
        branch: "test".to_string(),
        repo_root: PathBuf::from("/tmp/repo"),
        is_temporary: false,
        merge_target: "main".to_string(),
    };
    
    // Create initial session
    let metadata1 = SessionMetadata::new("test-id-1", "First message")
        .with_worktree(wt_info.clone());
    persist_to_worktree(&wt_path, &metadata1).unwrap();
    
    // Overwrite with new session
    let metadata2 = SessionMetadata::new("test-id-2", "Second message")
        .with_worktree(wt_info);
    persist_to_worktree(&wt_path, &metadata2).unwrap();
    
    // Should have the second session
    let loaded = load_from_worktree(&wt_path).unwrap();
    assert_eq!(loaded.id, "test-id-2");
    assert_eq!(loaded.first_message, "Second message");
}

#[test]
fn test_persist_doesnt_corrupt_on_multiple_writes() {
    let temp_dir = TempDir::new().unwrap();
    let wt_path = temp_dir.path();
    
    let wt_info = WorktreeInfo {
        path: wt_path.to_path_buf(),
        branch: "test".to_string(),
        repo_root: PathBuf::from("/tmp/repo"),
        is_temporary: false,
        merge_target: "main".to_string(),
    };
    
    // Write multiple times
    for i in 0..5 {
        let metadata = SessionMetadata::new(&format!("test-id-{}", i), &format!("Message {}", i))
            .with_worktree(wt_info.clone());
        persist_to_worktree(&wt_path, &metadata).unwrap();
        
        // Should always be loadable
        let loaded = load_from_worktree(&wt_path).unwrap();
        assert_eq!(loaded.id, format!("test-id-{}", i));
    }
}
