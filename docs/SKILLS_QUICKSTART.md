# Skills & Workflows Quick Start

## 5-Minute Guide to Skills and Workflows

### What Are Skills?

Skills are reusable capabilities that the AI agent can invoke. Think of them as functions the agent can call.

### What Are Workflows?

Workflows are multi-step processes that chain skills together to accomplish complex tasks.

## Using Built-in Skills

The calculator skill is available by default:

```bash
q chat "What is 15 + 27?"
```

The agent will automatically use the calculator skill to compute the result.

## Creating Your First Skill

### 1. Create a Skill Definition

Create `~/.q-skills/hello.json`:

```json
{
  "name": "hello",
  "description": "Greet a person by name",
  "parameters": [
    {
      "name": "name",
      "type": "string",
      "description": "Person's name",
      "required": true
    }
  ],
  "implementation": {
    "type": "command",
    "command": "echo 'Hello, {{name}}!'"
  }
}
```

### 2. Use Your Skill

```bash
q chat "Say hello to Alice"
```

The agent will discover and use your skill automatically.

## Creating Your First Workflow

### 1. Create a Workflow Definition

Create `~/.q-workflows/greet_and_count.json`:

```json
{
  "name": "greet_and_count",
  "description": "Greet someone and count letters in their name",
  "version": "1.0.0",
  "inputs": [
    {
      "name": "name",
      "type": "string",
      "required": true
    }
  ],
  "steps": [
    {
      "id": "greet",
      "type": "skill",
      "name": "hello",
      "inputs": {
        "name": "{{inputs.name}}"
      }
    },
    {
      "id": "count",
      "type": "skill",
      "name": "count_chars",
      "inputs": {
        "text": "{{inputs.name}}"
      }
    }
  ]
}
```

### 2. Use Your Workflow

```bash
q chat "Run the greet_and_count workflow for Bob"
```

## Common Patterns

### Skill with Multiple Parameters

```json
{
  "name": "send_email",
  "description": "Send an email",
  "parameters": [
    {
      "name": "to",
      "type": "string",
      "required": true
    },
    {
      "name": "subject",
      "type": "string",
      "required": true
    },
    {
      "name": "body",
      "type": "string",
      "required": false
    }
  ],
  "implementation": {
    "type": "command",
    "command": "mail -s '{{subject}}' {{to}} <<< '{{body}}'"
  }
}
```

### Workflow with Conditional Logic

```json
{
  "name": "process_data",
  "description": "Fetch and process data",
  "version": "1.0.0",
  "steps": [
    {
      "id": "fetch",
      "type": "skill",
      "name": "fetch_data",
      "inputs": {"source": "api"}
    },
    {
      "id": "validate",
      "type": "skill",
      "name": "validate_data",
      "inputs": {"data": "{{fetch.output}}"}
    },
    {
      "id": "process",
      "type": "skill",
      "name": "process_data",
      "inputs": {"data": "{{fetch.output}}"}
    }
  ]
}
```

## Tips

1. **Descriptive Names**: Use clear, descriptive names for skills and parameters
2. **Good Descriptions**: Help the agent understand when to use your skill
3. **Validation**: Mark required parameters and add constraints
4. **Test Incrementally**: Test skills individually before using in workflows
5. **Error Handling**: Skills should handle errors gracefully

## Next Steps

- Read the full [Skills & Workflows Integration Guide](SKILLS_WORKFLOWS_INTEGRATION.md)
- Check out example skills in `examples/skills/`
- Review integration tests for advanced patterns
- Join the community to share your skills

## Getting Help

- Check the [Troubleshooting Guide](SKILLS_WORKFLOWS_INTEGRATION.md#troubleshooting)
- Review error messages carefully
- Test skills with simple inputs first
- Use `q chat --help` for CLI options
