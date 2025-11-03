pub mod error;
pub mod io;
pub mod lock;
pub mod manager;
pub mod metadata;
pub mod metrics;
pub mod repository;
pub mod session_id;
pub mod worktree_repo;

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
