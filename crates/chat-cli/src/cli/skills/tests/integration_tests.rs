#[cfg(test)]
mod integration_tests {
    use crate::cli::skills::{SkillRegistry, SkillValidator};
    use serde_json::json;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_end_to_end_skill_loading_and_execution() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_dir = temp_dir.path().join("workspace");
        let skills_dir = workspace_dir.join(".q-skills");
        fs::create_dir_all(&skills_dir).unwrap();

        // Create a real skill file that should work end-to-end
        let echo_skill = skills_dir.join("echo-skill.json");
        let skill_config = json!({
            "name": "echo-test",
            "description": "Echo test for integration",
            "version": "1.0.0",
            "aliases": ["echo", "et"],
            "type": "code_inline",
            "command": "echo",
            "args": ["Integration test successful"]
        });
        
        fs::write(&echo_skill, serde_json::to_string_pretty(&skill_config).unwrap()).unwrap();

        // Load skills and verify the skill is registered
        let registry = SkillRegistry::with_workspace_skills(&workspace_dir).await.unwrap();
        
        // Verify skill is loaded
        assert!(registry.get("echo-test").is_some());
        let skill = registry.get("echo-test").unwrap();
        assert_eq!(skill.name(), "echo-test");
        assert_eq!(skill.aliases(), vec!["echo", "et"]);

