//! Skill tool implementation

#[derive(Debug, Clone)]
pub struct SkillTool {
    pub name: String,
    pub description: String,
}

impl SkillTool {
    pub fn new(name: String, description: String) -> Self {
        Self { name, description }
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
}
