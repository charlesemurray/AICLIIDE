//! Skill tool implementation

use std::collections::HashMap;
use std::path::PathBuf;

use eyre::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::cli::agent::{Agent, PermissionEvalResult};
use crate::os::Os;

#[derive(Debug, Clone)]
pub struct SkillTool {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum SkillImplementation {
    Script { path: String },
    Command { command: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDefinition {
    pub name: String,
    pub description: String,
    pub skill_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub implementation: Option<SkillImplementation>,
}

impl SkillTool {
    pub fn new(name: String, description: String) -> Self {
        Self { name, description }
    }

    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(eyre::eyre!("Skill name cannot be empty"));
        }
        Ok(())
    }

    pub fn eval_perm(&self, _os: &Os, _agent: &Agent) -> PermissionEvalResult {
        PermissionEvalResult::Allow
    }

    pub fn invoke(&self, _params: HashMap<String, Value>) -> Result<String> {
        Ok("not implemented".to_string())
    }

    pub fn get_script_path(&self, definition: &SkillDefinition) -> Result<PathBuf> {
        match &definition.implementation {
            Some(SkillImplementation::Script { path }) => {
                let path_buf = PathBuf::from(path);
                if path_buf.exists() {
                    Ok(path_buf)
                } else {
                    Err(eyre::eyre!("Script path does not exist: {}", path))
                }
            },
            _ => Err(eyre::eyre!("Skill does not have a script implementation")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_tool_creation() {
        let skill = SkillTool::new("test-skill".to_string(), "A test skill".to_string());
        assert_eq!(skill.name, "test-skill");
        assert_eq!(skill.description, "A test skill");
    }

    #[test]
    fn test_skill_tool_clone() {
        let skill = SkillTool::new("original".to_string(), "Original skill".to_string());
        let cloned = skill.clone();
        assert_eq!(cloned.name, skill.name);
        assert_eq!(cloned.description, skill.description);
    }

    #[test]
    fn test_skill_tool_validate_success() {
        let skill = SkillTool::new("valid-skill".to_string(), "Description".to_string());
        assert!(skill.validate().is_ok());
    }

    #[test]
    fn test_skill_tool_validate_empty_name() {
        let skill = SkillTool::new("".to_string(), "Description".to_string());
        let result = skill.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }

    #[tokio::test]
    async fn test_skill_tool_eval_perm() {
        use crate::cli::agent::{Agent, PermissionEvalResult};
        use crate::os::Os;

        let skill = SkillTool::new("test-skill".to_string(), "Test".to_string());
        let os = Os::new().await.unwrap();
        let agent = Agent::default();

        let result = skill.eval_perm(&os, &agent);
        assert_eq!(result, PermissionEvalResult::Allow);
    }

    #[test]
    fn test_skill_definition_deserialize() {
        let json = r#"{
            "name": "test-skill",
            "description": "A test skill",
            "skill_type": "code_inline"
        }"#;

        let definition: SkillDefinition = serde_json::from_str(json).unwrap();
        assert_eq!(definition.name, "test-skill");
        assert_eq!(definition.description, "A test skill");
        assert_eq!(definition.skill_type, "code_inline");
    }

    #[test]
    fn test_skill_definition_with_parameters() {
        let json = r#"{
            "name": "calculator",
            "description": "A calculator skill",
            "skill_type": "code_inline",
            "parameters": {
                "type": "object",
                "properties": {
                    "a": {"type": "number"},
                    "b": {"type": "number"}
                }
            }
        }"#;

        let definition: SkillDefinition = serde_json::from_str(json).unwrap();
        assert_eq!(definition.name, "calculator");
        assert!(definition.parameters.is_some());
        let params = definition.parameters.unwrap();
        assert!(params.get("type").is_some());
    }

    #[test]
    fn test_skill_definition_script_implementation() {
        let json = r#"{
            "name": "test-skill",
            "description": "A test skill",
            "skill_type": "code_inline",
            "implementation": {
                "type": "script",
                "path": "./scripts/test.sh"
            }
        }"#;

        let definition: SkillDefinition = serde_json::from_str(json).unwrap();
        assert!(definition.implementation.is_some());
        match definition.implementation.unwrap() {
            SkillImplementation::Script { path } => {
                assert_eq!(path, "./scripts/test.sh");
            },
            _ => panic!("Expected Script implementation"),
        }
    }

    #[test]
    fn test_skill_definition_command_implementation() {
        let json = r#"{
            "name": "test-skill",
            "description": "A test skill",
            "skill_type": "code_inline",
            "implementation": {
                "type": "command",
                "command": "echo 'Hello'"
            }
        }"#;

        let definition: SkillDefinition = serde_json::from_str(json).unwrap();
        assert!(definition.implementation.is_some());
        match definition.implementation.unwrap() {
            SkillImplementation::Command { command } => {
                assert_eq!(command, "echo 'Hello'");
            },
            _ => panic!("Expected Command implementation"),
        }
    }
}
