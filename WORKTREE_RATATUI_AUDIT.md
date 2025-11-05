# Worktree Ratatui UI Audit & Recommendations

## Executive Summary

Ratatui is currently used minimally (session indicators only). Worktrees use basic terminal output. **Opportunity**: Create rich TUI interfaces for worktree management with interactive selection, real-time status, and visual workflows.

---

## Current Ratatui Usage

### What's Implemented

**Location**: `crates/chat-cli/src/cli/chat/indicator.rs`

```rust
pub struct SessionIndicator {
    // Renders a small indicator box in top-right corner
    // Shows waiting sessions with borders and colors
}
```

**Current Features**:
- ✅ Bordered widgets
- ✅ Color styling
- ✅ Layout management (Rect positioning)
- ✅ Paragraph rendering
- ✅ Terminal drawing

**What's NOT Used**:
- ❌ Lists with selection
- ❌ Tables
- ❌ Progress bars
- ❌ Interactive navigation
- ❌ Full-screen TUIs
- ❌ Tabs/panels

---

## Worktree UI Opportunities

### 1. Interactive Worktree Selector (HIGH IMPACT)

#### Current (Text-based)
```
📂 Existing worktrees:
  1. feature-auth (/repo/.worktrees/feature-auth)
  2. fix-login (/repo/.worktrees/fix-login)

Create or select worktree [number/name/auto/N]:
```

#### Proposed (Ratatui TUI)
```rust
use ratatui::{
    widgets::{List, ListItem, ListState, Block, Borders},
    style::{Style, Color, Modifier},
    layout::{Layout, Constraint, Direction},
};

pub struct WorktreeSelector {
    worktrees: Vec<WorktreeInfo>,
    state: ListState,
}

impl WorktreeSelector {
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self.worktrees
            .iter()
            .enumerate()
            .map(|(i, wt)| {
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
            .block(Block::default()
                .title("📂 Select Worktree")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)))
            .highlight_style(Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD))
            .highlight_symbol("→ ");

        frame.render_stateful_widget(list, area, &mut self.state);
        
        // Help text at bottom
        let help = Paragraph::new("↑↓: Navigate | Enter: Select | n: New | q: Cancel")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(help, help_area);
    }
}
```

**Visual Output**:
```
┌─ 📂 Select Worktree ────────────────────────────────────┐
│                                                          │
│ → feature-auth [Feature]                                │
│     /repo/.worktrees/feature-auth                       │
│                                                          │
│   fix-login [Hotfix]                                    │
│     /repo/.worktrees/fix-login                          │
│                                                          │
│   refactor-api [Refactor]                               │
│     /repo/.worktrees/refactor-api                       │
│                                                          │
└──────────────────────────────────────────────────────────┘
 ↑↓: Navigate | Enter: Select | n: New | q: Cancel
```

**Benefits**:
- ✅ Visual navigation with arrow keys
- ✅ Highlighted selection
- ✅ Multi-line items with metadata
- ✅ Clear keyboard shortcuts
- ✅ Professional appearance

---

### 2. Worktree Dashboard (MEDIUM IMPACT)

#### Proposed: Full-Screen TUI
```rust
pub struct WorktreeDashboard {
    sessions: Vec<SessionMetadata>,
    selected: usize,
}

impl WorktreeDashboard {
    pub fn render(&mut self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),      // Header
                Constraint::Min(10),        // Session list
                Constraint::Length(5),      // Details panel
                Constraint::Length(3),      // Actions
            ])
            .split(frame.area());

        // Header
        let header = Paragraph::new("🌳 Worktree Sessions")
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(header, chunks[0]);

        // Session list with status
        let items: Vec<ListItem> = self.sessions.iter().map(|s| {
            let status_icon = match s.status {
                SessionStatus::Active => "●",
                SessionStatus::Completed => "✓",
                SessionStatus::Archived => "□",
                _ => "○",
            };
            
            let wt = s.worktree_info.as_ref().unwrap();
            let content = vec![
                Line::from(vec![
                    Span::styled(status_icon, Style::default().fg(Color::Green)),
                    Span::raw(" "),
                    Span::styled(&wt.branch, Style::default().fg(Color::Cyan)),
                    Span::raw(" "),
                    Span::styled(
                        format!("[{}]", s.session_type.display_name()),
                        Style::default().fg(Color::Yellow)
                    ),
                ]),
                Line::from(Span::styled(
                    format!("  {} messages | {}", s.message_count, wt.path.display()),
                    Style::default().fg(Color::DarkGray)
                )),
            ];
            ListItem::new(content)
        }).collect();

        let list = List::new(items)
            .block(Block::default().title("Sessions").borders(Borders::ALL))
            .highlight_style(Style::default().bg(Color::DarkGray));
        frame.render_widget(list, chunks[1]);

        // Details panel for selected session
        if let Some(session) = self.sessions.get(self.selected) {
            let details = self.render_details(session);
            frame.render_widget(details, chunks[2]);
        }

        // Actions
        let actions = Paragraph::new("m: Merge | c: Complete | d: Delete | q: Quit")
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(actions, chunks[3]);
    }
}
```

