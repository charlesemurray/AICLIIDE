# Workflows User Guide

## What are Workflows?

Workflows are multi-step processes that chain together skills and built-in tools. They enable complex automation by executing steps sequentially and passing data between them.

## Quick Start

### Listing Available Workflows

```bash
q workflows list
```

### Viewing Workflow Details

```bash
q workflows show data-pipeline
```

### Running a Workflow

Use natural language in chat:
```bash
q chat
> Run the data-pipeline workflow
```

## Workflow Structure

A workflow consists of:
- **Metadata**: Name, version, description
- **Steps**: Ordered list of operations
- **Context**: Shared data between steps

### Basic Example

```json
{
  "name": "backup-and-notify",
  "version": "1.0.0",
  "description": "Backup files and send notification",
  "steps": [
    {
      "name": "backup",
      "tool": "backup-files",
      "parameters": {
        "source": "./data",
        "destination": "./backups"
      }
    },
    {
      "name": "notify",
      "tool": "echo",
      "parameters": {
        "msg": "Backup completed"
      }
    }
  ]
}
```

## Managing Workflows

### Adding a Workflow

1. Create a workflow JSON file
2. Add it to your workspace:

```bash
q workflows add ./my-workflow.json
```

Workflows are stored in `.q-workflows/` directory.

### Removing a Workflow

```bash
q workflows remove my-workflow
```

You'll be prompted to confirm before deletion.

## Workflow Definition Format

### Required Fields

- `name` (string): Unique identifier
- `version` (string): Semantic version (e.g., "1.0.0")
- `description` (string): What the workflow does
- `steps` (array): List of steps to execute

### Optional Fields

- `context` (object): Initial context data shared across steps

### Step Definition

```json
{
  "name": "step-name",
  "tool": "tool-name",
  "parameters": {
    "param1": "value1",
    "param2": "value2"
  }
}
```

## Step Execution

### Sequential Processing

Steps execute in order, one after another:

```json
{
  "steps": [
    {"name": "step1", "tool": "echo", "parameters": {"msg": "First"}},
    {"name": "step2", "tool": "echo", "parameters": {"msg": "Second"}},
    {"name": "step3", "tool": "echo", "parameters": {"msg": "Third"}}
  ]
}
```

Output:
```
Executed 3 steps successfully in 1.23ms

Step 'step1': Executed step 'step1' with tool 'echo'
Step 'step2': Executed step 'step2' with tool 'echo'
Step 'step3': Executed step 'step3' with tool 'echo'
```

### Error Handling

If a step fails, the workflow stops immediately:

```json
{
  "steps": [
    {"name": "step1", "tool": "echo", "parameters": {"msg": "OK"}},
    {"name": "step2", "tool": "nonexistent", "parameters": {}},
    {"name": "step3", "tool": "echo", "parameters": {"msg": "Never runs"}}
  ]
}
```

Output:
```
Workflow failed at step 2 ('step2'): Unknown tool 'nonexistent'
```

## Passing Data Between Steps

### Using Context

Steps can access data from previous steps through context:

```json
{
  "name": "data-pipeline",
  "version": "1.0.0",
  "description": "Process data through multiple steps",
  "steps": [
    {
      "name": "fetch",
      "tool": "fetch-data",
      "parameters": {"url": "https://api.example.com/data"}
    },
    {
      "name": "process",
      "tool": "process-data",
      "parameters": {
        "input": "{{steps.fetch.output}}"
      }
    }
  ]
}
```

### Initial Context

Provide initial data to all steps:

```json
{
  "name": "deploy",
  "version": "1.0.0",
  "description": "Deploy application",
  "context": {
    "environment": "production",
    "region": "us-east-1"
  },
  "steps": [
    {
      "name": "build",
      "tool": "build-app",
      "parameters": {
        "env": "{{context.environment}}"
      }
    }
  ]
}
```

## Available Tools

Workflows can use:
- **Built-in tools**: echo, calculator
- **Custom skills**: Any skill in `.q-skills/`
- **Native tools**: fs_read, fs_write, execute_bash, etc.

## Examples

### Simple Backup Workflow

