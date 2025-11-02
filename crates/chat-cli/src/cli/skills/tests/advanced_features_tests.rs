#[cfg(test)]
mod advanced_features_tests {
    use serde_json::json;
    use tempfile::TempDir;

    use crate::cli::skills::{Skill, SkillError, SkillRegistry, SkillResult, SkillUI, UIElement};

    struct MockSkillWithAliases {
        name: String,
        aliases: Vec<String>,
    }

    impl MockSkillWithAliases {
        fn new(name: &str, aliases: Vec<&str>) -> Self {
            Self {
                name: name.to_string(),
                aliases: aliases.iter().map(|s| s.to_string()).collect(),
            }
        }
    }

    #[async_trait::async_trait]
    impl Skill for MockSkillWithAliases {
        fn name(&self) -> &str {
            &self.name
        }

        fn description(&self) -> &str {
            "Mock skill with aliases"
        }

        fn aliases(&self) -> Vec<String> {
            self.aliases.clone()
        }

        async fn execute(&self, _params: serde_json::Value) -> Result<SkillResult, SkillError> {
            Ok(SkillResult {
                output: format!("Executed {}", self.name),
                ui_updates: None,
                state_changes: None,
            })
        }

        async fn render_ui(&self) -> Result<SkillUI, SkillError> {
            Ok(SkillUI {
                elements: vec![UIElement::Text(format!("UI for {}", self.name))],
                interactive: false,
            })
        }
    }

    #[test]
    fn test_skill_aliases_registration() {
        let mut registry = SkillRegistry::new();
        let skill = Box::new(MockSkillWithAliases::new("calculator", vec!["calc", "math"]));

        registry.register_with_aliases(skill).unwrap();

        // Should be accessible by main name
        assert!(registry.get("calculator").is_some());

        // Should be accessible by aliases
        assert!(registry.get("calc").is_some());
        assert!(registry.get("math").is_some());

        // All should point to the same skill
        assert_eq!(registry.get("calculator").unwrap().name(), "calculator");
        assert_eq!(registry.get("calc").unwrap().name(), "calculator");
        assert_eq!(registry.get("math").unwrap().name(), "calculator");
    }

    #[test]
    fn test_skill_aliases_listing() {
        let mut registry = SkillRegistry::new();
        let skill = Box::new(MockSkillWithAliases::new("calculator", vec!["calc", "math"]));

        registry.register_with_aliases(skill).unwrap();

        let skills = registry.list();
        assert_eq!(skills.len(), 1); // Should only list unique skills, not aliases

        let aliases = registry.get_aliases("calculator");
        assert!(aliases.contains(&"calc".to_string()));
        assert!(aliases.contains(&"math".to_string()));
    }

    #[tokio::test]
    async fn test_skill_execution_by_alias() {
        let mut registry = SkillRegistry::new();
        let skill = Box::new(MockSkillWithAliases::new("calculator", vec!["calc"]));

        registry.register_with_aliases(skill).unwrap();

        // Execute by main name
        let result1 = registry.execute_skill("calculator", json!({})).await.unwrap();
        assert_eq!(result1.output, "Executed calculator");

        // Execute by alias
        let result2 = registry.execute_skill("calc", json!({})).await.unwrap();
        assert_eq!(result2.output, "Executed calculator");
    }

    #[tokio::test]
    async fn test_workspace_specific_skills() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_dir = temp_dir.path().join("workspace");
        std::fs::create_dir_all(&workspace_dir).unwrap();

        // Create a workspace-specific skill file
        let skill_file = workspace_dir.join(".q-skills").join("custom_skill.json");
        std::fs::create_dir_all(skill_file.parent().unwrap()).unwrap();

        let skill_config = json!({
            "name": "workspace_tool",
            "description": "A workspace-specific tool",
            "version": "1.0.0",
            "aliases": ["wt", "tool"],
            "type": "command",
            "command": "echo 'workspace tool executed'"
        });

        std::fs::write(&skill_file, serde_json::to_string_pretty(&skill_config).unwrap()).unwrap();

