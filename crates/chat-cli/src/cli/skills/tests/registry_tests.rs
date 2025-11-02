#[cfg(test)]
mod registry_tests {
    use std::fs;

    use serde_json::json;
    use tempfile::TempDir;

    use crate::cli::skills::{Skill, SkillError, SkillRegistry, SkillResult, SkillUI, UIElement};

    struct MockSkill {
        name: String,
        description: String,
    }

    impl MockSkill {
        fn new(name: &str, description: &str) -> Self {
            Self {
                name: name.to_string(),
                description: description.to_string(),
            }
        }
    }

    #[async_trait::async_trait]
    impl Skill for MockSkill {
        fn name(&self) -> &str {
            &self.name
        }

        fn description(&self) -> &str {
            &self.description
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

        fn supports_interactive(&self) -> bool {
            false
        }

        fn aliases(&self) -> Vec<String> {
            vec![]
        }
    }

    #[test]
    fn test_skill_registry_creation() {
        let registry = SkillRegistry::new();
        assert!(registry.list().is_empty());
    }

    #[test]
    fn test_skill_registration() {
        let mut registry = SkillRegistry::new();
        let skill = MockSkill::new("test", "Test skill");

        registry.register_override(Box::new(skill)).unwrap();
        assert_eq!(registry.list().len(), 1);
        assert!(registry.get("test").is_some());
    }

    #[tokio::test]
    async fn test_json_skill_loading() {
        let temp_dir = TempDir::new().unwrap();
        let skills_dir = temp_dir.path().join(".q-skills");
        fs::create_dir_all(&skills_dir).unwrap();

        // Create a valid JSON skill
        let skill_json = json!({
            "name": "test-skill",
            "description": "Test skill",
            "version": "1.0.0",
            "type": "code_inline",
            "command": "echo",
            "args": ["Hello"],
            "timeout": 30
        });

        fs::write(
            skills_dir.join("test-skill.json"),
            serde_json::to_string_pretty(&skill_json).unwrap(),
        )
        .unwrap();

        let registry = SkillRegistry::with_workspace_skills(temp_dir.path()).await.unwrap();
        assert!(registry.get("test-skill").is_some());
    }
}
