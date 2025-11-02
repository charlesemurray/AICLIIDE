# Skills and Workflows ToolSpec Integration - Implementation Plan

## Overview
Enable users to invoke skills and workflows through natural language by integrating them into the ToolSpec system. Each step is designed to be small, testable, and maintain compilation.

## Principles
- **Incremental**: Each step compiles and passes tests before moving forward
- **Testable**: Every feature has unit and integration tests
- **No Placeholders**: Complete implementations only, no TODOs or unimplemented!() macros
- **Git Discipline**: Commit after each completed step with descriptive messages
- **Quality Gates**: Validation at each milestone

---

## Phase 1: Foundation - Skill to ToolSpec Conversion

### Step 1.1: Create ToolSpec Conversion Trait
**Goal**: Define the interface for converting skills to ToolSpecs

**Files to Create**:
- `crates/chat-cli/src/cli/skills/toolspec_conversion.rs`

**Implementation**:
```rust
// Minimal trait definition
pub trait ToToolSpec {
    fn to_toolspec(&self) -> Result<ToolSpec, ConversionError>;
}

#[derive(Debug, thiserror::Error)]
pub enum ConversionError {
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Invalid schema: {0}")]
    InvalidSchema(String),
}
```

**Tests**:
- Test error types serialize correctly
- Test trait can be implemented

**Validation**:
- `cargo build` succeeds
- `cargo test` passes
- `cargo clippy` has no warnings

**Git Commit**: `feat(skills): add ToToolSpec trait for skill conversion`

---

### Step 1.2: Implement JsonSkill to ToolSpec Conversion
**Goal**: Convert existing JsonSkill definitions to ToolSpec format

**Files to Modify**:
- `crates/chat-cli/src/cli/skills/types.rs` (add impl)
- `crates/chat-cli/src/cli/skills/toolspec_conversion.rs` (add tests)

**Implementation**:
```rust
impl ToToolSpec for JsonSkill {
    fn to_toolspec(&self) -> Result<ToolSpec, ConversionError> {
        let input_schema = self.build_input_schema()?;
        
        Ok(ToolSpec {
            name: self.name.clone(),
            description: self.description.clone()
                .unwrap_or_else(|| format!("Execute {} skill", self.name)),
            input_schema: InputSchema(input_schema),
            tool_origin: ToolOrigin::Native,
        })
    }
}

impl JsonSkill {
    fn build_input_schema(&self) -> Result<serde_json::Value, ConversionError> {
        let mut properties = serde_json::Map::new();
        let mut required = Vec::new();
        
        if let Some(params) = &self.parameters {
            for param in params {
                properties.insert(
                    param.name.clone(),
                    self.param_to_schema(param)
                );
                if param.required.unwrap_or(false) {
                    required.push(param.name.clone());
                }
            }
        }
        
        Ok(json!({
            "type": "object",
            "properties": properties,
            "required": required
        }))
    }
    
    fn param_to_schema(&self, param: &Parameter) -> serde_json::Value {
        let mut schema = json!({
            "type": param.param_type.clone()
        });
        
        if let Some(values) = &param.values {
            schema["enum"] = json!(values);
        }
        
        if let Some(pattern) = &param.pattern {
            schema["pattern"] = json!(pattern);
        }
        
        schema
    }
}
```

