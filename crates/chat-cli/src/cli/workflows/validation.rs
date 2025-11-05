use eyre::{Result, eyre};

use crate::cli::chat::tools::workflow::WorkflowDefinition;

/// Validate a workflow definition
pub fn validate_workflow(workflow: &WorkflowDefinition) -> Result<()> {
    // Validate name
    if workflow.name.is_empty() {
        return Err(eyre!("Workflow name cannot be empty"));
    }

    if workflow.name.len() > 50 {
        return Err(eyre!("Workflow name must be 50 characters or less"));
    }

    if !workflow.name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err(eyre!("Workflow name can only contain alphanumeric characters, hyphens, and underscores"));
    }

    // Validate version
    if workflow.version.is_empty() {
        return Err(eyre!("Workflow version cannot be empty"));
    }

    // Validate description
    if workflow.description.is_empty() {
        return Err(eyre!("Workflow description cannot be empty"));
    }

    // Validate steps
    if workflow.steps.is_empty() {
        return Err(eyre!("Workflow must have at least one step"));
    }

    // Validate each step
    for (i, step) in workflow.steps.iter().enumerate() {
        if step.name.is_empty() {
            return Err(eyre!("Step {} name cannot be empty", i + 1));
        }

        if step.tool.is_empty() {
            return Err(eyre!("Step {} tool cannot be empty", i + 1));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::chat::tools::workflow::WorkflowStep;

    #[test]
    fn test_validate_valid_workflow() {
        let workflow = WorkflowDefinition {
            name: "test-workflow".to_string(),
            version: "1.0.0".to_string(),
            description: "Test workflow".to_string(),
            steps: vec![
                WorkflowStep {
                    name: "step1".to_string(),
                    tool: "fs_read".to_string(),
                    parameters: serde_json::json!({}),
                }
            ],
            context: None,
        };

        assert!(validate_workflow(&workflow).is_ok());
    }

    #[test]
    fn test_validate_empty_name() {
        let workflow = WorkflowDefinition {
            name: "".to_string(),
            version: "1.0.0".to_string(),
            description: "Test".to_string(),
            steps: vec![],
            context: None,
        };

        assert!(validate_workflow(&workflow).is_err());
    }

    #[test]
    fn test_validate_no_steps() {
        let workflow = WorkflowDefinition {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            description: "Test".to_string(),
            steps: vec![],
            context: None,
        };

        assert!(validate_workflow(&workflow).is_err());
    }
}
