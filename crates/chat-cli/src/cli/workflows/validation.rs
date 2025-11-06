use eyre::{Result, eyre};
use petgraph::graph::DiGraph;
use petgraph::algo::is_cyclic_directed;
use std::collections::HashMap;

use crate::cli::chat::tools::workflow::WorkflowDefinition;

/// Detect cycles in workflow step dependencies
fn detect_cycles(workflow: &WorkflowDefinition) -> Result<()> {
    let mut graph = DiGraph::<&str, ()>::new();
    let mut node_map = HashMap::new();
    
    // Create nodes for each step
    for step in &workflow.steps {
        let node = graph.add_node(step.name.as_str());
        node_map.insert(step.name.as_str(), node);
    }
    
    // Add edges for workflow tool invocations
    for step in &workflow.steps {
        // If this step invokes another workflow, it creates a dependency
        // For now, we check if the tool name matches any step name (self-reference)
        if let Some(&target_node) = node_map.get(step.tool.as_str()) {
            if let Some(&source_node) = node_map.get(step.name.as_str()) {
                graph.add_edge(source_node, target_node, ());
            }
        }
    }
    
    if is_cyclic_directed(&graph) {
        return Err(eyre!("Workflow contains cycles - steps cannot reference each other"));
    }
    
    Ok(())
}

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

    // Check for cycles
    detect_cycles(workflow)?;

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
