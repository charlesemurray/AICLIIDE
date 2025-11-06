# TUI Architecture - The Real Solution
## This WILL Work

## Core Principle

**One TUI application manages multiple session workers. The TUI owns the terminal, workers run headless.**

```
┌─────────────────────────────────────────────────────────┐
│                  Ratatui TUI App                        │
│  - Owns real terminal (stdin/stdout)                    │
│  - Renders all sessions                                 │
│  - Routes input to active session                       │
│  - Non-blocking event loop                              │
└─────────────────────────────────────────────────────────┘
           │ mpsc channels
           ▼
    ┌──────────────────────────────────────────┐
    │         Session Workers (tokio tasks)     │
    │  ┌────────┐  ┌────────┐  ┌────────┐     │
    │  │Worker 1│  │Worker 2│  │Worker 3│     │
    │  │        │  │        │  │        │     │
    │  │ LLM    │  │ LLM    │  │ LLM    │     │
    │  │ Tools  │  │ Tools  │  │ Tools  │     │
    │  │ State  │  │ State  │  │ State  │     │
    │  └────────┘  └────────┘  └────────┘     │
    └──────────────────────────────────────────┘
```

## Why This Works

1. **TUI owns terminal**: No rustyline, no blocking readline
2. **Workers are headless**: No terminal access needed
3. **Channel communication**: Workers send/receive messages
4. **Non-blocking**: TUI event loop can switch anytime
5. **True background**: Workers continue when not active

## Implementation

### 1. Session Worker (Headless)

```rust
use tokio::sync::mpsc;

pub struct SessionWorker {
    session_id: String,
    conversation: ConversationState,
    os: Os,
    
    // Communication
    input_rx: mpsc::UnboundedReceiver<WorkerInput>,
    output_tx: mpsc::UnboundedSender<WorkerOutput>,
    
    // State
    state: WorkerState,
}

pub enum WorkerInput {
    UserMessage(String),
    ToolApproval { tool_id: String, approved: bool },
    Pause,
    Resume,
    Shutdown,
}

pub enum WorkerOutput {
    AssistantText(String),
    ToolUse { id: String, name: String, params: serde_json::Value },
    ToolResult { id: String, result: String },
    Error(String),
    StateChange(WorkerState),
}

#[derive(Debug, Clone)]
pub enum WorkerState {
    Idle,
    Processing,
    WaitingForToolApproval,
    Error(String),
}

impl SessionWorker {
    pub async fn run(mut self) {
        loop {
            tokio::select! {
                // Handle input from TUI
                Some(input) = self.input_rx.recv() => {
                    match input {
                        WorkerInput::UserMessage(msg) => {
                            self.process_message(msg).await;
                        }
                        WorkerInput::ToolApproval { tool_id, approved } => {
                            self.handle_tool_approval(tool_id, approved).await;
                        }
                        WorkerInput::Shutdown => break,
                        _ => {}
                    }
                }
                
                // Process any pending work
                _ = self.process_pending() => {}
            }
        }
    }
    
    async fn process_message(&mut self, message: String) {
        self.state = WorkerState::Processing;
        let _ = self.output_tx.send(WorkerOutput::StateChange(self.state.clone()));
        
        // Set user message
        self.conversation.set_next_user_message(message);
        
        // Send to LLM
        match self.conversation.send_message(&mut self.os).await {
            Ok(response) => {
                // Stream response chunks
                for chunk in response.chunks {
                    let _ = self.output_tx.send(WorkerOutput::AssistantText(chunk));
                }
                
                // Handle tool uses
                if let Some(tool_uses) = response.tool_uses {
                    for tool_use in tool_uses {
                        self.state = WorkerState::WaitingForToolApproval;
                        let _ = self.output_tx.send(WorkerOutput::ToolUse {
                            id: tool_use.id.clone(),
                            name: tool_use.name.clone(),
                            params: tool_use.params.clone(),
                        });
                    }
                }
                
                self.state = WorkerState::Idle;
                let _ = self.output_tx.send(WorkerOutput::StateChange(self.state.clone()));
            }
            Err(e) => {
                self.state = WorkerState::Error(e.to_string());
                let _ = self.output_tx.send(WorkerOutput::Error(e.to_string()));
            }
        }
    }
}
```

### 2. TUI Application

