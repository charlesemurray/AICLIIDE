# Workflow System Implementation Plan v2.0

## Overview

Detailed implementation plan for integrating the workflow system into Q CLI, building on existing infrastructure for skills, agents, and CLI commands.

## Phase 1: Foundation & CLI Integration (Week 1-2)

### Goals
- Create workflow module structure
- Integrate with CLI command system
- Basic workflow parsing and validation
- Simple linear execution with skills

### Tasks

#### 1.1 Module Setup

**Create workflow module structure**
```bash
mkdir -p crates/chat-cli/src/cli/workflow
touch crates/chat-cli/src/cli/workflow/mod.rs
touch crates/chat-cli/src/cli/workflow/types.rs
touch crates/chat-cli/src/cli/workflow/parser.rs
touch crates/chat-cli/src/cli/workflow/error.rs
```

**Files to create:**
- [ ] `workflow/mod.rs` - Module exports and public API
- [ ] `workflow/types.rs` - Core data structures
- [ ] `workflow/parser.rs` - JSON parsing and validation
- [ ] `workflow/error.rs` - Error types

**Integration point:**
```rust
// In cli/mod.rs
pub mod workflow;

#[derive(Debug, Subcommand, PartialEq)]
pub enum Commands {
    // ... existing commands
    #[command(name = "workflow")]
    Workflow(workflow::WorkflowArgs),
}
```

#### 1.2 Core Data Structures

**In `workflow/types.rs`:**
```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub steps: Vec<Step>,
    #[serde(default)]
    pub context: WorkflowContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Step {
    Skill(SkillStep),
    Agent(AgentStep),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillStep {
    pub id: String,
    pub skill: String,
    #[serde(default)]
    pub inputs: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub outputs: Vec<String>,
    #[serde(default)]
    pub timeout: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStep {
    pub id: String,
    pub agent: String,
    #[serde(default)]
    pub inputs: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub outputs: Vec<String>,
    #[serde(default)]
    pub async_execution: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkflowContext {
    #[serde(default)]
    pub readonly: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowState {
    pub workflow_id: String,
    pub status: WorkflowStatus,
    pub current_step: Option<String>,
    pub outputs: HashMap<String, HashMap<String, serde_json::Value>>,
    pub context: WorkflowContext,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WorkflowStatus {
    Running,
    Completed,
    Failed,
    Stopped,
}
```

**Dependencies to add to `Cargo.toml`:**
```toml
[dependencies]
# Already present: tokio, serde, serde_json, eyre
uuid = { version = "1.0", features = ["v4", "serde"] }
```

#### 1.3 Workflow Parser

**In `workflow/parser.rs`:**
```rust
use super::types::Workflow;
use super::error::WorkflowError;
use std::path::Path;

pub struct WorkflowParser;

impl WorkflowParser {
    pub async fn parse_file(path: &Path) -> Result<Workflow, WorkflowError> {
        let content = tokio::fs::read_to_string(path).await?;
        Self::parse_json(&content)
    }
    
    pub fn parse_json(json: &str) -> Result<Workflow, WorkflowError> {
        let workflow: Workflow = serde_json::from_str(json)?;
        Self::validate(&workflow)?;
        Ok(workflow)
    }
    
    fn validate(workflow: &Workflow) -> Result<(), WorkflowError> {
        // Check step IDs are unique
        let mut seen = std::collections::HashSet::new();
        for step in &workflow.steps {
            let id = step.id();
            if !seen.insert(id) {
                return Err(WorkflowError::DuplicateStepId(id.to_string()));
            }
        }
        
        Ok(())
    }
}
```

#### 1.4 CLI Commands

