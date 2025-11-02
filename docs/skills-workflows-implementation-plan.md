# Skills and Workflows Implementation Plan (REVISED)

## Overview

Detailed implementation plan for adding skills and workflows natural language interaction to Q CLI.

**Related Document**: [Design Document](./skills-workflows-design.md)

## Development Workflow Rules

### Iteration Size
- **Maximum 2 hours** per iteration
- **Prefer 30-45 minutes** for most iterations
- Each iteration = 1 focused change

### Compilation Rule (STRICT)
- Code **MUST compile** after every iteration
- **NO `todo!()`, `unimplemented!()`, or placeholders** in committed code
- Use minimal working implementations instead
- If a feature isn't ready, don't add the API yet

### Testing Rule (STRICT)
- Every iteration includes **at least 1 test**
- All tests must **pass** before commit
- Run full test suite: `cargo test`

### Git Commit Rule (STRICT)
- **Commit after every iteration**
- Commit message format: `<action> <what>`
- Push to remote after every 3-4 commits

### Quality Checks (Before Every Commit)
```bash
cargo fmt
cargo clippy --all-targets
cargo test
git add -A
git commit -m "Clear, specific message"
```

### Analysis Checkpoints
- **After every 4 iterations**: Quick code review (10 min)
- **After every phase**: Full analysis (1 hour)
  - Run benchmarks
  - Check test coverage
  - Review technical debt
  - Update documentation

---

## Phase 1: Core Infrastructure (Days 1-3)

### 1.1 Extend ToolOrigin Enum

#### Iteration 1.1.1: Add Skill variant (30 min)
**Files**: `crates/chat-cli/src/cli/chat/tools/mod.rs`

- [ ] Add `Skill(String)` variant to `ToolOrigin` enum
- [ ] Update `Display` implementation for `ToolOrigin`
- [ ] Add test: `test_tool_origin_skill_display()`
- [ ] Add test: `test_tool_origin_skill_serialization()`
- [ ] ✅ **Compile + Test + Commit**: "Add Skill variant to ToolOrigin"

#### Iteration 1.1.2: Add Workflow variant (30 min)
**Files**: `crates/chat-cli/src/cli/chat/tools/mod.rs`

- [ ] Add `Workflow(String)` variant to `ToolOrigin` enum
- [ ] Update `Display` implementation for `ToolOrigin`
- [ ] Add test: `test_tool_origin_workflow_display()`
- [ ] Add test: `test_tool_origin_workflow_serialization()`
- [ ] ✅ **Compile + Test + Commit**: "Add Workflow variant to ToolOrigin"

**Checkpoint**: Review ToolOrigin changes (5 min)

---

### 1.2 Create Skill Module

#### Iteration 1.2.1: Create skill.rs with basic struct (45 min)
**Files**: `crates/chat-cli/src/cli/chat/tools/skill.rs` (new)

- [ ] Create file with module header
- [ ] Define `SkillTool` struct:
  ```rust
  #[derive(Debug, Clone)]
  pub struct SkillTool {
      pub name: String,
      pub description: String,
  }
  ```
- [ ] Implement `SkillTool::new()`
- [ ] Add test: `test_skill_tool_creation()`
- [ ] Add to `mod.rs`: `pub mod skill;`
- [ ] ✅ **Compile + Test + Commit**: "Create SkillTool struct"

#### Iteration 1.2.2: Add Skill to Tool enum (30 min)
**Files**: `crates/chat-cli/src/cli/chat/tools/mod.rs`

- [ ] Add `use skill::SkillTool;`
- [ ] Add `Skill(SkillTool)` variant to `Tool` enum
- [ ] Implement `Tool::display_name()` for Skill case
- [ ] Add test: `test_tool_skill_display_name()`
- [ ] ✅ **Compile + Test + Commit**: "Add Skill variant to Tool enum"

#### Iteration 1.2.3: Implement validate for Skill (30 min)
**Files**: `crates/chat-cli/src/cli/chat/tools/mod.rs`

- [ ] Add `Skill(_)` case to `Tool::validate()`
- [ ] Return `Ok(())` for now (real validation later)
- [ ] Add test: `test_skill_validate_success()`
- [ ] ✅ **Compile + Test + Commit**: "Add Skill validation stub"

#### Iteration 1.2.4: Implement eval_perm for Skill (30 min)
**Files**: `crates/chat-cli/src/cli/chat/tools/mod.rs`

- [ ] Add `Skill(skill)` case to `Tool::eval_perm()`
- [ ] Return `PermissionEvalResult::Approved` for now
- [ ] Add test: `test_skill_eval_perm()`
- [ ] ✅ **Compile + Test + Commit**: "Add Skill permission evaluation"

**Checkpoint**: Review Skill integration (10 min)

---

### 1.3 Create Workflow Module

#### Iteration 1.3.1: Create workflow.rs with basic struct (45 min)
**Files**: `crates/chat-cli/src/cli/chat/tools/workflow.rs` (new)

- [ ] Create file with module header
- [ ] Define `WorkflowTool` struct:
  ```rust
  #[derive(Debug, Clone)]
  pub struct WorkflowTool {
      pub name: String,
      pub description: String,
  }
  ```
- [ ] Implement `WorkflowTool::new()`
- [ ] Add test: `test_workflow_tool_creation()`
- [ ] Add to `mod.rs`: `pub mod workflow;`
- [ ] ✅ **Compile + Test + Commit**: "Create WorkflowTool struct"

#### Iteration 1.3.2: Add Workflow to Tool enum (30 min)
**Files**: `crates/chat-cli/src/cli/chat/tools/mod.rs`

- [ ] Add `use workflow::WorkflowTool;`
- [ ] Add `Workflow(WorkflowTool)` variant to `Tool` enum
- [ ] Implement `Tool::display_name()` for Workflow case
- [ ] Add test: `test_tool_workflow_display_name()`
- [ ] ✅ **Compile + Test + Commit**: "Add Workflow variant to Tool enum"

#### Iteration 1.3.3: Implement validate for Workflow (30 min)
**Files**: `crates/chat-cli/src/cli/chat/tools/mod.rs`

- [ ] Add `Workflow(_)` case to `Tool::validate()`
- [ ] Return `Ok(())` for now
- [ ] Add test: `test_workflow_validate_success()`
- [ ] ✅ **Compile + Test + Commit**: "Add Workflow validation stub"

