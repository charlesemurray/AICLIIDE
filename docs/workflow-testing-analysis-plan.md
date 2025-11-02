# Workflow System Testing & Analysis Plan

## Overview

Comprehensive testing strategy and success metrics for the workflow generation and iteration system.

## Testing Strategy

### Level 1: Unit Tests (Per Component)

#### Resource Discovery Tests
```rust
// In workflow/discovery.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_discover_skills() {
        let discovery = ResourceDiscovery::new().await.unwrap();
        let resources = discovery.discover_all().await;
        
        assert!(!resources.skills.is_empty(), "Should discover at least builtin skills");
    }
    
    #[tokio::test]
    async fn test_discover_agents() {
        let discovery = ResourceDiscovery::new().await.unwrap();
        let resources = discovery.discover_all().await;
        
        // May be empty if no agents configured
        assert!(resources.agents.len() >= 0);
    }
    
    #[test]
    fn test_format_for_llm() {
        let resources = DiscoveredResources {
            skills: vec![
                SkillInfo {
                    name: "test-skill".to_string(),
                    description: "Test skill".to_string(),
                    skill_type: "code_inline".to_string(),
                }
            ],
            agents: vec![],
            mcp_servers: vec![],
        };
        
        let discovery = ResourceDiscovery::new().await.unwrap();
        let formatted = discovery.format_for_llm(&resources);
        
        assert!(formatted.contains("test-skill"));
        assert!(formatted.contains("Available Skills:"));
    }
}
```

#### Generator Tests
```rust
// In workflow/generator.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_json_from_markdown() {
        let response = r#"
Here's the workflow:

```json
{
  "workflow": {
    "name": "test",
    "version": "1.0.0",
    "steps": []
  },
  "new_skills": [],
  "reasoning": "test"
}
```
"#;
        
        let generator = WorkflowGenerator::new().await.unwrap();
        let json = generator.extract_json(response).unwrap();
        
        assert!(json.contains("\"name\": \"test\""));
    }
    
    #[test]
    fn test_parse_generation_response() {
        let json = r#"{
            "workflow": {
                "name": "test-workflow",
                "version": "1.0.0",
                "steps": []
            },
            "new_skills": [],
            "reasoning": "Simple test workflow"
        }"#;
        
        let generator = WorkflowGenerator::new().await.unwrap();
        let result = generator.parse_generation_response(json).unwrap();
        
        assert_eq!(result.workflow.name, "test-workflow");
        assert_eq!(result.new_skills.len(), 0);
    }
    
    #[tokio::test]
    async fn test_build_generation_prompt() {
        let generator = WorkflowGenerator::new().await.unwrap();
        let resources = DiscoveredResources::default();
        
        let prompt = generator.build_generation_prompt(&resources);
        
        assert!(prompt.contains("Available Skills:"));
        assert!(prompt.contains("Generate a workflow JSON"));
    }
}
```

#### Editor Tests
```rust
// In workflow/editor.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_apply_add_step_modification() {
        let mut workflow = Workflow {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            steps: vec![
                Step::Skill(SkillStep {
                    id: "step1".to_string(),
                    skill: "test-skill".to_string(),
                    inputs: HashMap::new(),
                    outputs: vec![],
                    timeout: None,
                })
            ],
            context: WorkflowContext::default(),
        };
        
        let modification = WorkflowModification::AddStep {
            after: "step1".to_string(),
            step: Step::Skill(SkillStep {
                id: "step2".to_string(),
                skill: "another-skill".to_string(),
                inputs: HashMap::new(),
                outputs: vec![],
                timeout: None,
            }),
        };
        
        let editor = WorkflowEditor::new().await.unwrap();
        let updated = editor.apply_single_modification(workflow, modification).unwrap();
        
        assert_eq!(updated.steps.len(), 2);
        assert_eq!(updated.steps[1].id(), "step2");
    }
    
    #[test]
    fn test_apply_remove_step_modification() {
        let mut workflow = create_test_workflow_with_two_steps();
        
        let modification = WorkflowModification::RemoveStep {
            id: "step1".to_string(),
        };
        
        let editor = WorkflowEditor::new().await.unwrap();
        let updated = editor.apply_single_modification(workflow, modification).unwrap();
        
        assert_eq!(updated.steps.len(), 1);
        assert_eq!(updated.steps[0].id(), "step2");
    }
    
    #[test]
    fn test_build_edit_prompt_with_scope() {
        let workflow = create_test_workflow();
        let editor = WorkflowEditor::new().await.unwrap();
        
        let prompt = editor.build_edit_prompt(&workflow, Some(EditScope::Step("step1".to_string())));
        
        assert!(prompt.contains("ONLY modify step 'step1'"));
        assert!(prompt.contains("Do not change other steps"));
    }
}
```

