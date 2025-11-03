use std::path::PathBuf;

use chat_cli::cli::chat::worktree_session::{
    load_from_worktree,
    persist_to_worktree,
};
use chat_cli::git::{
    create_worktree,
    init_git_repo,
};
use chat_cli::session::metadata::WorktreeInfo;
use tempfile::TempDir;

#[test]
fn test_worktree_session_lifecycle() {
    let temp_dir = TempDir::new().unwrap();
    let repo_root = temp_dir.path();

    // Initialize git repo
    init_git_repo(repo_root).unwrap();

    // Create worktree
    let worktree_path = create_worktree(repo_root, "feature/test", "main", None).unwrap();

    // Persist session
    let worktree_info = WorktreeInfo {
        path: worktree_path.clone(),
        branch: "feature/test".to_string(),
        repo_root: repo_root.to_path_buf(),
        is_temporary: false,
        merge_target: "main".to_string(),
    };

    persist_to_worktree(&worktree_path, "test-conv-123", &worktree_info).unwrap();

    // Verify session file exists
    let session_file = worktree_path.join(".amazonq/session.json");
    assert!(session_file.exists());

    // Load session back
    let loaded = load_from_worktree(&worktree_path).unwrap();
    assert_eq!(loaded.id, "test-conv-123");
    assert!(loaded.is_worktree_session());
    assert_eq!(loaded.worktree_info.as_ref().unwrap().branch, "feature/test");
}
