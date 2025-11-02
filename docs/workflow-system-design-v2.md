# Workflow System Design v2.0

## Overview

A workflow system for Q CLI that orchestrates skills and agents to execute complex, multi-step tasks. Workflows integrate deeply with existing Q CLI infrastructure including the skills system, agent framework, MCP servers, and CLI command structure.

## Core Concepts

### Workflow
A JSON-defined sequence of steps that can include skills, agents, and control flow logic. Workflows are first-class citizens in Q CLI, stored alongside skills and agents, and managed through similar CLI commands.

### Skills
Discrete executable units from the existing Q CLI skills system. Workflows invoke skills through the `SkillRegistry` API, supporting all skill types:
- `code_inline`: Execute commands and return output
- `code_session`: Maintain persistent command sessions
- `conversation`: AI conversation prompts with context
- `prompt_inline`: Parameterized prompt templates

### Agents
Autonomous background workers defined by agent configurations. Workflows invoke agents through the existing agent framework, respecting agent configurations including:
- Tool access and permissions
- MCP server integrations
- Resource files and context
- Hooks and lifecycle events
- Model selection

### Steps
Individual units of work within a workflow. Each step type integrates with existing Q CLI systems:
- **Skill Step**: Invokes `SkillRegistry::get()` and `Skill::execute()`
- **Agent Step**: Creates agent context and spawns agent task
- **Parallel Step**: Uses tokio task spawning for concurrent execution
- **Conditional Step**: Evaluates expressions against workflow state
- **Loop Step**: Iterates with condition checking

## Architecture

### Module Structure

```
crates/
  chat-cli/
    src/
      cli/
        workflow/              # New workflow module
          mod.rs              # Public API and CLI integration
          engine.rs           # Workflow execution engine
          state.rs            # State management and persistence
          scheduler.rs        # Trigger scheduling
          executor.rs         # Step execution logic
          events.rs           # Event system
          types.rs            # Workflow data structures
          parser.rs           # JSON parsing and validation
          expression.rs       # Expression evaluator
          wizard.rs           # Interactive workflow creation
          error.rs            # Error types
        mod.rs                # Add workflow module
      cli/
        skills/               # Existing skills system
        agent/                # Existing agent system
```

### Integration Points

#### 1. CLI Command Structure

Workflows follow the same CLI pattern as skills and agents:

```rust
// In cli/mod.rs
#[derive(Debug, Subcommand, PartialEq)]
pub enum Commands {
    // ... existing commands
    Skills(SkillsArgs),
    Agent(AgentArgs),
    Workflow(WorkflowArgs),  // New workflow commands
}
```

#### 2. Skills Integration

Workflows use the existing `SkillRegistry` to discover and execute skills:

```rust
// In workflow/executor.rs
use crate::cli::skills::{SkillRegistry, Skill, SkillResult};

pub struct StepExecutor {
    skill_registry: SkillRegistry,
}

impl StepExecutor {
    async fn execute_skill_step(&self, step: &SkillStep) -> Result<StepOutput> {
        let skill = self.skill_registry.get(&step.skill)
            .ok_or(WorkflowError::SkillNotFound(step.skill.clone()))?;
        
        let params = self.resolve_inputs(&step.inputs)?;
        let result = skill.execute(params).await?;
        
        Ok(StepOutput::from_skill_result(result))
    }
}
```

**Key Integration Points:**
- Use `SkillRegistry::with_all_skills()` to load global and workspace skills
- Respect skill security configurations and resource limits
- Handle skill errors through workflow error handling strategies
- Support skill aliases through registry lookup

#### 3. Agent Integration

Workflows invoke agents through the existing agent framework:

