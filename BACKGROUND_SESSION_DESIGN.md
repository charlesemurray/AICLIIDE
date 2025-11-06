# Background Session Execution Design

## Goal
Enable sessions to continue LLM interactions in the background while user switches to other sessions.

## Current Architecture Issues

### Problems
1. **Blocking spawn()**: `ChatSession.spawn()` blocks until session exits
2. **Terminal coupling**: Session owns stdin/stdout, can't switch while active
3. **Os not Send**: Can't move `Os` to background tasks
4. **Single terminal**: Only one session can read input at a time

### What Works
- ✅ Conversation state persistence
- ✅ History restoration
- ✅ Session metadata tracking

## Proposed Architecture

### Core Concept: Detached Sessions

```
┌─────────────────────────────────────────────────────────┐
│                    Coordinator                          │
│  - Manages all sessions                                 │
│  - Routes terminal I/O to active session                │
│  - Monitors background sessions                         │
└─────────────────────────────────────────────────────────┘
           │                    │                    │
           ▼                    ▼                    ▼
    ┌──────────┐         ┌──────────┐         ┌──────────┐
    │ Session 1│         │ Session 2│         │ Session 3│
    │ ACTIVE   │         │BACKGROUND│         │BACKGROUND│
    │ (has TTY)│         │(buffered)│         │(buffered)│
    └──────────┘         └──────────┘         └──────────┘
         │                    │                    │
         ▼                    ▼                    ▼
    ┌──────────┐         ┌──────────┐         ┌──────────┐
    │   LLM    │         │   LLM    │         │   LLM    │
    │ Worker   │         │ Worker   │         │ Worker   │
    └──────────┘         └──────────┘         └──────────┘
```

## Design Components

### 1. Session Worker (New)

**Purpose**: Runs LLM interactions independently of terminal

```rust
pub struct SessionWorker {
    conversation: ConversationState,
    os: Os,  // Each worker gets its own Os instance
    
    // Communication channels
    input_rx: mpsc::Receiver<UserInput>,
    output_tx: mpsc::Sender<AssistantOutput>,
    control_rx: mpsc::Receiver<WorkerControl>,
    
    // State
    state: WorkerState,
}

enum WorkerState {
    Idle,
    Processing,
    WaitingForToolApproval,
    Error(String),
}

enum WorkerControl {
    Pause,
    Resume,
    Cancel,
    Shutdown,
}

struct UserInput {
    message: String,
    timestamp: Instant,
}

struct AssistantOutput {
    content: OutputChunk,
    timestamp: Instant,
}

enum OutputChunk {
    Text(String),
    ToolUse(ToolUseInfo),
    ToolResult(ToolResultInfo),
    Complete,
    Error(String),
}
```

**Key Features**:
- Runs in separate tokio task
- Independent of terminal I/O
- Communicates via channels
- Can be paused/resumed
- Buffers output when detached

### 2. Terminal Multiplexer (New)

**Purpose**: Routes terminal I/O to active session

```rust
pub struct TerminalMultiplexer {
    active_session_id: Option<String>,
    
    // Terminal state
    stdin: Stdin,
    stdout: Stdout,
    stderr: Stderr,
    
    // Session connections
    session_inputs: HashMap<String, mpsc::Sender<UserInput>>,
    session_outputs: HashMap<String, mpsc::Receiver<AssistantOutput>>,
    
    // Output buffers for background sessions
    buffers: HashMap<String, VecDeque<AssistantOutput>>,
}

impl TerminalMultiplexer {
    /// Attach terminal to a session
    async fn attach(&mut self, session_id: &str) -> Result<()>;
    
    /// Detach terminal from current session
    async fn detach(&mut self) -> Result<()>;
    
    /// Read input and send to active session
    async fn read_and_route(&mut self) -> Result<()>;
    
    /// Receive output from active session and display
    async fn receive_and_display(&mut self) -> Result<()>;
    
    /// Buffer output from background sessions
    async fn buffer_background_output(&mut self) -> Result<()>;
    
    /// Replay buffered output when attaching
    async fn replay_buffer(&mut self, session_id: &str) -> Result<()>;
}
```

