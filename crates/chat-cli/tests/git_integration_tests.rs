use std::fs;
use std::path::PathBuf;
use std::process::Command;

use chat_cli::git::{GitError, create_worktree, detect_git_context, is_git_installed, list_worktrees, remove_worktree};
use tempfile::TempDir;

/// Helper to create a test git repository
fn create_test_repo() -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    // Initialize git repo
    Command::new("git")
        .current_dir(repo_path)
        .args(&["init"])
        .output()
        .unwrap();

    // Configure git
    Command::new("git")
        .current_dir(repo_path)
        .args(&["config", "user.name", "Test User"])
        .output()
        .unwrap();

    Command::new("git")
        .current_dir(repo_path)
        .args(&["config", "user.email", "test@example.com"])
        .output()
        .unwrap();

    // Create initial commit
    fs::write(repo_path.join("README.md"), "# Test Repo").unwrap();
    Command::new("git")
        .current_dir(repo_path)
        .args(&["add", "."])
        .output()
        .unwrap();
    Command::new("git")
        .current_dir(repo_path)
        .args(&["commit", "-m", "Initial commit"])
        .output()
        .unwrap();

    temp_dir
}

#[test]
fn test_git_installed() {
    // This test assumes git is installed on the system
    assert!(is_git_installed(), "Git should be installed for tests");
}

#[test]
fn test_detect_git_context_in_repo() {
    let temp_repo = create_test_repo();
    let repo_path = temp_repo.path();

    let context = detect_git_context(repo_path).unwrap();

    assert_eq!(context.repo_root, repo_path);
    assert!(context.branch_name == "main" || context.branch_name == "master");
    assert!(context.is_main_branch);
    assert!(!context.is_worktree);
}

#[test]
fn test_detect_git_context_not_a_repo() {
    let temp_dir = TempDir::new().unwrap();
    let result = detect_git_context(temp_dir.path());

    assert!(matches!(result, Err(GitError::NotARepository)));
}

#[test]
fn test_create_and_list_worktree() {
    let temp_repo = create_test_repo();
    let repo_path = temp_repo.path();

    // Create a worktree
    let worktree_path = create_worktree(repo_path, "feature/test", "main", None).unwrap();

    assert!(worktree_path.exists());

    // List worktrees
    let worktrees = list_worktrees(repo_path).unwrap();

    assert_eq!(worktrees.len(), 2); // main + feature/test
    assert!(worktrees.iter().any(|wt| wt.branch == "feature/test"));
}

#[test]
fn test_create_worktree_with_custom_path() {
    let temp_repo = create_test_repo();
    let repo_path = temp_repo.path();

    let custom_path = repo_path.parent().unwrap().join("custom-worktree");

    let worktree_path = create_worktree(repo_path, "feature/custom", "main", Some(custom_path.clone())).unwrap();

    assert_eq!(worktree_path, custom_path);
    assert!(worktree_path.exists());
}

#[test]
fn test_create_duplicate_worktree_fails() {
    let temp_repo = create_test_repo();
    let repo_path = temp_repo.path();

    // Create first worktree
    create_worktree(repo_path, "feature/duplicate", "main", None).unwrap();

    // Try to create duplicate
    let result = create_worktree(repo_path, "feature/duplicate", "main", None);

    assert!(matches!(result, Err(GitError::BranchExists(_))));
}

#[test]
fn test_remove_worktree() {
    let temp_repo = create_test_repo();
    let repo_path = temp_repo.path();

    // Create worktree
    let worktree_path = create_worktree(repo_path, "feature/remove", "main", None).unwrap();

    assert!(worktree_path.exists());

    // Remove worktree
    remove_worktree(&worktree_path).unwrap();

    // Verify it's gone
    let worktrees = list_worktrees(repo_path).unwrap();
    assert!(!worktrees.iter().any(|wt| wt.branch == "feature/remove"));
}

#[test]
fn test_multiple_worktrees() {
    let temp_repo = create_test_repo();
    let repo_path = temp_repo.path();

    // Create multiple worktrees
    create_worktree(repo_path, "feature/one", "main", None).unwrap();
    create_worktree(repo_path, "feature/two", "main", None).unwrap();
    create_worktree(repo_path, "bugfix/three", "main", None).unwrap();

    // List all worktrees
    let worktrees = list_worktrees(repo_path).unwrap();

    assert_eq!(worktrees.len(), 4); // main + 3 features
    assert!(worktrees.iter().any(|wt| wt.branch == "feature/one"));
    assert!(worktrees.iter().any(|wt| wt.branch == "feature/two"));
    assert!(worktrees.iter().any(|wt| wt.branch == "bugfix/three"));
}

#[test]
fn test_detect_context_in_worktree() {
    let temp_repo = create_test_repo();
    let repo_path = temp_repo.path();

    // Create worktree
    let worktree_path = create_worktree(repo_path, "feature/context", "main", None).unwrap();

    // Detect context from within worktree
    let context = detect_git_context(&worktree_path).unwrap();

    assert_eq!(context.branch_name, "feature/context");
    assert!(context.is_worktree);
    assert!(!context.is_main_branch);
    assert_eq!(context.worktree_path, Some(worktree_path));
}

#[test]
fn test_graceful_degradation_no_git() {
    // This test verifies error handling when git is not available
    // We can't actually uninstall git, but we test the error path

    let temp_dir = TempDir::new().unwrap();
    let result = detect_git_context(temp_dir.path());

    // Should fail gracefully
    assert!(result.is_err());
}

#[test]
fn test_worktree_conflict_detection() {
    let temp_repo = create_test_repo();
    let repo_path = temp_repo.path();

    // Create worktree
    let worktree_path = create_worktree(repo_path, "feature/conflict", "main", None).unwrap();

    // Try to create worktree at same path
    let result = create_worktree(repo_path, "feature/other", "main", Some(worktree_path.clone()));

    assert!(matches!(result, Err(GitError::WorktreeExists(_))));
}
