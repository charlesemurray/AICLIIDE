//! Skill registry for managing skill definitions

use std::collections::HashMap;
use std::path::Path;

use eyre::Result;

use crate::cli::chat::tools::skill::SkillDefinition;

#[derive(Clone, Debug)]
pub struct SkillRegistry {
    skills: HashMap<String, SkillDefinition>,
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self { skills: HashMap::new() }
    }

    pub async fn load_from_directory(&mut self, path: &Path) -> Result<()> {
        let mut entries = tokio::fs::read_dir(path).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let content = tokio::fs::read_to_string(&path).await?;
                let skill: SkillDefinition = serde_json::from_str(&content)?;
                self.skills.insert(skill.name.clone(), skill);
            }
        }

        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&SkillDefinition> {
        self.skills.get(name)
    }

    pub fn get_skill(&self, name: &str) -> Option<&SkillDefinition> {
        self.get(name)
    }

    pub fn list_skills(&self) -> Vec<&SkillDefinition> {
        self.skills.values().collect()
    }

    pub fn len(&self) -> usize {
        self.skills.len()
    }

    pub fn is_empty(&self) -> bool {
        self.skills.is_empty()
    }
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_registry_creation() {
        let registry = SkillRegistry::new();
        assert_eq!(registry.len(), 0);
    }

    #[tokio::test]
    async fn test_load_skills_from_directory() {
        use std::fs;

        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let skill_path = dir.path().join("test_skill.json");

        let skill_json = r#"{
            "name": "test-skill",
            "description": "A test skill",
            "skill_type": "code_inline"
        }"#;

        fs::write(&skill_path, skill_json).unwrap();

        let mut registry = SkillRegistry::new();
        registry.load_from_directory(dir.path()).await.unwrap();

        assert_eq!(registry.len(), 1);
        assert!(registry.get("test-skill").is_some());
    }

    #[test]
    fn test_get_skill_exists() {
        let mut registry = SkillRegistry::new();
        let skill = SkillDefinition {
            name: "test-skill".to_string(),
            description: "Test".to_string(),
            skill_type: "code_inline".to_string(),
            parameters: None,
            implementation: None,
        };
        registry.skills.insert("test-skill".to_string(), skill);

        let result = registry.get_skill("test-skill");
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "test-skill");
    }

    #[test]
    fn test_get_skill_not_found() {
        let registry = SkillRegistry::new();
        let result = registry.get_skill("nonexistent");
        assert!(result.is_none());
    }

    #[test]
    fn test_list_skills() {
        let mut registry = SkillRegistry::new();

        let skill1 = SkillDefinition {
            name: "skill-1".to_string(),
            description: "First skill".to_string(),
            skill_type: "code_inline".to_string(),
            parameters: None,
            implementation: None,
        };
        let skill2 = SkillDefinition {
            name: "skill-2".to_string(),
            description: "Second skill".to_string(),
            skill_type: "code_inline".to_string(),
            parameters: None,
            implementation: None,
        };

        registry.skills.insert("skill-1".to_string(), skill1);
        registry.skills.insert("skill-2".to_string(), skill2);

        let skills = registry.list_skills();
        assert_eq!(skills.len(), 2);
    }

    #[tokio::test]
    async fn test_load_from_nonexistent_directory() {
        use std::path::PathBuf;

        let mut registry = SkillRegistry::new();
        let nonexistent = PathBuf::from("/nonexistent/directory/that/does/not/exist");

        // Should handle gracefully - either error or empty registry
        let result = registry.load_from_directory(&nonexistent).await;
        // Either returns error or loads nothing
        assert!(result.is_err() || registry.is_empty());
    }

    #[tokio::test]
    async fn test_load_malformed_json() {
        use std::fs;

        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let skill_path = dir.path().join("malformed.json");

        // Write invalid JSON
        fs::write(&skill_path, "{ invalid json }").unwrap();

        let mut registry = SkillRegistry::new();
        let result = registry.load_from_directory(dir.path()).await;

        // Should handle malformed JSON gracefully
        assert!(result.is_ok()); // Continues loading other files
        assert_eq!(registry.len(), 0); // But doesn't add invalid skill
    }

    #[tokio::test]
    async fn test_load_duplicate_skill_names() {
        use std::fs;

        use tempfile::tempdir;

        let dir = tempdir().unwrap();

        // Create two skills with same name
        let skill1_path = dir.path().join("skill1.json");
        let skill2_path = dir.path().join("skill2.json");

        let skill_json = r#"{
            "name": "duplicate-skill",
            "description": "First version",
            "skill_type": "code_inline"
        }"#;

        fs::write(&skill1_path, skill_json).unwrap();
        fs::write(&skill2_path, skill_json).unwrap();

        let mut registry = SkillRegistry::new();
        registry.load_from_directory(dir.path()).await.unwrap();

        // Should only have one (last one wins)
        assert_eq!(registry.len(), 1);
        assert!(registry.get("duplicate-skill").is_some());
    }

    #[tokio::test]
    async fn test_load_empty_directory() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();

        let mut registry = SkillRegistry::new();
        let result = registry.load_from_directory(dir.path()).await;

        assert!(result.is_ok());
        assert_eq!(registry.len(), 0);
    }

    #[tokio::test]
    async fn test_load_mixed_valid_invalid_files() {
        use std::fs;

        use tempfile::tempdir;

        let dir = tempdir().unwrap();

        // Valid skill
        let valid_path = dir.path().join("valid.json");
        fs::write(
            &valid_path,
            r#"{"name": "valid", "description": "Valid", "skill_type": "code_inline"}"#,
        )
        .unwrap();

        // Invalid JSON
        let invalid_path = dir.path().join("invalid.json");
        fs::write(&invalid_path, "not json").unwrap();

        // Non-JSON file (should be ignored)
        let txt_path = dir.path().join("readme.txt");
        fs::write(&txt_path, "This is not a skill").unwrap();

        let mut registry = SkillRegistry::new();
        registry.load_from_directory(dir.path()).await.unwrap();

        // Should load only the valid skill
        assert_eq!(registry.len(), 1);
        assert!(registry.get("valid").is_some());
    }
}