#### Iteration 1.3.4: Implement eval_perm for Workflow (30 min)
**Files**: `crates/chat-cli/src/cli/chat/tools/mod.rs`

- [ ] Add `Workflow(workflow)` case to `Tool::eval_perm()`
- [ ] Return `PermissionEvalResult::Approved` for now
- [ ] Add test: `test_workflow_eval_perm()`
- [ ] ✅ **Compile + Test + Commit**: "Add Workflow permission evaluation"

**Checkpoint**: Review Workflow integration (10 min)

---

### 1.4 Skill Definition Types

#### Iteration 1.4.1: Create SkillDefinition struct (45 min)
**Files**: `crates/chat-cli/src/cli/chat/tools/skill.rs`

- [ ] Define `SkillDefinition` struct with serde:
  ```rust
  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct SkillDefinition {
      pub name: String,
      pub description: String,
      pub skill_type: String,
  }
  ```
- [ ] Add test: `test_skill_definition_deserialize()`
- [ ] ✅ **Compile + Test + Commit**: "Add SkillDefinition struct"

#### Iteration 1.4.2: Add parameters to SkillDefinition (45 min)
**Files**: `crates/chat-cli/src/cli/chat/tools/skill.rs`

- [ ] Add `parameters` field to `SkillDefinition`
- [ ] Define `SkillParameter` struct
- [ ] Add test: `test_skill_definition_with_parameters()`
- [ ] ✅ **Compile + Test + Commit**: "Add parameters to SkillDefinition"

#### Iteration 1.4.3: Add implementation to SkillDefinition (45 min)
**Files**: `crates/chat-cli/src/cli/chat/tools/skill.rs`

- [ ] Add `implementation` field to `SkillDefinition`
- [ ] Define `SkillImplementation` enum (Script, Command)
- [ ] Add test: `test_skill_definition_script_implementation()`
- [ ] Add test: `test_skill_definition_command_implementation()`
- [ ] ✅ **Compile + Test + Commit**: "Add implementation to SkillDefinition"

**Checkpoint**: Review SkillDefinition (10 min)

---

### 1.5 Workflow Definition Types

#### Iteration 1.5.1: Create WorkflowDefinition struct (45 min)
**Files**: `crates/chat-cli/src/cli/chat/tools/workflow.rs`

- [ ] Define `WorkflowDefinition` struct with serde:
  ```rust
  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct WorkflowDefinition {
      pub name: String,
      pub version: String,
      pub description: String,
  }
  ```
- [ ] Add test: `test_workflow_definition_deserialize()`
- [ ] ✅ **Compile + Test + Commit**: "Add WorkflowDefinition struct"

#### Iteration 1.5.2: Add steps to WorkflowDefinition (45 min)
**Files**: `crates/chat-cli/src/cli/chat/tools/workflow.rs`

- [ ] Add `steps` field to `WorkflowDefinition`
- [ ] Define `WorkflowStep` struct
- [ ] Add test: `test_workflow_definition_with_steps()`
- [ ] ✅ **Compile + Test + Commit**: "Add steps to WorkflowDefinition"

#### Iteration 1.5.3: Add context to WorkflowDefinition (30 min)
**Files**: `crates/chat-cli/src/cli/chat/tools/workflow.rs`

- [ ] Add `context` field to `WorkflowDefinition`
- [ ] Define `WorkflowContext` struct
- [ ] Add test: `test_workflow_definition_with_context()`
- [ ] ✅ **Compile + Test + Commit**: "Add context to WorkflowDefinition"

**Checkpoint**: Review WorkflowDefinition (10 min)

---

### 1.6 Skill Registry - Part 1 (Loading)

#### Iteration 1.6.1: Create skill_registry.rs module (45 min)
**Files**: `crates/chat-cli/src/cli/chat/skill_registry.rs` (new)

- [ ] Create file with module header
- [ ] Define `SkillRegistry` struct:
  ```rust
  pub struct SkillRegistry {
      skills: HashMap<String, SkillDefinition>,
  }
  ```
- [ ] Implement `SkillRegistry::new()`
- [ ] Add test: `test_skill_registry_creation()`
- [ ] Add to `chat/mod.rs`: `pub mod skill_registry;`
- [ ] ✅ **Compile + Test + Commit**: "Create SkillRegistry struct"

#### Iteration 1.6.2: Implement load_from_directory (1 hour)
**Files**: `crates/chat-cli/src/cli/chat/skill_registry.rs`

- [ ] Implement `load_from_directory(&mut self, path: &Path)`
- [ ] Scan for `.json` files
- [ ] Parse each file as `SkillDefinition`
- [ ] Store in `skills` HashMap
- [ ] Add test: `test_load_skills_from_directory()` (with temp dir)
- [ ] ✅ **Compile + Test + Commit**: "Implement skill loading from directory"

#### Iteration 1.6.3: Add get_skill method (30 min)
**Files**: `crates/chat-cli/src/cli/chat/skill_registry.rs`

- [ ] Implement `get_skill(&self, name: &str) -> Option<&SkillDefinition>`
- [ ] Add test: `test_get_skill_exists()`
- [ ] Add test: `test_get_skill_not_found()`
- [ ] ✅ **Compile + Test + Commit**: "Add get_skill method"

#### Iteration 1.6.4: Add list_skills method (30 min)
**Files**: `crates/chat-cli/src/cli/chat/skill_registry.rs`

- [ ] Implement `list_skills(&self) -> Vec<&SkillDefinition>`
- [ ] Add test: `test_list_skills()`
- [ ] ✅ **Compile + Test + Commit**: "Add list_skills method"

**Checkpoint**: Review SkillRegistry (10 min)

---

### 1.7 Workflow Registry - Part 1 (Loading)

#### Iteration 1.7.1: Create workflow_registry.rs module (45 min)
**Files**: `crates/chat-cli/src/cli/chat/workflow_registry.rs` (new)

- [ ] Create file with module header
- [ ] Define `WorkflowRegistry` struct:
  ```rust
  pub struct WorkflowRegistry {
      workflows: HashMap<String, WorkflowDefinition>,
  }
  ```
- [ ] Implement `WorkflowRegistry::new()`
- [ ] Add test: `test_workflow_registry_creation()`
- [ ] Add to `chat/mod.rs`: `pub mod workflow_registry;`
- [ ] ✅ **Compile + Test + Commit**: "Create WorkflowRegistry struct"