**In `workflow/mod.rs`:**
```rust
use clap::{Args, Subcommand};
use std::path::PathBuf;
use std::process::ExitCode;
use eyre::Result;

mod types;
mod parser;
mod error;

pub use types::*;
pub use error::WorkflowError;

#[derive(Debug, Args, PartialEq)]
pub struct WorkflowArgs {
    #[command(subcommand)]
    pub command: WorkflowCommand,
}

#[derive(Debug, Subcommand, PartialEq)]
pub enum WorkflowCommand {
    /// Run a workflow from a file
    Run {
        /// Path to workflow JSON file
        path: PathBuf,
    },
    /// Validate a workflow definition
    Validate {
        /// Path to workflow JSON file
        path: PathBuf,
    },
}

impl WorkflowArgs {
    pub async fn execute(self, _os: &mut crate::os::Os) -> Result<ExitCode> {
        match self.command {
            WorkflowCommand::Run { path } => {
                let workflow = parser::WorkflowParser::parse_file(&path).await?;
                println!("Loaded workflow: {}", workflow.name);
                // TODO: Execute workflow
                Ok(ExitCode::SUCCESS)
            }
            WorkflowCommand::Validate { path } => {
                let workflow = parser::WorkflowParser::parse_file(&path).await?;
                println!("✓ Workflow '{}' is valid", workflow.name);
                Ok(ExitCode::SUCCESS)
            }
        }
    }
}
```

#### 1.5 Error Types

**In `workflow/error.rs`:**
```rust
#[derive(Debug, thiserror::Error)]
pub enum WorkflowError {
    #[error("Duplicate step ID: {0}")]
    DuplicateStepId(String),
    
    #[error("Invalid workflow definition: {0}")]
    InvalidDefinition(String),
    
    #[error(transparent)]
    Io(#[from] std::io::Error),
    
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
```

#### 1.6 Basic Execution Engine

**Create `workflow/engine.rs`:**
```rust
use super::types::*;
use super::error::WorkflowError;
use crate::cli::skills::SkillRegistry;
use uuid::Uuid;

pub struct WorkflowEngine {
    skill_registry: SkillRegistry,
}

impl WorkflowEngine {
    pub async fn new() -> Result<Self, WorkflowError> {
        let current_dir = std::env::current_dir()?;
        let skill_registry = SkillRegistry::with_all_skills(&current_dir)
            .await
            .unwrap_or_else(|_| SkillRegistry::with_builtins());
        
        Ok(Self { skill_registry })
    }
    
    pub async fn execute(&mut self, workflow: Workflow) -> Result<String, WorkflowError> {
        let workflow_id = Uuid::new_v4().to_string();
        
        println!("Starting workflow: {} ({})", workflow.name, workflow_id);
        
        for step in &workflow.steps {
            match step {
                Step::Skill(skill_step) => {
                    self.execute_skill_step(skill_step).await?;
                }
                Step::Agent(_) => {
                    // TODO: Phase 4
                    println!("Agent steps not yet implemented");
                }
            }
        }
        
        println!("Workflow completed: {}", workflow_id);
        Ok(workflow_id)
    }
    
    async fn execute_skill_step(&self, step: &SkillStep) -> Result<(), WorkflowError> {
        use crate::cli::skills::Skill;
        
        println!("Executing skill step: {} ({})", step.id, step.skill);
        
        let skill = self.skill_registry.get(&step.skill)
            .ok_or_else(|| WorkflowError::InvalidDefinition(
                format!("Skill not found: {}", step.skill)
            ))?;
        
        let params = serde_json::to_value(&step.inputs)?;
        let result = skill.execute(params).await
            .map_err(|e| WorkflowError::InvalidDefinition(e.to_string()))?;
        
        println!("  Output: {}", result.output);
        
        Ok(())
    }
}
```

**Update `workflow/mod.rs` to use engine:**
```rust
mod engine;

impl WorkflowArgs {
    pub async fn execute(self, _os: &mut crate::os::Os) -> Result<ExitCode> {
        match self.command {
            WorkflowCommand::Run { path } => {
                let workflow = parser::WorkflowParser::parse_file(&path).await?;
                let mut engine = engine::WorkflowEngine::new().await?;
                let workflow_id = engine.execute(workflow).await?;
                println!("Workflow completed: {}", workflow_id);
                Ok(ExitCode::SUCCESS)
            }
            // ...
        }
    }
}
```

