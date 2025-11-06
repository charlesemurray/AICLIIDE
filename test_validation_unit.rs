// Standalone unit test for workflow validation
// Run with: rustc --test test_validation_unit.rs && ./test_validation_unit

use std::collections::HashMap;

// Minimal types needed for testing
#[derive(Debug, Clone)]
struct WorkflowStep {
    name: String,
    tool: String,
    parameters: serde_json::Value,
}

#[derive(Debug, Clone)]
struct WorkflowDefinition {
    name: String,
    version: String,
    description: String,
    steps: Vec<WorkflowStep>,
}

// Validation function (simplified)
fn validate_workflow(workflow: &WorkflowDefinition) -> Result<(), String> {
    if workflow.name.is_empty() {
        return Err("Workflow name cannot be empty".to_string());
    }
    
    if !workflow.name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err("Workflow name can only contain alphanumeric characters, hyphens, and underscores".to_string());
    }
    
    if workflow.steps.is_empty() {
        return Err("Workflow must have at least one step".to_string());
    }
    
    // Check for self-referencing cycles
    for step in &workflow.steps {
        if step.tool == step.name {
            return Err("Workflow contains cycles - steps cannot reference themselves".to_string());
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_workflow() {
        let workflow = WorkflowDefinition {
            name: "test-workflow".to_string(),
            version: "1.0.0".to_string(),
            description: "Test".to_string(),
            steps: vec![
                WorkflowStep {
                    name: "step1".to_string(),
                    tool: "calculator".to_string(),
                    parameters: serde_json::json!({}),
                }
            ],
        };
        
        assert!(validate_workflow(&workflow).is_ok());
    }
    
    #[test]
    fn test_empty_name() {
        let workflow = WorkflowDefinition {
            name: "".to_string(),
            version: "1.0.0".to_string(),
            description: "Test".to_string(),
            steps: vec![],
        };
        
        assert!(validate_workflow(&workflow).is_err());
    }
    
    #[test]
    fn test_invalid_name() {
        let workflow = WorkflowDefinition {
            name: "invalid name".to_string(),
            version: "1.0.0".to_string(),
            description: "Test".to_string(),
            steps: vec![
                WorkflowStep {
                    name: "step1".to_string(),
                    tool: "echo".to_string(),
                    parameters: serde_json::json!({}),
                }
            ],
        };
        
        assert!(validate_workflow(&workflow).is_err());
    }
    
    #[test]
    fn test_empty_steps() {
        let workflow = WorkflowDefinition {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            description: "Test".to_string(),
            steps: vec![],
        };
        
        assert!(validate_workflow(&workflow).is_err());
    }
    
    #[test]
    fn test_self_reference_cycle() {
        let workflow = WorkflowDefinition {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            description: "Test".to_string(),
            steps: vec![
                WorkflowStep {
                    name: "step1".to_string(),
                    tool: "step1".to_string(),
                    parameters: serde_json::json!({}),
                }
            ],
        };
        
        assert!(validate_workflow(&workflow).is_err());
        assert!(validate_workflow(&workflow).unwrap_err().contains("cycle"));
    }
}

fn main() {
    println!("Run with: cargo test --test test_validation_unit");
}