```rust
// In workflow/executor.rs
use crate::cli::agent::{Agent, AgentConfigError};
use crate::cli::chat::ConversationState;

impl StepExecutor {
    async fn execute_agent_step(&self, step: &AgentStep) -> Result<StepOutput> {
        // Load agent configuration
        let agent = Agent::load_from_name(&step.agent).await?;
        
        // Apply workflow-specific guardrails
        let agent = self.apply_guardrails(agent, &step.guardrails)?;
        
        // Create agent context with workflow inputs
        let context = self.create_agent_context(&step.inputs)?;
        
        // Spawn agent task
        let handle = tokio::spawn(async move {
            // Execute agent with context
            // Monitor progress via events
            // Capture output
        });
        
        if step.async_execution {
            // Return immediately, track in background
            Ok(StepOutput::pending(handle))
        } else {
            // Wait for completion
            let result = handle.await?;
            Ok(StepOutput::from_agent_result(result))
        }
    }
    
    fn apply_guardrails(&self, mut agent: Agent, guardrails: &Guardrails) -> Result<Agent> {
        // Restrict allowed_tools based on guardrails
        if let Some(allowed) = &guardrails.allowed_tools {
            agent.allowed_tools = allowed.iter().cloned().collect();
        }
        
        // Apply resource limits
        if let Some(max_time) = guardrails.max_execution_time {
            // Set timeout
        }
        
        Ok(agent)
    }
}
```

**Key Integration Points:**
- Load agents from `~/.aws/amazonq/agents/` and `.amazonq/agents/`
- Respect agent tool permissions and MCP configurations
- Apply workflow-specific guardrails on top of agent config
- Use agent hooks for workflow lifecycle events
- Support agent model selection

#### 4. MCP Integration

Workflows leverage MCP servers through skills and agents:

```rust
// Skills can use MCP tools
{
  "id": "fetch-data",
  "type": "skill",
  "skill": "http-fetch",  // Skill may use MCP fetch server
  "inputs": {"url": "https://api.example.com"}
}

// Agents have MCP access configured
{
  "id": "analyze",
  "type": "agent",
  "agent": "data-analyzer",  // Agent config defines MCP servers
  "inputs": {"data": "fetch-data.output"}
}
```

**Key Integration Points:**
- No direct MCP integration in workflow engine
- MCP access controlled through skills and agents
- Workflow state can pass data between MCP-enabled steps

#### 5. State Persistence

Workflows use similar persistence patterns to skills and agents:

```rust
// In workflow/state.rs
use crate::database::Database;
use crate::util::paths;

pub struct StateManager {
    workflow_dir: PathBuf,
    db: Option<Database>,
}

impl StateManager {
    pub fn new() -> Result<Self> {
        let workflow_dir = paths::workflow_dir()?;
        std::fs::create_dir_all(&workflow_dir)?;
        
        Ok(Self {
            workflow_dir,
            db: None,
        })
    }
    
    pub async fn save_state(&self, workflow_id: &str, state: &WorkflowState) -> Result<()> {
        let state_file = self.workflow_dir.join(format!("{}.json", workflow_id));
        let json = serde_json::to_string_pretty(state)?;
        tokio::fs::write(state_file, json).await?;
        Ok(())
    }
    
    pub async fn load_state(&self, workflow_id: &str) -> Result<WorkflowState> {
        let state_file = self.workflow_dir.join(format!("{}.json", workflow_id));
        let json = tokio::fs::read_to_string(state_file).await?;
        let state = serde_json::from_str(&json)?;
        Ok(state)
    }
}
```

**Storage Locations:**
- Workflow definitions: `~/.aws/amazonq/workflows/` (global), `.amazonq/workflows/` (workspace)
- Workflow state: `~/.aws/amazonq/workflow-state/<workflow-id>.json`
- Workflow logs: `~/.aws/amazonq/logs/workflows/<workflow-id>.log`

#### 6. Resource Management

Workflows use similar resource limit patterns to skills:

```rust
// In workflow/executor.rs
use crate::cli::skills::{ResourceLimits, execute_with_timeout};

impl StepExecutor {
    async fn execute_with_limits<T>(
        &self,
        future: impl Future<Output = Result<T>>,
        limits: &ResourceLimits,
    ) -> Result<T> {
        // Use existing skills resource management
        execute_with_timeout(future, limits).await
    }
}
```

**Resource Tracking:**
- Per-workflow CPU and memory limits
- Per-step timeouts (inherited from skills)
- Concurrent workflow limits (new)
- Resource accounting via system APIs