### 3. Modified ManagedSession

```rust
pub struct ManagedSession {
    // Existing fields
    pub display: SessionDisplay,
    pub conversation_id: String,
    pub metadata: SessionMetadata,
    
    // New: Worker handle instead of ChatSession
    pub worker_handle: JoinHandle<()>,
    
    // Communication channels
    pub input_tx: mpsc::Sender<UserInput>,
    pub output_rx: mpsc::Receiver<AssistantOutput>,
    pub control_tx: mpsc::Sender<WorkerControl>,
    
    // State
    pub state: SessionState,
    pub output_buffer: VecDeque<AssistantOutput>,
    
    // Latest conversation state (synced periodically)
    pub conversation: ConversationState,
}
```

### 4. Modified Coordinator

```rust
impl MultiSessionCoordinator {
    pub async fn run(
        coord_arc: Arc<Mutex<Self>>,
        os: &mut Os,
    ) -> Result<()> {
        // Create terminal multiplexer
        let mut mux = TerminalMultiplexer::new(
            std::io::stdin(),
            std::io::stdout(),
            std::io::stderr(),
        );
        
        loop {
            let active_id = {
                let coord = coord_arc.lock().await;
                let state = coord.state.lock().await;
                state.active_session_id.clone()
            };
            
            if let Some(session_id) = active_id {
                // Attach terminal to active session
                mux.attach(&session_id).await?;
                
                // Run multiplexer loop until switch/quit
                loop {
                    tokio::select! {
                        // Route input to active session
                        _ = mux.read_and_route() => {},
                        
                        // Display output from active session
                        _ = mux.receive_and_display() => {},
                        
                        // Buffer output from background sessions
                        _ = mux.buffer_background_output() => {},
                        
                        // Check for session switch
                        _ = Self::check_for_switch(&coord_arc) => {
                            mux.detach().await?;
                            break;
                        }
                    }
                }
            }
        }
    }
    
    async fn create_session_worker(
        &mut self,
        config: SessionConfig,
        context: SessionContext,
    ) -> Result<String> {
        // Create channels
        let (input_tx, input_rx) = mpsc::channel(100);
        let (output_tx, output_rx) = mpsc::channel(100);
        let (control_tx, control_rx) = mpsc::channel(10);
        
        // Create worker
        let worker = SessionWorker::new(
            context.conversation_id.clone(),
            context,
            input_rx,
            output_tx,
            control_rx,
        );
        
        // Spawn worker task
        let worker_handle = tokio::spawn(async move {
            worker.run().await
        });
        
        // Create managed session
        let managed = ManagedSession {
            display: SessionDisplay::new(config.session_type, config.name),
            conversation_id: context.conversation_id.clone(),
            worker_handle,
            input_tx,
            output_rx,
            control_tx,
            state: SessionState::Idle,
            output_buffer: VecDeque::new(),
            conversation: ConversationState::new(...),
            metadata: SessionMetadata::new(),
        };
        
        // Store session
        let mut state = self.state.lock().await;
        state.sessions.insert(context.conversation_id.clone(), managed);
        
        Ok(context.conversation_id)
    }
}
```

## Implementation Phases

### Phase 1: Separate LLM Logic from Terminal I/O
**Goal**: Decouple conversation logic from terminal

- [ ] Extract LLM interaction into `SessionWorker`
- [ ] Create channel-based communication
- [ ] Move terminal I/O to `TerminalMultiplexer`
- [ ] Test single session with new architecture

**Files to modify**:
- `crates/chat-cli/src/cli/chat/session_worker.rs` (new)
- `crates/chat-cli/src/cli/chat/terminal_mux.rs` (new)
- `crates/chat-cli/src/cli/chat/coordinator.rs`

### Phase 2: Background Execution
**Goal**: Sessions continue when detached

- [ ] Implement output buffering
- [ ] Add pause/resume for workers
- [ ] Handle tool approval in background
- [ ] Sync conversation state periodically

