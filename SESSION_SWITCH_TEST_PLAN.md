# Session Switch Test Plan

## Basic Functionality Tests

### Test 1: Create and Switch to New Session
```bash
q chat
> /sessions new hello
> /sessions switch hello
> test message in hello
> /sessions switch back to first session
```

**Expected**:
- ✅ New session created
- ✅ Switch succeeds
- ✅ Prompt shows (hello)
- ✅ Message goes to hello session
- ✅ Can switch back
- ✅ No panics or crashes

### Test 2: Multiple Sessions
```bash
q chat
> /sessions new session-a
> /sessions new session-b
> /sessions new session-c
> /sessions list
> /sessions switch session-b
> message in b
> /sessions switch session-c
> message in c
> /sessions switch session-a
> message in a
```

**Expected**:
- ✅ All sessions created
- ✅ Can switch between any sessions
- ✅ Messages go to correct sessions
- ✅ No resource conflicts

### Test 3: Session with History
```bash
q chat
> message 1 in first session
> message 2 in first session
> /sessions new second
> /sessions switch second
> message in second session
> /sessions switch back to first
> message 3 in first session
```

**Expected**:
- ✅ First session preserves history
- ✅ Second session starts fresh
- ✅ Switching back to first shows correct context

### Test 4: Rapid Switching
```bash
> /sessions new a
> /sessions new b
> /sessions switch a
> /sessions switch b
> /sessions switch a
> /sessions switch b
```

**Expected**:
- ✅ No crashes
- ✅ No terminal corruption
- ✅ Prompt always shows correct session

## Edge Cases

### Test 5: Switch to Non-Existent Session
```bash
> /sessions switch doesnotexist
```

**Expected**:
- ✅ Error message
- ✅ Stays in current session
- ✅ No crash

### Test 6: Ctrl-C During Switch
```bash
> /sessions switch hello
[Press Ctrl-C immediately]
```

**Expected**:
- ✅ Graceful handling
- ✅ Either completes switch or stays in current session
- ✅ No zombie processes

### Test 7: Long-Running Tool in Session
```bash
> /sessions new background
> /sessions switch background
> run some long command
[While it's running]
> /sessions switch main
```

**Expected**:
- ✅ Switch succeeds
- ✅ Background command is interrupted
- ✅ Main session is usable

## Debug Output to Watch For

During testing, look for these debug messages:

1. `[DEBUG] SessionsSubcommand::execute() - command: Switch { name: "..." }`
2. `[DEBUG] switch_session called with name: ...`
3. `[DEBUG] Found target session ID: ...`
4. `[DEBUG] Returning SwitchSession with target_id: ...`
5. `[DEBUG] ChatState::SwitchSession handler - target_id: ...`
6. `[DEBUG] Session exited. Old ID: ..., New active ID: ...`
7. `[DEBUG] Active session changed, continuing loop`
8. `[DEBUG] Creating NEW ChatSession for session: ...` OR `[DEBUG] REUSING existing ChatSession for session: ...`

## Success Criteria

- ✅ No panics or crashes
- ✅ Prompt shows correct session name
- ✅ Messages go to correct session
- ✅ Can switch between sessions multiple times
- ✅ No terminal corruption
- ✅ No resource leaks (check with `ps` for zombie processes)

## Known Limitations (Acceptable)

- ❌ No conversation history displayed when switching
- ❌ Welcome message shows every time
- ❌ Screen not cleared on switch
- ❌ Old session's messages remain visible

These are UX issues to fix later, not blockers.
