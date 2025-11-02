# Multi-Session Implementation Summary

## Overview

This document provides a high-level summary of the multi-session feature implementation plan for Q CLI.

## Problem Statement

Users running multiple Q CLI instances simultaneously face coordination overhead:
- Hard to track which session is doing what
- Difficult to know when sessions need user input
- Context switching between terminal windows/tabs
- No unified view of all active work

## Solution

Native multi-session support within a single Q CLI instance with:
- Visual indicator showing sessions waiting for input
- Auto-generated descriptive session names
- Session management commands
- Works over SSH (no desktop notifications needed)

## Architecture Overview

### Current Architecture
- Single `ChatSession` per Q CLI instance
- Synchronous user interaction loop
- One `ConversationState` per session
- State machine: PromptUser → SendMessage → HandleResponse → Exit

### New Architecture
```
MultiSessionCoordinator
├── SessionManager (existing, extended)
│   └── Tracks session metadata and state
├── ManagedSession (new)
│   ├── SessionDisplay (existing)
│   ├── ConversationState (existing)
│   ├── ChatSession (modified for background mode)
│   └── Async task handle
├── SessionIndicator (new, ratatui)
│   └── Top-right corner visual display
└── MultiSessionInputRouter (new)
    └── Routes input to active session or handles commands
```

## Key Components

### 1. MultiSessionCoordinator
- Central orchestrator for all sessions
- Manages session lifecycle (create, switch, close)
- Handles state transitions
- Coordinates input/output routing

### 2. Modified ChatSession
- New `SessionMode`: Foreground vs Background
- Output buffering for background sessions
- Pause/resume capability
- State change notifications

### 3. SessionIndicator (ratatui)
- Renders in top-right corner
- Shows sessions with `WaitingForInput` status
- Updates automatically on state changes
- Uses existing `SessionColors` for styling

### 4. Session Commands
- `/sessions` - List all active sessions
- `/switch <name>` - Switch to a session
- `/new [type] [name]` - Create new session
- `/close [name]` - Close a session
- `/session-name [name]` - View/set session name
- `/rename <name>` - Rename current session

## Implementation Phases

### Phase 0: Architecture Analysis ✓
- Understand current chat loop
- Identify integration points
- Design multi-session coordinator

### Phase 1: Core Infrastructure
- Create `MultiSessionCoordinator`
- Modify `ChatSession` for background mode
- Extend `SessionManager` with new states
- Implement input routing

### Phase 2: Session Lifecycle
- Session creation with async tasks
- Session switching with pause/resume
- State change handling and notifications
- Output buffering and flushing

### Phase 3: Name Generation
- Keyword extraction from conversation
- Auto-detect session type
- Generate descriptive names
- Ensure uniqueness

### Phase 4: Visual Indicator
- Add ratatui dependency
- Create TUI component
- Render in top-right corner
- Update on state changes

### Phase 5: Commands
- Parse session commands
- Implement command handlers
- Add autocomplete support
- Update help text

### Phase 6: Persistence
- Extend database schema
- Save/load session metadata
- Auto-save on state changes
- Cleanup old sessions

### Phase 7: Integration
- Modify `ChatCommand::execute()`
- Feature flag for gradual rollout
- Backward compatibility
- Graceful shutdown

### Phase 8: Polish
- Configuration options
- Error handling
- Testing (unit, integration, SSH)
- Documentation

## Leveraging Existing Code

### Already Implemented
- `SessionManager` - Session tracking and management
- `SessionDisplay` - Display formatting with colors
- `SessionStatus` - Active, Paused, Completed states
- `ConversationState` - Conversation history and persistence
- `@session/` path resolution - File scoping

### Need to Extend
- Add `WaitingForInput` and `Processing` to `SessionStatus`
- Link `SessionManager` to `ConversationState` instances
- Add background mode to `ChatSession`
- Extend database schema for session metadata

### Need to Create
- `MultiSessionCoordinator` - Top-level orchestrator
- `ManagedSession` - Links display + conversation + task
- `SessionIndicator` - ratatui visual component
- `MultiSessionInputRouter` - Input routing logic
- Session name generator
- Command parser and handlers

## Technical Decisions

### Why ratatui?
- Built on existing `crossterm` backend
- Cross-platform (Linux, macOS)
- Works over SSH
- Efficient rendering
- Can be added incrementally

### Why Not Desktop Notifications?
- Don't work over SSH (primary use case)
- Platform-specific complications
- In-terminal solution is more reliable

### Why Feature Flag?
- Gradual rollout
- Backward compatibility
- Easy to disable if issues arise
- Allows beta testing

### Why Extend Existing SessionManager?
- Already has session types and colors
- Consistent with existing code patterns
- Less duplication
- Easier maintenance

## Success Criteria

- Users can run 3+ concurrent sessions without confusion
- Session switching takes < 1 second
- Session names are descriptive 80%+ of the time
- No performance degradation with up to 10 active sessions
- Works reliably over SSH connections
- Backward compatible with single-session mode

## Risks and Mitigations

### Risk: Complexity of Concurrent Sessions
**Mitigation:** Start with feature flag, thorough testing, clear state management

### Risk: Terminal Rendering Issues
**Mitigation:** Test on multiple terminal emulators, graceful degradation

### Risk: Performance with Many Sessions
**Mitigation:** Lazy loading, output buffering, session limits

### Risk: User Confusion
**Mitigation:** Clear documentation, intuitive commands, helpful error messages

## Timeline Estimate

- Phase 0 (Analysis): ✓ Complete
- Phase 1 (Core Infrastructure): 2-3 weeks
- Phase 2 (Session Lifecycle): 1-2 weeks
- Phase 3 (Name Generation): 1 week
- Phase 4 (Visual Indicator): 1 week
- Phase 5 (Commands): 1 week
- Phase 6 (Persistence): 1 week
- Phase 7 (Integration): 1 week
- Phase 8 (Polish): 2 weeks

**Total: 10-13 weeks**

## Next Steps

1. Review and approve design document
2. Create feature flag in settings
3. Begin Phase 1 implementation
4. Set up testing infrastructure
5. Create tracking issues for each phase
