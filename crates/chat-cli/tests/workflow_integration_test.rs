use std::collections::HashMap;
use serde_json::json;

use chat_cli::cli::chat::tools::workflow::{WorkflowDefinition, WorkflowStep, WorkflowTool};
use chat_cli::cli::workflows::validation::validate_workflow;

#[test]
fn test_cycle_detection_self_reference() {
    let workflow = WorkflowDefinition {
        name: "cycle-test".to_string(),
        version: "1.0.0".to_string(),
        description: "Test cycle detection".to_string(),
        steps: vec![
            WorkflowStep {
                name: "step1".to_string(),
                tool: "step1".to_string(), // Self-reference
                parameters: json!({}),
            }
        ],
        context: None,
    };

    let result = validate_workflow(&workflow);
    assert!(result.is_err(), "Should detect self-referencing cycle");
    assert!(result.unwrap_err().to_string().contains("cycle"));
}

#[test]
fn test_valid_workflow_passes_validation() {
    let workflow = WorkflowDefinition {
        name: "valid-workflow".to_string(),
        version: "1.0.0".to_string(),
        description: "Valid workflow".to_string(),
        steps: vec![
            WorkflowStep {
                name: "step1".to_string(),
                tool: "calculator".to_string(),
                parameters: json!({"operation": "add", "a": 1, "b": 2}),
            }
        ],
        context: None,
    };

    let result = validate_workflow(&workflow);
    assert!(result.is_ok(), "Valid workflow should pass validation");
}

#[tokio::test]
async fn test_workflow_execution_basic() {
    let workflow = WorkflowDefinition {
        name: "basic-test".to_string(),
        version: "1.0.0".to_string(),
        description: "Basic execution test".to_string(),
        steps: vec![
            WorkflowStep {
                name: "echo".to_string(),
                tool: "execute_bash_readonly".to_string(),
                parameters: json!({"command": "echo 'test'"}),
            }
        ],
        context: None,
    };

    let tool = WorkflowTool::from_definition(&workflow);
    let params = HashMap::new();
    
    let result = tool.invoke_with_definition(&workflow, params);
    assert!(result.is_ok(), "Basic workflow should execute successfully");
    
    let output = result.unwrap();
    assert!(output.contains("Executed 1 steps successfully"));
}

#[tokio::test]
async fn test_workflow_timeout() {
    use std::time::Duration;
    use tokio::time::timeout;
    
    let workflow = WorkflowDefinition {
        name: "timeout-test".to_string(),
        version: "1.0.0".to_string(),
        description: "Timeout test".to_string(),
        steps: vec![
            WorkflowStep {
                name: "long-sleep".to_string(),
                tool: "execute_bash".to_string(),
                parameters: json!({"command": "sleep 10"}),
            }
        ],
        context: None,
    };

    let tool = WorkflowTool::from_definition(&workflow);
    let params = HashMap::new();
    
    // This should timeout (workflow has 5min timeout per step, but we test with shorter)
    let result = timeout(Duration::from_secs(2), async {
        tool.invoke_with_definition(&workflow, params)
    }).await;
    
    assert!(result.is_err(), "Long-running workflow should timeout");
}

#[test]
fn test_workflow_permission_checking() {
    use chat_cli::cli::agent::{Agent, PermissionEvalResult};
    use chat_cli::os::Os;
    
    let workflow = WorkflowDefinition {
        name: "permission-test".to_string(),
        version: "1.0.0".to_string(),
        description: "Permission test".to_string(),
        steps: vec![
            WorkflowStep {
                name: "dangerous".to_string(),
                tool: "execute_bash".to_string(),
                parameters: json!({"command": "rm -rf /tmp/test"}),
            }
        ],
        context: None,
    };

    let tool = WorkflowTool::from_definition(&workflow);
    let os = Os::default();
    let agent = Agent::default();
    
    let perm = tool.eval_perm(&os, &agent);
    assert_eq!(perm, PermissionEvalResult::Ask, "Workflows should require permission");
}

#[test]
fn test_empty_workflow_fails_validation() {
    let workflow = WorkflowDefinition {
        name: "empty".to_string(),
        version: "1.0.0".to_string(),
        description: "Empty workflow".to_string(),
        steps: vec![],
        context: None,
    };

    let result = validate_workflow(&workflow);
    assert!(result.is_err(), "Empty workflow should fail validation");
}

#[test]
fn test_invalid_name_fails_validation() {
    let workflow = WorkflowDefinition {
        name: "invalid name with spaces".to_string(),
        version: "1.0.0".to_string(),
        description: "Invalid name".to_string(),
        steps: vec![
            WorkflowStep {
                name: "step1".to_string(),
                tool: "echo".to_string(),
                parameters: json!({}),
            }
        ],
        context: None,
    };

    let result = validate_workflow(&workflow);
    assert!(result.is_err(), "Invalid name should fail validation");
}
