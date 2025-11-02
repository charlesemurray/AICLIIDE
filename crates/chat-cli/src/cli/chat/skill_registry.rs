//! Skill registry for managing skill definitions

use std::collections::HashMap;
use std::path::Path;

use eyre::Result;

use crate::cli::chat::tools::skill::SkillDefinition;

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
}
