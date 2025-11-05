//! Terminal UI manager for multi-session interface

use std::io::Write;

use crossterm::style::{
    Color,
    Print,
    ResetColor,
    SetForegroundColor,
};
use crossterm::terminal::{
    self,
    Clear,
    ClearType,
};
use crossterm::{
    cursor,
    execute,
};
use eyre::Result;

use crate::cli::chat::coordinator::MultiSessionCoordinator;
use crate::theme::session::{
    SessionColors,
    SessionType,
};

/// Terminal UI manager for sessions
pub struct TerminalUI {
    pub(crate) show_indicator: bool,
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

        let (cols, rows) = terminal::size()?;
        
        // Get session info
        let (all_sessions, waiting_sessions) = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let all = coordinator.list_sessions().await;
                let waiting = coordinator.get_waiting_sessions().await;
                (all, waiting)
            })
        });

        if all_sessions.is_empty() {
            return Ok(());
        }

        // Save cursor position
        let pos = cursor::position().ok();

        // Build indicator box
        let max_width: usize = 30;
        let title = format!("┌─ Sessions ({}) ", all_sessions.len());
        let padding = "─".repeat(max_width.saturating_sub(title.len()).saturating_sub(1));
        let header = format!("{}{}┐", title, padding);
        
        // Position in top-right corner
        let start_col = cols.saturating_sub(max_width as u16);
        
        // Draw header
        execute!(writer, cursor::MoveTo(start_col, 0))?;
        execute!(writer, SetForegroundColor(Color::Cyan))?;
        execute!(writer, Print(&header))?;
        execute!(writer, ResetColor)?;

        // Draw sessions (max 5 to avoid taking too much space)
        for (idx, session_name) in all_sessions.iter().take(5).enumerate() {
            let row = (idx + 1) as u16;
            if row >= rows {
                break;
            }
            
            execute!(writer, cursor::MoveTo(start_col, row))?;
            execute!(writer, SetForegroundColor(Color::Cyan))?;
            execute!(writer, Print("│ "))?;
            
            // Check if waiting for input
            if waiting_sessions.contains(session_name) {
                execute!(writer, SetForegroundColor(Color::Yellow))?;
                execute!(writer, Print("⏸ "))?;
            } else {
                execute!(writer, SetForegroundColor(Color::Green))?;
                execute!(writer, Print("▶ "))?;
            }
            
            // Truncate name if too long
            let max_name_len = max_width.saturating_sub(6);
            let display_name = if session_name.len() > max_name_len {
                format!("{}…", &session_name[..max_name_len.saturating_sub(1)])
            } else {
                session_name.clone()
            };
            
            execute!(writer, SetForegroundColor(Color::White))?;
            execute!(writer, Print(&display_name))?;
            
            // Pad to box width
            let padding_len = max_width.saturating_sub(display_name.len()).saturating_sub(5);
            execute!(writer, Print(" ".repeat(padding_len)))?;
            execute!(writer, SetForegroundColor(Color::Cyan))?;
            execute!(writer, Print("│"))?;
            execute!(writer, ResetColor)?;
        }
        
        // Draw footer
        let footer_row = all_sessions.len().min(5) as u16 + 1;
        if footer_row < rows {
            execute!(writer, cursor::MoveTo(start_col, footer_row))?;
            execute!(writer, SetForegroundColor(Color::Cyan))?;
            let footer = format!("└{}┘", "─".repeat(max_width.saturating_sub(2)));
            execute!(writer, Print(&footer))?;
            execute!(writer, ResetColor)?;
        }

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
