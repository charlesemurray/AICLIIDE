#[cfg(test)]
mod skill_types_tests {
    use crate::cli::skills::{Skill, EnhancedSkillInfo, SkillType, SkillParameter};
    use crate::cli::skills::builtin::TypedSkill;
    use serde_json::json;

    #[tokio::test]
    async fn test_code_inline_skill() {
        let skill_info = EnhancedSkillInfo {
            name: "echo-test".to_string(),
            description: "Echo test command".to_string(),
            version: "1.0.0".to_string(),
            aliases: Some(vec!["echo".to_string()]),
            scope: None,
            skill_type: SkillType::CodeInline {
                command: "echo".to_string(),
                args: Some(vec!["Hello World".to_string()]),
                working_dir: None,
            },
        };

        let skill = TypedSkill::new(skill_info).unwrap();
        assert_eq!(skill.name(), "echo-test");
        assert_eq!(skill.aliases(), vec!["echo"]);

        let result = skill.execute(json!({})).await.unwrap();
        assert!(result.output.contains("Hello World"));
    }

    #[tokio::test]
    async fn test_prompt_inline_skill() {
        let skill_info = EnhancedSkillInfo {
            name: "template-test".to_string(),
            description: "Template test".to_string(),
            version: "1.0.0".to_string(),
            aliases: None,
            scope: None,
            skill_type: SkillType::PromptInline {
                prompt: "Hello {name}, welcome to {place}!".to_string(),
                parameters: Some(vec![
                    SkillParameter {
                        name: "name".to_string(),
                        description: "Your name".to_string(),
                        required: true,
                        default: None,
                    },
                    SkillParameter {
                        name: "place".to_string(),
                        description: "Location".to_string(),
                        required: false,
                        default: Some("Q CLI".to_string()),
                    },
                ]),
            },
        };

        let skill = TypedSkill::new(skill_info).unwrap();
        
        let params = json!({
            "name": "Alice",
            "place": "Wonderland"
        });
        
        let result = skill.execute(params).await.unwrap();
        assert_eq!(result.output, "Hello Alice, welcome to Wonderland!");
    }

    #[tokio::test]
    async fn test_conversation_skill() {
        let skill_info = EnhancedSkillInfo {
            name: "chat-test".to_string(),
            description: "Chat test".to_string(),
            version: "1.0.0".to_string(),
            aliases: None,
            scope: None,
            skill_type: SkillType::Conversation {
                prompt_template: "Analyze this code: {code}".to_string(),
                context_files: Some(vec!["*.rs".to_string()]),
                model: Some("claude-3-sonnet".to_string()),
            },
        };

        let skill = TypedSkill::new(skill_info).unwrap();
        
        let params = json!({
            "code": "fn main() { println!(\"Hello\"); }"
        });
        
        let result = skill.execute(params).await.unwrap();
        assert!(result.output.contains("Analyze this code"));
        assert!(result.state_changes.is_some());
    }

    #[tokio::test]
    async fn test_code_session_skill() {
        let skill_info = EnhancedSkillInfo {
            name: "session-test".to_string(),
            description: "Session test".to_string(),
            version: "1.0.0".to_string(),
            aliases: None,
            scope: None,
            skill_type: SkillType::CodeSession {
                command: "echo".to_string(),
                args: Some(vec!["Session started".to_string()]),
                working_dir: None,
                session_timeout: Some(60),
            },
        };

        let skill = TypedSkill::new(skill_info).unwrap();
        let result = skill.execute(json!({})).await.unwrap();
        assert!(result.output.contains("Session started"));
    }

    #[tokio::test]
    async fn test_skill_ui_rendering() {
        let skill_info = EnhancedSkillInfo {
            name: "ui-test".to_string(),
            description: "UI test".to_string(),
            version: "1.0.0".to_string(),
            aliases: None,
            scope: None,
            skill_type: SkillType::PromptInline {
                prompt: "Test prompt".to_string(),
                parameters: Some(vec![
                    SkillParameter {
                        name: "input1".to_string(),
                        description: "First input".to_string(),
                        required: true,
                        default: None,
                    },
                ]),
            },
        };

        let skill = TypedSkill::new(skill_info).unwrap();
        let ui = skill.render_ui().await.unwrap();
        
        assert!(ui.interactive);
        assert_eq!(ui.elements.len(), 2); // Text + Input
    }

    #[test]
    fn test_enhanced_skill_info_serialization() {
        let skill_info = EnhancedSkillInfo {
            name: "test".to_string(),
            description: "Test skill".to_string(),
            version: "1.0.0".to_string(),
            aliases: Some(vec!["t".to_string()]),
            scope: None,
            skill_type: SkillType::CodeInline {
                command: "echo".to_string(),
                args: None,
                working_dir: None,
            },
        };

        let json = serde_json::to_string(&skill_info).unwrap();
        let deserialized: EnhancedSkillInfo = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.name, "test");
        assert_eq!(deserialized.aliases, Some(vec!["t".to_string()]));
    }
}
