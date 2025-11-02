//! Terminal state management for session switching

use std::io::Write;

use crossterm::{cursor, style, terminal};
use eyre::Result;

/// Saved terminal state for restoration
#[derive(Debug, Clone)]
pub struct TerminalState {
    /// Cursor position (column, row)
    pub cursor_position: Option<(u16, u16)>,
    /// Whether raw mode was enabled
    pub raw_mode_enabled: bool,
}

impl TerminalState {
    /// Capture current terminal state
    pub fn capture() -> Result<Self> {
        let cursor_position = cursor::position().ok();
        let raw_mode_enabled = false; // We don't use raw mode in chat

        Ok(Self {
            cursor_position,
            raw_mode_enabled,
        })
    }

    /// Restore terminal state
    pub fn restore<W: Write>(&self, writer: &mut W) -> Result<()> {
        if let Some((col, row)) = self.cursor_position {
            crossterm::execute!(writer, cursor::MoveTo(col, row))?;
        }

        if self.raw_mode_enabled {
            terminal::enable_raw_mode()?;
        } else {
            terminal::disable_raw_mode().ok(); // Ignore error if already disabled
        }

        Ok(())
    }

    /// Clear screen and reset cursor
    pub fn clear_screen<W: Write>(writer: &mut W) -> Result<()> {
        crossterm::execute!(writer, terminal::Clear(terminal::ClearType::All), cursor::MoveTo(0, 0))?;
        Ok(())
    }

    /// Save cursor position
    pub fn save_cursor<W: Write>(writer: &mut W) -> Result<()> {
        crossterm::execute!(writer, cursor::SavePosition)?;
        Ok(())
    }

    /// Restore cursor position
    pub fn restore_cursor<W: Write>(writer: &mut W) -> Result<()> {
        crossterm::execute!(writer, cursor::RestorePosition)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_state_capture() {
        let state = TerminalState::capture();
        assert!(state.is_ok());
        let state = state.unwrap();
        assert!(!state.raw_mode_enabled);
    }

    #[test]
    fn test_terminal_state_restore() {
        let state = TerminalState {
            cursor_position: Some((10, 5)),
            raw_mode_enabled: false,
        };

        let mut buffer = Vec::new();
        let result = state.restore(&mut buffer);
        assert!(result.is_ok());
    }

    #[test]
    fn test_clear_screen() {
        let mut buffer = Vec::new();
        let result = TerminalState::clear_screen(&mut buffer);
        assert!(result.is_ok());
        assert!(!buffer.is_empty());
    }

    #[test]
    fn test_save_restore_cursor() {
        let mut buffer = Vec::new();

        let result = TerminalState::save_cursor(&mut buffer);
        assert!(result.is_ok());

        let result = TerminalState::restore_cursor(&mut buffer);
        assert!(result.is_ok());
    }
}
