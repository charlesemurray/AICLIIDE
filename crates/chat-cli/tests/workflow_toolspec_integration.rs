use chat_cli::cli::chat::tools::ToolOrigin;
use chat_cli::cli::skills::toolspec_conversion::ToToolSpec;
use chat_cli::cli::workflow::types::{
    StepType,
    Workflow,
    WorkflowInput,
    WorkflowStep,
};

#[test]
fn test_workflow_to_toolspec_conversion() {
    // Create a workflow
    let workflow = Workflow {
        name: "test-workflow".to_string(),
        description: "Test workflow".to_string(),
        version: "1.0.0".to_string(),
        steps: vec![WorkflowStep {
            id: "step1".to_string(),
            step_type: StepType::Skill {
                name: "calculator".to_string(),
                inputs: serde_json::json!({"a": 5.0, "b": 3.0, "op": "add"}),
            },
        }],
        inputs: vec![],
    };

    // Convert to ToolSpec
    let toolspec = workflow.to_toolspec().unwrap();
    assert_eq!(toolspec.name, "test-workflow");
    assert!(matches!(toolspec.tool_origin, ToolOrigin::Workflow(_)));
}

#[test]
fn test_workflow_with_inputs_schema() {
    let workflow = Workflow {
        name: "input-workflow".to_string(),
        description: "Workflow with inputs".to_string(),
        version: "1.0.0".to_string(),
        steps: vec![],
        inputs: vec![
            WorkflowInput {
                name: "value1".to_string(),
                input_type: "number".to_string(),
                required: true,
            },
            WorkflowInput {
                name: "value2".to_string(),
                input_type: "number".to_string(),
                required: false,
            },
        ],
    };

    let toolspec = workflow.to_toolspec().unwrap();
    let schema = &toolspec.input_schema.0;

    // Verify schema
    assert!(schema["properties"]["value1"].is_object());
    assert!(schema["properties"]["value2"].is_object());

    let required = schema["required"].as_array().unwrap();
    assert_eq!(required.len(), 1);
    assert!(required.contains(&serde_json::json!("value1")));
}

#[tokio::test]
async fn test_workflow_executor_integration() {
    use chat_cli::cli::skills::SkillRegistry;
    use chat_cli::cli::workflow::WorkflowExecutor;

    let registry = SkillRegistry::with_builtins();
    let executor = WorkflowExecutor::new(registry);

    let workflow = Workflow {
        name: "calc-workflow".to_string(),
        description: "Calculator workflow".to_string(),
        version: "1.0.0".to_string(),
        steps: vec![WorkflowStep {
            id: "step1".to_string(),
            step_type: StepType::Skill {
                name: "calculator".to_string(),
                inputs: serde_json::json!({"a": 10.0, "b": 5.0, "op": "multiply"}),
            },
        }],
        inputs: vec![],
    };

    let result = executor.execute(&workflow, serde_json::json!({})).await;
    assert!(result.is_ok(), "Workflow execution should succeed");

    let output = result.unwrap();
    assert!(output.contains("50"), "Output should contain result");
}
