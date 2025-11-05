use std::io;
use std::path::PathBuf;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::execute;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use eyre::Result;

use crate::git::GitWorktreeInfo;
use crate::theme::session::SessionType;

pub enum SelectorAction {
    Selected(usize),
    CreateNew(String),
    Cancel,
}

pub struct WorktreeSelector {
    worktrees: Vec<GitWorktreeInfo>,
    state: ListState,
    input_mode: bool,
    input_buffer: String,
}

impl WorktreeSelector {
    pub fn new(worktrees: Vec<GitWorktreeInfo>) -> Self {
        let mut state = ListState::default();
        if !worktrees.is_empty() {
            state.select(Some(0));
        }
        
        Self {
            worktrees,
            state,
            input_mode: false,
            input_buffer: String::new(),
        }
    }

    pub fn run(mut self) -> Result<SelectorAction> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let result = self.event_loop(&mut terminal);

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

        result
    }

    fn event_loop(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<SelectorAction> {
        loop {
            terminal.draw(|f| self.render(f))?;

            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                if self.input_mode {
                    match key.code {
                        KeyCode::Enter => {
                            let name = self.input_buffer.clone();
                            return Ok(SelectorAction::CreateNew(name));
                        }
                        KeyCode::Esc => {
                            self.input_mode = false;
                            self.input_buffer.clear();
                        }
                        KeyCode::Char(c) => {
                            self.input_buffer.push(c);
                        }
                        KeyCode::Backspace => {
                            self.input_buffer.pop();
                        }
                        _ => {}
                    }
                } else {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            return Ok(SelectorAction::Cancel);
                        }
                        KeyCode::Char('n') => {
                            self.input_mode = true;
                        }
                        KeyCode::Enter => {
                            if let Some(selected) = self.state.selected() {
                                return Ok(SelectorAction::Selected(selected));
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            self.next();
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            self.previous();
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn next(&mut self) {
        if self.worktrees.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.worktrees.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        if self.worktrees.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.worktrees.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),   // Title
                Constraint::Min(10),     // List
                Constraint::Length(3),   // Input or help
            ])
            .split(area);

        // Title
        let title = Paragraph::new("📂 Select Worktree")
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(title, chunks[0]);

        // Worktree list
        if self.worktrees.is_empty() {
            let empty = Paragraph::new("No existing worktrees found")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL));
            frame.render_widget(empty, chunks[1]);
        } else {
            let items: Vec<ListItem> = self.worktrees
                .iter()
                .map(|wt| {
                    let session_type = detect_session_type(&wt.branch);
                    let type_badge = format!("[{}]", session_type.display_name());
                    
                    let content = vec![
                        Line::from(vec![
                            Span::styled(&wt.branch, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                            Span::raw(" "),
                            Span::styled(type_badge, Style::default().fg(Color::Yellow)),
                        ]),
                        Line::from(Span::styled(
                            format!("  {}", wt.path.display()),
                            Style::default().fg(Color::DarkGray)
                        )),
                    ];
                    
                    ListItem::new(content)
                })
                .collect();

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL))
                .highlight_style(Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD))
                .highlight_symbol("→ ");

            frame.render_stateful_widget(list, chunks[1], &mut self.state);
        }

        // Input or help
        if self.input_mode {
            let input = Paragraph::new(format!("New worktree name: {}", self.input_buffer))
                .style(Style::default().fg(Color::Green))
                .block(Block::default().borders(Borders::ALL).title("Create New"));
            frame.render_widget(input, chunks[2]);
        } else {
            let help = Paragraph::new("↑↓/jk: Navigate | Enter: Select | n: New | q: Cancel")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL));
            frame.render_widget(help, chunks[2]);
        }
    }
}

fn detect_session_type(branch: &str) -> SessionType {
    if branch.starts_with("feature/") || branch.starts_with("feat/") {
        SessionType::Feature
    } else if branch.starts_with("fix/") || branch.starts_with("hotfix/") {
        SessionType::Hotfix
    } else if branch.starts_with("refactor/") {
        SessionType::Refactor
    } else {
        SessionType::Development
    }
}
