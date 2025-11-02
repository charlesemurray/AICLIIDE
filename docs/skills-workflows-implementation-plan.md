# Skills and Workflows Implementation Plan

## Overview

Detailed implementation plan for adding skills and workflows natural language interaction to Q CLI.

**Related Document**: [Design Document](./skills-workflows-design.md)

## Phase 1: Core Infrastructure (Week 1-2)

### 1.1 Extend Core Types
**Files**: `crates/chat-cli/src/cli/chat/tools/mod.rs`

- [ ] Extend `ToolOrigin` enum
  ```rust
  pub enum ToolOrigin {
      Native,
      McpServer(String),
      Skill(String),      // Add
      Workflow(String),   // Add
  }
  ```

- [ ] Extend `Tool` enum
  ```rust
  pub enum Tool {
      // ... existing ...
      Skill(SkillTool),
      Workflow(WorkflowTool),
  }
  ```

- [ ] Update `Tool::display_name()` to handle new variants
- [ ] Update `Tool::validate()` to handle new variants
- [ ] Update `Tool::eval_perm()` to handle new variants

**Estimated Time**: 2 hours

### 1.2 Create Skill/Workflow Structures
**Files**: 
- `crates/chat-cli/src/cli/chat/tools/skill.rs` (new)
- `crates/chat-cli/src/cli/chat/tools/workflow.rs` (new)

#### skill.rs
- [ ] Define `SkillTool` struct
- [ ] Define `SkillDefinition` struct (for loading from JSON)
- [ ] Define `SkillImplementation` enum (Script, Command)
- [ ] Implement `SkillTool::invoke()`
- [ ] Implement `SkillTool::validate()`
- [ ] Implement `SkillTool::eval_perm()`
- [ ] Add unit tests

**Estimated Time**: 4 hours

#### workflow.rs
- [ ] Define `WorkflowTool` struct
- [ ] Define `WorkflowDefinition` struct (for loading from JSON)
- [ ] Define `WorkflowStep` struct
- [ ] Implement `WorkflowTool::invoke()`
- [ ] Implement `WorkflowTool::validate()`
- [ ] Implement `WorkflowTool::eval_perm()`
- [ ] Add unit tests

**Estimated Time**: 4 hours

### 1.3 Registry System
**Files**: 
- `crates/chat-cli/src/cli/chat/skill_registry.rs` (new)
- `crates/chat-cli/src/cli/chat/workflow_registry.rs` (new)

#### skill_registry.rs
- [ ] Define `SkillRegistry` struct
- [ ] Implement `load_skills()` - scan skills directory
- [ ] Implement `get_skill()` - retrieve by name/id
- [ ] Implement `validate_skill_definition()` - JSON schema validation
- [ ] Implement `skill_to_tool_spec()` - convert to ToolSpec
- [ ] Add error types
- [ ] Add unit tests with mock filesystem

**Estimated Time**: 6 hours

#### workflow_registry.rs
- [ ] Define `WorkflowRegistry` struct
- [ ] Implement `load_workflows()` - scan workflows directory
- [ ] Implement `get_workflow()` - retrieve by name/id
- [ ] Implement `validate_workflow_definition()` - JSON schema validation
- [ ] Implement `workflow_to_tool_spec()` - convert to ToolSpec
- [ ] Add error types
- [ ] Add unit tests with mock filesystem

**Estimated Time**: 6 hours

### 1.4 Integrate with ToolManager
**Files**: `crates/chat-cli/src/cli/chat/tool_manager.rs`

- [ ] Add `skill_registry: SkillRegistry` field to `ToolManager`
- [ ] Add `workflow_registry: WorkflowRegistry` field to `ToolManager`
- [ ] Update `ToolManagerBuilder::build()` to load skills/workflows
- [ ] Update `get_tool_from_tool_use()` to handle skill/workflow names
- [ ] Update `load_tools()` to include skill/workflow specs in schema
- [ ] Add integration tests

**Estimated Time**: 6 hours

**Phase 1 Total**: ~28 hours (3.5 days)

---

## Phase 2: Execution (Week 3)

### 2.1 Skill Execution
**Files**: `crates/chat-cli/src/cli/chat/tools/skill.rs`

