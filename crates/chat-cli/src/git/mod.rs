pub mod context;
pub mod error;
pub mod worktree;

pub use context::{
    GitContext,
    detect_git_context,
    is_git_installed,
};
pub use error::GitError;
pub use worktree::{
    GitWorktreeInfo,
    create_worktree,
    list_worktrees,
    remove_worktree,
};
