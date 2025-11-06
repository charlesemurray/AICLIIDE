# Skills with Session Management

## Overview
Skills can now create, switch to, and close chat sessions. This is useful for skills that need persistent conversation context or isolated workspaces.

## Use Cases

### 1. Code Session Skills
Skills that need a dedicated coding environment:
- Code review sessions
- Debugging sessions
- Feature development sessions

### 2. Conversation Skills
Skills that manage multi-turn conversations:
- Interview/questionnaire skills
- Guided workflows
- Interactive tutorials

## API

### SkillResult Fields

```rust
pub struct SkillResult {
    pub output: String,
    pub ui_updates: Option<Vec<UIUpdate>>,
    pub state_changes: Option<serde_json::Value>,
    
    // Session management
    pub create_session: Option<SessionRequest>,
    pub switch_to_session: Option<String>,
    pub close_session: Option<String>,
}

pub struct SessionRequest {
    pub name: String,
    pub session_type: SessionType,
    pub initial_prompt: Option<String>,
}
```

### Helper Methods

```rust
// Create a new session
SkillResult::with_session(
    "Created debugging session".to_string(),
    "debug-auth".to_string(),
    SessionType::Debug
)

// Switch to existing session
SkillResult::switch_session(
    "Switching to feature work".to_string(),
    "feature-login".to_string()
)

// Close a session
SkillResult::close_session(
    "Closing completed session".to_string(),
    "old-session".to_string()
)
```

## Example Skills

### Example 1: Code Review Skill

```json
{
  "name": "start-review",
  "description": "Start a code review session",
  "type": "code_session",
  "parameters": [
    {
      "name": "pr_number",
      "type": "string",
      "required": true
    }
  ],
  "implementation": {
    "type": "command",
    "command": "echo 'Starting review for PR {{pr_number}}'"
  }
}
```

Skill implementation:
```rust
async fn execute(&self, params: Value) -> Result<SkillResult> {
    let pr_number = params["pr_number"].as_str().unwrap();
    
    Ok(SkillResult::with_session(
        format!("Starting code review for PR #{}", pr_number),
        format!("review-pr-{}", pr_number),
        SessionType::CodeReview
    ))
}
```

### Example 2: Feature Development Skill

```rust
async fn execute(&self, params: Value) -> Result<SkillResult> {
    let feature_name = params["feature"].as_str().unwrap();
    
    let mut result = SkillResult::with_session(
        format!("Created session for {}", feature_name),
        format!("feature-{}", feature_name),
        SessionType::Feature
    );
    
    // Set initial prompt
    if let Some(ref mut req) = result.create_session {
        req.initial_prompt = Some(format!(
            "Let's implement the {} feature. What should we start with?",
            feature_name
        ));
    }
    
    Ok(result)
}
```

### Example 3: Session Switcher Skill

```rust
async fn execute(&self, params: Value) -> Result<SkillResult> {
    let session_name = params["session"].as_str().unwrap();
    
    Ok(SkillResult::switch_session(
        format!("Switching to {}", session_name),
        session_name.to_string()
    ))
}
```

## Skill Types That Benefit

### CodeSession
- Persistent coding environment
- File tracking across conversation
- Checkpoint/restore capability

### Conversation
- Multi-turn dialogues
- Context preservation
- Guided workflows

## User Experience

When a skill requests session management:

```bash
> /skills execute start-review --pr_number 123

✓ Skill completed in 0.15s
Starting code review for PR #123

[Session Request] Creating session: review-pr-123
Use /sessions switch review-pr-123 to activate
```

User can then:
```bash
> /sessions switch review-pr-123
# Now in dedicated review session

> # Do review work...

> /close
# Session archived, back to main session
```

## Implementation Notes

### Session Creation
- Skills request session creation via `create_session` field
- Actual creation happens in chat system
- User must explicitly switch to new session

### Session Switching
- Skills can suggest switching to existing sessions
- Useful for resuming previous work
- User confirmation required

### Session Closing
- Skills can request session closure
- Marks session as archived
- Not restored on next launch

## Best Practices

1. **Descriptive Names**: Use clear session names
   - Good: `review-pr-123`, `feature-auth`
   - Bad: `session1`, `temp`

2. **Appropriate Types**: Choose correct SessionType
   - `CodeReview` for reviews
   - `Feature` for new features
   - `Debug` for debugging
   - `Development` for general work

3. **Initial Prompts**: Provide context
   ```rust
   req.initial_prompt = Some("Context for this session...");
   ```

4. **Clean Up**: Close sessions when done
   ```rust
   SkillResult::close_session(output, session_name)
   ```

## Future Enhancements

1. **Auto-switch**: Automatically switch after creation
2. **Session templates**: Pre-configured session setups
3. **Session groups**: Related sessions
4. **Session inheritance**: Child sessions from parent
5. **Auto-close**: Close on completion

## Testing

```bash
# Test session creation
> /skills execute start-review --pr_number 123
# Verify session created

# Test session switch
> /sessions list
# Verify session exists
> /sessions switch review-pr-123
# Verify switched

# Test session close
> /close
# Verify session archived
```

## Related Files
- `crates/chat-cli/src/cli/skills/mod.rs` - SkillResult definition
- `crates/chat-cli/src/cli/chat/tools/skill_tool.rs` - Session request handling
- `crates/chat-cli/src/cli/chat/coordinator.rs` - Session management
