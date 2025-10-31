# Development Sessions Design

## Overview

Development sessions provide dedicated, isolated environments for creating and iterating on skills, slash commands, and agent configurations without polluting the main conversation context. These sessions include built-in testing frameworks and development workflows to enable complete in-chat development experiences.

## Core Principles

- **Context Isolation**: Development work happens in separate sessions from main conversation
- **Complete Testing Environment**: Full testing capabilities within development sessions
- **Iterative Development**: Create, test, improve cycle without leaving the session
- **Expert Guidance**: LLM provides specialized development assistance
- **Seamless Integration**: Developed components integrate smoothly with main Q CLI experience

## Session Types

### Skill Development Sessions
Dedicated environments for creating and testing skills of all types (code_inline, code_session, conversation, prompt_inline).

### Command Development Sessions  
Focused environments for building and testing custom slash commands with various handler types.

### Agent Development Sessions
Specialized sessions for creating and configuring agent definitions with proper testing workflows.

## Development Session Workflow

### Session Initiation
```bash
# Main conversation
> I need a skill to convert temperatures

🔧 Starting skill development session (dev-temperature)
[dev-temperature] I'll help you create a temperature conversion skill.
[dev-temperature] What temperature units should it support?

> Fahrenheit, Celsius, and Kelvin
[dev-temperature] What should the input format look like?

> Natural language like "100f to c" or "32 celsius to kelvin"
[dev-temperature] 🔧 Creating temperature conversion skill...
[dev-temperature] ✓ Generated skill configuration
[dev-temperature] ✓ Created conversion logic  
[dev-temperature] ✓ Added natural language parsing
[dev-temperature] Ready for testing
```

### In-Session Testing
```bash
[dev-temperature] > test @temperature 100f to c
[dev-temperature] 100°F = 37.8°C
[dev-temperature] ✓ Test passed

[dev-temperature] > test @temperature 0 kelvin to fahrenheit  
[dev-temperature] 0K = -459.67°F
[dev-temperature] ✓ Test passed

[dev-temperature] > test @temperature invalid input
[dev-temperature] Error: Could not parse temperature input
[dev-temperature] ✓ Error handling works
```

### Iterative Improvement
```bash
[dev-temperature] The error message could be more helpful

[dev-temperature] 🔧 Improving error messages...
[dev-temperature] ✓ Enhanced input validation
[dev-temperature] ✓ Added usage examples in errors

[dev-temperature] > test @temperature invalid input
[dev-temperature] Error: Could not parse temperature input.
[dev-temperature] Try formats like: "100f to c", "32 celsius to kelvin", "273k to fahrenheit"
[dev-temperature] ✓ Improved error message
```

### Session Completion
```bash
[dev-temperature] Skill looks good! Should I finalize it?

> Yes, make it available
[dev-temperature] Where should I install this skill?
[dev-temperature] 1. Workspace only (current project)
[dev-temperature] 2. Global (available everywhere)

> Workspace only
[dev-temperature] ✓ Skill development complete
[dev-temperature] ✓ @temperature skill installed in workspace
[dev-temperature] ✓ Available in this project only
✓ Closed development session (dev-temperature)

# Alternative global installation
> Global
[dev-temperature] ✓ Skill development complete  
[dev-temperature] ✓ @temperature skill installed globally
[dev-temperature] ✓ Available in all Q CLI sessions
✓ Closed development session (dev-temperature)
```

## Scope Management

### Workspace vs Global Installation

**Workspace Scope:**
- Skills/commands available only in current project directory
- Stored in `./.qcli/skills/` or `./.qcli/commands/`
- Project-specific configurations and dependencies
- Team sharing through version control
- Automatic activation when in project directory

**Global Scope:**
- Skills/commands available in all Q CLI sessions
- Stored in `~/.aws/amazonq/skills/` or `~/.aws/amazonq/commands/`
- Personal productivity tools and utilities
- Available across all projects and directories

### Installation Options
```bash
[dev-git-helper] Ready to install the git helper command. Choose scope:

# Quick selection
> workspace
[dev-git-helper] ✓ Installing /git-helper in current workspace

> global  
[dev-git-helper] ✓ Installing /git-helper globally

# Explicit specification during development
[dev-database-skill] This skill uses project-specific database configs.
[dev-database-skill] Recommending workspace installation.
[dev-database-skill] Install in workspace? (y/n)

> y
[dev-database-skill] ✓ @database skill installed in workspace
```

