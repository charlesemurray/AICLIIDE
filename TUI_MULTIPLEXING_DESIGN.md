# TUI-Based Session Multiplexing Design
## Using Ratatui + Virtual Terminals

## Core Concept

Use **ratatui** to create a TUI that manages multiple virtual terminal buffers, each with its own rustyline instance reading from a virtual stdin/stdout.

```
┌─────────────────────────────────────────────────────────────┐
│                    Ratatui TUI Layer                        │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │  Session 1  │  │  Session 2  │  │  Session 3  │        │
│  │   (Active)  │  │ (Background)│  │ (Background)│        │
│  │             │  │             │  │             │        │
│  │  Virtual    │  │  Virtual    │  │  Virtual    │        │
│  │  Terminal   │  │  Terminal   │  │  Terminal   │        │
│  └─────────────┘  └─────────────┘  └─────────────┘        │
│         ↓                ↓                ↓                 │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │  Rustyline  │  │  Rustyline  │  │  Rustyline  │        │
│  │  Instance   │  │  Instance   │  │  Instance   │        │
│  └─────────────┘  └─────────────┘  └─────────────┘        │
│         ↓                ↓                ↓                 │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │   Virtual   │  │   Virtual   │  │   Virtual   │        │
│  │   Stdin     │  │   Stdin     │  │   Stdin     │        │
│  └─────────────┘  └─────────────┘  └─────────────┘        │
│         ↑                                                   │
│         └───────── Real Stdin (multiplexed) ───────────────┘
└─────────────────────────────────────────────────────────────┘
```

## Architecture Components

### 1. Virtual Terminal

```rust
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};

/// Virtual stdin that can be written to programmatically
pub struct VirtualStdin {
    buffer: Arc<Mutex<VecDeque<u8>>>,
    waker: Arc<Mutex<Option<std::task::Waker>>>,
}

impl VirtualStdin {
    pub fn new() -> Self {
        Self {
            buffer: Arc::new(Mutex::new(VecDeque::new())),
            waker: Arc::new(Mutex::new(None)),
        }
    }
    
    /// Write data to virtual stdin (called by TUI)
    pub fn write_input(&self, data: &[u8]) {
        let mut buffer = self.buffer.lock().unwrap();
        buffer.extend(data);
        
        // Wake up any waiting reader
        if let Some(waker) = self.waker.lock().unwrap().take() {
            waker.wake();
        }
    }
    
    /// Get a handle for rustyline to read from
    pub fn reader(&self) -> VirtualStdinReader {
        VirtualStdinReader {
            buffer: self.buffer.clone(),
            waker: self.waker.clone(),
        }
    }
}

pub struct VirtualStdinReader {
    buffer: Arc<Mutex<VecDeque<u8>>>,
    waker: Arc<Mutex<Option<std::task::Waker>>>,
}

impl Read for VirtualStdinReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut buffer = self.buffer.lock().unwrap();
        
        if buffer.is_empty() {
            // Would block - in real impl, use async or blocking wait
            return Err(std::io::Error::new(
                std::io::ErrorKind::WouldBlock,
                "No data available"
            ));
        }
        
        let len = buf.len().min(buffer.len());
        for i in 0..len {
            buf[i] = buffer.pop_front().unwrap();
        }
        Ok(len)
    }
}

/// Virtual stdout that captures output
pub struct VirtualStdout {
    buffer: Arc<Mutex<Vec<u8>>>,
}

impl VirtualStdout {
    pub fn new() -> Self {
        Self {
            buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// Get captured output (called by TUI)
    pub fn read_output(&self) -> Vec<u8> {
        let mut buffer = self.buffer.lock().unwrap();
        std::mem::take(&mut *buffer)
    }
    
    /// Get a handle for rustyline to write to
    pub fn writer(&self) -> VirtualStdoutWriter {
        VirtualStdoutWriter {
            buffer: self.buffer.clone(),
        }
    }
}

pub struct VirtualStdoutWriter {
    buffer: Arc<Mutex<Vec<u8>>>,
}

impl Write for VirtualStdoutWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut buffer = self.buffer.lock().unwrap();
        buffer.extend_from_slice(buf);
        Ok(buf.len())
    }
    
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
```

### 2. Session Terminal Widget

