//! Session persistence with error handling

use std::fs;
use std::path::{Path, PathBuf};

use eyre::{Context, Result, bail};
use serde::{Deserialize, Serialize};

use crate::theme::session::{SessionType, SessionStatus};

/// Persisted session data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedSession {
    pub conversation_id: String,
    pub name: String,
    pub session_type: SessionType,
    pub status: SessionStatus,
    pub created_at: u64,
    pub last_active: u64,
}

/// Session persistence manager
pub struct SessionPersistence {
    sessions_dir: PathBuf,
}

impl SessionPersistence {
    pub fn new(base_dir: &Path) -> Result<Self> {
        let sessions_dir = base_dir.join("sessions");
        fs::create_dir_all(&sessions_dir)
            .wrap_err_with(|| format!("Failed to create sessions directory: {}", sessions_dir.display()))?;
        
        Ok(Self { sessions_dir })
    }

    pub fn save_session(&self, session: &PersistedSession) -> Result<()> {
        let path = self.sessions_dir.join(format!("{}.json", session.conversation_id));
        let json = serde_json::to_string_pretty(session)?;
        let temp_path = path.with_extension("tmp");
        fs::write(&temp_path, json)?;
        fs::rename(&temp_path, &path)?;
        Ok(())
    }

    pub fn load_session(&self, conversation_id: &str) -> Result<PersistedSession> {
        let path = self.sessions_dir.join(format!("{}.json", conversation_id));
        if !path.exists() {
            bail!("Session file not found: {}", conversation_id);
        }
        let contents = fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&contents)?)
    }

    pub fn load_all_sessions(&self) -> Result<Vec<PersistedSession>> {
        let mut sessions = Vec::new();
        for entry in fs::read_dir(&self.sessions_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(id) = path.file_stem().and_then(|s| s.to_str()) {
                    if let Ok(session) = self.load_session(id) {
                        sessions.push(session);
                    }
                }
            }
        }
        Ok(sessions)
    }

    pub fn delete_session(&self, conversation_id: &str) -> Result<()> {
        let path = self.sessions_dir.join(format!("{}.json", conversation_id));
        if path.exists() {
            fs::remove_file(&path)?;
        }
        Ok(())
    }
}
