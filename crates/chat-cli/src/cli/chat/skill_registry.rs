//! Skill registry for managing skill definitions

use std::collections::HashMap;
use std::io::Write;
use std::path::Path;

use eyre::Result;

use crate::cli::chat::tools::skill::SkillDefinition;

#[derive(Clone, Debug)]
pub struct SkillRegistry {
    skills: HashMap<String, SkillDefinition>,
}

#[derive(Debug)]
pub struct LoadingSummary {
    pub loaded: Vec<String>,
    pub failed: Vec<(String, String)>,
}

impl LoadingSummary {
    pub fn new() -> Self {
        Self {
            loaded: Vec::new(),
            failed: Vec::new(),
        }
    }

    pub fn print(&self, output: &mut impl Write) -> Result<()> {
        if !self.loaded.is_empty() {
            for name in &self.loaded {
                writeln!(output, "✓ Loaded skill: {}", name)?;
            }
        }

        if !self.failed.is_empty() {
            for (file, error) in &self.failed {
                writeln!(output, "✗ Failed to load {}: {}", file, error)?;
            }
        }

        if !self.loaded.is_empty() || !self.failed.is_empty() {
            writeln!(
                output,
                "\nLoaded {} skill(s), {} failed",
                self.loaded.len(),
                self.failed.len()
            )?;
        }

        Ok(())
    }
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self { skills: HashMap::new() }
    }

    pub async fn load_from_directory(&mut self, path: &Path) -> Result<()> {
        self.load_from_directory_with_feedback(path, &mut std::io::sink()).await
    }

    pub async fn load_from_directory_with_feedback(&mut self, path: &Path, output: &mut impl Write) -> Result<()> {
        let mut summary = LoadingSummary::new();
        let mut entries = tokio::fs::read_dir(path).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let filename = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                match tokio::fs::read_to_string(&path).await {
                    Ok(content) => match serde_json::from_str::<SkillDefinition>(&content) {
                        Ok(skill) => {
                            let name = skill.name.clone();
                            self.skills.insert(name.clone(), skill);
                            summary.loaded.push(name);
                        },
                        Err(e) => {
                            summary.failed.push((filename, e.to_string()));
                        },
                    },
                    Err(e) => {
                        summary.failed.push((filename, e.to_string()));
                    },
                }
            }
        }

        summary.print(output)?;
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

    pub fn exists(&self, name: &str) -> bool {
        self.skills.contains_key(name)
    }

    pub async fn execute_skill(&self, name: &str, params: serde_json::Value) -> Result<String> {
        use crate::cli::chat::tools::skill::SkillTool;
        
        let skill = self.get(name)
            .ok_or_else(|| eyre::eyre!("Skill '{}' not found", name))?;
        
        let tool = SkillTool::from_definition(skill);
        
        // Convert Value to HashMap<String, Value>
        let param_map = if let serde_json::Value::Object(obj) = params {
            obj.into_iter().collect()
        } else {
            std::collections::HashMap::new()
        };
        
        tool.invoke_direct(param_map)
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

    #[tokio::test]
    async fn test_loading_feedback() {
        use std::fs;

        use tempfile::tempdir;

        let dir = tempdir().unwrap();

        // Valid skill
        let valid_path = dir.path().join("valid.json");
        fs::write(
            &valid_path,
            r#"{"name": "test-skill", "description": "Test", "skill_type": "code_inline"}"#,
        )
        .unwrap();

        // Invalid JSON
        let invalid_path = dir.path().join("invalid.json");
        fs::write(&invalid_path, "not json").unwrap();

        let mut registry = SkillRegistry::new();
        let mut output = Vec::new();
        registry
            .load_from_directory_with_feedback(dir.path(), &mut output)
            .await
            .unwrap();

        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("✓ Loaded skill: test-skill"));
        assert!(output_str.contains("✗ Failed to load invalid.json"));
        assert!(output_str.contains("Loaded 1 skill(s), 1 failed"));
    }
}
