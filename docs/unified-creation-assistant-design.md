# Unified Creation Assistant Design

## Overview

The Unified Creation Assistant provides a single, consistent interface for creating all types of artifacts in the Q CLI system: Skills, Custom Commands, and Agents. This design eliminates code duplication, provides consistent user experience, and makes adding new creation types trivial.

## Architecture

### Core Components

```rust
pub enum CreationType {
    Skill(SkillType),
    CustomCommand,
    Agent,
}

pub enum CreationState {
    Discovery,      // Understanding what user wants to build
    Configuration,  // Setting up parameters and details
    Testing,        // Testing functionality (skills only)
    Completion,     // Finalizing and saving
}

pub struct UnifiedCreationAssistant {
    creation_type: CreationType,
    state: CreationState,
    name: String,
    description: String,
    parameters: Vec<Parameter>,
    // Type-specific fields
}
```

### Workflow Phases

1. **Discovery Phase**
   - Understand what the user wants to create
   - Determine creation type and initial configuration
   - Collect high-level description

2. **Configuration Phase**
   - Gather type-specific details
   - Configure commands, prompts, or instructions
   - Detect and configure parameters

3. **Testing Phase** (Skills only)
   - Test prompts and functionality
   - Validate expected outputs
   - Iterate on configuration

4. **Completion Phase**
   - Final review of configuration
   - User confirmation
   - Save artifact to appropriate location

## Creation Types

### Custom Commands ✅ WELL COVERED

**Discovery Prompts:**
- "What should this command do?"
- Examples: 'run git status', 'deploy to staging', 'create alias for ls -la'

**Type Detection:**
- Contains "alias" → Alias command
- Contains "builtin"/"save"/"context" → Builtin command
- Default → Script command

**Configuration:**
- Script: Shell command with {{param}} substitution
- Alias: Target command to alias
- Builtin: Function name (save_context, clear_context, show_stats)

**Parameters:**
- Automatic detection of {{param}} syntax
- Required vs optional configuration
- Description and validation

**Security:**
- Dangerous command validation (rm -rf, sudo rm, dd, etc.)
- Parameter sanitization
- Script safety checks

### Skills ⚠️ REQUIRES COMPREHENSIVE COVERAGE

**Discovery Prompts:**
- "What are you trying to accomplish with this skill?"
- Type-specific guidance based on SkillType

**Skill Types (4 distinct types with unique requirements):**
- **CodeInline**: Execute commands/scripts inline
  - Command or script configuration
  - Parameter handling
  - Output processing
- **PromptInline**: Template-based text generation
  - Template text with variable substitution
  - Context injection
  - Output formatting
- **Conversation**: Interactive chat assistants
  - System prompt configuration
  - Conversation flow management
  - Context retention
- **CodeSession**: REPL-style interactive sessions
  - Session initialization
  - State management
  - Multi-turn interactions

**Security Configuration (CRITICAL - Missing from current design):**
```rust
pub struct SecurityConfig {
    pub permissions: Option<Permissions>,
    pub resource_limits: Option<ResourceLimits>,
}

pub struct Permissions {
    pub file_read: Option<Vec<String>>,     // File read permissions
    pub file_write: Option<Vec<String>>,    // File write permissions  
    pub network_access: Option<bool>,       // Network access control
}

pub struct ResourceLimits {
    pub max_memory_mb: Option<u32>,         // Memory usage limits
    pub max_execution_time: Option<u32>,    // Execution time limits
    pub max_cpu_percent: Option<u32>,       // CPU usage limits
}
```

**Testing Phase (CRITICAL - Barely covered):**
- **Prompt Testing**: Test prompts with sample inputs
- **Output Validation**: Verify expected outputs
- **Iterative Refinement**: Improve based on test results
- **Security Validation**: Test within resource limits
- **Permission Testing**: Validate file/network access

**Configuration Complexity:**
- JSON-based skill definitions
- Parameter validation and type checking
- Context file integration
- Environment variable handling

### Agents ⚠️ COMPLEX SYSTEM - COMPREHENSIVE REQUIREMENTS