### Scope-Aware Development
```bash
[dev-project-manager] I see this skill accesses Cargo.toml and package.json
[dev-project-manager] This appears to be project-specific functionality.
[dev-project-manager] Should I configure it for workspace installation?

> Yes, and make it work with different project types
[dev-project-manager] ✓ Added multi-project-type support
[dev-project-manager] ✓ Configured for workspace scope
[dev-project-manager] This skill will adapt to each project's structure
```

### Scope Migration
```bash
# Promote workspace skill to global
> /skills promote @database global
✓ Moved @database skill from workspace to global scope
✓ Now available in all Q CLI sessions

# Demote global skill to workspace
> /skills demote @project-helper workspace  
✓ Moved @project-helper skill from global to workspace scope
✓ Now available only in current project

# Copy skill between scopes
> /skills copy @calculator workspace
✓ Copied @calculator skill to workspace
✓ Available in both global and workspace scope
```

## Testing Framework

### Direct Testing Commands
Available within development sessions:

```bash
# Test skills
> test @skillname param1 param2
> test @skillname "complex input with spaces"

# Test slash commands  
> test /commandname --option value
> test /commandname arg1 arg2

# Test agents
> test agent "sample input for agent"
> test agent --scenario "specific test case"
```

### Performance Testing
```bash
# Measure execution time
> test performance @skillname input
Response time: 0.3s
✓ Performance target met (<1s)

# Benchmark with multiple iterations
> benchmark @skillname 50 iterations
Average response time: 0.28s
Min: 0.21s, Max: 0.45s
✓ Consistent performance
```

### Automated Test Suites
```bash
# Run comprehensive tests
> run test suite
Running automated tests for calculator skill...

✓ Basic arithmetic: add, subtract, multiply, divide
✓ Edge cases: zero, negative numbers, decimals  
✓ Error handling: division by zero, invalid input
✓ Input validation: non-numeric inputs, empty input
✓ Performance: all operations <100ms

Test Results: 5/5 passed
```

### Validation Testing
```bash
# Configuration validation
> validate configuration
✓ JSON schema valid
✓ All referenced files exist
✓ Handler type supported
✓ Parameters properly defined

# Dependency checking
> check dependencies
✓ Python 3.9+ available
✓ Required packages installed
✓ File permissions correct
❌ Warning: API key not configured
```

## Session Management

### Active Session Tracking
```bash
> /dev-sessions
Active development sessions:
  🔧 dev-temperature     Temperature skill (testing phase)
  🔧 dev-git-helper      Git command (iteration 2)
  🔧 dev-code-agent      Code review agent (configuration)

> /switch dev-git-helper
[dev-git-helper] Continuing git helper command development...

> /close dev-temperature
✓ Temperature skill development complete
✓ Skill @temperature is now available
```

### Session Context Preservation
- Development sessions maintain full context across switches
- Test results and iteration history preserved
- Configuration changes tracked with version history
- Performance metrics retained for optimization
- Dependency and permission changes logged

### Session State Management
```bash
# Session state storage
~/.aws/amazonq/dev-sessions/
├── active/
│   ├── dev-calculator.json
│   └── dev-weather.json
├── completed/
│   ├── dev-database-helper.json
│   └── dev-git-tools.json
└── archived/
    └── old-sessions/

# Session cleanup
> /cleanup dev-sessions --completed --older-than 7d
✓ Archived 5 completed development sessions
✓ Freed 12MB of session data

# Session recovery
> /recover dev-session dev-calculator
✓ Recovered development session from backup
✓ Restored 3 test iterations and configuration history
```

## Development Session Features

### Context-Aware Development
```bash
[dev-project-helper] I see you're in a Rust project with Cargo.toml
[dev-project-helper] Creating project management skill with Rust-specific features...
[dev-project-helper] ✓ Added cargo build integration
[dev-project-helper] ✓ Added dependency management
[dev-project-helper] ✓ Added test runner integration
[dev-project-helper] 
[dev-project-helper] Checking dependencies...
[dev-project-helper] ✓ Rust 1.70.0 found (compatible)
[dev-project-helper] ✓ Cargo available
[dev-project-helper] ⚠️  clippy not installed (recommended for code quality)
[dev-project-helper] 
[dev-project-helper] Install clippy? (y/n)
> y
[dev-project-helper] ✓ Installing clippy...
[dev-project-helper] ✓ Project helper configured for Rust development
```

