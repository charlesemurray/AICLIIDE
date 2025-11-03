//! Terminal UI manager for multi-session interface

use std::io::Write;

use crossterm::{
    cursor, execute, style::{Color, Print, ResetColor, SetForegroundColor}, terminal::{self, Clear, ClearType}
};
use eyre::Result;

use crate::cli::chat::coordinator::MultiSessionCoordinator;
use crate::theme::session::{SessionColors, SessionType};

/// Terminal UI manager for sessions
pub struct TerminalUI {
    show_indicator: bool,
}

impl TerminalUI {
    pub fn new() -> Self {
        Self { show_indicator: true }
    }

    /// Enable/disable session indicator
    pub fn set_indicator_visible(&mut self, visible: bool) {
        self.show_indicator = visible;
    }

    /// Render session indicator in top-right corner
    pub fn render_indicator<W: Write>(&self, writer: &mut W, coordinator: &MultiSessionCoordinator) -> Result<()> {
        if !self.show_indicator {
            return Ok(());
        }

        let (cols, _) = terminal::size()?;
        let waiting = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(coordinator.get_waiting_sessions())
        });

        if waiting.is_empty() {
            return Ok(());
        }

        // Save cursor position
        let pos = cursor::position().ok();

        // Move to top-right corner
        execute!(writer, cursor::MoveTo(cols.saturating_sub(25), 0))?;
        execute!(writer, SetForegroundColor(Color::Yellow))?;
        execute!(writer, Print(format!("⏸ {} waiting", waiting.len())))?;
        execute!(writer, ResetColor)?;

        // Restore cursor
        if let Some((col, row)) = pos {
            execute!(writer, cursor::MoveTo(col, row))?;
        }

        Ok(())
    }

    /// Display session switch message
    pub fn show_switch_message<W: Write>(&self, writer: &mut W, from: &str, to: &str) -> Result<()> {
        execute!(writer, SetForegroundColor(Color::Cyan))?;
        writeln!(writer, "\n→ Switched from '{}' to '{}'", from, to)?;
        execute!(writer, ResetColor)?;
        Ok(())
    }

    /// Display session list
    pub fn show_session_list<W: Write>(&self, writer: &mut W, sessions: &[(String, SessionType, bool)]) -> Result<()> {
        let colors = SessionColors::default();
        writeln!(writer, "\nActive Sessions:")?;
        for (name, session_type, is_active) in sessions {
            let marker = if *is_active { "→" } else { " " };
            let color = session_type.color(&colors);
            execute!(writer, SetForegroundColor(color))?;
            writeln!(writer, "{} {} [{}]", marker, name, session_type.prefix())?;
        }
        execute!(writer, ResetColor)?;
        Ok(())
    }

    /// Clear screen for session switch
    pub fn clear_for_switch<W: Write>(&self, writer: &mut W) -> Result<()> {
        execute!(writer, Clear(ClearType::All), cursor::MoveTo(0, 0))?;
        Ok(())
    }
}

impl Default for TerminalUI {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_terminal_ui() {
        let ui = TerminalUI::new();
        assert!(ui.show_indicator);
    }

    #[test]
    fn test_set_indicator_visible() {
        let mut ui = TerminalUI::new();
        ui.set_indicator_visible(false);
        assert!(!ui.show_indicator);
    }

    #[test]
    fn test_show_switch_message() {
        let ui = TerminalUI::new();
        let mut buffer = Vec::new();
        
        let result = ui.show_switch_message(&mut buffer, "session-1", "session-2");
        assert!(result.is_ok());
        
        let output = String::from_utf8_lossy(&buffer);
        assert!(output.contains("session-1"));
        assert!(output.contains("session-2"));
    }

    #[test]
    fn test_show_session_list() {
        let ui = TerminalUI::new();
        let mut buffer = Vec::new();
        
        let sessions = vec![
            ("dev-session".to_string(), SessionType::Development, true),
            ("debug-session".to_string(), SessionType::Debug, false),
        ];
        
        let result = ui.show_session_list(&mut buffer, &sessions);
        assert!(result.is_ok());
        
        let output = String::from_utf8_lossy(&buffer);
        assert!(output.contains("dev-session"));
        assert!(output.contains("debug-session"));
    }

    #[test]
    fn test_clear_for_switch() {
        let ui = TerminalUI::new();
        let mut buffer = Vec::new();
        
        let result = ui.clear_for_switch(&mut buffer);
        assert!(result.is_ok());
    }
}
