use async_trait::async_trait;
use serde_json::{json, Value};
use crate::cli::skills::{Skill, SkillResult, SkillUI, UIElement, Result};

pub struct Test-skill2Skill;

#[async_trait]
impl Skill for Test-skill2Skill {
    fn name(&self) -> &str {
        "test-skill2"
    }

    fn description(&self) -> &str {
        "A custom skill"
    }

    async fn execute(&self, params: Value) -> Result<SkillResult> {
        Ok(SkillResult {
            output: "Hello from test-skill2!".to_string(),
            ui_updates: None,
            state_changes: None,
        })
    }

    async fn render_ui(&self) -> Result<SkillUI> {
        Ok(SkillUI {
            elements: vec![UIElement::Text("Custom skill UI".to_string())],
            interactive: false,
        })
    }
}
