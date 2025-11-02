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

### Step 1: Extend SessionManager for Multi-Conversation Support
- Add `WaitingForInput` and `Processing` to existing `SessionStatus` enum
- Create `ManagedSession` struct linking `SessionDisplay` with `ConversationState`
- Extend `SessionManager` to manage multiple `ConversationState` instances
- Add async task spawning for concurrent conversation processing
- Store sessions in memory with conversation_id as key

### Step 2: Session Name Generation
- Implement conversation analysis using existing `ConversationState` history
- Generate descriptive names from context
- Auto-detect session type from conversation content
- Add manual override capability via new command
- Update `SessionDisplay` name when generated

### Step 3: ratatui Integration for Visual Indicator
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