### Intelligent Suggestions and Safety
```bash
[dev-api-skill] I notice you have a .env file with API keys
[dev-api-skill] Should I configure the skill to use environment variables for authentication?

> Yes
[dev-api-skill] ✓ Added environment variable support
[dev-api-skill] ✓ Configured secure API key handling
[dev-api-skill] 
[dev-api-skill] Security check: This skill will access:
[dev-api-skill] - Environment variable: API_KEY
[dev-api-skill] - Network access: api.weather.com
[dev-api-skill] - File read: ./.env (for fallback)
[dev-api-skill] 
[dev-api-skill] Permissions look reasonable. Install in workspace for security isolation?
> Yes
[dev-api-skill] ✓ Configured for workspace scope
[dev-api-skill] ✓ API keys will be isolated per project
```

### Error-Driven Development with Diagnostics
```bash
[dev-database-skill] > test @db_query "SELECT * FROM users"
[dev-database-skill] Error: Connection refused - database not running
[dev-database-skill] 
[dev-database-skill] 🔍 Diagnosing database connection...
[dev-database-skill] ✓ Database configuration found
[dev-database-skill] ❌ PostgreSQL not running on localhost:5432
[dev-database-skill] ✓ Database credentials valid
[dev-database-skill] 
[dev-database-skill] I can add connection resilience and better error handling.
[dev-database-skill] Should I implement that?

> Yes
[dev-database-skill] 🔧 Adding connection resilience...
[dev-database-skill] ✓ Added retry logic with exponential backoff
[dev-database-skill] ✓ Enhanced error messages with troubleshooting hints
[dev-database-skill] ✓ Added connection health checks
[dev-database-skill] ✓ Added fallback to read-only mode when write fails
```

## Development Workflows

### Test-Driven Development
```bash
[dev-calculator] Let's start with test cases. What calculations should it handle?

> Basic arithmetic and scientific functions
[dev-calculator] Creating test suite first...
[dev-calculator] ✓ Defined test cases for +, -, *, /, sqrt, pow
[dev-calculator] ✓ Added edge case tests
[dev-calculator] Now implementing to pass these tests...
[dev-calculator] ✓ Basic arithmetic implemented
[dev-calculator] ❌ Scientific functions failing - implementing now...
[dev-calculator] ✓ All tests now passing
```

### Incremental Feature Development
```bash
[dev-weather] > test @weather Seattle
[dev-weather] Current weather: 52°F, Cloudy
[dev-weather] ✓ Basic functionality works

[dev-weather] Should I add forecast capability?

> Yes, 5-day forecast
[dev-weather] 🔧 Adding forecast feature...
[dev-weather] ✓ Extended API integration
[dev-weather] ✓ Added forecast formatting

[dev-weather] > test @weather forecast Seattle 5 days
[dev-weather] 5-day forecast for Seattle:
[dev-weather] Today: 52°F, Cloudy
[dev-weather] Tomorrow: 48°F, Rain
[dev-weather] ...
[dev-weather] ✓ Forecast feature working
```

### Configuration Iteration
```bash
[dev-code-agent] > test agent "review this Python code"
[dev-code-agent] Agent response too verbose - 500 words
[dev-code-agent] ❌ Response too long for practical use

[dev-code-agent] 🔧 Adjusting agent configuration...
[dev-code-agent] ✓ Added conciseness instructions
[dev-code-agent] ✓ Limited response length

[dev-code-agent] > test agent "review this Python code"  
[dev-code-agent] Agent response: 3 key issues identified in 50 words
[dev-code-agent] ✓ Response length appropriate
```

## Integration with Main Systems

### Skills System Integration
- Development sessions create skills that integrate seamlessly with existing skills registry
- Hot reloading ensures immediate availability in main chat
- Configuration validation prevents conflicts with existing skills
- Scope-aware installation (workspace vs global)
- Automatic skill discovery based on current directory

### Slash Commands Integration  
- Custom commands developed in sessions integrate with existing command system
- Namespace validation prevents conflicts with built-in commands
- Immediate availability after session completion
- Workspace-specific commands override global ones when available
- Scope migration tools for promoting/demoting command availability

### Agent System Integration
- Agent configurations created in development sessions integrate with existing agent management
- Validation ensures compatibility with Q CLI agent system
- Seamless switching between developed agents and existing ones
- Project-specific agent configurations for workspace scope
- Global agent availability for cross-project use cases