```rust
use ratatui::{
    backend::Backend,
    layout::Rect,
    widgets::{Block, Borders, Paragraph, Widget},
    Frame,
};

pub struct SessionTerminal {
    session_id: String,
    session_name: String,
    is_active: bool,
    
    // Virtual I/O
    virtual_stdin: VirtualStdin,
    virtual_stdout: VirtualStdout,
    
    // Display buffer
    output_lines: Vec<String>,
    input_line: String,
    cursor_pos: usize,
}

impl SessionTerminal {
    pub fn new(session_id: String, session_name: String) -> Self {
        Self {
            session_id,
            session_name,
            is_active: false,
            virtual_stdin: VirtualStdin::new(),
            virtual_stdout: VirtualStdout::new(),
            output_lines: Vec::new(),
            input_line: String::new(),
            cursor_pos: 0,
        }
    }
    
    /// Update display from virtual stdout
    pub fn update_output(&mut self) {
        let output = self.virtual_stdout.read_output();
        if !output.is_empty() {
            let text = String::from_utf8_lossy(&output);
            for line in text.lines() {
                self.output_lines.push(line.to_string());
            }
            
            // Keep last N lines
            if self.output_lines.len() > 1000 {
                self.output_lines.drain(0..self.output_lines.len() - 1000);
            }
        }
    }
    
    /// Send input to virtual stdin
    pub fn send_input(&mut self, input: &str) {
        self.virtual_stdin.write_input(input.as_bytes());
        self.virtual_stdin.write_input(b"\n");
    }
    
    /// Render terminal widget
    pub fn render<B: Backend>(&self, frame: &mut Frame<B>, area: Rect) {
        let block = Block::default()
            .title(format!(" {} {} ", 
                if self.is_active { "●" } else { "○" },
                self.session_name
            ))
            .borders(Borders::ALL);
        
        // Show last N lines that fit in area
        let visible_lines = (area.height as usize).saturating_sub(3);
        let start = self.output_lines.len().saturating_sub(visible_lines);
        let visible = &self.output_lines[start..];
        
        let text = visible.join("\n");
        let paragraph = Paragraph::new(text)
            .block(block);
        
        frame.render_widget(paragraph, area);
        
        // Show input line at bottom if active
        if self.is_active {
            // Render input line with cursor
        }
    }
}
```

### 3. TUI Application

```rust
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};

pub struct SessionTUI {
    sessions: HashMap<String, SessionTerminal>,
    active_session_id: Option<String>,
    coordinator: Arc<Mutex<MultiSessionCoordinator>>,
}

impl SessionTUI {
    pub fn new(coordinator: Arc<Mutex<MultiSessionCoordinator>>) -> Self {
        Self {
            sessions: HashMap::new(),
            active_session_id: None,
            coordinator,
        }
    }
    
    pub async fn run(&mut self) -> Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;
        
        loop {
            // Update all session outputs
            for session in self.sessions.values_mut() {
                session.update_output();
            }
            
            // Render
            terminal.draw(|f| self.render(f))?;
            
            // Handle input
            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if !self.handle_key(key).await? {
                        break;
                    }
                }
            }
            
            // Check for quit
            let coord = self.coordinator.lock().await;
            let state = coord.state.lock().await;
            if state.should_quit {
                break;
            }
        }
        
        // Cleanup
        disable_raw_mode()?;
        terminal.show_cursor()?;
        
        Ok(())
    }
    
    fn render<B: Backend>(&self, frame: &mut Frame<B>) {
        let size = frame.size();
        
        // Split screen for multiple sessions
        // Active session gets more space
        
        if let Some(active_id) = &self.active_session_id {
            if let Some(session) = self.sessions.get(active_id) {
                session.render(frame, size);
            }
        }
    }
    
    async fn handle_key(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            // Ctrl+N: Next session
            KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.switch_next_session().await?;
            }
            
            // Ctrl+P: Previous session
            KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.switch_prev_session().await?;
            }
            
            // Ctrl+Q: Quit
            KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                let coord = self.coordinator.lock().await;
                coord.quit().await;
                return Ok(false);
            }
            
            // Regular input - send to active session
            KeyCode::Char(c) => {
                if let Some(active_id) = &self.active_session_id {
                    if let Some(session) = self.sessions.get_mut(active_id) {
                        session.input_line.push(c);
                    }
                }
            }
            
            KeyCode::Enter => {
                if let Some(active_id) = &self.active_session_id {
                    if let Some(session) = self.sessions.get_mut(active_id) {
                        let input = std::mem::take(&mut session.input_line);
                        session.send_input(&input);
                    }
                }
            }
            
            _ => {}
        }
        
        Ok(true)
    }
    
    async fn switch_next_session(&mut self) -> Result<()> {
        // Switch to next session in coordinator
        let coord = self.coordinator.lock().await;
        let mut state = coord.state.lock().await;
        
        // Get next session ID
        if let Some(current) = &state.active_session_id {
            let ids: Vec<_> = state.session_order.iter().collect();
            if let Some(pos) = ids.iter().position(|id| *id == current) {
                let next_pos = (pos + 1) % ids.len();
                state.active_session_id = Some(ids[next_pos].clone());
                self.active_session_id = Some(ids[next_pos].clone());
            }
        }
        
        Ok(())
    }
}
```

