//! Visual feedback for session operations

use std::io::Write;

use crossterm::execute;
use crossterm::style::{
    Color,
    Print,
    ResetColor,
    SetForegroundColor,
};
use eyre::Result;

/// Visual feedback manager
pub struct VisualFeedback;

impl VisualFeedback {
    /// Show success message
    pub fn success<W: Write>(writer: &mut W, message: &str) -> Result<()> {
        execute!(writer, SetForegroundColor(Color::Green))?;
        writeln!(writer, "✓ {}", message)?;
        execute!(writer, ResetColor)?;
        Ok(())
    }

    /// Show error message
    pub fn error<W: Write>(writer: &mut W, message: &str) -> Result<()> {
        execute!(writer, SetForegroundColor(Color::Red))?;
        writeln!(writer, "✗ {}", message)?;
        execute!(writer, ResetColor)?;
        Ok(())
    }

    /// Show info message
    pub fn info<W: Write>(writer: &mut W, message: &str) -> Result<()> {
        execute!(writer, SetForegroundColor(Color::Cyan))?;
        writeln!(writer, "ℹ {}", message)?;
        execute!(writer, ResetColor)?;
        Ok(())
    }

    /// Show warning message
    pub fn warning<W: Write>(writer: &mut W, message: &str) -> Result<()> {
        execute!(writer, SetForegroundColor(Color::Yellow))?;
        writeln!(writer, "⚠ {}", message)?;
        execute!(writer, ResetColor)?;
        Ok(())
    }

    /// Show progress indicator
    pub fn progress<W: Write>(writer: &mut W, message: &str) -> Result<()> {
        execute!(writer, SetForegroundColor(Color::Blue))?;
        write!(writer, "⏳ {}...", message)?;
        execute!(writer, ResetColor)?;
        writer.flush()?;
        Ok(())
    }

    /// Clear progress indicator
    pub fn clear_progress<W: Write>(writer: &mut W) -> Result<()> {
        write!(writer, "\r")?;
        writer.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_success_message() {
        let mut buffer = Vec::new();
        let result = VisualFeedback::success(&mut buffer, "Operation completed");
        assert!(result.is_ok());
        let output = String::from_utf8_lossy(&buffer);
        assert!(output.contains("Operation completed"));
    }

    #[test]
    fn test_error_message() {
        let mut buffer = Vec::new();
        let result = VisualFeedback::error(&mut buffer, "Operation failed");
        assert!(result.is_ok());
        let output = String::from_utf8_lossy(&buffer);
        assert!(output.contains("Operation failed"));
    }

    #[test]
    fn test_info_message() {
        let mut buffer = Vec::new();
        let result = VisualFeedback::info(&mut buffer, "Information");
        assert!(result.is_ok());
        let output = String::from_utf8_lossy(&buffer);
        assert!(output.contains("Information"));
    }

    #[test]
    fn test_warning_message() {
        let mut buffer = Vec::new();
        let result = VisualFeedback::warning(&mut buffer, "Warning message");
        assert!(result.is_ok());
        let output = String::from_utf8_lossy(&buffer);
        assert!(output.contains("Warning message"));
    }

    #[test]
    fn test_progress_indicator() {
        let mut buffer = Vec::new();
        let result = VisualFeedback::progress(&mut buffer, "Loading");
        assert!(result.is_ok());
        let output = String::from_utf8_lossy(&buffer);
        assert!(output.contains("Loading"));
    }

    #[test]
    fn test_clear_progress() {
        let mut buffer = Vec::new();
        let result = VisualFeedback::clear_progress(&mut buffer);
        assert!(result.is_ok());
    }
}
