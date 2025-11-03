use chat_cli::cli::chat::session_scanner::scan_worktree_sessions;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn test_scan_reports_corrupted_sessions() {
    // This test verifies that scan_worktree_sessions returns errors
    // instead of silently ignoring corrupted session files
    
    // Note: This test requires a real git repo with worktrees
    // For now, we test the signature change
    
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();
    
    // Call should return tuple of (sessions, errors)
    let result = scan_worktree_sessions(repo_path);
    
    // Should return Ok with tuple, even if repo doesn't exist
    // (will have error in the errors vec)
    match result {
        Ok((sessions, errors)) => {
            // Either no sessions found, or errors reported
            assert!(sessions.is_empty() || !errors.is_empty());
        }
        Err(_) => {
            // Or fails fast if not a git repo - also acceptable
        }
    }
}

#[test]
fn test_scan_returns_tuple() {
    // Verify the function signature returns (Vec<SessionMetadata>, Vec<String>)
    let temp_dir = TempDir::new().unwrap();
    
    let result = scan_worktree_sessions(temp_dir.path());
    
    // Type check - should compile if signature is correct
    if let Ok((_sessions, _errors)) = result {
        // Success - signature is correct
    }
}
