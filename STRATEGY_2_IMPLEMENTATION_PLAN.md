# Strategy 2 Implementation Plan
## Background LLM Processing

## Code Analysis Summary

### Current Architecture
```
ChatSession.spawn() 
  └─ loop: next() 
      ├─ PromptUser → read_line() [BLOCKS HERE]
      ├─ HandleInput → prepare message
      ├─ HandleResponseStream → send_message()
      │   └─ handle_response()
      │       └─ rx.recv() [streams LLM response]
      └─ ExecuteTools → run tools
```

### Key Findings

1. **LLM Call is Already Async**
   - `send_message()` returns `SendMessageStream`
   - `rx.recv().await` streams chunks
   - Already non-blocking!

2. **The Only Blocker**
   - `input_source.read_line()` in `PromptUser` state
   - Uses rustyline which blocks

3. **State Machine**
   - ChatSession uses `ChatState` enum
   - Already has state transitions
   - Easy to add switch detection

4. **Streaming Already Works**
   - `handle_response()` processes chunks in loop
   - Can check for switch signal in this loop

## Revised Assessment

### Good News
**The LLM processing is ALREADY non-blocking!** We just need to:
1. Add switch detection during streaming
2. Save partial responses
3. Resume on switch back

### Complexity: MUCH EASIER Than Expected

**Original estimate: 2 weeks**
**Revised estimate: 3-4 days**

## Implementation Plan

### Phase 1: Add Switch Detection (Day 1 - 8 hours)

#### 1.1 Add switch checking method
```rust
// crates/chat-cli/src/cli/chat/mod.rs
impl ChatSession {
    fn should_switch(&self) -> bool {
        if let Some(ref coord) = self.coordinator {
            // Check if active session changed
            let coord = coord.try_lock().ok()?;
            let state = coord.state.try_lock().ok()?;
            let current_id = self.conversation.conversation_id();
            state.active_session_id.as_ref() != Some(&current_id.to_string())
        } else {
            false
        }
    }
}
```

**Files**: `mod.rs`
**Lines**: ~15 new

#### 1.2 Check during streaming
```rust
// In handle_response(), modify the recv loop:
loop {
    // Check for switch before receiving next chunk
    if self.should_switch() {
        // Save partial response
        self.conversation.save_partial_response(&buf)?;
        return Ok(ChatState::SwitchSession { 
            target_id: self.get_target_session_id()? 
        });
    }
    
    match rx.recv().await {
        // ... existing code
    }
}
```

**Files**: `mod.rs` 
**Lines**: ~10 modified

### Phase 2: Partial Response Handling (Day 2 - 8 hours)

#### 2.1 Add partial response storage
```rust
// crates/chat-cli/src/cli/chat/conversation.rs
impl ConversationState {
    pub fn save_partial_response(&mut self, text: &str) -> Result<()> {
        // Store in a temporary field
        self.partial_response = Some(text.to_string());
        Ok(())
    }
    
    pub fn has_partial_response(&self) -> bool {
        self.partial_response.is_some()
    }
    
    pub fn take_partial_response(&mut self) -> Option<String> {
        self.partial_response.take()
    }
}
```

**Files**: `conversation.rs`
**Lines**: ~20 new, add field to struct

#### 2.2 Resume partial response
```rust
// In handle_response(), at start:
if let Some(partial) = self.conversation.take_partial_response() {
    // Display what we already had
    write!(self.stdout, "{}", partial)?;
    buf = partial;
    // Continue streaming from where we left off
}
```

**Files**: `mod.rs`
**Lines**: ~5 modified

### Phase 3: Coordinator Integration (Day 3 - 8 hours)

#### 3.1 Modify coordinator loop
```rust
// crates/chat-cli/src/cli/chat/coordinator.rs
// In run(), modify the session spawn handling:

match session.spawn(os).await {
    Ok(_) => {
        // Check if it was a switch
        let new_active = state.active_session_id.clone();
        if new_active != initial_active_id {
            // Switch happened, continue loop
            continue;
        }
    }
}
```

**Files**: `coordinator.rs`
**Lines**: Already mostly done, ~5 lines to verify

