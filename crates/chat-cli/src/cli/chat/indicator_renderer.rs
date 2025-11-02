use std::io;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use crate::theme::session::SessionStatus;
use super::indicator::SessionIndicator;

pub struct IndicatorRenderer {
    indicator: SessionIndicator,
    enabled: bool,
}

impl IndicatorRenderer {
    pub fn new(enabled: bool) -> Self {
        Self {
            indicator: SessionIndicator::new(),
            enabled,
        }
    }

    pub fn update(&mut self, waiting_sessions: &[(String, SessionStatus)]) -> io::Result<()> {
        if !self.enabled || waiting_sessions.is_empty() {
            return Ok(());
        }

        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        
        self.indicator.render(&mut terminal, waiting_sessions)?;
        
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        
        Ok(())
    }

    pub fn clear(&mut self) -> io::Result<()> {
        if !self.enabled {
            return Ok(());
        }
        
        let mut stdout = io::stdout();
        execute!(stdout, LeaveAlternateScreen)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_renderer_creation() {
        let renderer = IndicatorRenderer::new(true);
        assert!(renderer.enabled);
    }

    #[test]
    fn test_renderer_disabled() {
        let renderer = IndicatorRenderer::new(false);
        assert!(!renderer.enabled);
    }

    #[test]
    fn test_update_when_disabled() {
        let mut renderer = IndicatorRenderer::new(false);
        let sessions = vec![("test".to_string(), SessionStatus::WaitingForInput)];
        let result = renderer.update(&sessions);
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_empty_sessions() {
        let mut renderer = IndicatorRenderer::new(true);
        let sessions = vec![];
        let result = renderer.update(&sessions);
        assert!(result.is_ok());
    }

    #[test]
    fn test_clear_when_disabled() {
        let mut renderer = IndicatorRenderer::new(false);
        let result = renderer.clear();
        assert!(result.is_ok());
    }
}
