//! Skill creation flow - medium complexity

use crate::cli::creation::{
    CreationFlow, CreationConfig, CreationArtifact, CreationType, CreationPhase, PhaseResult,
    CreationMode, SkillType, SecurityLevel, TerminalUI, CreationContext
};
use eyre::Result;
use serde::{Serialize, Deserialize};
use std::path::Path;

#[cfg(test)]
mod skill_flow_tests;

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

    fn get_name(&self) -> &str {
        &self.name
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

        // STEP 1: Always ask for skill type first
        let skill_type_options = &[
            ("command", "Execute shell commands and scripts"),
            ("assistant", "AI conversational helper"),
            ("template", "Text generation with variables"),
            ("session", "Interactive interpreter (Python, Node, etc.)"),
        ];

        let selected_type = ui.select_option(
            "What type of skill do you want to create?",
            skill_type_options
        )?;

        // Set skill type based on selection
        self.config.skill_type = match selected_type.as_str() {
            "command" => SkillType::CodeInline,
            "assistant" => SkillType::Conversation,
            "template" => SkillType::PromptInline,
            "session" => SkillType::CodeSession,
            _ => SkillType::CodeInline, // Default fallback
        };

        // STEP 2: Ask type-specific questions based on selection
        match selected_type.as_str() {
            "command" => {
                self.config.command = ui.prompt_required("Command to execute")?;
                
                if matches!(self.mode, CreationMode::Guided | CreationMode::Expert) {
                    if let Some(desc) = ui.prompt_optional("Description", Some(&format!("Executes: {}", self.config.command)))? {
                        self.config.description = desc;
                    }
                }
            }
            "assistant" => {
                self.config.command = ui.prompt_required("System prompt (e.g., 'You are a helpful code reviewer')")?;
                
                if matches!(self.mode, CreationMode::Guided | CreationMode::Expert) {
                    if let Some(desc) = ui.prompt_optional("Description", Some(&format!("AI assistant: {}", self.config.name)))? {
                        self.config.description = desc;
                    }
                }
            }
            "template" => {
                self.config.command = ui.prompt_required("Template text (use {{variable}} for parameters)")?;
                
                if matches!(self.mode, CreationMode::Guided | CreationMode::Expert) {
                    if let Some(desc) = ui.prompt_optional("Description", Some(&format!("Text generator: {}", self.config.name)))? {
                        self.config.description = desc;
                    }
                }
            }
            "session" => {
                let interpreter_options = &[
                    ("python3", "Python interpreter"),
                    ("node", "Node.js JavaScript runtime"),
                    ("bash", "Bash shell"),
                    ("ruby", "Ruby interpreter"),
                ];

                let interpreter = ui.select_option(
                    "Which interpreter should this session use?",
                    interpreter_options
                )?;

                self.config.command = interpreter.clone();
                
                if matches!(self.mode, CreationMode::Guided | CreationMode::Expert) {
                    if let Some(desc) = ui.prompt_optional("Description", Some(&format!("Interactive {} session", interpreter)))? {
                        self.config.description = desc;
                    }
                }
            }
            _ => {}
        }

        // STEP 3: Expert mode additional configuration
        if matches!(self.mode, CreationMode::Expert) {
            // Security configuration will be handled in execute_security phase
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
        match phase {
            CreationPhase::Discovery => {
                let mut ui = crate::cli::creation::TerminalUIImpl::new();
                self.execute_discovery(&mut ui)
            }
            CreationPhase::BasicConfig => {
                if self.config.description.is_empty() {
                    self.config.description = format!("Skill: {}", self.config.name);
                }
                Ok(PhaseResult::Continue)
            }
            CreationPhase::Security => {
                let mut ui = crate::cli::creation::TerminalUIImpl::new();
                self.execute_security(&mut ui)
            }
            CreationPhase::Testing => {
                self.config.validate()?;
                let mut ui = crate::cli::creation::TerminalUIImpl::new();
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
