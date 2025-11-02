//! Session execution mode for foreground vs background operation

use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

use crate::cli::chat::managed_session::OutputBuffer;

/// Execution mode for a chat session
#[derive(Debug, Clone)]
pub enum SessionMode {
    /// Foreground mode - direct output to terminal
    Foreground,
    /// Background mode - buffer output for later replay
    Background {
        /// Shared output buffer
        buffer: Arc<Mutex<OutputBuffer>>,
        /// Channel to signal state changes
        state_tx: mpsc::UnboundedSender<SessionStateChange>,
    },
}

/// State change notifications from background sessions
#[derive(Debug, Clone)]
pub enum SessionStateChange {
    /// Session needs user input
    NeedsInput(String), // conversation_id
    /// Session is processing
    Processing(String), // conversation_id
    /// Session completed successfully
    Completed(String), // conversation_id
    /// Session encountered an error
    Error(String, String), // conversation_id, error_message
}

impl SessionMode {
    /// Check if session is in foreground mode
    pub fn is_foreground(&self) -> bool {
        matches!(self, SessionMode::Foreground)
    }

    /// Check if session is in background mode
    pub fn is_background(&self) -> bool {
        matches!(self, SessionMode::Background { .. })
    }

    /// Get the output buffer if in background mode
    pub fn buffer(&self) -> Option<Arc<Mutex<OutputBuffer>>> {
        match self {
            SessionMode::Background { buffer, .. } => Some(buffer.clone()),
            SessionMode::Foreground => None,
        }
    }

    /// Send a state change notification (only in background mode)
    pub fn notify_state_change(&self, change: SessionStateChange) {
        if let SessionMode::Background { state_tx, .. } = self {
            let _ = state_tx.send(change);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_mode_is_foreground() {
        let mode = SessionMode::Foreground;
        assert!(mode.is_foreground());
        assert!(!mode.is_background());
    }

    #[test]
    fn test_session_mode_is_background() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let buffer = Arc::new(Mutex::new(OutputBuffer::new(1000)));
        let mode = SessionMode::Background {
            buffer,
            state_tx: tx,
        };
        assert!(!mode.is_foreground());
        assert!(mode.is_background());
    }

    #[test]
    fn test_session_mode_buffer() {
        let mode = SessionMode::Foreground;
        assert!(mode.buffer().is_none());

        let (tx, _rx) = mpsc::unbounded_channel();
        let buffer = Arc::new(Mutex::new(OutputBuffer::new(1000)));
        let mode = SessionMode::Background {
            buffer: buffer.clone(),
            state_tx: tx,
        };
        assert!(mode.buffer().is_some());
    }

    #[test]
    fn test_notify_state_change() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let buffer = Arc::new(Mutex::new(OutputBuffer::new(1000)));
        let mode = SessionMode::Background {
            buffer,
            state_tx: tx,
        };

        mode.notify_state_change(SessionStateChange::NeedsInput("test-id".to_string()));

        // Should receive the notification
        let change = rx.try_recv().unwrap();
        match change {
            SessionStateChange::NeedsInput(id) => assert_eq!(id, "test-id"),
            _ => panic!("Wrong state change type"),
        }
    }

    #[test]
    fn test_notify_state_change_foreground_no_op() {
        let mode = SessionMode::Foreground;
        // Should not panic
        mode.notify_state_change(SessionStateChange::NeedsInput("test-id".to_string()));
    }
}
