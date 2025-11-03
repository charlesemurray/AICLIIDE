//! Session transition manager for smooth switching UX

use std::io::Write;

use eyre::Result;

use crate::cli::chat::coordinator::MultiSessionCoordinator;

/// Manages smooth transitions between sessions
pub struct SessionTransition {
    clear_on_switch: bool,
    pub(crate) replay_buffer: bool,
}

impl SessionTransition {
    pub fn new() -> Self {
        Self {
            clear_on_switch: false,
            replay_buffer: true,
        }
    }

    /// Set whether to clear screen on switch
    pub fn set_clear_on_switch(&mut self, clear: bool) {
        self.clear_on_switch = clear;
    }

    /// Set whether to replay buffer on switch
    pub fn set_replay_buffer(&mut self, replay: bool) {
        self.replay_buffer = replay;
    }

    /// Perform transition to new session
    pub async fn transition_to<W: Write>(
        &self,
        coordinator: &MultiSessionCoordinator,
        target_id: &str,
        writer: &mut W,
    ) -> Result<()> {
        // Clear screen if configured
        if self.clear_on_switch {
            crossterm::execute!(
                writer,
                crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
                crossterm::cursor::MoveTo(0, 0)
            )?;
        }

        // Replay buffer if configured
        if self.replay_buffer {
            self.replay_session_buffer(coordinator, target_id, writer).await?;
        }

        Ok(())
    }

    /// Replay buffered output for session
    async fn replay_session_buffer<W: Write>(
        &self,
        coordinator: &MultiSessionCoordinator,
        session_id: &str,
        writer: &mut W,
    ) -> Result<()> {
        let state = coordinator.state.lock().await;
        if let Some(session) = state.sessions.get(session_id) {
            let buffer = session.output_buffer.lock().await;
            buffer.replay(writer)?;
        }
        Ok(())
    }
}

impl Default for SessionTransition {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_transition() {
        let transition = SessionTransition::new();
        assert!(!transition.clear_on_switch);
        assert!(transition.replay_buffer);
    }

    #[test]
    fn test_set_clear_on_switch() {
        let mut transition = SessionTransition::new();
        transition.set_clear_on_switch(true);
        assert!(transition.clear_on_switch);
    }

    #[test]
    fn test_set_replay_buffer() {
        let mut transition = SessionTransition::new();
        transition.set_replay_buffer(false);
        assert!(!transition.replay_buffer);
    }
}
