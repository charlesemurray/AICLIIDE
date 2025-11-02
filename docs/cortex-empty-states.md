# Cortex Memory - Empty States Design

## Empty State Catalog

### 1. First Use - No Memories Exist

**Context**: User tries to use memory features before any memories are stored

#### 1.1 First `/recall` Attempt

```
You: /recall Lambda deployment

[No memories stored yet]

Memory will automatically save your conversations.
Ask a few questions, then try /recall again!

Tips:
  • Memory saves after each Q response
  • Use /memory config to view settings
  • Use /help to see all commands

You: 
```

**Design Principles**:
- Explain how memory works
- Encourage natural usage
- Provide clear next steps
- Don't block the user

#### 1.2 First `/memory list` Attempt

```
You: /memory list

[No memories stored yet]

Memories are automatically saved as you chat with Q.

Start a conversation:
  • Ask Q a question
  • Get a response
  • Your conversation will be saved

Then use:
  • /memory list - See your memories
  • /recall <query> - Search conversations
  • /memory config - View settings

You: 
```

**Design Principles**:
- Educational, not blocking
- Show what will happen
- Provide examples

#### 1.3 First `/memory stats` Attempt

```
You: /memory stats

Memory Statistics:

Total Memories: 0
Storage: 0 MB / 100 MB (0%)
Sessions: 0

No memories stored yet.

Memory will automatically save your conversations as you use Q.
Start chatting to build your memory!

Configuration:
  Retention: 30 days
  Max Size: 100 MB
  Status: Enabled ✓

You: 
```

**Design Principles**:
- Show configuration even when empty
- Confirm memory is working
- Encourage usage

---

### 2. Empty Search Results

**Context**: User searches but no matches found

#### 2.1 No Matches in Current Session

```
You: /recall Kubernetes deployment

[No memories found matching "Kubernetes deployment"]

This session has no memories about "Kubernetes deployment".

Tips:
  • Try broader terms: /recall Kubernetes
  • Search all sessions: /recall --global Kubernetes
  • Ask Q about it to create a memory

You: 
```

**Design Principles**:
- Acknowledge the search
- Suggest alternatives
- Offer to help anyway

#### 2.2 No Matches Globally

```
You: /recall --global Kubernetes

▰▰▰▱▱▱▱ Searching all sessions...

[No memories found matching "Kubernetes"]

You haven't discussed "Kubernetes" with Q yet.

Would you like to ask about it now?
  • Q can help with Kubernetes questions
  • Your conversation will be saved for future reference

You: 
```

**Design Principles**:
- Searched everywhere, found nothing
- Offer to start the conversation
- Turn empty state into opportunity

#### 2.3 No Matches in Specific Session

```
You: /recall --session abc123 Lambda

[No memories found in session 'abc123' matching "Lambda"]

This session doesn't have memories about "Lambda".

Try:
  • /recall Lambda - Search current session
  • /recall --global Lambda - Search all sessions
  • /recall --list-sessions - See what's in each session

You: 
```

**Design Principles**:
- Specific about what was searched
- Offer alternatives
- Help user find what they need

---

### 3. Empty Session

**Context**: User tries to access a session with no memories

#### 3.1 Empty Session Recall

```
You: /recall --session xyz789 anything

[Session 'xyz789' has no memories]

This session hasn't stored any memories yet.

Sessions with memories:
  1. session-abc123 - "AWS Lambda deployment" (15 memories)
  2. session-def456 - "React patterns" (8 memories)

Use: /recall --session <number> <query>

You: 
```

**Design Principles**:
- Explain why empty
- Show sessions that do have memories
- Provide actionable alternatives

#### 3.2 Empty Session List

```
You: /memory list --session xyz789

[Session 'xyz789' has no memories]

This session hasn't stored any conversations yet.

To see memories from other sessions:
  • /memory list - Current session
  • /recall --list-sessions - All sessions with memories

You: 
```

**Design Principles**:
- Clear about which session
- Offer to see other sessions
- Don't dead-end the user

---

### 4. After Cleanup

**Context**: User just deleted all memories

#### 4.1 After Full Cleanup

```
You: /memory cleanup --all
⚠️  This will delete ALL 1,247 memories. Type 'yes' to confirm: yes

[Deleting memories...]
▰▰▰▰▰▰▰ 100% (1,247/1,247)

✓ Deleted 1,247 memories
✓ Freed 45.2 MB of storage

Memory database is now empty.
New memories will be saved as you continue using Q.

You: /memory list

[No memories stored]

You recently cleaned all memories.
New conversations will be saved automatically.

You: 
```

**Design Principles**:
- Confirm action completed
- Reassure user
- Explain what happens next