**Tests**:
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_simple_skill_to_toolspec() {
        let skill = JsonSkill {
            name: "test-skill".to_string(),
            description: Some("Test description".to_string()),
            skill_type: SkillType::Command,
            command: Some("echo".to_string()),
            parameters: None,
            // ... other fields
        };
        
        let toolspec = skill.to_toolspec().unwrap();
        assert_eq!(toolspec.name, "test-skill");
        assert_eq!(toolspec.description, "Test description");
    }
    
    #[test]
    fn test_skill_with_parameters() {
        let skill = JsonSkill {
            name: "param-skill".to_string(),
            parameters: Some(vec![
                Parameter {
                    name: "input".to_string(),
                    param_type: "string".to_string(),
                    required: Some(true),
                    values: None,
                    pattern: None,
                }
            ]),
            // ... other fields
        };
        
        let toolspec = skill.to_toolspec().unwrap();
        let schema = toolspec.input_schema.0;
        assert!(schema["required"].as_array().unwrap().contains(&json!("input")));
    }
    
    #[test]
    fn test_skill_with_enum_values() {
        // Test parameter with enum values
    }
    
    #[test]
    fn test_skill_with_pattern() {
        // Test parameter with regex pattern
    }
}
```

**Validation**:
- All tests pass
- Code coverage >80% for new code
- `cargo clippy` clean

**Git Commit**: `feat(skills): implement JsonSkill to ToolSpec conversion`

---

### Step 1.3: Add SkillRegistry ToolSpec Export
**Goal**: Enable registry to export all skills as ToolSpecs

**Files to Modify**:
- `crates/chat-cli/src/cli/skills/registry.rs`

**Implementation**:
```rust
impl SkillRegistry {
    pub fn get_all_toolspecs(&self) -> Vec<ToolSpec> {
        self.skills.values()
            .filter_map(|skill| {
                skill.to_toolspec()
                    .map_err(|e| {
                        warn!("Failed to convert skill {} to toolspec: {}", 
                              skill.name(), e);
                        e
                    })
                    .ok()
            })
            .collect()
    }
    
    pub fn get_toolspec(&self, name: &str) -> Option<ToolSpec> {
        self.get(name)
            .and_then(|skill| skill.to_toolspec().ok())
    }
}
```

**Tests**:
```rust
#[tokio::test]
async fn test_registry_exports_toolspecs() {
    let registry = SkillRegistry::with_builtins();
    let toolspecs = registry.get_all_toolspecs();
    assert!(!toolspecs.is_empty());
}

#[tokio::test]
async fn test_get_specific_toolspec() {
    let mut registry = SkillRegistry::new();
    // Add test skill
    let toolspec = registry.get_toolspec("test-skill");
    assert!(toolspec.is_some());
}

#[tokio::test]
async fn test_invalid_skill_filtered_out() {
    // Test that skills that fail conversion are filtered
}
```

**Validation**:
- Tests pass
- No unwrap() or expect() in production code
- Error handling is explicit

**Git Commit**: `feat(skills): add ToolSpec export to SkillRegistry`

---

### Step 1.4: Create Skill Tool Executor
**Goal**: Execute skills when invoked as tools

**Files to Create**:
- `crates/chat-cli/src/cli/chat/tools/skill_tool.rs`

**Implementation**:
```rust
use crate::cli::skills::{SkillRegistry, SkillResult};
use super::{Tool, InvokeOutput, OutputKind};

#[derive(Debug, Clone)]
pub struct SkillTool {
    pub skill_name: String,
    pub params: serde_json::Value,
}

impl SkillTool {
    pub fn new(skill_name: String, params: serde_json::Value) -> Self {
        Self { skill_name, params }
    }
    
