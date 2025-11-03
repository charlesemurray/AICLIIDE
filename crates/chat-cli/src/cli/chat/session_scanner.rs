use std::path::Path;

use eyre::Result;

use crate::cli::chat::worktree_session::load_from_worktree;
use crate::git::{
    detect_git_context,
    list_worktrees,
};
use crate::session::metadata::SessionMetadata;

/// Scan for worktree-based sessions
pub fn scan_worktree_sessions(repo_root: &Path) -> Result<(Vec<SessionMetadata>, Vec<String>)> {
    let worktrees = list_worktrees(repo_root)?;
    let mut sessions = Vec::new();
    let mut errors = Vec::new();

    for wt in worktrees {
        match load_from_worktree(&wt.path) {
            Ok(metadata) => sessions.push(metadata),
            Err(e) => errors.push(format!("{}: {}", wt.path.display(), e)),
        }
    }

    Ok((sessions, errors))
}

/// Get all worktree sessions in current repository
pub fn get_current_repo_sessions() -> Result<(Vec<SessionMetadata>, Vec<String>)> {
    let current_dir = std::env::current_dir()?;
    let git_ctx = detect_git_context(&current_dir)?;
    scan_worktree_sessions(&git_ctx.repo_root)
}