#### Iteration 1.7.2: Implement load_from_directory (1 hour)
**Files**: `crates/chat-cli/src/cli/chat/workflow_registry.rs`

- [ ] Implement `load_from_directory(&mut self, path: &Path)`
- [ ] Scan for `.json` files
- [ ] Parse each file as `WorkflowDefinition`
- [ ] Store in `workflows` HashMap
- [ ] Add test: `test_load_workflows_from_directory()` (with temp dir)
- [ ] ✅ **Compile + Test + Commit**: "Implement workflow loading from directory"

#### Iteration 1.7.3: Add get_workflow method (30 min)
**Files**: `crates/chat-cli/src/cli/chat/workflow_registry.rs`

- [ ] Implement `get_workflow(&self, name: &str) -> Option<&WorkflowDefinition>`
- [ ] Add test: `test_get_workflow_exists()`
- [ ] Add test: `test_get_workflow_not_found()`
- [ ] ✅ **Compile + Test + Commit**: "Add get_workflow method"

#### Iteration 1.7.4: Add list_workflows method (30 min)
**Files**: `crates/chat-cli/src/cli/chat/workflow_registry.rs`

- [ ] Implement `list_workflows(&self) -> Vec<&WorkflowDefinition>`
- [ ] Add test: `test_list_workflows()`
- [ ] ✅ **Compile + Test + Commit**: "Add list_workflows method"

**Checkpoint**: Review WorkflowRegistry (10 min)

---

### 1.8 ToolManager Integration - Part 1

#### Iteration 1.8.1: Add skill_registry field (30 min)
**Files**: `crates/chat-cli/src/cli/chat/tool_manager.rs`

- [ ] Add `skill_registry: SkillRegistry` field to `ToolManager`
- [ ] Update `ToolManager::new()` to initialize registry
- [ ] Add test: `test_tool_manager_has_skill_registry()`
- [ ] ✅ **Compile + Test + Commit**: "Add skill_registry to ToolManager"

#### Iteration 1.8.2: Add workflow_registry field (30 min)
**Files**: `crates/chat-cli/src/cli/chat/tool_manager.rs`

- [ ] Add `workflow_registry: WorkflowRegistry` field to `ToolManager`
- [ ] Update `ToolManager::new()` to initialize registry
- [ ] Add test: `test_tool_manager_has_workflow_registry()`
- [ ] ✅ **Compile + Test + Commit**: "Add workflow_registry to ToolManager"

#### Iteration 1.8.3: Load skills on initialization (45 min)
**Files**: `crates/chat-cli/src/cli/chat/tool_manager.rs`

- [ ] In `ToolManager::new()`, call `skill_registry.load_from_directory()`
- [ ] Use skills directory path from config
- [ ] Add test: `test_tool_manager_loads_skills()`
- [ ] ✅ **Compile + Test + Commit**: "Load skills on ToolManager init"

#### Iteration 1.8.4: Load workflows on initialization (45 min)
**Files**: `crates/chat-cli/src/cli/chat/tool_manager.rs`

- [ ] In `ToolManager::new()`, call `workflow_registry.load_from_directory()`
- [ ] Use workflows directory path from config
- [ ] Add test: `test_tool_manager_loads_workflows()`
- [ ] ✅ **Compile + Test + Commit**: "Load workflows on ToolManager init"

**Phase 1 Checkpoint**: Full analysis (1 hour)
- [ ] Run full test suite
- [ ] Check test coverage (should be >80%)
- [ ] Run benchmarks for loading performance
- [ ] Review all code for quality
- [ ] Document any technical debt
- [ ] ✅ **Commit**: "Phase 1 complete - Core infrastructure"

**Phase 1 Total**: ~16 hours (2 days) across 24 iterations

---

## Phase 2: Skill Execution (Days 4-5)

### 2.1 Script Execution Foundation

#### Iteration 2.1.1: Add invoke method stub (30 min)
**Files**: `crates/chat-cli/src/cli/chat/tools/skill.rs`

- [ ] Add `invoke(&self, params: HashMap<String, Value>) -> Result<String>` to `SkillTool`
- [ ] Return `Ok("not implemented".to_string())` for now
- [ ] Add test: `test_skill_invoke_stub()`
- [ ] ✅ **Compile + Test + Commit**: "Add SkillTool invoke stub"

#### Iteration 2.1.2: Parse script path (45 min)
**Files**: `crates/chat-cli/src/cli/chat/tools/skill.rs`

- [ ] Add `get_script_path(&self) -> Result<PathBuf>` method
- [ ] Parse from `SkillImplementation::Script`
- [ ] Validate path exists
- [ ] Add test: `test_get_script_path_valid()`
- [ ] Add test: `test_get_script_path_not_found()`
- [ ] ✅ **Compile + Test + Commit**: "Add script path parsing"

#### Iteration 2.1.3: Build environment variables (45 min)
**Files**: `crates/chat-cli/src/cli/chat/tools/skill.rs`

- [ ] Add `build_env_vars(&self, params: &HashMap<String, Value>) -> HashMap<String, String>`
- [ ] Convert JSON values to strings
- [ ] Prefix with `SKILL_PARAM_`
- [ ] Add test: `test_build_env_vars()`
- [ ] ✅ **Compile + Test + Commit**: "Add environment variable builder"

#### Iteration 2.1.4: Execute script (1 hour)
**Files**: `crates/chat-cli/src/cli/chat/tools/skill.rs`

- [ ] Implement script execution in `invoke()`
- [ ] Use `std::process::Command`
- [ ] Set environment variables
- [ ] Capture stdout
- [ ] Add test: `test_execute_simple_script()` (with temp script)
- [ ] ✅ **Compile + Test + Commit**: "Implement basic script execution"

**Checkpoint**: Review script execution (10 min)

---

### 2.2 Script Execution - Error Handling

#### Iteration 2.2.1: Add timeout support (1 hour)
**Files**: `crates/chat-cli/src/cli/chat/tools/skill.rs`

- [ ] Add `timeout` field to `SkillDefinition`
- [ ] Use `tokio::time::timeout` for execution
- [ ] Kill process on timeout
- [ ] Add test: `test_script_timeout()`
- [ ] ✅ **Compile + Test + Commit**: "Add script execution timeout"

#### Iteration 2.2.2: Capture stderr (30 min)
**Files**: `crates/chat-cli/src/cli/chat/tools/skill.rs`