#### Versioning Tests
```rust
// In workflow/versioning.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_save_and_load_version() {
        let version_manager = VersionManager::new().unwrap();
        let workflow = create_test_workflow();
        
        version_manager.save_version(&workflow).unwrap();
        
        let loaded = version_manager.load_version(&workflow.name, 1).unwrap();
        
        assert_eq!(loaded.name, workflow.name);
        assert_eq!(loaded.steps.len(), workflow.steps.len());
    }
    
    #[test]
    fn test_list_versions() {
        let version_manager = VersionManager::new().unwrap();
        let workflow = create_test_workflow();
        
        // Save multiple versions
        version_manager.save_version(&workflow).unwrap();
        
        let mut workflow_v2 = workflow.clone();
        workflow_v2.version = "2.0.0".to_string();
        version_manager.save_version(&workflow_v2).unwrap();
        
        let versions = version_manager.list_versions(&workflow.name).unwrap();
        
        assert_eq!(versions.len(), 2);
        assert_eq!(versions[0].version, 2); // Most recent first
        assert_eq!(versions[1].version, 1);
    }
    
    #[test]
    fn test_parse_version_number() {
        let version_manager = VersionManager::new().unwrap();
        
        assert_eq!(version_manager.parse_version_number("1.0.0"), 1);
        assert_eq!(version_manager.parse_version_number("2.5.3"), 2);
        assert_eq!(version_manager.parse_version_number("10.0.0"), 10);
    }
}
```

### Level 2: Integration Tests (Cross-Component)

```rust
// In workflow/tests/integration_tests.rs
#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_end_to_end_workflow_creation() {
        // Test complete flow: generate -> approve -> save
        let generator = WorkflowGenerator::new().await.unwrap();
        
        let generated = generator.generate_from_prompt(
            "fetch data from API and save to file"
        ).await.unwrap();
        
        assert!(!generated.workflow.steps.is_empty());
        assert!(!generated.workflow.name.is_empty());
        
        // Save workflow
        generated.workflow.save().await.unwrap();
        
        // Verify it can be loaded
        let loaded = Workflow::load(&generated.workflow.name).await.unwrap();
        assert_eq!(loaded.name, generated.workflow.name);
    }
    
    #[tokio::test]
    async fn test_edit_and_version_workflow() {
        // Create initial workflow
        let generator = WorkflowGenerator::new().await.unwrap();
        let generated = generator.generate_from_prompt("simple workflow").await.unwrap();
        generated.workflow.save().await.unwrap();
        
        // Edit workflow
        let editor = WorkflowEditor::new().await.unwrap();
        editor.edit_workflow(
            &generated.workflow.name,
            "add error handling",
            None
        ).await.unwrap();
        
        // Check version was saved
        let version_manager = VersionManager::new().unwrap();
        let versions = version_manager.list_versions(&generated.workflow.name).unwrap();
        
        assert!(versions.len() >= 1);
    }
    
    #[tokio::test]
    async fn test_rollback_workflow() {
        // Create and edit workflow multiple times
        let generator = WorkflowGenerator::new().await.unwrap();
        let generated = generator.generate_from_prompt("test workflow").await.unwrap();
        let workflow_name = generated.workflow.name.clone();
        generated.workflow.save().await.unwrap();
        
        let editor = WorkflowEditor::new().await.unwrap();
        editor.edit_workflow(&workflow_name, "add step", None).await.unwrap();
        editor.edit_workflow(&workflow_name, "add another step", None).await.unwrap();
        
        // Rollback to v1
        let version_manager = VersionManager::new().unwrap();
        version_manager.rollback(&workflow_name, 1, false).unwrap();
        
        // Verify rolled back
        let loaded = Workflow::load(&workflow_name).await.unwrap();
        assert_eq!(loaded.version, "1.0.0");
    }
    
    #[tokio::test]
    async fn test_skill_creation_integration() {
        // Test that generated skills can be created and used
        let generator = WorkflowGenerator::new().await.unwrap();
        let generated = generator.generate_from_prompt(
            "do something that requires a new skill"
        ).await.unwrap();
        
        if !generated.new_skills.is_empty() {
            let skill_spec = &generated.new_skills[0];
            
            // Create skill JSON
            let skill_json = serde_json::json!({
                "name": skill_spec.name,
                "type": skill_spec.skill_type,
                "description": skill_spec.description,
            });
            
            // Save skill
            let skills_dir = crate::util::paths::workspace_skills_dir().unwrap();
            std::fs::create_dir_all(&skills_dir).unwrap();
            let path = skills_dir.join(format!("{}.json", skill_spec.name));
            std::fs::write(path, serde_json::to_string_pretty(&skill_json).unwrap()).unwrap();
            
            // Verify skill can be discovered
            let discovery = ResourceDiscovery::new().await.unwrap();
            let resources = discovery.discover_all().await;
            
            assert!(resources.skills.iter().any(|s| s.name == skill_spec.name));
        }
    }
}
```

