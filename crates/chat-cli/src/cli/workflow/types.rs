use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::cli::chat::tools::{InputSchema, ToolOrigin, ToolSpec};
use crate::cli::skills::toolspec_conversion::{ConversionError, ToToolSpec};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub name: String,
    pub description: String,
    pub version: String,
    pub steps: Vec<WorkflowStep>,
    pub inputs: Vec<WorkflowInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub id: String,
    #[serde(flatten)]
    pub step_type: StepType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StepType {
    #[serde(rename = "skill")]
    Skill { name: String, inputs: serde_json::Value },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowInput {
    pub name: String,
    #[serde(rename = "type")]
    pub input_type: String,
    pub required: bool,
}

impl ToToolSpec for Workflow {
    fn to_toolspec(&self) -> Result<ToolSpec, ConversionError> {
        let input_schema = self.build_input_schema()?;

        Ok(ToolSpec {
            name: self.name.clone(),
            description: self.description.clone(),
            input_schema: InputSchema(input_schema),
            tool_origin: ToolOrigin::Workflow(self.name.clone()),
        })
    }
}

impl Workflow {
    fn build_input_schema(&self) -> Result<serde_json::Value, ConversionError> {
        let mut properties = serde_json::Map::new();
        let mut required = Vec::new();

        for input in &self.inputs {
            properties.insert(input.name.clone(), json!({"type": input.input_type}));

            if input.required {
                required.push(input.name.clone());
            }
        }

        Ok(json!({
            "type": "object",
            "properties": properties,
            "required": required
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_serialization() {
        let workflow = Workflow {
            name: "test".to_string(),
            description: "Test workflow".to_string(),
            version: "1.0.0".to_string(),
            steps: vec![],
            inputs: vec![],
        };

        let json = serde_json::to_string(&workflow).unwrap();
        let deserialized: Workflow = serde_json::from_str(&json).unwrap();
        assert_eq!(workflow.name, deserialized.name);
        assert_eq!(workflow.version, deserialized.version);
    }

    #[test]
    fn test_step_type_serialization() {
        let step = WorkflowStep {
            id: "step1".to_string(),
            step_type: StepType::Skill {
                name: "test-skill".to_string(),
                inputs: serde_json::json!({"param": "value"}),
            },
        };

        let json = serde_json::to_string(&step).unwrap();
        assert!(json.contains("\"type\":\"skill\""));
        assert!(json.contains("test-skill"));

        let deserialized: WorkflowStep = serde_json::from_str(&json).unwrap();
        assert_eq!(step.id, deserialized.id);
    }

    #[test]
    fn test_workflow_input_serialization() {
        let input = WorkflowInput {
            name: "input1".to_string(),
            input_type: "string".to_string(),
            required: true,
        };

        let json = serde_json::to_string(&input).unwrap();
        let deserialized: WorkflowInput = serde_json::from_str(&json).unwrap();
        assert_eq!(input.name, deserialized.name);
        assert_eq!(input.input_type, deserialized.input_type);
        assert_eq!(input.required, deserialized.required);
    }

    #[test]
    fn test_workflow_to_toolspec() {
        let workflow = Workflow {
            name: "test-workflow".to_string(),
            description: "Test workflow".to_string(),
            version: "1.0.0".to_string(),
            steps: vec![],
            inputs: vec![WorkflowInput {
                name: "input1".to_string(),
                input_type: "string".to_string(),
                required: true,
            }],
        };

        let toolspec = workflow.to_toolspec().unwrap();
        assert_eq!(toolspec.name, "test-workflow");
        assert_eq!(toolspec.description, "Test workflow");
        assert!(matches!(
            toolspec.tool_origin,
            ToolOrigin::Workflow(ref name) if name == "test-workflow"
        ));
    }

    #[test]
    fn test_workflow_input_schema() {
        let workflow = Workflow {
            name: "schema-test".to_string(),
            description: "Test".to_string(),
            version: "1.0.0".to_string(),
            steps: vec![],
            inputs: vec![
                WorkflowInput {
                    name: "required_input".to_string(),
                    input_type: "string".to_string(),
                    required: true,
                },
                WorkflowInput {
                    name: "optional_input".to_string(),
                    input_type: "number".to_string(),
                    required: false,
                },
            ],
        };

        let toolspec = workflow.to_toolspec().unwrap();
        let schema = toolspec.input_schema.0;

        // Check properties exist
        assert!(schema["properties"]["required_input"].is_object());
        assert!(schema["properties"]["optional_input"].is_object());

        // Check types
        assert_eq!(schema["properties"]["required_input"]["type"], "string");
        assert_eq!(schema["properties"]["optional_input"]["type"], "number");

        // Check required array
        let required = schema["required"].as_array().unwrap();
        assert_eq!(required.len(), 1);
        assert!(required.contains(&json!("required_input")));
        assert!(!required.contains(&json!("optional_input")));
    }
}
