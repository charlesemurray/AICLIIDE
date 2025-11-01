# Creation Builder System

A comprehensive system for creating, validating, and managing both AI assistant prompts and executable commands with specialized builders and shared foundations.

## Overview

The creation system provides specialized, type-safe builders for different creation types while sharing common functionality through traits. This ensures clarity of intent and prevents mixing incompatible concepts.

## Architecture

### Separate Specialized Builders

- **PromptBuilder**: Creates AI assistant prompts with roles, capabilities, constraints
- **CommandBuilder**: Creates executable commands with parameters, working directories, validation

### Shared Foundation

```rust
trait CreationBuilder {
    fn with_name(self, name: String) -> Self;
    fn with_description(self, description: String) -> Self;
    fn validate(&self) -> ValidationResult;
    fn build(self) -> Result<Self::Output>;
}
```

## Core Components

### PromptBuilder

Specialized for AI assistant creation:

```rust
use crate::cli::creation::prompt_system::*;

let template = PromptBuilder::new()
    .with_name("Code Reviewer".to_string())
    .with_description("Expert code reviewer".to_string())
    .with_role("You are an expert code reviewer with 10+ years of experience".to_string())
    .add_capability("security analysis".to_string())
    .add_constraint("be constructive".to_string())
    .with_example(
        "Review this code: def process(data): return eval(data)".to_string(),
        "This code has a security vulnerability...".to_string()
    )
    .build()?;
```

### CommandBuilder

Specialized for executable command creation:

```rust
use crate::cli::creation::prompt_system::*;

let command = CommandBuilder::new()
    .with_name("git-status".to_string())
    .with_description("Show git repository status".to_string())
    .with_command("git status".to_string())
    .add_parameter("--short".to_string())
    .with_working_directory("/path/to/repo".to_string())
    .with_timeout(30)
    .build()?;
```

## Design Rationale

### Why Separate Builders?

1. **Cognitive Clarity**: Clear intent from the start
   - `CommandBuilder` = "I'm making something that executes"
   - `PromptBuilder` = "I'm making an AI assistant"

2. **Type Safety**: Compile-time prevention of nonsensical combinations
   ```rust
   // This won't compile - good!
   CommandBuilder::new().with_role("You are...") // ❌
   ```

3. **Specialized Validation**: Domain-specific rules
   - Commands: executable exists, parameters valid, timeout reasonable
   - Prompts: role clarity, capability completeness, example quality

4. **Future Extensibility**: Easy to add type-specific features
   - Commands: retry logic, environment variables, output parsing
   - Prompts: model selection, temperature, token limits

### Shared Functionality

Common operations are shared through traits:
- Name and description validation
- Template management integration
- Error handling patterns
- Testing infrastructure
### Key Features Implemented

#### PromptBuilder Features
- **Fluent Builder Pattern**: Intuitive method chaining for prompt construction
- **Validation System**: Quality scoring and error detection with actionable feedback
- **Template Management**: Save, load, and organize prompt templates
- **Examples Library**: Pre-built templates for common use cases

#### CommandBuilder Features  
- **Executable Validation**: Verify commands exist and are accessible
- **Parameter Management**: Handle flags, arguments, and environment variables
- **Working Directory**: Set execution context
- **Timeout Control**: Prevent hanging commands
- **Output Handling**: Capture and process command results

#### Shared Features
- **Performance Optimized**: Fast creation, validation, and generation
- **Memory Efficient**: No memory leaks, minimal overhead
- **Error Handling**: Comprehensive error messages with suggestions
- **Testing Infrastructure**: Extensive test coverage for both builders

## Builder Methods

### PromptBuilder Methods

#### Basic Information
- `with_name(name: String)` - Set template name
- `with_description(description: String)` - Set template description
- `with_role(role: String)` - Set the assistant's role definition

#### AI-Specific Configuration
- `add_capability(capability: String)` - Add a single capability
- `with_capabilities(capabilities: Vec<String>)` - Set all capabilities
- `add_constraint(constraint: String)` - Add a single constraint
- `with_constraints(constraints: Vec<String>)` - Set all constraints
- `with_example(input: String, output: String)` - Add example conversation
- `with_category(category: TemplateCategory)` - Set template category
- `with_difficulty(difficulty: DifficultyLevel)` - Set difficulty level

### CommandBuilder Methods

#### Basic Information
- `with_name(name: String)` - Set command name
- `with_description(description: String)` - Set command description

#### Execution Configuration
- `with_command(command: String)` - Set the executable command
- `add_parameter(param: String)` - Add command parameter/flag
- `with_parameters(params: Vec<String>)` - Set all parameters
- `with_working_directory(dir: String)` - Set execution directory
- `with_timeout(seconds: u64)` - Set execution timeout
- `with_environment(key: String, value: String)` - Add environment variable

#### Validation and Building
- `validate()` - Validate current configuration without building
- `preview()` - Show what the command would execute
- `build()` - Build and validate the final configuration

## Validation Systems

### PromptBuilder Validation
- Template name cannot be empty
- Role definition should be descriptive (>20 characters recommended)
- Capabilities and constraints improve quality score
- Examples enhance template effectiveness

### CommandBuilder Validation
- Command name must be valid identifier
- Executable must exist on PATH or be absolute path
- Parameters must be properly formatted
- Working directory must exist and be accessible
- Timeout must be reasonable (1-3600 seconds)

## Usage Examples

### Creating an AI Assistant
```rust
let assistant = PromptBuilder::new()
    .with_name("Security Reviewer".to_string())
    .with_role("You are a cybersecurity expert with 15+ years of experience".to_string())
    .add_capability("vulnerability assessment".to_string())
    .add_capability("secure coding practices".to_string())
    .add_constraint("always explain security implications".to_string())
    .with_example(
        "Review: password = '123456'".to_string(),
        "Critical: Hardcoded weak password. Use environment variables and strong passwords.".to_string()
    )
    .build()?;
```

### Creating an Executable Command
```rust
let command = CommandBuilder::new()
    .with_name("docker-logs".to_string())
    .with_description("Show Docker container logs with follow".to_string())
    .with_command("docker logs".to_string())
    .add_parameter("--follow".to_string())
    .add_parameter("--tail=100".to_string())
    .with_timeout(300)
    .build()?;
```

## Integration with Q CLI

Both builders integrate seamlessly with the Q CLI creation system:

```bash
# Creates AI assistant
q create assistant --name "Code Helper" --guided

# Creates executable command  
q create command --name "git-status" --quick
```

The system automatically detects the creation type and uses the appropriate builder with validation and quality feedback.
