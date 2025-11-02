#[cfg(test)]
mod cli_integration_tests {
    use serde_json::json;

    use crate::cli::skills::{SkillError, SkillRegistry};

    #[tokio::test]
    async fn test_cli_skills_list_command() {
        let registry = SkillRegistry::with_builtins();
        let skills = registry.list();

        // Should have at least the calculator skill
        assert!(!skills.is_empty());
        assert!(skills.iter().any(|s| s.name() == "calculator"));
    }

    #[tokio::test]
    async fn test_cli_skills_run_command() {
        let registry = SkillRegistry::with_builtins();

        // Test running calculator skill
        let result = registry
            .execute_skill("calculator", json!({"a": 2, "b": 3, "op": "add"}))
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().output, "5");
    }

    #[tokio::test]
    async fn test_cli_skills_info_command() {
        let registry = SkillRegistry::with_builtins();

        // Test getting skill info
        let skill = registry.get("calculator");
        assert!(skill.is_some());

        let skill = skill.unwrap();
        assert_eq!(skill.name(), "calculator");
        assert!(!skill.description().is_empty());
        assert!(skill.aliases().contains(&"calc".to_string()));
    }

    #[tokio::test]
    async fn test_cli_error_handling() {
        let registry = SkillRegistry::with_builtins();

        // Test non-existent skill
        let result = registry.execute_skill("nonexistent", json!({})).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SkillError::NotFound));
    }
}
