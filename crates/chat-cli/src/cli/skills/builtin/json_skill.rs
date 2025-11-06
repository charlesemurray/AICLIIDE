use std::collections::HashMap;

use async_trait::async_trait;

use crate::cli::chat::tools::ToolSpec;
use crate::cli::skills::registry::SkillInfo;
use crate::cli::skills::toolspec_conversion::{
    ConversionError,
    ToToolSpec,
};
use crate::cli::skills::types::{
    JsonSkill as EnhancedJsonSkill,
    SkillType,
};
use crate::cli::skills::{
    ResourceLimits,
    Result,
    Skill,
    SkillError,
    SkillResult,
    SkillUI,
    UIElement,
    execute_with_timeout,
};

pub struct JsonSkill {
    info: SkillInfo,
    enhanced_skill: EnhancedJsonSkill,
    limits: ResourceLimits,
}

impl JsonSkill {
    pub fn new(info: SkillInfo, config: String) -> std::result::Result<Self, SkillError> {
        let enhanced_skill = serde_json::from_str::<EnhancedJsonSkill>(&config)
            .map_err(|e| SkillError::InvalidConfiguration(format!("Failed to parse enhanced JSON skill: {}", e)))?;

        // Extract resource limits from security config
        let limits = enhanced_skill
            .security
            .as_ref()
            .and_then(|s| s.resource_limits.as_ref())
            .map(|rl| crate::cli::skills::ResourceLimits {
                timeout_seconds: rl.max_execution_time.unwrap_or(30) as u64,
                max_memory_mb: rl.max_memory_mb.map(|m| m as u64),
                max_cpu_percent: rl.max_cpu_percent.map(|c| c as u64),
            })
            .unwrap_or_default();

        Ok(Self {
            info,
            enhanced_skill,
            limits,
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

    async fn execute(&self, params: serde_json::Value) -> Result<SkillResult> {
        let execution_future = async {
            // Validate parameters first
            if let Some(param_defs) = &self.enhanced_skill.parameters {
                crate::cli::skills::validation::SkillValidator::validate_parameters(&params, param_defs)?;
            }

            // Convert JSON params to HashMap<String, String>
            let param_map = if let serde_json::Value::Object(obj) = params {
                obj.into_iter()
                    .map(|(k, v)| (k, v.as_str().unwrap_or("").to_string()))
                    .collect()
            } else {
                HashMap::new()
            };

            // Execute using enhanced skill
            let output = self
                .enhanced_skill
                .execute(param_map)
                .await
                .map_err(|e| SkillError::ExecutionFailed(e))?;

            Ok(SkillResult {
                output,
                ui_updates: None,
                state_changes: None,
                create_session: None,
                switch_to_session: None,
                close_session: None,
            })
        };

        execute_with_timeout(execution_future, &self.limits).await
    }

    async fn render_ui(&self) -> Result<SkillUI> {
        let skill_type_desc = match self.enhanced_skill.skill_type {
            SkillType::Command => "Command",
            SkillType::CodeInline => "Code Inline",
            SkillType::CodeSession => "Code Session",
            SkillType::Conversation => "Conversation",
            SkillType::PromptInline => "Prompt Inline",
        };

        let mut elements = vec![
            UIElement::Text(format!("{}: {}", self.info.name, self.info.description)),
            UIElement::Text(format!("Type: {}", skill_type_desc)),
        ];

        if let Some(command) = &self.enhanced_skill.command {
            elements.push(UIElement::Text(format!("Command: {}", command)));
        }

        if let Some(security) = &self.enhanced_skill.security {
            elements.push(UIElement::Text("Security: Configured".to_string()));
        }

        Ok(SkillUI {
            elements,
            interactive: false,
        })
    }

    fn to_toolspec(&self) -> std::result::Result<ToolSpec, ConversionError> {
        self.enhanced_skill.to_toolspec()
    }
}
