//! Multi-session coordinator for managing concurrent chat sessions

use std::collections::HashMap;
use std::sync::Arc;

use eyre::{Result, bail};
use tokio::sync::{Mutex, mpsc};

use crate::cli::chat::managed_session::{ManagedSession, OutputBuffer};
use crate::cli::chat::session_mode::SessionStateChange;
use crate::theme::session::{SessionDisplay, SessionStatus, SessionType};

/// Configuration for multi-session coordinator
#[derive(Debug, Clone)]
pub struct CoordinatorConfig {
    /// Maximum number of active sessions
    pub max_active_sessions: usize,
    /// Output buffer size per session in bytes
    pub buffer_size_bytes: usize,
    /// Maximum concurrent API calls
    pub max_concurrent_api_calls: usize,
}

impl Default for CoordinatorConfig {
    fn default() -> Self {
        Self {
            max_active_sessions: 10,
            buffer_size_bytes: 10 * 1024 * 1024, // 10 MB
            max_concurrent_api_calls: 5,
        }
    }
}

/// Coordinates multiple chat sessions
pub struct MultiSessionCoordinator {
    /// All managed sessions by conversation_id
    sessions: Arc<Mutex<HashMap<String, ManagedSession>>>,
    /// Currently active session ID
    active_session_id: Arc<Mutex<Option<String>>>,
    /// Configuration
    config: CoordinatorConfig,
    /// State change receiver
    state_rx: mpsc::UnboundedReceiver<SessionStateChange>,
    /// State change sender (cloned for each session)
    state_tx: mpsc::UnboundedSender<SessionStateChange>,
}