### File Structure and Discovery

**Workspace Structure:**
```
project-root/
├── .qcli/
│   ├── skills/
│   │   ├── project-helper.json
│   │   └── database-analyzer.json
│   ├── commands/
│   │   ├── build.json
│   │   └── deploy.json
│   └── agents/
│       └── code-reviewer.json
├── src/
└── README.md
```

**Global Structure:**
```
~/.aws/amazonq/
├── skills/
│   ├── calculator.json
│   └── weather.json
├── commands/
│   ├── git-helper.json
│   └── system-info.json
└── agents/
    └── general-assistant.json
```

**Discovery Priority:**
1. Workspace-specific skills/commands (`./.qcli/`)
2. Global skills/commands (`~/.aws/amazonq/`)
3. Built-in skills/commands

## Security and Safety in Development Sessions

### Sandboxed Development Environment
- Development sessions run in isolated environments
- Test execution cannot affect main Q CLI functionality or system
- Resource limits prevent runaway processes during testing
- Temporary files cleaned up automatically after session completion

### Permission Validation During Development
```bash
[dev-file-processor] This skill will need permissions:
[dev-file-processor] 📁 Read: ./data/, ./config/
[dev-file-processor] 📁 Write: ./output/, ./logs/
[dev-file-processor] 🌐 Network: api.example.com
[dev-file-processor] 
[dev-file-processor] These permissions will be requested when skill is first used.
[dev-file-processor] Permissions look reasonable for a file processing skill.
```

### Configuration Validation and Safety Checks
```bash
[dev-deploy-cmd] ⚠️  Security review for deploy command:
[dev-deploy-cmd] - Executes system commands (docker, kubectl)
[dev-deploy-cmd] - Network access to production systems
[dev-deploy-cmd] - Modifies deployment configurations
[dev-deploy-cmd] 
[dev-deploy-cmd] This is a high-risk command. Recommendations:
[dev-deploy-cmd] 1. Add confirmation prompts for production deployments
[dev-deploy-cmd] 2. Require explicit approval for destructive operations
[dev-deploy-cmd] 3. Add rollback capabilities
[dev-deploy-cmd] 
[dev-deploy-cmd] Implement safety measures? (y/n)
> y
[dev-deploy-cmd] ✓ Added confirmation prompts
[dev-deploy-cmd] ✓ Added rollback support
[dev-deploy-cmd] ✓ Added deployment validation checks
```

### Dependency Security Scanning
```bash
[dev-api-client] Checking dependencies for security issues...
[dev-api-client] ✓ requests==2.28.1 (no known vulnerabilities)
[dev-api-client] ⚠️  urllib3==1.26.5 (1 medium severity vulnerability)
[dev-api-client] ❌ pyyaml==5.4.1 (1 high severity vulnerability)
[dev-api-client] 
[dev-api-client] Update vulnerable dependencies? (y/n)
> y
[dev-api-client] ✓ Updated urllib3 to 1.26.18 (vulnerability fixed)
[dev-api-client] ✓ Updated pyyaml to 6.0.1 (vulnerability fixed)
[dev-api-client] ✓ All dependencies now secure
```

## Future Enhancements

### Advanced Testing Features
- Integration testing with external services
- Load testing for performance validation
- Regression testing for configuration changes
- A/B testing for different implementations

### Collaborative Development
- Shared development sessions for team collaboration
- Version control integration for configuration tracking
- Code review workflows within development sessions
- Template sharing and reuse

### Enhanced Development Tools
- Visual configuration builders
- Interactive debugging capabilities
- Performance profiling and optimization suggestions
- Automated documentation generation

## Migration Path

### Phase 1: Basic Development Sessions
- Implement session isolation and management
- Basic testing framework for skills and commands
- Simple create-test-iterate workflow
- Integration with existing systems

### Phase 2: Advanced Testing
- Automated test suite generation
- Performance testing and benchmarking
- Configuration validation and dependency checking
- Error-driven development workflows

### Phase 3: Enhanced Development Experience
- Context-aware development assistance
- Intelligent suggestions and optimizations
- Advanced testing scenarios and edge cases
- Collaborative development features

### Phase 4: Complete Development Environment
- Visual development tools
- Advanced debugging and profiling
- Template and pattern libraries
- Community sharing and collaboration features
