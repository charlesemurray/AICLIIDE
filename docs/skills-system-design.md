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

### Skill Types and Interaction Patterns

Skills are categorized by execution model and interaction pattern:

- **`code_inline`**: Fast execution, stateless, returns immediately (< 1s)
- **`code_session`**: Can maintain state across interactions, session-based
- **`conversation`**: Always session-based with guided workflow and context summarization

### Base Schema Structure

```json
{
  "$schema": "https://raw.githubusercontent.com/aws/amazon-q-developer-cli/main/schemas/skill-v1.json",
  "skill_type": "code_inline|code_session|conversation",
  "metadata": {
    "name": "string",
    "description": "string",
    "aliases": ["string"],
    "version": "string",
    "author": "string"
  }
}
```

### Code Inline Skills (Stateless)

Fast-executing skills that return immediate results without maintaining conversation state.

```json
{
  "$schema": "https://raw.githubusercontent.com/aws/amazon-q-developer-cli/main/schemas/skill-v1.json",
  "skill_type": "code_inline",
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
    "type": "inline",
    "language": "python",
    "code": "file://./scripts/calculator.py",
    "timeout": 1000
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

### Code Session Skills (Stateful)

Code-executing skills that can maintain state across multiple interactions within a session.

```json
{
  "$schema": "https://raw.githubusercontent.com/aws/amazon-q-developer-cli/main/schemas/skill-v1.json",
  "skill_type": "code_session",
  "metadata": {
    "name": "data_analyzer",
    "description": "Interactive data analysis with persistent state",
    "aliases": ["analyze", "data"],
    "version": "1.0.0",
    "author": "Q CLI Team"
  },
  "prompt": {
    "input_validation": "file://./prompts/data_input.md",
    "argument_mapping": "file://./prompts/data_mapping.md",
    "output_formatting": "Format analysis results with charts and summaries"
  },
  "executor": {
    "type": "command",
    "command": "python3",
    "args": ["file://./scripts/data_analyzer.py", "--session", "{session_id}"],
    "timeout": 5000,
    "working_directory": "./skills/data_analyzer"
  },
  "session": {
    "max_duration": 3600000,
    "cleanup_on_exit": true,
    "state_persistence": "memory"
  }
}
```

### Conversational Skills (Session-Based)

Guided conversation skills that maintain context and provide structured problem-solving workflows.

```json
{
  "$schema": "https://raw.githubusercontent.com/aws/amazon-q-developer-cli/main/schemas/skill-v1.json",
  "skill_type": "conversation",
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
  "session": {
    "max_duration": 1800000,
    "cleanup_on_exit": true,
    "state_persistence": "memory"
  },
  "completion": {
    "summary_format": "## Debug Session Summary\n**Problem**: {problem}\n**Solution**: {solution}\n**Key Steps**: {steps}",
    "return_to_context": true
  }
}
```

### Configuration Constraints

**Valid Combinations:**
- `code_inline`: Must have `executor`, cannot have `conversation` or `session`
- `code_session`: Must have `executor` and `session`, cannot have `conversation`  
- `conversation`: Must have `conversation` and `session`, cannot have `executor`

**Invalid Configurations:**
```json
// ❌ Invalid: inline skills cannot have sessions
{
  "skill_type": "code_inline",
  "executor": { ... },
  "session": { ... }  // Error: inline skills are stateless
}

// ❌ Invalid: conversations cannot have executors
{
  "skill_type": "conversation", 
  "executor": { ... },  // Error: conversations don't execute code
  "conversation": { ... }
}

// ❌ Invalid: code skills cannot have conversation config
{
  "skill_type": "code_session",
  "executor": { ... },
  "conversation": { ... }  // Error: code skills don't use conversation flow
}
```

## Executor Types

### Inline Executor (Code Inline Skills Only)
```json
{
  "type": "inline",
  "language": "python|javascript|bash",
  "code": "file://./scripts/calculator.py",
  "timeout": 1000
}
```

### Command Executor (Code Skills)
```json
{
  "type": "command",
  "command": "python3",
  "args": ["script.py", "{param1}", "{param2}"],
  "timeout": 5000,
  "working_directory": "./skills/myskill",
  "env": {"KEY": "value"}
}
```

### MCP Executor (Code Skills)
```json
{
  "type": "mcp",
  "server": "weather-mcp",
  "tool": "get_current_weather",
  "timeout": 5000,
  "mapping": {
    "location": "{location}",
    "units": "{units}"
  }
}
```

### Auto-Delegation (Long-Running Tasks)
```json
{
  "type": "delegate",
  "agent": "data-processor",
  "timeout": 300000,
  "delegation_threshold": 2000
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

**Inline Skills (Immediate Response):**
```bash
# Natural language input, immediate response
> Calculate 15% of 250
37.5

> What's the weather in Seattle?
Current weather: 52°F, Cloudy

> Convert 100 fahrenheit to celsius  
100°F = 37.8°C
```

**Session Skills (Multi-turn Conversations):**
```bash
# Starting a session skill
> I need help debugging database performance
🔍 Started debug session (debug-1)
What database system are you using?

> PostgreSQL
🔍 [debug-1] What specific queries are slow?

# Multiple concurrent sessions
> Let me also plan a migration
📋 Started planning session (plan-1)  
What system are you migrating from?

# Session switching
> /switch debug-1
🔍 [debug-1] You mentioned slow queries...

# Session management
> /sessions
Active sessions:
  🔍 debug-1    Database debugging (3 messages)
  📋 plan-1     Migration planning (1 message)
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