- [ ] Capture stderr in addition to stdout
- [ ] Include in error messages
- [ ] Add test: `test_capture_stderr()`
- [ ] ✅ **Compile + Test + Commit**: "Capture stderr from scripts"

#### Iteration 2.2.3: Handle exit codes (30 min)
**Files**: `crates/chat-cli/src/cli/chat/tools/skill.rs`

- [ ] Check exit code after execution
- [ ] Return error if non-zero
- [ ] Include exit code in error message
- [ ] Add test: `test_nonzero_exit_code()`
- [ ] ✅ **Compile + Test + Commit**: "Handle script exit codes"

**Checkpoint**: Review error handling (10 min)

---

### 2.3 Command Execution

#### Iteration 2.3.1: Parse command template (45 min)
**Files**: `crates/chat-cli/src/cli/chat/tools/skill.rs`

- [ ] Add `parse_command_template(&self, params: &HashMap<String, Value>) -> Result<String>`
- [ ] Replace `{{param_name}}` with values
- [ ] Add test: `test_parse_command_template()`
- [ ] ✅ **Compile + Test + Commit**: "Add command template parsing"

#### Iteration 2.3.2: Execute command (45 min)
**Files**: `crates/chat-cli/src/cli/chat/tools/skill.rs`

- [ ] Implement command execution in `invoke()`
- [ ] Handle `SkillImplementation::Command` case
- [ ] Use shell execution
- [ ] Add test: `test_execute_command()`
- [ ] ✅ **Compile + Test + Commit**: "Implement command execution"

#### Iteration 2.3.3: Add command timeout (30 min)
**Files**: `crates/chat-cli/src/cli/chat/tools/skill.rs`

- [ ] Apply timeout to command execution
- [ ] Reuse timeout logic from scripts
- [ ] Add test: `test_command_timeout()`
- [ ] ✅ **Compile + Test + Commit**: "Add command execution timeout"

**Checkpoint**: Review command execution (10 min)

---

### 2.4 Output Formatting

#### Iteration 2.4.1: Format success output (30 min)
**Files**: `crates/chat-cli/src/cli/chat/tools/skill.rs`

- [ ] Add `format_output(&self, stdout: String, stderr: String) -> String`
- [ ] Create structured output for LLM
- [ ] Add test: `test_format_output()`
- [ ] ✅ **Compile + Test + Commit**: "Add output formatting"

#### Iteration 2.4.2: Truncate large outputs (45 min)
**Files**: `crates/chat-cli/src/cli/chat/tools/skill.rs`

- [ ] Add max output size constant
- [ ] Truncate if exceeds limit
- [ ] Add truncation indicator
- [ ] Add test: `test_truncate_large_output()`
- [ ] ✅ **Compile + Test + Commit**: "Add output truncation"

#### Iteration 2.4.3: Format error output (30 min)
**Files**: `crates/chat-cli/src/cli/chat/tools/skill.rs`

- [ ] Add `format_error(&self, error: &Error) -> String`
- [ ] Include helpful context
- [ ] Add test: `test_format_error()`
- [ ] ✅ **Compile + Test + Commit**: "Add error formatting"

**Checkpoint**: Review output formatting (10 min)

---

### 2.5 Integration with Tool System

#### Iteration 2.5.1: Wire up SkillTool invoke (30 min)
**Files**: `crates/chat-cli/src/cli/chat/tools/mod.rs`

- [ ] Update `Tool::Skill` case in execution logic
- [ ] Call `skill.invoke(params)`
- [ ] Add test: `test_tool_skill_invoke()`
- [ ] ✅ **Compile + Test + Commit**: "Wire up SkillTool invoke"

#### Iteration 2.5.2: Add skill to tool schema (45 min)
**Files**: `crates/chat-cli/src/cli/chat/tool_manager.rs`

- [ ] Convert `SkillDefinition` to tool spec
- [ ] Add to schema in `load_tools()`
- [ ] Add test: `test_skill_in_tool_schema()`
- [ ] ✅ **Compile + Test + Commit**: "Add skills to tool schema"

#### Iteration 2.5.3: Handle skill tool use (45 min)
**Files**: `crates/chat-cli/src/cli/chat/tool_manager.rs`

- [ ] Update `get_tool_from_tool_use()` to handle skills
- [ ] Look up skill by name in registry
- [ ] Create `SkillTool` instance
- [ ] Add test: `test_get_skill_from_tool_use()`
- [ ] ✅ **Compile + Test + Commit**: "Handle skill tool use"

**Phase 2 Checkpoint**: Full analysis (1 hour)
- [ ] Run full test suite
- [ ] Test with real skill definitions
- [ ] Check execution performance
- [ ] Review error handling
- [ ] ✅ **Commit**: "Phase 2 complete - Skill execution"

**Phase 2 Total**: ~12 hours (1.5 days) across 18 iterations

---

## Phase 3: Workflow Execution (Days 6-7)

### 3.1 Step Execution Foundation

#### Iteration 3.1.1: Add invoke method stub (30 min)
**Files**: `crates/chat-cli/src/cli/chat/tools/workflow.rs`

- [ ] Add `invoke(&self, params: HashMap<String, Value>) -> Result<String>` to `WorkflowTool`
- [ ] Return `Ok("not implemented".to_string())` for now
- [ ] Add test: `test_workflow_invoke_stub()`
- [ ] ✅ **Compile + Test + Commit**: "Add WorkflowTool invoke stub"

#### Iteration 3.1.2: Create step executor struct (45 min)
**Files**: `crates/chat-cli/src/cli/chat/tools/workflow.rs`

- [ ] Define `StepExecutor` struct
- [ ] Add `execute_step(&self, step: &WorkflowStep) -> Result<StepResult>`
- [ ] Return stub result
- [ ] Add test: `test_step_executor_creation()`
- [ ] ✅ **Compile + Test + Commit**: "Create StepExecutor"

#### Iteration 3.1.3: Resolve tool references (1 hour)
**Files**: `crates/chat-cli/src/cli/chat/tools/workflow.rs`

- [ ] Add `resolve_tool(&self, step: &WorkflowStep) -> Result<Tool>`
- [ ] Look up skill/native tool by name
- [ ] Add test: `test_resolve_skill_tool()`
- [ ] Add test: `test_resolve_native_tool()`
- [ ] ✅ **Compile + Test + Commit**: "Add tool resolution for workflow steps"

