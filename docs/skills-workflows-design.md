# Skills and Workflows Natural Language Interaction

## Overview

Enable users to interact with skills and workflows through natural language by exposing them as tools to the LLM, following the same pattern used for MCP servers.

## Goals

- Allow LLM to discover and invoke skills/workflows based on user intent
- Reuse existing tool infrastructure (ToolSpec, Tool enum)
- Maintain consistency with MCP server integration pattern
- Support parameter passing through natural language

## Non-Goals

- Modifying existing MCP server architecture
- Creating a new tool specification format
- Real-time skill/workflow creation during chat

## Architecture

### 1. Tool Registration

Skills and workflows register as tools during ToolManager initialization, similar to MCP servers.

```rust
// Extend ToolOrigin enum
pub enum ToolOrigin {
    Native,
    McpServer(String),
    Skill(String),      // New
    Workflow(String),   // New
}

// Extend Tool enum
pub enum Tool {
    // ... existing variants ...
    Skill(SkillTool),
    Workflow(WorkflowTool),
}
```

### 2. Tool Structures

```rust
pub struct SkillTool {
    pub name: String,
    pub skill_id: String,
    pub description: String,
    pub params: Option<serde_json::Map<String, serde_json::Value>>,
}

pub struct WorkflowTool {
    pub name: String,
    pub workflow_id: String,
    pub description: String,
    pub params: Option<serde_json::Map<String, serde_json::Value>>,
}
```

### 3. Discovery and Loading

#### Skills Registry
- Location: `~/.config/amazon-q/skills/`
- Format: JSON files with skill definitions
- Schema:
```json
{
  "id": "skill_analyze_logs",
  "name": "analyze_logs",
  "description": "Analyze application logs for errors and patterns",
  "input_schema": {
    "type": "object",
    "properties": {
      "log_file": {
        "type": "string",
        "description": "Path to log file"
      },
      "pattern": {
        "type": "string",
        "description": "Pattern to search for"
      }
    },
    "required": ["log_file"]
  },
  "implementation": {
    "type": "script",
    "path": "./analyze_logs.sh"
  }
}
```

#### Workflows Registry
- Location: `~/.config/amazon-q/workflows/`
- Format: JSON files with workflow definitions
- Schema:
```json
{
  "id": "workflow_deploy_app",
  "name": "deploy_app",
  "description": "Deploy application to production environment",
  "input_schema": {
    "type": "object",
    "properties": {
      "environment": {
        "type": "string",
        "enum": ["staging", "production"],
        "description": "Target environment"
      },
      "version": {
        "type": "string",
        "description": "Version to deploy"
      }
    },
    "required": ["environment", "version"]
  },
  "steps": [
    {"action": "build", "tool": "execute_bash"},
    {"action": "test", "tool": "execute_bash"},
    {"action": "deploy", "tool": "use_aws"}
  ]
}
```

### 4. Integration Points

#### ToolManager Modifications

```rust
impl ToolManagerBuilder {
    pub async fn build(
        mut self,
        os: &mut Os,
        mut output: Box<dyn Write + Send + Sync + 'static>,
        interactive: bool,
    ) -> eyre::Result<ToolManager> {
        // ... existing MCP server loading ...
        
        // Load skills
        let skill_specs = load_skills(os).await?;
        
        // Load workflows
        let workflow_specs = load_workflows(os).await?;
        
        // Add to schema
        // ... rest of initialization ...
    }
}
```

#### Tool Resolution

```rust
pub async fn get_tool_from_tool_use(&mut self, value: AssistantToolUse) -> Result<Tool, ToolResult> {
    Ok(match value.name.as_str() {
        // ... existing cases ...
        
        name if self.skill_registry.contains_key(name) => {
            let skill = self.skill_registry.get(name).unwrap();
            Tool::Skill(SkillTool {
                name: skill.name.clone(),
                skill_id: skill.id.clone(),
                description: skill.description.clone(),
                params: value.args.as_object().cloned(),
            })
        },
        
        name if self.workflow_registry.contains_key(name) => {
            let workflow = self.workflow_registry.get(name).unwrap();
            Tool::Workflow(WorkflowTool {
                name: workflow.name.clone(),
                workflow_id: workflow.id.clone(),
                description: workflow.description.clone(),
                params: value.args.as_object().cloned(),
            })
        },
        
        // ... rest of cases ...
    })
}
```

#### Tool Execution