        // Load registry with workspace skills
        let registry = SkillRegistry::with_workspace_skills(&workspace_dir).await.unwrap();

        // Should have both builtin and workspace skills
        let skills = registry.list();
        let skill_names: Vec<&str> = skills.iter().map(|s| s.name()).collect();

        assert!(skill_names.contains(&"calculator")); // builtin
        assert!(skill_names.contains(&"workspace_tool")); // workspace-specific
    }

    #[tokio::test]
    async fn test_workspace_skill_priority() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_dir = temp_dir.path().join("workspace");
        std::fs::create_dir_all(&workspace_dir).unwrap();

        // Create a workspace skill that overrides a builtin
        let skill_file = workspace_dir.join(".q-skills").join("calculator.json");
        std::fs::create_dir_all(skill_file.parent().unwrap()).unwrap();

        let skill_config = json!({
            "name": "calculator",
            "description": "Custom workspace calculator",
            "version": "2.0.0",
            "type": "command",
            "command": "echo 'custom calculator'"
        });

        std::fs::write(&skill_file, serde_json::to_string_pretty(&skill_config).unwrap()).unwrap();

        let registry = SkillRegistry::with_workspace_skills(&workspace_dir).await.unwrap();

        // Workspace skill should override builtin
        let calculator = registry.get("calculator").unwrap();
        assert_eq!(calculator.description(), "Custom workspace calculator");
    }

    #[test]
    fn test_skill_discovery_in_multiple_locations() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_dir = temp_dir.path().join("workspace");
        let global_dir = temp_dir.path().join("global");

        std::fs::create_dir_all(&workspace_dir.join(".q-skills")).unwrap();
        std::fs::create_dir_all(&global_dir).unwrap();

        // Create skills in different locations
        let workspace_skill = workspace_dir.join(".q-skills").join("workspace_skill.json");
        let global_skill = global_dir.join("global_skill.json");

        std::fs::write(
            &workspace_skill,
            json!({
                "name": "workspace_skill",
                "description": "Workspace skill",
                "version": "1.0.0"
            })
            .to_string(),
        )
        .unwrap();

        std::fs::write(
            &global_skill,
            json!({
                "name": "global_skill",
                "description": "Global skill",
                "version": "1.0.0"
            })
            .to_string(),
        )
        .unwrap();

        let workspace_skills_dir = workspace_dir.join(".q-skills");
        let locations = vec![workspace_skills_dir.as_path(), global_dir.as_path()];
        let discovered = SkillRegistry::discover_skills_in_locations(&locations);

        assert_eq!(discovered.len(), 2);
        assert!(discovered.iter().any(|s| s.name == "workspace_skill"));
        assert!(discovered.iter().any(|s| s.name == "global_skill"));
    }

    #[test]
    fn test_skill_metadata_with_aliases() {
        let skill = MockSkillWithAliases::new("test_skill", vec!["ts", "test"]);

        assert_eq!(skill.name(), "test_skill");
        assert_eq!(skill.aliases(), vec!["ts", "test"]);
        assert_eq!(skill.description(), "Mock skill with aliases");
    }

    #[tokio::test]
    async fn test_workspace_skills_hot_reload() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_dir = temp_dir.path().join("workspace");
        let skills_dir = workspace_dir.join(".q-skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        let mut registry = SkillRegistry::with_workspace_skills(&workspace_dir).await.unwrap();

        // Initially should only have builtins
        let initial_count = registry.list().len();

        // Add a new skill file
        let new_skill_file = skills_dir.join("new_skill.json");
        std::fs::write(
            &new_skill_file,
            json!({
                "name": "new_skill",
                "description": "Dynamically added skill",
                "version": "1.0.0",
                "type": "command",
                "command": "echo",
                "args": ["Hello from new skill"]
            })
            .to_string(),
        )
        .unwrap();

        // Reload workspace skills
        registry.reload_workspace_skills(&workspace_dir).await.unwrap();

        // Should now have the new skill
        assert_eq!(registry.list().len(), initial_count + 1);
        assert!(registry.get("new_skill").is_some());
    }
}
