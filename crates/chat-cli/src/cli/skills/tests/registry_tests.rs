#[cfg(test)]
mod registry_tests {
    use crate::cli::skills::{Skill, SkillResult, SkillError, SkillUI, UIElement, SkillRegistry};
    use serde_json::json;
    use tempfile::TempDir;

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
    }

    #[test]
    fn test_skill_registry_creation() {
        let registry = SkillRegistry::new();
        assert!(registry.list().is_empty());
    }

    #[test]
    fn test_skill_registration() {
        let mut registry = SkillRegistry::new();
        let skill = Box::new(MockSkill::new("test_skill", "A test skill"));
        
        registry.register(skill).unwrap();
        assert_eq!(registry.list().len(), 1);
        assert!(registry.get("test_skill").is_some());
    }

    #[test]
    fn test_skill_registration_duplicate_name() {
        let mut registry = SkillRegistry::new();
        let skill1 = Box::new(MockSkill::new("duplicate", "First skill"));
        let skill2 = Box::new(MockSkill::new("duplicate", "Second skill"));
        
        registry.register(skill1).unwrap();
        let result = registry.register(skill2);
        
        assert!(result.is_err());
        match result.unwrap_err() {
            SkillError::InvalidInput(msg) => assert!(msg.contains("already registered")),
            _ => panic!("Expected InvalidInput error for duplicate registration"),
        }
    }

    #[test]
    fn test_skill_retrieval() {
        let mut registry = SkillRegistry::new();
        let skill = Box::new(MockSkill::new("retrieval_test", "Test retrieval"));
        
        registry.register(skill).unwrap();
        
        let retrieved = registry.get("retrieval_test");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name(), "retrieval_test");
        
        let not_found = registry.get("nonexistent");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_skill_listing() {
        let mut registry = SkillRegistry::new();
        
        registry.register(Box::new(MockSkill::new("skill1", "First skill"))).unwrap();
        registry.register(Box::new(MockSkill::new("skill2", "Second skill"))).unwrap();
        registry.register(Box::new(MockSkill::new("skill3", "Third skill"))).unwrap();
        
        let skills = registry.list();
        assert_eq!(skills.len(), 3);
        
        let names: Vec<&str> = skills.iter().map(|s| s.name()).collect();
        assert!(names.contains(&"skill1"));
        assert!(names.contains(&"skill2"));
        assert!(names.contains(&"skill3"));
    }

    #[test]
    fn test_skill_unregistration() {
        let mut registry = SkillRegistry::new();
        let skill = Box::new(MockSkill::new("removable", "Will be removed"));
        
        registry.register(skill).unwrap();
        assert!(registry.get("removable").is_some());
        
        let removed = registry.unregister("removable");
        assert!(removed.is_some());
        assert!(registry.get("removable").is_none());
        
        let not_found = registry.unregister("nonexistent");
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn test_skill_discovery_from_directory() {
        let temp_dir = TempDir::new().unwrap();
        let skills_dir = temp_dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();
        
        // Create a mock skill file
        let skill_file = skills_dir.join("calculator.json");
        let skill_config = json!({
            "name": "calculator",
            "description": "Basic calculator operations",
            "version": "1.0.0"
        });
        std::fs::write(&skill_file, serde_json::to_string_pretty(&skill_config).unwrap()).unwrap();
        
        let registry = SkillRegistry::from_directory(&skills_dir).await.unwrap();
        let discovered_skills = registry.discover_skills().await.unwrap();
        
        assert!(!discovered_skills.is_empty());
        assert!(discovered_skills.iter().any(|info| info.name == "calculator"));
    }

    #[test]
    fn test_builtin_skills_registration() {
        let registry = SkillRegistry::with_builtins();
        let skills = registry.list();
        
        // Should have at least some builtin skills
        assert!(!skills.is_empty());
        
        // Check for expected builtin skills
        let names: Vec<&str> = skills.iter().map(|s| s.name()).collect();
        assert!(names.contains(&"calculator"));
    }

    #[tokio::test]
    async fn test_skill_execution_through_registry() {
        let mut registry = SkillRegistry::new();
        let skill = Box::new(MockSkill::new("executor", "Test execution"));
        
        registry.register(skill).unwrap();
        
        let result = registry.execute_skill("executor", json!({})).await.unwrap();
        assert_eq!(result.output, "Executed executor");
    }

    #[tokio::test]
    async fn test_skill_execution_not_found() {
        let registry = SkillRegistry::new();
        
        let result = registry.execute_skill("nonexistent", json!({})).await;
        assert!(result.is_err());
        
        match result.unwrap_err() {
            SkillError::NotFound => {}, // Expected
            _ => panic!("Expected NotFound error"),
        }
    }
}
