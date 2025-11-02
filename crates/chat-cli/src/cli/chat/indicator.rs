use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::io;
use crate::theme::session::{SessionColors, SessionStatus};

pub struct SessionIndicator {
    colors: SessionColors,
}

impl SessionIndicator {
    pub fn new() -> Self {
        Self {
            colors: SessionColors::default(),
        }
    }

    pub fn render(
        &self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        waiting_sessions: &[(String, SessionStatus)],
    ) -> io::Result<()> {
        if waiting_sessions.is_empty() {
            return Ok(());
        }

        terminal.draw(|frame| {
            let area = frame.area();
            let indicator_area = self.calculate_indicator_area(area, waiting_sessions.len());

            let content = self.build_content(waiting_sessions);
            let block = Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Yellow));

            let paragraph = Paragraph::new(content)
                .block(block)
                .alignment(Alignment::Left);

            frame.render_widget(paragraph, indicator_area);
        })?;

        Ok(())
    }

    fn calculate_indicator_area(&self, area: Rect, session_count: usize) -> Rect {
        let width = 25;
        let height = (session_count + 2).min(10) as u16;
        
        Rect {
            x: area.width.saturating_sub(width + 1),
            y: 0,
            width,
            height,
        }
    }

    fn build_content(&self, sessions: &[(String, SessionStatus)]) -> Vec<Line<'static>> {
        let mut lines = vec![Line::from(Span::styled(
            "⏳ Waiting:",
            Style::default().fg(Color::Yellow),
        ))];

        for (name, _status) in sessions.iter().take(8) {
            lines.push(Line::from(Span::raw(format!("  • {}", name))));
        }

        if sessions.len() > 8 {
            lines.push(Line::from(Span::styled(
                format!("  +{} more", sessions.len() - 8),
                Style::default().fg(Color::DarkGray),
            )));
        }

        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_indicator_creation() {
        let indicator = SessionIndicator::new();
        assert!(indicator.colors.active.is_some());
    }

    #[test]
    fn test_calculate_indicator_area() {
        let indicator = SessionIndicator::new();
        let area = Rect { x: 0, y: 0, width: 100, height: 50 };
        
        let result = indicator.calculate_indicator_area(area, 3);
        assert_eq!(result.width, 25);
        assert_eq!(result.height, 5);
        assert_eq!(result.x, 74);
        assert_eq!(result.y, 0);
    }

    #[test]
    fn test_build_content_empty() {
        let indicator = SessionIndicator::new();
        let sessions = vec![];
        let content = indicator.build_content(&sessions);
        assert_eq!(content.len(), 1);
    }

    #[test]
    fn test_build_content_few_sessions() {
        let indicator = SessionIndicator::new();
        let sessions = vec![
            ("debug-api".to_string(), SessionStatus::WaitingForInput),
            ("test-fix".to_string(), SessionStatus::WaitingForInput),
        ];
        let content = indicator.build_content(&sessions);
        assert_eq!(content.len(), 3);
    }

    #[test]
    fn test_build_content_many_sessions() {
        let indicator = SessionIndicator::new();
        let sessions: Vec<_> = (0..10)
            .map(|i| (format!("session-{}", i), SessionStatus::WaitingForInput))
            .collect();
        let content = indicator.build_content(&sessions);
        assert_eq!(content.len(), 10); // 1 header + 8 sessions + 1 "more"
    }

    #[test]
    fn test_indicator_area_max_height() {
        let indicator = SessionIndicator::new();
        let area = Rect { x: 0, y: 0, width: 100, height: 50 };
        
        let result = indicator.calculate_indicator_area(area, 20);
        assert_eq!(result.height, 10);
    }
}
