# Session Numbering System

## Overview
Implemented a numbering system for sessions that allows users to reference and switch between sessions using simple numbers (1, 2, 3, etc.) instead of names. Numbers are automatically assigned based on creation order and renumbered when sessions are closed.

## Changes Made

### 1. Added Session Order Tracking
**File**: `crates/chat-cli/src/cli/chat/coordinator.rs`

Added `session_order: Vec<String>` to `SessionState`:
- Maintains ordered list of session IDs
- Determines session numbers (1-indexed)
- Automatically updated on create/close

### 2. Session Creation
When a session is created:
- Session ID is appended to `session_order`
- Position in vector determines its number (index + 1)

### 3. Session Closing with Renumbering
When a session is closed:
- Session ID is removed from `session_order` using `retain()`
- Remaining sessions automatically renumber (no gaps)
- Example: Close session 2 → sessions 3,4,5 become 2,3,4

### 4. Number-Based Switching
**Modified**: `switch_session()` method

Now accepts both names and numbers:
- Tries to parse input as number first
- If number: looks up session by position in `session_order`
- If not number: falls back to name/ID lookup
- Validates number is in range (1 to session count)

### 5. Enhanced Session Listing
**Added**: `list_sessions_with_numbers()` method

Returns: `Vec<(usize, String, bool)>`
- `usize`: Session number (1-indexed)
- `String`: Session name
- `bool`: Is active session

**Updated**: `session_switcher.rs` `list_sessions()`
- Displays sessions with numbers: `[1] session-name *`
- Active session marked with `*`
- Shows sessions in creation order

### 6. Helper Methods
Added utility methods:
- `get_session_id_by_number(number: usize)` - Get session ID from number
- `get_session_number(id: &str)` - Get number from session ID

## Usage Examples

### List Sessions with Numbers
```bash
/sessions list

Active Sessions:
  [1] default *
  [2] feature-work
  [3] debugging
```

### Switch by Number
```bash
# Switch to session 2
/sessions switch 2

# Or use /switch shorthand (if implemented)
/switch 3
```

### Switch by Name (Still Works)
```bash
/sessions switch feature-work
```

### Automatic Renumbering
```bash
# Initial state
[1] session-a *
[2] session-b
[3] session-c
[4] session-d

# Close session 2
/close session-b

# After close - automatic renumbering
[1] session-a *
[2] session-c
[3] session-d
```

## Technical Details

### Number Assignment
- Numbers are 1-indexed (user-friendly)
- Internally stored as 0-indexed Vec positions
- Conversion: `number = index + 1`

### Renumbering Algorithm
```rust
// Remove from order
state.session_order.retain(|id| id != &target_id);

// Remaining sessions automatically renumber
// No explicit renumbering needed - position determines number
```

### Number Validation
```rust
if num == 0 || num > state.session_order.len() {
    bail!("Session number {} out of range (1-{})", num, state.session_order.len());
}
```

### Switch Logic
```rust
// Try parse as number
if let Ok(num) = name.parse::<usize>() {
    // Get by position
    state.session_order.get(num - 1)
} else {
    // Find by name
    state.sessions.iter().find(|(_, s)| s.display.name == name)
}
```

## Benefits

1. **Faster Switching**: Type `2` instead of full session name
2. **No Gaps**: Numbers always sequential (1, 2, 3...)
3. **Visual Clarity**: Easy to see session count at a glance
4. **Backward Compatible**: Name-based switching still works
5. **Predictable**: Creation order preserved

## Edge Cases Handled

1. **Number 0**: Rejected (numbers start at 1)
2. **Out of Range**: Clear error message with valid range
3. **Empty Sessions**: List shows "No active sessions"
4. **Single Session**: Shows `[1] session-name *`
5. **Close Last Session**: New session gets number 1

## Testing Recommendations

1. **Create multiple sessions**: Verify sequential numbering
2. **Switch by number**: Test `/sessions switch 2`
3. **Close middle session**: Verify renumbering (no gaps)
4. **Close first session**: Verify all shift down
5. **Close last session**: Verify auto-create gets number 1
6. **Invalid numbers**: Test 0, negative, out of range
7. **Name still works**: Verify `/sessions switch name` works
8. **List display**: Check numbers and active marker

## Future Enhancements

1. **Shorthand switch**: `/s 2` or `/2` for quick switching
2. **Relative switching**: `/next`, `/prev`, `/+1`, `/-1`
3. **Session history**: Remember recently used sessions
4. **Pinned sessions**: Keep certain numbers fixed
5. **Custom numbering**: Allow user-defined numbers

## Related Files
- `crates/chat-cli/src/cli/chat/coordinator.rs` - Core numbering logic
- `crates/chat-cli/src/cli/chat/session_switcher.rs` - Display with numbers
- `crates/chat-cli/src/cli/chat/cli/mod.rs` - Command parsing