### Level 3: End-to-End Tests (CLI)

```rust
// In workflow/tests/cli_tests.rs
#[cfg(test)]
mod cli_tests {
    use std::process::Command;
    
    #[test]
    fn test_workflow_create_command() {
        let output = Command::new("cargo")
            .args(&["run", "--bin", "chat_cli", "--", "workflow", "create", "test workflow"])
            .output()
            .expect("Failed to execute command");
        
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Analyzing your request"));
    }
    
    #[test]
    fn test_workflow_edit_command() {
        // First create a workflow
        Command::new("cargo")
            .args(&["run", "--bin", "chat_cli", "--", "workflow", "create", "test workflow"])
            .output()
            .expect("Failed to create workflow");
        
        // Then edit it
        let output = Command::new("cargo")
            .args(&["run", "--bin", "chat_cli", "--", "workflow", "edit", "test-workflow", "add step"])
            .output()
            .expect("Failed to execute command");
        
        assert!(output.status.success());
    }
    
    #[test]
    fn test_workflow_history_command() {
        let output = Command::new("cargo")
            .args(&["run", "--bin", "chat_cli", "--", "workflow", "history", "test-workflow"])
            .output()
            .expect("Failed to execute command");
        
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Workflow:"));
    }
}
```

### Level 4: LLM Response Tests (Mock & Real)

```rust
// In workflow/tests/llm_tests.rs
#[cfg(test)]
mod llm_tests {
    use super::*;
    
    #[test]
    fn test_parse_valid_llm_response() {
        let mock_response = r#"
Here's the workflow I generated:

```json
{
  "workflow": {
    "name": "api-monitor",
    "version": "1.0.0",
    "description": "Monitor API health",
    "steps": [
      {
        "id": "fetch",
        "type": "skill",
        "skill": "http-fetch",
        "inputs": {"url": "context.api_url"}
      }
    ]
  },
  "new_skills": [],
  "reasoning": "Simple API monitoring workflow"
}
```
"#;
        
        let generator = WorkflowGenerator::new().await.unwrap();
        let result = generator.parse_generation_response(mock_response).unwrap();
        
        assert_eq!(result.workflow.name, "api-monitor");
        assert_eq!(result.workflow.steps.len(), 1);
    }
    
    #[test]
    fn test_parse_llm_response_with_new_skills() {
        let mock_response = r#"
```json
{
  "workflow": {
    "name": "data-pipeline",
    "version": "1.0.0",
    "steps": []
  },
  "new_skills": [
    {
      "name": "data-validator",
      "type": "code_inline",
      "description": "Validate data",
      "command": "python validate.py",
      "reasoning": "Need to validate data structure"
    }
  ],
  "reasoning": "Pipeline needs validation"
}
```
"#;
        
        let generator = WorkflowGenerator::new().await.unwrap();
        let result = generator.parse_generation_response(mock_response).unwrap();
        
        assert_eq!(result.new_skills.len(), 1);
        assert_eq!(result.new_skills[0].name, "data-validator");
    }
    
    #[tokio::test]
    #[ignore] // Only run with real API access
    async fn test_real_llm_generation() {
        let generator = WorkflowGenerator::new().await.unwrap();
        
        let result = generator.generate_from_prompt(
            "Create a workflow that checks if a file exists and processes it"
        ).await;
        
        assert!(result.is_ok());
        let generated = result.unwrap();
        assert!(!generated.workflow.steps.is_empty());
    }
}
```