- [ ] Implement script execution
  - [ ] Parse script path
  - [ ] Inject parameters as environment variables
  - [ ] Execute with timeout
  - [ ] Capture stdout/stderr
  - [ ] Handle exit codes

- [ ] Implement command execution
  - [ ] Template parameter substitution
  - [ ] Execute command
  - [ ] Capture output

- [ ] Add execution telemetry
- [ ] Add error handling and recovery
- [ ] Add integration tests

**Estimated Time**: 8 hours

### 2.2 Workflow Execution
**Files**: `crates/chat-cli/src/cli/chat/tools/workflow.rs`

- [ ] Implement step executor
  - [ ] Resolve tool references
  - [ ] Pass parameters to steps
  - [ ] Collect step results
  - [ ] Handle step failures

- [ ] Implement sequential execution
- [ ] Add workflow state tracking
- [ ] Add execution telemetry
- [ ] Add error handling and rollback
- [ ] Add integration tests

**Estimated Time**: 10 hours

### 2.3 Output Formatting
**Files**: `crates/chat-cli/src/cli/chat/tools/skill.rs`, `workflow.rs`

- [ ] Format skill output for LLM consumption
- [ ] Format workflow output with step details
- [ ] Handle large outputs (truncation/summarization)
- [ ] Add output tests

**Estimated Time**: 4 hours

**Phase 2 Total**: ~22 hours (2.75 days)

---

## Phase 3: CLI Management (Week 4)

### 3.1 Skills CLI
**Files**: 
- `crates/chat-cli/src/cli/skills.rs` (new)
- `crates/chat-cli/src/cli/mod.rs`

- [ ] Create `skills` subcommand module
- [ ] Implement `q skills list`
  - [ ] Display table of skills
  - [ ] Show name, description, parameters
  - [ ] Add filtering options

- [ ] Implement `q skills add <path>`
  - [ ] Validate JSON schema
  - [ ] Copy to skills directory
  - [ ] Handle duplicates

- [ ] Implement `q skills remove <name>`
  - [ ] Confirm deletion
  - [ ] Remove from filesystem

- [ ] Implement `q skills show <name>`
  - [ ] Display full skill definition
  - [ ] Show example usage

- [ ] Add CLI tests

**Estimated Time**: 8 hours

### 3.2 Workflows CLI
**Files**: 
- `crates/chat-cli/src/cli/workflows.rs` (new)
- `crates/chat-cli/src/cli/mod.rs`

- [ ] Create `workflows` subcommand module
- [ ] Implement `q workflows list`
- [ ] Implement `q workflows add <path>`
- [ ] Implement `q workflows remove <name>`
- [ ] Implement `q workflows show <name>`
- [ ] Add CLI tests

**Estimated Time**: 8 hours

### 3.3 Validation
**Files**: `crates/chat-cli/src/cli/chat/skill_registry.rs`, `workflow_registry.rs`

- [ ] JSON schema validation for skill definitions
- [ ] JSON schema validation for workflow definitions
- [ ] Validate script paths exist
- [ ] Validate workflow step references
- [ ] Add validation error messages
- [ ] Add validation tests

**Estimated Time**: 6 hours

### 3.4 Documentation
**Files**: 
- `docs/skills-guide.md` (new)
- `docs/workflows-guide.md` (new)
- `README.md`

- [ ] Write skills user guide
  - [ ] Creating skills
  - [ ] Skill definition format
  - [ ] Examples

- [ ] Write workflows user guide
  - [ ] Creating workflows
  - [ ] Workflow definition format
  - [ ] Examples

- [ ] Update main README with skills/workflows section
- [ ] Add inline code documentation

**Estimated Time**: 6 hours

**Phase 3 Total**: ~28 hours (3.5 days)

---

## Phase 4: Advanced Features (Week 5-6)

### 4.1 Templates
**Files**: `crates/chat-cli/src/cli/templates/` (new directory)

- [ ] Create skill template system
- [ ] Create workflow template system
- [ ] Implement `q skills init` - create from template
- [ ] Implement `q workflows init` - create from template
- [ ] Add common templates (bash script, python script, AWS workflow)

**Estimated Time**: 8 hours

### 4.2 Parameter Validation
**Files**: `crates/chat-cli/src/cli/chat/tools/skill.rs`, `workflow.rs`

- [ ] Validate parameters against input schema
- [ ] Type checking
- [ ] Required field validation
- [ ] Enum validation
- [ ] Custom validators
- [ ] Add validation tests

