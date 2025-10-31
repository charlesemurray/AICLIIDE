# Q CLI Skills System Design

## Overview

The Q CLI Skills System extends Amazon Q CLI with two types of user-defined capabilities:
1. **Code-Centric Skills**: Execute external code with prompt-guided input/output handling
2. **Conversational Skills**: Create focused sub-conversations that summarize back to main context

## Architecture

### Skill Types

#### 1. Code-Centric Skills
- Execute external commands, scripts, or API calls
- Use prompts to validate input and format output
- Designed for deterministic operations with structured parameters
- Examples: calculators, file processors, API integrations

#### 2. Conversational Skills
- Create isolated conversation threads for specific problem domains
- Use guided questions to walk users through complex workflows
- Summarize results back to main conversation context
- Examples: debugging assistants, planning wizards, troubleshooting guides

### Integration Points

Skills integrate with Q CLI through three access patterns:
- **CLI Commands**: `q skills run calculator --params '{"operation": "add", "a": 5, "b": 3}'`
- **Chat @-syntax**: `@calculator add 5 3` or `@calc 5 + 3`
- **Slash Commands**: `/skills run calculator --params '{"operation": "add"}'`

## Configuration Schema

### Base Schema Structure

```json
{
  "$schema": "https://raw.githubusercontent.com/aws/amazon-q-developer-cli/main/schemas/skill-v1.json",
  "type": "code|conversation",
  "metadata": {
    "name": "string",
    "description": "string",
    "aliases": ["string"],
    "version": "string",
    "author": "string"
  }
}
```

### Code-Centric Skills

```json
{
  "$schema": "https://raw.githubusercontent.com/aws/amazon-q-developer-cli/main/schemas/skill-v1.json",
  "type": "code",
  "metadata": {
    "name": "calculator",
    "description": "Perform arithmetic calculations",
    "aliases": ["calc", "math"],
    "version": "1.0.0",
    "author": "Q CLI Team"
  },
  "prompt": {
    "input_validation": "file://./prompts/calculator_input.md",
    "argument_mapping": "file://./prompts/calculator_mapping.md", 
    "output_formatting": "Present calculation results clearly with original expression"
  },
  "executor": {
    "type": "command",
    "command": "python3",
    "args": ["file://./scripts/calculator.py", "{operation}", "{operand1}", "{operand2}"],
    "timeout": 30,
    "working_directory": "./skills/calculator",
    "env": {
      "PYTHONPATH": "/usr/local/lib/python3.9/site-packages"
    }
  },
  "parameters": {
    "operation": {
      "type": "string",
      "required": true,
      "enum": ["add", "subtract", "multiply", "divide"],
      "description": "Mathematical operation to perform"
    },
    "operand1": {
      "type": "number",
      "required": true,
      "description": "First number in the operation"
    },
    "operand2": {
      "type": "number",
      "required": true,
      "description": "Second number in the operation"
    }
  }
}
```

### Conversational Skills

```json
{
  "$schema": "https://raw.githubusercontent.com/aws/amazon-q-developer-cli/main/schemas/skill-v1.json",
  "type": "conversation",
  "metadata": {
    "name": "debug_helper",
    "description": "Help debug code issues through guided conversation",
    "aliases": ["debug", "troubleshoot"],
    "version": "1.0.0",
    "author": "Q CLI Team"
  },
  "conversation": {
    "initial_prompt": "file://./prompts/debug_initial.md",
    "max_turns": 10,
    "context_summary_prompt": "file://./prompts/debug_summary.md",
    "guided_questions": [
      "What programming language and framework are you using?",
      "What error message or unexpected behavior are you seeing?",
      "What were you trying to accomplish when this happened?",
      "Can you share the relevant code snippet?",
      "What have you already tried to fix it?"
    ]
  },
  "completion": {
    "summary_format": "## Debug Session Summary\n**Problem**: {problem}\n**Solution**: {solution}\n**Key Steps**: {steps}",
    "return_to_context": true
  }
}
```

## Executor Types

### Command Executor
```json
{
  "type": "command",
  "command": "python3",
  "args": ["script.py", "{param1}", "{param2}"],
  "timeout": 30,
  "working_directory": "./skills/myskill",
  "env": {"KEY": "value"}
}
```

