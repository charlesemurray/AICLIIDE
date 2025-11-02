//! Skill tool implementation

use eyre::Result;

use crate::cli::agent::{
    Agent,
    PermissionEvalResult,
};
use crate::os::Os;

#[derive(Debug, Clone)]
pub struct SkillTool {
    pub name: String,
    pub description: String,
}

impl SkillTool {
    pub fn new(name: String, description: String) -> Self {
        Self { name, description }
    }

    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(eyre::eyre!("Skill name cannot be empty"));
        }
        Ok(())
    }

    pub fn eval_perm(&self, _os: &Os, _agent: &Agent) -> PermissionEvalResult {
        PermissionEvalResult::Allow
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_tool_creation() {
        let skill = SkillTool::new("test-skill".to_string(), "A test skill".to_string());
        assert_eq!(skill.name, "test-skill");
        assert_eq!(skill.description, "A test skill");
    }

    #[test]
    fn test_skill_tool_clone() {
        let skill = SkillTool::new("original".to_string(), "Original skill".to_string());
        let cloned = skill.clone();
        assert_eq!(cloned.name, skill.name);
        assert_eq!(cloned.description, skill.description);
    }

    #[test]
    fn test_skill_tool_validate_success() {
        let skill = SkillTool::new("valid-skill".to_string(), "Description".to_string());
        assert!(skill.validate().is_ok());
    }

    #[test]
    fn test_skill_tool_validate_empty_name() {
        let skill = SkillTool::new("".to_string(), "Description".to_string());
        let result = skill.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }

    #[tokio::test]
    async fn test_skill_tool_eval_perm() {
        use crate::cli::agent::{
            Agent,
            PermissionEvalResult,
        };
        use crate::os::Os;

        let skill = SkillTool::new("test-skill".to_string(), "Test".to_string());
        let os = Os::new().await.unwrap();
        let agent = Agent::default();

        let result = skill.eval_perm(&os, &agent);
        assert_eq!(result, PermissionEvalResult::Allow);
    }
}
