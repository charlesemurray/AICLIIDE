# SlashCommand Integration Audit

## Current State

### Two Parallel Command Systems

**System 1: Primitive String Matching (Line 2743)**
```rust
if user_input.starts_with("/sessions")
    || user_input.starts_with("/switch")
    || user_input.starts_with("/new")
    || user_input.starts_with("/close")
    || user_input.starts_with("/rename")
    || user_input.starts_with("/session-name")
{
    // Routes to session_integration::handle_session_command()
    // Uses coordinator
}
```

**System 2: SlashCommand Enum (Line 3090)**
```rust
match SlashCommand::try_parse_from(args) {
    Ok(command) => command.execute(os, self).await,
    // Includes telemetry, error handling, help text
}
```

### Execution Flow

```
User Input (line 2738)
    ↓
Session Commands Check (line 2743) ← INTERCEPTS /sessions, /switch, /new, etc.
    ↓ (if not session command)
/worktree Check (line 2782)
    ↓ (if not worktree)
Clipboard Paste Handling (line 2820)
    ↓
Skill Execution (line 2900)
    ↓
Pending Prompts (line 3000)
    ↓
Conversation Mode Commands (line 3070)
    ↓
SlashCommand Parsing (line 3090) ← NEVER REACHED for session commands
```

## Problems with Current Approach

### 1. Bypasses SlashCommand Infrastructure
- ❌ No automatic help text generation
- ❌ No telemetry tracking
- ❌ No error handling consistency
- ❌ No command validation
- ❌ Manual string parsing instead of clap

### 2. Duplicate Command Definitions
- `SlashCommand::Switch` exists but does nothing (line 220)
- Session commands defined in both:
  - `SessionCommand` enum (input_router.rs)
  - String matching (mod.rs line 2743)

### 3. Inconsistent with Other Commands
- `/skills`, `/memory`, `/agent` use SlashCommand
- `/sessions`, `/switch`, `/new` use primitive matching

### 4. Missing Features
- No `--help` for individual commands
- No command aliases (e.g., `/s` for `/switch` works but not documented)
- No tab completion integration

## SlashCommand System Analysis

### How It Works

1. **Parsing** (line 3090)
   ```rust
   SlashCommand::try_parse_from(args) // Uses clap
   ```

2. **Execution** (line 162)
   ```rust
   async fn execute(self, os: &mut Os, session: &mut ChatSession) -> Result<ChatState, ChatError>
   ```

3. **Telemetry** (line 3095)
   ```rust
   send_slash_command_telemetry(command_name, subcommand_name, result)
   ```

### Existing Session-Related Commands

**SlashCommand::Switch** (line 139)
```rust
Switch {
    name: String,
}
```
- Currently just prints dummy message
- Not connected to coordinator

**SlashCommand::SessionMgmt** (line 133)
```rust
SessionMgmt(SessionMgmtArgs)
```
- Different from our session commands
- Uses `/session` (singular) not `/sessions`

## Integration Challenges

### Challenge 1: Context Factory Access

**Problem:** SlashCommand::execute() signature:
```rust
async fn execute(self, os: &mut Os, session: &mut ChatSession)
```

**Need:** Context factory for session creation:
```rust
|| SessionContext {
    conversation_id: uuid::Uuid::new_v4().to_string(),
    os: os.clone(),
    agents: session.conversation.agents.clone(),
    tool_config: HashMap::new(),
    tool_manager: session.conversation.tool_manager.clone(),
    model_id: None,
}
```

**Solution Options:**
1. Add method to ChatSession: `session.build_session_context(os)`
2. Pass coordinator reference through execute()
3. Store context factory in ChatSession

### Challenge 2: Coordinator Access

**Problem:** Coordinator is in `session.coordinator: Option<Arc<Mutex<MultiSessionCoordinator>>>`

**Current:** Direct access at line 2753
```rust
if let Some(ref coord) = self.coordinator {
    let mut coord_lock = coord.lock().await;
    // Use coordinator
}
```

**In SlashCommand:** Would need to access through session
```rust
impl SlashCommand {
    async fn execute(self, os: &mut Os, session: &mut ChatSession) {
        match self {
            Self::Sessions(subcommand) => {
                if let Some(ref coord) = session.coordinator {
                    // Access coordinator
                }
            }
        }
    }
}
```

