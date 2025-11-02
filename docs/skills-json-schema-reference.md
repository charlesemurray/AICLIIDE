# Skills JSON Schema Reference

This document provides the complete specification for Q CLI skill JSON files. All skills must conform to this schema to be loaded and executed properly.

## Core Schema Structure

```json
{
  "name": "string (required)",
  "description": "string (optional)",
  "type": "skill_type (required)",
  "timeout": "number (optional)",
  "security": "object (optional)",
  "session_config": "object (optional)", 
  "prompt_template": "string (optional)",
  "prompt": "string (optional, alias for prompt_template)",
  "command": "string (optional)",
  "args": "array (optional)",
  "context_files": "object (optional)",
  "parameters": "array (optional)"
}
```

## Core Fields

### `name` (required)
- **Type**: String
- **Description**: Unique identifier for the skill
- **Format**: Alphanumeric characters, hyphens, and underscores only
- **Example**: `"greeting-skill"`, `"code-reviewer"`

### `description` (optional)
- **Type**: String  
- **Description**: Human-readable description of what the skill does
- **Example**: `"Generate personalized greetings with custom parameters"`

### `type` (required)
- **Type**: String (enum)
- **Description**: Defines the skill execution type
- **Valid Values**:
  - `"command"` - Execute a single command
  - `"code_inline"` - Execute command and return output immediately
  - `"code_session"` - Maintain persistent command session
  - `"conversation"` - AI conversation with context
  - `"prompt_inline"` - Parameterized prompt template

### `timeout` (optional)
- **Type**: Number (seconds)
- **Description**: Maximum execution time for the skill
- **Default**: System default timeout
- **Example**: `30`

## Type-Specific Fields

### For `command` and `code_inline` types:

#### `command` (required)
- **Type**: String
- **Description**: Command to execute
- **Example**: `"echo"`, `"python3"`, `"git"`

#### `args` (optional)
- **Type**: Array of strings
- **Description**: Command arguments
- **Example**: `["--version"]`, `["-c", "print('hello')"]`

### For `code_session` type:

#### `command` (required)
- **Type**: String
- **Description**: Command to start the session
- **Example**: `"python3"`, `"node"`

#### `session_config` (optional)
- **Type**: Object
- **Description**: Session configuration options
- **Fields**:
  - `session_timeout`: Number (seconds)
  - `max_sessions`: Number
  - `cleanup_on_exit`: Boolean

```json
{
  "session_config": {
    "session_timeout": 3600,
    "max_sessions": 5,
    "cleanup_on_exit": true
  }
}
```

### For `conversation` type:

#### `prompt_template` (required)
- **Type**: String
- **Description**: Template for AI conversation
- **Supports**: Parameter substitution with `{parameter_name}`
- **Example**: `"Analyze this {language} code: {code}"`

#### `context_files` (optional)
- **Type**: Object
- **Description**: File context configuration
- **Fields**:
  - `patterns`: Array of glob patterns
  - `max_files`: Number (optional)
  - `max_file_size_kb`: Number (optional)

```json
{
  "context_files": {
    "patterns": ["*.rs", "*.py"],
    "max_files": 10,
    "max_file_size_kb": 100
  }
}
```

### For `prompt_inline` type:

#### `prompt_template` or `prompt` (required)
- **Type**: String
- **Description**: Template string with parameter placeholders
- **Supports**: Parameter substitution with `{parameter_name}`
- **Example**: `"Hello {name}! Welcome to {place}."`
- **Note**: `prompt_template` and `prompt` are mutually exclusive aliases - use one or the other, not both

## Parameters Schema

The `parameters` field defines input parameters for skills that support them (`prompt_inline`, `conversation`).

### Parameter Object Structure

```json
{
  "name": "string (required)",
  "type": "string (required)",
  "required": "boolean (optional, default: false)",
  "values": "array (optional)",
  "pattern": "string (optional)"
}
```

### Parameter Fields

#### `name` (required)
- **Type**: String
- **Description**: Parameter identifier used in templates
- **Format**: Alphanumeric characters and underscores
- **Example**: `"user_name"`, `"language"`, `"count"`

#### `type` (required)
- **Type**: String (enum)
- **Description**: Parameter data type for validation
- **Valid Values**: 
  - `"string"` - Text values
  - `"number"` - Numeric values  
  - `"enum"` - Restricted to predefined values (requires `values` field)

#### `required` (optional)
- **Type**: Boolean
- **Description**: Whether parameter must be provided
- **Default**: `false`
- **Example**: `true`, `false`

#### `values` (optional)
- **Type**: Array of strings
- **Description**: Allowed values for `enum` type parameters
- **Required**: When `type` is `"enum"`
- **Example**: `["small", "medium", "large"]`

#### `pattern` (optional)
- **Type**: String (regex)
- **Description**: Validation pattern for `string` type parameters
- **Example**: `"^[a-zA-Z0-9_]+$"` (alphanumeric and underscores only)

### Parameter Examples