### Execution Model

#### Event-Driven Architecture

```rust
// In workflow/events.rs
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum WorkflowEvent {
    WorkflowStarted { workflow_id: String },
    StepStarted { workflow_id: String, step_id: String },
    StepCompleted { workflow_id: String, step_id: String, output: StepOutput },
    StepFailed { workflow_id: String, step_id: String, error: String },
    WorkflowCompleted { workflow_id: String },
    WorkflowFailed { workflow_id: String, error: String },
}

pub struct EventBus {
    tx: mpsc::UnboundedSender<WorkflowEvent>,
    rx: mpsc::UnboundedReceiver<WorkflowEvent>,
}

impl EventBus {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self { tx, rx }
    }
    
    pub fn emit(&self, event: WorkflowEvent) {
        let _ = self.tx.send(event);
    }
    
    pub async fn next(&mut self) -> Option<WorkflowEvent> {
        self.rx.recv().await
    }
}
```

#### Workflow Engine

```rust
// In workflow/engine.rs
use tokio::task::JoinHandle;

pub struct WorkflowEngine {
    state_manager: StateManager,
    executor: StepExecutor,
    event_bus: EventBus,
    active_workflows: HashMap<String, JoinHandle<()>>,
}

impl WorkflowEngine {
    pub async fn execute_workflow(&mut self, workflow: Workflow) -> Result<String> {
        let workflow_id = self.generate_id();
        
        // Initialize state
        let mut state = WorkflowState::new(workflow_id.clone(), workflow.clone());
        self.state_manager.save_state(&workflow_id, &state).await?;
        
        // Emit start event
        self.event_bus.emit(WorkflowEvent::WorkflowStarted { 
            workflow_id: workflow_id.clone() 
        });
        
        // Spawn workflow task
        let handle = tokio::spawn(async move {
            self.run_workflow_loop(workflow_id, workflow, state).await
        });
        
        self.active_workflows.insert(workflow_id.clone(), handle);
        
        Ok(workflow_id)
    }
    
    async fn run_workflow_loop(
        &mut self,
        workflow_id: String,
        workflow: Workflow,
        mut state: WorkflowState,
    ) -> Result<()> {
        for step in &workflow.steps {
            // Check if workflow should continue
            if state.status == WorkflowStatus::Stopped {
                break;
            }
            
            // Execute step
            match self.executor.execute_step(step, &state).await {
                Ok(output) => {
                    state.add_output(step.id.clone(), output.clone());
                    self.event_bus.emit(WorkflowEvent::StepCompleted {
                        workflow_id: workflow_id.clone(),
                        step_id: step.id.clone(),
                        output,
                    });
                }
                Err(err) => {
                    // Handle error based on strategy
                    let strategy = workflow.error_handling
                        .per_step
                        .get(&step.id)
                        .unwrap_or(&workflow.error_handling.default);
                    
                    match self.handle_error(strategy, step, err, &mut state).await {
                        ErrorAction::Continue => continue,
                        ErrorAction::Retry => { /* retry logic */ }
                        ErrorAction::Halt => break,
                        ErrorAction::Rollback => { /* rollback logic */ break; }
                    }
                }
            }
            
            // Save state after each step
            self.state_manager.save_state(&workflow_id, &state).await?;
        }
        
        state.status = WorkflowStatus::Completed;
        self.state_manager.save_state(&workflow_id, &state).await?;
        
        self.event_bus.emit(WorkflowEvent::WorkflowCompleted { workflow_id });
        
        Ok(())
    }
}
```

### Data Flow

#### Context System

```rust
// In workflow/types.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowContext {
    pub readonly: HashMap<String, serde_json::Value>,
    pub mutable: HashMap<String, serde_json::Value>,
}

impl WorkflowContext {
    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.readonly.get(key).or_else(|| self.mutable.get(key))
    }
    
    pub fn set_mutable(&mut self, key: String, value: serde_json::Value) -> Result<()> {
        // Validate key doesn't conflict with readonly
        if self.readonly.contains_key(&key) {
            return Err(WorkflowError::ReadonlyContextViolation(key));
        }
        self.mutable.insert(key, value);
        Ok(())
    }
}
```

