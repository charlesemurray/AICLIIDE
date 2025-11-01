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

### Custom Commands

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

### Skills

**Discovery Prompts:**
- "What are you trying to accomplish with this skill?"
- Type-specific guidance based on SkillType

**Configuration:**
- CodeInline: Command or script
- PromptInline: Template text
- Conversation: System prompt
- CodeSession: REPL configuration

**Testing Phase:**
- Prompt testing with sample inputs
- Output validation
- Iterative refinement

### Agents

**Discovery Prompts:**
- "What role should this agent play?"
- Examples: 'code reviewer for Python', 'documentation writer'

**Configuration:**
- Instructions and behavior guidelines
- Specialized knowledge areas
- Interaction patterns

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
