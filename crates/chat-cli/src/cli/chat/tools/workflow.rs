//! Workflow tool implementation

#[derive(Debug, Clone)]
pub struct WorkflowTool {
    pub name: String,
    pub description: String,
}

impl WorkflowTool {
    pub fn new(name: String, description: String) -> Self {
        Self { name, description }
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
}
