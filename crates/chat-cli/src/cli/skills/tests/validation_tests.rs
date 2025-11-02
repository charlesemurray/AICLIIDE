#[cfg(test)]
mod validation_tests {
    use serde_json::{Value, json};

    use crate::cli::skills::types::Parameter;
    use crate::cli::skills::validation::SkillValidator;

    #[test]
    fn test_skill_json_validation() {
        let valid_skill = json!({
            "name": "test-skill",
            "description": "A test skill",
            "version": "1.0.0",
            "type": "code_inline",
            "command": "echo",
            "args": ["Hello World"],
            "timeout": 30
        });

        let skill_str = serde_json::to_string(&valid_skill).unwrap();
        let result = SkillValidator::validate_skill_json(&skill_str);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_skill_json() {
        let invalid_skill = json!({
            "name": "test-skill",
            // Missing required fields
            "type": "code_inline"
        });

        let skill_str = serde_json::to_string(&invalid_skill).unwrap();
        let result = SkillValidator::validate_skill_json(&skill_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_parameter_validation() {
        let params = json!({
            "name": "Alice",
            "age": 25
        });

        let param_defs = vec![
            Parameter {
                name: "name".to_string(),
                param_type: "string".to_string(),
                values: None,
                required: Some(true),
                pattern: None,
            },
            Parameter {
                name: "age".to_string(),
                param_type: "number".to_string(),
                values: None,
                required: Some(false),
                pattern: None,
            },
        ];

        let result = SkillValidator::validate_parameters(&params, &param_defs);
        assert!(result.is_ok());
    }

    #[test]
    fn test_missing_required_parameter() {
        let params = json!({
            "age": 25
            // Missing required "name" parameter
        });

        let param_defs = vec![Parameter {
            name: "name".to_string(),
            param_type: "string".to_string(),
            values: None,
            required: Some(true),
            pattern: None,
        }];

        let result = SkillValidator::validate_parameters(&params, &param_defs);
        assert!(result.is_err());
    }
}
