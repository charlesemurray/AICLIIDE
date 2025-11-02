# Skills and Workflows Integration

## Overview

Skills and workflows are now fully integrated into the Amazon Q CLI tool system, enabling natural language invocation through the agent interface. This integration allows users to create custom skills and workflows that can be discovered and executed by the AI agent.

## Architecture

### ToToolSpec Trait

The `ToToolSpec` trait provides a standard interface for converting skills and workflows into ToolSpec format:

```rust
pub trait ToToolSpec {
    fn to_toolspec(&self) -> Result<ToolSpec, ConversionError>;
}
```

**Implementations:**
- `JsonSkill` - Converts skill definitions to ToolSpecs
- `Workflow` - Converts workflow definitions to ToolSpecs

### Tool Integration

Skills and workflows are integrated as first-class tools:

```rust
pub enum Tool {
    // ... existing variants
    Skill(SkillTool),
    Workflow(WorkflowTool),
}
```

## Usage

### Natural Language Invocation

Users can invoke skills through natural language:

```bash
q chat "Calculate 5 + 3 using the calculator skill"
```

The agent will:
1. Discover available skills through ToolManager
2. Match the request to the calculator skill
3. Extract parameters from natural language
4. Invoke the skill with proper inputs
5. Return the result to the user

### Programmatic Usage

#### Using Skills

```rust
use chat_cli::cli::skills::SkillRegistry;
use chat_cli::cli::chat::tool_manager::ToolManager;

// Initialize with skills
let os = Os::new().await?;
let tool_manager = ToolManager::new_with_skills(&os).await?;

// Skills are now available as tools
```

#### Using Workflows

```rust
use chat_cli::cli::workflow::types::{Workflow, WorkflowStep, StepType};
use chat_cli::cli::workflow::executor::WorkflowExecutor;

// Create a workflow
let workflow = Workflow {
    name: "data_processing".to_string(),
    description: "Process data through multiple steps".to_string(),
    version: "1.0.0".to_string(),
    steps: vec![
        WorkflowStep {
            id: "step1".to_string(),
            step_type: StepType::Skill {
                name: "fetch_data".to_string(),
                inputs: json!({"source": "api"}),
            },
        }
    ],
    inputs: vec![],
};

// Execute workflow
let executor = WorkflowExecutor::new(skill_registry);
let result = executor.execute(&workflow, json!({})).await?;
```

## Creating Custom Skills

### Skill Definition Format

Skills are defined in JSON format:

```json
{
  "name": "my_skill",
  "description": "Description of what the skill does",
  "parameters": [
    {
      "name": "input",
      "type": "string",
      "description": "Input parameter",
      "required": true
    }
  ],
  "implementation": {
    "type": "command",
    "command": "echo {{input}}"
  }
}
```

### Parameter Types

Supported parameter types:
- `string` - Text input
- `number` - Numeric input
- `boolean` - True/false
- `array` - List of values
- `object` - Structured data

### Parameter Validation

Parameters support validation rules:
- `required` - Parameter must be provided
- `enum` - Value must be from a list
- `pattern` - Value must match regex
- `minimum`/`maximum` - Numeric bounds

## Creating Workflows

### Workflow Definition Format

```json
{
  "name": "my_workflow",
  "description": "Multi-step workflow",
  "version": "1.0.0",
  "inputs": [
    {
      "name": "source",
      "type": "string",
      "required": true
    }
  ],
  "steps": [
    {
      "id": "fetch",
      "type": "skill",
      "name": "fetch_data",
      "inputs": {
        "source": "{{inputs.source}}"
      }
    },
    {
      "id": "process",
      "type": "skill",
      "name": "process_data",
      "inputs": {
        "data": "{{fetch.output}}"
      }
    }
  ]
}
```

### Variable Interpolation

Workflows support variable interpolation:
- `{{inputs.name}}` - Access workflow inputs
- `{{step_id.output}}` - Access previous step outputs