### Deliverables
- [ ] Workflow module integrated into CLI
- [ ] `q workflow run <file>` command works
- [ ] `q workflow validate <file>` command works
- [ ] Linear skill execution functional
- [ ] Basic error handling

### Testing

**Create `workflow/tests.rs`:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_simple_workflow() {
        let json = r#"{
            "name": "test",
            "version": "1.0.0",
            "steps": [
                {
                    "type": "skill",
                    "id": "step1",
                    "skill": "echo"
                }
            ]
        }"#;
        
        let workflow = parser::WorkflowParser::parse_json(json).unwrap();
        assert_eq!(workflow.name, "test");
        assert_eq!(workflow.steps.len(), 1);
    }
}
```

## Phase 2: State Management & Data Flow (Week 3)

### Goals
- Persistent state storage
- Output capture and storage
- Input/output wiring between steps
- Context system

### Tasks

#### 2.1 State Persistence

**Create `workflow/state.rs`:**
```rust
use super::types::*;
use super::error::WorkflowError;
use std::path::PathBuf;

pub struct StateManager {
    state_dir: PathBuf,
}

impl StateManager {
    pub fn new() -> Result<Self, WorkflowError> {
        let state_dir = crate::util::paths::workflow_state_dir()?;
        std::fs::create_dir_all(&state_dir)?;
        Ok(Self { state_dir })
    }
    
    pub async fn save(&self, state: &WorkflowState) -> Result<(), WorkflowError> {
        let path = self.state_dir.join(format!("{}.json", state.workflow_id));
        let json = serde_json::to_string_pretty(state)?;
        tokio::fs::write(path, json).await?;
        Ok(())
    }
    
    pub async fn load(&self, workflow_id: &str) -> Result<WorkflowState, WorkflowError> {
        let path = self.state_dir.join(format!("{}.json", workflow_id));
        let json = tokio::fs::read_to_string(path).await?;
        let state = serde_json::from_str(&json)?;
        Ok(state)
    }
}
```

**Add to `util/paths.rs`:**
```rust
pub fn workflow_state_dir() -> Result<PathBuf, DirectoryError> {
    let base = amazonq_dir()?;
    Ok(base.join("workflow-state"))
}
```

#### 2.2 Output Capture

**Update `engine.rs` to capture outputs:**
```rust
pub struct WorkflowEngine {
    skill_registry: SkillRegistry,
    state_manager: StateManager,
}

impl WorkflowEngine {
    pub async fn execute(&mut self, workflow: Workflow) -> Result<String, WorkflowError> {
        let workflow_id = Uuid::new_v4().to_string();
        
        let mut state = WorkflowState {
            workflow_id: workflow_id.clone(),
            status: WorkflowStatus::Running,
            current_step: None,
            outputs: HashMap::new(),
            context: workflow.context.clone(),
        };
        
        for step in &workflow.steps {
            state.current_step = Some(step.id().to_string());
            
            match step {
                Step::Skill(skill_step) => {
                    let output = self.execute_skill_step(skill_step, &state).await?;
                    state.outputs.insert(skill_step.id.clone(), output);
                }
                Step::Agent(_) => {
                    // TODO
                }
            }
            
            self.state_manager.save(&state).await?;
        }
        
        state.status = WorkflowStatus::Completed;
        state.current_step = None;
        self.state_manager.save(&state).await?;
        
        Ok(workflow_id)
    }
    
