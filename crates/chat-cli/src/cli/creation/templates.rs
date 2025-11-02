//! Template system for creation flows

use std::path::Path;

use eyre::Result;
use serde_json::Value;

use crate::cli::creation::{
    CreationError,
    CreationType,
};

/// Template manager for loading and applying templates
pub struct TemplateManager {
    base_path: std::path::PathBuf,
}

impl TemplateManager {
    pub fn new(base_path: &Path) -> Self {
        Self {
            base_path: base_path.to_path_buf(),
        }
    }

    pub fn load_template(&self, name: &str, creation_type: &CreationType) -> Result<Value> {
        let template_dir = match creation_type {
            CreationType::CustomCommand => self.base_path.join(".q-commands"),
            CreationType::Skill => self.base_path.join(".q-skills"),
            CreationType::Agent => self.base_path.join(".amazonq").join("cli-agents"),
        };

        let template_file = template_dir.join(format!("{}.json", name));

        if !template_file.exists() {
            let available = self.list_available_templates(creation_type)?;
            return Err(CreationError::template_not_found(name, available).into());
        }

        let content = std::fs::read_to_string(template_file)?;
        let template: Value = serde_json::from_str(&content)?;
        Ok(template)
    }

    pub fn list_available_templates(&self, creation_type: &CreationType) -> Result<Vec<String>> {
        let template_dir = match creation_type {
            CreationType::CustomCommand => self.base_path.join(".q-commands"),
            CreationType::Skill => self.base_path.join(".q-skills"),
            CreationType::Agent => self.base_path.join(".amazonq").join("cli-agents"),
        };

        let mut templates = Vec::new();

        if template_dir.exists() {
            for entry in std::fs::read_dir(template_dir)? {
                let entry = entry?;
                if let Some(ext) = entry.path().extension() {
                    if ext == "json" {
                        if let Some(stem) = entry.path().file_stem() {
                            if let Some(name) = stem.to_str() {
                                templates.push(name.to_string());
                            }
                        }
                    }
                }
            }
        }

        Ok(templates)
    }

    pub fn apply_template_to_config<T>(&self, template: &Value, config: &mut T, new_name: &str) -> Result<()>
    where
        T: serde::de::DeserializeOwned + serde::Serialize,
    {
        // Clone template and update name
        let mut template_config = template.clone();
        if let Some(obj) = template_config.as_object_mut() {
            obj.insert("name".to_string(), Value::String(new_name.to_string()));

            // Remove runtime fields that shouldn't be copied
            obj.remove("created_at");
            obj.remove("usage_count");
        }

        // Deserialize template into config type
        *config = serde_json::from_value(template_config)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn test_template_manager_load() {
        let temp_dir = TempDir::new().unwrap();
        let skills_dir = temp_dir.path().join(".q-skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        // Create template skill
        let template_skill = r#"{
            "name": "template-skill",
            "type": "code_inline",
            "command": "python {{script}}",
            "description": "Python script runner"
        }"#;

        std::fs::write(skills_dir.join("template-skill.json"), template_skill).unwrap();

        let manager = TemplateManager::new(temp_dir.path());
        let template = manager.load_template("template-skill", &CreationType::Skill).unwrap();

        assert_eq!(template["name"], "template-skill");
        assert_eq!(template["command"], "python {{script}}");
    }

    #[test]
    fn test_template_manager_list() {
        let temp_dir = TempDir::new().unwrap();
        let commands_dir = temp_dir.path().join(".q-commands");
        std::fs::create_dir_all(&commands_dir).unwrap();

        // Create multiple templates
        std::fs::write(commands_dir.join("template1.json"), "{}").unwrap();
        std::fs::write(commands_dir.join("template2.json"), "{}").unwrap();

        let manager = TemplateManager::new(temp_dir.path());
        let templates = manager.list_available_templates(&CreationType::CustomCommand).unwrap();

        assert_eq!(templates.len(), 2);
        assert!(templates.contains(&"template1".to_string()));
        assert!(templates.contains(&"template2".to_string()));
    }

    #[test]
    fn test_template_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let manager = TemplateManager::new(temp_dir.path());

        let result = manager.load_template("nonexistent", &CreationType::Skill);
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.to_string().contains("Template 'nonexistent' not found"));
    }
}
