# Custom Slash Commands Design

## Overview

Custom slash commands extend Q CLI with user-defined system utilities that execute immediately without LLM involvement. They provide direct access to system operations, development tools, and workflow automation while maintaining clear separation from the skills system.

## Core Principles

- **No LLM interaction**: Commands execute directly without AI processing
- **Immediate execution**: Zero-latency system operations
- **System-focused**: File system, git, environment, and development tool operations
- **Always available**: Function even when LLM backend is unavailable
- **Deterministic output**: Consistent, predictable responses

## Architecture

### Command vs Skills Separation

**Custom `/` Commands:**
- **Purpose**: System control, environment inspection, workflow automation
- **Execution**: Direct system calls, no external code execution
- **Latency**: Immediate response (< 50ms)
- **Availability**: Always available, independent of LLM status
- **Output**: Structured system information
- **Examples**: `/ls`, `/git status`, `/env`, `/cd`

**Skills (`@` syntax):**
- **Purpose**: Content processing, problem solving, AI-powered tasks
- **Execution**: Code execution, LLM calls, external APIs
- **Latency**: Variable (100ms - 2s for inline, sessions for complex)
- **Availability**: Depends on executors and LLM availability
- **Output**: Processed content, analysis, generated responses
- **Examples**: `@calculator`, `@summarize`, `@debug_helper`

### Namespace Isolation

**Command Resolution Order:**
1. Built-in slash commands (`/quit`, `/sessions`, `/switch`)
2. Custom slash commands (`/ls`, `/git`, `/env`)
3. Skills system commands (`/skills list`, `/skills create`)
4. Error: command not found

**Conflict Prevention:**
- Custom commands cannot override built-in commands
- Custom commands use different namespace than skills
- Skills cannot register `/` commands
- Clear error messages for conflicts

## Configuration Schema

### Command Definition
```json
{
  "$schema": "https://raw.githubusercontent.com/aws/amazon-q-developer-cli/main/schemas/slash-command-v1.json",
  "command": "ls",
  "description": "List files and directories",
  "aliases": ["dir", "list"],
  "version": "1.0.0",
  "author": "Q CLI Team",
  "handler": {
    "type": "system",
    "action": "list_directory",
    "options": {
      "show_hidden": false,
      "show_details": false,
      "sort_by": "name"
    }
  },
  "parameters": {
    "path": {
      "type": "string",
      "description": "Directory path to list",
      "required": false,
      "default": "."
    },
    "all": {
      "type": "boolean",
      "description": "Show hidden files",
      "required": false,
      "default": false
    }
  }
}
```

### Handler Types

**System Handler:**
```json
{
  "type": "system",
  "action": "list_directory|change_directory|get_environment|get_working_directory",
  "options": {
    "key": "value"
  }
}
```

**Command Handler:**
```json
{
  "type": "command",
  "executable": "git",
  "args": ["status", "--porcelain"],
  "timeout": 5000,
  "working_directory": ".",
  "env": {
    "GIT_PAGER": ""
  }
}
```

**Script Handler:**
```json
{
  "type": "script",
  "language": "bash|python|javascript",
  "script": "file://./scripts/custom_command.sh",
  "timeout": 10000,
  "working_directory": "."
}
```

## Built-in Command Examples

### File System Commands
```json
{
  "command": "ls",
  "description": "List directory contents",
  "handler": {
    "type": "system",
    "action": "list_directory"
  },
  "parameters": {
    "path": {"type": "string", "default": "."},
    "all": {"type": "boolean", "default": false},
    "long": {"type": "boolean", "default": false}
  }
}
```

```json
{
  "command": "cd",
  "description": "Change current directory",
  "handler": {
    "type": "system",
    "action": "change_directory"
  },
  "parameters": {
    "path": {"type": "string", "required": true}
  }
}
```

```json
{
  "command": "pwd",
  "description": "Print working directory",
  "handler": {
    "type": "system",
    "action": "get_working_directory"
  }
}
```

### Git Commands
```json
{
  "command": "git",
  "description": "Git operations",
  "handler": {
    "type": "command",
    "executable": "git"
  },
  "parameters": {
    "subcommand": {"type": "string", "required": true},
    "args": {"type": "array", "default": []}
  }
}
```

