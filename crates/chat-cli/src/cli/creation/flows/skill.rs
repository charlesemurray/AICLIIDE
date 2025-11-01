//! Skill creation flow - medium complexity

use crate::cli::creation::{
    CreationFlow, CreationConfig, CreationArtifact, CreationType, CreationPhase, PhaseResult,
    CreationMode, SkillType, SecurityLevel, TerminalUI, CreationContext
};
use eyre::Result;
use serde::{Serialize, Deserialize};
use std::path::Path;

/// Skill creation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillConfig {
    pub name: String,
    pub skill_type: SkillType,
    pub command: String,
    pub description: String,
    pub security: SecurityConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub enabled: bool,
    pub level: SecurityLevel,
    pub resource_limit: u64,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            level: SecurityLevel::Medium,
            resource_limit: 1000,
        }
    }
}

impl CreationConfig for SkillConfig {
    fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(crate::cli::creation::CreationError::missing_required_field("name", "my-skill").into());
        }
        Ok(())
    }

    fn apply_defaults(&mut self) {
        if self.description.is_empty() {
            self.description = format!("Skill: {}", self.name);
        }
    }

    fn is_complete(&self) -> bool {
        !self.name.is_empty() && !self.command.is_empty()
    }
}

/// Skill creation artifact
pub struct SkillArtifact {
    config: SkillConfig,
}

impl CreationArtifact for SkillArtifact {
    fn persist(&self, location: &Path) -> Result<()> {
        std::fs::create_dir_all(location)?;
        let file_path = location.join(format!("{}.json", self.config.name));
        let json = serde_json::to_string_pretty(&self.config)?;
        std::fs::write(file_path, json)?;
        Ok(())
    }

    fn validate_before_save(&self) -> Result<()> {
        self.config.validate()
    }

    fn get_name(&self) -> &str {
        &self.config.name
    }
}

/// Skill creation flow
pub struct SkillCreationFlow {
    config: SkillConfig,
    mode: CreationMode,
    context: CreationContext,
    ui: Option<Box<dyn TerminalUI>>,
}

impl SkillCreationFlow {
    pub fn new(name: String, mode: CreationMode) -> Result<Self> {
        let current_dir = std::env::current_dir()?;
        let context = CreationContext::new(&current_dir)?;
        
        // Validate name upfront
        let validation = context.validate_name(&name, &CreationType::Skill);
        if !validation.is_valid {
            return Err(crate::cli::creation::CreationError::invalid_name("skill", &name).into());
        }

        let mut config = SkillConfig {
            name,
            skill_type: SkillType::CodeInline,
            command: String::new(),
            description: String::new(),
            security: SecurityConfig::default(),
        };

        // Apply smart defaults
        let defaults = context.suggest_defaults(&CreationType::Skill);
        if let Some(skill_type) = defaults.skill_type {
            config.skill_type = skill_type;
        }
        if let Some(command) = defaults.command {
            config.command = command;
        }
        if !defaults.description.is_empty() {
            config.description = defaults.description;
        }

        Ok(Self { config, mode, context, ui: None })
    }

    pub fn with_ui(mut self, ui: Box<dyn TerminalUI>) -> Self {
        self.ui = Some(ui);
        self
    }