## Error Handling

### Graceful Failures

The system handles errors gracefully:

```rust
// Non-existent skill returns None
let skill = registry.get_skill("nonexistent");
assert!(skill.is_none());

// Invalid parameters caught by schema validation
// Runtime validation provides clear error messages
```

### Error Types

- `ConversionError::MissingField` - Required field missing in definition
- `ConversionError::InvalidSchema` - Schema validation failed
- Skill execution errors return descriptive messages

## Extending the System

### Implementing ToToolSpec

To add new tool types:

```rust
use chat_cli::cli::skills::toolspec_conversion::ToToolSpec;

impl ToToolSpec for MyCustomType {
    fn to_toolspec(&self) -> Result<ToolSpec, ConversionError> {
        Ok(ToolSpec {
            name: self.name.clone(),
            description: self.description.clone(),
            input_schema: self.build_schema()?,
            tool_origin: ToolOrigin::Custom,
        })
    }
}
```

### Adding to ToolManager

Register custom tools:

```rust
// Tools implementing ToToolSpec can be registered
tool_manager.register_custom_tools(my_tools);
```

## Best Practices

### Skill Design

1. **Single Responsibility** - Each skill should do one thing well
2. **Clear Descriptions** - Help the agent understand when to use the skill
3. **Validation** - Define parameter constraints upfront
4. **Error Messages** - Provide clear feedback on failures

### Workflow Design

1. **Modular Steps** - Break complex tasks into simple steps
2. **Input Validation** - Validate workflow inputs early
3. **Error Handling** - Handle step failures gracefully
4. **Documentation** - Document expected inputs and outputs

### Performance

1. **Lazy Loading** - Skills loaded on-demand
2. **Caching** - ToolSpecs cached after conversion
3. **Parallel Execution** - Independent workflow steps can run in parallel

## Testing

### Unit Tests

Test individual components:

```rust
#[tokio::test]
async fn test_skill_conversion() {
    let skill = create_test_skill();
    let toolspec = skill.to_toolspec().unwrap();
    assert_eq!(toolspec.name, "test_skill");
}
```

### Integration Tests

Test end-to-end flows:

```rust
#[tokio::test]
async fn test_skill_invocation() {
    let tool_manager = ToolManager::new_with_skills(&os).await?;
    // Verify skills are discoverable and invocable
}
```

## Troubleshooting

### Skill Not Found

**Problem**: Agent can't find a skill

**Solutions**:
- Verify skill is in the registry
- Check skill name matches exactly
- Ensure skill definition is valid JSON

### Parameter Validation Errors

**Problem**: Skill invocation fails with parameter errors

**Solutions**:
- Check parameter types match schema
- Verify required parameters are provided
- Review parameter constraints (enum, pattern, etc.)

### Workflow Execution Fails

**Problem**: Workflow stops mid-execution

**Solutions**:
- Check all referenced skills exist
- Verify variable interpolation syntax
- Review step dependencies
- Check for circular dependencies

## API Reference

### Core Types

- `ToToolSpec` - Trait for ToolSpec conversion
- `SkillTool` - Skill executor
- `WorkflowTool` - Workflow executor
- `WorkflowExecutor` - Workflow execution engine
- `SkillRegistry` - Skill management
- `ToolManager` - Tool discovery and invocation

### Key Methods

- `to_toolspec()` - Convert to ToolSpec
- `ToolManager::new_with_skills()` - Initialize with skills
- `WorkflowExecutor::execute()` - Execute workflow
- `SkillRegistry::get_skill()` - Retrieve skill by name

## Examples

See integration tests for complete examples:
- `crates/chat-cli/tests/skill_toolspec_integration.rs`
- `crates/chat-cli/tests/workflow_toolspec_integration.rs`
- `crates/chat-cli/tests/natural_language_skill_invocation.rs`
- `crates/chat-cli/tests/skill_workflow_error_handling.rs`