## Success Metrics & Analysis

### Phase Completion Criteria

#### Phase 1: Resource Discovery
**Metrics:**
- [ ] Discovers all builtin skills (100%)
- [ ] Discovers workspace skills (if present)
- [ ] Discovers global agents (if present)
- [ ] Discovers MCP servers (if configured)
- [ ] Formats output correctly for LLM

**Analysis:**
```bash
# Run discovery and check output
cargo test --package chat-cli --lib workflow::discovery::tests

# Manual verification
q workflow debug discovery
```

#### Phase 2: Basic Generation
**Metrics:**
- [ ] Successfully generates workflow from simple prompt (90%+ success rate)
- [ ] Parses LLM response correctly (95%+ success rate)
- [ ] Identifies existing skills correctly (100%)
- [ ] Identifies missing skills correctly (80%+)
- [ ] Generated workflows are valid JSON (100%)

**Analysis:**
```bash
# Test with various prompts
cargo test --package chat-cli --lib workflow::generator::tests

# Manual testing
q workflow create "fetch data from API"
q workflow create "process files in parallel"
q workflow create "monitor system health"
```

**Success Criteria:**
- 9/10 simple prompts generate valid workflows
- 7/10 complex prompts generate valid workflows
- All generated JSON passes schema validation

#### Phase 3: Approval Flow
**Metrics:**
- [ ] Summary displays correctly
- [ ] User can view full workflow
- [ ] User can approve/reject
- [ ] Workflow saves correctly after approval

**Analysis:**
```bash
# Integration test
cargo test --package chat-cli workflow::tests::integration_tests::test_end_to_end_workflow_creation

# Manual UX testing
q workflow create "test workflow"
# Verify prompts are clear
# Verify choices work correctly
```

#### Phase 4: Skill Creation
**Metrics:**
- [ ] Skills generated from specs
- [ ] Skills saved to correct location
- [ ] Skills discoverable after creation
- [ ] Skills usable in workflows

**Analysis:**
```bash
# Test skill creation
cargo test --package chat-cli workflow::tests::integration_tests::test_skill_creation_integration

# Verify skill files
ls ~/.amazonq/skills/
cat ~/.amazonq/skills/generated-skill.json
```

#### Phase 5: Workflow Editing
**Metrics:**
- [ ] Generates modifications (not full rewrites) (100%)
- [ ] Modifications are surgical (90%+)
- [ ] Data flow preserved (100%)
- [ ] Diff preview accurate (100%)

**Analysis:**
```bash
# Test editing
cargo test --package chat-cli --lib workflow::editor::tests

# Manual testing
q workflow create "simple workflow"
q workflow edit simple-workflow "add retry logic"
# Verify only retry logic added, nothing else changed
```

**Success Criteria:**
- 95%+ of edits modify only requested parts
- 0% of edits break existing data flow
- Users understand what will change before applying

#### Phase 6: Versioning
**Metrics:**
- [ ] Versions saved automatically (100%)
- [ ] Version history accurate (100%)
- [ ] Rollback works correctly (100%)
- [ ] No data loss on rollback (100%)

**Analysis:**
```bash
# Test versioning
cargo test --package chat-cli --lib workflow::versioning::tests

# Manual testing
q workflow create "test"
q workflow edit test "change 1"
q workflow edit test "change 2"
q workflow history test
# Should show v1, v2, v3

q workflow rollback test --to v1
# Verify rolled back correctly
```

#### Phase 7: Interactive Refinement
**Metrics:**
- [ ] Menu navigation works (100%)
- [ ] Multi-turn conversation maintains context (90%+)
- [ ] Changes queue correctly (100%)
- [ ] Batch application works (100%)

