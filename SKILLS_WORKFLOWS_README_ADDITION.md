# Skills & Workflows Feature - README Addition

## Add this section to the main README.md

---

## Skills & Workflows

Amazon Q CLI now supports custom skills and workflows, enabling you to extend the agent's capabilities with reusable tools.

### What's New

- **Skills**: Create custom capabilities the agent can invoke through natural language
- **Workflows**: Chain multiple skills together for complex multi-step tasks
- **Natural Language**: Invoke skills and workflows conversationally
- **Type Safety**: Full schema validation for parameters and inputs
- **Error Handling**: Graceful error handling with clear messages

### Quick Example

Create a skill in `~/.q-skills/hello.json`:

```json
{
  "name": "hello",
  "description": "Greet a person by name",
  "parameters": [
    {
      "name": "name",
      "type": "string",
      "required": true
    }
  ],
  "implementation": {
    "type": "command",
    "command": "echo 'Hello, {{name}}!'"
  }
}
```

Then use it naturally:

```bash
q chat "Say hello to Alice"
```

The agent will automatically discover and use your skill.

### Built-in Skills

- **calculator**: Perform arithmetic operations
- More skills coming soon!

### Documentation

- [Quick Start Guide](docs/SKILLS_QUICKSTART.md) - Get started in 5 minutes
- [Full Integration Guide](docs/SKILLS_WORKFLOWS_INTEGRATION.md) - Complete documentation
- [API Reference](docs/SKILLS_WORKFLOWS_INTEGRATION.md#api-reference) - For developers

### Examples

See `crates/chat-cli/tests/` for complete examples:
- `skill_toolspec_integration.rs` - Skill integration examples
- `workflow_toolspec_integration.rs` - Workflow examples
- `natural_language_skill_invocation.rs` - Natural language usage
- `skill_workflow_error_handling.rs` - Error handling patterns

---

## Suggested Placement

Add this section after the "Project Layout" section and before "Security" in the main README.md.
