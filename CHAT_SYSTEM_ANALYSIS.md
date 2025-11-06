# Chat System Flow Analysis

## Single Session Flow (Working)

### 1. Initialization
```
ChatArgs::execute()
  └─> ChatSession::new()
        └─> Creates ConversationState with next_message = None
        └─> Sets up input/output conduits
        └─> Returns ChatSession
  └─> session.spawn(os)
```

### 2. Main Loop (spawn)
```
ChatSession::spawn()
  └─> while !Exit {
        └─> self.next(os)
      }
```

### 3. State Machine (next)
```
ChatSession::next()
  └─> match self.inner {
        PromptUser => prompt_user()
        HandleInput => handle_input()
        HandleResponseStream => handle_response()
        ExecuteTools => execute_tools()
        Exit => return
      }
```

### 4. Message Flow
```
PromptUser:
  - Display prompt
  - Read user input
  - Return HandleInput state

HandleInput:
  - conversation.set_next_user_message(input)  // Sets next_message
  - Create request
  - Return HandleResponseStream state

HandleResponseStream:
  - Stream response from LLM
  - conversation.push_assistant_message()
  - If tools requested: Return ExecuteTools
  - Else: Return PromptUser

ExecuteTools:
  - Execute tools
  - conversation.push_tool_results()
  - Return HandleResponseStream (to get next response)

After response completes:
  - conversation.enforce_conversation_invariants()
  - conversation.reset_next_user_message()  // Clears next_message
  - Return PromptUser
```

### Key Invariant
**next_message should be None when entering PromptUser state**

## Multi-Session Flow (Broken)

### 1. Initialization
```
ChatArgs::execute()
  └─> Creates coordinator
  └─> Creates initial ChatSession (session A)
  └─> Stores in coordinator
  └─> coordinator.run(os)
```

### 2. Coordinator Loop
```
coordinator.run()
  └─> loop {
        - Get active_session_id
        - Get/create ChatSession for that session
        - session.spawn(os)
        - If switched: continue loop with new session
        - Else: break
      }
```

### 3. Creating New Session
```
/sessions new "what"
  └─> coordinator.create_session()
        └─> Creates fresh ConversationState (next_message = None)
        └─> Stores in ManagedSession
        └─> Does NOT create ChatSession yet
```

### 4. Switching Sessions
```
/sessions switch "what"
  └─> coordinator.switch_session("what")
        └─> Updates active_session_id
  └─> Returns SwitchSession state
  └─> Current ChatSession exits
  └─> Coordinator loop continues
  └─> Gets "what" session
  └─> ChatSession doesn't exist, so creates it:
        └─> ChatSession::from_conversation(conversation.clone())
              └─> Clones conversation from ManagedSession
              └─> conversation.reset_next_user_message()
              └─> conversation.enforce_conversation_invariants()
              └─> Creates new ChatSession
```

### 5. First Message in New Session (Works)
```
User types: "Will you send anything to the llm"
  └─> PromptUser -> HandleInput
  └─> conversation.set_next_user_message()  // next_message = None ✓
  └─> HandleResponseStream
  └─> Response completes
  └─> conversation.reset_next_user_message()  // next_message = None ✓
  └─> PromptUser
```

### 6. Second Message in New Session (PANIC)
```
User types: "So what do you think of google"
  └─> PromptUser -> HandleInput
  └─> conversation.set_next_user_message()  // PANIC: next_message is NOT None!
```

## The Bug

**Problem**: After the first message completes in a switched session, `next_message` is not None when the second message starts.

**Why?**
1. The ChatSession is reused (not recreated)
2. After first message, conversation state should be clean
3. But something is preventing `reset_next_user_message()` from being called
4. OR something is setting `next_message` again after it's cleared

## Hypothesis

Looking at the flow, after `HandleResponseStream` completes:
- It should call `conversation.reset_next_user_message()`
- Then return `PromptUser` state

But in a switched session, maybe:
1. The conversation state is not being properly managed
2. The ChatSession's conversation reference is stale
3. The reset is happening on a different conversation instance

## Key Difference: Conversation Ownership

**Single Session:**
- ChatSession owns the conversation
- All state changes happen on the same instance
- reset_next_user_message() affects the same conversation

**Multi-Session:**
- ManagedSession stores conversation
- ChatSession clones conversation when created
- ChatSession modifies its clone
- When session exits, we save conversation back to ManagedSession
- When session resumes, we clone again

**THE BUG**: We're cloning the conversation, but the clone might be stale or the save-back isn't working properly!

## Solution

We need to ensure that:
1. When a ChatSession exits (switch), it saves its conversation state back
2. When a ChatSession resumes, it gets the latest conversation state
3. The conversation state is properly cleaned between messages

OR

We should NOT clone the conversation - the ChatSession should hold a reference to the ManagedSession's conversation directly.
