#[cfg(test)]
mod skill_types_tests {
    use crate::cli::skills::types::{JsonSkill, SkillType};
    use serde_json::json;

    #[test]
    fn test_code_inline_skill_deserialization() {
        let skill_json = json!({
            "name": "echo-skill",
            "description": "Echo command skill",
            "type": "code_inline",
            "command": "echo",
            "args": ["Hello World"],
            "timeout": 30
        });

        let skill: JsonSkill = serde_json::from_value(skill_json).unwrap();
        assert_eq!(skill.name, "echo-skill");
        assert_eq!(skill.skill_type, SkillType::CodeInline);
        assert_eq!(skill.command, Some("echo".to_string()));
        assert_eq!(skill.args, Some(vec!["Hello World".to_string()]));
        assert_eq!(skill.timeout, Some(30));
    }

    #[test]
    fn test_prompt_inline_skill_deserialization() {
        let skill_json = json!({
            "name": "greeting-skill",
            "description": "Greeting template skill",
            "type": "prompt_inline",
            "prompt": "Hello {name}, welcome to {place}!",
            "parameters": [
                {
                    "name": "name",
                    "type": "string",
                    "required": true
                },
                {
                    "name": "place", 
                    "type": "string",
                    "required": true
                }
            ]
        });

        let skill: JsonSkill = serde_json::from_value(skill_json).unwrap();
        assert_eq!(skill.name, "greeting-skill");
        assert_eq!(skill.skill_type, SkillType::PromptInline);
        assert_eq!(skill.prompt_template, Some("Hello {name}, welcome to {place}!".to_string()));
        assert!(skill.parameters.is_some());
        assert_eq!(skill.parameters.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_conversation_skill_deserialization() {
        let skill_json = json!({
            "name": "code-reviewer",
            "description": "Code review assistant",
            "type": "conversation",
            "prompt_template": "Analyze this code: {code}",
            "context_files": {
                "patterns": ["*.rs"],
                "max_files": 10,
                "max_file_size_kb": 100
            }
        });

        let skill: JsonSkill = serde_json::from_value(skill_json).unwrap();
        assert_eq!(skill.name, "code-reviewer");
        assert_eq!(skill.skill_type, SkillType::Conversation);
        assert_eq!(skill.prompt_template, Some("Analyze this code: {code}".to_string()));
        assert!(skill.context_files.is_some());
    }

    #[test]
    fn test_code_session_skill_deserialization() {
        let skill_json = json!({
            "name": "python-repl",
            "description": "Python REPL session",
            "type": "code_session",
            "command": "python3",
            "session_config": {
                "session_timeout": 3600,
                "max_sessions": 5,
                "cleanup_on_exit": true
            }
        });

        let skill: JsonSkill = serde_json::from_value(skill_json).unwrap();
        assert_eq!(skill.name, "python-repl");
        assert_eq!(skill.skill_type, SkillType::CodeSession);
        assert_eq!(skill.command, Some("python3".to_string()));
        assert!(skill.session_config.is_some());
    }

    #[test]
    fn test_minimal_skill_deserialization() {
        let skill_json = json!({
            "name": "minimal-skill",
            "type": "code_inline",
            "command": "echo"
        });

        let skill: JsonSkill = serde_json::from_value(skill_json).unwrap();
        assert_eq!(skill.name, "minimal-skill");
        assert_eq!(skill.skill_type, SkillType::CodeInline);
        assert_eq!(skill.command, Some("echo".to_string()));
        assert!(skill.description.is_none());
        assert!(skill.args.is_none());
    }
}
