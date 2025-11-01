use crate::cli::skills::{Skill, SkillResult, SkillError, SkillUI, UIElement, Result, ResourceLimits, execute_with_timeout};
use crate::cli::skills::registry::SkillInfo;
use async_trait::async_trait;

pub struct JsonSkill {
    info: SkillInfo,
    _config: String, // Store the full JSON config for future use
    limits: ResourceLimits,
}

impl JsonSkill {
    pub fn new(info: SkillInfo, config: String) -> std::result::Result<Self, SkillError> {
        Ok(Self {
            info,
            _config: config,
            limits: ResourceLimits::default(),
        })
    }
}

#[async_trait]
impl Skill for JsonSkill {
    fn name(&self) -> &str {
        &self.info.name
    }

    fn description(&self) -> &str {
        &self.info.description
    }

    async fn execute(&self, _params: serde_json::Value) -> Result<SkillResult> {
        let execution_future = async {
            // For now, just return a simple success message
            // In a full implementation, this would execute the command specified in the JSON
            Ok(SkillResult {
                output: format!("Executed {} skill", self.info.name),
                ui_updates: None,
                state_changes: None,
            })
        };
        
        execute_with_timeout(execution_future, &self.limits).await
    }

    async fn render_ui(&self) -> Result<SkillUI> {
        Ok(SkillUI {
            elements: vec![UIElement::Text(format!("{}: {}", self.info.name, self.info.description))],
            interactive: false,
        })
    }
}
