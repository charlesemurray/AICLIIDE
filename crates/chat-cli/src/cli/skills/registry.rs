use std::collections::HashMap;
use std::path::Path;

use serde::{
    Deserialize,
    Serialize,
};
use tracing::warn;

use super::{
    Skill,
    SkillError,
    SkillResult,
};
use crate::cli::chat::tools::ToolSpec;
use crate::cli::skills::builtin::PlaceholderSkill;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInfo {
    pub name: String,
    pub description: String,
    pub version: String,
}

pub struct SkillRegistry {
    skills: HashMap<String, Box<dyn Skill>>,
    aliases: HashMap<String, String>, // alias -> skill_name mapping
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
            aliases: HashMap::new(),
        }
    }

    pub fn with_builtins() -> Self {
        let mut registry = Self::new();
        registry.register_builtins();
        registry
    }

    pub async fn with_all_skills(workspace_path: &Path) -> Result<Self, SkillError> {
        let mut registry = Self::with_builtins();

        // Load global skills first
        registry.load_global_skills().await?;

        // Load workspace skills (can override global)
        registry.load_workspace_skills(workspace_path).await?;

        Ok(registry)
    }

    pub async fn with_workspace_skills(workspace_path: &Path) -> Result<Self, SkillError> {
        let mut registry = Self::with_builtins();
        registry.load_workspace_skills(workspace_path).await?;
        Ok(registry)
    }

    pub async fn from_directory(path: &Path) -> Result<Self, SkillError> {
        let mut registry = Self::new();
        registry.load_from_directory(path).await?;
        Ok(registry)
    }

    pub fn register(&mut self, skill: Box<dyn Skill>) -> Result<(), SkillError> {
        let name = skill.name().to_string();

        if self.skills.contains_key(&name) {
            return Err(SkillError::InvalidInput(format!(
                "Skill '{}' is already registered",
                name
            )));
        }

        self.skills.insert(name, skill);
        Ok(())
    }

    pub fn register_override(&mut self, skill: Box<dyn Skill>) -> Result<(), SkillError> {
        let name = skill.name().to_string();
        self.skills.insert(name, skill);
        Ok(())
    }

    pub fn register_with_aliases(&mut self, skill: Box<dyn Skill>) -> Result<(), SkillError> {
        let name = skill.name().to_string();
        let aliases = skill.aliases();

        // Check for conflicts
        if self.skills.contains_key(&name) {
            return Err(SkillError::InvalidInput(format!(
                "Skill '{}' is already registered",
                name
            )));
        }

        for alias in &aliases {
            if self.skills.contains_key(alias) || self.aliases.contains_key(alias) {
                return Err(SkillError::InvalidInput(format!(
                    "Alias '{}' conflicts with existing skill or alias",
                    alias
                )));
            }
        }

        // Register skill
        self.skills.insert(name.clone(), skill);

        // Register aliases
        for alias in aliases {
            self.aliases.insert(alias, name.clone());
        }

        Ok(())
    }

    pub fn unregister(&mut self, name: &str) -> Option<Box<dyn Skill>> {
        if let Some(skill) = self.skills.remove(name) {
            // Remove associated aliases
            let aliases_to_remove: Vec<String> = self
                .aliases
                .iter()
                .filter(|(_, skill_name)| *skill_name == name)
                .map(|(alias, _)| alias.clone())
                .collect();

            for alias in aliases_to_remove {
                self.aliases.remove(&alias);
            }

            Some(skill)
        } else {
            None
        }
    }

    pub fn get(&self, name: &str) -> Option<&dyn Skill> {
        // Try direct skill lookup first
        if let Some(skill) = self.skills.get(name) {
            return Some(skill.as_ref());
        }

        // Try alias lookup
        if let Some(skill_name) = self.aliases.get(name) {
            return self.skills.get(skill_name).map(|s| s.as_ref());
        }

        None
    }

    pub fn get_aliases(&self, skill_name: &str) -> Vec<String> {
        self.aliases
            .iter()
            .filter(|(_, name)| *name == skill_name)
            .map(|(alias, _)| alias.clone())
            .collect()
    }

    pub fn list(&self) -> Vec<&dyn Skill> {
        self.skills.values().map(|s| s.as_ref()).collect()
    }

    pub async fn execute_skill(&self, name: &str, params: serde_json::Value) -> Result<SkillResult, SkillError> {
        match self.get(name) {
            Some(skill) => skill.execute(params).await,
            None => Err(SkillError::NotFound(name.to_string())),
        }
    }

    pub async fn discover_skills(&self) -> Result<Vec<SkillInfo>, SkillError> {
        let mut discovered = Vec::new();

        for skill in self.list() {
            discovered.push(SkillInfo {
                name: skill.name().to_string(),
                description: skill.description().to_string(),
                version: "1.0.0".to_string(), // Default version
            });
        }

        Ok(discovered)
    }

    pub fn discover_skills_in_locations(locations: &[&Path]) -> Vec<SkillInfo> {
        let mut discovered = Vec::new();

        for location in locations {
            if let Ok(entries) = std::fs::read_dir(location) {
                for entry in entries.flatten() {
                    if let Some(name) = entry.file_name().to_str() {
                        if name.ends_with(".json") {
                            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                                if let Ok(info) = serde_json::from_str::<SkillInfo>(&content) {
                                    discovered.push(info);
                                }
                            }
                        }
                    }
                }
            }
        }

        discovered
    }

    pub async fn reload_workspace_skills(&mut self, workspace_path: &Path) -> Result<(), SkillError> {
        // Remove existing workspace skills (keep builtins)
        let builtin_names = vec!["calculator"]; // Add more as needed
        let skills_to_remove: Vec<String> = self
            .skills
            .keys()
            .filter(|name| !builtin_names.contains(&name.as_str()))
            .cloned()
            .collect();

        for skill_name in skills_to_remove {
            self.unregister(&skill_name);
        }

        // Reload workspace skills
        self.load_workspace_skills(workspace_path).await
    }

    async fn load_global_skills(&mut self) -> Result<(), SkillError> {
        // Try to find global skills directory
        let global_dirs = [
            dirs::config_dir().map(|d| d.join("q-cli").join("skills")),
            dirs::home_dir().map(|d| d.join(".q-skills")),
            Some(std::path::PathBuf::from("/usr/local/share/q-cli/skills")),
        ];

        for global_dir in global_dirs.into_iter().flatten() {
            if global_dir.exists() {
                self.load_from_directory(&global_dir).await?;
                break; // Use first available global directory
            }
        }

        Ok(())
    }

    async fn load_workspace_skills(&mut self, workspace_path: &Path) -> Result<(), SkillError> {
        // Load from .q-skills directory (JSON skills)
        let skills_dir = workspace_path.join(".q-skills");
        if skills_dir.exists() {
            self.load_from_directory(&skills_dir).await?;
        }

        // Load .rs files from workspace directory
        self.load_rust_skills_from_directory(workspace_path).await?;

        Ok(())
    }

    async fn load_from_directory(&mut self, path: &Path) -> Result<(), SkillError> {
        if !path.exists() {
            return Ok(());
        }

        let entries = std::fs::read_dir(path).map_err(|e| SkillError::Io(e))?;

        for entry in entries {
            let entry = entry.map_err(|e| SkillError::Io(e))?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let content = std::fs::read_to_string(&path).map_err(|e| {
                    tracing::error!("Failed to read skill file {}: {}", path.display(), e);
                    SkillError::Io(e)
                })?;

                tracing::debug!("Loading skill from: {}", path.display());

                // Parse as enhanced JSON skill directly
                match serde_json::from_str::<crate::cli::skills::types::JsonSkill>(&content) {
                    Ok(enhanced_skill) => {
                        tracing::info!("Successfully parsed skill: {}", enhanced_skill.name);
                        
                        // Create SkillInfo from the enhanced skill
                        let skill_info = SkillInfo {
                            name: enhanced_skill.name.clone(),
                            description: enhanced_skill
                                .description
                                .clone()
                                .unwrap_or_else(|| format!("A {} skill", enhanced_skill.name)),
                            version: "1.0.0".to_string(),
                        };

                        match crate::cli::skills::builtin::JsonSkill::new(skill_info, content) {
                            Ok(json_skill) => {
                                match self.register_override(Box::new(json_skill)) {
                                    Ok(_) => {
                                        tracing::info!("Successfully registered skill: {}", enhanced_skill.name);
                                    },
                                    Err(e) => {
                                        tracing::error!("Failed to register skill {}: {}", enhanced_skill.name, e);
                                    }
                                }
                            },
                            Err(e) => {
                                tracing::error!("Failed to create JsonSkill for {}: {}", enhanced_skill.name, e);
                            }
                        }
                    },
                    Err(e) => {
                        tracing::warn!("Failed to parse skill file {}: {}", path.display(), e);
                        tracing::debug!("File content: {}", content);
                    }
                }
            }
        }

        Ok(())
    }

    async fn load_rust_skills_from_directory(&mut self, path: &Path) -> Result<(), SkillError> {
        if !path.exists() {
            return Ok(());
        }

        let entries = std::fs::read_dir(path).map_err(|e| SkillError::Io(e))?;

        for entry in entries {
            let entry = entry.map_err(|e| SkillError::Io(e))?;
            let file_path = entry.path();

            if file_path.extension().and_then(|s| s.to_str()) == Some("rs") {
                if let Some(file_name) = file_path.file_stem().and_then(|s| s.to_str()) {
                    // Extract skill name from filename (remove -skill suffix if present)
                    let skill_name = file_name.strip_suffix("-skill").unwrap_or(file_name);

                    // Create a placeholder skill for the .rs file
                    let placeholder = PlaceholderSkill::new(skill_name.to_string());
                    let _ = self.register_override(Box::new(placeholder));
                }
            }
        }

        Ok(())
    }

    fn register_builtins(&mut self) {
        // Register builtin calculator skill with aliases
        if let Ok(calculator) = crate::cli::skills::builtin::calculator::Calculator::new() {
            let _ = self.register_with_aliases(Box::new(calculator));
        }
    }

    pub fn get_all_toolspecs(&self) -> Vec<ToolSpec> {
        self.skills
            .values()
            .filter_map(|skill| {
                skill
                    .to_toolspec()
                    .map_err(|e| {
                        warn!("Failed to convert skill {} to toolspec: {}", skill.name(), e);
                        e
                    })
                    .ok()
            })
            .collect()
    }

    pub fn get_toolspec(&self, name: &str) -> Option<ToolSpec> {
        self.get(name).and_then(|skill| skill.to_toolspec().ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_registry_exports_toolspecs() {
        let registry = SkillRegistry::with_builtins();
        let toolspecs = registry.get_all_toolspecs();

        // Should have at least the calculator skill
        assert!(!toolspecs.is_empty(), "Registry should export at least one toolspec");
    }

    #[tokio::test]
    async fn test_get_specific_toolspec() {
        let registry = SkillRegistry::with_builtins();

        // Calculator should be available
        let toolspec = registry.get_toolspec("calculator");
        assert!(toolspec.is_some(), "Calculator toolspec should be available");

        if let Some(ts) = toolspec {
            assert_eq!(ts.name, "calculator");
        }
    }

    #[tokio::test]
    async fn test_nonexistent_toolspec() {
        let registry = SkillRegistry::with_builtins();
        let toolspec = registry.get_toolspec("nonexistent-skill");
        assert!(toolspec.is_none(), "Nonexistent skill should return None");
    }
}
