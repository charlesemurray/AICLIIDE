# Phase 2 Implementation: Pause/Resume + Coordinator Main Loop

## Current State
- ChatSession has pause_rx/resume_tx fields (unused)
- spawn() method runs single session to completion
- Coordinator tracks multiple ManagedSessions with chat_session field
- No actual pause/resume or background execution

## Implementation Steps

### 1. Add Pause/Resume to ChatSession
- Check pause_rx in spawn() main loop
- When paused: save state, stop processing, wait for resume
- When resumed: restore state, continue processing
- Minimal changes to spawn() loop

### 2. Coordinator Main Loop
- Add coordinator.run() method
- Manages active ChatSession rendering
- Handles session switching by pausing current, resuming target
- Routes input to active session

### 3. Refactor ChatArgs::execute()
- Don't call session.spawn() directly
- Call coordinator.run() instead
- Coordinator owns the main loop

## Key Design Decisions
- Keep spawn() mostly unchanged - just add pause check
- Coordinator.run() wraps spawn() execution
- Use existing pause_rx/resume_tx channels
- Minimal code changes for maximum impact
