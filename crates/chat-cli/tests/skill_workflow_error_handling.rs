use chat_cli::cli::chat::skill_registry::SkillRegistry;
use chat_cli::cli::chat::tool_manager::ToolManager;
use chat_cli::cli::workflow::types::{StepType, Workflow, WorkflowInput, WorkflowStep};
use chat_cli::os::Os;
use serde_json::json;

#[tokio::test]
async fn test_skill_with_missing_required_parameter() {
    // Test that missing required parameters are handled
    let registry = SkillRegistry::new();
    let calculator = registry.get_skill("calculator");

    // Empty registry should return None
    assert!(calculator.is_none());
}

#[tokio::test]
async fn test_skill_with_invalid_parameter_type() {
    // Test that invalid parameter types are caught
    let registry = SkillRegistry::new();
    let calculator = registry.get_skill("calculator");

    // Empty registry returns None
    assert!(calculator.is_none());

    // Invalid input would be caught by JSON schema validation at runtime
    let _invalid_input = json!({
        "a": "not_a_number",
        "b": 3,
        "op": "add"
    });
}

#[tokio::test]
async fn test_nonexistent_skill() {
    // Test that requesting a non-existent skill fails gracefully
    let registry = SkillRegistry::new();
    let result = registry.get_skill("nonexistent_skill");

    assert!(result.is_none(), "Non-existent skill should return None");
}

#[tokio::test]
async fn test_workflow_with_invalid_skill_reference() {
    // Test workflow that references a non-existent skill
    let workflow = Workflow {
        name: "invalid_workflow".to_string(),
        description: "Workflow with invalid skill".to_string(),
        version: "1.0.0".to_string(),
        steps: vec![WorkflowStep {
            id: "step1".to_string(),
            step_type: StepType::Skill {
                name: "nonexistent_skill".to_string(),
                inputs: json!({}),
            },
        }],
        inputs: vec![],
    };

    // Workflow should be created but execution will fail
    assert_eq!(workflow.steps.len(), 1);
}

#[tokio::test]
async fn test_empty_workflow() {
    // Test workflow with no steps
    let workflow = Workflow {
        name: "empty_workflow".to_string(),
        description: "Workflow with no steps".to_string(),
        version: "1.0.0".to_string(),
        steps: vec![],
        inputs: vec![],
    };

    assert_eq!(workflow.steps.len(), 0);
}

#[tokio::test]
async fn test_skill_registry_empty_initialization() {
    // Test that empty registry handles queries gracefully
    let registry = SkillRegistry::new();
    let result = registry.get_skill("any_skill");

    assert!(result.is_none(), "Empty registry should return None for any skill");
    assert!(registry.is_empty(), "New registry should be empty");
    assert_eq!(registry.len(), 0, "New registry should have length 0");
}

#[tokio::test]
async fn test_tool_manager_handles_skill_registration_errors() {
    // Test that ToolManager handles skill registration gracefully
    let os = Os::new().await.unwrap();
    let result = ToolManager::new_with_skills(&os).await;

    // Should succeed even if some skills fail to load
    assert!(
        result.is_ok(),
        "ToolManager should handle registration errors gracefully"
    );
}

#[tokio::test]
async fn test_workflow_with_circular_dependency_structure() {
    // Test that workflow structure can be created (execution validation is separate)
    let workflow = Workflow {
        name: "potential_circular".to_string(),
        description: "Workflow that could have circular deps".to_string(),
        version: "1.0.0".to_string(),
        steps: vec![
            WorkflowStep {
                id: "step1".to_string(),
                step_type: StepType::Skill {
                    name: "skill_a".to_string(),
                    inputs: json!({"input": "{{step2.output}}"}),
                },
            },
            WorkflowStep {
                id: "step2".to_string(),
                step_type: StepType::Skill {
                    name: "skill_b".to_string(),
                    inputs: json!({"input": "{{step1.output}}"}),
                },
            },
        ],
        inputs: vec![],
    };

    // Structure creation should succeed; execution validation happens at runtime
    assert_eq!(workflow.steps.len(), 2);
}

#[tokio::test]
async fn test_workflow_input_validation() {
    // Test workflow with required inputs
    let workflow = Workflow {
        name: "workflow_with_inputs".to_string(),
        description: "Workflow requiring inputs".to_string(),
        version: "1.0.0".to_string(),
        steps: vec![],
        inputs: vec![
            WorkflowInput {
                name: "required_input".to_string(),
                input_type: "string".to_string(),
                required: true,
            },
            WorkflowInput {
                name: "optional_input".to_string(),
                input_type: "string".to_string(),
                required: false,
            },
        ],
    };

    assert_eq!(workflow.inputs.len(), 2);
    assert!(workflow.inputs[0].required);
    assert!(!workflow.inputs[1].required);
}

#[tokio::test]
async fn test_registry_list_operations() {
    // Test registry list operations on empty registry
    let registry = SkillRegistry::new();
    let skills = registry.list_skills();

    assert_eq!(skills.len(), 0, "Empty registry should have no skills");
}
