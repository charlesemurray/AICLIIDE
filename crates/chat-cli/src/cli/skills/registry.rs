use super::{Skill, SkillError, SkillResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInfo {
    pub name: String,
    pub description: String,
    pub version: String,
}

pub struct SkillRegistry {
    skills: HashMap<String, Box<dyn Skill>>,
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
        }
    }

    pub fn with_builtins() -> Self {
        let mut registry = Self::new();
        registry.register_builtins();
        registry
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
                "Skill '{}' is already registered", name
            )));
        }
        
        self.skills.insert(name, skill);
        Ok(())
    }

    pub fn unregister(&mut self, name: &str) -> Option<Box<dyn Skill>> {
        self.skills.remove(name)
    }

    pub fn get(&self, name: &str) -> Option<&dyn Skill> {
        self.skills.get(name).map(|s| s.as_ref())
    }

    pub fn list(&self) -> Vec<&dyn Skill> {
        self.skills.values().map(|s| s.as_ref()).collect()
    }

    pub async fn execute_skill(
        &self,
        name: &str,
        params: serde_json::Value,
    ) -> Result<SkillResult, SkillError> {
        match self.get(name) {
            Some(skill) => skill.execute(params).await,
            None => Err(SkillError::NotFound),
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

    async fn load_from_directory(&mut self, _path: &Path) -> Result<(), SkillError> {
        // TODO: Implement directory-based skill loading
        // For now, just register builtins
        self.register_builtins();
        Ok(())
    }

    fn register_builtins(&mut self) {
        // Register builtin calculator skill
        if let Ok(calculator) = crate::cli::skills::builtin::calculator::Calculator::new() {
            let _ = self.register(Box::new(calculator));
        }
    }
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}