#### Iteration 3.1.4: Pass parameters to steps (45 min)
**Files**: `crates/chat-cli/src/cli/chat/tools/workflow.rs`

- [ ] Add `build_step_params(&self, step: &WorkflowStep, context: &WorkflowContext) -> HashMap<String, Value>`
- [ ] Resolve parameter references
- [ ] Add test: `test_build_step_params()`
- [ ] ✅ **Compile + Test + Commit**: "Add parameter passing to steps"

**Checkpoint**: Review step execution foundation (10 min)

---

### 3.2 Sequential Execution

#### Iteration 3.2.1: Execute single step (1 hour)
**Files**: `crates/chat-cli/src/cli/chat/tools/workflow.rs`

- [ ] Implement `execute_step()` fully
- [ ] Resolve tool
- [ ] Build params
- [ ] Invoke tool
- [ ] Add test: `test_execute_single_step()`
- [ ] ✅ **Compile + Test + Commit**: "Implement single step execution"

#### Iteration 3.2.2: Execute steps sequentially (1 hour)
**Files**: `crates/chat-cli/src/cli/chat/tools/workflow.rs`

- [ ] Implement `invoke()` to loop through steps
- [ ] Execute each step in order
- [ ] Collect results
- [ ] Add test: `test_execute_sequential_steps()`
- [ ] ✅ **Compile + Test + Commit**: "Implement sequential step execution"

#### Iteration 3.2.3: Pass outputs between steps (1 hour)
**Files**: `crates/chat-cli/src/cli/chat/tools/workflow.rs`

- [ ] Store step outputs in context
- [ ] Reference previous outputs in params
- [ ] Add test: `test_step_output_passing()`
- [ ] ✅ **Compile + Test + Commit**: "Add output passing between steps"

**Checkpoint**: Review sequential execution (10 min)

---

### 3.3 Error Handling

#### Iteration 3.3.1: Handle step failures (45 min)
**Files**: `crates/chat-cli/src/cli/chat/tools/workflow.rs`

- [ ] Catch errors from step execution
- [ ] Stop workflow on error
- [ ] Include step context in error
- [ ] Add test: `test_workflow_stops_on_error()`
- [ ] ✅ **Compile + Test + Commit**: "Add step failure handling"

#### Iteration 3.3.2: Add workflow state tracking (45 min)
**Files**: `crates/chat-cli/src/cli/chat/tools/workflow.rs`

- [ ] Define `WorkflowState` enum (Running, Completed, Failed)
- [ ] Track current step
- [ ] Add test: `test_workflow_state_tracking()`
- [ ] ✅ **Compile + Test + Commit**: "Add workflow state tracking"

#### Iteration 3.3.3: Format workflow errors (30 min)
**Files**: `crates/chat-cli/src/cli/chat/tools/workflow.rs`

- [ ] Add `format_workflow_error(&self, error: &Error, step_id: &str) -> String`
- [ ] Include step context and partial results
- [ ] Add test: `test_format_workflow_error()`
- [ ] ✅ **Compile + Test + Commit**: "Add workflow error formatting"

**Checkpoint**: Review error handling (10 min)

---

### 3.4 Output Formatting

#### Iteration 3.4.1: Format workflow results (45 min)
**Files**: `crates/chat-cli/src/cli/chat/tools/workflow.rs`

- [ ] Add `format_results(&self, results: Vec<StepResult>) -> String`
- [ ] Include step summaries
- [ ] Format for LLM consumption
- [ ] Add test: `test_format_workflow_results()`
- [ ] ✅ **Compile + Test + Commit**: "Add workflow result formatting"

#### Iteration 3.4.2: Add step timing (30 min)
**Files**: `crates/chat-cli/src/cli/chat/tools/workflow.rs`

- [ ] Track execution time per step
- [ ] Include in output
- [ ] Add test: `test_step_timing()`
- [ ] ✅ **Compile + Test + Commit**: "Add step timing"

**Checkpoint**: Review output formatting (10 min)

---

### 3.5 Integration with Tool System

#### Iteration 3.5.1: Wire up WorkflowTool invoke (30 min)
**Files**: `crates/chat-cli/src/cli/chat/tools/mod.rs`

- [ ] Update `Tool::Workflow` case in execution logic
- [ ] Call `workflow.invoke(params)`
- [ ] Add test: `test_tool_workflow_invoke()`
- [ ] ✅ **Compile + Test + Commit**: "Wire up WorkflowTool invoke"

#### Iteration 3.5.2: Add workflow to tool schema (45 min)
**Files**: `crates/chat-cli/src/cli/chat/tool_manager.rs`

- [ ] Convert `WorkflowDefinition` to tool spec
- [ ] Add to schema in `load_tools()`
- [ ] Add test: `test_workflow_in_tool_schema()`
- [ ] ✅ **Compile + Test + Commit**: "Add workflows to tool schema"

#### Iteration 3.5.3: Handle workflow tool use (45 min)
**Files**: `crates/chat-cli/src/cli/chat/tool_manager.rs`

- [ ] Update `get_tool_from_tool_use()` to handle workflows
- [ ] Look up workflow by name in registry
- [ ] Create `WorkflowTool` instance
- [ ] Add test: `test_get_workflow_from_tool_use()`
- [ ] ✅ **Compile + Test + Commit**: "Handle workflow tool use"

**Phase 3 Checkpoint**: Full analysis (1 hour)
- [ ] Run full test suite
- [ ] Test with real workflow definitions
- [ ] Check execution performance
- [ ] Review error handling
- [ ] ✅ **Commit**: "Phase 3 complete - Workflow execution"

**Phase 3 Total**: ~12 hours (1.5 days) across 16 iterations

---

## Phase 4: CLI Management (Days 8-10)

### 4.1 Skills CLI - List Command

#### Iteration 4.1.1: Create skills subcommand module (30 min)
**Files**: `crates/chat-cli/src/cli/skills.rs` (new)

- [ ] Create file with module header
- [ ] Define `SkillsCommand` enum
- [ ] Add to `cli/mod.rs`
- [ ] ✅ **Compile + Test + Commit**: "Create skills CLI module"

#### Iteration 4.1.2: Add list subcommand (45 min)
**Files**: `crates/chat-cli/src/cli/skills.rs`

- [ ] Add `List` variant to `SkillsCommand`
- [ ] Parse CLI args
- [ ] Add test: `test_parse_skills_list()`
- [ ] ✅ **Compile + Test + Commit**: "Add skills list subcommand"