```rust
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};

pub struct SessionTUI {
    sessions: HashMap<String, SessionState>,
    active_session_id: Option<String>,
    coordinator: Arc<Mutex<MultiSessionCoordinator>>,
}

struct SessionState {
    name: String,
    worker_input_tx: mpsc::UnboundedSender<WorkerInput>,
    worker_output_rx: mpsc::UnboundedReceiver<WorkerOutput>,
    
    // Display state
    output_buffer: Vec<String>,
    input_buffer: String,
    cursor_pos: usize,
    worker_state: WorkerState,
}

impl SessionTUI {
    pub async fn run(&mut self) -> Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        stdout.execute(EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        
        loop {
            // Update session outputs
            self.update_all_sessions().await;
            
            // Render
            terminal.draw(|f| self.render(f))?;
            
            // Handle input (non-blocking)
            if event::poll(std::time::Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    if !self.handle_key(key).await? {
                        break;
                    }
                }
            }
        }
        
        // Cleanup
        disable_raw_mode()?;
        terminal.backend_mut().execute(LeaveAlternateScreen)?;
        
        Ok(())
    }
    
    async fn update_all_sessions(&mut self) {
        for (id, session) in &mut self.sessions {
            // Non-blocking receive
            while let Ok(output) = session.worker_output_rx.try_recv() {
                match output {
                    WorkerOutput::AssistantText(text) => {
                        session.output_buffer.push(text);
                    }
                    WorkerOutput::ToolUse { name, .. } => {
                        session.output_buffer.push(format!("🔧 Tool: {}", name));
                    }
                    WorkerOutput::StateChange(state) => {
                        session.worker_state = state;
                    }
                    WorkerOutput::Error(err) => {
                        session.output_buffer.push(format!("❌ Error: {}", err));
                    }
                    _ => {}
                }
            }
        }
    }
    
    fn render<B: Backend>(&self, frame: &mut Frame<B>) {
        let size = frame.size();
        
        // Split screen: session list (left) + active session (right)
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(80),
            ])
            .split(size);
        
        // Render session list
        self.render_session_list(frame, chunks[0]);
        
        // Render active session
        if let Some(active_id) = &self.active_session_id {
            if let Some(session) = self.sessions.get(active_id) {
                self.render_session(frame, chunks[1], session);
            }
        }
    }
    
    fn render_session_list<B: Backend>(&self, frame: &mut Frame<B>, area: Rect) {
        let items: Vec<ListItem> = self.sessions
            .iter()
            .map(|(id, session)| {
                let marker = if Some(id) == self.active_session_id.as_ref() {
                    "● "
                } else {
                    "○ "
                };
                let state_icon = match session.worker_state {
                    WorkerState::Processing => "⚙️ ",
                    WorkerState::WaitingForToolApproval => "⏸️ ",
                    WorkerState::Error(_) => "❌ ",
                    _ => "",
                };
                ListItem::new(format!("{}{}{}", marker, state_icon, session.name))
            })
            .collect();
        
        let list = List::new(items)
            .block(Block::default().title("Sessions").borders(Borders::ALL));
        
        frame.render_widget(list, area);
    }
    
    fn render_session<B: Backend>(&self, frame: &mut Frame<B>, area: Rect, session: &SessionState) {
        // Split: output (top) + input (bottom)
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),
                Constraint::Length(3),
            ])
            .split(area);
        
        // Output
        let output_text = session.output_buffer.join("\n");
        let output = Paragraph::new(output_text)
            .block(Block::default().title(session.name.clone()).borders(Borders::ALL))
            .scroll((session.output_buffer.len().saturating_sub(chunks[0].height as usize) as u16, 0));
        frame.render_widget(output, chunks[0]);
        
        // Input
        let input = Paragraph::new(session.input_buffer.as_str())
            .block(Block::default().title("Input").borders(Borders::ALL));
        frame.render_widget(input, chunks[1]);
    }
    
    async fn handle_key(&mut self, key: KeyEvent) -> Result<bool> {
        match (key.code, key.modifiers) {
            // Ctrl+C: Quit
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                return Ok(false);
            }
            
            // Ctrl+N: Next session
            (KeyCode::Char('n'), KeyModifiers::CONTROL) => {
                self.switch_next_session().await;
            }
            
            // Ctrl+P: Previous session
            (KeyCode::Char('p'), KeyModifiers::CONTROL) => {
                self.switch_prev_session().await;
            }
            
            // Enter: Send message
            (KeyCode::Enter, _) => {
                self.send_current_input().await;
            }
            
            // Backspace
            (KeyCode::Backspace, _) => {
                if let Some(active_id) = &self.active_session_id {
                    if let Some(session) = self.sessions.get_mut(active_id) {
                        session.input_buffer.pop();
                    }
                }
            }
            
            // Regular character
            (KeyCode::Char(c), _) => {
                if let Some(active_id) = &self.active_session_id {
                    if let Some(session) = self.sessions.get_mut(active_id) {
                        session.input_buffer.push(c);
                    }
                }
            }
            
            _ => {}
        }
        
        Ok(true)
    }
    
    async fn send_current_input(&mut self) {
        if let Some(active_id) = &self.active_session_id {
            if let Some(session) = self.sessions.get_mut(active_id) {
                let input = std::mem::take(&mut session.input_buffer);
                if !input.is_empty() {
                    let _ = session.worker_input_tx.send(WorkerInput::UserMessage(input));
                }
            }
        }
    }
    
    async fn switch_next_session(&mut self) {
        // Get session IDs in order
        let coord = self.coordinator.lock().await;
        let state = coord.state.lock().await;
        let ids = &state.session_order;
        
        if let Some(current) = &self.active_session_id {
            if let Some(pos) = ids.iter().position(|id| id == current) {
                let next_pos = (pos + 1) % ids.len();
                self.active_session_id = Some(ids[next_pos].clone());
            }
        }
    }
}
```

