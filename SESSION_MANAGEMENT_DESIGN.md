# Session Management Design

## Overview

Unified session management system for Q CLI that handles active work, background tasks, and historical reference.

## Session Types

### 1. Active Sessions
**Purpose:** Interactive work contexts the user switches between

**Use cases:**
- Working on feature A, switch to debug bug B, switch back to feature A
- Multiple concurrent development tasks
- Skill/agent development and iteration

**Characteristics:**
- User actively types and interacts
- Can switch between them
- Persist across Q CLI restarts
- Limited number (e.g., 5-10 active at once)

### 2. Background Sessions
**Purpose:** Long-running autonomous tasks/agents

**Use cases:**
- Monitoring deployments
- Continuous test running
- Code review watching
- Autonomous refactoring agents
- Scheduled/periodic tasks
- Log analysis

**Characteristics:**
- Run without active user interaction
- User checks status periodically
- Can be paused/resumed/stopped
- Generate notifications/alerts
- May run for hours/days

### 3. Historical Sessions
**Purpose:** Completed conversations for reference

**Use cases:**
- "What analysis did I do last week?"
- Reference old design decisions
- Copy artifacts from previous work
- Audit trail of development

**Characteristics:**
- Read-only (or archive mode)
- Searchable by topic/date
- Can be cleaned up after time
- Artifacts preserved

## Directory Structure

```
.amazonq/sessions/
  {uuid-1}/
    metadata.json
    analysis.md
    design.md
    logs/
      agent.log
  {uuid-2}/
    metadata.json
    implementation-plan.md
  ...
```

## Metadata Schema

`.amazonq/sessions/{uuid}/metadata.json`:
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "active" | "background" | "archived",
  "created": "2025-11-02T18:00:00Z",
  "last_active": "2025-11-02T19:15:00Z",
  "first_message": "Analyze the authentication flow",
  "name": null,
  "type": "feature" | "bugfix" | "skill" | "agent" | "analysis",
  "file_count": 3,
  "message_count": 15,
  "background_task": {
    "status": "running" | "paused" | "completed" | "failed",
    "started": "2025-11-02T18:00:00Z",
    "last_update": "2025-11-02T19:00:00Z",
    "description": "Monitoring deployment"
  }
}
```

## Commands

### `/sessions list`
Show active sessions
```
Active Sessions:
  1. feature-auth (2 hours ago) - "Implement OAuth flow"
  2. bugfix-login (1 day ago) - "Fix login redirect"
  3. skill-deploy (current) - "Create deployment skill"
```

### `/sessions background`
Show background tasks
```
Background Sessions:
  1. deploy-monitor (running, 3h) - "Monitor prod deployment"
  2. test-runner (running, 1d) - "Continuous test execution"
  3. code-review (paused) - "Watch for PR comments"
```

### `/sessions history [--limit N] [--search TERM]`
Show historical sessions
```
Recent History:
  1. "Analyze authentication flow" (2 days ago) - 3 files
  2. "Create user management" (1 week ago) - 5 files
  3. "Fix login bug" (2 weeks ago) - 2 files
```

### `/sessions switch <name|number>`
Switch to an active session
```
/sessions switch feature-auth
Switched to session: feature-auth
```

### `/sessions name <name>`
Name the current session
```
/sessions name feature-auth
Session named: feature-auth
```

### `/sessions archive [<name|number>]`
Archive current or specified session
```
/sessions archive
Session archived: feature-auth
```

### `/sessions background start <description>`
Start a background task in current session
```
/sessions background start "Monitor deployment"
Background task started
```

### `/sessions background stop <name|number>`
Stop a background task
```
/sessions background stop deploy-monitor
Background task stopped
```

### `/sessions clean --older-than <days>`
Clean up old archived sessions
```
/sessions clean --older-than 30
Cleaned up 5 sessions older than 30 days
```

## Session Lifecycle

### Active Session
```
Create → Active → Archive → Historical
              ↓
         Background (optional)
```

### Background Session
```
Start → Running → Paused → Running → Completed → Historical
                     ↓
                  Stopped
```

## Implementation Phases

### Phase 1: Core Infrastructure (Current)
- ✅ Session directories created per conversation
- ✅ System prompt tells Q to use session workspace
- ⏳ Metadata file creation

### Phase 2: Session Listing
- Read `.amazonq/sessions/` directory
- Parse metadata files
- Display by status (active/background/archived)
- Basic filtering and search

### Phase 3: Session Switching
- Save current conversation state
- Load target session state
- Restore context and history
- Update metadata timestamps

### Phase 4: Background Tasks
- Background task execution framework
- Status monitoring and updates
- Notification system
- Pause/resume/stop controls

### Phase 5: Session Management
- Archive/unarchive
- Naming and tagging
- Cleanup and retention policies
- Export/import

## Key Design Decisions

### 1. Filesystem-based
- All sessions stored in `.amazonq/sessions/`
- No in-memory HashMap (remove existing implementation)
- Metadata in JSON for easy inspection/editing

### 2. Explicit Status
- Sessions have clear status: active/background/archived
- Status determines behavior and UI presentation
- Transitions are explicit user actions

### 3. Metadata-driven
- All session info in metadata.json
- Easy to query without loading full conversation
- Supports future extensions (tags, labels, etc.)

### 4. Workspace per Session
- Each session has isolated directory
- Prevents cross-contamination
- Easy to find artifacts by session

### 5. Background Task Support
- Background sessions can run autonomously
- Status updates written to metadata
- Logs stored in session directory

## Migration from Current System

### Current State
- In-memory HashMap for "development sessions"
- No persistence
- No session folders
- No metadata

### Migration Steps
1. Keep existing `/sessions` commands as-is initially
2. Add metadata creation to new sessions
3. Implement `/sessions list` to read from filesystem
4. Deprecate HashMap-based system
5. Remove old implementation

## Future Enhancements

### Session Templates
- Create sessions from templates
- Pre-populate with common files/structure

### Session Sharing
- Export session as archive
- Import shared sessions
- Collaboration features

### Session Analytics
- Time spent per session
- Productivity metrics
- Common patterns

### Smart Archiving
- Auto-archive inactive sessions after N days
- Suggest archiving based on completion signals

### Session Search
- Full-text search across session artifacts
- Search by date, type, tags
- Find similar sessions

## Open Questions

1. **How many active sessions is reasonable?**
   - Suggest limit of 10 active sessions
   - Warn user if approaching limit
   - Suggest archiving old ones

2. **Background task execution model?**
   - Separate process? Thread? Async task?
   - How to handle Q CLI restarts?
   - Persistence of background state?

3. **Session switching behavior?**
   - Save current state automatically?
   - Prompt to save changes?
   - What about unsaved work?

4. **Metadata update frequency?**
   - Update on every message? (expensive)
   - Update on session close? (may lose data)
   - Periodic updates? (complexity)

5. **Cleanup policies?**
   - Default retention period?
   - User-configurable?
   - Warn before deletion?
