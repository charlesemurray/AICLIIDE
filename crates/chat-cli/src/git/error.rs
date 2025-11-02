use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum GitError {
    #[error("Git is not installed or not in PATH")]
    NotInstalled,

    #[error("Not a git repository")]
    NotARepository,

    #[error("Worktree already exists: {0}")]
    WorktreeExists(String),

    #[error("Branch already exists: {0}")]
    BranchExists(String),

    #[error("Git command failed: {0}")]
    CommandFailed(String),

    #[error("Failed to parse git output: {0}")]
    ParseError(String),

    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
}

pub type Result<T> = std::result::Result<T, GitError>;