### 3. Integration with Coordinator

```rust
impl MultiSessionCoordinator {
    pub async fn run_tui(
        coord_arc: Arc<Mutex<Self>>,
        os: &mut Os,
    ) -> Result<()> {
        let mut tui = SessionTUI::new(coord_arc.clone());
        
        // Create initial session
        let coord = coord_arc.lock().await;
        let session_ids: Vec<_> = {
            let state = coord.state.lock().await;
            state.sessions.keys().cloned().collect()
        };
        drop(coord);
        
        // Spawn worker for each session
        for session_id in session_ids {
            let (input_tx, input_rx) = mpsc::unbounded_channel();
            let (output_tx, output_rx) = mpsc::unbounded_channel();
            
            // Get session data
            let coord = coord_arc.lock().await;
            let state = coord.state.lock().await;
            let session = state.sessions.get(&session_id).unwrap();
            
            let worker = SessionWorker {
                session_id: session_id.clone(),
                conversation: session.conversation.clone(),
                os: os.clone(),
                input_rx,
                output_tx,
                state: WorkerState::Idle,
            };
            
            // Spawn worker
            tokio::spawn(async move {
                worker.run().await;
            });
            
            // Add to TUI
            tui.sessions.insert(session_id.clone(), SessionState {
                name: session.display.name.clone(),
                worker_input_tx: input_tx,
                worker_output_rx: output_rx,
                output_buffer: Vec::new(),
                input_buffer: String::new(),
                cursor_pos: 0,
                worker_state: WorkerState::Idle,
            });
        }
        
        // Set first session as active
        if let Some(first_id) = tui.sessions.keys().next() {
            tui.active_session_id = Some(first_id.clone());
        }
        
        // Run TUI
        tui.run().await?;
        
        Ok(())
    }
}
```

## What This Achieves

### ✅ Everything You Want
1. **Switch anytime**: TUI event loop is non-blocking
2. **Background processing**: Workers continue when not active
3. **Multiple sessions**: All running simultaneously
4. **See all sessions**: Visual list with state indicators
5. **No blocking**: No rustyline, no readline()

### ✅ Better UX
1. **Visual feedback**: See which sessions are processing
2. **Fast switching**: Ctrl+N/Ctrl+P
3. **Split screen**: Session list + active session
4. **State indicators**: ⚙️ processing, ⏸️ waiting, ❌ error

## Implementation Timeline

### Week 1: Foundation
- [ ] SessionWorker implementation
- [ ] Channel communication
- [ ] Basic TUI layout
- [ ] Single session working

### Week 2: Multi-Session
- [ ] Multiple workers
- [ ] Session switching
- [ ] Output buffering
- [ ] State synchronization

### Week 3: Features
- [ ] Tool approval UI
- [ ] History display
- [ ] Error handling
- [ ] Session creation/deletion

### Week 4: Polish
- [ ] Keyboard shortcuts
- [ ] Visual improvements
- [ ] Testing
- [ ] Documentation

**Total: 4 weeks**

## This WILL Work Because

1. **No rustyline**: TUI handles input directly
2. **No blocking**: Event loop with `poll()`
3. **Workers are headless**: No terminal access
4. **Channels work**: Proven pattern
5. **Ratatui is mature**: Used in production apps

## Commitment

This is a **major rewrite** but it's the **right architecture**. It will:
- ✅ Work as designed
- ✅ Enable all requested features
- ✅ Provide better UX
- ✅ Be maintainable

**I'm confident this will work. Let's build it.**
