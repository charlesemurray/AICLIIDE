pub mod error;
pub mod io;
pub mod manager;
pub mod metadata;
pub mod repository;
pub mod worktree_repo;

#[cfg(test)]
mod integration_tests;

pub use error::SessionError;
pub use io::{
    load_metadata,
    save_metadata,
};
pub use manager::SessionManager;
pub use metadata::{
    METADATA_VERSION,
    SessionMetadata,
    SessionStatus,
    WorktreeInfo,
    validate_session_name,
};
pub use repository::{
    InMemoryRepository,
    SessionFilter,
    SessionRepository,
};
