//! Status bar for displaying session information

use std::io::Write;
use crossterm::{execute, style};
use eyre::Result;

use crate::theme::StyledText;

/// Status bar information
pub struct StatusBar {
    /// Current session name
    pub session_name: String,
    /// Total number of sessions
    pub session_count: usize,
    /// Number of sessions with notifications
    pub notification_count: usize,
    /// Whether background work is in progress
    pub background_active: bool,
}

impl StatusBar {
    pub fn new(session_name: String, session_count: usize) -> Self {
        Self {
            session_name,
            session_count,
            notification_count: 0,
            background_active: false,
        }
    }
    
    /// Update status bar state
    pub fn update(&mut self, notification_count: usize, background_active: bool) {
        self.notification_count = notification_count;
        self.background_active = background_active;
    }
    
    /// Render the status bar
    pub fn render<W: Write>(&self, writer: &mut W) -> Result<()> {
        execute!(
            writer,
            StyledText::secondary_fg(),
            style::Print("─".repeat(60)),
            style::Print("\n"),
            StyledText::reset()
        )?;
        
        // Session info
        execute!(
            writer,
            StyledText::info_fg(),
            style::Print("Session: "),
            StyledText::reset(),
            style::Print(&self.session_name),
            style::Print("  "),
        )?;
        
        // Session count
        execute!(
            writer,
            StyledText::secondary_fg(),
            style::Print(format!("({}/{})", 1, self.session_count)),
            StyledText::reset(),
            style::Print("  "),
        )?;
        
        // Notifications
        if self.notification_count > 0 {
            execute!(
                writer,
                StyledText::success_fg(),
                style::Print(format!("📬 {} ", self.notification_count)),
                StyledText::reset(),
            )?;
        }
        
        // Background work
        if self.background_active {
            execute!(
                writer,
                StyledText::info_fg(),
                style::Print("⚙️  Processing"),
                StyledText::reset(),
            )?;
        }
        
        execute!(
            writer,
            style::Print("\n"),
            StyledText::secondary_fg(),
            style::Print("─".repeat(60)),
            style::Print("\n"),
            StyledText::reset()
        )?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_status_bar_creation() {
        let bar = StatusBar::new("test-session".to_string(), 3);
        assert_eq!(bar.session_name, "test-session");
        assert_eq!(bar.session_count, 3);
        assert_eq!(bar.notification_count, 0);
        assert!(!bar.background_active);
    }
    
    #[test]
    fn test_status_bar_render() {
        let bar = StatusBar::new("test".to_string(), 2);
        let mut output = Vec::new();
        assert!(bar.render(&mut output).is_ok());
        assert!(!output.is_empty());
    }
}