**Visual Output**:
```
┌─ 🌳 Worktree Sessions ──────────────────────────────────┐
│                                                          │
└──────────────────────────────────────────────────────────┘
┌─ Sessions ───────────────────────────────────────────────┐
│                                                          │
│ → ● feature-auth [Feature]                              │
│     12 messages | /repo/.worktrees/feature-auth         │
│                                                          │
│   ✓ fix-login [Hotfix]                                  │
│     5 messages | /repo/.worktrees/fix-login             │
│                                                          │
│   □ refactor-api [Refactor]                             │
│     23 messages | /repo/.worktrees/refactor-api         │
│                                                          │
└──────────────────────────────────────────────────────────┘
┌─ Details ────────────────────────────────────────────────┐
│ Branch: feature-auth → main                             │
│ Created: 2 hours ago                                     │
│ Status: Active                                           │
└──────────────────────────────────────────────────────────┘
┌──────────────────────────────────────────────────────────┐
│ m: Merge | c: Complete | d: Delete | q: Quit            │
└──────────────────────────────────────────────────────────┘
```

**Benefits**:
- ✅ All sessions visible at once
- ✅ Real-time status updates
- ✅ Detailed view of selected session
- ✅ Quick actions with keyboard
- ✅ Professional dashboard feel

---

### 3. Merge Progress Visualization (HIGH IMPACT)

#### Current (Text-based)
```
🔀 Preparing to merge worktree session...
Merging feature-auth into main...
✓ Merge successful!
✓ Cleaned up worktree and branch
```

#### Proposed (Ratatui with Progress)
```rust
use ratatui::widgets::{Gauge, LineGauge};

pub struct MergeProgress {
    steps: Vec<MergeStep>,
    current_step: usize,
}

enum MergeStep {
    DetectingConflicts,
    CheckingOut,
    Merging,
    CleaningUp,
    Complete,
}

impl MergeProgress {
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),   // Title
                Constraint::Length(3),   // Progress bar
                Constraint::Min(5),      // Steps
                Constraint::Length(3),   // Current action
            ])
            .split(area);

        // Title
        let title = Paragraph::new("🔀 Merging Worktree")
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(title, chunks[0]);

        // Progress bar
        let progress = (self.current_step as f64 / self.steps.len() as f64) * 100.0;
        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL))
            .gauge_style(Style::default().fg(Color::Cyan))
            .percent(progress as u16)
            .label(format!("{:.0}%", progress));
        frame.render_widget(gauge, chunks[1]);

        // Steps checklist
        let steps_text: Vec<Line> = self.steps.iter().enumerate().map(|(i, step)| {
            let (icon, style) = if i < self.current_step {
                ("✓", Style::default().fg(Color::Green))
            } else if i == self.current_step {
                ("→", Style::default().fg(Color::Yellow))
            } else {
                ("○", Style::default().fg(Color::DarkGray))
            };
            
            Line::from(vec![
                Span::styled(icon, style),
                Span::raw(" "),
                Span::styled(step.name(), style),
            ])
        }).collect();

        let steps_widget = Paragraph::new(steps_text)
            .block(Block::default().title("Steps").borders(Borders::ALL));
        frame.render_widget(steps_widget, chunks[2]);

        // Current action
        let action = Paragraph::new(self.steps[self.current_step].description())
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(action, chunks[3]);
    }
}
```

**Visual Output**:
```
┌─ 🔀 Merging Worktree ───────────────────────────────────┐
│                                                          │
└──────────────────────────────────────────────────────────┘
┌──────────────────────────────────────────────────────────┐
│ ████████████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  60%  │
└──────────────────────────────────────────────────────────┘
┌─ Steps ──────────────────────────────────────────────────┐
│ ✓ Detecting conflicts                                    │
│ ✓ Checking out target branch                            │
│ → Merging branches                                       │
│ ○ Cleaning up worktree                                  │
│ ○ Complete                                               │
└──────────────────────────────────────────────────────────┘
┌──────────────────────────────────────────────────────────┐
│ Merging feature-auth into main...                       │
└──────────────────────────────────────────────────────────┘
```