#### String Parameter
```json
{
  "name": "message",
  "type": "string",
  "required": true,
  "pattern": "^[^;|&$`]+$"
}
```

#### Number Parameter
```json
{
  "name": "count",
  "type": "number",
  "required": false
}
```

#### Enum Parameter
```json
{
  "name": "priority",
  "type": "enum",
  "values": ["low", "medium", "high"],
  "required": true
}
```

## Security Configuration

### `security` (optional)
- **Type**: Object
- **Description**: Security constraints and permissions
- **Fields**:
  - `resource_limits`: Object with execution limits
  - `permissions`: Object with access permissions

```json
{
  "security": {
    "resource_limits": {
      "max_memory_mb": 100,
      "max_execution_time": 30
    },
    "permissions": {
      "file_read": ["./src", "./tests"],
      "network_access": false
    }
  }
}
```

## Complete Skill Examples

### Prompt Inline Skill
```json
{
  "name": "greeting",
  "description": "Generate personalized greetings",
  "type": "prompt_inline",
  "prompt_template": "Hello {name}! Welcome to {place}. Today is {day}.",
  "parameters": [
    {
      "name": "name",
      "type": "string",
      "required": true,
      "pattern": "^[a-zA-Z\\s]+$"
    },
    {
      "name": "place",
      "type": "string",
      "required": false
    },
    {
      "name": "day",
      "type": "enum",
      "values": ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"],
      "required": true
    }
  ]
}
```

### Code Session Skill
```json
{
  "name": "python-repl",
  "description": "Interactive Python session",
  "type": "code_session",
  "command": "python3",
  "session_config": {
    "session_timeout": 1800,
    "max_sessions": 3,
    "cleanup_on_exit": true
  },
  "security": {
    "resource_limits": {
      "max_memory_mb": 256,
      "max_execution_time": 60
    },
    "permissions": {
      "file_read": ["./"],
      "network_access": false
    }
  }
}
```

### Conversation Skill
```json
{
  "name": "code-reviewer",
  "description": "AI-powered code review assistant",
  "type": "conversation",
  "prompt_template": "Review this {language} code and provide feedback: {code}",
  "parameters": [
    {
      "name": "language",
      "type": "enum",
      "values": ["python", "javascript", "rust", "java"],
      "required": true
    },
    {
      "name": "code",
      "type": "string",
      "required": true
    }
  ],
  "context_files": {
    "patterns": ["*.py", "*.js", "*.rs", "*.java"],
    "max_files": 5,
    "max_file_size_kb": 50
  }
}
```

## Common Validation Errors

1. **Missing required fields**: `name`, `type` must be present
2. **Invalid skill type**: Must be one of the 5 supported types
3. **Invalid parameter type**: Must be `string`, `number`, or `enum`
4. **Missing enum values**: `enum` type requires `values` array
5. **Invalid regex pattern**: `pattern` field must be valid regex
- **Description**: Maximum execution time before termination
- **Default**: 30 seconds
- **Example**: `60`

## Type-Specific Fields

### For `command` and `code_inline` types:

#### `command` (required)
- **Type**: String
- **Description**: Command to execute
- **Example**: `"echo"`, `"python3"`, `"ls"`

#### `args` (optional)
- **Type**: Array of strings
- **Description**: Command line arguments
- **Example**: `["--version"]`, `["-la", "/tmp"]`

### For `code_session` type:

#### `command` (required)
- **Type**: String
- **Description**: Command to start the session
- **Example**: `"python3"`, `"node"`

#### `session_config` (optional)
- **Type**: Object
- **Description**: Session management configuration
- **Fields**:
  - `session_timeout`: Number (seconds)
  - `max_sessions`: Number
  - `cleanup_on_exit`: Boolean

```json
{
  "session_config": {
    "session_timeout": 3600,
    "max_sessions": 5,
    "cleanup_on_exit": true
  }
}
```

### For `conversation` type:

#### `prompt_template` (required)
- **Type**: String
- **Description**: Template for AI conversation
- **Supports**: Parameter substitution with `{parameter_name}`
- **Example**: `"Analyze this code: {code}"`

#### `context_files` (optional)
- **Type**: Object
- **Description**: File context configuration
- **Fields**:
  - `patterns`: Array of file patterns
  - `max_files`: Number
  - `max_file_size_kb`: Number

```json
{
  "context_files": {
    "patterns": ["*.rs", "*.py"],
    "max_files": 10,
    "max_file_size_kb": 100
  }
}
```

### For `prompt_inline` type:

#### `prompt_template` or `prompt` (required)
- **Type**: String
- **Description**: Template string with parameter placeholders
- **Supports**: Parameter substitution with `{parameter_name}`
- **Example**: `"Hello {name}! Welcome to {place}."`
- **Note**: `prompt_template` and `prompt` are mutually exclusive aliases - use one or the other, not both

## Parameters Schema

The `parameters` field defines input parameters for skills that support them (`prompt_inline`, `conversation`).

### Parameter Object Structure

```json
{
  "name": "string (required)",
  "type": "string (required)",
  "required": "boolean (optional, default: false)",
  "values": "array (optional)",
  "pattern": "string (optional)"
}
```

### Parameter Fields

#### `name` (required)
- **Type**: String
- **Description**: Parameter identifier used in templates
- **Format**: Alphanumeric characters and underscores
- **Example**: `"user_name"`, `"file_path"`

#### `type` (required)
- **Type**: String
- **Description**: Parameter data type
- **Valid Values**:
  - `"string"` - Text input
  - `"number"` - Numeric input
  - `"boolean"` - True/false input
  - `"enum"` - Selection from predefined values

#### `required` (optional)
- **Type**: Boolean
- **Description**: Whether parameter is mandatory
- **Default**: `false`

#### `values` (optional)
- **Type**: Array of strings
- **Description**: Valid values for `enum` type parameters
- **Example**: `["small", "medium", "large"]`

#### `pattern` (optional)
- **Type**: String (regex)
- **Description**: Validation pattern for string parameters
- **Example**: `"^[a-zA-Z0-9]+$"`

### Parameter Examples

```json
{
  "parameters": [
    {
      "name": "name",
      "type": "string",
      "required": true,
      "pattern": "^[a-zA-Z ]+$"
    },
    {
      "name": "size",
      "type": "enum",
      "values": ["small", "medium", "large"],
      "required": false
    },
    {
      "name": "count",
      "type": "number",
      "required": true
    }
  ]
}
```

## Security Configuration

### `security` (optional)
- **Type**: Object
- **Description**: Security and resource limit configuration

```json
{
  "security": {
    "resource_limits": {
      "max_memory_mb": 100,
      "max_execution_time": 30,
      "max_cpu_percent": 50
    },
    "permissions": {
      "allow_network": false,
      "allow_file_write": false,
      "allowed_paths": ["/tmp"]
    }
  }
}
```

## Complete Examples

### Code Inline Skill
```json
{
  "name": "echo-test",
  "description": "Simple echo command for testing",
  "type": "code_inline",
  "command": "echo",
  "args": ["Hello World"],
  "timeout": 10
}
```

### Prompt Inline Skill
```json
{
  "name": "greeting",
  "description": "Generate personalized greetings",
  "type": "prompt_inline",
  "prompt_template": "Hello {name}! Welcome to {place}. Today is {day}.",
  "parameters": [
    {
      "name": "name",
      "type": "string",
      "required": true,
      "pattern": "^[a-zA-Z ]+$"
    },
    {
      "name": "place",
      "type": "string",
      "required": false
    },
    {
      "name": "day",
      "type": "enum",
      "values": ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday"],
      "required": true
    }
  ]
}
```

### Code Session Skill
```json
{
  "name": "python-repl",
  "description": "Interactive Python session",
  "type": "code_session",
  "command": "python3",
  "session_config": {
    "session_timeout": 3600,
    "max_sessions": 3,
    "cleanup_on_exit": true
  },
  "security": {
    "resource_limits": {
      "max_memory_mb": 256,
      "max_execution_time": 300
    }
  }
}
```

### Conversation Skill
```json
{
  "name": "code-reviewer",
  "description": "AI-powered code review assistant",
  "type": "conversation",
  "prompt_template": "Review this {language} code and provide feedback: {code}",
  "parameters": [
    {
      "name": "language",
      "type": "enum",
      "values": ["python", "javascript", "rust", "java"],
      "required": true
    },
    {
      "name": "code",
      "type": "string",
      "required": true
    }
  ],
  "context_files": {
    "patterns": ["*.py", "*.js", "*.rs", "*.java"],
    "max_files": 5,
    "max_file_size_kb": 50
  }
}
```

## Validation Rules

1. **Required Fields**: `name` and `type` must always be present
2. **Type Consistency**: Fields must match their skill type requirements
3. **Parameter Names**: Must be unique within a skill
4. **Template Variables**: All `{variable}` references in templates must have corresponding parameters
5. **Enum Values**: `enum` type parameters must include `values` array
6. **Pattern Format**: `pattern` must be valid regex
7. **Resource Limits**: Numeric values must be positive

## Migration Guide

### From Legacy Schema
If you have skills using the old parameter format:

**Old Format:**
```json
{
  "parameters": [
    {
      "name": "input",
      "description": "User input",
      "required": true,
      "default": "hello"
    }
  ]
}
```

**New Format:**
```json
{
  "parameters": [
    {
      "name": "input",
      "type": "string",
      "required": true
    }
  ]
}
```

### Key Changes
1. Add required `type` field to all parameters
2. Remove `description` and `default` fields (not currently supported)
3. Use `prompt_template` or `prompt` (both work via alias)
4. Ensure all template variables have corresponding parameters

## Error Messages

Common validation errors and solutions:

- **"Missing required field 'type'"**: Add `"type": "string"` to parameter
- **"Unknown skill type"**: Use one of: `command`, `code_inline`, `code_session`, `conversation`, `prompt_inline`
- **"Template variable not found"**: Add parameter definition for `{variable}` in template
- **"Invalid parameter type"**: Use: `string`, `number`, `boolean`, or `enum`