```json
{
  "name": "daily-backup",
  "version": "1.0.0",
  "description": "Backup important files daily",
  "steps": [
    {
      "name": "backup-code",
      "tool": "backup-files",
      "parameters": {
        "source": "./src",
        "destination": "./backups/src"
      }
    },
    {
      "name": "backup-data",
      "tool": "backup-files",
      "parameters": {
        "source": "./data",
        "destination": "./backups/data"
      }
    },
    {
      "name": "verify",
      "tool": "echo",
      "parameters": {
        "msg": "Backup completed successfully"
      }
    }
  ]
}
```

### Data Processing Pipeline

```json
{
  "name": "data-pipeline",
  "version": "1.0.0",
  "description": "Fetch, process, and store data",
  "steps": [
    {
      "name": "fetch",
      "tool": "fetch-api-data",
      "parameters": {
        "endpoint": "/api/users"
      }
    },
    {
      "name": "transform",
      "tool": "transform-json",
      "parameters": {
        "filter": ".[] | {id, name, email}"
      }
    },
    {
      "name": "save",
      "tool": "save-to-file",
      "parameters": {
        "file": "./output/users.json"
      }
    }
  ]
}
```

### Build and Deploy

```json
{
  "name": "build-deploy",
  "version": "2.0.0",
  "description": "Build and deploy application",
  "context": {
    "environment": "production"
  },
  "steps": [
    {
      "name": "test",
      "tool": "run-tests",
      "parameters": {}
    },
    {
      "name": "build",
      "tool": "build-app",
      "parameters": {
        "env": "{{context.environment}}"
      }
    },
    {
      "name": "deploy",
      "tool": "deploy-app",
      "parameters": {
        "target": "{{context.environment}}"
      }
    },
    {
      "name": "notify",
      "tool": "send-notification",
      "parameters": {
        "message": "Deployed to {{context.environment}}"
      }
    }
  ]
}
```

## Best Practices

### 1. Descriptive Names

Use clear, descriptive names for workflows and steps:

```json
{
  "name": "backup-and-cleanup",
  "steps": [
    {"name": "backup-files", "tool": "..."},
    {"name": "remove-old-backups", "tool": "..."},
    {"name": "verify-backup", "tool": "..."}
  ]
}
```

### 2. Version Your Workflows

Use semantic versioning:
- `1.0.0` - Initial version
- `1.1.0` - Add new step
- `2.0.0` - Breaking change

### 3. Handle Errors Gracefully

Design workflows to fail fast and provide clear error messages.

### 4. Keep Steps Focused

Each step should do one thing well:

❌ Bad:
```json
{"name": "do-everything", "tool": "complex-script"}
```

✅ Good:
```json
[
  {"name": "validate-input", "tool": "validator"},
  {"name": "process-data", "tool": "processor"},
  {"name": "save-output", "tool": "saver"}
]
```

### 5. Document Context Usage

Comment on what context data is expected:

```json
{
  "description": "Deploy app (requires context.environment and context.region)",
  "context": {
    "environment": "production",
    "region": "us-east-1"
  }
}
```

## Timing and Performance

Workflows track execution time:

```
Executed 5 steps successfully in 2.45ms

Step 'fetch': completed in 0.82ms
Step 'process': completed in 1.12ms
Step 'save': completed in 0.51ms
```

Use this to:
- Identify slow steps
- Optimize performance
- Set appropriate timeouts

## Troubleshooting

### Workflow Not Found
```bash
q workflows list  # Check available workflows
ls .q-workflows/  # Verify file exists
```

### Step Fails
- Check tool name is correct
- Verify tool exists: `q skills list`
- Check parameter format
- Review error message for step number

### Context Not Working
- Verify context syntax: `{{steps.step_name.output}}`
- Check step name matches exactly
- Ensure previous step completed successfully

## Advanced Topics

### Complex Context

```json
{
  "context": {
    "config": {
      "database": {
        "host": "localhost",
        "port": 5432
      },
      "api": {
        "endpoint": "https://api.example.com"
      }
    }
  }
}
```

Access with: `{{context.config.database.host}}`

### Conditional Logic

Currently, workflows execute all steps sequentially. For conditional logic, use skills with built-in conditionals.

### Parallel Execution

Not yet supported. Steps execute sequentially in order.

## See Also

- [Skills User Guide](SKILLS_USER_GUIDE.md) - Create custom tools for workflows
- [Skills Quick Start](SKILLS_QUICKSTART.md) - Get started quickly
- [Skills & Workflows Integration](SKILLS_WORKFLOWS_INTEGRATION.md) - Technical details