**Files to modify**:
- `crates/chat-cli/src/cli/chat/session_worker.rs`
- `crates/chat-cli/src/cli/chat/managed_session.rs`

### Phase 3: Seamless Switching
**Goal**: Switch sessions without interruption

- [ ] Implement attach/detach in multiplexer
- [ ] Replay buffered output on attach
- [ ] Handle input during switch
- [ ] Clear screen and show history

**Files to modify**:
- `crates/chat-cli/src/cli/chat/terminal_mux.rs`
- `crates/chat-cli/src/cli/chat/coordinator.rs`

### Phase 4: Tool Approval in Background
**Goal**: Handle tool approvals across sessions

- [ ] Queue tool approvals
- [ ] Show pending approvals on switch
- [ ] Allow approval from any session
- [ ] Timeout for unapproved tools

**Files to modify**:
- `crates/chat-cli/src/cli/chat/session_worker.rs`
- `crates/chat-cli/src/cli/chat/tools/mod.rs`

## Key Challenges & Solutions

### Challenge 1: Os not Send
**Problem**: Can't share Os between threads

**Solution**: Each SessionWorker gets its own Os instance
```rust
// In coordinator
let os_clone = os.clone();  // Clone Os for worker
let worker = SessionWorker::new(os_clone, ...);
```

### Challenge 2: Terminal Control
**Problem**: Only one session can read stdin

**Solution**: TerminalMultiplexer owns stdin, routes to active session
```rust
// Multiplexer reads input
let input = mux.read_line().await?;

// Route to active session
if let Some(tx) = mux.session_inputs.get(&active_id) {
    tx.send(UserInput { message: input, ... }).await?;
}
```

### Challenge 3: Output Buffering
**Problem**: Background sessions produce output no one sees

**Solution**: Buffer output, replay on attach
```rust
// Background session sends output
output_tx.send(AssistantOutput { content, ... }).await?;

// Multiplexer buffers it
if session_id != active_id {
    buffers.entry(session_id).or_default().push_back(output);
}

// Replay on attach
for output in buffers.get(&session_id).unwrap() {
    display_output(output).await?;
}
```

### Challenge 4: Tool Approval
**Problem**: Background session needs tool approval

**Solution**: Pause worker, notify user, resume after approval
```rust
// Worker encounters tool use
if needs_approval {
    control_tx.send(WorkerControl::Pause).await?;
    output_tx.send(AssistantOutput::ToolUse(info)).await?;
    
    // Wait for approval
    let approval = approval_rx.recv().await?;
    
    if approval {
        control_tx.send(WorkerControl::Resume).await?;
    }
}
```

## Benefits

1. **True Background Execution**: LLM continues processing while switched away
2. **No Context Loss**: Full conversation state maintained
3. **Seamless Switching**: Switch during LLM response
4. **Output Preservation**: See what happened while away
5. **Multiple Active Sessions**: All sessions can be processing simultaneously

## Migration Path

1. **Phase 1**: New sessions use worker architecture, old code still works
2. **Phase 2**: Gradually migrate features to worker model
3. **Phase 3**: Remove old ChatSession.spawn() once stable
4. **Phase 4**: Full background execution enabled

## Testing Strategy

1. **Unit Tests**: Test SessionWorker in isolation
2. **Integration Tests**: Test multiplexer with multiple sessions
3. **Manual Tests**: 
   - Start session, switch during LLM response
   - Background session completes, switch back, see output
   - Tool approval in background session
   - Multiple sessions processing simultaneously

## Estimated Effort

- Phase 1: 2-3 days (foundation)
- Phase 2: 2-3 days (background execution)
- Phase 3: 1-2 days (switching)
- Phase 4: 1-2 days (tool approval)

**Total**: ~1-2 weeks for full implementation

## Next Steps

1. Review and approve design
2. Create Phase 1 implementation plan
3. Implement SessionWorker
4. Implement TerminalMultiplexer
5. Test with single session
6. Proceed to Phase 2
