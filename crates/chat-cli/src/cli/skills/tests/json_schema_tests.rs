#[cfg(test)]
mod json_schema_tests {
    use crate::cli::skills::types::{JsonSkill, SkillType, Parameter};
    use serde_json::json;

    // Core Fields Tests
    #[test]
    fn test_required_fields_only() {
        let skill_json = json!({
            "name": "minimal-skill",
            "type": "code_inline"
        });

        let skill: JsonSkill = serde_json::from_value(skill_json).unwrap();
        assert_eq!(skill.name, "minimal-skill");
        assert_eq!(skill.skill_type, SkillType::CodeInline);
        assert!(skill.description.is_none());
        assert!(skill.timeout.is_none());
    }

    #[test]
    fn test_all_core_fields() {
        let skill_json = json!({
            "name": "full-skill",
            "description": "Complete skill with all core fields",
            "type": "code_inline",
            "timeout": 60,
            "command": "echo",
            "args": ["test"]
        });

        let skill: JsonSkill = serde_json::from_value(skill_json).unwrap();
        assert_eq!(skill.name, "full-skill");
        assert_eq!(skill.description, Some("Complete skill with all core fields".to_string()));
        assert_eq!(skill.skill_type, SkillType::CodeInline);
        assert_eq!(skill.timeout, Some(60));
        assert_eq!(skill.command, Some("echo".to_string()));
        assert_eq!(skill.args, Some(vec!["test".to_string()]));
    }

