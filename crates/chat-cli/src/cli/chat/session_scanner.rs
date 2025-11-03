use crate::git::{list_worktrees, detect_git_context};
use crate::cli::chat::worktree_session::load_from_worktree;
use crate::session::metadata::SessionMetadata;
use eyre::Result;
use std::path::Path;

/// Scan for worktree-based sessions
pub fn scan_worktree_sessions(repo_root: &Path) -> Result<Vec<SessionMetadata>> {
    let worktrees = list_worktrees(repo_root)?;
    let mut sessions = Vec::new();
    
    for wt in worktrees {
        if let Ok(metadata) = load_from_worktree(&wt.path) {
            sessions.push(metadata);
        }
    }
    
    Ok(sessions)
}

/// Get all worktree sessions in current repository
pub fn get_current_repo_sessions() -> Result<Vec<SessionMetadata>> {
    let current_dir = std::env::current_dir()?;
    let git_ctx = detect_git_context(&current_dir)?;
    scan_worktree_sessions(&git_ctx.repo_root)
}
