//! Live indicator for session state updates

use std::io::Write;
use crossterm::{execute, style, cursor};
use eyre::Result;

/// Live indicator that can be updated
pub struct LiveIndicator {
    /// Number of notifications
    pub notification_count: usize,
    /// Whether background work is active
    pub background_active: bool,
    /// Last rendered state
    last_state: String,
}

impl LiveIndicator {
    pub fn new() -> Self {
        Self {
            notification_count: 0,
            background_active: false,
            last_state: String::new(),
        }
    }
    
    /// Update and render if state changed
    pub fn update_and_render<W: Write>(
        &mut self,
        notification_count: usize,
        background_active: bool,
        writer: &mut W,
    ) -> Result<()> {
        self.notification_count = notification_count;
        self.background_active = background_active;
        
        let new_state = self.render_state();
        
        // Only render if state changed
        if new_state != self.last_state {
            self.render_inline(writer)?;
            self.last_state = new_state;
        }
        
        Ok(())
    }
    
    fn render_state(&self) -> String {
        let mut parts = Vec::new();
        
        if self.notification_count > 0 {
            parts.push(format!("📬 {}", self.notification_count));
        }
        
        if self.background_active {
            parts.push("⚙️".to_string());
        }
        
        parts.join(" ")
    }
    
    fn render_inline<W: Write>(&self, writer: &mut W) -> Result<()> {
        let state = self.render_state();
        
        if !state.is_empty() {
            execute!(
                writer,
                cursor::SavePosition,
                cursor::MoveToColumn(0),
                style::Print(&state),
                cursor::RestorePosition,
            )?;
        }
        
        Ok(())
    }
}

impl Default for LiveIndicator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_indicator_state() {
        let mut indicator = LiveIndicator::new();
        assert_eq!(indicator.render_state(), "");
        
        indicator.notification_count = 2;
        assert!(indicator.render_state().contains("📬 2"));
        
        indicator.background_active = true;
        assert!(indicator.render_state().contains("⚙️"));
    }
}