**Estimated Time**: 6 hours

### 4.3 Workflow Dependencies
**Files**: `crates/chat-cli/src/cli/chat/tools/workflow.rs`

- [ ] Add step dependency graph
- [ ] Implement parallel execution for independent steps
- [ ] Add conditional step execution
- [ ] Add step output passing
- [ ] Add tests

**Estimated Time**: 10 hours

### 4.4 Progress Tracking
**Files**: `crates/chat-cli/src/cli/chat/tools/workflow.rs`

- [ ] Add workflow progress events
- [ ] Display progress in UI
- [ ] Add step timing
- [ ] Add cancellation support
- [ ] Add tests

**Estimated Time**: 6 hours

**Phase 4 Total**: ~30 hours (3.75 days)

---

## Testing Strategy

### Unit Tests (Per Phase)
**Phase 1:**
- [ ] `ToolOrigin` serialization/deserialization
- [ ] `SkillTool` and `WorkflowTool` struct creation
- [ ] Skill registry loading from JSON
- [ ] Workflow registry loading from JSON
- [ ] Tool spec conversion accuracy
- [ ] Invalid JSON handling

**Phase 2:**
- [ ] Skill script execution with parameters
- [ ] Skill command execution with parameters
- [ ] Workflow step execution
- [ ] Error propagation
- [ ] Timeout handling
- [ ] Output formatting

**Phase 3:**
- [ ] CLI command parsing
- [ ] Skill add/remove operations
- [ ] Workflow add/remove operations
- [ ] Validation error messages
- [ ] File system operations

**Phase 4:**
- [ ] Template generation
- [ ] Parameter type validation
- [ ] Workflow dependency resolution
- [ ] Progress event emission

**Estimated Time**: 8 hours

### Integration Tests
- [ ] End-to-end skill execution from chat
- [ ] End-to-end workflow execution from chat
- [ ] CLI command execution with real files
- [ ] Error scenarios (missing files, invalid JSON, execution failures)
- [ ] Permission checks
- [ ] Multi-step workflow execution
- [ ] Skill/workflow interaction with other tools

**Estimated Time**: 6 hours

### LLM Interaction Tests
- [ ] LLM can discover skills from schema
- [ ] LLM can invoke skills with correct parameters
- [ ] LLM can handle skill errors gracefully
- [ ] LLM can chain multiple skills
- [ ] LLM can execute workflows
- [ ] LLM parameter mapping accuracy
- [ ] LLM handles missing required parameters

**Estimated Time**: 4 hours

### Performance Tests
- [ ] Skill loading time (<100ms for 100 skills)
- [ ] Workflow loading time (<100ms for 100 workflows)
- [ ] Execution overhead (<50ms)
- [ ] Memory usage (no leaks)
- [ ] Concurrent skill execution

**Estimated Time**: 2 hours

**Total Testing Time**: ~20 hours (2.5 days)

---

## Analysis and Validation

### Phase 1 Analysis (End of Week 2)
**Deliverable**: Analysis Report 1

- [ ] **Code Review**
  - [ ] All types properly defined
  - [ ] Error handling comprehensive
  - [ ] Code follows Q CLI patterns
  - [ ] Documentation complete

- [ ] **Functionality Verification**
  - [ ] Skills load from filesystem
  - [ ] Workflows load from filesystem
  - [ ] Tool specs generated correctly
  - [ ] ToolManager integration works

- [ ] **Test Coverage**
  - [ ] Unit test coverage >80%
  - [ ] All error paths tested
  - [ ] Edge cases covered

- [ ] **Performance Baseline**
  - [ ] Measure loading time
  - [ ] Measure memory usage
  - [ ] Document baseline metrics

**Estimated Time**: 4 hours

### Phase 2 Analysis (End of Week 3)
**Deliverable**: Analysis Report 2

- [ ] **Code Review**
  - [ ] Execution logic secure
  - [ ] Error handling robust
  - [ ] Output formatting correct
  - [ ] Telemetry implemented

- [ ] **Functionality Verification**
  - [ ] Skills execute successfully
  - [ ] Workflows execute successfully
  - [ ] Parameters passed correctly
  - [ ] Output captured properly

