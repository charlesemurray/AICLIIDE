use std::io::Write;

use eyre::Result;

use super::{
    InvokeOutput,
    OutputKind,
};
use crate::cli::workflow::{
    Workflow,
    WorkflowExecutor,
};

#[derive(Debug, Clone)]
pub struct WorkflowTool {
    pub workflow: Workflow,
    pub params: serde_json::Value,
}

impl WorkflowTool {
    pub fn new(workflow: Workflow, params: serde_json::Value) -> Self {
        Self { workflow, params }
    }

    pub async fn invoke(&self, executor: &WorkflowExecutor, stdout: &mut impl Write) -> Result<InvokeOutput> {
        let output = executor.execute(&self.workflow, self.params.clone()).await?;

        writeln!(stdout, "{}", output)?;

        Ok(InvokeOutput {
            output: OutputKind::Text(output),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::skills::SkillRegistry;
    use crate::cli::workflow::types::{
        StepType,
        WorkflowStep,
    };
    use crate::cli::workflow::{
        Workflow,
        WorkflowExecutor,
    };

    #[tokio::test]
    async fn test_workflow_tool_invocation() {
        let registry = SkillRegistry::with_builtins();
        let executor = WorkflowExecutor::new(registry);

        let workflow = Workflow {
            name: "test-workflow".to_string(),
            description: "Test".to_string(),
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

        let tool = WorkflowTool::new(workflow, serde_json::json!({}));
        let mut output = Vec::new();

        let result = tool.invoke(&executor, &mut output).await;
        assert!(result.is_ok());

        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("8"));
    }
}
