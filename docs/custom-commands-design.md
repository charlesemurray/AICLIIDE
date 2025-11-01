# Custom Commands System Design

## Overview

Custom `/` commands allow users to create reusable chat commands that execute scripts, functions, or aliases. This system builds on lessons learned from the skills system implementation.

## Architecture

### Core Types
```rust
pub struct CustomCommand {
    pub name: String,
    pub description: String,
    pub handler: CommandHandler,
    pub parameters: Vec<CommandParameter>,
    pub created_at: String,
    pub usage_count: u32,
}

pub enum CommandHandler {
    Script { command: String, args: Vec<String> },
    Alias { target: String },
    Builtin { function_name: String },
}

pub struct CommandParameter {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub default_value: Option<String>,
}
```

### Registry System
```rust
pub struct CustomCommandRegistry {
    commands: HashMap<String, CustomCommand>,
    commands_dir: PathBuf,
}
```

## File Structure
```
.q-commands/
├── hello.json          # Individual command files
├── deploy.json
└── git-status.json
```

## Command Types

### 1. Script Commands
Execute shell commands with parameters:
```bash
q commands create hello --script "echo 'Hello, {{name}}!'" --param name:required
# Usage: /hello --name Alice
```

### 2. Alias Commands  
Shortcut to existing commands:
```bash
q commands create gs --alias "git status"
# Usage: /gs
```

### 3. Builtin Commands
Pre-defined Q functionality:
```bash
q commands create save-context --builtin save_current_context
# Usage: /save-context
```

## CLI Interface

### Creation Commands
```bash
q commands create <name> [OPTIONS]
  --script <command>     # Shell script to execute
  --alias <target>       # Alias to existing command
  --builtin <function>   # Built-in Q function
  --param <name:type>    # Add parameter (required/optional)
  --description <text>   # Command description
```

### Management Commands
```bash
q commands list                    # List all custom commands
q commands show <name>            # Show command details
q commands delete <name>          # Delete command
q commands edit <name>            # Edit command interactively
```

### Execution
```bash
/<command-name> [args]            # Execute custom command
/help <command-name>              # Show command help
```

## Implementation Plan

### Phase 1: Core Infrastructure
1. **Command Types** - Define CustomCommand and CommandHandler structs
2. **Registry** - Load/save commands from `.q-commands/` directory
3. **Basic Execution** - Execute script and alias commands
4. **Unit Tests** - 10+ tests covering core functionality

### Phase 2: CLI Integration
1. **Creation Interface** - `q commands create` with all options
2. **Management Interface** - list, show, delete, edit commands
3. **Chat Integration** - Execute commands with `/command-name` syntax
4. **Integration Tests** - End-to-end command workflows

### Phase 3: Advanced Features
1. **Parameter System** - Handle required/optional parameters with validation
2. **Auto-completion** - Tab completion for custom commands and parameters
3. **Help System** - Generate help text from command metadata
4. **User Acceptance Tests** - Real user scenarios

### Phase 4: Validation
1. **Manual Testing** - CLI interface validation
2. **Performance Testing** - Command lookup and execution speed
3. **Security Review** - Prevent dangerous command execution
4. **Documentation** - User guide and examples

## Security Considerations

### Command Validation
- Restrict dangerous shell commands (rm -rf, etc.)
- Validate parameter inputs to prevent injection
- Sandbox script execution when possible

### Access Control
- Commands are user-specific (stored in user's `.q-commands/`)
- No system-wide command installation
- Clear indication when executing user-defined commands

## Error Handling

### Creation Errors
- Duplicate command names
- Invalid parameter definitions
- Malformed script syntax

### Execution Errors
- Command not found
- Missing required parameters
- Script execution failures
- Permission denied

## Success Criteria

### Functional Requirements
- [ ] Users can create script, alias, and builtin commands
- [ ] Commands persist between Q sessions
- [ ] Commands execute with `/command-name` syntax
- [ ] Parameter validation works correctly
- [ ] Tab completion suggests custom commands
- [ ] Help system shows command documentation

### Quality Requirements
- [ ] All unit tests pass (10+ tests)
- [ ] Integration tests validate end-to-end workflows
- [ ] User acceptance tests cover real scenarios
- [ ] Manual testing confirms CLI usability
- [ ] Performance is acceptable (< 100ms command lookup)
- [ ] Security validation prevents dangerous operations

## Example Usage

### Create and Use Script Command
```bash
# Create command
q commands create deploy --script "git push origin main && kubectl apply -f k8s/" --description "Deploy to production"

# Use command
/deploy
```

### Create and Use Parameterized Command
```bash
# Create command with parameters
q commands create greet --script "echo 'Hello, {{name}}! Today is {{day:-Monday}}'" --param name:required --param day:optional

# Use command
/greet --name Alice --day Friday
```

### Create and Use Alias
```bash
# Create alias
q commands create st --alias "git status --short"

# Use alias
/st
```

This design ensures we build a complete, working system rather than placeholder functionality, following the successful patterns from our skills implementation.
