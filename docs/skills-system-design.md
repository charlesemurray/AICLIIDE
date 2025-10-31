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
- **`prompt_inline`**: LLM-powered transformations with conversation context access

### Base Schema Structure

```json
{
  "$schema": "https://raw.githubusercontent.com/aws/amazon-q-developer-cli/main/schemas/skill-v1.json",
  "skill_type": "code_inline|code_session|conversation|prompt_inline",
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

### Prompt Inline Skills (Context-Aware)

LLM-powered skills that transform input using conversation context and return immediate results.

```json
{
  "$schema": "https://raw.githubusercontent.com/aws/amazon-q-developer-cli/main/schemas/skill-v1.json",
  "skill_type": "prompt_inline",
  "metadata": {
    "name": "summarize",
    "description": "Summarize conversation or provided text",
    "aliases": ["summary", "tldr"],
    "version": "1.0.0",
    "author": "Q CLI Team"
  },
  "prompt": {
    "system_prompt": "file://./prompts/summarize_system.md",
    "user_template": "Summarize this content: {user_input}\n\nContext from conversation: {conversation_history}",
    "max_tokens": 500,
    "temperature": 0.3
  },
  "context": {
    "conversation": true,
    "max_history_messages": 10,
    "workspace": false,
    "environment": false
  }
}
```

**Usage Examples:**
```bash
# Summarize recent conversation
> @summarize
**Summary:** Discussed database performance issues, identified missing index on users.created_at, provided solution with CREATE INDEX command.

# Summarize provided content
> @summarize "We discussed three main issues: database performance, API latency, and memory leaks."
**Summary:** Three technical issues addressed - database performance, API latency, and memory leaks.

# Code explanation with conversation context
> @explain this function
def fibonacci(n):
    if n <= 1: return n
    return fibonacci(n-1) + fibonacci(n-2)

**Explanation:** This is a recursive Fibonacci function. Given our earlier discussion about performance, note that this has O(2^n) time complexity and would benefit from memoization.
```

### Configuration Constraints

**Valid Combinations:**
- `code_inline`: Must have `executor`, cannot have `conversation`, `session`, or `prompt`
- `code_session`: Must have `executor` and `session`, cannot have `conversation` or `prompt`  
- `conversation`: Must have `conversation` and `session`, cannot have `executor` or `prompt`
- `prompt_inline`: Must have `prompt`, cannot have `executor`, `conversation`, or `session`

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

// ❌ Invalid: prompt skills cannot have executors
{
  "skill_type": "prompt_inline",
  "executor": { ... },  // Error: prompt skills use LLM, not code execution
  "prompt": { ... }
}

// ❌ Invalid: prompt skills cannot have sessions
{
  "skill_type": "prompt_inline",
  "prompt": { ... },
  "session": { ... }  // Error: prompt skills return immediate results
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

## Context Access Configuration

Skills can optionally access various types of context:

```json
{
  "context": {
    "conversation": true,           // Access recent conversation history
    "max_history_messages": 10,    // Limit conversation context size
    "workspace": true,              // Access current directory, git status
    "environment": true,            // Access environment variables, PATH
    "files": ["*.py", "*.js"]       // Access specific file patterns
  }
}
```

**Context Injection Methods:**
- **Environment variables**: `SKILL_WORKSPACE_DIR`, `SKILL_GIT_BRANCH`
- **Template variables**: `{conversation_history}`, `{current_directory}`
- **Parameter injection**: Context passed as structured parameters to executors

### Chat Integration

**Inline Skills (Immediate Response):**
```bash
# Code inline skills
> Calculate 15% of 250
37.5

> What's the weather in Seattle?
Current weather: 52°F, Cloudy

# Prompt inline skills  
> @summarize the last few messages
**Summary:** Discussed database performance optimization, identified indexing solution, and provided CREATE INDEX command.

> @explain this code: def fib(n): return n if n <= 1 else fib(n-1) + fib(n-2)
**Explanation:** Recursive Fibonacci function with O(2^n) complexity. Consider memoization for better performance.
```

**Session Skills (Multi-turn Conversations):**
```bash
# Starting a named session skill
> @debug_helper database_performance
🔍 Started debug session (database_performance)
What database system are you using?

> PostgreSQL
🔍 [database_performance] What specific queries are slow?

# Multiple concurrent sessions
> @planning_helper feature_roadmap
📋 Started planning session (feature_roadmap)  
What's the main goal of this feature?

# Session switching
> /switch database_performance
🔍 [database_performance] You mentioned slow queries...

# Session management
> /sessions
Active sessions:
  🔍 database_performance    Database debugging (3 messages)
  📋 feature_roadmap         Feature planning (1 message)

# Named session continuation
> /switch feature_roadmap
📋 [feature_roadmap] What's the main goal of this feature?
```

### Skill Discovery and Help

**Contextual Recommendations:**
```bash
# After discussing math problems
> I need to calculate compound interest
💡 Try @calculator for arithmetic operations
💡 Try @summarize to recap our discussion

# Skill help system
> @calculator help
Calculator skill - Perform arithmetic operations
Usage: @calculator <operation> <number1> <number2>
Operations: add, subtract, multiply, divide
Examples:
  @calculator add 5 3
  @calculator multiply 4.5 2
  
# List skills by type
> /skills list
Available skills:
  calculator (calc, math)     Arithmetic operations [code_inline]
  weather                     Weather information [code_inline]
  summarize (summary, tldr)   Summarize content [prompt_inline]
  debug_helper (debug)        Debug code issues [conversation]
