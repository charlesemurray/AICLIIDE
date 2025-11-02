#[cfg(test)]
mod integration_tests {
    use std::fs;

    use serde_json::json;
    use tempfile::TempDir;

    use crate::cli::skills::SkillRegistry;

    #[tokio::test]
    async fn test_end_to_end_skill_loading_and_execution() {
        let temp_dir = TempDir::new().unwrap();
        let skills_dir = temp_dir.path().join(".q-skills");
        fs::create_dir_all(&skills_dir).unwrap();

        // Create a real skill file that should work end-to-end
        let echo_skill = skills_dir.join("echo-skill.json");
        let skill_config = json!({
            "name": "echo-test",
            "description": "Echo test for integration",
            "version": "1.0.0",
            "type": "code_inline",
            "command": "echo",
            "args": ["Integration test successful"],
            "timeout": 30
        });

        fs::write(&echo_skill, serde_json::to_string_pretty(&skill_config).unwrap()).unwrap();

        // Load skills and verify the skill is registered
        let registry = SkillRegistry::with_workspace_skills(temp_dir.path()).await.unwrap();

        // Verify skill is loaded
        assert!(registry.get("echo-test").is_some());
        let skill = registry.get("echo-test").unwrap();
        assert_eq!(skill.name(), "echo-test");
        assert_eq!(skill.description(), "Echo test for integration");

        // Execute the skill and verify output
        let result = skill.execute(json!({})).await.unwrap();
        assert!(result.output.contains("Integration test successful"));
    }

    #[tokio::test]
    async fn test_template_skill_with_parameters() {
        let temp_dir = TempDir::new().unwrap();
        let skills_dir = temp_dir.path().join(".q-skills");
        fs::create_dir_all(&skills_dir).unwrap();

        let template_skill = skills_dir.join("template-skill.json");
        let skill_config = json!({
            "name": "template-test",
            "description": "Template test skill",
            "version": "1.0.0",
            "type": "prompt_inline",
            "prompt": "Hello {name}!",
            "parameters": [
                {
                    "name": "name",
                    "type": "string",
                    "required": true
                }
            ]
        });

        fs::write(&template_skill, serde_json::to_string_pretty(&skill_config).unwrap()).unwrap();

        let registry = SkillRegistry::with_workspace_skills(temp_dir.path()).await.unwrap();
        assert!(registry.get("template-test").is_some());
    }

    #[tokio::test]
    async fn test_conversation_skill_loading() {
        let temp_dir = TempDir::new().unwrap();
        let skills_dir = temp_dir.path().join(".q-skills");
        fs::create_dir_all(&skills_dir).unwrap();

        let conversation_skill = skills_dir.join("conversation-skill.json");
        let skill_config = json!({
            "name": "conversation-test",
            "description": "Conversation test skill",
            "version": "1.0.0",
            "type": "conversation",
            "prompt_template": "You are a helpful assistant",
            "context_files": {
                "patterns": ["*.rs", "*.py"],
                "max_files": 10,
                "max_file_size_kb": 100
            }
        });

        fs::write(
            &conversation_skill,
            serde_json::to_string_pretty(&skill_config).unwrap(),
        )
        .unwrap();

        let registry = SkillRegistry::with_workspace_skills(temp_dir.path()).await.unwrap();
        assert!(registry.get("conversation-test").is_some());
    }
}