    async fn execute_skill_step(
        &self,
        step: &SkillStep,
        state: &WorkflowState,
    ) -> Result<HashMap<String, serde_json::Value>, WorkflowError> {
        let skill = self.skill_registry.get(&step.skill)
            .ok_or_else(|| WorkflowError::InvalidDefinition(
                format!("Skill not found: {}", step.skill)
            ))?;
        
        // Resolve inputs
        let params = self.resolve_inputs(&step.inputs, state)?;
        
        let result = skill.execute(params).await
            .map_err(|e| WorkflowError::InvalidDefinition(e.to_string()))?;
        
        // Capture outputs
        let mut outputs = HashMap::new();
        if step.outputs.is_empty() {
            outputs.insert("output".to_string(), serde_json::json!(result.output));
        } else {
            for output_name in &step.outputs {
                outputs.insert(output_name.clone(), serde_json::json!(result.output));
            }
        }
        
        Ok(outputs)
    }
}
```

#### 2.3 Input Resolution

**Add to `engine.rs`:**
```rust
impl WorkflowEngine {
    fn resolve_inputs(
        &self,
        inputs: &HashMap<String, serde_json::Value>,
        state: &WorkflowState,
    ) -> Result<serde_json::Value, WorkflowError> {
        let mut resolved = serde_json::Map::new();
        
        for (key, value) in inputs {
            let resolved_value = match value {
                serde_json::Value::String(s) if s.contains('.') => {
                    self.resolve_reference(s, state)?
                }
                _ => value.clone(),
            };
            resolved.insert(key.clone(), resolved_value);
        }
        
        Ok(serde_json::Value::Object(resolved))
    }
    
    fn resolve_reference(
        &self,
        reference: &str,
        state: &WorkflowState,
    ) -> Result<serde_json::Value, WorkflowError> {
        let parts: Vec<&str> = reference.split('.').collect();
        
        match parts.as_slice() {
            ["context", key] => {
                state.context.readonly.get(*key)
                    .cloned()
                    .ok_or_else(|| WorkflowError::InvalidDefinition(
                        format!("Context key not found: {}", key)
                    ))
            }
            [step_id, output_name] => {
                state.outputs.get(*step_id)
                    .and_then(|outputs| outputs.get(*output_name))
                    .cloned()
                    .ok_or_else(|| WorkflowError::InvalidDefinition(
                        format!("Output not found: {}.{}", step_id, output_name)
                    ))
            }
            _ => Err(WorkflowError::InvalidDefinition(
                format!("Invalid reference: {}", reference)
            )),
        }
    }
}
```

### Deliverables
- [ ] State persisted to disk after each step
- [ ] Step outputs captured and stored
- [ ] Input references resolved (e.g., `"step1.output"`)
- [ ] Context system working

## Phase 3: Async & Parallel Execution (Week 4)

### Goals
- Event-driven architecture
- Asynchronous step execution
- Parallel step groups
- Background task tracking

### Tasks

#### 3.1 Event System

**Create `workflow/events.rs`:**
```rust
use super::types::*;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum WorkflowEvent {
    WorkflowStarted { workflow_id: String },
    StepStarted { workflow_id: String, step_id: String },
    StepCompleted { workflow_id: String, step_id: String },
    StepFailed { workflow_id: String, step_id: String, error: String },
    WorkflowCompleted { workflow_id: String },
    WorkflowFailed { workflow_id: String, error: String },
}

pub struct EventBus {
    tx: mpsc::UnboundedSender<WorkflowEvent>,
}

impl EventBus {
    pub fn new() -> (Self, mpsc::UnboundedReceiver<WorkflowEvent>) {
        let (tx, rx) = mpsc::unbounded_channel();
        (Self { tx }, rx)
    }
    
    pub fn emit(&self, event: WorkflowEvent) {
        let _ = self.tx.send(event);
    }
}
```

#### 3.2 Parallel Steps

**Add to `types.rs`:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Step {
    Skill(SkillStep),
    Agent(AgentStep),
    Parallel(ParallelStep),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelStep {
    pub id: String,
    pub steps: Vec<Step>,
    #[serde(default = "default_wait_for")]
    pub wait_for: WaitStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WaitStrategy {
    All,
    Any,
    None,
}

fn default_wait_for() -> WaitStrategy {
    WaitStrategy::All
}
```

