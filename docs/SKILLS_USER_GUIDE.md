# Skills User Guide

## What are Skills?

Skills are reusable capabilities that extend Amazon Q CLI. They allow you to create custom commands, scripts, and integrations that the AI assistant can invoke through natural language.

## Quick Start

### Listing Available Skills

```bash
q skills list
```

This shows all skills available in your workspace and globally installed skills.

### Running a Skill

```bash
q skills run calculator --params '{"operation": "add", "a": 5, "b": 3}'
```

Or use natural language in chat:
```bash
q chat
> Calculate 5 + 3
```

### Viewing Skill Details

```bash
q skills info calculator
```

## Skill Types

### 1. Command Skills (code_inline)
Execute shell commands with parameters.

**Example**: `hello.json`
```json
{
  "name": "hello",
  "description": "Greet a person by name",
  "skill_type": "code_inline",
  "parameters": [
    {
      "name": "name",
      "type": "string",
      "required": true,
      "description": "Name of the person to greet"
    }
  ],
  "implementation": {
    "type": "command",
    "command": "echo 'Hello, {{name}}!'"
  }
}
```

### 2. Script Skills
Execute scripts with environment variables.

**Example**: `backup.json`
```json
{
  "name": "backup",
  "description": "Backup files to a directory",
  "skill_type": "code_inline",
  "parameters": [
    {
      "name": "source",
      "type": "string",
      "required": true
    },
    {
      "name": "destination",
      "type": "string",
      "required": true
    }
  ],
  "implementation": {
    "type": "script",
    "path": "./scripts/backup.sh"
  }
}
```

**Script**: `scripts/backup.sh`
```bash
#!/bin/bash
cp -r "$SKILL_PARAM_source" "$SKILL_PARAM_destination"
echo "Backed up $SKILL_PARAM_source to $SKILL_PARAM_destination"
```

## Managing Skills

### Adding a Skill

1. Create a skill JSON file
2. Add it to your workspace:

```bash
q skills install ./my-skill.json
```

Skills are stored in `.q-skills/` directory in your workspace.

### Removing a Skill

```bash
q skills remove my-skill
```

You'll be prompted to confirm before deletion.

### Creating Skills

Use the interactive creation assistant:

```bash
q chat
> /skills create my-skill command
```

Or use the unified creation system:

```bash
q create skill my-skill guided
```

## Skill Definition Format

### Required Fields

- `name` (string): Unique identifier for the skill
- `description` (string): What the skill does
- `skill_type` (string): Type of skill (code_inline, code_session, conversation, prompt_inline, rust)

### Optional Fields

- `parameters` (array): Input parameters
- `implementation` (object): How the skill executes
- `aliases` (array): Alternative names
- `security` (object): Security settings
- `timeout` (number): Execution timeout in seconds

### Parameter Definition

```json
{
  "name": "param_name",
  "type": "string|number|boolean|array|object",
  "required": true|false,
  "description": "What this parameter does",
  "default": "default_value"
}
```

### Implementation Types

#### Command
```json
{
  "type": "command",
  "command": "echo {{param}}"
}
```

#### Script
```json
{
  "type": "script",
  "path": "./path/to/script.sh"
}
```

## Environment Variables

When using script implementation, parameters are passed as environment variables:

- Parameter `name` becomes `SKILL_PARAM_name`
- Parameter `count` becomes `SKILL_PARAM_count`

**Example**:
```bash
#!/bin/bash
echo "Processing $SKILL_PARAM_count items"
echo "Output to $SKILL_PARAM_output_file"
```

## Best Practices

### 1. Clear Descriptions
Write descriptions that explain what the skill does and when to use it:
```json
{
  "description": "Compress files into a tar.gz archive with optional encryption"
}
```

### 2. Validate Parameters
Use required fields and types to ensure correct usage:
```json
{
  "parameters": [
    {
      "name": "file",
      "type": "string",
      "required": true,
      "description": "Path to the file to process"
    }
  ]
}
```

### 3. Handle Errors
Return meaningful error messages:
```bash
#!/bin/bash
if [ ! -f "$SKILL_PARAM_file" ]; then
  echo "Error: File not found: $SKILL_PARAM_file"
  exit 1
fi
```

### 4. Use Timeouts
Set reasonable timeouts for long-running operations:
```json
{
  "timeout": 300
}
```

### 5. Document Output
Explain what the skill returns:
```json
{
  "description": "Returns the number of lines in the file"
}
```

## Examples

### File Counter
```json
{
  "name": "count-lines",
  "description": "Count lines in a file",
  "skill_type": "code_inline",
  "parameters": [
    {
      "name": "file",
      "type": "string",
      "required": true
    }
  ],
  "implementation": {
    "type": "command",
    "command": "wc -l {{file}}"
  }
}
```

### Git Status
```json
{
  "name": "git-status",
  "description": "Show git repository status",
  "skill_type": "code_inline",
  "implementation": {
    "type": "command",
    "command": "git status --short"
  }
}
```

### Data Processor
```json
{
  "name": "process-data",
  "description": "Process JSON data with jq",
  "skill_type": "code_inline",
  "parameters": [
    {
      "name": "file",
      "type": "string",
      "required": true
    },
    {
      "name": "filter",
      "type": "string",
      "required": true
    }
  ],
  "implementation": {
    "type": "command",
    "command": "jq '{{filter}}' {{file}}"
  }
}
```

## Troubleshooting

### Skill Not Found
- Check skill name: `q skills list`
- Verify file exists in `.q-skills/`
- Ensure JSON is valid: `cat .q-skills/my-skill.json | jq`

### Execution Fails
- Check script permissions: `chmod +x script.sh`
- Verify script path in implementation
- Check parameter names match environment variables

### Timeout Errors
- Increase timeout in skill definition
- Optimize script performance
- Consider breaking into smaller skills

## Advanced Topics

### Security Settings
```json
{
  "security": {
    "allow_network": false,
    "allow_filesystem": true,
    "allowed_paths": ["/tmp", "./data"]
  }
}
```

### Aliases
```json
{
  "name": "calculator",
  "aliases": ["calc", "math"]
}
```

### Complex Parameters
```json
{
  "parameters": [
    {
      "name": "config",
      "type": "object",
      "required": true,
      "properties": {
        "host": {"type": "string"},
        "port": {"type": "number"}
      }
    }
  ]
}
```

## See Also

- [Workflows User Guide](WORKFLOWS_USER_GUIDE.md) - Chain multiple skills together
- [Skills Quick Start](SKILLS_QUICKSTART.md) - Get started in 5 minutes
- [Skills & Workflows Integration](SKILLS_WORKFLOWS_INTEGRATION.md) - Technical details
