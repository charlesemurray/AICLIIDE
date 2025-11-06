# Session Switch UX Improvements

## Current State (After Basic Fix)

When you switch from session A to session B:

1. ❌ Screen is NOT cleared - old messages from session A remain visible
2. ❌ Session B's conversation history is NOT displayed
3. ❌ Welcome message appears every time you switch to a session
4. ❌ User sees old session's messages mixed with new session's prompt
5. ✅ Functionally works - messages go to correct session
6. ✅ Prompt shows correct session name

## Problems

### Visual Confusion
```
(session-a) > What is AWS?
> AWS is Amazon Web Services...

(session-a) > /sessions switch session-b

✓ Switched to 'session-b'

🤖 Welcome to Amazon Q Developer CLI!  ← Confusing: shows welcome again

(session-b) >  ← User sees session-a's messages above
```

### Lost Context
- User switches to session-b which has 10 previous messages
- None of those messages are visible
- User has no context about what was discussed
- Must remember or check `/sessions history`

## Required Improvements

### 1. Clear Screen on Switch
**File**: `crates/chat-cli/src/cli/chat/mod.rs`
**Location**: `from_conversation()` or start of `spawn()`

```rust
// Clear screen when switching to a session
execute!(
    self.stderr,
    terminal::Clear(terminal::ClearType::All),
    cursor::MoveTo(0, 0)
)?;
```

### 2. Detect Existing Conversation
**File**: `crates/chat-cli/src/cli/chat/mod.rs`
**Location**: `from_conversation()` line ~1395

**Current**:
```rust
existing_conversation: false,  // Always false
```

**Should be**:
```rust
existing_conversation: !conversation.history().is_empty(),
```

This will:
- Show "Resuming conversation..." instead of welcome text
- Skip tips and greeting for existing sessions

### 3. Display Conversation History
**File**: `crates/chat-cli/src/cli/chat/mod.rs`
**Location**: `spawn()` after greeting, before main loop

**Add**:
```rust
// Display conversation history when resuming
if self.existing_conversation && !self.conversation.history().is_empty() {
    execute!(
        self.stderr,
        style::Print("─".repeat(80).dark_grey()),
        style::Print("\n"),
        StyledText::secondary_fg(),
        style::Print("Previous conversation:\n"),
        StyledText::reset(),
    )?;
    
    // Show last N messages (e.g., last 5 exchanges)
    let history = self.conversation.history();
    let start_idx = history.len().saturating_sub(5);
    
    for entry in history.iter().skip(start_idx) {
        // Display user message
        execute!(
            self.stderr,
            style::Print("\n"),
            StyledText::brand_fg(),
            style::Print("> "),
            StyledText::reset(),
            style::Print(&entry.user.content),
            style::Print("\n"),
        )?;
        
        // Display assistant message (truncated if too long)
        let assistant_text = entry.assistant.content();
        let display_text = if assistant_text.len() > 500 {
            format!("{}... [truncated]", &assistant_text[..500])
        } else {
            assistant_text.to_string()
        };
        
        execute!(
            self.stderr,
            StyledText::success_fg(),
            style::Print("> "),
            StyledText::reset(),
            style::Print(&display_text),
            style::Print("\n"),
        )?;
    }
    
    execute!(
        self.stderr,
        style::Print("\n"),
        style::Print("─".repeat(80).dark_grey()),
        style::Print("\n\n"),
    )?;
}
```

### 4. Visual Separator for Switch
**File**: `crates/chat-cli/src/cli/chat/session_switcher.rs`
**Location**: After switch success message

**Add**:
```rust
// Clear screen and show switch banner
execute!(
    writer,
    terminal::Clear(terminal::ClearType::All),
    cursor::MoveTo(0, 0),
    StyledText::brand_fg(),
    style::Print("═".repeat(80)),
    style::Print("\n"),
    style::Print(format!("  Switched to session: {}\n", target_name)),
    style::Print("═".repeat(80)),
    style::Print("\n\n"),
    StyledText::reset(),
)?;
```

## Implementation Order

1. **Clear screen on switch** (easiest, immediate improvement)
2. **Detect existing conversation** (simple boolean fix)
3. **Visual separator** (polish)
4. **Display history** (most complex, needs formatting)

## Alternative: Minimal Approach

If full history display is too complex, at minimum:

```rust
// In spawn(), after greeting check
if self.existing_conversation {
    let msg_count = self.conversation.history().len();
    execute!(
        self.stderr,
        StyledText::secondary_fg(),
        style::Print(format!("Resuming conversation ({} previous messages)\n", msg_count)),
        style::Print("Use /history to view previous messages\n\n"),
        StyledText::reset(),
    )?;
}
```

## Testing Checklist

- [ ] Switch to new session - should show welcome
- [ ] Switch to existing session - should NOT show welcome
- [ ] Switch to existing session - should show history or message count
- [ ] Screen should be clear/separated on switch
- [ ] Prompt shows correct session name
- [ ] Messages go to correct session
- [ ] History is preserved across switches

## Files to Modify

1. `crates/chat-cli/src/cli/chat/mod.rs` - from_conversation(), spawn()
2. `crates/chat-cli/src/cli/chat/session_switcher.rs` - switch_to()
3. Possibly add `/history` command to view full conversation history
