# Unified Creation Assistant Design v2

## Overview

The Unified Creation Assistant integrates with the existing Q CLI system to provide consistent, terminal-native creation experiences for Skills, Custom Commands, and Agents. This design respects existing CLI patterns, follows UX principles, and handles the full complexity of each creation type.

## Existing Q CLI System Analysis

### Current Command Structure
```bash
q agent list|create|edit|validate|migrate|set-default
q skills list|run|info|install|create [--interactive|--wizard|--quick]
q chat [--agent name]
q settings
```

### Existing Creation Patterns
- **Skills**: Multiple creation modes (interactive, wizard, quick)
- **Agents**: File-based creation with complex JSON configuration
- **Commands**: No creation interface (gap we're filling)

### User Experience Principles (from ux-design.md)
- **Terminal-native**: ANSI colors, no emojis, CLI conventions
- **Cognitive load management**: Progressive disclosure, smart defaults
- **Power user efficiency**: Minimal keystrokes, predictable responses
- **Semantic color mapping**: Blue=debug, Green=success, Yellow=warnings, Red=errors

## Unified Creation Assistant Architecture

### Integration with Existing CLI
```bash
# Existing commands (preserved)
q skills create name --interactive    # Existing skills creation
q agent create --name agent-name      # Existing agent creation

# New unified commands (enhanced experience)
q create skill name [--type type]     # Enhanced skills creation
q create command name                  # New custom commands creation  
q create agent name                    # Enhanced agent creation

# Backward compatibility maintained
q skills create → redirects to unified system with skills-specific flow
q agent create → redirects to unified system with agent-specific flow
```

### Creation Type Requirements

#### 1. Custom Commands (Simple → Complex)
**Complexity Level: LOW**
- 3 command types (Script, Alias, Builtin)
- Parameter system with {{param}} substitution
- Security validation for dangerous commands
- File persistence in `.q-commands/`

**Creation Flow:**
```
Discovery → Configuration → [Parameters] → Completion
```

#### 2. Skills (Medium → High)
**Complexity Level: MEDIUM-HIGH**
- 4 skill types with distinct requirements:
  - `code_inline`: Commands/scripts with execution
  - `code_session`: REPL-style interactive sessions
  - `conversation`: Chat assistants with system prompts
  - `prompt_inline`: Template-based text generation
- Security configuration (permissions, resource limits)
- Testing phase with prompt validation
- File persistence in `.q-skills/`

**Creation Flow:**
```
Discovery → Configuration → Security → Testing → Completion
```

#### 3. Agents (High → Very High)
**Complexity Level: VERY HIGH**
- 12+ configuration fields:
  - Basic: name, description, prompt (inline or file://)
  - MCP: mcpServers with command/args/env/timeout
  - Tools: tools, allowedTools, toolAliases, toolsSettings
  - Resources: file:// URIs with path resolution
  - Hooks: 5 lifecycle events (agentSpawn, userPromptSubmit, preToolUse, postToolUse, stop)
  - Legacy: useLegacyMcpJson, migration support
  - Model: specific model ID configuration
- Multi-location storage (local vs global)
- Agent precedence and fallback hierarchy
- MCP server lifecycle management

**Creation Flow:**
```
Discovery → Basic Config → MCP Setup → Tools Config → Resources → Hooks → Testing → Completion
```

## Customer Experience Design

### Progressive Disclosure Strategy

#### Level 1: Quick Creation (80% of users)
```bash
q create command deploy
> What should this command do? deploy to staging
> What command to run? kubectl apply -f deployment.yaml
✅ Created /deploy command

q create skill reviewer  
> What should this skill do? review code for issues
> What type? code_inline
✅ Created reviewer skill

q create agent helper
> What role should this agent play? general development helper
> Use default configuration? yes
✅ Created helper agent
```

#### Level 2: Guided Creation (15% of users)
```bash
q create command deploy --guided
Discovery: What should this command do?
Configuration: Command details and parameters
Parameters: Required/optional parameter setup
Security: Validation and safety checks
Completion: Review and save

q create skill reviewer --guided
Discovery: Skill purpose and type
Configuration: Prompts and commands
Security: Permissions and resource limits
Testing: Validate with sample inputs
Completion: Review and save

q create agent helper --guided
Discovery: Agent role and purpose
Basic Config: Name, description, prompt
MCP Setup: External server connections
Tools Config: Available tools and permissions
Resources: File and context resources
Hooks: Lifecycle event handling
Testing: Validate configuration
Completion: Review and save
```

#### Level 3: Expert Mode (5% of users)
```bash
q create command deploy --expert
# Full configuration wizard with all options
# Advanced parameter validation
# Custom security rules
# Performance optimization

q create agent helper --expert  
# All 12+ configuration fields
# Custom MCP server setup
# Advanced hook configuration
# Complex tool aliasing
# Resource optimization
```

### Terminal-Native UX Design

#### Color Coding (Following existing UX guidelines)
```bash
# Discovery phase - Blue (analysis/debug)
[34mCreation Assistant[0m - Custom Command
Creating command: [1m/deploy[0m

# Configuration phase - Cyan (data/information)
[36mWhat shell command should this execute?[0m
Use [33m{{param}}[0m for parameters

# Success states - Green
[32m✅ Command created successfully![0m

# Warnings - Yellow  
[33m⚠️ This command uses sudo - are you sure?[0m

# Errors - Red
[31m❌ Command name already exists[0m
```

#### Progressive Information Display
```bash
# Simple start
Creating command: /deploy
What should this command do?

# Expand based on complexity
[Detected: Script command with parameters]
What shell command should this execute?
💡 Tip: Use {{param}} for parameters

# Show examples contextually
Examples:
  git checkout {{branch}}
  kubectl apply -f deployment.yaml -n {{namespace}}
  docker run -p {{port}}:80 {{image}}
```

### Existing System Integration

#### Respect Current Patterns
```bash
# Skills creation modes (preserve existing)
q skills create name --interactive    # Existing guided mode
q skills create name --wizard         # Existing step-by-step
q skills create name --quick          # Existing minimal mode

# Enhanced with unified system
q create skill name                   # New unified interface
q create skill name --mode quick      # Unified quick mode
q create skill name --mode guided     # Unified guided mode
q create skill name --mode expert     # New expert mode
```

#### File System Integration
```bash
# Existing locations (preserve)
Skills: .q-skills/
Agents: .amazonq/cli-agents/ (local) or ~/.aws/amazonq/cli-agents/ (global)

# New location (add)
Custom Commands: .q-commands/

# Unified management
q list                               # List all artifacts
q list skills                       # List skills only
q list commands                      # List custom commands only
q list agents                       # List agents only
```

## Implementation Requirements

### Phase 1: Foundation (Current Status)
- ✅ Custom commands basic creation
- ❌ Skills integration missing
- ❌ Agents integration missing
- ❌ UX design compliance missing
- ❌ Progressive disclosure missing

### Phase 2: Skills Integration
- Integrate 4 skill types with existing SkillRegistry
- Add security configuration wizard
- Implement testing phase workflow
- Maintain backward compatibility with existing skills CLI

### Phase 3: Agent Integration  
- Handle 12+ agent configuration fields
- MCP server discovery and configuration
- Hook system setup and validation
- Tool management with wildcards and aliases
- Resource configuration with file:// URIs
- Local vs global agent placement

### Phase 4: UX Enhancement
- Terminal-native color coding
- Progressive disclosure (quick/guided/expert modes)
- Smart defaults and suggestions
- Error recovery and validation
- Accessibility compliance

## Customer Experience Journey

### New User (First Time)
```bash
q create command hello
> What should this command do? say hello to someone
> What command to run? echo "Hello {{name}}"
> Configure parameters? name: required, Person to greet
✅ Created /hello command
💡 Try it: /hello name=Alice
```

### Intermediate User (Some Experience)
```bash
q create skill reviewer --guided
Discovery: What should this skill do?
> review code for security issues

Configuration: What type of skill?
1. Command (execute code analysis)
2. Template (generate review templates)  
3. Assistant (interactive code review)
4. REPL (step-by-step analysis)
> 1

Security: What permissions needed?
> file_read for source code access
✅ Created reviewer skill
```

### Expert User (Advanced Usage)
```bash
q create agent devops --expert
[12-step configuration wizard]
- Basic configuration
- MCP server setup (kubernetes, aws, docker)
- Tool configuration with aliases
- Resource management
- Hook system setup
- Security validation
- Testing and validation
✅ Created production-ready devops agent
```

## Success Metrics

### Customer Experience
- **Time to first success**: < 30 seconds for simple creations
- **Error recovery**: Clear guidance when things go wrong
- **Learning curve**: Progressive complexity revelation
- **Consistency**: Same patterns across all creation types

### Technical Quality
- **Backward compatibility**: 100% with existing CLI commands
- **Test coverage**: 95% for all creation workflows
- **Performance**: < 1 second response times
- **Reliability**: Graceful error handling and recovery

## Critical Design Decisions

### 1. Preserve Existing CLI Patterns
- Keep `q skills create` working exactly as before
- Keep `q agent create` working exactly as before
- Add `q create` as enhanced unified interface
- No breaking changes to existing workflows

### 2. Progressive Complexity
- Default to simplest possible creation flow
- Reveal complexity only when needed
- Provide escape hatches for power users
- Smart defaults based on user input analysis

### 3. Terminal-Native UX
- Follow existing color coding system
- Use ANSI formatting instead of emojis
- Respect CLI conventions and accessibility
- Maintain consistency with existing Q CLI UX

### 4. Extensibility Without Complexity
- Easy to add new creation types
- Shared components for common workflows
- Type-specific handling where needed
- Clean separation of concerns

## Conclusion

The unified creation assistant must handle vastly different complexity levels:
- **Custom Commands**: Simple 3-field configuration
- **Skills**: Medium complexity with security and testing
- **Agents**: Very high complexity with 12+ fields and MCP integration

The design ensures excellent customer experience through progressive disclosure, respects existing CLI patterns, and provides a foundation for future enhancements while maintaining the terminal-native UX principles that Q CLI users expect.
