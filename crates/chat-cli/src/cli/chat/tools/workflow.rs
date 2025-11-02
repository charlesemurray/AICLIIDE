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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Value>,
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

    pub fn invoke_with_definition(
        &self,
        definition: &WorkflowDefinition,
        _params: std::collections::HashMap<String, serde_json::Value>,
    ) -> Result<String> {
        for step in &definition.steps {
            // Validate tool exists
            let known_tools = ["echo", "calculator"];
            if !known_tools.contains(&step.tool.as_str()) {
                return Err(eyre::eyre!("Unknown tool '{}' in step '{}'", step.tool, step.name));
            }
        }
        Ok("Success".to_string())
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
            "description": "A test workflow",
            "steps": []
        }"#;

        let definition: WorkflowDefinition = serde_json::from_str(json).unwrap();
        assert_eq!(definition.name, "test-workflow");
        assert_eq!(definition.version, "1.0.0");
        assert_eq!(definition.description, "A test workflow");
        assert_eq!(definition.steps.len(), 0);
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

    #[test]
    fn test_workflow_definition_with_context() {
        let json = r#"{
            "name": "deploy-workflow",
            "version": "1.0.0",
            "description": "A deployment workflow",
            "steps": [],
            "context": {
                "environment": "production",
                "region": "us-east-1"
            }
        }"#;

        let definition: WorkflowDefinition = serde_json::from_str(json).unwrap();
        assert!(definition.context.is_some());
        let context = definition.context.unwrap();
        assert_eq!(context.get("environment").unwrap(), "production");
        assert_eq!(context.get("region").unwrap(), "us-east-1");
    }

    #[test]
    fn test_workflow_stops_on_error() {
        let definition = WorkflowDefinition {
            name: "test".to_string(),
            version: "1.0".to_string(),
            description: "Test".to_string(),
            steps: vec![
                WorkflowStep {
                    name: "step1".to_string(),
                    tool: "echo".to_string(),
                    parameters: serde_json::json!({"msg": "first"}),
                },
                WorkflowStep {
                    name: "failing_step".to_string(),
                    tool: "nonexistent_tool".to_string(),
                    parameters: serde_json::json!({}),
                },
                WorkflowStep {
                    name: "step3".to_string(),
                    tool: "echo".to_string(),
                    parameters: serde_json::json!({"msg": "third"}),
                },
            ],
            context: None,
        };

        let workflow = WorkflowTool::new("test".to_string(), "Test".to_string());
        let params = HashMap::new();

        let result = workflow.invoke_with_definition(&definition, params);

        assert!(result.is_err());
        let error = result.unwrap_err();
        let error_msg = error.to_string();
        assert!(error_msg.contains("failing_step") || error_msg.contains("nonexistent"));
    }
}
