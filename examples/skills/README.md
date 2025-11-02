# Example Skills

This directory contains example skills demonstrating various capabilities.

## Available Examples

### 1. hello.json
**Description**: Simple greeting skill  
**Usage**: `q chat "Say hello to Alice"`  
**Parameters**:
- `name` (string, required): Person's name

### 2. count_lines.json
**Description**: Count lines in a file  
**Usage**: `q chat "Count lines in README.md"`  
**Parameters**:
- `file_path` (string, required): Path to file

### 3. git_status.json
**Description**: Get git repository status  
**Usage**: `q chat "What's the git status?"`  
**Parameters**:
- `directory` (string, optional): Repository directory

### 4. weather.json
**Description**: Get current weather  
**Usage**: `q chat "What's the weather in Seattle?"`  
**Parameters**:
- `location` (string, required): City name
- `format` (string, optional): "short" or "full"

### 5. format_json.json
**Description**: Format JSON data  
**Usage**: `q chat "Format this JSON: {\"key\":\"value\"}"`  
**Parameters**:
- `json_string` (string, required): JSON to format

## Using These Examples

### Option 1: Copy to Skills Directory
```bash
cp examples/skills/*.json ~/.q-skills/
```

### Option 2: Use Directly
Skills in this directory can be loaded by the agent when running from the repository root.

## Creating Your Own Skills

Use these examples as templates:

1. **Simple Command**: See `hello.json`
2. **File Operations**: See `count_lines.json`
3. **External APIs**: See `weather.json`
4. **Data Processing**: See `format_json.json`
5. **Optional Parameters**: See `git_status.json`

## Skill Structure

```json
{
  "name": "skill_name",
  "description": "What the skill does",
  "parameters": [
    {
      "name": "param_name",
      "type": "string|number|boolean",
      "description": "Parameter description",
      "required": true|false,
      "enum": ["option1", "option2"]  // optional
    }
  ],
  "implementation": {
    "type": "command",
    "command": "shell command with {{param_name}}"
  }
}
```

## Tips

1. **Clear Descriptions**: Help the agent understand when to use your skill
2. **Parameter Validation**: Use `required` and `enum` for validation
3. **Default Values**: Use `{{param:default}}` syntax for optional parameters
4. **Error Handling**: Commands should handle errors gracefully
5. **Testing**: Test skills individually before using in workflows

## Next Steps

- Read the [Quick Start Guide](../../docs/SKILLS_QUICKSTART.md)
- Check the [Full Integration Guide](../../docs/SKILLS_WORKFLOWS_INTEGRATION.md)
- Review integration tests in `crates/chat-cli/tests/`