### Inline Code Executor
```json
{
  "type": "inline",
  "language": "python",
  "code": "file://./scripts/inline_calculator.py",
  "timeout": 10
}
```

### HTTP API Executor
```json
{
  "type": "http",
  "method": "POST",
  "url": "https://api.example.com/calculate",
  "headers": {
    "Content-Type": "application/json",
    "Authorization": "Bearer {env.API_KEY}"
  },
  "body": {
    "operation": "{operation}",
    "operands": ["{operand1}", "{operand2}"]
  },
  "timeout": 15
}
```

### Docker Executor
```json
{
  "type": "docker",
  "image": "calculator:latest",
  "command": ["calculate", "{operation}", "{operand1}", "{operand2}"],
  "timeout": 30,
  "volumes": ["./data:/app/data"]
}
```

## File Reference System

Skills support external file references using the `file://` URI scheme:

- **Relative paths**: `file://./prompts/input.md` (relative to skill definition)
- **Absolute paths**: `file:///usr/local/skills/shared/prompt.md`
- **Supported file types**: `.md`, `.txt`, `.py`, `.js`, `.sh`, `.json`

### Directory Structure
```
skills/
├── calculator/
│   ├── calculator.json          # Skill definition
│   ├── prompts/
│   │   ├── input.md
│   │   ├── mapping.md
│   │   └── output.md
│   └── scripts/
│       └── calculator.py
└── debug_helper/
    ├── debug_helper.json
    ├── prompts/
    │   ├── initial.md
    │   └── summary.md
    └── conversation_flow.json
```

## CLI Commands

### Skills Management
```bash
# List all skills (shows aliases by default)
q skills list

# Show detailed skill information
q skills info calculator

# Run a skill with parameters
q skills run calculator --params '{"operation": "add", "operand1": 5, "operand2": 3}'

# Create new skill template
q skills create weather

# Install skill from file or URL
q skills install ./my-skill.json
q skills install https://github.com/user/skill/skill.json
```

### Chat Integration
```bash
# @-syntax with skill name
@calculator add 5 3
@calc 5 + 3
@debug help me with this error

# Slash commands
/skills list
/skills run calculator --params '{"operation": "multiply", "operand1": 4, "operand2": 7}'
/skills info debug_helper
```

## Implementation Details

### Skill Registry
- Central registry manages all loaded skills
- Supports skill aliases for user convenience
- Handles skill discovery and validation
- Manages skill lifecycle (load, execute, unload)

### Prompt Integration
- Skills use prompts to guide input validation and output formatting
- Prompts are processed by Q's language model before skill execution
- Support for both inline prompts and external prompt files

### Context Management
- Code-centric skills return structured results to main context
- Conversational skills maintain separate conversation threads
- Summary mechanism condenses conversation results for main context
- Context size management prevents conversation bloat

### Security Considerations
- Skill execution runs in controlled environment
- File system access restrictions based on skill configuration
- Network access controls for HTTP executors
- Docker container isolation for untrusted code
- User confirmation for potentially dangerous operations

### Error Handling
- Comprehensive error reporting for skill failures
- Timeout handling for long-running operations
- Graceful degradation when skills are unavailable
- User-friendly error messages with troubleshooting hints

## Future Enhancements

### Skill Marketplace
- Central repository for community-contributed skills
- Skill rating and review system
- Automatic updates and dependency management
- Skill publishing and distribution tools

### Advanced Features
- Skill composition and chaining
- Conditional execution based on context
- Integration with external tool ecosystems
- Visual skill builder interface
- Skill performance monitoring and analytics

### Workspace Integration
- Project-specific skill configurations
- Team skill sharing and synchronization
- Integration with development workflows
- Custom skill templates for organizations

## Migration Path

### Phase 1: Core Implementation
- Basic skill registry and execution engine
- Support for command and inline executors
- CLI commands for skill management
- File reference system

### Phase 2: Chat Integration
- @-syntax parsing and execution
- Slash command integration
- Conversational skill support
- Context management improvements

### Phase 3: Advanced Features
- HTTP and Docker executors
- Skill marketplace foundation
- Enhanced security controls
- Performance optimizations

### Phase 4: Ecosystem
- Community skill repository
- Advanced composition features
- Visual tools and interfaces
- Enterprise features and controls