#### Input/Output Resolution

```rust
// In workflow/executor.rs
impl StepExecutor {
    fn resolve_inputs(
        &self,
        inputs: &HashMap<String, serde_json::Value>,
        state: &WorkflowState,
    ) -> Result<serde_json::Value> {
        let mut resolved = serde_json::Map::new();
        
        for (key, value) in inputs {
            let resolved_value = match value {
                serde_json::Value::String(s) if s.contains('.') => {
                    // Reference to previous step output: "step-id.output-name"
                    self.resolve_reference(s, state)?
                }
                _ => value.clone(),
            };
            resolved.insert(key.clone(), resolved_value);
        }
        
        Ok(serde_json::Value::Object(resolved))
    }
    
    fn resolve_reference(&self, reference: &str, state: &WorkflowState) -> Result<serde_json::Value> {
        let parts: Vec<&str> = reference.split('.').collect();
        
        match parts.as_slice() {
            ["context", key] => {
                state.context.get(key)
                    .cloned()
                    .ok_or_else(|| WorkflowError::ContextKeyNotFound(key.to_string()))
            }
            [step_id, output_name] => {
                state.get_output(step_id, output_name)
                    .ok_or_else(|| WorkflowError::OutputNotFound {
                        step_id: step_id.to_string(),
                        output_name: output_name.to_string(),
                    })
            }
            _ => Err(WorkflowError::InvalidReference(reference.to_string())),
        }
    }
}
```

### CLI Integration

#### Command Structure

```rust
// In cli/workflow/mod.rs
use clap::{Args, Subcommand};

#[derive(Debug, Args, PartialEq)]
pub struct WorkflowArgs {
    #[command(subcommand)]
    pub command: WorkflowCommand,
}

#[derive(Debug, Subcommand, PartialEq)]
pub enum WorkflowCommand {
    /// List available workflows
    List {
        /// Show only global or workspace workflows
        #[arg(long)]
        scope: Option<WorkflowScope>,
    },
    /// Run a workflow
    Run {
        /// Name of the workflow to run
        name: String,
        /// Override context values (JSON)
        #[arg(long)]
        context: Option<String>,
        /// Run in background
        #[arg(long, short)]
        background: bool,
    },
    /// Show workflow status
    Status {
        /// Workflow ID or name
        workflow: String,
    },
    /// Stop a running workflow
    Stop {
        /// Workflow ID
        workflow_id: String,
    },
    /// View workflow logs
    Logs {
        /// Workflow ID
        workflow_id: String,
        /// Follow logs in real-time
        #[arg(long, short)]
        follow: bool,
    },
    /// Create a new workflow
    Create {
        /// Workflow name
        name: String,
        /// Use interactive wizard
        #[arg(long, short)]
        wizard: bool,
    },
    /// Validate a workflow definition
    Validate {
        /// Path to workflow JSON file
        path: PathBuf,
    },
    /// Delete a workflow
    Delete {
        /// Workflow name
        name: String,
    },
}

impl WorkflowArgs {
    pub async fn execute(self, os: &mut Os) -> Result<ExitCode> {
        let mut engine = WorkflowEngine::new().await?;
        
        match self.command {
            WorkflowCommand::Run { name, context, background } => {
                let workflow = Workflow::load_from_name(&name).await?;
                let workflow_id = engine.execute_workflow(workflow).await?;
                
                if background {
                    println!("Workflow started: {}", workflow_id);
                } else {
                    // Wait for completion and show progress
                    engine.wait_for_completion(&workflow_id).await?;
                }
                
                Ok(ExitCode::SUCCESS)
            }
            // ... other commands
        }
    }
}
```

### Scheduler Integration