**Analysis:**
```bash
# Test refinement
cargo test --package chat-cli --lib workflow::refine::tests

# Manual UX testing
q workflow refine test-workflow
# Go through multiple iterations
# Verify context maintained
```

#### Phase 8: Diff & Validation
**Metrics:**
- [ ] Diffs accurate (100%)
- [ ] Impact analysis correct (95%+)
- [ ] Warnings for destructive changes (100%)
- [ ] Validation catches errors (95%+)

**Analysis:**
```bash
# Test diff generation
cargo test --package chat-cli --lib workflow::diff::tests

# Manual testing
q workflow diff test-workflow --from v1 --to v3
# Verify diff is clear and accurate
```

### Overall System Metrics

#### Quality Metrics
- **Workflow Generation Success Rate**: 85%+ of prompts generate valid workflows
- **Edit Accuracy**: 95%+ of edits modify only requested parts
- **Version Safety**: 100% of versions can be restored
- **Data Integrity**: 0% data loss or corruption

#### Performance Metrics
- **Generation Time**: < 10 seconds for simple workflows
- **Edit Time**: < 5 seconds for simple edits
- **Discovery Time**: < 2 seconds to discover all resources

#### User Experience Metrics
- **Time to First Workflow**: < 2 minutes from prompt to saved workflow
- **Iteration Speed**: < 1 minute per edit
- **Error Recovery**: < 30 seconds to rollback

### Continuous Analysis

#### Daily Checks
```bash
# Run all tests
cargo test --package chat-cli --lib workflow

# Check test coverage
cargo tarpaulin --package chat-cli --lib workflow

# Run lints
cargo clippy --package chat-cli --lib workflow
```

#### Weekly Analysis
```bash
# Generate test report
cargo test --package chat-cli --lib workflow -- --test-threads=1 --nocapture > test_report.txt

# Analyze failures
grep "FAILED" test_report.txt

# Check performance
cargo bench --package chat-cli --lib workflow
```

#### Phase Completion Checklist
```markdown
## Phase X Completion

### Tests
- [ ] All unit tests passing
- [ ] All integration tests passing
- [ ] Manual testing completed
- [ ] Edge cases tested

### Metrics
- [ ] Success rate meets target
- [ ] Performance meets target
- [ ] No regressions in previous phases

### Documentation
- [ ] Code documented
- [ ] User docs updated
- [ ] Examples added

### Review
- [ ] Code review completed
- [ ] UX review completed
- [ ] Security review completed
```

## Test Data & Fixtures

### Sample Workflows
```rust
// In workflow/tests/fixtures.rs
pub fn create_simple_workflow() -> Workflow {
    Workflow {
        name: "simple-test".to_string(),
        version: "1.0.0".to_string(),
        description: Some("Test workflow".to_string()),
        steps: vec![
            Step::Skill(SkillStep {
                id: "step1".to_string(),
                skill: "echo".to_string(),
                inputs: HashMap::new(),
                outputs: vec!["output".to_string()],
                timeout: None,
            })
        ],
        context: WorkflowContext::default(),
    }
}

pub fn create_complex_workflow() -> Workflow {
    // Multi-step workflow with parallel execution, conditionals, etc.
}

pub fn create_mock_llm_response() -> String {
    r#"{"workflow": {...}, "new_skills": [], "reasoning": "..."}"#.to_string()
}
```

## Regression Testing

After each phase, run full regression suite:
```bash
# Run all previous phase tests
cargo test --package chat-cli --lib workflow::discovery
cargo test --package chat-cli --lib workflow::generator
cargo test --package chat-cli --lib workflow::editor
# etc.

# Verify no regressions
diff previous_test_results.txt current_test_results.txt
```

## Success Criteria Summary

**MVP (Phases 1-4) Success:**
- 85%+ workflow generation success rate
- All tests passing
- Users can create workflows in < 2 minutes
- Generated workflows are valid and executable

**Full System (Phases 1-8) Success:**
- 90%+ generation success rate
- 95%+ edit accuracy
- 100% version safety
- Users can iterate confidently without fear
- Average workflow reaches v5+ (shows active iteration)