    #[test]
    fn test_missing_name_fails() {
        let skill_json = json!({
            "type": "code_inline"
        });

        let result = serde_json::from_value::<JsonSkill>(skill_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_type_fails() {
        let skill_json = json!({
            "name": "test-skill"
        });

        let result = serde_json::from_value::<JsonSkill>(skill_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_skill_type_fails() {
        let skill_json = json!({
            "name": "test-skill",
            "type": "invalid_type"
        });

        let result = serde_json::from_value::<JsonSkill>(skill_json);
        assert!(result.is_err());
    }

    // Skill Type Tests
    #[test]
    fn test_all_skill_types() {
        let types = vec![
            ("command", SkillType::Command),
            ("code_inline", SkillType::CodeInline),
            ("code_session", SkillType::CodeSession),
            ("conversation", SkillType::Conversation),
            ("prompt_inline", SkillType::PromptInline),
        ];

        for (type_str, expected_type) in types {
            let skill_json = json!({
                "name": "test-skill",
                "type": type_str
            });

            let skill: JsonSkill = serde_json::from_value(skill_json).unwrap();
            assert_eq!(skill.skill_type, expected_type);
        }
    }

    // Code Inline/Command Tests
    #[test]
    fn test_code_inline_with_command_and_args() {
        let skill_json = json!({
            "name": "echo-skill",
            "type": "code_inline",
            "command": "echo",
            "args": ["Hello", "World", "--flag"]
        });

        let skill: JsonSkill = serde_json::from_value(skill_json).unwrap();
        assert_eq!(skill.command, Some("echo".to_string()));
        assert_eq!(skill.args, Some(vec!["Hello".to_string(), "World".to_string(), "--flag".to_string()]));
    }

    #[test]
    fn test_command_without_args() {
        let skill_json = json!({
            "name": "ls-skill",
            "type": "command",
            "command": "ls"
        });

        let skill: JsonSkill = serde_json::from_value(skill_json).unwrap();
        assert_eq!(skill.command, Some("ls".to_string()));
        assert!(skill.args.is_none());
    }

    // Prompt Template Tests
    #[test]
    fn test_prompt_template_field() {
        let skill_json = json!({
            "name": "prompt-skill",
            "type": "prompt_inline",
            "prompt_template": "Hello {name}!"
        });

        let skill: JsonSkill = serde_json::from_value(skill_json).unwrap();
        assert_eq!(skill.prompt_template, Some("Hello {name}!".to_string()));
    }

    #[test]
    fn test_prompt_alias_field() {
        let skill_json = json!({
            "name": "prompt-skill",
            "type": "prompt_inline",
            "prompt": "Hello {name}!"
        });

        let skill: JsonSkill = serde_json::from_value(skill_json).unwrap();
        assert_eq!(skill.prompt_template, Some("Hello {name}!".to_string()));
    }

    #[test]
    fn test_both_prompt_fields() {
        // prompt_template should take precedence
        let skill_json = json!({
            "name": "prompt-skill",
            "type": "prompt_inline",
            "prompt_template": "Template field",
            "prompt": "Alias field"
        });

        let skill: JsonSkill = serde_json::from_value(skill_json).unwrap();
        assert_eq!(skill.prompt_template, Some("Template field".to_string()));
    }

    // Parameter Tests
    #[test]
    fn test_parameter_all_fields() {
        let skill_json = json!({
            "name": "param-skill",
            "type": "prompt_inline",
            "prompt": "Hello {name}!",
            "parameters": [
                {
                    "name": "name",
                    "type": "string",
                    "required": true,
                    "pattern": "^[a-zA-Z]+$",
                    "values": ["Alice", "Bob"]
                }
            ]
        });

        let skill: JsonSkill = serde_json::from_value(skill_json).unwrap();
        let params = skill.parameters.unwrap();
        assert_eq!(params.len(), 1);
        
        let param = &params[0];
        assert_eq!(param.name, "name");
        assert_eq!(param.param_type, "string");
        assert_eq!(param.required, Some(true));
        assert_eq!(param.pattern, Some("^[a-zA-Z]+$".to_string()));
        assert_eq!(param.values, Some(vec!["Alice".to_string(), "Bob".to_string()]));
    }

    #[test]
    fn test_parameter_minimal() {
        let skill_json = json!({
            "name": "param-skill",
            "type": "prompt_inline",
            "prompt": "Hello {name}!",
            "parameters": [
                {
                    "name": "name",
                    "type": "string"
                }
            ]
        });

        let skill: JsonSkill = serde_json::from_value(skill_json).unwrap();
        let params = skill.parameters.unwrap();
        let param = &params[0];
        
        assert_eq!(param.name, "name");
        assert_eq!(param.param_type, "string");
        assert!(param.required.is_none());
        assert!(param.pattern.is_none());
        assert!(param.values.is_none());
    }

    #[test]
    fn test_parameter_missing_name_fails() {
        let skill_json = json!({
            "name": "param-skill",
            "type": "prompt_inline",
            "parameters": [
                {
                    "type": "string"
                }
            ]
        });

        let result = serde_json::from_value::<JsonSkill>(skill_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_parameter_missing_type_fails() {
        let skill_json = json!({
            "name": "param-skill",
            "type": "prompt_inline",
            "parameters": [
                {
                    "name": "test"
                }
            ]
        });

        let result = serde_json::from_value::<JsonSkill>(skill_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_parameters() {
        let skill_json = json!({
            "name": "multi-param-skill",
            "type": "prompt_inline",
            "prompt": "Hello {name} from {place}!",
            "parameters": [
                {
                    "name": "name",
                    "type": "string",
                    "required": true
                },
                {
                    "name": "place",
                    "type": "enum",
                    "values": ["home", "work", "school"],
                    "required": false
                }
            ]
        });

        let skill: JsonSkill = serde_json::from_value(skill_json).unwrap();
        let params = skill.parameters.unwrap();
        assert_eq!(params.len(), 2);
        
        assert_eq!(params[0].name, "name");
        assert_eq!(params[0].param_type, "string");
        assert_eq!(params[0].required, Some(true));
        
        assert_eq!(params[1].name, "place");
        assert_eq!(params[1].param_type, "enum");
        assert_eq!(params[1].required, Some(false));
        assert_eq!(params[1].values, Some(vec!["home".to_string(), "work".to_string(), "school".to_string()]));
    }

    // Security Configuration Tests
    #[test]
    fn test_security_config() {
        let skill_json = json!({
            "name": "secure-skill",
            "type": "code_inline",
            "command": "echo",
            "security": {
                "resource_limits": {
                    "max_memory_mb": 100,
                    "max_execution_time": 30,
                    "max_cpu_percent": 50
                }
            }
        });

        let skill: JsonSkill = serde_json::from_value(skill_json).unwrap();
        assert!(skill.security.is_some());
        
        let security = skill.security.unwrap();
        assert!(security.resource_limits.is_some());
        
        let limits = security.resource_limits.unwrap();
        assert_eq!(limits.max_memory_mb, Some(100));
        assert_eq!(limits.max_execution_time, Some(30));
        assert_eq!(limits.max_cpu_percent, Some(50));
    }

    // Session Config Tests
    #[test]
    fn test_session_config() {
        let skill_json = json!({
            "name": "session-skill",
            "type": "code_session",
            "command": "python3",
            "session_config": {
                "session_timeout": 3600,
                "max_sessions": 5,
                "cleanup_on_exit": true
            }
        });

        let skill: JsonSkill = serde_json::from_value(skill_json).unwrap();
        assert!(skill.session_config.is_some());
        
        let config = skill.session_config.unwrap();
        assert_eq!(config.session_timeout, Some(3600));
    }

    // Context Files Tests
    #[test]
    fn test_context_files() {
        let skill_json = json!({
            "name": "context-skill",
            "type": "conversation",
            "prompt_template": "Analyze {code}",
            "context_files": {
                "patterns": ["*.rs", "*.py"],
                "max_files": 10,
                "max_file_size_kb": 100
            }
        });

        let skill: JsonSkill = serde_json::from_value(skill_json).unwrap();
        assert!(skill.context_files.is_some());
        
        let context = skill.context_files.unwrap();
        assert_eq!(context.patterns, vec!["*.rs", "*.py"]);
        assert_eq!(context.max_files, Some(10));
        assert_eq!(context.max_file_size_kb, Some(100));
    }

    // Extra Fields Tests (serde flatten)
    #[test]
    fn test_extra_fields_captured() {
        let skill_json = json!({
            "name": "extra-skill",
            "type": "code_inline",
            "command": "echo",
            "version": "1.0.0",
            "author": "test",
            "custom_field": "custom_value"
        });

        let skill: JsonSkill = serde_json::from_value(skill_json).unwrap();
        assert_eq!(skill.extra.get("version").unwrap(), "1.0.0");
        assert_eq!(skill.extra.get("author").unwrap(), "test");
        assert_eq!(skill.extra.get("custom_field").unwrap(), "custom_value");
    }

    // Complete Skill Examples Tests
    #[test]
    fn test_complete_prompt_inline_skill() {
        let skill_json = json!({
            "name": "greeting",
            "description": "Generate personalized greetings",
            "type": "prompt_inline",
            "prompt_template": "Hello {name}! Welcome to {place}. Today is {day}.",
            "timeout": 10,
            "parameters": [
                {
                    "name": "name",
                    "type": "string",
                    "required": true,
                    "pattern": "^[a-zA-Z ]+$"
                },
                {
                    "name": "place",
                    "type": "string",
                    "required": false
                },
                {
                    "name": "day",
                    "type": "enum",
                    "values": ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday"],
                    "required": true
                }
            ]
        });

        let skill: JsonSkill = serde_json::from_value(skill_json).unwrap();
        assert_eq!(skill.name, "greeting");
        assert_eq!(skill.description, Some("Generate personalized greetings".to_string()));
        assert_eq!(skill.skill_type, SkillType::PromptInline);
        assert_eq!(skill.prompt_template, Some("Hello {name}! Welcome to {place}. Today is {day}.".to_string()));
        assert_eq!(skill.timeout, Some(10));
        
        let params = skill.parameters.unwrap();
        assert_eq!(params.len(), 3);
        assert_eq!(params[0].name, "name");
        assert_eq!(params[1].name, "place");
        assert_eq!(params[2].name, "day");
    }

    #[test]
    fn test_complete_code_session_skill() {
        let skill_json = json!({
            "name": "python-repl",
            "description": "Interactive Python session",
            "type": "code_session",
            "command": "python3",
            "timeout": 300,
            "session_config": {
                "session_timeout": 3600,
                "max_sessions": 3,
                "cleanup_on_exit": true
            },
            "security": {
                "resource_limits": {
                    "max_memory_mb": 256,
                    "max_execution_time": 300
                }
            }
        });

        let skill: JsonSkill = serde_json::from_value(skill_json).unwrap();
        assert_eq!(skill.name, "python-repl");
        assert_eq!(skill.skill_type, SkillType::CodeSession);
        assert_eq!(skill.command, Some("python3".to_string()));
        assert!(skill.session_config.is_some());
        assert!(skill.security.is_some());
    }

    #[test]
    fn test_complete_conversation_skill() {
        let skill_json = json!({
            "name": "code-reviewer",
            "description": "AI-powered code review assistant",
            "type": "conversation",
            "prompt_template": "Review this {language} code: {code}",
            "parameters": [
                {
                    "name": "language",
                    "type": "enum",
                    "values": ["python", "javascript", "rust"],
                    "required": true
                },
                {
                    "name": "code",
                    "type": "string",
                    "required": true
                }
            ],
            "context_files": {
                "patterns": ["*.py", "*.js", "*.rs"],
                "max_files": 5,
                "max_file_size_kb": 50
            }
        });

        let skill: JsonSkill = serde_json::from_value(skill_json).unwrap();
        assert_eq!(skill.name, "code-reviewer");
        assert_eq!(skill.skill_type, SkillType::Conversation);
        assert!(skill.parameters.is_some());
        assert!(skill.context_files.is_some());
    }
}