```rust
// In workflow/scheduler.rs
use cron::Schedule;
use std::str::FromStr;

pub struct WorkflowScheduler {
    engine: WorkflowEngine,
    schedules: HashMap<String, Schedule>,
}

impl WorkflowScheduler {
    pub async fn start(&mut self) -> Result<()> {
        // Load all workflows with schedule triggers
        let workflows = self.load_scheduled_workflows().await?;
        
        for workflow in workflows {
            for trigger in &workflow.triggers {
                if let TriggerType::Schedule { cron, .. } = trigger {
                    let schedule = Schedule::from_str(cron)?;
                    self.schedules.insert(workflow.name.clone(), schedule);
                }
            }
        }
        
        // Start polling loop
        loop {
            let now = chrono::Utc::now();
            
            for (name, schedule) in &self.schedules {
                if let Some(next) = schedule.upcoming(chrono::Utc).next() {
                    if next <= now {
                        // Trigger workflow
                        let workflow = Workflow::load_from_name(name).await?;
                        self.engine.execute_workflow(workflow).await?;
                    }
                }
            }
            
            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    }
}
```

## Security Considerations

### Workflow Validation

- JSON schema validation before execution
- Step ID uniqueness enforcement
- Reference validation (no circular dependencies)
- Resource limit validation

### Execution Security

- Workflows inherit security from skills and agents
- No privilege escalation through workflows
- Audit logging for all workflow executions
- State isolation between concurrent workflows

### File Access

- Workflow definitions follow same permissions as skills/agents
- State files protected with user-only permissions
- Logs follow existing Q CLI logging security

## Error Handling

### Error Types

```rust
// In workflow/error.rs
#[derive(Debug, thiserror::Error)]
pub enum WorkflowError {
    #[error("Workflow not found: {0}")]
    NotFound(String),
    
    #[error("Skill not found: {0}")]
    SkillNotFound(String),
    
    #[error("Agent not found: {0}")]
    AgentNotFound(String),
    
    #[error("Invalid workflow definition: {0}")]
    InvalidDefinition(String),
    
    #[error("Step execution failed: {step_id}: {error}")]
    StepFailed { step_id: String, error: String },
    
    #[error("Output not found: {step_id}.{output_name}")]
    OutputNotFound { step_id: String, output_name: String },
    
    #[error("Context key not found: {0}")]
    ContextKeyNotFound(String),
    
    #[error("Readonly context violation: {0}")]
    ReadonlyContextViolation(String),
    
    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),
    
    #[error(transparent)]
    Skill(#[from] crate::cli::skills::SkillError),
    
    #[error(transparent)]
    Agent(#[from] crate::cli::agent::AgentConfigError),
    
    #[error(transparent)]
    Io(#[from] std::io::Error),
    
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_workflow_execution() {
        let mut engine = WorkflowEngine::new().await.unwrap();
        let workflow = Workflow {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            steps: vec![/* ... */],
            // ...
        };
        
        let workflow_id = engine.execute_workflow(workflow).await.unwrap();
        assert!(!workflow_id.is_empty());
    }
    
    #[test]
    fn test_input_resolution() {
        let executor = StepExecutor::new();
        let state = WorkflowState::default();
        // Test reference resolution
    }
}
```

### Integration Tests

- Test with real skills from registry
- Test with mock agents
- Test concurrent workflow execution
- Test error handling strategies
- Test state persistence and recovery

## Future Enhancements

### Phase 1 (Post-MVP)
- Nested workflows (workflows calling workflows)
- Workflow templates and variables
- Enhanced expression language (JSONPath, custom functions)

### Phase 2
- Visual workflow editor
- Workflow marketplace/sharing
- Advanced scheduling (dependencies, priorities)
- Workflow versioning and rollback

### Phase 3
- Real-time collaboration on workflows
- Workflow analytics and optimization
- Distributed execution (multi-machine)
- Integration with AWS Step Functions

## Migration Path

### From Skills to Workflows

Users can gradually migrate from individual skills to workflows:

1. Start with single-step workflows wrapping existing skills
2. Add data flow between steps
3. Introduce error handling and retries
4. Add parallel execution for performance
5. Integrate agents for complex tasks

### Backward Compatibility

- Existing skills and agents work unchanged
- Workflows are opt-in, don't affect existing functionality
- CLI commands follow established patterns
- Configuration files use familiar JSON format
