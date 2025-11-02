//! Workflow tool implementation

use eyre::Result;
use serde::{
    Deserialize,
    Serialize,
};

use crate::cli::agent::{
    Agent,
    PermissionEvalResult,
};
use crate::os::Os;

#[derive(Debug, Clone)]
pub struct WorkflowTool {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub name: String,
    pub tool: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub name: String,
    pub version: String,
    pub description: String,
    pub steps: Vec<WorkflowStep>,
}

impl WorkflowTool {
    pub fn new(name: String, description: String) -> Self {
        Self { name, description }
    }

    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(eyre::eyre!("Workflow name cannot be empty"));
        }
        Ok(())
    }

    pub fn eval_perm(&self, _os: &Os, _agent: &Agent) -> PermissionEvalResult {
        PermissionEvalResult::Allow
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_tool_creation() {
        let workflow = WorkflowTool::new("test-workflow".to_string(), "A test workflow".to_string());
        assert_eq!(workflow.name, "test-workflow");
        assert_eq!(workflow.description, "A test workflow");
    }

    #[test]
    fn test_workflow_tool_clone() {
        let workflow = WorkflowTool::new("original".to_string(), "Original workflow".to_string());
        let cloned = workflow.clone();
        assert_eq!(cloned.name, workflow.name);
        assert_eq!(cloned.description, workflow.description);
    }

    #[test]
    fn test_workflow_tool_validate_success() {
        let workflow = WorkflowTool::new("valid-workflow".to_string(), "Description".to_string());
        assert!(workflow.validate().is_ok());
    }

    #[test]
    fn test_workflow_tool_validate_empty_name() {
        let workflow = WorkflowTool::new("".to_string(), "Description".to_string());
        let result = workflow.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }

    #[tokio::test]
    async fn test_workflow_tool_eval_perm() {
        use crate::cli::agent::{
            Agent,
            PermissionEvalResult,
        };
        use crate::os::Os;

        let workflow = WorkflowTool::new("test-workflow".to_string(), "Test".to_string());
        let os = Os::new().await.unwrap();
        let agent = Agent::default();

        let result = workflow.eval_perm(&os, &agent);
        assert_eq!(result, PermissionEvalResult::Allow);
    }

    #[test]
    fn test_workflow_definition_deserialize() {
        let json = r#"{
            "name": "test-workflow",
            "version": "1.0.0",
            "description": "A test workflow"
        }"#;

        let definition: WorkflowDefinition = serde_json::from_str(json).unwrap();
        assert_eq!(definition.name, "test-workflow");
        assert_eq!(definition.version, "1.0.0");
        assert_eq!(definition.description, "A test workflow");
    }

    #[test]
    fn test_workflow_definition_with_steps() {
        let json = r#"{
            "name": "build-workflow",
            "version": "1.0.0",
            "description": "A build workflow",
            "steps": [
                {
                    "name": "compile",
                    "tool": "execute_bash",
                    "parameters": {"command": "cargo build"}
                }
            ]
        }"#;

        let definition: WorkflowDefinition = serde_json::from_str(json).unwrap();
        assert_eq!(definition.steps.len(), 1);
        assert_eq!(definition.steps[0].name, "compile");
        assert_eq!(definition.steps[0].tool, "execute_bash");
    }
}