#### Iteration 4.1.3: Implement list logic (1 hour)
**Files**: `crates/chat-cli/src/cli/skills.rs`

- [ ] Load skills from registry
- [ ] Format as table
- [ ] Display name, type, description
- [ ] Add test: `test_skills_list_output()`
- [ ] ✅ **Compile + Test + Commit**: "Implement skills list"

#### Iteration 4.1.4: Add filtering options (45 min)
**Files**: `crates/chat-cli/src/cli/skills.rs`

- [ ] Add `--type` filter flag
- [ ] Filter skills by type
- [ ] Add test: `test_skills_list_filter()`
- [ ] ✅ **Compile + Test + Commit**: "Add skills list filtering"

**Checkpoint**: Review list command (10 min)

---

### 4.2 Skills CLI - Show Command

#### Iteration 4.2.1: Add show subcommand (30 min)
**Files**: `crates/chat-cli/src/cli/skills.rs`

- [ ] Add `Show { name: String }` variant
- [ ] Parse CLI args
- [ ] Add test: `test_parse_skills_show()`
- [ ] ✅ **Compile + Test + Commit**: "Add skills show subcommand"

#### Iteration 4.2.2: Implement show logic (45 min)
**Files**: `crates/chat-cli/src/cli/skills.rs`

- [ ] Load skill by name
- [ ] Display full definition
- [ ] Format nicely
- [ ] Add test: `test_skills_show_output()`
- [ ] ✅ **Compile + Test + Commit**: "Implement skills show"

#### Iteration 4.2.3: Add example usage (30 min)
**Files**: `crates/chat-cli/src/cli/skills.rs`

- [ ] Generate example usage from parameters
- [ ] Display in show output
- [ ] Add test: `test_skills_show_example()`
- [ ] ✅ **Compile + Test + Commit**: "Add example usage to skills show"

**Checkpoint**: Review show command (10 min)

---

### 4.3 Skills CLI - Add Command

#### Iteration 4.3.1: Add add subcommand (30 min)
**Files**: `crates/chat-cli/src/cli/skills.rs`

- [ ] Add `Add { path: PathBuf }` variant
- [ ] Parse CLI args
- [ ] Add test: `test_parse_skills_add()`
- [ ] ✅ **Compile + Test + Commit**: "Add skills add subcommand"

#### Iteration 4.3.2: Validate skill JSON (1 hour)
**Files**: `crates/chat-cli/src/cli/skills.rs`

- [ ] Load JSON from path
- [ ] Parse as `SkillDefinition`
- [ ] Validate required fields
- [ ] Add test: `test_validate_skill_json()`
- [ ] ✅ **Compile + Test + Commit**: "Add skill JSON validation"

#### Iteration 4.3.3: Copy to skills directory (45 min)
**Files**: `crates/chat-cli/src/cli/skills.rs`

- [ ] Get skills directory path
- [ ] Copy file to directory
- [ ] Handle duplicates (prompt user)
- [ ] Add test: `test_skills_add_copy()`
- [ ] ✅ **Compile + Test + Commit**: "Implement skills add"

**Checkpoint**: Review add command (10 min)

---

### 4.4 Skills CLI - Remove Command

#### Iteration 4.4.1: Add remove subcommand (30 min)
**Files**: `crates/chat-cli/src/cli/skills.rs`

- [ ] Add `Remove { name: String }` variant
- [ ] Parse CLI args
- [ ] Add test: `test_parse_skills_remove()`
- [ ] ✅ **Compile + Test + Commit**: "Add skills remove subcommand"

#### Iteration 4.4.2: Implement remove logic (45 min)
**Files**: `crates/chat-cli/src/cli/skills.rs`

- [ ] Find skill file by name
- [ ] Prompt for confirmation
- [ ] Delete file
- [ ] Add test: `test_skills_remove()`
- [ ] ✅ **Compile + Test + Commit**: "Implement skills remove"

**Checkpoint**: Review remove command (10 min)

---

### 4.5 Workflows CLI (Mirror Skills CLI)

#### Iteration 4.5.1: Create workflows CLI module (30 min)
**Files**: `crates/chat-cli/src/cli/workflows.rs` (new)

- [ ] Create file with module header
- [ ] Define `WorkflowsCommand` enum
- [ ] Add to `cli/mod.rs`
- [ ] ✅ **Compile + Test + Commit**: "Create workflows CLI module"

#### Iteration 4.5.2: Implement list command (1 hour)
**Files**: `crates/chat-cli/src/cli/workflows.rs`

- [ ] Add `List` variant
- [ ] Implement list logic (similar to skills)
- [ ] Add test: `test_workflows_list()`
- [ ] ✅ **Compile + Test + Commit**: "Implement workflows list"

#### Iteration 4.5.3: Implement show command (45 min)
**Files**: `crates/chat-cli/src/cli/workflows.rs`

- [ ] Add `Show` variant
- [ ] Implement show logic
- [ ] Add test: `test_workflows_show()`
- [ ] ✅ **Compile + Test + Commit**: "Implement workflows show"

#### Iteration 4.5.4: Implement add command (1 hour)
**Files**: `crates/chat-cli/src/cli/workflows.rs`

- [ ] Add `Add` variant
- [ ] Implement validation and copy
- [ ] Add test: `test_workflows_add()`
- [ ] ✅ **Compile + Test + Commit**: "Implement workflows add"

#### Iteration 4.5.5: Implement remove command (45 min)
**Files**: `crates/chat-cli/src/cli/workflows.rs`

- [ ] Add `Remove` variant
- [ ] Implement remove logic
- [ ] Add test: `test_workflows_remove()`
- [ ] ✅ **Compile + Test + Commit**: "Implement workflows remove"

**Checkpoint**: Review workflows CLI (10 min)

---

### 4.6 Validation Enhancement

#### Iteration 4.6.1: Add JSON schema validation for skills (1 hour)
**Files**: `crates/chat-cli/src/cli/chat/skill_registry.rs`

- [ ] Define JSON schema for skills
- [ ] Validate on load
- [ ] Add test: `test_skill_schema_validation()`
- [ ] ✅ **Compile + Test + Commit**: "Add skill JSON schema validation"

#### Iteration 4.6.2: Add JSON schema validation for workflows (1 hour)
**Files**: `crates/chat-cli/src/cli/chat/workflow_registry.rs`

