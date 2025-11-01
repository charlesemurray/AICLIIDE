use crate::cli::skills::{EnhancedSkillInfo, SkillType, SkillError};
use serde_json::Value;

pub struct SkillValidator;

impl SkillValidator {
    pub fn validate_skill_json(content: &str) -> Result<EnhancedSkillInfo, SkillError> {
        // First, validate it's valid JSON
        let json: Value = serde_json::from_str(content)
            .map_err(|e| SkillError::InvalidInput(format!("Invalid JSON: {}", e)))?;

        // Validate required fields
        Self::validate_required_fields(&json)?;
        
        // Validate skill type specific fields
        Self::validate_skill_type(&json)?;

        // If validation passes, deserialize to EnhancedSkillInfo
        serde_json::from_str::<EnhancedSkillInfo>(content)
            .map_err(|e| SkillError::InvalidInput(format!("Invalid skill configuration: {}", e)))
    }

    fn validate_required_fields(json: &Value) -> Result<(), SkillError> {
        let obj = json.as_object()
            .ok_or_else(|| SkillError::InvalidInput("Skill configuration must be an object".to_string()))?;

        // Check required fields
        let required_fields = ["name", "description", "version", "type"];
        for field in &required_fields {
            if !obj.contains_key(*field) {
                return Err(SkillError::InvalidInput(format!("Missing required field: {}", field)));
            }
        }

        // Validate name format
        if let Some(name) = obj.get("name").and_then(|v| v.as_str()) {
            if name.is_empty() || !name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
                return Err(SkillError::InvalidInput(
                    "Skill name must be non-empty and contain only alphanumeric characters, hyphens, and underscores".to_string()
                ));
            }
        }

        // Validate version format (basic semver check)
        if let Some(version) = obj.get("version").and_then(|v| v.as_str()) {
            if !version.split('.').all(|part| part.parse::<u32>().is_ok()) {
                return Err(SkillError::InvalidInput(
                    "Version must be in semver format (e.g., 1.0.0)".to_string()
                ));
            }
        }

        Ok(())
    }

    fn validate_skill_type(json: &Value) -> Result<(), SkillError> {
        let obj = json.as_object().unwrap();
        let skill_type = obj.get("type").and_then(|v| v.as_str())
            .ok_or_else(|| SkillError::InvalidInput("Missing or invalid type field".to_string()))?;

        match skill_type {
            "code_inline" | "code_session" => {
                if !obj.contains_key("command") {
                    return Err(SkillError::InvalidInput("Code skills require 'command' field".to_string()));
                }
            }
            "conversation" => {
                if !obj.contains_key("prompt_template") {
                    return Err(SkillError::InvalidInput("Conversation skills require 'prompt_template' field".to_string()));
                }
            }
            "prompt_inline" => {
                if !obj.contains_key("prompt") {
                    return Err(SkillError::InvalidInput("Prompt inline skills require 'prompt' field".to_string()));
                }
            }
            _ => {
                return Err(SkillError::InvalidInput(format!("Unknown skill type: {}", skill_type)));
            }
        }

        Ok(())
    }

    pub fn validate_parameters(skill_type: &SkillType, params: &Value) -> Result<(), SkillError> {
        match skill_type {
            SkillType::PromptInline { parameters: Some(param_defs), .. } => {
                let param_obj = params.as_object()
                    .ok_or_else(|| SkillError::InvalidInput("Parameters must be an object".to_string()))?;

                // Check required parameters
                for param_def in param_defs {
                    if param_def.required && !param_obj.contains_key(&param_def.name) {
                        return Err(SkillError::InvalidInput(format!("Missing required parameter: {}", param_def.name)));
                    }
                }
            }
            _ => {} // Other skill types don't have parameter validation yet
        }

        Ok(())
    }
}
