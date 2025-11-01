#[cfg(test)]
mod validation_tests {
    use crate::cli::skills::{validation::SkillValidator, SkillType, SkillParameter};
    use serde_json::json;

    #[test]
    fn test_valid_skill_configurations() {
        // Test valid code_inline skill
        let valid_code_inline = json!({
            "name": "test-skill",
            "description": "Test skill",
            "version": "1.0.0",
            "type": "code_inline",
            "command": "echo",
            "args": ["hello"]
        });
        
        let result = SkillValidator::validate_skill_json(&valid_code_inline.to_string());
        assert!(result.is_ok());
        let skill_info = result.unwrap();
        assert_eq!(skill_info.name, "test-skill");

        // Test valid prompt_inline skill
        let valid_prompt_inline = json!({
            "name": "prompt-skill",
            "description": "Prompt skill",
            "version": "2.1.0",
            "type": "prompt_inline",
            "prompt": "Hello {name}!",
            "parameters": [
                {
                    "name": "name",
                    "description": "User name",
                    "required": true
                }
            ]
        });
        
        let result = SkillValidator::validate_skill_json(&valid_prompt_inline.to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_skill_configurations() {
        // Test missing required fields
        let missing_name = json!({
            "description": "Test skill",
            "version": "1.0.0",
            "type": "code_inline",
            "command": "echo"
        });
        
        let result = SkillValidator::validate_skill_json(&missing_name.to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing required field: name"));

        // Test invalid name format
        let invalid_name = json!({
            "name": "invalid name with spaces!",
            "description": "Test skill",
            "version": "1.0.0",
            "type": "code_inline",
            "command": "echo"
        });
        
        let result = SkillValidator::validate_skill_json(&invalid_name.to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("alphanumeric characters"));

        // Test invalid version format
        let invalid_version = json!({
            "name": "test-skill",
            "description": "Test skill",
            "version": "not.a.version",
            "type": "code_inline",
            "command": "echo"
        });
        
        let result = SkillValidator::validate_skill_json(&invalid_version.to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("semver format"));
    }

    #[test]
    fn test_skill_type_specific_validation() {
        // Test code skill without command
        let code_without_command = json!({
            "name": "test-skill",
            "description": "Test skill",
            "version": "1.0.0",
            "type": "code_inline"
        });
        
        let result = SkillValidator::validate_skill_json(&code_without_command.to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("require 'command' field"));

        // Test conversation skill without prompt_template
        let conversation_without_template = json!({
            "name": "test-skill",
            "description": "Test skill",
            "version": "1.0.0",
            "type": "conversation"
        });
        
        let result = SkillValidator::validate_skill_json(&conversation_without_template.to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("require 'prompt_template' field"));

        // Test prompt_inline skill without prompt
        let prompt_without_prompt = json!({
            "name": "test-skill",
            "description": "Test skill",
            "version": "1.0.0",
            "type": "prompt_inline"
        });
        
        let result = SkillValidator::validate_skill_json(&prompt_without_prompt.to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("require 'prompt' field"));
    }

    #[test]
    fn test_parameter_validation() {
        let skill_type = SkillType::PromptInline {
            prompt: "Hello {name}!".to_string(),
            parameters: Some(vec![
                SkillParameter {
                    name: "name".to_string(),
                    description: "User name".to_string(),
                    required: true,
                    default: None,
                },
                SkillParameter {
                    name: "greeting".to_string(),
                    description: "Greeting type".to_string(),
                    required: false,
                    default: Some("Hello".to_string()),
                },
            ]),
        };

        // Test valid parameters
        let valid_params = json!({
            "name": "Alice",
            "greeting": "Hi"
        });
        
        let result = SkillValidator::validate_parameters(&skill_type, &valid_params);
        assert!(result.is_ok());

        // Test missing required parameter
        let missing_required = json!({
            "greeting": "Hi"
        });
        
        let result = SkillValidator::validate_parameters(&skill_type, &missing_required);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing required parameter: name"));

        // Test with only required parameter (optional missing is OK)
        let only_required = json!({
            "name": "Bob"
        });
        
        let result = SkillValidator::validate_parameters(&skill_type, &only_required);
        assert!(result.is_ok());
    }

    #[test]
    fn test_unknown_skill_type() {
        let unknown_type = json!({
            "name": "test-skill",
            "description": "Test skill",
            "version": "1.0.0",
            "type": "unknown_type",
            "command": "echo"
        });
        
        let result = SkillValidator::validate_skill_json(&unknown_type.to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown skill type: unknown_type"));
    }

    #[test]
    fn test_invalid_json() {
        let invalid_json = "{ invalid json }";
        
        let result = SkillValidator::validate_skill_json(invalid_json);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid JSON"));
    }

    #[test]
    fn test_non_object_json() {
        let array_json = "[1, 2, 3]";
        
        let result = SkillValidator::validate_skill_json(array_json);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must be an object"));
    }
}