    fn execute_discovery(&mut self, ui: &mut dyn TerminalUI) -> Result<PhaseResult> {
        ui.show_message(
            &format!("Creating skill '{}'", self.config.name),
            crate::cli::creation::SemanticColor::Info
        );

        match self.mode {
            CreationMode::Quick => {
                // Use smart defaults, minimal prompts
                if self.config.command.is_empty() {
                    self.config.command = ui.prompt_required("Command")?;
                }
            }
            CreationMode::Guided => {
                // Step-by-step with explanations
                self.config.command = ui.prompt_required("Command to execute")?;
                
                if let Some(desc) = ui.prompt_optional("Description", Some(&self.config.description))? {
                    self.config.description = desc;
                }

                // Skill type selection
                ui.show_message("Skill types:", crate::cli::creation::SemanticColor::Info);
                ui.show_message("  code_inline: Execute shell commands/scripts", crate::cli::creation::SemanticColor::Debug);
                ui.show_message("  conversation: Chat-based assistant", crate::cli::creation::SemanticColor::Debug);
                
                let skill_type_input = ui.prompt_optional("Skill type", Some("code_inline"))?;
                if let Some(st) = skill_type_input {
                    match st.as_str() {
                        "conversation" => self.config.skill_type = SkillType::Conversation,
                        "code_session" => self.config.skill_type = SkillType::CodeSession,
                        "prompt_inline" => self.config.skill_type = SkillType::PromptInline,
                        _ => self.config.skill_type = SkillType::CodeInline,
                    }
                }
            }
            CreationMode::Expert => {
                // Full configuration options
                self.config.command = ui.prompt_required("Command/Prompt")?;
                self.config.description = ui.prompt_required("Description")?;
                
                let skill_type_input = ui.prompt_required("Skill type (code_inline|code_session|conversation|prompt_inline)")?;
                match skill_type_input.as_str() {
                    "code_inline" => self.config.skill_type = SkillType::CodeInline,
                    "code_session" => self.config.skill_type = SkillType::CodeSession,
                    "conversation" => self.config.skill_type = SkillType::Conversation,
                    "prompt_inline" => self.config.skill_type = SkillType::PromptInline,
                    _ => return Err(crate::cli::creation::CreationError::validation_failed(
                        "skill_type", &skill_type_input, "Invalid skill type", "Use: code_inline"
                    ).into()),
                }
            }
            _ => {}
        }
        Ok(PhaseResult::Continue)
    }

    fn execute_security(&mut self, ui: &mut dyn TerminalUI) -> Result<PhaseResult> {
        match self.mode {
            CreationMode::Expert => {
                let enable_security = ui.confirm("Enable security restrictions")?;
                self.config.security.enabled = enable_security;
                
                if enable_security {
                    ui.show_message("Security levels:", crate::cli::creation::SemanticColor::Info);
                    ui.show_message("  low: Basic restrictions", crate::cli::creation::SemanticColor::Debug);
                    ui.show_message("  medium: Standard restrictions (recommended)", crate::cli::creation::SemanticColor::Debug);
                    ui.show_message("  high: Strict restrictions", crate::cli::creation::SemanticColor::Debug);
                    
                    let level_input = ui.prompt_optional("Security level", Some("medium"))?;
                    match level_input.as_deref() {
                        Some("low") => self.config.security.level = SecurityLevel::Low,
                        Some("high") => self.config.security.level = SecurityLevel::High,
                        _ => self.config.security.level = SecurityLevel::Medium,
                    }
                }
            }
            CreationMode::Guided => {
                let enable_security = ui.confirm("Enable security (recommended)")?;
                self.config.security.enabled = enable_security;
                if enable_security {
                    self.config.security.level = SecurityLevel::Medium;
                }
            }
            _ => {
                // Quick mode uses defaults
                self.config.security.enabled = true;
                self.config.security.level = SecurityLevel::Medium;
            }
        }
        Ok(PhaseResult::Continue)
    }
}

impl CreationFlow for SkillCreationFlow {
    type Config = SkillConfig;
    type Artifact = SkillArtifact;

    fn creation_type(&self) -> CreationType {
        CreationType::Skill
    }

    fn execute_phase(&mut self, phase: CreationPhase) -> Result<PhaseResult> {
        // Use injected UI or create default
        let mut default_ui;
        let ui: &mut dyn TerminalUI = if let Some(ref mut ui) = self.ui {
            ui.as_mut()
        } else {
            default_ui = crate::cli::creation::TerminalUIImpl::new();
            &mut default_ui
        };

        match phase {
            CreationPhase::Discovery => self.execute_discovery(ui),
            CreationPhase::BasicConfig => {
                if self.config.description.is_empty() {
                    self.config.description = format!("Skill: {}", self.config.name);
                }
                Ok(PhaseResult::Continue)
            }
            CreationPhase::Security => self.execute_security(ui),
            CreationPhase::Testing => {
                // Validate skill configuration
                self.config.validate()?;
                ui.show_message("Skill configuration validated", crate::cli::creation::SemanticColor::Success);
                Ok(PhaseResult::Continue)
            }
            CreationPhase::Completion => {
                self.config.apply_defaults();
                Ok(PhaseResult::Complete)
            }
            _ => Ok(PhaseResult::Continue),
        }
    }

    fn create_artifact(&self) -> Result<Self::Artifact> {
        self.config.validate()?;
        Ok(SkillArtifact {
            config: self.config.clone(),
        })
    }

    fn get_config(&self) -> &Self::Config {
        &self.config
    }
}
