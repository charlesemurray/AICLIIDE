use chat_cli::cli::chat::tools::{Tool, ToolOrigin};
use chat_cli::cli::skills::toolspec_conversion::ToToolSpec;
use chat_cli::cli::workflow::types::{StepType, Workflow, WorkflowInput, WorkflowStep};
use chat_cli::os::Os;

#[tokio::test]
async fn test_workflow_invocation_via_natural_language() {
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

#[tokio::test]
async fn test_workflow_execution_through_tool_enum() {
    let os = Os::new().await.unwrap();
    let agents = chat_cli::cli::agent::Agents::default();

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

    let workflow_tool = chat_cli::cli::chat::tools::workflow_tool::WorkflowTool::new(workflow, serde_json::json!({}));
    let tool = Tool::Workflow(workflow_tool);

    let mut output = Vec::new();
    let mut line_tracker = std::collections::HashMap::new();

    let result = tool.invoke(&os, &mut output, &mut line_tracker, &agents).await;
    assert!(result.is_ok(), "Workflow execution should succeed");

    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("50"), "Output should contain result");
}

#[tokio::test]
async fn test_workflow_with_multiple_steps() {
    let os = Os::new().await.unwrap();
    let agents = chat_cli::cli::agent::Agents::default();

    let workflow = Workflow {
        name: "multi-step-workflow".to_string(),
        description: "Multi-step workflow".to_string(),
        version: "1.0.0".to_string(),
        steps: vec![
            WorkflowStep {
                id: "step1".to_string(),
                step_type: StepType::Skill {
                    name: "calculator".to_string(),
                    inputs: serde_json::json!({"a": 5.0, "b": 3.0, "op": "add"}),
                },
            },
            WorkflowStep {
                id: "step2".to_string(),
                step_type: StepType::Skill {
                    name: "calculator".to_string(),
                    inputs: serde_json::json!({"a": 10.0, "b": 2.0, "op": "subtract"}),
                },
            },
        ],
        inputs: vec![],
    };

    let workflow_tool = chat_cli::cli::chat::tools::workflow_tool::WorkflowTool::new(workflow, serde_json::json!({}));
    let tool = Tool::Workflow(workflow_tool);

    let mut output = Vec::new();
    let mut line_tracker = std::collections::HashMap::new();

    let result = tool.invoke(&os, &mut output, &mut line_tracker, &agents).await;
    assert!(result.is_ok(), "Multi-step workflow should succeed");

    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("8"), "Output should contain first result");
    assert!(output_str.contains("8"), "Output should contain second result");
}

#[tokio::test]
async fn test_workflow_with_inputs() {
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