- [ ] Define JSON schema for workflows
- [ ] Validate on load
- [ ] Add test: `test_workflow_schema_validation()`
- [ ] ✅ **Compile + Test + Commit**: "Add workflow JSON schema validation"

#### Iteration 4.6.3: Validate script paths exist (30 min)
**Files**: `crates/chat-cli/src/cli/chat/skill_registry.rs`

- [ ] Check script paths on load
- [ ] Warn if missing
- [ ] Add test: `test_validate_script_paths()`
- [ ] ✅ **Compile + Test + Commit**: "Validate skill script paths"

#### Iteration 4.6.4: Validate workflow step references (45 min)
**Files**: `crates/chat-cli/src/cli/chat/workflow_registry.rs`

- [ ] Check that referenced tools exist
- [ ] Warn if missing
- [ ] Add test: `test_validate_step_references()`
- [ ] ✅ **Compile + Test + Commit**: "Validate workflow step references"

**Phase 4 Checkpoint**: Full analysis (1 hour)
- [ ] Run full test suite
- [ ] Test all CLI commands manually
- [ ] Check error messages are helpful
- [ ] Review UX
- [ ] ✅ **Commit**: "Phase 4 complete - CLI management"

**Phase 4 Total**: ~14 hours (1.75 days) across 22 iterations

---

## Phase 5: Documentation & Polish (Day 11)

### 5.1 User Documentation

#### Iteration 5.1.1: Write skills guide (1 hour)
**Files**: `docs/skills-guide.md` (new)

- [ ] Document skill creation
- [ ] Document skill definition format
- [ ] Add examples
- [ ] ✅ **Compile + Test + Commit**: "Add skills user guide"

#### Iteration 5.1.2: Write workflows guide (1 hour)
**Files**: `docs/workflows-guide.md` (new)

- [ ] Document workflow creation
- [ ] Document workflow definition format
- [ ] Add examples
- [ ] ✅ **Compile + Test + Commit**: "Add workflows user guide"

#### Iteration 5.1.3: Update main README (30 min)
**Files**: `README.md`

- [ ] Add skills/workflows section
- [ ] Link to guides
- [ ] Add quick start
- [ ] ✅ **Compile + Test + Commit**: "Update README with skills/workflows"

**Checkpoint**: Review documentation (10 min)

---

### 5.2 Code Documentation

#### Iteration 5.2.1: Document skill module (30 min)
**Files**: `crates/chat-cli/src/cli/chat/tools/skill.rs`

- [ ] Add module-level docs
- [ ] Document public APIs
- [ ] Add examples
- [ ] ✅ **Compile + Test + Commit**: "Document skill module"

#### Iteration 5.2.2: Document workflow module (30 min)
**Files**: `crates/chat-cli/src/cli/chat/tools/workflow.rs`

- [ ] Add module-level docs
- [ ] Document public APIs
- [ ] Add examples
- [ ] ✅ **Compile + Test + Commit**: "Document workflow module"

#### Iteration 5.2.3: Document registries (30 min)
**Files**: `crates/chat-cli/src/cli/chat/skill_registry.rs`, `workflow_registry.rs`

- [ ] Add module-level docs
- [ ] Document public APIs
- [ ] ✅ **Compile + Test + Commit**: "Document registry modules"

**Checkpoint**: Review code documentation (10 min)

---

### 5.3 Example Skills & Workflows

#### Iteration 5.3.1: Create example bash skill (30 min)
**Files**: `examples/skills/hello.json`, `examples/skills/hello.sh` (new)

- [ ] Create simple bash skill
- [ ] Add to examples directory
- [ ] Test it works
- [ ] ✅ **Compile + Test + Commit**: "Add example bash skill"

#### Iteration 5.3.2: Create example Python skill (30 min)
**Files**: `examples/skills/fetch-data.json`, `examples/skills/fetch-data.py` (new)

- [ ] Create Python skill with parameters
- [ ] Add to examples directory
- [ ] Test it works
- [ ] ✅ **Compile + Test + Commit**: "Add example Python skill"

#### Iteration 5.3.3: Create example workflow (45 min)
**Files**: `examples/workflows/data-pipeline.json` (new)

- [ ] Create multi-step workflow
- [ ] Use example skills
- [ ] Test it works
- [ ] ✅ **Compile + Test + Commit**: "Add example workflow"

**Checkpoint**: Review examples (10 min)

---

### 5.4 Error Messages & UX Polish

#### Iteration 5.4.1: Improve skill error messages (45 min)
**Files**: `crates/chat-cli/src/cli/chat/tools/skill.rs`

- [ ] Review all error messages
- [ ] Make them more helpful
- [ ] Add suggestions
- [ ] ✅ **Compile + Test + Commit**: "Improve skill error messages"

#### Iteration 5.4.2: Improve workflow error messages (45 min)
**Files**: `crates/chat-cli/src/cli/chat/tools/workflow.rs`

- [ ] Review all error messages
- [ ] Make them more helpful
- [ ] Add context
- [ ] ✅ **Compile + Test + Commit**: "Improve workflow error messages"

#### Iteration 5.4.3: Improve CLI help text (30 min)
**Files**: `crates/chat-cli/src/cli/skills.rs`, `workflows.rs`

- [ ] Review help text
- [ ] Add examples
- [ ] Make it clearer
- [ ] ✅ **Compile + Test + Commit**: "Improve CLI help text"

**Phase 5 Checkpoint**: Full analysis (1 hour)
- [ ] Review all documentation
- [ ] Test all examples
- [ ] Check error messages
- [ ] ✅ **Commit**: "Phase 5 complete - Documentation & polish"

**Phase 5 Total**: ~8 hours (1 day) across 12 iterations

---

## Final Integration & Testing (Day 12)

### Integration Testing

#### Iteration 6.1: End-to-end skill test (1 hour)
**Files**: `crates/chat-cli/tests/integration/skills.rs` (new)

- [ ] Create integration test
- [ ] Test full flow: add skill → invoke from chat
- [ ] ✅ **Compile + Test + Commit**: "Add end-to-end skill test"

#### Iteration 6.2: End-to-end workflow test (1 hour)
**Files**: `crates/chat-cli/tests/integration/workflows.rs` (new)

- [ ] Create integration test
- [ ] Test full flow: add workflow → invoke from chat
- [ ] ✅ **Compile + Test + Commit**: "Add end-to-end workflow test"