**Agent System Architecture (From existing documentation):**
- **JSON-based configuration** with 12+ distinct fields
- **MCP (Model Context Protocol) integration** with server management
- **Hook system** with 5 different lifecycle events
- **Tool management** with wildcards, aliases, and permissions
- **Resource management** with file URI support
- **Legacy system migration** support
- **Multi-location storage** (local vs global agents)
- **Agent precedence** and fallback hierarchy

**Discovery Requirements:**
- "What role should this agent play?"
- "What MCP servers do you need access to?"
- "What tools should be available?"
- "What resources should be included?"
- "What hooks are needed for lifecycle management?"
- "Should this be a local or global agent?"

**Configuration Fields (12 distinct areas):**

1. **Basic Identity:**
   - `name`: Agent identifier
   - `description`: Human-readable description

2. **Core Behavior:**
   - `prompt`: System prompt (inline or file:// URI)
   - `model`: Specific model ID to use

3. **MCP Server Integration:**
   ```json
   "mcpServers": {
     "server_name": {
       "command": "executable_name",
       "args": ["arg1", "arg2"],
       "env": {"VAR": "value"},
       "timeout": 120000
     }
   }
   ```

4. **Tool Management:**
   - `tools`: Available tools (`["*"]`, `["@builtin"]`, `["@server_name"]`, `["tool_name"]`)
   - `allowedTools`: Pre-approved tools (security)
   - `toolAliases`: Name remapping for collision resolution
   - `toolsSettings`: Tool-specific configuration

5. **Resource Management:**
   - `resources`: File/URI resources with file:// support
   - Path resolution (relative/absolute)
   - Automatic context inclusion

6. **Hook System (5 lifecycle events):**
   - `agentSpawn`: Agent activation
   - `userPromptSubmit`: User input processing
   - `preToolUse`: Tool execution validation
   - `postToolUse`: Tool result processing
   - `stop`: Turn completion cleanup

7. **Security & Permissions:**
   - Tool permission validation
   - Wildcard pattern matching
   - MCP server access controls
   - Hook-based security validation

8. **Legacy Support:**
   - `useLegacyMcpJson`: Legacy MCP configuration
   - Migration from old agent format
   - Backward compatibility

**File System Requirements:**
- **Local agents**: `.amazonq/cli-agents/` (workspace-specific)
- **Global agents**: `~/.aws/amazonq/cli-agents/` (user-wide)
- **Precedence rules**: Local overrides global
- **Conflict resolution**: Warning messages for naming conflicts

**Agent Selection Hierarchy:**
1. Command-line specified (`--agent name`)
2. User-defined default (`q settings chat.defaultAgent`)
3. Built-in default agent with fallback configuration

**Hook System Complexity:**
- **Event-driven architecture** with JSON event payloads
- **Exit code semantics**: 0=success, 2=block, other=warning
- **Tool matching patterns**: Exact, wildcard, MCP server patterns
- **Timeout management**: Default 30s, configurable
- **Caching system**: TTL-based result caching
- **Context injection**: Hook output added to conversation

**MCP Integration Requirements:**
- **Server lifecycle management**: Start/stop MCP servers
- **Tool discovery**: Dynamic tool enumeration from servers
- **Namespace management**: `@server_name/tool_name` format
- **Environment configuration**: Per-server environment variables
- **Timeout handling**: Per-request timeout configuration

**Creation Assistant Implications:**
- **Multi-step configuration**: 12+ distinct configuration areas
- **Validation complexity**: JSON schema validation required
- **File URI handling**: Relative/absolute path resolution
- **MCP server discovery**: Available server enumeration
- **Tool enumeration**: Built-in + MCP tool discovery
- **Security validation**: Permission and access control setup
- **Testing requirements**: Hook validation, MCP connectivity testing

**Agent Types (Inferred from documentation):**
- **General Purpose**: Default agent with basic tools
- **Development Agents**: Code-focused with development tools
- **Infrastructure Agents**: AWS/cloud-focused with specialized MCP servers
- **Security Agents**: Restricted tools with extensive hooks
- **Project-Specific**: Local agents with project context

## User Experience Flow

### Command Creation Example

```
🛠️ Creation Assistant - Custom Command
Creating command: /deploy

What should this command do?
Examples:
- 'run git status'
- 'deploy to staging'
- 'create alias for ls -la'
- 'save current context'

> deploy application to kubernetes environment

What shell command should this execute?
Use {{param}} for parameters.
Example: 'git checkout {{branch}}' or 'echo Hello {{name}}'

> kubectl apply -f deployment.yaml -n {{namespace}} --context {{cluster}}

I see your command uses parameters. Let's configure them.
For each {{param}} in your command, provide:
name: required/optional, description

Example:
branch: required, Git branch to checkout
force: optional, Force checkout

> namespace: required, Kubernetes namespace
> cluster: optional, Kubernetes cluster context

✅ Custom Command Ready!

Name: /deploy
Type: Script
Description: deploy application to kubernetes environment
Command: kubectl apply -f deployment.yaml -n {{namespace}} --context {{cluster}}
Parameters:
  - namespace: Kubernetes namespace (required)
  - cluster: Kubernetes cluster context (optional)

Save this command? (yes/no)

> yes
✅ Command created successfully!
You can now use it with: /deploy
```

## Implementation Benefits

### Eliminated Code Duplication
- Single workflow implementation
- Shared parameter configuration
- Common completion and save logic
- Unified error handling

### Consistent User Experience
- Same workflow phases for all types
- Consistent prompts and error messages
- Unified CLI interface
- Predictable interaction patterns

### Improved Extensibility
- Adding new creation types requires minimal code
- Shared improvements benefit all types
- Easy to add new workflow phases
- Modular type-specific handling

### Better Maintainability
- Single place to improve creation experience
- Easier testing and debugging
- Cleaner code organization
- Reduced maintenance burden

## CLI Integration

### Commands

```bash
# Unified creation commands
q create skill <name> [--type <skill-type>]
q create command <name>
q create agent <name>

# Legacy support (redirects to unified system)
q skills create <name>
q commands create <name>
```

### Interactive Flow

All creation types follow the same interactive pattern:
1. Start with creation type and name
2. Discovery phase with type-specific prompts
3. Configuration phase with intelligent defaults
4. Parameter configuration if needed
5. Final confirmation and save

## Testing Strategy

### Unit Tests
- Creation workflow for each type
- Parameter parsing and validation
- State transitions
- Error handling
- Type detection logic

### Integration Tests
- End-to-end creation workflows
- File system persistence
- CLI integration
- Cross-type compatibility

### User Acceptance Tests
- Real-world creation scenarios
- User workflow validation
- Error recovery testing
- Performance validation

## Security Considerations

### Input Validation
- Parameter sanitization
- Command safety validation
- Path traversal prevention
- Injection attack prevention

### File System Safety
- Proper error handling for file operations
- User isolation for command storage
- Backup and recovery mechanisms
- Permission validation

## Future Enhancements

### Advanced Features
- Template system for common patterns
- Import/export of creation configurations
- Batch creation from configuration files
- Advanced parameter validation

### AI Integration
- Intelligent type detection
- Automatic parameter inference
- Suggestion system for improvements
- Natural language to configuration

### Collaboration Features
- Shared creation templates
- Team-wide command libraries
- Version control integration
- Approval workflows

## Migration Strategy

### Phase 1: Core Implementation ✅
- Unified creation assistant structure
- Custom command creation working
- Basic testing in place

### Phase 2: Skills Migration
- Migrate existing skills creation to unified system
- Maintain backward compatibility
- Update documentation

### Phase 3: Agent Implementation
- Add agent creation support
- Implement agent-specific workflows
- Integration with agent system

### Phase 4: Advanced Features
- Template system
- AI-powered suggestions
- Collaboration features
- Performance optimizations

## Success Metrics

### Code Quality
- Reduced code duplication (target: 80% reduction)
- Improved test coverage (target: 95%)
- Faster development of new creation types

### User Experience
- Consistent workflow across all types
- Reduced learning curve for new features
- Higher user satisfaction scores

### Maintainability
- Faster bug fixes and feature additions
- Easier onboarding for new developers
- Reduced maintenance overhead

## Conclusion

The Unified Creation Assistant represents a significant architectural improvement that eliminates code duplication, provides consistent user experience, and makes the system highly extensible. By consolidating creation workflows into a single, well-designed system, we achieve better maintainability, improved user experience, and easier feature development.

The design is production-ready for custom commands, with clear paths for skills migration and agent implementation. The modular architecture ensures that future enhancements can be added without disrupting existing functionality.