- [ ] **Test Coverage**
  - [ ] Integration tests passing
  - [ ] Error scenarios covered
  - [ ] Timeout handling verified

- [ ] **Security Analysis**
  - [ ] Path traversal prevented
  - [ ] Command injection prevented
  - [ ] Resource limits enforced
  - [ ] Permissions checked

- [ ] **Performance Analysis**
  - [ ] Execution overhead measured
  - [ ] Compare to baseline
  - [ ] Identify bottlenecks

**Estimated Time**: 6 hours

### Phase 3 Analysis (End of Week 4)
**Deliverable**: Analysis Report 3

- [ ] **Code Review**
  - [ ] CLI commands intuitive
  - [ ] Validation comprehensive
  - [ ] Error messages helpful
  - [ ] Documentation clear

- [ ] **Usability Testing**
  - [ ] Test with sample users
  - [ ] Collect feedback
  - [ ] Identify pain points
  - [ ] Document improvements

- [ ] **Functionality Verification**
  - [ ] All CLI commands work
  - [ ] Validation catches errors
  - [ ] Help text accurate
  - [ ] Examples work

- [ ] **Documentation Review**
  - [ ] User guide complete
  - [ ] Examples clear
  - [ ] API docs accurate
  - [ ] Troubleshooting section

**Estimated Time**: 6 hours

### Phase 4 Analysis (End of Week 6)
**Deliverable**: Analysis Report 4 + Final Report

- [ ] **Code Review**
  - [ ] Advanced features stable
  - [ ] No regressions
  - [ ] Code quality maintained
  - [ ] Technical debt minimal

- [ ] **Functionality Verification**
  - [ ] Templates work
  - [ ] Validation comprehensive
  - [ ] Dependencies resolve correctly
  - [ ] Progress tracking accurate

- [ ] **Performance Analysis**
  - [ ] Final performance metrics
  - [ ] Compare to baseline
  - [ ] Optimization opportunities
  - [ ] Resource usage acceptable

- [ ] **LLM Effectiveness Analysis**
  - [ ] Success rate of skill invocation
  - [ ] Parameter mapping accuracy
  - [ ] Error recovery effectiveness
  - [ ] User satisfaction metrics

**Estimated Time**: 8 hours

### Final Analysis (End of Week 6)
**Deliverable**: Comprehensive Final Report

- [ ] **Feature Completeness**
  - [ ] All planned features implemented
  - [ ] All acceptance criteria met
  - [ ] No critical bugs
  - [ ] Documentation complete

- [ ] **Quality Metrics**
  - [ ] Test coverage >85%
  - [ ] All tests passing
  - [ ] No security vulnerabilities
  - [ ] Performance targets met

- [ ] **User Readiness**
  - [ ] Documentation reviewed
  - [ ] Examples tested
  - [ ] Migration guide (if needed)
  - [ ] Support plan

- [ ] **Recommendations**
  - [ ] Future enhancements prioritized
  - [ ] Known limitations documented
  - [ ] Maintenance plan
  - [ ] Monitoring strategy

**Estimated Time**: 4 hours

**Total Analysis Time**: ~28 hours (3.5 days)

---

## Milestones

### M1: Core Infrastructure Complete (End of Week 2)
- ✓ Types extended
- ✓ Registry system working
- ✓ ToolManager integration
- ✓ Basic loading functional

### M2: Execution Working (End of Week 3)
- ✓ Skills can execute
- ✓ Workflows can execute
- ✓ Output formatting complete
- ✓ Error handling in place

### M3: CLI Complete (End of Week 4)
- ✓ All CLI commands working
- ✓ Validation in place
- ✓ Documentation written
- ✓ Ready for user testing

### M4: Advanced Features (End of Week 6)
- ✓ Templates available
- ✓ Advanced validation
- ✓ Workflow dependencies
- ✓ Progress tracking

---

## Total Effort Estimate

- **Phase 1**: 28 hours (3.5 days)
- **Phase 2**: 22 hours (2.75 days)
- **Phase 3**: 28 hours (3.5 days)
- **Phase 4**: 30 hours (3.75 days)
- **Testing**: 20 hours (2.5 days)
- **Analysis**: 28 hours (3.5 days)

**Total**: ~156 hours (~19.5 days of development)

With buffer for code review, bug fixes, and iteration: **~25 days (5 weeks)**

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
