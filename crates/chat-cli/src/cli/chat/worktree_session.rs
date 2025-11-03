use crate::session::metadata::{SessionMetadata, WorktreeInfo};
use eyre::Result;
use std::path::Path;

/// Persist session metadata to worktree
pub fn persist_to_worktree(
    worktree_path: &Path,
    conversation_id: &str,
    worktree_info: &WorktreeInfo,
) -> Result<()> {
    let session_dir = worktree_path.join(".amazonq");
    std::fs::create_dir_all(&session_dir)?;
    let metadata = SessionMetadata::new(conversation_id, "").with_worktree(worktree_info.clone());
    let session_file = session_dir.join("session.json");
    let json = serde_json::to_string_pretty(&metadata)?;
    std::fs::write(session_file, json)?;
    Ok(())
}

/// Load session metadata from worktree
pub fn load_from_worktree(worktree_path: &Path) -> Result<SessionMetadata> {
    let session_file = worktree_path.join(".amazonq/session.json");
    let json = std::fs::read_to_string(session_file)?;
    let metadata: SessionMetadata = serde_json::from_str(&json)?;
    Ok(metadata)
}