```rust
impl SkillTool {
    pub async fn invoke(&self, os: &Os, updates: &mut impl Write) -> Result<InvokeOutput> {
        // Load skill definition
        let skill = load_skill_by_id(&self.skill_id, os)?;
        
        // Execute based on implementation type
        match skill.implementation.r#type {
            ImplementationType::Script => {
                // Execute script with params
                execute_script(&skill.implementation.path, &self.params, os).await
            },
            ImplementationType::Command => {
                // Execute command with params
                execute_command(&skill.implementation.command, &self.params, os).await
            },
        }
    }
}

impl WorkflowTool {
    pub async fn invoke(&self, os: &Os, updates: &mut impl Write) -> Result<InvokeOutput> {
        // Load workflow definition
        let workflow = load_workflow_by_id(&self.workflow_id, os)?;
        
        // Execute steps sequentially
        let mut results = Vec::new();
        for step in workflow.steps {
            let result = execute_workflow_step(step, &self.params, os).await?;
            results.push(result);
        }
        
        Ok(InvokeOutput {
            output: OutputKind::Json(serde_json::json!({
                "workflow": self.name,
                "results": results
            }))
        })
    }
}
```

### 5. CLI Commands

Add subcommands for managing skills and workflows:

```bash
# Skills
q skills list                          # List all skills
q skills add <path>                    # Add skill from file
q skills remove <name>                 # Remove skill
q skills show <name>                   # Show skill details

# Workflows  
q workflows list                       # List all workflows
q workflows add <path>                 # Add workflow from file
q workflows remove <name>              # Remove workflow
q workflows show <name>                # Show workflow details
```

## Implementation Plan

### Phase 1: Core Infrastructure
1. Extend `ToolOrigin` and `Tool` enums
2. Create `SkillTool` and `WorkflowTool` structs
3. Implement skill/workflow loading from filesystem
4. Add to `ToolManager` schema

### Phase 2: Execution
1. Implement `SkillTool::invoke()`
2. Implement `WorkflowTool::invoke()`
3. Add error handling and validation
4. Add telemetry

### Phase 3: CLI Management
1. Add `q skills` subcommand
2. Add `q workflows` subcommand
3. Add validation for skill/workflow definitions
4. Add documentation

### Phase 4: Advanced Features
1. Skill/workflow templates
2. Parameter validation
3. Workflow step dependencies
4. Async workflow execution

## Example Usage

```bash
# User adds a skill
$ q skills add my-skill.json

# User starts chat
$ q chat

# Natural language interaction
User: "Analyze the logs in /var/log/app.log for errors"
Q: [Invokes analyze_logs skill with log_file="/var/log/app.log", pattern="error"]

User: "Deploy version 2.1.0 to production"
Q: [Invokes deploy_app workflow with environment="production", version="2.1.0"]
```

## Security Considerations

1. **Skill Validation**: Validate skill definitions before loading
2. **Path Restrictions**: Restrict script paths to skill directories
3. **Permission Model**: Reuse existing tool permission system
4. **Sandboxing**: Consider sandboxing skill execution
5. **Audit Logging**: Log all skill/workflow invocations

## Testing Strategy

1. **Unit Tests**: Test skill/workflow loading and parsing
2. **Integration Tests**: Test end-to-end skill/workflow execution
3. **LLM Tests**: Verify LLM can correctly select and invoke skills/workflows
4. **Error Cases**: Test malformed definitions, missing files, execution failures

## Open Questions

1. Should skills/workflows support async execution with callbacks?
2. How to handle long-running workflows (progress updates)?
3. Should workflows support conditional steps?
4. How to version skills/workflows?
5. Should there be a marketplace/registry for sharing skills?

## Alternatives Considered

### Alternative 1: Separate Tool Type
Create a completely new tool system separate from existing tools.
- **Rejected**: Duplicates infrastructure, inconsistent UX

### Alternative 2: MCP Server Wrapper
Wrap skills/workflows as MCP servers.
- **Rejected**: Unnecessary complexity, overhead of MCP protocol

### Alternative 3: Hardcode as Native Tools
Add each skill/workflow as a native tool in code.
- **Rejected**: Not extensible, requires recompilation

## References

- [MCP Client Implementation](../crates/chat-cli/src/mcp_client/client.rs)
- [Tool Manager](../crates/chat-cli/src/cli/chat/tool_manager.rs)
- [Tool Definitions](../crates/chat-cli/src/cli/chat/tools/mod.rs)
- [Custom Tool Implementation](../crates/chat-cli/src/cli/chat/tools/custom_tool.rs)