### Environment Commands
```json
{
  "command": "env",
  "description": "Show environment variables",
  "handler": {
    "type": "system",
    "action": "get_environment"
  },
  "parameters": {
    "filter": {"type": "string", "required": false}
  }
}
```

## Usage Examples

### Basic File Operations
```bash
> /ls
README.md
src/
tests/
Cargo.toml

> /ls src
main.rs
lib.rs
cli/

> /ls --all
.git/
.gitignore
README.md
src/

> /cd src
✓ Changed directory to src/

> /pwd
/local/workspace/q-cli/amazon-q-developer-cli/src
```

### Git Integration
```bash
> /git status
On branch main
Changes not staged for commit:
  modified: cli/mod.rs
  modified: docs/design.md

> /git branch
* main
  feature/skills
  feature/slash-commands

> /git log --oneline -5
a1b2c3d Add custom slash commands design
e4f5g6h Implement skills system
h7i8j9k Initial commit
```

### Environment Inspection
```bash
> /env
RUST_LOG=debug
CARGO_TARGET_DIR=/tmp/cargo
PATH=/usr/local/bin:/usr/bin:/bin

> /env RUST
RUST_LOG=debug
RUSTC_VERSION=1.70.0

> /which cargo
/usr/local/bin/cargo
```

### Development Tools
```bash
> /cargo check
Checking q-cli v0.1.0
✓ Finished dev [unoptimized + debuginfo] target(s) in 2.3s

> /npm test
Running tests...
✓ 15 tests passed

> /make build
Building project...
✓ Build completed successfully
```

## Command Discovery and Help

### Autocomplete
```bash
> /<TAB>
/ls        List directory contents
/cd        Change directory
/pwd       Print working directory
/git       Git operations
/env       Environment variables
/cargo     Cargo operations

> /git <TAB>
status     Show working tree status
branch     List, create, or delete branches
log        Show commit logs
add        Add file contents to index
commit     Record changes to repository
```

### Help System
```bash
> /ls --help
ls - List directory contents

Usage: /ls [path] [options]

Parameters:
  path     Directory to list (default: current directory)
  --all    Show hidden files
  --long   Show detailed file information

Examples:
  /ls
  /ls src
  /ls --all
  /ls src --long
```

## Development Workflow

### Command Creation
```bash
# Create new command template
q slash-commands create mycommand

# Creates: mycommand.json with basic structure
{
  "command": "mycommand",
  "description": "Custom command description",
  "handler": {
    "type": "system",
    "action": "custom_action"
  }
}
```

### Hot Reloading
- Monitor command definition files for changes
- Automatically reload command configurations
- Validate new commands before activation
- Preserve command history during reloads

### Testing and Validation
```bash
# Test command without installing
q slash-commands test ./mycommand.json

# Validate command configuration
q slash-commands validate ./mycommand.json
✓ Configuration valid
✓ Handler type supported
✓ No conflicts with existing commands

# List all custom commands
q slash-commands list
mycommand    Custom command description
git-helper   Enhanced git operations
project      Project management utilities
```

## Security and Safety

### Execution Limits
- **Timeout enforcement**: Commands killed after timeout
- **Resource limits**: Memory and CPU usage caps
- **Working directory restrictions**: Commands run in safe directories
- **Environment isolation**: Limited environment variable access

### Command Validation
- **Syntax checking**: JSON schema validation
- **Conflict detection**: Prevent override of built-in commands
- **Handler verification**: Ensure handler types are supported
- **Parameter validation**: Type checking and required field validation

### Safe Defaults
- **Read-only operations**: Default to non-destructive commands
- **Confirmation prompts**: Ask before destructive operations
- **Sandboxed execution**: Isolate command execution from Q CLI core
- **Error handling**: Graceful failure without crashing Q CLI

## Integration with Skills System

### Clear Boundaries
- **Different syntax**: `/` for commands, `@` for skills
- **Different purposes**: System control vs content processing
- **Different execution**: Direct system calls vs code execution
- **Different availability**: Always available vs context-dependent