**Add parallel execution to `engine.rs`:**
```rust
async fn execute_parallel_step(
    &self,
    step: &ParallelStep,
    state: &WorkflowState,
) -> Result<HashMap<String, serde_json::Value>, WorkflowError> {
    let mut handles = vec![];
    
    for child_step in &step.steps {
        let child_step = child_step.clone();
        let state = state.clone();
        let engine = self.clone(); // Need to make engine cloneable
        
        let handle = tokio::spawn(async move {
            // Execute child step
            match child_step {
                Step::Skill(skill_step) => {
                    engine.execute_skill_step(&skill_step, &state).await
                }
                _ => todo!(),
            }
        });
        
        handles.push(handle);
    }
    
    match step.wait_for {
        WaitStrategy::All => {
            let results = futures::future::try_join_all(handles).await?;
            // Merge outputs
            Ok(HashMap::new())
        }
        WaitStrategy::Any => {
            // Wait for first completion
            todo!()
        }
        WaitStrategy::None => {
            // Don't wait
            Ok(HashMap::new())
        }
    }
}
```

### Deliverables
- [ ] Event system emitting workflow events
- [ ] Parallel step groups execute concurrently
- [ ] Wait strategies (all, any, none) work
- [ ] Async execution functional

## Phase 4: Agent Integration (Week 5)

### Goals
- Agent step execution
- Agent guardrails
- Background agent tasks
- Agent output capture

### Tasks

#### 4.1 Agent Execution

**Add to `types.rs`:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guardrails {
    pub max_iterations: Option<u32>,
    pub allowed_tools: Option<Vec<String>>,
    pub max_execution_time: Option<u64>,
}
```

**Add to `engine.rs`:**
```rust
use crate::cli::agent::Agent;

async fn execute_agent_step(
    &self,
    step: &AgentStep,
    state: &WorkflowState,
) -> Result<HashMap<String, serde_json::Value>, WorkflowError> {
    // Load agent configuration
    let mut agent = Agent::load_from_name(&step.agent).await
        .map_err(|e| WorkflowError::InvalidDefinition(e.to_string()))?;
    
    // Apply guardrails
    if let Some(guardrails) = &step.guardrails {
        if let Some(allowed_tools) = &guardrails.allowed_tools {
            agent.allowed_tools = allowed_tools.iter().cloned().collect();
        }
    }
    
    // Resolve inputs
    let inputs = self.resolve_inputs(&step.inputs, state)?;
    
    // TODO: Execute agent with inputs
    // This requires deeper integration with chat system
    
    Ok(HashMap::new())
}
```

### Deliverables
- [ ] Agent steps load agent configurations
- [ ] Guardrails applied to agents
- [ ] Agent execution integrated
- [ ] Agent outputs captured

## Phase 5-10: Remaining Features

Due to length constraints, phases 5-10 follow similar patterns:

**Phase 5: Error Handling** - Implement retry, rollback, skip, halt strategies
**Phase 6: Control Flow** - Add conditional and loop steps with expression evaluation
**Phase 7: Resource Management** - Enforce CPU/memory limits, concurrent workflow limits
**Phase 8: Scheduling** - Implement cron-based triggers and scheduler daemon
**Phase 9: Wizard System** - Interactive workflow creation with dialoguer
**Phase 10: CLI Enhancements** - Complete command set (list, stop, logs, delete)

## Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_skill_execution() {
        // Test with mock skill registry
    }
    
    #[test]
    fn test_input_resolution() {
        // Test reference resolution
    }
}
```

### Integration Tests
```rust
#[cfg(test)]
mod integration_tests {
    #[tokio::test]
    async fn test_end_to_end_workflow() {
        // Test complete workflow execution
    }
}
```

## Timeline Summary

| Phase | Duration | Deliverable |
|-------|----------|-------------|
| 1 | Week 1-2 | CLI integration, basic execution |
| 2 | Week 3 | State management, data flow |
| 3 | Week 4 | Async & parallel execution |
| 4 | Week 5 | Agent integration |
| 5 | Week 6 | Error handling |
| 6 | Week 7 | Control flow |
| 7 | Week 8 | Resource management |
| 8 | Week 9 | Scheduling |
| 9 | Week 10 | Wizard system |
| 10 | Week 11 | CLI enhancements |

**MVP (Phases 1-3): 4 weeks**
**Production Ready: 11 weeks**
