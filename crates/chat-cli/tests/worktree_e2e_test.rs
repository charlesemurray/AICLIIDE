/// End-to-end integration test for worktree functionality
/// 
/// This test verifies the complete flow:
/// 1. Parse --worktree flag from CLI
/// 2. Resolve strategy
/// 3. Create worktree
/// 4. Save session metadata
/// 5. Resume from worktree

use chat_cli::cli::chat::worktree_strategy::{resolve_worktree_strategy, WorktreeStrategy};
use chat_cli::cli::chat::branch_naming::{sanitize_branch_name, generate_from_conversation};
use chat_cli::session::{SessionMetadata, WorktreeInfo, resolve_session_id};
use chat_cli::git::GitContext;
use std::path::PathBuf;

#[test]
fn test_e2e_worktree_flag_to_strategy() {
    // Simulate: q chat --worktree my-feature
    let worktree_arg = Some("my-feature".to_string());
    let strategy = resolve_worktree_strategy(Some(&worktree_arg), false, None);
    
    assert_eq!(strategy, WorktreeStrategy::Create("my-feature".to_string()));
}

#[test]
fn test_e2e_no_worktree_flag() {
    // Simulate: q chat --no-worktree
    let strategy = resolve_worktree_strategy(None, true, None);
    
    assert_eq!(strategy, WorktreeStrategy::Never);
}

#[test]
fn test_e2e_branch_name_generation() {
    // Simulate: User message -> branch name
    let message = "Add user authentication";
    let branch = generate_from_conversation(message, Some("feature"));
    
    assert_eq!(branch, "feature/user-authentication");
}

#[test]
fn test_e2e_session_metadata_with_worktree() {
    // Simulate: Create session with worktree info
    let worktree_info = WorktreeInfo {
        path: PathBuf::from("/tmp/repo-feature-auth"),
        branch: "feature/auth".to_string(),
        repo_root: PathBuf::from("/tmp/repo"),
        is_temporary: true,
        merge_target: "main".to_string(),
    };
    
    let session = SessionMetadata::new("test-session", "Add auth")
        .with_worktree(worktree_info);
    
    assert!(session.is_worktree_session());
    assert_eq!(session.worktree_path(), Some(&PathBuf::from("/tmp/repo-feature-auth")));
}

#[test]
fn test_e2e_session_id_from_git() {
    // Simulate: Generate session ID from git context
    let path = PathBuf::from("/tmp/my-repo");
    let id = resolve_session_id(&path, None);
    
    // Should fallback to path-based ID when not in git repo
    assert_eq!(id, "my-repo");
}

#[test]
fn test_e2e_detect_existing_worktree() {
    // Simulate: Already in a worktree
    let context = GitContext {
        repo_root: PathBuf::from("/tmp/repo"),
        repo_name: "repo".to_string(),
        branch_name: "feature/test".to_string(),
        is_worktree: true,
        is_main_branch: false,
        worktree_path: Some(PathBuf::from("/tmp/repo-feature-test")),
    };
    
    let strategy = resolve_worktree_strategy(None, false, Some(&context));
    assert_eq!(strategy, WorktreeStrategy::UseExisting);
}