#### 4.2 After Partial Cleanup

```
You: /memory cleanup

[Cleaning up old memories...]
▰▰▰▰▰▰▰ 100% (127/127)

✓ Deleted 127 memories older than 30 days
✓ Freed 4.8 MB of storage

Remaining: 1,120 memories (40.4 MB)

You: /memory list --session old-session

[Session 'old-session' has no memories]

This session's memories were cleaned up (older than 30 days).

Active sessions with memories:
  • session-abc123 - 156 memories
  • session-xyz789 - 89 memories

You: 
```

**Design Principles**:
- Explain what was cleaned
- Show what remains
- Help user find active memories

---

### 5. Memory Disabled

**Context**: User tries to use memory features when disabled

#### 5.1 Recall When Disabled

```
You: /recall Lambda

[Memory is currently disabled]

Memory features are not available.

To enable memory:
  /memory toggle

To learn more:
  /memory config
  /help memory

You: 
```

**Design Principles**:
- Clear about state
- Show how to enable
- Provide help resources

#### 5.2 List When Disabled

```
You: /memory list

[Memory is currently disabled]

Memory features are not available.
Your existing memories are preserved but not accessible.

To enable memory:
  /memory toggle

You: 
```

**Design Principles**:
- Reassure data is safe
- Show how to re-enable
- Don't lose user's data

#### 5.3 Stats When Disabled

```
You: /memory stats

Memory Statistics:

Status: Disabled ✗

Memory features are currently disabled.
Your existing memories are preserved.

To enable memory:
  /memory toggle

To view configuration:
  /memory config

You: 
```

**Design Principles**:
- Show status clearly
- Reassure data preserved
- Provide next steps

---

### 6. No Sessions with Memories

**Context**: User tries to list sessions but none have memories

#### 6.1 List Sessions - None Found

```
You: /recall --list-sessions

[No sessions with memories found]

You haven't had any conversations with Q yet, or all memories
have been cleaned up.

Start chatting:
  • Ask Q a question
  • Your conversation will be saved
  • Then use /recall to search it

You: 
```

**Design Principles**:
- Explain why empty
- Encourage usage
- Show what will happen

---

## Empty State Design Principles

### 1. Be Helpful, Not Blocking
- Never just say "empty" and stop
- Always provide next steps
- Turn empty into opportunity

### 2. Educate Users
- Explain how the feature works
- Show what will happen when they use it
- Provide examples

### 3. Offer Alternatives
- Suggest related commands
- Show what is available
- Help user find what they need

### 4. Maintain Context
- Explain why empty (first use, cleaned, disabled)
- Show related state (config, other sessions)
- Keep user oriented

### 5. Encourage Action
- Invite user to try the feature
- Show benefits of using it
- Make next step clear

---

## Empty State Template

```
[State description]

Brief explanation of why empty.

What will happen:
  • Action 1
  • Action 2

Try:
  • Command 1 - Description
  • Command 2 - Description

You: 
```

---

## Implementation Checklist

### Phase 1 (Must Have)
- [x] First use - no memories
- [x] No search results
- [x] Empty session
- [x] Memory disabled

### Phase 2 (Should Have)
- [x] After cleanup
- [x] No sessions with memories
- [x] Empty stats display

### Phase 3 (Nice to Have)
- [ ] Contextual tips based on usage
- [ ] Progressive onboarding
- [ ] Smart suggestions

---

## Testing Empty States

### Manual Testing

```bash
# Test first use
rm -rf ~/.q/memory/
q chat
/recall anything
/memory list
/memory stats

# Test no results
q chat
/recall nonexistent-topic-xyz-123

# Test disabled
q settings set memory.enabled false
q chat
/recall anything

# Test after cleanup
q chat
/memory cleanup --all
/memory list
```

### Automated Testing

```rust
#[test]
fn test_empty_state_first_use() {
    let cortex = CortexMemory::new_empty();
    let results = cortex.recall_context("test", 5).unwrap();
    
    assert_eq!(results.len(), 0);
    // Should show helpful empty state message
}

#[test]
fn test_empty_state_no_results() {
    let mut cortex = CortexMemory::new_with_data();
    let results = cortex.recall_context("nonexistent", 5).unwrap();
    
    assert_eq!(results.len(), 0);
    // Should suggest alternatives
}
```

---

## Summary

**Empty states designed**: 15+ scenarios
**All empty states have**:
- Clear explanation of why empty
- Helpful next steps
- Encouragement to use feature
- Alternative actions

**Principles followed**:
- Helpful, not blocking
- Educational
- Actionable
- Contextual

**Ready for implementation** ✅