### Phase 4: Testing & Polish (Day 4 - 8 hours)

#### 4.1 Test scenarios
- [ ] Switch during LLM streaming
- [ ] Switch back and resume
- [ ] Multiple switches
- [ ] Tool use during switch
- [ ] Error handling

#### 4.2 Edge cases
- [ ] Switch during tool execution
- [ ] Switch with pending tool approval
- [ ] Rapid switching
- [ ] Switch to non-existent session

## Detailed File Changes

### File 1: `crates/chat-cli/src/cli/chat/mod.rs`

**Changes needed:**
1. Add `should_switch()` method (~15 lines)
2. Modify `handle_response()` to check for switch (~10 lines)
3. Handle partial response resume (~5 lines)

**Total: ~30 lines modified/added**

### File 2: `crates/chat-cli/src/cli/chat/conversation.rs`

**Changes needed:**
1. Add `partial_response: Option<String>` field to struct
2. Add `save_partial_response()` method (~8 lines)
3. Add `has_partial_response()` method (~3 lines)
4. Add `take_partial_response()` method (~3 lines)

**Total: ~15 lines added**

### File 3: `crates/chat-cli/src/cli/chat/coordinator.rs`

**Changes needed:**
1. Verify switch detection logic (already mostly done)
2. Add debug logging (~5 lines)

**Total: ~5 lines modified**

## Total Code Changes

- **New code**: ~50 lines
- **Modified code**: ~20 lines
- **Files touched**: 3
- **Total effort**: 3-4 days

## Why This is Much Easier

### 1. LLM Already Async
The streaming is already non-blocking. We just need to check for switches.

### 2. State Machine Exists
ChatState enum already handles transitions. Just add switch detection.

### 3. Minimal Changes
- No new files needed
- No architecture changes
- Just add checks in existing loops

### 4. Low Risk
- Existing code unchanged
- Switch detection is additive
- Easy to test incrementally

## Implementation Steps

### Day 1: Switch Detection
1. Add `should_switch()` method
2. Add switch check in `handle_response()` loop
3. Test basic switch detection

### Day 2: Partial Responses
1. Add partial response fields/methods
2. Implement save on switch
3. Implement resume on switch back
4. Test partial response handling

### Day 3: Integration
1. Verify coordinator loop handles switches
2. Add debug logging
3. Test end-to-end switching

### Day 4: Testing & Polish
1. Test all scenarios
2. Handle edge cases
3. Add error handling
4. Documentation

## What You Get

### After Implementation
✅ Switch during LLM streaming (main goal)
✅ Partial responses preserved
✅ Resume where you left off
✅ All existing features work
❌ Still can't switch during readline() (acceptable)

### Limitations
- Background sessions don't process automatically
- Must return to session to continue
- No visual session list

### Future Enhancements (Optional)
- Add background message queue
- Process messages when session inactive
- Show "session has updates" indicator

## Risk Assessment

### Low Risk Because:
1. **Additive changes**: Not removing/replacing code
2. **Small scope**: ~70 lines total
3. **Existing patterns**: Using existing state machine
4. **Easy rollback**: Can revert easily
5. **Incremental**: Can test each phase

### Potential Issues:
1. **Race conditions**: Coordinator lock timing
   - **Mitigation**: Use try_lock(), handle None gracefully
2. **Partial response corruption**: Incomplete state
   - **Mitigation**: Save atomically, validate on resume
3. **Tool use during switch**: Tools mid-execution
   - **Mitigation**: Check switch before tool execution too

## Success Criteria

### Must Have:
- [ ] Can switch during LLM streaming
- [ ] Partial response preserved
- [ ] Resume works correctly
- [ ] No crashes or hangs

### Nice to Have:
- [ ] Smooth visual transition
- [ ] Clear user feedback
- [ ] Debug logging for troubleshooting

## Conclusion

**Original estimate: 2 weeks (80 hours)**
**Revised estimate: 3-4 days (24-32 hours)**

**Why the difference?**
- LLM is already async (no worker needed)
- State machine exists (no rewrite needed)
- Just add switch detection (minimal code)

**This is MUCH easier than expected. Let's do it!**