**Benefits**:
- ✅ Visual progress indication
- ✅ Clear step-by-step process
- ✅ Real-time status updates
- ✅ Reduces perceived wait time
- ✅ Professional UX

---

### 4. Conflict Resolution TUI (CRITICAL IMPACT)

#### Current (Text-based)
```
⚠️  Conflicts detected in 2 file(s):
  • src/auth.rs
  • src/login.rs

📋 Resolution options:
  1. Resolve manually: ...
  2. Force merge: ...
  3. Cancel: ...
```

#### Proposed (Interactive TUI)
```rust
pub struct ConflictResolver {
    conflicts: Vec<String>,
    selected_option: usize,
    options: Vec<ResolutionOption>,
}

impl ConflictResolver {
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),   // Header
                Constraint::Min(5),      // Conflict list
                Constraint::Length(10),  // Options
                Constraint::Length(3),   // Help
            ])
            .split(area);

        // Header with warning
        let header = Paragraph::new("⚠  Merge Conflicts Detected")
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(header, chunks[0]);

        // Conflict file list
        let conflicts: Vec<ListItem> = self.conflicts.iter().map(|file| {
            ListItem::new(Line::from(vec![
                Span::styled("⚠ ", Style::default().fg(Color::Yellow)),
                Span::styled(file, Style::default().fg(Color::Red)),
            ]))
        }).collect();

        let list = List::new(conflicts)
            .block(Block::default().title("Conflicting Files").borders(Borders::ALL));
        frame.render_widget(list, chunks[1]);

        // Resolution options
        let options: Vec<ListItem> = self.options.iter().enumerate().map(|(i, opt)| {
            let style = if i == self.selected_option {
                Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            
            let content = vec![
                Line::from(Span::styled(opt.title(), style.fg(Color::Cyan))),
                Line::from(Span::styled(opt.description(), style.fg(Color::DarkGray))),
            ];
            
            ListItem::new(content).style(style)
        }).collect();

        let options_list = List::new(options)
            .block(Block::default().title("Resolution Options").borders(Borders::ALL))
            .highlight_symbol("→ ");
        frame.render_widget(options_list, chunks[2]);

        // Help
        let help = Paragraph::new("↑↓: Select | Enter: Confirm | q: Cancel")
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(help, chunks[3]);
    }
}
```

**Visual Output**:
```
┌─ ⚠  Merge Conflicts Detected ───────────────────────────┐
│                                                          │
└──────────────────────────────────────────────────────────┘
┌─ Conflicting Files ──────────────────────────────────────┐
│ ⚠ src/auth.rs                                           │
│ ⚠ src/login.rs                                          │
│ ⚠ src/config.rs                                         │
└──────────────────────────────────────────────────────────┘
┌─ Resolution Options ─────────────────────────────────────┐
│ → Resolve Manually                                       │
│     Open editor to fix conflicts                         │
│                                                          │
│   Force Merge                                            │
│     Merge anyway, resolve later                          │
│                                                          │
│   Cancel Merge                                           │
│     Return to worktree                                   │
└──────────────────────────────────────────────────────────┘
┌──────────────────────────────────────────────────────────┐
│ ↑↓: Select | Enter: Confirm | q: Cancel                 │
└──────────────────────────────────────────────────────────┘
```

**Benefits**:
- ✅ Interactive option selection
- ✅ Clear conflict visualization
- ✅ Guided resolution process
- ✅ Reduces user confusion
- ✅ Professional error handling

---

### 5. Session Status Panel (LOW IMPACT, NICE TO HAVE)

#### Proposed: Persistent Status Bar
```rust
pub struct SessionStatusBar {
    current_session: Option<SessionMetadata>,
}

impl SessionStatusBar {
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if let Some(session) = &self.current_session {
            let wt = session.worktree_info.as_ref().unwrap();
            
            let status_text = vec![
                Span::styled("🌳 ", Style::default().fg(Color::Green)),
                Span::styled(&wt.branch, Style::default().fg(Color::Cyan)),
                Span::raw(" | "),
                Span::styled(
                    format!("{} msgs", session.message_count),
                    Style::default().fg(Color::Yellow)
                ),
                Span::raw(" | "),
                Span::styled(
                    format!("{:?}", session.status),
                    Style::default().fg(Color::Green)
                ),
            ];
            
            let paragraph = Paragraph::new(Line::from(status_text))
                .style(Style::default().bg(Color::Black));
            
            frame.render_widget(paragraph, area);
        }
    }
}
```

