use std::io;
use crossterm::execute;
use crossterm::cursor;
use crossterm::terminal;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use eyre::Result;

pub struct ContextStats {
    pub worktree_name: Option<String>,
    pub session_type: Option<String>,
    pub tokens_used: usize,
    pub tokens_limit: usize,
    pub message_count: usize,
}

impl ContextStats {
    pub fn new() -> Self {
        Self {
            worktree_name: None,
            session_type: None,
            tokens_used: 0,
            tokens_limit: 200_000,
            message_count: 0,
        }
    }

    pub fn update_worktree(&mut self, name: String, session_type: String) {
        self.worktree_name = Some(name);
        self.session_type = Some(session_type);
    }

    pub fn update_tokens(&mut self, used: usize) {
        self.tokens_used = used;
    }

    pub fn increment_messages(&mut self) {
        self.message_count += 1;
    }

    /// Render the stats widget in the top-right corner
    pub fn render(&self) -> Result<()> {
        let mut stdout = io::stdout();
        
        // Save cursor position
        let original_pos = cursor::position().ok();
        
        // Get terminal size
        let (cols, _rows) = terminal::size()?;
        
        // Calculate widget position (top-right)
        let width = 30;
        let height = 6;
        let x = cols.saturating_sub(width + 1);
        let y = 0;
        
        // Create mini terminal for widget
        let backend = CrosstermBackend::new(&mut stdout);
        let mut terminal = Terminal::new(backend)?;
        
        terminal.draw(|frame| {
            let area = Rect {
                x,
                y,
                width,
                height,
            };
            
            self.render_widget(frame, area);
        })?;
        
        // Restore cursor position
        if let Some((col, row)) = original_pos {
            execute!(stdout, cursor::MoveTo(col, row))?;
        }
        
        Ok(())
    }

    fn render_widget(&self, frame: &mut ratatui::Frame, area: Rect) {
        let usage_percent = (self.tokens_used as f64 / self.tokens_limit as f64 * 100.0) as u16;
        let usage_color = if usage_percent > 90 {
            Color::Red
        } else if usage_percent > 70 {
            Color::Yellow
        } else {
            Color::Green
        };

        let mut lines = vec![];

        // Worktree info
        if let Some(ref name) = self.worktree_name {
            lines.push(Line::from(vec![
                Span::styled("🌳 ", Style::default().fg(Color::Green)),
                Span::styled(name, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]));
            
            if let Some(ref session_type) = self.session_type {
                lines.push(Line::from(vec![
                    Span::raw("   "),
                    Span::styled(
                        format!("[{}]", session_type),
                        Style::default().fg(Color::Yellow)
                    ),
                ]));
            }
        } else {
            lines.push(Line::from(Span::styled(
                "No worktree",
                Style::default().fg(Color::DarkGray)
            )));
        }

        // Context usage
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("Context: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}%", usage_percent),
                Style::default().fg(usage_color).add_modifier(Modifier::BOLD)
            ),
        ]));
        
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                format!("{}/{}", format_tokens(self.tokens_used), format_tokens(self.tokens_limit)),
                Style::default().fg(Color::DarkGray)
            ),
        ]));

        // Message count
        lines.push(Line::from(vec![
            Span::styled("Messages: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                self.message_count.to_string(),
                Style::default().fg(Color::Cyan)
            ),
        ]));

        let paragraph = Paragraph::new(lines)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)))
            .alignment(Alignment::Left);

        frame.render_widget(paragraph, area);
    }
}

fn format_tokens(tokens: usize) -> String {
    if tokens >= 1_000_000 {
        format!("{:.1}M", tokens as f64 / 1_000_000.0)
    } else if tokens >= 1_000 {
        format!("{:.1}K", tokens as f64 / 1_000.0)
    } else {
        tokens.to_string()
    }
}

impl Default for ContextStats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_tokens() {
        assert_eq!(format_tokens(500), "500");
        assert_eq!(format_tokens(1_500), "1.5K");
        assert_eq!(format_tokens(150_000), "150.0K");
        assert_eq!(format_tokens(1_500_000), "1.5M");
    }

    #[test]
    fn test_stats_creation() {
        let stats = ContextStats::new();
        assert_eq!(stats.tokens_used, 0);
        assert_eq!(stats.message_count, 0);
        assert!(stats.worktree_name.is_none());
    }

    #[test]
    fn test_update_worktree() {
        let mut stats = ContextStats::new();
        stats.update_worktree("feature-auth".to_string(), "Feature".to_string());
        assert_eq!(stats.worktree_name, Some("feature-auth".to_string()));
        assert_eq!(stats.session_type, Some("Feature".to_string()));
    }

    #[test]
    fn test_increment_messages() {
        let mut stats = ContextStats::new();
        stats.increment_messages();
        stats.increment_messages();
        assert_eq!(stats.message_count, 2);
    }
}