### Challenge 3: Command Structure

**Current SessionCommand enum:**
```rust
pub enum SessionCommand {
    List { all: bool, waiting: bool },
    Switch(String),
    New { session_type: Option<SessionType>, name: Option<String> },
    Close(Option<String>),
    Rename(String),
    SessionName(Option<String>),
}
```

**Would need SlashCommand variant:**
```rust
#[derive(Parser)]
pub enum SlashCommand {
    /// Manage sessions
    #[command(subcommand)]
    Sessions(SessionsSubcommand),
    
    /// Switch to a session (already exists!)
    Switch { name: String },
}

#[derive(Parser)]
pub enum SessionsSubcommand {
    /// List sessions
    List {
        #[arg(long)]
        all: bool,
        #[arg(long)]
        waiting: bool,
    },
    /// Create new session
    New {
        name: Option<String>,
        #[arg(long)]
        session_type: Option<SessionType>,
    },
    // etc.
}
```

### Challenge 4: Early Command Handling

**Problem:** Line 343 handles commands before ChatSession exists
```rust
if let (Some(coord), Some(inp)) = (&mut coordinator, &input) {
    if inp.starts_with("/sessions") || inp.starts_with("/switch") {
        // Handle before ChatSession creation
    }
}
```

**SlashCommand parsing requires ChatSession** for execute()

### Challenge 5: Backward Compatibility

**Current users expect:**
- `/sessions` - works
- `/sessions --waiting` - works
- `/switch name` - works
- `/new name` - works

**Must maintain exact same behavior**

## Recommended Approach

### Option A: Minimal Integration (Easiest)

Keep current system but add SlashCommand variants that delegate:

```rust
impl SlashCommand {
    async fn execute(self, os: &mut Os, session: &mut ChatSession) {
        match self {
            Self::Switch { name } => {
                // Delegate to existing session_integration
                if let Some(ref coord) = session.coordinator {
                    let mut coord_lock = coord.lock().await;
                    let context_factory = || session.build_session_context(os);
                    session_integration::handle_session_command(
                        &format!("/switch {}", name),
                        &mut coord_lock,
                        &mut session.stderr,
                        context_factory,
                    ).await?;
                }
                Ok(ChatState::PromptUser { skip_printing_tools: false })
            }
        }
    }
}
```

**Pros:**
- Minimal changes
- Keeps existing logic
- Adds SlashCommand benefits (help, telemetry)

**Cons:**
- Still have two systems
- Delegation overhead

### Option B: Full Migration (Cleanest)

1. Add `Sessions` subcommand to SlashCommand
2. Remove primitive string matching
3. Implement execute() methods
4. Add `build_session_context()` to ChatSession

**Pros:**
- Single command system
- Consistent with other commands
- Full clap benefits

**Cons:**
- More work
- Need to handle early command case
- Risk of breaking existing behavior

### Option C: Hybrid (Recommended)

1. Keep primitive matching for now (it works)
2. Add SlashCommand variants that work the same way
3. Both routes go to same coordinator code
4. Deprecate primitive matching later

**Pros:**
- Gradual migration
- No breaking changes
- Can test both paths

**Cons:**
- Temporary duplication

## Effort Estimate

### Option A: 2-3 hours
- Add helper method to ChatSession
- Update SlashCommand::Switch
- Add Sessions subcommand
- Test both paths work

### Option B: 6-8 hours
- Full refactor
- Remove primitive matching
- Handle edge cases
- Extensive testing

### Option C: 3-4 hours
- Add SlashCommand variants
- Keep both systems
- Add deprecation warnings

## Recommendation

**Start with Option C (Hybrid):**
1. Add `build_session_context()` helper to ChatSession
2. Add `Sessions` subcommand to SlashCommand
3. Update `Switch` to use coordinator
4. Keep primitive matching as fallback
5. Add telemetry to track which path is used
6. Migrate fully in next iteration

This gives us:
- ✅ Proper command system integration
- ✅ No breaking changes
- ✅ Gradual migration path
- ✅ Telemetry to verify behavior
