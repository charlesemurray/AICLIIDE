use eyre::Result;

use super::types::{
    StepType,
    Workflow,
};
use crate::cli::skills::SkillRegistry;

pub struct WorkflowExecutor {
    skill_registry: SkillRegistry,
}

impl WorkflowExecutor {
    pub fn new(skill_registry: SkillRegistry) -> Self {
        Self { skill_registry }
    }

    pub async fn execute(&self, workflow: &Workflow, inputs: serde_json::Value) -> Result<String> {
        let mut outputs = Vec::new();

        for step in &workflow.steps {
            let output = self.execute_step(step, &inputs).await?;
            outputs.push(output);
        }

        Ok(outputs.join("\n"))
    }

    async fn execute_step(&self, step: &super::types::WorkflowStep, inputs: &serde_json::Value) -> Result<String> {
        match &step.step_type {
            StepType::Skill {
                name,
                inputs: step_inputs,
            } => {
                let skill = self
                    .skill_registry
                    .get(name)
                    .ok_or_else(|| eyre::eyre!("Skill not found: {}", name))?;

                let merged_inputs = self.merge_inputs(inputs, step_inputs);
                let result = skill
                    .execute(merged_inputs)
                    .await
                    .map_err(|e| eyre::eyre!("Skill execution failed: {}", e))?;

                Ok(result.output)
            },
        }
    }

    fn merge_inputs(&self, workflow_inputs: &serde_json::Value, step_inputs: &serde_json::Value) -> serde_json::Value {
        // Simple merge: step inputs override workflow inputs
        let mut merged = workflow_inputs.clone();
        if let (Some(wf_obj), Some(step_obj)) = (merged.as_object_mut(), step_inputs.as_object()) {
            for (k, v) in step_obj {
                wf_obj.insert(k.clone(), v.clone());
            }
        }
        merged
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::skills::SkillRegistry;
    use crate::cli::workflow::types::{
        StepType,
        Workflow,
        WorkflowInput,
        WorkflowStep,
    };

    #[tokio::test]
    async fn test_workflow_execution() {
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
                    inputs: serde_json::json!({
                        "a": 5.0,
                        "b": 3.0,
                        "op": "add"
                    }),
                },
            }],
            inputs: vec![],
        };

        let result = executor.execute(&workflow, serde_json::json!({})).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("8"));
    }

    #[tokio::test]
    async fn test_workflow_with_multiple_steps() {
        let registry = SkillRegistry::with_builtins();
        let executor = WorkflowExecutor::new(registry);

        let workflow = Workflow {
            name: "multi-step".to_string(),
            description: "Test".to_string(),
            version: "1.0.0".to_string(),
            steps: vec![
                WorkflowStep {
                    id: "step1".to_string(),
                    step_type: StepType::Skill {
                        name: "calculator".to_string(),
                        inputs: serde_json::json!({"a": 10.0, "b": 5.0, "op": "add"}),
                    },
                },
                WorkflowStep {
                    id: "step2".to_string(),
                    step_type: StepType::Skill {
                        name: "calculator".to_string(),
                        inputs: serde_json::json!({"a": 20.0, "b": 10.0, "op": "multiply"}),
                    },
                },
            ],
            inputs: vec![],
        };

        let result = executor.execute(&workflow, serde_json::json!({})).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("15"));
        assert!(output.contains("200"));
    }

    #[test]
    fn test_input_merging() {
        let registry = SkillRegistry::with_builtins();
        let executor = WorkflowExecutor::new(registry);

        let workflow_inputs = serde_json::json!({"a": 1, "b": 2});
        let step_inputs = serde_json::json!({"b": 3, "c": 4});

        let merged = executor.merge_inputs(&workflow_inputs, &step_inputs);

        assert_eq!(merged["a"], 1);
        assert_eq!(merged["b"], 3); // Step input overrides workflow input
        assert_eq!(merged["c"], 4);
    }

    #[tokio::test]
    async fn test_workflow_skill_not_found() {
        let registry = SkillRegistry::new();
        let executor = WorkflowExecutor::new(registry);

        let workflow = Workflow {
            name: "test".to_string(),
            description: "Test".to_string(),
            version: "1.0.0".to_string(),
            steps: vec![WorkflowStep {
                id: "step1".to_string(),
                step_type: StepType::Skill {
                    name: "nonexistent".to_string(),
                    inputs: serde_json::json!({}),
                },
            }],
            inputs: vec![],
        };

        let result = executor.execute(&workflow, serde_json::json!({})).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Skill not found"));
    }
}
