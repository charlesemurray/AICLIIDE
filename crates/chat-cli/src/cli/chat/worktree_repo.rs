//! Worktree-aware session repository

use std::path::Path;
use eyre::Result;

use crate::session::metadata::SessionMetadata;
use crate::git::worktree::{list_worktrees, GitWorktreeInfo};

/// Discover sessions in git worktrees
pub fn discover_worktree_sessions(repo_root: &Path) -> Result<Vec<SessionMetadata>> {
    let worktrees = list_worktrees(repo_root)?;
    let mut sessions = Vec::new();
    
    for worktree in worktrees {
        // Try to load session from worktree
        if let Ok(metadata) = super::worktree_session::load_from_worktree(&worktree.path) {
            sessions.push(metadata);
        }
    }
    
    Ok(sessions)
}

/// Save session to worktree
pub fn save_to_worktree(worktree_path: &Path, metadata: &SessionMetadata) -> Result<()> {
    super::worktree_session::persist_to_worktree(worktree_path, metadata)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_discover_empty() {
        // Should handle non-existent path gracefully
        let result = discover_worktree_sessions(Path::new("/nonexistent"));
        assert!(result.is_err());
    }
}
