//! Workflow tool implementation

use eyre::Result;

#[derive(Debug, Clone)]
pub struct WorkflowTool {
    pub name: String,
    pub description: String,
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
}
