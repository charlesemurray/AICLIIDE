# Multi-Session Support with TUI Design

## Overview

Add native multi-session support to Q CLI, allowing users to run multiple concurrent chat sessions within a single Q CLI instance. This eliminates the need for multiple terminal windows/tabs and provides better coordination through visual indicators and session management.

## Goals

- Run multiple Q chat sessions concurrently in one Q CLI instance
- Auto-generate descriptive session names from conversation context
- Visual indicator showing which sessions are waiting for user input
- Easy session switching and management
- Cross-platform support (Linux, macOS)
- Works over SSH (no desktop notifications required)

## Non-Goals

- Desktop notifications (not reliable over SSH)
- Windows support (Q CLI doesn't officially support Windows yet)
- Multi-user session sharing

## Existing Session Infrastructure

Q CLI already has session-related components that we can leverage:

**Existing Components:**

1. **`SessionManager`** (`theme/session_manager.rs`)
   - Already manages multiple sessions
   - Tracks session types: Debug, Planning, Development, CodeReview
   - Handles session status: Active, Paused, Completed
   - Provides session switching, pausing, resuming
   - Message counting per session

2. **`SessionDisplay`** (`theme/session.rs`)
   - Session display information with colors
   - Session type prefixes and formatting
   - Colored list entries using crossterm

3. **`ConversationState`** (`cli/chat/conversation.rs`)
   - Already has `conversation_id` for each conversation
   - Manages conversation history and state
   - Persists to database

4. **Session Path Resolution** (`util/session_paths.rs`)
   - Resolves `@session/` prefixed paths
   - Maps to `.amazonq/sessions/{conversation_id}/`

**What's Missing:**

- Integration between `SessionManager` and `ConversationState`
- Multi-conversation concurrency support
- Visual indicator for waiting sessions
- Session name auto-generation
- Commands to manage multiple concurrent sessions
- `WaitingForInput` and `Processing` states

## Architecture

### Existing Session Infrastructure

Q CLI already has session-related components that we can leverage:

**Existing Components:**

1. **`SessionManager`** (`theme/session_manager.rs`)
   - Already manages multiple sessions
   - Tracks session types: Debug, Planning, Development, CodeReview
   - Handles session status: Active, Paused, Completed
   - Provides session switching, pausing, resuming
   - Message counting per session

2. **`SessionDisplay`** (`theme/session.rs`)
   - Session display information with colors
   - Session type prefixes and formatting
   - Colored list entries using crossterm

3. **`ConversationState`** (`cli/chat/conversation.rs`)
   - Already has `conversation_id` for each conversation
   - Manages conversation history and state
   - Persists to database

4. **Session Path Resolution** (`util/session_paths.rs`)
   - Resolves `@session/` prefixed paths
   - Maps to `.amazonq/sessions/{conversation_id}/`

### Phase 1: Extend Existing Session Infrastructure

**What We Need to Add:**

1. **Multi-Session Coordination**
   - Extend `SessionManager` to track multiple concurrent `ConversationState` instances
   - Add `WaitingForInput` and `Processing` states to existing `SessionStatus` enum
   - Link each `SessionDisplay` to its corresponding `ConversationState`

2. **Enhanced Session State**
   ```rust
   // Extend existing SessionStatus enum
   enum SessionStatus {
       Active,           // Currently displayed
       WaitingForInput,  // NEW: Completed response, needs user input
       Processing,       // NEW: Waiting for Q response
       Paused,          // Existing
       Completed,       // Existing
   }
   
   // New struct to link SessionDisplay with ConversationState
   struct ManagedSession {
       display: SessionDisplay,
       conversation: ConversationState,
       conversation_id: String,
   }
   ```

3. **Session Name Generator**
   - Analyze first 2-3 user messages in conversation
   - Extract key topics/actions (e.g., "api", "refactor", "s3", "debug")
   - Generate short kebab-case name (max 20 chars)
   - Fallback: use existing session type prefix + number
   - Allow manual override via `/session-name` command
   - Auto-detect session type (Debug, Planning, Development, CodeReview) from content

### Phase 2: TUI Layout

**Layout Structure:**
```
┌─────────────────────────────────────────────────────────┐
│ Main Chat Area                    [api-refactor]        │
│                                    [s3-debug]            │
│ Q: How can I help?                 [lambda-deploy]      │
│                                                          │
│ User: Help me refactor the API                          │
│                                                          │
│ Q: I can help with that...                              │
│                                                          │
│                                                          │
│                                                          │
│                                                          │
│                                                          │
│ [Current: api-refactor] q>                              │
└─────────────────────────────────────────────────────────┘
```

**Components:**

1. **Session Indicator (Top-Right Corner)**
   - Shows sessions in `WaitingForInput` state
   - Format: `[session-name]` stacked vertically
   - Color-coded: yellow/orange for waiting
   - Max 5 sessions shown, `+N more` if overflow
   - Updates automatically when session state changes

2. **Main Chat Area**
   - Displays active session conversation
   - Scrollable history
   - Existing chat rendering logic

3. **Status Bar (Bottom)**
   - Shows current session name
   - Input prompt
   - Optional: session count indicator

### Phase 3: Session Management Commands

**New CLI Commands:**

- `/sessions` - List all active sessions with status
  ```
  Active Sessions:
  * api-refactor    [ACTIVE]
    s3-debug        [WAITING]
    lambda-deploy   [PROCESSING]
  ```

- `/switch <session-name>` - Switch to a different session
  - Autocomplete from session names
  - Alias: `/s <name>`

- `/new [name]` - Create new session
  - Optional name parameter
  - Auto-generates name if not provided

- `/session-name [name]` - View or set current session name
  - No args: show current name
  - With arg: set custom name

- `/close [session-name]` - Close a session
  - Defaults to current session
  - Confirm before closing

- `/rename <new-name>` - Rename current session

### Phase 4: Session Persistence

**Storage:**
- Extend existing SQLite database schema (already has conversations table)
- Add session metadata columns to existing conversations table
- Leverage existing conversation_id for session tracking

**Schema Extension:**
```sql
-- Extend existing conversations table (migration 008)
ALTER TABLE conversations ADD COLUMN session_name TEXT;
ALTER TABLE conversations ADD COLUMN session_type TEXT; -- 'Debug', 'Planning', 'Development', 'CodeReview'
ALTER TABLE conversations ADD COLUMN session_status TEXT; -- 'Active', 'WaitingForInput', 'Processing', 'Paused', 'Completed'
ALTER TABLE conversations ADD COLUMN last_active INTEGER;

-- Create index for faster session queries
CREATE INDEX idx_conversations_session_status ON conversations(session_status);
```

**Behavior:**
- Sessions persist across Q CLI restarts (leverage existing conversation persistence)
- `/sessions --all` shows historical sessions
- Auto-cleanup of old inactive sessions (configurable)
- Use existing `ConversationState::save()` and database methods

## Implementation Plan

### Step 0: Understand Current Architecture

**Current Chat Flow:**
1. `ChatCommand::execute()` - Entry point, creates single `conversation_id`
2. `ChatSession::new()` - Creates one `ConversationState` and `ChatSession`
3. `ChatSession::spawn()` - Main loop: `while !matches!(self.inner, Some(ChatState::Exit))`
4. `ChatSession::next()` - State machine handling user input → API call → response
5. Single-threaded, synchronous user interaction (rustyline for input)

**Key Structures:**
- `ChatSession` - Owns one `ConversationState`, handles I/O, tool execution
- `ConversationState` - Conversation history, message state, backend communication
- `InputSource` - Wraps rustyline, handles user input
- `ChatState` enum - State machine: `PromptUser`, `SendMessage`, `HandleResponse`, `Exit`

**Integration Points:**
- Input: `InputSource::read_line()` - blocks waiting for user input
- Output: `ChatSession.stdout` and `ChatSession.stderr` - conduits for output
- State: `ChatSession.inner: Option<ChatState>` - current state
- Loop: `spawn()` calls `next()` repeatedly until `ChatState::Exit`

### Step 1: Refactor for Multi-Session Architecture

**1.1 Create Multi-Session Coordinator**
```rust
// New top-level coordinator
struct MultiSessionCoordinator {
    sessions: HashMap<String, ManagedSession>,
    active_session_id: Option<String>,
    session_manager: SessionManager,
    shared_os: Arc<Mutex<Os>>,
}

struct ManagedSession {
    display: SessionDisplay,
    conversation: ConversationState,
    chat_session: ChatSession,
    task_handle: Option<JoinHandle<Result<()>>>,
    state: SessionState,
}

enum SessionState {
    WaitingForInput,  // Blocked on user input
    Processing,       // Actively processing (API call, tool execution)
    Active,          // Currently displayed to user
}
```

**1.2 Modify ChatSession for Background Execution**
- Add `SessionMode` enum: `Foreground` (current behavior) vs `Background`
- In background mode:
  - Buffer output instead of writing to stderr/stdout
  - Signal coordinator when reaching `ChatState::PromptUser`
  - Pause execution until switched to foreground
- Add channels for coordinator communication:
  ```rust
  struct ChatSession {
      // ... existing fields
      mode: SessionMode,
      output_buffer: Arc<Mutex<Vec<OutputEvent>>>,
      state_change_tx: mpsc::Sender<SessionStateChange>,
      resume_rx: mpsc::Receiver<ResumeSignal>,
  }
  ```

**1.3 Extend SessionManager**
- Add `WaitingForInput` and `Processing` to `SessionStatus` enum
- Add methods:
  - `register_conversation(&mut self, conversation_id: String, conversation: ConversationState)`
  - `get_waiting_sessions(&self) -> Vec<&SessionDisplay>`
  - `update_session_state(&mut self, id: &str, state: SessionStatus)`
  - `get_session_by_id(&self, id: &str) -> Option<&ManagedSession>`

**1.4 Input Routing**
- Create `MultiSessionInputRouter`:
  ```rust
  struct MultiSessionInputRouter {
      coordinator: Arc<Mutex<MultiSessionCoordinator>>,
      input_source: InputSource,
  }
  
  impl MultiSessionInputRouter {
      async fn read_line(&mut self) -> Result<String> {
          // Read from rustyline
          // Check for session commands (/switch, /sessions, /new)
          // Route to active session or handle command
      }
  }
  ```

### Step 2: Implement Session Lifecycle Management

**2.1 Session Creation**
```rust
impl MultiSessionCoordinator {
    async fn create_session(
        &mut self,
        os: &Os,
        session_type: SessionType,
        name: Option<String>,
    ) -> Result<String> {
        // Generate conversation_id
        let conversation_id = uuid::Uuid::new_v4().to_string();
        
        // Create ConversationState (existing code)
        let conversation = ConversationState::new(...).await?;
        
        // Create ChatSession in background mode
        let chat_session = ChatSession::new_background(...).await?;
        
        // Generate or use provided name
        let session_name = name.unwrap_or_else(|| 
            self.generate_session_name(&conversation)
        );
        
        // Create SessionDisplay
        let display = SessionDisplay::new(session_type, session_name);
        
        // Register with SessionManager
        self.session_manager.start_session(session_type, display.name.clone())?;
        
        // Spawn async task for session
        let task_handle = tokio::spawn(async move {
            chat_session.spawn_background(os).await
        });
        
        // Store ManagedSession
        self.sessions.insert(conversation_id.clone(), ManagedSession {
            display,
            conversation,
            chat_session,
            task_handle: Some(task_handle),
            state: SessionState::Active,
        });
        
        Ok(conversation_id)
    }
}
```

**2.2 Session Switching**
```rust
impl MultiSessionCoordinator {
    async fn switch_to_session(&mut self, name: &str) -> Result<()> {
        // Find session by name
        let target_id = self.find_session_id_by_name(name)?;
        
        // Pause current active session
        if let Some(current_id) = &self.active_session_id {
            let current = self.sessions.get_mut(current_id).unwrap();
            current.chat_session.pause().await?;
            current.state = SessionState::WaitingForInput;
            self.session_manager.update_session_state(
                &current.display.name, 
                SessionStatus::WaitingForInput
            )?;
        }
        
        // Resume target session
        let target = self.sessions.get_mut(&target_id).unwrap();
        
        // Flush buffered output
        self.flush_session_output(&target_id).await?;
        
        // Resume execution
        target.chat_session.resume().await?;
        target.state = SessionState::Active;
        self.session_manager.update_session_state(
            &target.display.name,
            SessionStatus::Active
        )?;
        
        self.active_session_id = Some(target_id);
        Ok(())
    }
}
```

**2.3 State Change Handling**
```rust
impl MultiSessionCoordinator {
    async fn handle_state_changes(&mut self) -> Result<()> {
        // Listen for state changes from all sessions
        // When a session reaches PromptUser state:
        //   - Update SessionStatus to WaitingForInput
        //   - Update visual indicator
        //   - If it's the active session, allow input
        //   - If it's a background session, keep it paused
        
        loop {
            tokio::select! {
                Some(change) = self.state_change_rx.recv() => {
                    match change {
                        SessionStateChange::NeedsInput(session_id) => {
                            let session = self.sessions.get_mut(&session_id).unwrap();
                            session.state = SessionState::WaitingForInput;
                            self.session_manager.update_session_state(
                                &session.display.name,
                                SessionStatus::WaitingForInput
                            )?;
                            self.update_visual_indicator().await?;
                        }
                        SessionStateChange::Processing(session_id) => {
                            let session = self.sessions.get_mut(&session_id).unwrap();
                            session.state = SessionState::Processing;
                            self.session_manager.update_session_state(
                                &session.display.name,
                                SessionStatus::Processing
                            )?;
                            self.update_visual_indicator().await?;
                        }
                    }
                }
            }
        }
    }
}
```
- Implement conversation analysis using existing `ConversationState` history
- Generate descriptive names from context
- Auto-detect session type from conversation content
- Add manual override capability via new command
- Update `SessionDisplay` name when generated

### Step 3: Session Name Generation

**3.1 Implement Name Generator**
```rust
impl MultiSessionCoordinator {
    fn generate_session_name(&self, conversation: &ConversationState) -> String {
        // Analyze first 2-3 messages in conversation history
        let history = conversation.history();
        let messages: Vec<_> = history.iter().take(3).collect();
        
        // Extract keywords from user messages
        let keywords = self.extract_keywords(&messages);
        
        // Generate kebab-case name (max 20 chars)
        let name = self.format_session_name(&keywords);
        
        // Ensure uniqueness
        self.ensure_unique_name(name)
    }
    
    fn extract_keywords(&self, messages: &[&HistoryEntry]) -> Vec<String> {
        // Common technical terms and actions
        let important_terms = [
            "api", "refactor", "debug", "test", "deploy", "lambda",
            "s3", "database", "fix", "optimize", "implement", "review"
        ];
        
        // Extract from user messages
        // Use simple keyword matching or basic NLP
        // Return top 2-3 most relevant terms
    }
    
    fn detect_session_type(&self, keywords: &[String]) -> SessionType {
        // Heuristics to detect session type from keywords
        if keywords.iter().any(|k| k.contains("debug") || k.contains("fix")) {
            SessionType::Debug
        } else if keywords.iter().any(|k| k.contains("plan") || k.contains("design")) {
            SessionType::Planning
        } else if keywords.iter().any(|k| k.contains("review") || k.contains("check")) {
            SessionType::CodeReview
        } else {
            SessionType::Development
        }
    }
}
```

**3.2 Manual Override**
- Add `/session-name <new-name>` command handler
- Update `SessionDisplay.name` in `SessionManager`
- Persist to database

### Step 4: ratatui Integration for Visual Indicator

**4.1 Add ratatui Dependency**
```toml
# Cargo.toml
[dependencies]
ratatui = "0.26"  # Uses existing crossterm backend
```

**4.2 Create TUI Component**
```rust
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph},
    Terminal,
};

struct SessionIndicator {
    coordinator: Arc<Mutex<MultiSessionCoordinator>>,
    terminal: Terminal<CrosstermBackend<std::io::Stderr>>,
}

impl SessionIndicator {
    fn render(&mut self) -> Result<()> {
        let coordinator = self.coordinator.lock().await;
        let waiting_sessions = coordinator.session_manager.get_waiting_sessions();
        
        if waiting_sessions.is_empty() {
            return Ok(());
        }
        
        // Get terminal size
        let size = terminal::size()?;
        
        // Calculate position (top-right corner)
        let indicator_width = 25;
        let indicator_height = waiting_sessions.len() as u16 + 2;
        let x = size.0.saturating_sub(indicator_width);
        let y = 0;
        
        // Save cursor position
        execute!(std::io::stderr(), cursor::SavePosition)?;
        
        // Draw indicator
        self.terminal.draw(|f| {
            let area = Rect {
                x,
                y,
                width: indicator_width,
                height: indicator_height,
            };
            
            let lines: Vec<Line> = waiting_sessions
                .iter()
                .map(|session| {
                    Line::from(vec![
                        Span::styled(
                            format!("[{}]", session.name),
                            Style::default().fg(Color::Yellow),
                        )
                    ])
                })
                .collect();
            
            let paragraph = Paragraph::new(lines)
                .alignment(Alignment::Right);
            
            f.render_widget(paragraph, area);
        })?;
        
        // Restore cursor position
        execute!(std::io::stderr(), cursor::RestorePosition)?;
        
        Ok(())
    }
    
    async fn update_on_state_change(&mut self) {
        // Subscribe to state changes
        // Redraw indicator when sessions change state
    }
}
```

**4.3 Integration with Main Loop**
```rust
impl MultiSessionCoordinator {
    async fn run(&mut self, os: &mut Os) -> Result<()> {
        // Create indicator
        let indicator = SessionIndicator::new(Arc::clone(&self));
        
        // Spawn indicator update task
        tokio::spawn(async move {
            indicator.update_on_state_change().await
        });
        
        // Main loop
        loop {
            // Update indicator
            self.update_visual_indicator().await?;
            
            // Handle input
            let input = self.input_router.read_line().await?;
            
            // Process input (route to active session or handle command)
            self.process_input(input, os).await?;
        }
    }
}
```

### Step 5: Session Management Commands

**5.1 Command Parser**
```rust
enum SessionCommand {
    List,
    Switch(String),
    New { session_type: Option<SessionType>, name: Option<String> },
    Close(Option<String>),
    Rename(String),
    SessionName(Option<String>),
}

impl MultiSessionCoordinator {
    fn parse_session_command(&self, input: &str) -> Option<SessionCommand> {
        if input.starts_with("/sessions") {
            Some(SessionCommand::List)
        } else if let Some(name) = input.strip_prefix("/switch ") {
            Some(SessionCommand::Switch(name.trim().to_string()))
        } else if input.starts_with("/new") {
            // Parse optional type and name
            Some(SessionCommand::New { ... })
        } else if let Some(name) = input.strip_prefix("/close") {
            Some(SessionCommand::Close(...))
        } else if let Some(name) = input.strip_prefix("/rename ") {
            Some(SessionCommand::Rename(name.trim().to_string()))
        } else if input.starts_with("/session-name") {
            Some(SessionCommand::SessionName(...))
        } else {
            None
        }
    }
}
```

**5.2 Command Handlers**
```rust
impl MultiSessionCoordinator {
    async fn handle_command(&mut self, cmd: SessionCommand, os: &Os) -> Result<()> {
        match cmd {
            SessionCommand::List => {
                let sessions = self.session_manager.list_sessions();
                for session in sessions {
                    println!("{}", session.colored_list_entry(&theme::theme().session));
                }
            }
            SessionCommand::Switch(name) => {
                self.switch_to_session(&name).await?;
            }
            SessionCommand::New { session_type, name } => {
                let session_type = session_type.unwrap_or(SessionType::Development);
                let id = self.create_session(os, session_type, name).await?;
                self.switch_to_session_by_id(&id).await?;
            }
            SessionCommand::Close(name) => {
                let name = name.unwrap_or_else(|| {
                    self.get_active_session_name().unwrap()
                });
                self.close_session(&name).await?;
            }
            SessionCommand::Rename(new_name) => {
                self.rename_active_session(new_name)?;
            }
            SessionCommand::SessionName(new_name) => {
                if let Some(name) = new_name {
                    self.rename_active_session(name)?;
                } else {
                    println!("Current session: {}", self.get_active_session_name()?);
                }
            }
        }
        Ok(())
    }
}
```

**5.3 Command Autocomplete**
- Extend existing rustyline completer
- Add session names to completion candidates
- Add session command completion

### Step 6: Enhanced Session Persistence
- Add ratatui dependency (uses existing crossterm backend)
- Create TUI component for top-right corner indicator
- Render sessions with `WaitingForInput` status
- Use existing `SessionDisplay::colored_list_entry()` for formatting
- Maintain existing input/output flow with rustyline

### Step 4: Session Management Commands
- Implement `/sessions` - list all sessions (extend existing list functionality)
- Implement `/switch <name>` - switch active conversation
- Implement `/new [type] [name]` - create new session (leverage existing session types)
- Implement `/session-name [name]` - view/set name
- Add command autocomplete for session names
- Update help text

### Step 5: Enhanced Session Persistence
- Extend existing database schema (already has conversations table)
- Add session metadata columns to conversations table
- Store session type, name, and status
- Link existing conversation_id to session metadata
- Implement session save/load using existing database methods
- Add session cleanup logic

### Step 6: Polish
- Use existing `SessionColors` for styling
- Improve session name generation algorithm
- Add configuration options to existing settings
- Update documentation
- Test over SSH

## Detailed Implementation Steps

### Step 6 Details: Database Persistence

**6.1 Database Schema Migration**
```sql
-- Migration 008: Add session metadata to conversations table
ALTER TABLE conversations ADD COLUMN session_name TEXT;
ALTER TABLE conversations ADD COLUMN session_type TEXT; 
ALTER TABLE conversations ADD COLUMN session_status TEXT;
ALTER TABLE conversations ADD COLUMN last_active INTEGER;

CREATE INDEX idx_conversations_session_status ON conversations(session_status);
```

**6.2 Persistence Methods**
```rust
impl MultiSessionCoordinator {
    async fn save_session(&self, session_id: &str, os: &Os) -> Result<()> {
        let session = self.sessions.get(session_id).unwrap();
        
        // Save conversation state (already exists)
        session.conversation.save(os).await?;
        
        // Save session metadata
        os.database.execute(
            "UPDATE conversations SET 
             session_name = ?, 
             session_type = ?, 
             session_status = ?,
             last_active = ?
             WHERE conversation_id = ?",
            params![
                session.display.name,
                format!("{:?}", session.display.session_type),
                format!("{:?}", session.display.status),
                time::OffsetDateTime::now_utc().unix_timestamp(),
                session_id,
            ]
        ).await?;
        
        Ok(())
    }
    
    async fn load_sessions(&mut self, os: &Os) -> Result<()> {
        // Load recent sessions from database
        let rows = os.database.query(
            "SELECT conversation_id, session_name, session_type, session_status 
             FROM conversations 
             WHERE session_status IN ('Active', 'WaitingForInput', 'Paused')
             ORDER BY last_active DESC
             LIMIT 10",
            []
        ).await?;
        
        for row in rows {
            // Reconstruct sessions from database
        }
        
        Ok(())
    }
}
```

### Step 7: Entry Point Integration

**7.1 Modify ChatCommand::execute()**
```rust
impl ChatCommand {
    pub async fn execute(mut self, os: &mut Os) -> Result<ExitCode> {
        // Check if multi-session mode is enabled
        let multi_session_enabled = os.database
            .settings
            .get_bool(Setting::MultiSessionEnabled)
            .unwrap_or(false);
        
        if multi_session_enabled {
            // Use new multi-session coordinator
            let mut coordinator = MultiSessionCoordinator::new(os).await?;
            coordinator.load_sessions(os).await?;
            
            if coordinator.sessions.is_empty() {
                let id = coordinator.create_session(os, SessionType::Development, None).await?;
                coordinator.switch_to_session_by_id(&id).await?;
            }
            
            coordinator.run(os).await?;
            Ok(ExitCode::SUCCESS)
        } else {
            // Use existing single-session flow (backward compatibility)
            // ... existing code ...
        }
    }
}
```

### Step 8: Testing and Polish

**8.1 Configuration Options**
```toml
[sessions]
multi_session_enabled = false  # Feature flag
max_active_sessions = 10
auto_generate_names = true
persist_sessions = true
cleanup_after_days = 30

[ui]
show_session_indicator = true
indicator_position = "top-right"
max_indicator_sessions = 5
```

**8.2 Testing Strategy**
- Unit tests for SessionManager operations
- Integration tests for session switching
- Manual testing over SSH
- Test concurrent API calls
- Test graceful shutdown and recovery

**8.3 Rollout Plan**
- Phase 1: Feature flag (default off)
- Phase 2: Beta testing with opt-in
- Phase 3: General availability

## Technical Considerations

### Leveraging Existing Code

**Session Management:**
- Build on existing `SessionManager` and `SessionDisplay` in `theme/` module
- Extend existing `SessionStatus` enum rather than creating new state types
- Use existing session type system (Debug, Planning, Development, CodeReview)
- Leverage existing colored formatting with `SessionColors`

**Conversation Management:**
- Each session wraps an existing `ConversationState` instance
- Use existing `conversation_id` for session identification
- Leverage existing conversation persistence in database
- Use existing `@session/` path resolution for session-scoped files

**Database:**
- Extend existing conversations table rather than creating new tables
- Use existing database connection pool and migration system
- Leverage existing `Os::database` methods for persistence

### Terminal Compatibility
- Use `crossterm` backend for ratatui (already in dependencies)
- Test over SSH connections
- Handle terminal resize events
- Graceful degradation if terminal too small
- Existing crossterm utilities in `theme/crossterm_ext.rs`

### Performance
- Lazy load conversation history for inactive sessions
- Limit in-memory sessions (e.g., max 10 active)
- Efficient rendering updates (only redraw changed components)
- Leverage existing conversation state caching

### Concurrency
- Each session runs in its own async task
- Session manager coordinates state updates
- Thread-safe session access with `Arc<Mutex<ManagedSession>>`
- Use existing tokio runtime

### Backward Compatibility
- Single-session mode remains default behavior
- Existing conversations automatically become default session
- Multi-session features opt-in via commands
- Existing conversation history migrates seamlessly

## Future Enhancements

### Phase 5+: Advanced TUI Features
- Split-pane view (multiple sessions visible)
- Session tabs instead of corner indicator
- File browser component
- Log viewer component
- Interactive session picker (fuzzy search)

### Potential Migration to tui-realm
If UI complexity grows significantly:
- Component-based architecture
- Reusable UI components
- Event-driven model
- Easier to add new interactive features

## Configuration

**New config options:**
```toml
[sessions]
max_active = 10
auto_generate_names = true
name_max_length = 20
persist_sessions = true
cleanup_after_days = 30

[ui]
show_session_indicator = true
indicator_position = "top-right"  # or "top-left", "bottom-right"
max_indicator_sessions = 5
```

## Testing Strategy

- Unit tests for session manager
- Unit tests for name generation
- Integration tests for session switching
- Manual testing over SSH
- Terminal compatibility testing (various terminal emulators)
- Performance testing with many sessions

## Documentation Updates

- Update README with multi-session features
- Add user guide for session management
- Update command reference
- Add examples and screenshots

## Success Metrics

- Users can run 3+ concurrent sessions without confusion
- Session switching takes < 1 second
- Session names are descriptive 80%+ of the time
- No performance degradation with up to 10 active sessions
- Works reliably over SSH connections
