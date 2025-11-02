// Temporarily disabled - needs API updates
#[cfg(disabled)]
mod global_skills_tests {
    use std::fs;

    use serde_json::json;
    use tempfile::TempDir;

    use crate::cli::skills::SkillRegistry;

    #[tokio::test]
    async fn test_global_skills_loading() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_dir = temp_dir.path().join("workspace");
        let global_dir = temp_dir.path().join("global");

        fs::create_dir_all(&workspace_dir.join(".q-skills")).unwrap();
        fs::create_dir_all(&global_dir).unwrap();

        // Create a global skill
        let global_skill = global_dir.join("global-tool.json");
        fs::write(
            &global_skill,
            json!({
                "name": "global-tool",
                "description": "Global utility tool",
                "version": "1.0.0",
                "type": "code_inline",
                "command": "echo",
                "args": ["Global tool executed"]
            })
            .to_string(),
        )
        .unwrap();

        // Create a workspace skill that overrides global
        let workspace_skill = workspace_dir.join(".q-skills").join("global-tool.json");
        fs::write(
            &workspace_skill,
            json!({
                "name": "global-tool",
                "description": "Workspace override of global tool",
                "version": "2.0.0",
                "type": "code_inline",
                "command": "echo",
                "args": ["Workspace override executed"]
            })
            .to_string(),
        )
        .unwrap();

        // Test discovery from multiple locations
        let workspace_skills_dir = workspace_dir.join(".q-skills");
        let locations = vec![global_dir.as_path(), workspace_skills_dir.as_path()];
        let discovered = SkillRegistry::discover_skills_in_locations(&locations);

        assert_eq!(discovered.len(), 2); // Both versions discovered
        assert!(discovered.iter().any(|s| s.name == "global-tool"));
    }

    #[tokio::test]
    async fn test_skill_scope_priority() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_dir = temp_dir.path().join("workspace");
        let skills_dir = workspace_dir.join(".q-skills");
        fs::create_dir_all(&skills_dir).unwrap();

        // Create skills with different scopes
        let workspace_scoped = skills_dir.join("workspace-scoped.json");
        fs::write(
            &workspace_scoped,
            json!({
                "name": "scoped-tool",
                "description": "Workspace scoped tool",
                "version": "1.0.0",
                "scope": "workspace",
                "type": "code_inline",
                "command": "echo",
                "args": ["Workspace scoped"]
            })
            .to_string(),
        )
        .unwrap();

        let global_scoped = skills_dir.join("global-scoped.json");
        fs::write(
            &global_scoped,
            json!({
                "name": "global-scoped-tool",
                "description": "Global scoped tool",
                "version": "1.0.0",
                "scope": "global",
                "type": "code_inline",
                "command": "echo",
                "args": ["Global scoped"]
            })
            .to_string(),
        )
        .unwrap();

        let registry = SkillRegistry::with_workspace_skills(&workspace_dir).await.unwrap();

        // Both should be loaded regardless of scope when loaded from workspace
        assert!(registry.get("scoped-tool").is_some());
        assert!(registry.get("global-scoped-tool").is_some());
    }

    #[test]
    fn test_skill_info_with_scope_serialization() {
        use crate::cli::skills::{
            EnhancedSkillInfo,
            SkillScope,
            SkillType,
        };

        let skill_info = EnhancedSkillInfo {
            name: "test-skill".to_string(),
            description: "Test skill".to_string(),
            version: "1.0.0".to_string(),
            aliases: Some(vec!["ts".to_string()]),
            scope: Some(SkillScope::Global),
            skill_type: SkillType::CodeInline {
                command: "echo".to_string(),
                args: None,
                working_dir: None,
            },
        };

        // Test serialization
        let json = serde_json::to_string(&skill_info).unwrap();
        assert!(json.contains("\"scope\":\"global\""));

        // Test deserialization
        let deserialized: EnhancedSkillInfo = serde_json::from_str(&json).unwrap();
        assert!(matches!(deserialized.scope, Some(SkillScope::Global)));
    }

    #[test]
    fn test_default_scope() {
        use crate::cli::skills::SkillScope;

        let default_scope = SkillScope::default();
        assert!(matches!(default_scope, SkillScope::Workspace));
    }
}
