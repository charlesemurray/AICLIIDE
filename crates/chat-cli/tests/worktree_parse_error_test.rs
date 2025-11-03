use chat_cli::git::list_worktrees;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn test_list_worktrees_handles_invalid_repo() {
    let temp_dir = TempDir::new().unwrap();
    let non_repo = temp_dir.path();
    
    // Should return error, not panic
    let result = list_worktrees(non_repo);
    
    // Either returns error or empty list, but shouldn't panic
    match result {
        Ok(worktrees) => {
            // Empty list is acceptable for non-repo
            assert!(worktrees.is_empty());
        }
        Err(_) => {
            // Error is also acceptable
        }
    }
}

#[test]
fn test_list_worktrees_doesnt_panic_on_nonexistent_path() {
    let non_existent = Path::new("/nonexistent/path/that/does/not/exist");
    
    // Should return error, not panic
    let result = list_worktrees(non_existent);
    
    // Should be an error
    assert!(result.is_err());
}