impl MultiSessionCoordinator {
    /// Create a new coordinator
    pub fn new(config: CoordinatorConfig) -> Self {
        let (state_tx, state_rx) = mpsc::unbounded_channel();

        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            active_session_id: Arc::new(Mutex::new(None)),
            config,
            state_rx,
            state_tx,
        }
    }

    /// Create a new session
    pub async fn create_session(
        &mut self,
        conversation_id: String,
        session_type: SessionType,
        name: Option<String>,
    ) -> Result<String> {
        let mut sessions = self.sessions.lock().await;

        // Check session limit
        if sessions.len() >= self.config.max_active_sessions {
            bail!(
                "Maximum active sessions ({}) reached",
                self.config.max_active_sessions
            );
        }

        // Generate name if not provided
        let session_name = name.unwrap_or_else(|| {
            format!("session-{}", sessions.len() + 1)
        });

        // Check for duplicate names
        if sessions.values().any(|s| s.display.name == session_name) {
            bail!("Session with name '{}' already exists", session_name);
        }

        // Create session display
        let display = SessionDisplay::new(session_type, session_name);

        // Create output buffer
        let buffer = Arc::new(Mutex::new(OutputBuffer::new(self.config.buffer_size_bytes)));

        // Create managed session (without ConversationState for now)
        // TODO: Integrate with actual ConversationState in next step
        let session = ManagedSession {
            display,
            conversation: unsafe { std::mem::zeroed() }, // Placeholder
            conversation_id: conversation_id.clone(),
            state: crate::cli::chat::managed_session::SessionState::Active,
            output_buffer: buffer,
            task_handle: None,
            last_error: None,
        };

        sessions.insert(conversation_id.clone(), session);

        // Set as active if first session
        let mut active_id = self.active_session_id.lock().await;
        if active_id.is_none() {
            *active_id = Some(conversation_id.clone());
        }

        Ok(conversation_id)
    }

    /// Switch to a different session
    pub async fn switch_session(&mut self, name: &str) -> Result<()> {
        let sessions = self.sessions.lock().await;

        // Find session by name
        let target_id = sessions
            .iter()
            .find(|(_, s)| s.display.name == name)
            .map(|(id, _)| id.clone())
            .ok_or_else(|| eyre::eyre!("Session '{}' not found", name))?;

        // Update active session
        let mut active_id = self.active_session_id.lock().await;
        *active_id = Some(target_id);

        Ok(())
    }

    /// Close a session
    pub async fn close_session(&mut self, name: &str) -> Result<()> {
        let mut sessions = self.sessions.lock().await;

        // Find session by name
        let target_id = sessions
            .iter()
            .find(|(_, s)| s.display.name == name)
            .map(|(id, _)| id.clone())
            .ok_or_else(|| eyre::eyre!("Session '{}' not found", name))?;

        // Remove session
        sessions.remove(&target_id);

        // Clear active if it was the active session
        let mut active_id = self.active_session_id.lock().await;
        if active_id.as_ref() == Some(&target_id) {
            *active_id = None;
        }

        Ok(())
    }

    /// Get the active session ID
    pub async fn active_session_id(&self) -> Option<String> {
        self.active_session_id.lock().await.clone()
    }

    /// Get session by ID
    pub async fn get_session(&self, id: &str) -> Option<ManagedSession> {
        let sessions = self.sessions.lock().await;
        sessions.get(id).cloned()
    }

    /// List all session names
    pub async fn list_sessions(&self) -> Vec<String> {
        let sessions = self.sessions.lock().await;
        sessions.values().map(|s| s.display.name.clone()).collect()
    }

    /// Get sessions waiting for input
    pub async fn get_waiting_sessions(&self) -> Vec<String> {
        let sessions = self.sessions.lock().await;
        sessions
            .values()
            .filter(|s| matches!(
                s.state,
                crate::cli::chat::managed_session::SessionState::WaitingForInput
            ))
            .map(|s| s.display.name.clone())
            .collect()
    }

    /// Update session state
    pub async fn update_session_state(
        &mut self,
        id: &str,
        new_state: crate::cli::chat::managed_session::SessionState,
    ) -> Result<()> {
        let mut sessions = self.sessions.lock().await;

        if let Some(session) = sessions.get_mut(id) {
            session.update_state(new_state)?;

            // Update display status
            session.display.status = match new_state {
                crate::cli::chat::managed_session::SessionState::Active => SessionStatus::Active,
                crate::cli::chat::managed_session::SessionState::WaitingForInput => {
                    SessionStatus::WaitingForInput
                }
                crate::cli::chat::managed_session::SessionState::Processing => {
                    SessionStatus::Processing
                }
            };
        }

        Ok(())
    }

    /// Get state change sender for new sessions
    pub fn state_sender(&self) -> mpsc::UnboundedSender<SessionStateChange> {
        self.state_tx.clone()
    }

    /// Process state changes from background sessions
    pub async fn process_state_changes(&mut self) -> Result<()> {
        while let Ok(change) = self.state_rx.try_recv() {
            match change {
                SessionStateChange::NeedsInput(id) => {
                    self.update_session_state(
                        &id,
                        crate::cli::chat::managed_session::SessionState::WaitingForInput,
                    )
                    .await?;
                }
                SessionStateChange::Processing(id) => {
                    self.update_session_state(
                        &id,
                        crate::cli::chat::managed_session::SessionState::Processing,
                    )
                    .await?;
                }
                SessionStateChange::Completed(id) => {
                    // Mark as completed
                    let mut sessions = self.sessions.lock().await;
                    if let Some(session) = sessions.get_mut(&id) {
                        session.display.status = SessionStatus::Completed;
                    }
                }
                SessionStateChange::Error(id, error) => {
                    let mut sessions = self.sessions.lock().await;
                    if let Some(session) = sessions.get_mut(&id) {
                        session.last_error = Some(error);
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_session() {
        let mut coordinator = MultiSessionCoordinator::new(CoordinatorConfig::default());

        let id = coordinator
            .create_session("test-1".to_string(), SessionType::Development, Some("dev-session".to_string()))
            .await
            .unwrap();

        assert_eq!(id, "test-1");
        assert_eq!(coordinator.active_session_id().await, Some("test-1".to_string()));
    }

    #[tokio::test]
    async fn test_create_multiple_sessions() {
        let mut coordinator = MultiSessionCoordinator::new(CoordinatorConfig::default());

        coordinator
            .create_session("test-1".to_string(), SessionType::Development, Some("session-1".to_string()))
            .await
            .unwrap();

        coordinator
            .create_session("test-2".to_string(), SessionType::Debug, Some("session-2".to_string()))
            .await
            .unwrap();

        let sessions = coordinator.list_sessions().await;
        assert_eq!(sessions.len(), 2);
        assert!(sessions.contains(&"session-1".to_string()));
        assert!(sessions.contains(&"session-2".to_string()));
    }

    #[tokio::test]
    async fn test_session_limit() {
        let config = CoordinatorConfig {
            max_active_sessions: 2,
            ..Default::default()
        };
        let mut coordinator = MultiSessionCoordinator::new(config);

        coordinator
            .create_session("test-1".to_string(), SessionType::Development, Some("s1".to_string()))
            .await
            .unwrap();

        coordinator
            .create_session("test-2".to_string(), SessionType::Development, Some("s2".to_string()))
            .await
            .unwrap();

        let result = coordinator
            .create_session("test-3".to_string(), SessionType::Development, Some("s3".to_string()))
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Maximum active sessions"));
    }

    #[tokio::test]
    async fn test_switch_session() {
        let mut coordinator = MultiSessionCoordinator::new(CoordinatorConfig::default());

        coordinator
            .create_session("test-1".to_string(), SessionType::Development, Some("session-1".to_string()))
            .await
            .unwrap();

        coordinator
            .create_session("test-2".to_string(), SessionType::Debug, Some("session-2".to_string()))
            .await
            .unwrap();

        coordinator.switch_session("session-2").await.unwrap();
        assert_eq!(coordinator.active_session_id().await, Some("test-2".to_string()));
    }

    #[tokio::test]
    async fn test_close_session() {
        let mut coordinator = MultiSessionCoordinator::new(CoordinatorConfig::default());

        coordinator
            .create_session("test-1".to_string(), SessionType::Development, Some("session-1".to_string()))
            .await
            .unwrap();

        coordinator.close_session("session-1").await.unwrap();

        let sessions = coordinator.list_sessions().await;
        assert_eq!(sessions.len(), 0);
        assert_eq!(coordinator.active_session_id().await, None);
    }

    #[tokio::test]
    async fn test_get_waiting_sessions() {
        let mut coordinator = MultiSessionCoordinator::new(CoordinatorConfig::default());

        let id = coordinator
            .create_session("test-1".to_string(), SessionType::Development, Some("session-1".to_string()))
            .await
            .unwrap();

        coordinator
            .update_session_state(
                &id,
                crate::cli::chat::managed_session::SessionState::WaitingForInput,
            )
            .await
            .unwrap();

        let waiting = coordinator.get_waiting_sessions().await;
        assert_eq!(waiting.len(), 1);
        assert_eq!(waiting[0], "session-1");
    }
}