### 4. Modified InputSource for Virtual I/O

```rust
impl InputSource {
    pub fn new_virtual(
        virtual_stdin: VirtualStdinReader,
        virtual_stdout: VirtualStdoutWriter,
    ) -> Result<Self> {
        // Create rustyline with virtual I/O
        // This requires modifying rustyline or using a wrapper
        
        // Pseudocode:
        let config = rustyline::Config::builder()
            .auto_add_history(true)
            .build();
        
        let mut rl = rustyline::Editor::with_config(config)?;
        
        // Set custom I/O (if rustyline supports it)
        // Otherwise, we need to use a different approach
        
        Ok(Self {
            inner: inner::Inner::Readline(rl),
            paste_state: PasteState::new(),
        })
    }
}
```

## Key Advantages

### ✅ Solves All Problems
1. **Multiple Sessions**: Each has virtual terminal
2. **Switch Anytime**: TUI handles real stdin, routes to active session
3. **Background Processing**: Sessions continue in background
4. **Visual Feedback**: See all sessions at once
5. **No Blocking**: TUI event loop is non-blocking

### ✅ Better UX
1. **Split screen**: See multiple sessions
2. **Visual indicators**: Active session highlighted
3. **Session list**: Quick overview
4. **Keyboard shortcuts**: Fast switching

## Implementation Challenges

### Challenge 1: Rustyline Virtual I/O
**Problem**: Rustyline expects real stdin/stdout

**Solutions**:
1. **Fork rustyline**: Add virtual I/O support
2. **Use alternative**: Replace with custom line editor
3. **Wrapper**: Intercept rustyline I/O calls

**Recommended**: Use `tui-textarea` crate instead of rustyline
```rust
use tui_textarea::TextArea;

// Built for ratatui, handles input natively
let mut textarea = TextArea::default();
```

### Challenge 2: Session Workers
**Problem**: Sessions need to run in background

**Solution**: Spawn each session in tokio task
```rust
for session in sessions {
    tokio::spawn(async move {
        session.run_worker().await
    });
}
```

### Challenge 3: Output Synchronization
**Problem**: Multiple sessions writing to virtual stdout

**Solution**: Each session has own virtual stdout, TUI polls them
```rust
for session in sessions.values_mut() {
    session.update_output(); // Reads from virtual stdout
}
```

## Implementation Plan

### Phase 1: TUI Foundation (2-3 days)
- [ ] Setup ratatui
- [ ] Create SessionTerminal widget
- [ ] Implement basic rendering
- [ ] Test with single session

### Phase 2: Virtual I/O (2-3 days)
- [ ] Implement VirtualStdin/Stdout
- [ ] Replace rustyline with tui-textarea
- [ ] Test input/output routing
- [ ] Verify line editing works

### Phase 3: Multi-Session (2-3 days)
- [ ] Spawn sessions as background tasks
- [ ] Implement session switching
- [ ] Add keyboard shortcuts
- [ ] Test with multiple sessions

### Phase 4: Polish (1-2 days)
- [ ] Split screen layout
- [ ] Session indicators
- [ ] Output buffering
- [ ] Error handling

**Total**: 7-11 days

## Dependencies

```toml
[dependencies]
ratatui = "0.26"
crossterm = "0.27"
tui-textarea = "0.4"  # Replaces rustyline for TUI
tokio = { version = "1", features = ["full"] }
```

## Benefits Over Previous Designs

1. **Actually Feasible**: Works with real constraints
2. **Better UX**: Visual interface, not just text
3. **True Background**: Sessions run independently
4. **Switch Anytime**: Not blocked by readline
5. **Scalable**: Can show many sessions at once

## Conclusion

This design is **FEASIBLE** and **SUPERIOR** to previous approaches:
- ✅ Respects rustyline/crossterm constraints
- ✅ Enables true background execution
- ✅ Provides better user experience
- ✅ Allows switching anytime
- ✅ Reasonable implementation timeline

**Recommendation**: Proceed with TUI-based approach.
