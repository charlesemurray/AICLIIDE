//! Session registry for managing session storage and lifecycle

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use eyre::{Result, bail};
use tokio::sync::Mutex;

use crate::cli::chat::managed_session::ManagedSession;
use crate::cli::chat::session_lock::SessionLockManager;
use crate::cli::chat::session_persistence::SessionPersistence;

/// Combined session state
pub(crate) struct SessionState {
    pub(crate) sessions: HashMap<String, ManagedSession>,
    pub(crate) active_session_id: Option<String>,
}

/// Registry for session storage and lifecycle management
pub struct SessionRegistry {
    state: Arc<Mutex<SessionState>>,
    persistence: Option<SessionPersistence>,
    lock_manager: SessionLockManager,
}

impl SessionRegistry {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(SessionState {
                sessions: HashMap::new(),
                active_session_id: None,
            })),
            persistence: None,
            lock_manager: SessionLockManager::default(),
        }
    }

    pub fn state(&self) -> Arc<Mutex<SessionState>> {
        self.state.clone()
    }

    pub fn enable_persistence(&mut self, base_dir: std::path::PathBuf) -> Result<()> {
        self.persistence = Some(SessionPersistence::new(&base_dir)?);
        Ok(())
    }

    pub async fn cleanup_inactive(&mut self, max_age: Duration) -> Result<usize> {
        let mut state = self.state.lock().await;
        let now = std::time::Instant::now();
        
        let to_remove: Vec<_> = state.sessions
            .iter()
            .filter(|(_, session)| {
                now.duration_since(session.metadata.last_active) > max_age
            })
            .map(|(id, _)| id.clone())
            .collect();
        
        for id in &to_remove {
            state.sessions.remove(id);
            if let Some(p) = &self.persistence {
                let _ = p.delete_session(id);
            }
        }
        
        Ok(to_remove.len())
    }

    pub fn lock_manager(&self) -> &SessionLockManager {
        &self.lock_manager
    }
}

impl Default for SessionRegistry {
    fn default() -> Self {
        Self::new()
    }
}