**Visual Output** (Bottom of terminal):
```
🌳 feature-auth | 12 msgs | Active
```

---

## Implementation Plan

### Phase 1: Interactive Selector (Day 1 - 6 hours)
**Priority**: HIGH

1. Create `WorktreeSelector` widget
2. Add keyboard navigation (↑↓, Enter, q)
3. Integrate with startup prompt
4. Add "create new" option

**Files**:
- `crates/chat-cli/src/cli/chat/worktree_selector.rs`
- Update `mod.rs` to use TUI when available

### Phase 2: Merge Progress (Day 1-2 - 4 hours)
**Priority**: HIGH

1. Create `MergeProgress` widget
2. Add progress bar and step tracking
3. Integrate with merge workflow
4. Add real-time updates

**Files**:
- `crates/chat-cli/src/cli/chat/merge_progress.rs`
- Update `merge_workflow.rs`

### Phase 3: Conflict Resolver (Day 2 - 5 hours)
**Priority**: CRITICAL

1. Create `ConflictResolver` TUI
2. Add interactive option selection
3. Integrate with conflict detection
4. Add file preview (optional)

**Files**:
- `crates/chat-cli/src/cli/chat/conflict_resolver.rs`
- Update conflict handling in `merge_workflow.rs`

### Phase 4: Dashboard (Day 3 - 6 hours)
**Priority**: MEDIUM

1. Create `WorktreeDashboard` full-screen TUI
2. Add session list with navigation
3. Add details panel
4. Add quick actions (m, c, d)

**Files**:
- `crates/chat-cli/src/cli/chat/worktree_dashboard.rs`
- Add `/sessions tui` command

### Phase 5: Status Bar (Day 3 - 2 hours)
**Priority**: LOW

1. Create persistent status bar
2. Update on session changes
3. Add to chat interface

**Total Effort**: 23 hours (3 days)

---

## Quick Wins (Can Do Today - 4 hours)

### 1. Interactive Worktree Selector (3 hours)
Replace text prompt with ratatui List widget

### 2. Merge Progress Bar (1 hour)
Add simple progress gauge to merge

**Impact**: Immediate visual improvement, professional feel

---

## Technical Requirements

### Dependencies (Already Present)
```toml
[dependencies]
ratatui = "0.29.0"
crossterm = "0.28"
tokio = { version = "1", features = ["full"] }
```

### Architecture Pattern
```rust
// Reusable TUI component pattern
pub trait TuiComponent {
    fn render(&mut self, frame: &mut Frame, area: Rect);
    fn handle_event(&mut self, event: Event) -> Action;
}

// Integration with existing code
pub fn show_worktree_selector(worktrees: Vec<WorktreeInfo>) -> Result<Option<usize>> {
    let mut terminal = setup_terminal()?;
    let mut selector = WorktreeSelector::new(worktrees);
    
    loop {
        terminal.draw(|f| selector.render(f, f.area()))?;
        
        if let Event::Key(key) = event::read()? {
            match selector.handle_event(key) {
                Action::Select(idx) => return Ok(Some(idx)),
                Action::Cancel => return Ok(None),
                _ => {}
            }
        }
    }
}
```

---

## Expected Impact

### User Experience
- **Before**: Text-based, requires typing
- **After**: Visual, keyboard navigation, interactive

### Discoverability
- **Before**: Options listed in text
- **After**: Highlighted, navigable options

### Error Handling
- **Before**: Text instructions
- **After**: Interactive guided resolution

### Professional Feel
- **Before**: CLI tool
- **After**: Modern TUI application

---

## Recommendation

**Priority**: HIGH

**Start With**:
1. Interactive Worktree Selector (3 hours) - Immediate impact
2. Merge Progress Visualization (1 hour) - Professional feel

**Then Add**:
3. Conflict Resolver TUI (5 hours) - Critical UX improvement
4. Full Dashboard (6 hours) - Power user feature

**Total Quick Win**: 4 hours for massive UX improvement

---

## Code Example: Before & After

### Before
```rust
eprintln!("\n📂 Existing worktrees:");
for (idx, wt) in worktrees.iter().enumerate() {
    eprintln!("  {}. {} ({})", idx + 1, wt.branch, wt.path.display());
}
eprint!("Create or select worktree [number/name/auto/N]: ");
```

### After
```rust
let selected = show_worktree_selector(worktrees)?;
match selected {
    Some(idx) => use_worktree(&worktrees[idx]),
    None => create_new_worktree(),
}
```

**Result**: Interactive, visual, keyboard-driven selection with professional TUI.