### Complementary Usage
```bash
# Use slash commands to gather system information
> /ls src
main.rs
lib.rs
cli/

> /git status
modified: src/main.rs

# Use skills to process that information
> @code_review analyze the changes in src/main.rs
**Analysis:** The changes in main.rs add error handling for file operations...

# Use slash commands for quick actions
> /git add src/main.rs
✓ Added src/main.rs to staging area
```

### Shared Context
- Commands can provide context for skills
- Skills can suggest relevant commands
- Both systems access same working directory
- Consistent file path handling

## CLI Management Commands

### Command Management
```bash
# List all slash commands
q slash-commands list

# Show command details
q slash-commands info ls

# Create new command
q slash-commands create mycommand

# Install command from file
q slash-commands install ./mycommand.json

# Remove custom command
q slash-commands remove mycommand

# Reload all commands
q slash-commands reload
```

### Command Categories
```bash
# List commands by category
q slash-commands list --category filesystem
q slash-commands list --category git
q slash-commands list --category development

# Search commands
q slash-commands search "directory"
ls       List directory contents
cd       Change directory
mkdir    Create directory
```

## File Structure

### Command Organization
```
slash-commands/
├── filesystem/
│   ├── ls.json
│   ├── cd.json
│   ├── pwd.json
│   └── find.json
├── git/
│   ├── git.json
│   └── git-helper.json
├── development/
│   ├── cargo.json
│   ├── npm.json
│   └── make.json
└── custom/
    ├── mycommand.json
    └── project.json
```

### Configuration Discovery
- Scan `~/.aws/amazonq/slash-commands/` directory
- Load commands from current project directory
- Support for workspace-specific commands
- Hierarchical command loading (global → project → local)

## Error Handling

### Command Execution Errors
```bash
> /git status
❌ Error: git command failed
   Repository not found in current directory

> /cd nonexistent
❌ Error: Directory 'nonexistent' does not exist

> /cargo check
❌ Error: No Cargo.toml found in current directory
   Try: /cd to a Rust project directory
```

### Configuration Errors
```bash
> q slash-commands install ./broken.json
❌ Error: Invalid command configuration
   Line 5: Missing required field 'handler'
   Line 12: Unknown handler type 'invalid'

> q slash-commands create existing
❌ Error: Command 'existing' already exists
   Use --force to override or choose a different name
```

## Performance Considerations

### Execution Speed
- **Direct system calls**: Minimize overhead for common operations
- **Command caching**: Cache command definitions in memory
- **Lazy loading**: Load commands only when needed
- **Parallel execution**: Support concurrent command execution

### Resource Management
- **Timeout enforcement**: Prevent hanging commands
- **Memory limits**: Cap memory usage per command
- **Process cleanup**: Ensure child processes are terminated
- **Error recovery**: Handle command failures gracefully

## Future Enhancements

### Advanced Features
- **Command composition**: Chain multiple commands together
- **Conditional execution**: Execute commands based on conditions
- **Background execution**: Run long commands in background
- **Command history**: Track and replay command sequences

### Integration Improvements
- **IDE integration**: Commands that work with editor state
- **Project awareness**: Commands that understand project structure
- **Tool detection**: Automatically discover available development tools
- **Custom aliases**: User-defined shortcuts for complex commands

### Ecosystem Development
- **Command templates**: Pre-built commands for common workflows
- **Community sharing**: Share custom commands with team members
- **Plugin system**: Extend command functionality with plugins
- **Documentation generation**: Auto-generate help from command definitions

## Migration Path

### Phase 1: Core Implementation
- Basic command registry and execution engine
- System and command handler types
- File system and git commands
- Hot reloading and validation

### Phase 2: Developer Experience
- Command creation and management tools
- Autocomplete and help system
- Error handling and debugging
- Integration with existing Q CLI features

### Phase 3: Advanced Features
- Script handler support
- Command composition and chaining
- Background execution capabilities
- Performance optimization

### Phase 4: Ecosystem
- Community command sharing
- Advanced integration features
- Plugin system development
- Documentation and tooling improvements
