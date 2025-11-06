# Session Architecture Analysis

## Current Architecture

### 1. ChatArgs::execute() - Entry Point
```
ChatArgs::execute()
├── Creates MultiSessionCoordinator (line 299)
├── Loads saved sessions from disk
├── Creates ONE ChatSession (line 807)
├── Registers it with coordinator (line 830-856)
└── Calls session.spawn() (line 858)
```

### 2. ChatSession - The Active Session
- **Purpose**: Manages ONE active conversation
- **Contains**:
  - `conversation: ConversationState` - The actual chat history
  - `coordinator: Option<Arc<Mutex<MultiSessionCoordinator>>>` - Reference to coordinator
  - `tool_uses`, `pending_prompts`, etc. - Active state
- **Lifecycle**: Created once, runs until exit

### 3. MultiSessionCoordinator - Session Manager
- **Purpose**: Manages MULTIPLE sessions
- **Contains**:
  - `sessions: HashMap<String, ManagedSession>` - All sessions
  - `active_session_id: Option<String>` - Which one is "active"
- **Storage**: Each ManagedSession has its own ConversationState

### 4. The Problem

**Current Flow:**
```
User starts Q
  → Creates Coordinator (empty)
  → Creates ChatSession with conversation A
  → Registers A in coordinator
  → User types "/sessions new hello"
    → Creates conversation B in coordinator
  → User types "/switch hello"
    → Coordinator sets active_session_id = B
    → ChatSession STILL RUNNING conversation A
    → Messages go to A, not B!
```

**The Issue**: ChatSession is the "runner" but it only knows about ONE conversation. The coordinator tracks multiple conversations but has no way to make ChatSession use a different one.

## What We Need

### Goal: Multiple Independent Sessions
Each session should have:
- Its own conversation history
- Its own tool state
- Its own pending messages
- Ability to run in background
- Ability to switch between them

### The UI Library Approach (What You Suggested)

Instead of swapping conversations, we should:
1. Keep all ChatSessions alive in the coordinator
2. Have ONE active/visible session
3. Others run in background (buffered output)
4. Switch = hide current, show target

## Proposed Architecture

### Option 1: Coordinator Owns ChatSessions (Proper)
```
MultiSessionCoordinator
├── sessions: HashMap<String, ManagedSession>
│   ├── ManagedSession {
│   │     conversation: ConversationState,
│   │     chat_session: ChatSession,  // ← Add this
│   │     output_buffer: OutputBuffer,
│   │     state: Active/Background
│   │   }
│   └── ...
├── active_session_id: String
└── Methods:
    ├── switch_session(id) → Pauses current, activates target
    ├── render_active() → Displays active session's output
    └── tick_background() → Processes background sessions
```

**Changes Needed:**
1. Move ChatSession INTO ManagedSession
2. ChatArgs::execute() creates coordinator, coordinator creates sessions
3. Main loop asks coordinator "what should I display?"
4. Coordinator manages all session lifecycles

### Option 2: Single ChatSession as View (Simpler)
```
ChatSession (just a renderer)
├── coordinator: Arc<Mutex<Coordinator>>
├── active_session_id: String
└── Methods:
    ├── render() → Gets active session from coordinator, displays it
    ├── handle_input() → Sends to coordinator's active session
    └── next() → Asks coordinator for next state
```

**Changes Needed:**
1. ChatSession becomes stateless view
2. All state lives in coordinator
3. ChatSession just renders whatever coordinator says

### Option 3: Hybrid (Current + Fixes)
```
Keep current architecture but:
1. When switching, actually swap the ConversationState
2. Save old conversation back to coordinator
3. Load new conversation from coordinator
4. Update all related state (tool_uses, etc.)
```

**This is what we've been trying** - it's hacky but might work.

## Recommendation

**Go with Option 1** - Coordinator Owns ChatSessions

### Why:
- Clean separation: Coordinator = session manager, ChatSession = conversation runner
- Background sessions can actually run (not just stored)
- Switching is just changing which one is visible
- Matches your "hide/show" vision
- Uses the UI library properly (OutputBuffer already exists for this!)

### Implementation Steps:
1. Add `chat_session: Option<ChatSession>` to ManagedSession
2. When creating session in coordinator, create its ChatSession too
3. Main loop becomes: `coordinator.tick()` which runs active session
4. Switch = `coordinator.set_active(id)` which pauses current, resumes target
5. Render = `coordinator.render_active()` which displays active session's output

### Key Insight:
The coordinator ALREADY has OutputBuffer for background sessions! It was designed for this. We just need to actually use it.

## Current Blockers

1. **ChatSession is created outside coordinator** (line 807)
   - Should be created BY coordinator

2. **ChatSession.spawn() is the main loop** (line 858)
   - Should be coordinator.run() that manages all sessions

3. **Conversation swapping is fragile**
   - Should just switch which ChatSession is active

## Next Steps

1. Decide on architecture (recommend Option 1)
2. Refactor ChatArgs::execute() to use coordinator as main controller
3. Move ChatSession creation into coordinator
4. Implement proper session switching
5. Test with multiple sessions running simultaneously
