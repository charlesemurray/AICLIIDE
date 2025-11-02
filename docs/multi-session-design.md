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

## Architecture

### Phase 1: Foundation (ratatui integration)

**Dependencies:**
- Add `ratatui` (built on existing `crossterm` backend)
- Keep existing `rustyline`, `dialoguer`, etc. for now

**Core Components:**

1. **Session Manager**
   - Maintains list of active sessions
   - Tracks session state: `Active`, `WaitingForInput`, `Processing`
   - Generates and stores session names
   - Handles session switching

2. **Session State**
   ```rust
   struct Session {
       id: Uuid,
       name: String,  // auto-generated or user-provided
       state: SessionState,
       conversation_history: Vec<Message>,
       created_at: DateTime,
       last_active: DateTime,
   }
   
   enum SessionState {
       Active,           // Currently displayed
       WaitingForInput,  // Completed response, needs user input
       Processing,       // Waiting for Q response
   }
   ```

3. **Session Name Generator**
   - Analyze first 2-3 user messages in conversation
   - Extract key topics/actions (e.g., "api", "refactor", "s3", "debug")
   - Generate short kebab-case name (max 20 chars)
   - Fallback: `session-{number}` if generation fails
   - Allow manual override via `/session-name` command

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
- Extend existing SQLite database schema
- Store session metadata and conversation history per session
- Auto-save on session state changes

**Schema:**
```sql
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    state TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    last_active INTEGER NOT NULL
);

-- Extend existing messages table
ALTER TABLE messages ADD COLUMN session_id TEXT REFERENCES sessions(id);
```

**Behavior:**
- Sessions persist across Q CLI restarts
- `/sessions --all` shows historical sessions
- Auto-cleanup of old inactive sessions (configurable)

## Implementation Plan

### Step 1: Session Manager Core
- Create session manager module
- Implement session state tracking
- Add basic session switching (no UI yet)
- Store sessions in memory

### Step 2: Session Name Generation
- Implement conversation analysis
- Generate descriptive names from context
- Add manual override capability

### Step 3: ratatui Integration
- Add ratatui dependency
- Create basic layout with main area + indicator
- Render session indicator in top-right corner
- Maintain existing input/output flow

### Step 4: Session Commands
- Implement `/sessions`, `/switch`, `/new` commands
- Add command autocomplete for session names
- Update help text

### Step 5: Session Persistence
- Extend database schema
- Implement session save/load
- Add session cleanup logic

### Step 6: Polish
- Add colors and styling
- Improve session name generation
- Add configuration options
- Update documentation

## Technical Considerations

### Terminal Compatibility
- Use `crossterm` backend for ratatui (already in dependencies)
- Test over SSH connections
- Handle terminal resize events
- Graceful degradation if terminal too small

### Performance
- Lazy load conversation history for inactive sessions
- Limit in-memory sessions (e.g., max 10 active)
- Efficient rendering updates (only redraw changed components)

### Concurrency
- Each session runs in its own async task
- Session manager coordinates state updates
- Thread-safe session access with `Arc<Mutex<Session>>`

### Backward Compatibility
- Single-session mode remains default behavior
- Multi-session features opt-in via commands
- Existing conversation history migrates to default session

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