```

## Development Workflow

### Hot Reloading System

Skills support hot reloading for rapid development iteration without restarting Q CLI.

**File Watching:**
- Monitors skill directories for changes to `.json`, `.py`, `.js`, `.md` files
- Automatically reloads skill definitions, scripts, and prompts
- Preserves active sessions during reloads
- Validates new configurations before replacing existing skills

**Development Cycle:**
```bash
# Create new skill template
q skills create weather --type code_inline
# Creates: weather.json, scripts/weather.py, prompts/

q skills create summarize --type prompt_inline  
# Creates: summarize.json, prompts/system.md, prompts/user_template.md

# Edit skill files in your preferred editor
vim weather.json          # Update configuration
vim scripts/weather.py    # Modify execution logic
vim prompts/system.md     # Refine prompts

# Q CLI automatically detects and reloads changes
✓ Reloaded skill: weather (config updated)
✓ Reloaded skill: summarize (prompt updated)

# Test immediately without restart
> @weather Seattle
Current weather: 52°F, Cloudy

> @summarize our conversation
**Summary:** Created weather and summarize skills, tested hot reloading functionality.
```

**Reload Behavior:**
- **Configuration changes**: Update skill metadata, parameters, aliases
- **Script changes**: Reload executor code and dependencies
- **Prompt changes**: Refresh system prompts and user templates
- **Error handling**: Display reload errors, maintain previous working version on failure
- **Session preservation**: Active skill sessions continue uninterrupted during reloads

**Developer Feedback:**
```bash
✓ Reloaded skill: calculator (script updated)
✓ Reloaded skill: summarize (prompt template updated)
❌ Failed to reload weather: syntax error in weather.json line 12
   → Keeping previous version active
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
├── summarize/
│   ├── summarize.json           # Prompt skill definition
│   └── prompts/
│       ├── system.md
│       └── user_template.md
└── debug_helper/
    ├── debug_helper.json
    ├── prompts/
    │   ├── initial.md
    │   └── summary.md
    └── conversation_flow.json
```

## Resource Management

**Per-Skill Resource Limits:**
```json
{
  "executor": {
    "type": "inline",
    "code": "file://./heavy_task.py",
    "resources": {
      "memory_mb": 100,        // Max 100MB RAM
      "cpu_seconds": 2,        // Max 2 CPU seconds  
      "timeout_ms": 5000,      // Kill after 5s wall time
      "temp_files_mb": 50      // Max temp file usage
    }
  }
}
```

**Prompt Skill Limits:**
```json
{
  "prompt": {
    "system_prompt": "file://./system.md",
    "user_template": "Analyze: {user_input}",
    "max_tokens": 1000,       // Limit response length
    "timeout_ms": 10000       // LLM request timeout
  }
}
```

### Skill Testing and Validation

**Local Testing:**
```bash
# Test skill without installing
q skills test ./weather.json

# Validate skill configuration
q skills validate ./weather.json
✓ Configuration valid
✓ All referenced files exist
✓ Executor dependencies available
❌ Warning: timeout value seems high (30s)

# Test different skill types
q skills test ./summarize.json
✓ Prompt skill valid
✓ System prompt file exists
✓ Template variables properly defined

# Dry run skill execution
q skills run weather Seattle --dry-run
q skills run summarize "test content" --dry-run
```

**Basic Error Handling:**
- **Skill crashes**: Isolate failures, don't crash Q CLI
- **Missing dependencies**: Clear error messages with installation hints
- **File not found**: Helpful errors when skill files are missing/moved
- **Timeout handling**: Graceful termination of stuck skills
- **Configuration errors**: Syntax validation with line numbers
- **LLM failures**: Fallback behavior for prompt skills when LLM is unavailable

## CLI Commands

### Skills Management
```bash
# List all skills (shows aliases and types by default)
q skills list

# Show detailed skill information
q skills info calculator

# Run a skill with parameters
q skills run calculator add 5 3
q skills run summarize "long text to summarize"

# Create new skill templates
q skills create weather --type code_inline
q skills create explain --type prompt_inline
q skills create debug --type conversation

# Test and validate skills
q skills test ./my-skill.json
q skills validate ./my-skill.json

# Reload specific skill manually
q skills reload calculator
```
## Future Enhancements

### Skill Sharing and Distribution
- Simple export/import for sharing skills with friends
- Git-based skill repositories for version control
- Skill templates for common patterns (API wrappers, file processors)
- Dependency management for Python packages and Node modules

### Advanced Features
- Skill composition and chaining
- Conditional execution based on context
- Background processing for long-running tasks
- Skill performance monitoring and caching

### Workspace Integration
- Project-specific skill configurations
- Skills that understand current development environment
- Integration with existing CLI tools and workflows
- Custom skill templates for specific project types

## Migration Path

### Phase 1: Core Implementation
- Four skill types: code_inline, code_session, conversation, prompt_inline
- Basic skill registry and execution engine
- Hot reloading system for development
- File reference system and resource management

### Phase 2: Chat Integration
- Natural language skill invocation
- Session management with named sessions
- Context access (conversation, workspace, environment)
- Autocomplete and help system

### Phase 3: Developer Experience
- Skill testing and validation framework
- Template generation for all skill types
- Error handling and debugging tools
- Performance optimization and caching

### Phase 4: Ecosystem
- Skill sharing mechanisms
- Advanced context integration
- Skill composition features
- Community templates and examples