#### Iteration 6.3: LLM interaction test (1 hour)
**Files**: `crates/chat-cli/tests/integration/llm_skills.rs` (new)

- [ ] Test LLM can discover skills
- [ ] Test LLM can invoke skills correctly
- [ ] ✅ **Compile + Test + Commit**: "Add LLM interaction test"

**Checkpoint**: Review integration tests (10 min)

---

### Performance Testing

#### Iteration 6.4: Benchmark skill loading (45 min)
**Files**: `crates/chat-cli/benches/skill_loading.rs` (new)

- [ ] Create benchmark for loading 100 skills
- [ ] Target: <100ms
- [ ] ✅ **Compile + Test + Commit**: "Add skill loading benchmark"

#### Iteration 6.5: Benchmark workflow loading (45 min)
**Files**: `crates/chat-cli/benches/workflow_loading.rs` (new)

- [ ] Create benchmark for loading 100 workflows
- [ ] Target: <100ms
- [ ] ✅ **Compile + Test + Commit**: "Add workflow loading benchmark"

#### Iteration 6.6: Benchmark execution overhead (45 min)
**Files**: `crates/chat-cli/benches/execution.rs` (new)

- [ ] Benchmark skill execution overhead
- [ ] Target: <50ms overhead
- [ ] ✅ **Compile + Test + Commit**: "Add execution overhead benchmark"

**Checkpoint**: Review performance (10 min)

---

### Final Polish

#### Iteration 6.7: Run full test suite (30 min)
- [ ] Run `cargo test --all`
- [ ] Fix any failures
- [ ] ✅ **Compile + Test + Commit**: "Fix test failures"

#### Iteration 6.8: Run clippy (30 min)
- [ ] Run `cargo clippy --all-targets`
- [ ] Fix all warnings
- [ ] ✅ **Compile + Test + Commit**: "Fix clippy warnings"

#### Iteration 6.9: Check test coverage (30 min)
- [ ] Run coverage tool
- [ ] Ensure >85% coverage
- [ ] Add missing tests if needed
- [ ] ✅ **Compile + Test + Commit**: "Improve test coverage"

#### Iteration 6.10: Final documentation review (30 min)
- [ ] Review all docs
- [ ] Fix typos
- [ ] Ensure completeness
- [ ] ✅ **Compile + Test + Commit**: "Final documentation review"

**Final Checkpoint**: Complete analysis (2 hours)
- [ ] Run all tests
- [ ] Run all benchmarks
- [ ] Review all code
- [ ] Review all documentation
- [ ] Create final report
- [ ] ✅ **Commit**: "Skills and workflows MVP complete"

**Final Integration Total**: ~8 hours (1 day) across 10 iterations

---

## Summary

### Total Effort
- **Phase 1**: 16 hours (24 iterations)
- **Phase 2**: 12 hours (18 iterations)
- **Phase 3**: 12 hours (16 iterations)
- **Phase 4**: 14 hours (22 iterations)
- **Phase 5**: 8 hours (12 iterations)
- **Final**: 8 hours (10 iterations)

**Total**: ~70 hours (12 days) across 102 iterations

### Key Metrics
- **Average iteration**: 41 minutes
- **Commits**: 102 minimum
- **Tests**: 102+ (at least 1 per iteration)
- **Compilation checks**: 102 (every iteration)

### Success Criteria
- [ ] All 102 iterations complete
- [ ] All tests passing (>85% coverage)
- [ ] All code compiles with no warnings
- [ ] All documentation complete
- [ ] Performance targets met
- [ ] No placeholders in code
- [ ] Regular git commits (102+)

---

## Daily Breakdown

**Day 1-2**: Phase 1 (Core Infrastructure)
- 24 iterations, 16 hours
- Foundation for skills and workflows

**Day 3-4**: Phase 2 (Skill Execution)
- 18 iterations, 12 hours
- Skills can execute scripts and commands

**Day 5-6**: Phase 3 (Workflow Execution)
- 16 iterations, 12 hours
- Workflows can execute multi-step processes

**Day 7-9**: Phase 4 (CLI Management)
- 22 iterations, 14 hours
- Full CLI for managing skills and workflows

**Day 10**: Phase 5 (Documentation)
- 12 iterations, 8 hours
- Complete documentation and examples

**Day 11-12**: Final Integration
- 10 iterations, 8 hours
- Testing, benchmarking, polish

---

## Risk Mitigation

### Compilation Failures
- **Risk**: Code doesn't compile mid-phase
- **Mitigation**: Compile after every iteration (strict rule)

### Test Failures
- **Risk**: Tests break as code evolves
- **Mitigation**: Run full suite after every 4 iterations

### Scope Creep
- **Risk**: Adding features not in plan
- **Mitigation**: Stick to iteration plan, defer extras

### Performance Issues
- **Risk**: Execution too slow
- **Mitigation**: Benchmark early (Phase 2), optimize if needed

### Integration Problems
- **Risk**: Components don't work together
- **Mitigation**: Integration tests in Phase 6

---

## Next Steps

1. **Review this plan** - Ensure all stakeholders agree
2. **Set up environment** - Ensure dev environment ready
3. **Start Iteration 1.1.1** - Begin implementation
4. **Follow workflow rules** - Strict adherence to process
5. **Track progress** - Check off iterations as complete



---

## Dependencies

### External
- None (uses existing Q CLI infrastructure)

### Internal
- Existing tool system
- ToolManager
- MCP client patterns (for reference)

---

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Schema complexity | High | Start with simple schemas, iterate |
| Execution security | High | Sandbox execution, validate paths |
| LLM parameter mapping | Medium | Extensive testing, clear descriptions |
| Workflow complexity | Medium | Start with sequential, add features incrementally |
| Performance | Low | Async execution, caching |

---

## Success Criteria

- [ ] Users can add skills via CLI
- [ ] Users can add workflows via CLI
- [ ] LLM can discover and invoke skills through natural language
- [ ] LLM can execute workflows through natural language
- [ ] All tests passing
- [ ] Documentation complete
- [ ] Performance acceptable (<100ms overhead)

---

## Future Enhancements (Post-MVP)

- Skill marketplace/sharing
- Remote skill execution
- Skill versioning
- Workflow visualization
- Skill composition (skills calling skills)
- Hot reload of skills/workflows
- Skill debugging tools
