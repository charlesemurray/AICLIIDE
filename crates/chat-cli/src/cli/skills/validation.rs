use crate::cli::skills::SkillError;
use crate::cli::skills::types::JsonSkill;
use serde_json::Value;

pub struct SkillValidator;

impl SkillValidator {
    pub fn validate_skill_json(content: &str) -> Result<Value, SkillError> {
        // First, validate it's valid JSON
        let json: Value = serde_json::from_str(content)
            .map_err(|e| SkillError::InvalidInput(format!("Invalid JSON: {}", e)))?;

        // Validate required fields
        Self::validate_required_fields(&json)?;
        
        // Validate skill type specific fields
        Self::validate_skill_type(&json)?;

        // Try to parse as JsonSkill to ensure all fields are valid
        JsonSkill::from_json(json.clone())
            .map_err(|e| SkillError::InvalidConfiguration(format!("Invalid skill configuration: {}", e)))?;

        Ok(json)
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
                return Err(SkillError::InvalidInput(format!("Unknown skill type: {}. Valid types: code_inline, code_session, conversation, prompt_inline", skill_type)));
            }
        }

        Ok(())
    }

    pub fn validate_parameters(params: &Value, param_defs: &[crate::cli::skills::types::Parameter]) -> Result<(), SkillError> {
        let param_obj = params.as_object()
            .ok_or_else(|| SkillError::InvalidInput("Parameters must be an object".to_string()))?;

        // Check required parameters
        for param_def in param_defs {
            if param_def.required.unwrap_or(false) && !param_obj.contains_key(&param_def.name) {
                return Err(SkillError::InvalidInput(format!("Missing required parameter: {}", param_def.name)));
            }
            
            // Validate parameter types
            if let Some(value) = param_obj.get(&param_def.name) {
                Self::validate_parameter_type(value, param_def)?;
            }
        }

        Ok(())
    }
    
    fn validate_parameter_type(value: &Value, param_def: &crate::cli::skills::types::Parameter) -> Result<(), SkillError> {
        match param_def.param_type.as_str() {
            "string" => {
                if !value.is_string() {
                    return Err(SkillError::InvalidInput(format!("Parameter '{}' must be a string", param_def.name)));
                }
                
                if let Some(str_val) = value.as_str() {
                    // Security: Check for malicious patterns
                    Self::validate_input_security(str_val, &param_def.name)?;
                    
                    // Validate pattern if provided
                    if let Some(pattern) = &param_def.pattern {
                        let regex = regex::Regex::new(pattern)
                            .map_err(|_| SkillError::InvalidInput(format!("Invalid regex pattern for parameter '{}'", param_def.name)))?;
                        if !regex.is_match(str_val) {
                            return Err(SkillError::InvalidInput(format!("Parameter '{}' does not match pattern '{}'", param_def.name, pattern)));
                        }
                    }
                }
            }
            "number" => {
                if !value.is_number() {
                    return Err(SkillError::InvalidInput(format!("Parameter '{}' must be a number", param_def.name)));
                }
            }
            "enum" => {
                if let Some(str_val) = value.as_str() {
                    if let Some(allowed_values) = &param_def.values {
                        if !allowed_values.contains(&str_val.to_string()) {
                            return Err(SkillError::InvalidInput(format!("Parameter '{}' must be one of: {:?}", param_def.name, allowed_values)));
                        }
                    }
                } else {
                    return Err(SkillError::InvalidInput(format!("Parameter '{}' must be a string for enum type", param_def.name)));
                }
            }
            _ => {
                return Err(SkillError::InvalidInput(format!("Unknown parameter type: {}", param_def.param_type)));
            }
        }
        
        Ok(())
    }
    
    fn validate_input_security(input: &str, param_name: &str) -> Result<(), SkillError> {
        // Check for command injection patterns
        let dangerous_patterns = [
            ";", "|", "&", "$", "`", "$(", "rm -rf", "sudo", "chmod", "chown",
            "../", "..\\", "/etc/", "C:\\", "powershell", "cmd.exe", "bash -c",
            "sh -c", "eval", "exec", "system", "popen", "shell_exec"
        ];
        
        let input_lower = input.to_lowercase();
        for pattern in &dangerous_patterns {
            if input_lower.contains(pattern) {
                return Err(SkillError::InvalidInput(format!(
                    "Parameter '{}' contains potentially dangerous pattern: '{}'", 
                    param_name, pattern
                )));
            }
        }
        
        Ok(())
    }
}
