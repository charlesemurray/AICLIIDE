use chat_cli::git::worktree::GitWorktreeInfo;
use chat_cli::session::metadata::WorktreeInfo;
use std::path::PathBuf;

#[test]
fn test_git_worktree_to_session_worktree_conversion() {
    let git_wt = GitWorktreeInfo {
        path: PathBuf::from("/tmp/wt"),
        branch: "feature".into(),
        commit: "abc123".into(),
    };
    
    let session_wt = git_wt.to_session_info(
        PathBuf::from("/tmp/repo"),
        "main".into(),
    );
    
    assert_eq!(session_wt.path, git_wt.path);
    assert_eq!(session_wt.branch, git_wt.branch);
    assert_eq!(session_wt.repo_root, PathBuf::from("/tmp/repo"));
    assert_eq!(session_wt.merge_target, "main");
    assert_eq!(session_wt.is_temporary, false);
}

#[test]
fn test_conversion_preserves_path_and_branch() {
    let git_wt = GitWorktreeInfo {
        path: PathBuf::from("/custom/path"),
        branch: "hotfix/urgent".into(),
        commit: "def456".into(),
    };
    
    let session_wt = git_wt.to_session_info(
        PathBuf::from("/repo/root"),
        "develop".into(),
    );
    
    assert_eq!(session_wt.path, PathBuf::from("/custom/path"));
    assert_eq!(session_wt.branch, "hotfix/urgent");
}
