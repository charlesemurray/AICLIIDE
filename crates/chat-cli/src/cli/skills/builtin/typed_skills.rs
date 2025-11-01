use crate::cli::skills::{Skill, SkillResult, SkillError, SkillUI, UIElement, Result, ResourceLimits, execute_with_timeout};
use crate::cli::skills::types::{EnhancedSkillInfo, SkillType};
use async_trait::async_trait;
use std::process::Command;

pub struct TypedSkill {
    info: EnhancedSkillInfo,
    limits: ResourceLimits,
}

impl TypedSkill {
    pub fn new(info: EnhancedSkillInfo) -> std::result::Result<Self, SkillError> {
        Ok(Self { 
            info,
            limits: ResourceLimits::default(),
        })
    }
}

#[async_trait]
impl Skill for TypedSkill {
    fn name(&self) -> &str {
        &self.info.name
    }

    fn description(&self) -> &str {
        &self.info.description
    }

    fn aliases(&self) -> Vec<String> {
        self.info.aliases.clone().unwrap_or_default()
    }

    async fn execute(&self, params: serde_json::Value) -> Result<SkillResult> {
        let execution_future = async {
            match &self.info.skill_type {
                SkillType::CodeInline { command, args, working_dir } => {
                    self.execute_code_inline(command, args.as_ref(), working_dir.as_ref()).await
                }
                SkillType::CodeSession { command, args, working_dir, .. } => {
                    self.execute_code_session(command, args.as_ref(), working_dir.as_ref()).await
                }
                SkillType::Conversation { prompt_template, .. } => {
                    self.execute_conversation(prompt_template, &params).await
                }
                SkillType::PromptInline { prompt, .. } => {
                    self.execute_prompt_inline(prompt, &params).await
                }
            }
        };
        
        execute_with_timeout(execution_future, &self.limits).await
    }

    async fn render_ui(&self) -> Result<SkillUI> {
        let mut elements = vec![
            UIElement::Text(format!("{}: {}", self.info.name, self.info.description))
        ];

        // Add type-specific UI elements
        match &self.info.skill_type {
            SkillType::PromptInline { parameters: Some(params), .. } => {
                for param in params {
                    elements.push(UIElement::Input {
                        id: param.name.clone(),
                        placeholder: param.description.clone(),
                    });
                }
            }
            _ => {}
        }

        Ok(SkillUI {
            elements,
            interactive: matches!(self.info.skill_type, SkillType::PromptInline { .. }),
        })
    }
}

impl TypedSkill {
    async fn execute_code_inline(&self, command: &str, args: Option<&Vec<String>>, working_dir: Option<&String>) -> Result<SkillResult> {
        let mut cmd = Command::new(command);
        
        if let Some(args) = args {
            cmd.args(args);
        }
        
        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }

        let output = cmd.output()
            .map_err(|e| SkillError::ExecutionFailed(format!("Failed to execute command: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let result = if output.status.success() {
            stdout.to_string()
        } else {
            format!("Command failed:\n{}", stderr)
        };

        Ok(SkillResult {
            output: result,
            ui_updates: None,
            state_changes: None,
        })
    }

    async fn execute_code_session(&self, command: &str, args: Option<&Vec<String>>, working_dir: Option<&String>) -> Result<SkillResult> {
        // For now, same as code_inline but could be extended for session management
        self.execute_code_inline(command, args, working_dir).await
    }

    async fn execute_conversation(&self, prompt_template: &str, params: &serde_json::Value) -> Result<SkillResult> {
        // Replace parameters in prompt template
        let mut prompt = prompt_template.to_string();
        
        if let Some(obj) = params.as_object() {
            for (key, value) in obj {
                let placeholder = format!("{{{}}}", key);
                if let Some(str_value) = value.as_str() {
                    prompt = prompt.replace(&placeholder, str_value);
                }
            }
        }

        Ok(SkillResult {
            output: format!("Conversation prompt: {}", prompt),
            ui_updates: None,
            state_changes: Some(serde_json::json!({
                "conversation_started": true,
                "prompt": prompt
            })),
        })
    }

    async fn execute_prompt_inline(&self, prompt: &str, params: &serde_json::Value) -> Result<SkillResult> {
        let mut processed_prompt = prompt.to_string();
        
        if let Some(obj) = params.as_object() {
            for (key, value) in obj {
                let placeholder = format!("{{{}}}", key);
                if let Some(str_value) = value.as_str() {
                    processed_prompt = processed_prompt.replace(&placeholder, str_value);
                }
            }
        }

        Ok(SkillResult {
            output: processed_prompt,
            ui_updates: None,
            state_changes: None,
        })
    }
}
