//! Session switcher with UX integration

use std::io::Write;

use eyre::Result;

use crate::cli::chat::coordinator::MultiSessionCoordinator;
use crate::cli::chat::session_transition::SessionTransition;
use crate::cli::chat::terminal_ui::TerminalUI;
use crate::cli::chat::visual_feedback::VisualFeedback;

/// Handles session switching with UX
pub struct SessionSwitcher {
    ui: TerminalUI,
    transition: SessionTransition,
}

impl SessionSwitcher {
    pub fn new() -> Self {
        Self {
            ui: TerminalUI::new(),
            transition: SessionTransition::new(),
        }
    }

    /// Enable/disable smooth transitions
    pub fn set_smooth_transitions(&mut self, enabled: bool) {
        self.transition.set_replay_buffer(enabled);
    }

    /// Switch to a different session with visual feedback
    pub async fn switch_to<W: Write>(
        &mut self,
        coordinator: &mut MultiSessionCoordinator,
        target_name: &str,
        writer: &mut W,
    ) -> Result<()> {
        // Show progress
        VisualFeedback::progress(writer, &format!("Switching to '{}'", target_name))?;

        // Get current session name
        let current_id = coordinator.active_session_id().await;
        let current_name = if let Some(id) = current_id {
            coordinator.get_session(&id).await
        } else {
            None
        };

        // Perform switch
        match coordinator.switch_session(target_name).await {
            Ok(_) => {
                VisualFeedback::clear_progress(writer)?;
                VisualFeedback::success(writer, &format!("Switched to '{}'", target_name))?;
            }
            Err(e) => {
                VisualFeedback::clear_progress(writer)?;
                VisualFeedback::error(writer, &format!("Failed to switch: {}", e))?;
                return Err(e);
            }
        }

        // Get new session ID
        let new_id = coordinator.active_session_id().await
            .ok_or_else(|| eyre::eyre!("Failed to get new session ID"))?;

        // Perform transition
        self.transition.transition_to(coordinator, &new_id, writer).await?;

        // Show feedback
        if let Some(from) = current_name {
            self.ui.show_switch_message(writer, &from, target_name)?;
        }

        // Update indicator
        self.ui.render_indicator(writer, coordinator)?;

        Ok(())
    }

    /// List all sessions with visual formatting
    pub async fn list_sessions<W: Write>(
        &self,
        coordinator: &MultiSessionCoordinator,
        writer: &mut W,
    ) -> Result<()> {
        let sessions = coordinator.list_sessions().await;

        let mut session_info = Vec::new();
        for name in sessions {
            // Simplified - all sessions shown as Development type
            session_info.push((name, crate::theme::session::SessionType::Development, false));
        }

        self.ui.show_session_list(writer, &session_info)?;
        Ok(())
    }

    /// Clear screen for session switch
    pub fn clear_screen<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.ui.clear_for_switch(writer)
    }

    /// Update session indicator
    pub fn update_indicator<W: Write>(
        &self,
        coordinator: &MultiSessionCoordinator,
        writer: &mut W,
    ) -> Result<()> {
        self.ui.render_indicator(writer, coordinator)
    }
}

impl Default for SessionSwitcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::chat::coordinator::CoordinatorConfig;

    #[tokio::test]
    async fn test_create_switcher() {
        let switcher = SessionSwitcher::new();
        assert!(switcher.ui.show_indicator);
    }

    #[tokio::test]
    async fn test_list_sessions() {
        let coordinator = MultiSessionCoordinator::new(CoordinatorConfig::default());
        let switcher = SessionSwitcher::new();
        let mut buffer = Vec::new();

        let result = switcher.list_sessions(&coordinator, &mut buffer).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_clear_screen() {
        let switcher = SessionSwitcher::new();
        let mut buffer = Vec::new();

        let result = switcher.clear_screen(&mut buffer);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_set_smooth_transitions() {
        let mut switcher = SessionSwitcher::new();
        switcher.set_smooth_transitions(false);
        assert!(!switcher.transition.replay_buffer);
    }
}
