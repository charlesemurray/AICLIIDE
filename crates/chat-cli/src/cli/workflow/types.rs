use serde::{
    Deserialize,
    Serialize,
};

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
}
