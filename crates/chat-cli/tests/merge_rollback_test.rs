use chat_cli::cli::chat::merge_workflow::merge_branch;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

fn setup_test_repo(temp_dir: &TempDir) -> std::path::PathBuf {
    let repo_path = temp_dir.path().join("test-repo");
    std::fs::create_dir(&repo_path).unwrap();
    
    // Initialize git repo
    Command::new("git")
        .arg("init")
        .current_dir(&repo_path)
        .output()
        .unwrap();
    
    // Configure git
    Command::new("git")
        .args(&["config", "user.email", "test@example.com"])
        .current_dir(&repo_path)
        .output()
        .unwrap();
    
    Command::new("git")
        .args(&["config", "user.name", "Test User"])
        .current_dir(&repo_path)
        .output()
        .unwrap();
    
    // Create initial commit on main
    std::fs::write(repo_path.join("file.txt"), "initial content").unwrap();
    Command::new("git")
        .args(&["add", "."])
        .current_dir(&repo_path)
        .output()
        .unwrap();
    
    Command::new("git")
        .args(&["commit", "-m", "Initial commit"])
        .current_dir(&repo_path)
        .output()
        .unwrap();
    
    repo_path
}

fn get_current_branch(repo_path: &Path) -> String {
    let output = Command::new("git")
        .args(&["branch", "--show-current"])
        .current_dir(repo_path)
        .output()
        .unwrap();
    
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

#[test]
fn test_merge_failure_returns_to_original_branch() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = setup_test_repo(&temp_dir);
    
    // Create a conflicting branch
    Command::new("git")
        .args(&["checkout", "-b", "feature"])
        .current_dir(&repo_path)
        .output()
        .unwrap();
    
    std::fs::write(repo_path.join("file.txt"), "feature content").unwrap();
    Command::new("git")
        .args(&["add", "."])
        .current_dir(&repo_path)
        .output()
        .unwrap();
    
    Command::new("git")
        .args(&["commit", "-m", "Feature change"])
        .current_dir(&repo_path)
        .output()
        .unwrap();
    
    // Go back to main and make conflicting change
    Command::new("git")
        .args(&["checkout", "main"])
        .current_dir(&repo_path)
        .output()
        .unwrap();
    
    std::fs::write(repo_path.join("file.txt"), "main content").unwrap();
    Command::new("git")
        .args(&["add", "."])
        .current_dir(&repo_path)
        .output()
        .unwrap();
    
    Command::new("git")
        .args(&["commit", "-m", "Main change"])
        .current_dir(&repo_path)
        .output()
        .unwrap();
    
    // Verify we're on main
    assert_eq!(get_current_branch(&repo_path), "main");
    
    // Attempt merge that will fail
    let result = merge_branch(&repo_path, "feature", "main");
    
    // Should fail
    assert!(result.is_err());
    
    // Should still be on main (rollback successful)
    let current = get_current_branch(&repo_path);
    assert_eq!(current, "main", "Should return to main branch after failed merge");
}

#[test]
fn test_successful_merge_stays_on_target() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = setup_test_repo(&temp_dir);
    
    // Create a non-conflicting branch
    Command::new("git")
        .args(&["checkout", "-b", "feature"])
        .current_dir(&repo_path)
        .output()
        .unwrap();
    
    std::fs::write(repo_path.join("new-file.txt"), "new content").unwrap();
    Command::new("git")
        .args(&["add", "."])
        .current_dir(&repo_path)
        .output()
        .unwrap();
    
    Command::new("git")
        .args(&["commit", "-m", "Add new file"])
        .current_dir(&repo_path)
        .output()
        .unwrap();
    
    Command::new("git")
        .args(&["checkout", "main"])
        .current_dir(&repo_path)
        .output()
        .unwrap();
    
    // Merge should succeed
    let result = merge_branch(&repo_path, "feature", "main");
    
    assert!(result.is_ok());
    
    // Should be on main
    assert_eq!(get_current_branch(&repo_path), "main");
}
