pub mod error;
pub mod fs_repository;
pub mod io;
pub mod lock;
pub mod manager;
pub mod metadata;
pub mod metrics;
pub mod preview;
pub mod repository;
pub mod session_id;
pub mod worktree_repo;

pub use error::SessionError;
pub use fs_repository::FileSystemRepository;
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
pub use metrics::{
    MetricsSnapshot,
    SessionMetrics,
};
pub use repository::{
    InMemoryRepository,
    SessionFilter,
    SessionRepository,
};
pub use session_id::resolve_session_id;
pub use worktree_repo::WorktreeSessionRepository;