    pub async fn invoke(
        &self,
        registry: &SkillRegistry,
        stdout: &mut impl Write,
    ) -> Result<InvokeOutput> {
        let skill = registry.get(&self.skill_name)
            .ok_or_else(|| eyre::eyre!("Skill not found: {}", self.skill_name))?;
        
        let result = skill.execute(self.params.clone()).await
            .map_err(|e| eyre::eyre!("Skill execution failed: {}", e))?;
        
        writeln!(stdout, "{}", result.output)?;
        
        Ok(InvokeOutput {
            output: OutputKind::Text(result.output),
        })
    }
}
```

**Tests**:
```rust
#[tokio::test]
async fn test_skill_tool_execution() {
    let registry = create_test_registry();
    let tool = SkillTool::new("test-skill".to_string(), json!({}));
    let mut output = Vec::new();
    
    let result = tool.invoke(&registry, &mut output).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_skill_not_found() {
    let registry = SkillRegistry::new();
    let tool = SkillTool::new("nonexistent".to_string(), json!({}));
    let mut output = Vec::new();
    
    let result = tool.invoke(&registry, &mut output).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_skill_with_parameters() {
    // Test parameter passing
}
```

**Validation**:
- All error paths tested
- No panics possible
- Resource cleanup verified

**Git Commit**: `feat(tools): add SkillTool executor for skill invocation`

---

### Step 1.5: Integrate SkillTool into Tool Enum
**Goal**: Make skills available as native tools

**Files to Modify**:
- `crates/chat-cli/src/cli/chat/tools/mod.rs`

**Implementation**:
```rust
pub enum Tool {
    // ... existing variants
    Skill(SkillTool),
}

impl Tool {
    pub fn display_name(&self) -> String {
        match self {
            // ... existing matches
            Tool::Skill(skill_tool) => &skill_tool.skill_name,
        }
        .to_owned()
    }
    
    pub fn requires_acceptance(&self, os: &Os, agent: &Agent) -> PermissionEvalResult {
        match self {
            // ... existing matches
            Tool::Skill(_) => PermissionEvalResult::Allow, // Skills have their own security
        }
    }
    
    pub async fn invoke(
        &self,
        os: &Os,
        stdout: &mut impl Write,
        line_tracker: &mut HashMap<String, FileLineTracker>,
        agents: &crate::cli::agent::Agents,
    ) -> Result<InvokeOutput> {
        match self {
            // ... existing matches
            Tool::Skill(skill_tool) => {
                let registry = SkillRegistry::with_all_skills(&os.env.cwd()?).await?;
                skill_tool.invoke(&registry, stdout).await
            }
        }
    }
}
```

**Tests**:
```rust
#[tokio::test]
async fn test_skill_tool_in_enum() {
    let tool = Tool::Skill(SkillTool::new("test".to_string(), json!({})));
    assert_eq!(tool.display_name(), "test");
}

#[tokio::test]
async fn test_skill_tool_invocation_through_enum() {
    // Test full invocation path
}
```

**Validation**:
- Pattern matching is exhaustive
- No compilation warnings
- All existing tests still pass

**Git Commit**: `feat(tools): integrate SkillTool into Tool enum`

---

### Step 1.6: Add ToolManager Skill Registration
**Goal**: Register skills as tools in the tool manager

**Files to Modify**:
- `crates/chat-cli/src/cli/chat/tool_manager.rs`

**Implementation**:
```rust
impl ToolManager {
    pub async fn register_skills(&mut self, os: &Os) -> Result<()> {
        let registry = SkillRegistry::with_all_skills(&os.env.cwd()?).await?;
        let toolspecs = registry.get_all_toolspecs();
        
        for toolspec in toolspecs {
            self.available_tools.insert(toolspec.name.clone(), toolspec);
        }
        
        Ok(())
    }
    
    pub async fn new_with_skills(os: &Os) -> Result<Self> {
        let mut manager = Self::new();
        manager.register_skills(os).await?;
        Ok(manager)
    }
}
```

**Tests**:
```rust
#[tokio::test]
async fn test_skills_registered_in_tool_manager() {
    let os = Os::new().await.unwrap();
    let manager = ToolManager::new_with_skills(&os).await.unwrap();
    
    // Verify skills are available
    assert!(manager.available_tools.len() > 0);
}

#[tokio::test]
async fn test_skill_toolspec_retrievable() {
    // Test that registered skills can be retrieved
}
```

**Validation**:
- Integration test with real OS
- Memory usage acceptable
- Performance <100ms for registration

**Git Commit**: `feat(tool-manager): add skill registration support`

---

## Phase 2: Workflow to ToolSpec Integration

### Step 2.1: Create Workflow Type Definitions
**Goal**: Define minimal workflow structure

**Files to Create**:
- `crates/chat-cli/src/cli/workflow/types.rs`
- `crates/chat-cli/src/cli/workflow/mod.rs`

**Implementation**:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub name: String,
    pub description: String,
    pub version: String,
    pub steps: Vec<WorkflowStep>,
    pub inputs: Vec<WorkflowInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub id: String,
    pub step_type: StepType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StepType {
    Skill { name: String, inputs: serde_json::Value },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowInput {
    pub name: String,
    pub input_type: String,
    pub required: bool,
}
```

**Tests**:
```rust
#[test]
fn test_workflow_serialization() {
    let workflow = Workflow {
        name: "test".to_string(),
        description: "Test workflow".to_string(),
        version: "1.0.0".to_string(),
        steps: vec![],
        inputs: vec![],
    };
    
    let json = serde_json::to_string(&workflow).unwrap();
    let deserialized: Workflow = serde_json::from_str(&json).unwrap();
    assert_eq!(workflow.name, deserialized.name);
}

#[test]
fn test_step_type_serialization() {
    // Test StepType variants serialize correctly
}
```

**Validation**:
- Serde derives work correctly
- JSON round-trip successful
- No unsafe code

**Git Commit**: `feat(workflow): add core workflow type definitions`

---

### Step 2.2: Implement Workflow to ToolSpec Conversion
**Goal**: Convert workflows to ToolSpecs

**Files to Modify**:
- `crates/chat-cli/src/cli/workflow/types.rs`

**Implementation**:
```rust
use crate::cli::skills::toolspec_conversion::{ToToolSpec, ConversionError};
use crate::cli::chat::tools::{ToolSpec, InputSchema, ToolOrigin};

impl ToToolSpec for Workflow {
    fn to_toolspec(&self) -> Result<ToolSpec, ConversionError> {
        let input_schema = self.build_input_schema()?;
        
        Ok(ToolSpec {
            name: self.name.clone(),
            description: self.description.clone(),
            input_schema: InputSchema(input_schema),
            tool_origin: ToolOrigin::Native,
        })
    }
}

impl Workflow {
    fn build_input_schema(&self) -> Result<serde_json::Value, ConversionError> {
        let mut properties = serde_json::Map::new();
        let mut required = Vec::new();
        
        for input in &self.inputs {
            properties.insert(
                input.name.clone(),
                json!({ "type": input.input_type })
            );
            
            if input.required {
                required.push(input.name.clone());
            }
        }
        
        Ok(json!({
            "type": "object",
            "properties": properties,
            "required": required
        }))
    }
}
```

**Tests**:
```rust
#[test]
fn test_workflow_to_toolspec() {
    let workflow = Workflow {
        name: "test-workflow".to_string(),
        description: "Test".to_string(),
        version: "1.0.0".to_string(),
        steps: vec![],
        inputs: vec![
            WorkflowInput {
                name: "input1".to_string(),
                input_type: "string".to_string(),
                required: true,
            }
        ],
    };
    
    let toolspec = workflow.to_toolspec().unwrap();
    assert_eq!(toolspec.name, "test-workflow");
}

#[test]
fn test_workflow_input_schema() {
    // Test schema generation
}
```

**Validation**:
- Schema matches JSON Schema spec
- Required fields handled correctly
- Error cases covered

**Git Commit**: `feat(workflow): implement Workflow to ToolSpec conversion`

---

### Step 2.3: Create Workflow Executor
**Goal**: Execute workflows as tools

**Files to Create**:
- `crates/chat-cli/src/cli/workflow/executor.rs`

**Implementation**:
```rust
use super::types::{Workflow, StepType};
use crate::cli::skills::SkillRegistry;

pub struct WorkflowExecutor {
    skill_registry: SkillRegistry,
}

impl WorkflowExecutor {
    pub fn new(skill_registry: SkillRegistry) -> Self {
        Self { skill_registry }
    }
    
    pub async fn execute(
        &self,
        workflow: &Workflow,
        inputs: serde_json::Value,
    ) -> Result<String> {
        let mut outputs = Vec::new();
        
        for step in &workflow.steps {
            let output = self.execute_step(step, &inputs).await?;
            outputs.push(output);
        }
        
        Ok(outputs.join("\n"))
    }
    
    async fn execute_step(
        &self,
        step: &WorkflowStep,
        inputs: &serde_json::Value,
    ) -> Result<String> {
        match &step.step_type {
            StepType::Skill { name, inputs: step_inputs } => {
                let skill = self.skill_registry.get(name)
                    .ok_or_else(|| eyre::eyre!("Skill not found: {}", name))?;
                
                let merged_inputs = self.merge_inputs(inputs, step_inputs);
                let result = skill.execute(merged_inputs).await?;
                
                Ok(result.output)
            }
        }
    }
    
    fn merge_inputs(
        &self,
        workflow_inputs: &serde_json::Value,
        step_inputs: &serde_json::Value,
    ) -> serde_json::Value {
        // Simple merge: step inputs override workflow inputs
        let mut merged = workflow_inputs.clone();
        if let (Some(wf_obj), Some(step_obj)) = (merged.as_object_mut(), step_inputs.as_object()) {
            for (k, v) in step_obj {
                wf_obj.insert(k.clone(), v.clone());
            }
        }
        merged
    }
}
```

**Tests**:
```rust
#[tokio::test]
async fn test_workflow_execution() {
    let registry = create_test_registry();
    let executor = WorkflowExecutor::new(registry);
    
    let workflow = create_test_workflow();
    let result = executor.execute(&workflow, json!({})).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_workflow_with_multiple_steps() {
    // Test multi-step execution
}

#[tokio::test]
async fn test_input_merging() {
    // Test input merge logic
}
```

**Validation**:
- Sequential execution works
- Error propagation correct
- No resource leaks

**Git Commit**: `feat(workflow): add workflow executor`

---

### Step 2.4: Create Workflow Tool Wrapper
**Goal**: Wrap workflow execution as a Tool

**Files to Create**:
- `crates/chat-cli/src/cli/chat/tools/workflow_tool.rs`

**Implementation**:
```rust
use crate::cli::workflow::{Workflow, WorkflowExecutor};
use super::{InvokeOutput, OutputKind};

#[derive(Debug, Clone)]
pub struct WorkflowTool {
    pub workflow: Workflow,
    pub params: serde_json::Value,
}

impl WorkflowTool {
    pub fn new(workflow: Workflow, params: serde_json::Value) -> Self {
        Self { workflow, params }
    }
    
    pub async fn invoke(
        &self,
        executor: &WorkflowExecutor,
        stdout: &mut impl Write,
    ) -> Result<InvokeOutput> {
        let output = executor.execute(&self.workflow, self.params.clone()).await?;
        
        writeln!(stdout, "{}", output)?;
        
        Ok(InvokeOutput {
            output: OutputKind::Text(output),
        })
    }
}
```

**Tests**:
```rust
#[tokio::test]
async fn test_workflow_tool_invocation() {
    // Test workflow tool execution
}
```

**Validation**:
- Integrates with Tool enum pattern
- Error handling consistent

**Git Commit**: `feat(tools): add WorkflowTool wrapper`

---

### Step 2.5: Integrate WorkflowTool into Tool Enum
**Goal**: Make workflows available as tools

**Files to Modify**:
- `crates/chat-cli/src/cli/chat/tools/mod.rs`

**Implementation**:
```rust
pub enum Tool {
    // ... existing variants
    Workflow(WorkflowTool),
}

// Add pattern matches for Workflow variant in all impl blocks
```

**Tests**:
```rust
#[tokio::test]
async fn test_workflow_tool_in_enum() {
    // Test workflow tool integration
}
```

**Validation**:
- Exhaustive pattern matching
- No regressions in existing tests

**Git Commit**: `feat(tools): integrate WorkflowTool into Tool enum`

---

## Phase 3: End-to-End Integration

### Step 3.1: Create Integration Test Suite
**Goal**: Validate complete skill/workflow invocation path

**Files to Create**:
- `crates/chat-cli/tests/skill_toolspec_integration.rs`
- `crates/chat-cli/tests/workflow_toolspec_integration.rs`

**Tests**:
```rust
#[tokio::test]
async fn test_skill_invocation_via_natural_language() {
    // Simulate LLM requesting skill execution
    // Verify tool is invoked correctly
    // Verify output is returned
}

#[tokio::test]
async fn test_workflow_invocation_via_natural_language() {
    // Simulate LLM requesting workflow execution
}

#[tokio::test]
async fn test_skill_not_found_error_handling() {
    // Test error path
}

#[tokio::test]
async fn test_concurrent_skill_invocations() {
    // Test thread safety
}
```

**Validation**:
- All integration tests pass
- No flaky tests
- Tests run in <5s

**Git Commit**: `test: add integration tests for skill/workflow ToolSpec`

---

### Step 3.2: Add Performance Benchmarks
**Goal**: Ensure performance requirements met

**Files to Create**:
- `crates/chat-cli/benches/toolspec_conversion.rs`

**Implementation**:
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_skill_to_toolspec(c: &mut Criterion) {
    c.bench_function("skill_to_toolspec", |b| {
        let skill = create_test_skill();
        b.iter(|| {
            black_box(skill.to_toolspec())
        });
    });
}

fn bench_workflow_execution(c: &mut Criterion) {
    // Benchmark workflow execution
}

criterion_group!(benches, bench_skill_to_toolspec, bench_workflow_execution);
criterion_main!(benches);
```

**Validation**:
- Skill conversion <1ms
- Workflow execution <100ms for simple workflows
- No performance regressions

**Git Commit**: `perf: add benchmarks for ToolSpec conversion`

---

### Step 3.3: Add Documentation
**Goal**: Document the integration for users and developers

**Files to Create**:
- `docs/skills-as-tools.md`
- `docs/workflows-as-tools.md`

**Content**:
- How skills become tools
- How to create tool-compatible skills
- Workflow tool invocation examples
- Troubleshooting guide

**Validation**:
- Documentation builds without errors
- Examples are tested and work
- Links are valid

**Git Commit**: `docs: add skills and workflows ToolSpec integration guide`

---

## Quality Gates

### After Each Step
- [ ] `cargo build` succeeds
- [ ] `cargo test` passes (all tests)
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo fmt --check` passes
- [ ] New code has >80% test coverage
- [ ] Git commit with descriptive message

### After Each Phase
- [ ] Integration tests pass
- [ ] Performance benchmarks meet targets
- [ ] Documentation updated
- [ ] Code review checklist completed
- [ ] Manual testing performed

### Final Validation
- [ ] All tests pass (unit + integration)
- [ ] Benchmarks show acceptable performance
- [ ] Documentation complete and accurate
- [ ] No TODO or unimplemented!() in production code
- [ ] No clippy warnings
- [ ] Code coverage >85%
- [ ] Manual end-to-end test successful

---

## Risk Mitigation

### Risk: Skill execution security
**Mitigation**: Reuse existing skill security framework, no new permissions

### Risk: Performance degradation
**Mitigation**: Benchmark at each step, optimize before moving forward

### Risk: Breaking existing functionality
**Mitigation**: Run full test suite after each commit, maintain backward compatibility

### Risk: Incomplete implementations
**Mitigation**: Small steps, complete each fully before proceeding

---

## Rollback Strategy

Each git commit is a safe rollback point. If issues arise:
1. Identify the problematic commit
2. Run `git revert <commit-hash>`
3. Fix the issue in a new commit
4. Continue from the working state

---

## Success Metrics

- [ ] Users can invoke skills via natural language
- [ ] Users can invoke workflows via natural language
- [ ] No performance regression (within 5% of baseline)
- [ ] Zero critical bugs in production
- [ ] Test coverage >85%
- [ ] All quality gates passed