        // Execute the skill and verify output
        let result = skill.execute(json!({})).await.unwrap();
        assert!(result.output.contains("Integration test successful"));
    }

    #[tokio::test]
    async fn test_prompt_inline_skill_with_parameters() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_dir = temp_dir.path().join("workspace");
        let skills_dir = workspace_dir.join(".q-skills");
        fs::create_dir_all(&skills_dir).unwrap();

        // Create a parameterized prompt skill
        let prompt_skill = skills_dir.join("greeting-skill.json");
        let skill_config = json!({
            "name": "greeting",
            "description": "Generate personalized greetings",
            "version": "1.0.0",
            "type": "prompt_inline",
            "prompt": "Hello {name}! Welcome to {place}. Today is {day}.",
            "parameters": [
                {
                    "name": "name",
                    "description": "Person's name",
                    "required": true
                },
                {
                    "name": "place",
                    "description": "Location",
                    "required": false,
                    "default": "Q CLI"
                },
                {
                    "name": "day",
                    "description": "Day of the week",
                    "required": true
                }
            ]
        });
        
        fs::write(&prompt_skill, serde_json::to_string_pretty(&skill_config).unwrap()).unwrap();

        let registry = SkillRegistry::with_workspace_skills(&workspace_dir).await.unwrap();
        let skill = registry.get("greeting").unwrap();

        // Test with all parameters
        let params = json!({
            "name": "Alice",
            "place": "Wonderland", 
            "day": "Friday"
        });
        
        let result = skill.execute(params).await.unwrap();
        assert_eq!(result.output, "Hello Alice! Welcome to Wonderland. Today is Friday.");

        // Test UI rendering
        let ui = skill.render_ui().await.unwrap();
        assert!(ui.interactive);
        assert_eq!(ui.elements.len(), 4); // Text + 3 inputs
    }

    #[tokio::test]
    async fn test_skill_priority_workspace_overrides_builtin() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_dir = temp_dir.path().join("workspace");
        let skills_dir = workspace_dir.join(".q-skills");
        fs::create_dir_all(&skills_dir).unwrap();

        // Create a workspace skill that overrides the builtin calculator
        let calculator_override = skills_dir.join("calculator.json");
        let skill_config = json!({
            "name": "calculator",
            "description": "Custom workspace calculator",
            "version": "2.0.0",
            "type": "code_inline",
            "command": "echo",
            "args": ["Custom calculator result: 42"]
        });
        
        fs::write(&calculator_override, serde_json::to_string_pretty(&skill_config).unwrap()).unwrap();

        let registry = SkillRegistry::with_workspace_skills(&workspace_dir).await.unwrap();
        let calculator = registry.get("calculator").unwrap();
        
        // Should get the workspace version, not the builtin
        assert_eq!(calculator.description(), "Custom workspace calculator");
        
        let result = calculator.execute(json!({})).await.unwrap();
        assert!(result.output.contains("Custom calculator result: 42"));
    }

    #[tokio::test]
    async fn test_skill_validation_rejects_invalid_configs() {
        // Test missing required fields
        let invalid_config = json!({
            "name": "test",
            "description": "Test skill"
            // Missing version and type
        });
        
        let result = SkillValidator::validate_skill_json(&invalid_config.to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing required field"));

        // Test invalid skill type
        let invalid_type_config = json!({
            "name": "test",
            "description": "Test skill",
            "version": "1.0.0",
            "type": "invalid_type"
        });
        
        let result = SkillValidator::validate_skill_json(&invalid_type_config.to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown skill type"));

        // Test code skill without command
        let missing_command_config = json!({
            "name": "test",
            "description": "Test skill",
            "version": "1.0.0",
            "type": "code_inline"
            // Missing command field
        });
        
        let result = SkillValidator::validate_skill_json(&missing_command_config.to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("require 'command' field"));
    }

    #[tokio::test]
    async fn test_skill_hot_reload_integration() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_dir = temp_dir.path().join("workspace");
        let skills_dir = workspace_dir.join(".q-skills");
        fs::create_dir_all(&skills_dir).unwrap();

        // Start with empty registry
        let mut registry = SkillRegistry::with_workspace_skills(&workspace_dir).await.unwrap();
        let initial_count = registry.list().len();

        // Add a new skill file
        let new_skill = skills_dir.join("dynamic-skill.json");
        let skill_config = json!({
            "name": "dynamic",
            "description": "Dynamically added skill",
            "version": "1.0.0",
            "type": "code_inline",
            "command": "echo",
            "args": ["Dynamic skill works!"]
        });
        
        fs::write(&new_skill, serde_json::to_string_pretty(&skill_config).unwrap()).unwrap();

        // Reload and verify the skill is available
        registry.reload_workspace_skills(&workspace_dir).await.unwrap();
        
        assert_eq!(registry.list().len(), initial_count + 1);
        let dynamic_skill = registry.get("dynamic").unwrap();
        
        let result = dynamic_skill.execute(json!({})).await.unwrap();
        assert!(result.output.contains("Dynamic skill works!"));
    }

    #[tokio::test]
    async fn test_conversation_skill_with_context() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_dir = temp_dir.path().join("workspace");
        let skills_dir = workspace_dir.join(".q-skills");
        fs::create_dir_all(&skills_dir).unwrap();

        // Create a conversation skill
        let conversation_skill = skills_dir.join("review-skill.json");
        let skill_config = json!({
            "name": "code-reviewer",
            "description": "AI code review assistant",
            "version": "1.0.0",
            "type": "conversation",
            "prompt_template": "Review this {language} code:\n\n{code}\n\nFocus on: {focus}",
            "context_files": ["*.rs", "*.py"],
            "model": "claude-3-sonnet"
        });
        
        fs::write(&conversation_skill, serde_json::to_string_pretty(&skill_config).unwrap()).unwrap();

        let registry = SkillRegistry::with_workspace_skills(&workspace_dir).await.unwrap();
        let skill = registry.get("code-reviewer").unwrap();

        let params = json!({
            "language": "Rust",
            "code": "fn main() { println!(\"Hello\"); }",
            "focus": "performance and safety"
        });
        
        let result = skill.execute(params).await.unwrap();
        assert!(result.output.contains("Review this Rust code"));
        assert!(result.output.contains("performance and safety"));
        assert!(result.state_changes.is_some());
    }

    #[test]
    fn test_skill_discovery_in_multiple_real_locations() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create multiple skill directories
        let workspace_dir = temp_dir.path().join("workspace").join(".q-skills");
        let global_dir = temp_dir.path().join("global");
        fs::create_dir_all(&workspace_dir).unwrap();
        fs::create_dir_all(&global_dir).unwrap();

        // Create skills in different locations
        let workspace_skill = workspace_dir.join("workspace-skill.json");
        let global_skill = global_dir.join("global-skill.json");
        
        fs::write(&workspace_skill, json!({
            "name": "workspace-tool",
            "description": "Workspace specific tool",
            "version": "1.0.0",
            "type": "code_inline",
            "command": "echo",
            "args": ["workspace"]
        }).to_string()).unwrap();
        
        fs::write(&global_skill, json!({
            "name": "global-tool",
            "description": "Global tool",
            "version": "1.0.0", 
            "type": "code_inline",
            "command": "echo",
            "args": ["global"]
        }).to_string()).unwrap();

        // Test discovery
        let locations = vec![workspace_dir.as_path(), global_dir.as_path()];
        let discovered = SkillRegistry::discover_skills_in_locations(&locations);
        
        assert_eq!(discovered.len(), 2);
        assert!(discovered.iter().any(|s| s.name == "workspace-tool"));
        assert!(discovered.iter().any(|s| s.name == "global-tool"));
    }
}
