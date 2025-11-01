use async_trait::async_trait;
use serde_json::Value;
use crate::cli::skills::{Skill, SkillResult, SkillError, SkillUI, UIElement};

pub struct PlaceholderSkill {
    name: String,
}

impl PlaceholderSkill {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

#[async_trait]
impl Skill for PlaceholderSkill {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        "Rust skill file found but not compiled"
    }

    fn aliases(&self) -> Vec<String> {
        vec![]
    }

    async fn execute(&self, _params: Value) -> Result<SkillResult, SkillError> {
        Err(SkillError::ExecutionFailed(format!(
            "Skill '{}' is not compiled. Dynamic compilation not yet implemented.", 
            self.name
        )))
    }

    async fn render_ui(&self) -> Result<SkillUI, SkillError> {
        Ok(SkillUI {
            elements: vec![
                UIElement::Text(format!("Skill: {}", self.name)),
                UIElement::Text("Status: Not compiled".to_string()),
                UIElement::Text("Note: Dynamic compilation not yet implemented".to_string()),
            ],
            interactive: false,
        })
    }
}
