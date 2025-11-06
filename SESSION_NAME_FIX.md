# Session Name Display Fix

## Problem
After implementing Phase 2 (pause/resume + coordinator main loop), session switching still showed the wrong session name in the prompt:

```
(lively-dolphin) > /switch hello
✓ Switched to 'hello'
(lively-dolphin) >  # ← Wrong! Should show (hello)
```

## Root Cause
The `generate_tool_trust_prompt()` method had a fallback logic that prioritized `self.active_session_name` (cached value) over looking up the current session name from the coordinator based on `conversation_id`.

When switching sessions:
1. `switch_conversation()` updates `self.conversation` to the new conversation
2. `self.active_session_name` is set to the new name
3. But on the NEXT prompt, the cached `active_session_name` might be stale or the conversation_id lookup wasn't being used

## Solution
Changed the session name lookup priority in `generate_tool_trust_prompt()`:

**Before:**
```rust
let session_name = if let Some(ref name) = self.active_session_name {
    Some(name.clone())  // Use cached value first
} else if let Some(ref coord) = self.coordinator {
    coord_lock.get_session_name(self.conversation.conversation_id())  // Fallback to lookup
} else {
    None
};
```

**After:**
```rust
let session_name = if let Some(ref coord) = self.coordinator {
    coord_lock.get_session_name(self.conversation.conversation_id())  // Always lookup by conversation_id
} else {
    self.active_session_name.clone()  // Fallback to cached value
};
```

## Impact
- Session name in prompt now always reflects the current conversation_id
- Switching sessions immediately shows the correct name in the next prompt
- No caching issues or stale values

## Files Modified
- `crates/chat-cli/src/cli/chat/mod.rs` - Fixed `generate_tool_trust_prompt()` method

## Testing
1. Start chat: `q chat`
2. Create new session: `/sessions new hello`
3. Switch to it: `/switch hello`
4. Verify prompt shows: `(hello) >`
5. Switch back: `/switch lively-dolphin`
6. Verify prompt shows: `(lively-dolphin) >`
