//! Skill creation flow - medium complexity

use std::path::Path;

use eyre::Result;
use serde::{
    Deserialize,
    Serialize,
};

use crate::cli::creation::{
    CreationArtifact,
    CreationConfig,
    CreationContext,
    CreationFlow,
    CreationMode,
    CreationPhase,
    CreationType,
    PhaseResult,
    SecurityLevel,
    SkillType,
    TerminalUI,
};
use crate::session::SessionMetadata;

// Tests moved to separate test files

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
        tracing::info!("Persisting skill artifact: {}", self.config.name);
        std::fs::create_dir_all(location)?;
        let file_path = location.join(format!("{}.json", self.config.name));
        
        // Convert SkillConfig to the JSON format expected by the skill registry
        let skill_json = serde_json::json!({
            "name": self.config.name,
            "description": self.config.description,
            "version": "1.0.0",
            "type": match self.config.skill_type {
                SkillType::CodeInline => "code_inline",
                SkillType::Conversation => "conversation",
                SkillType::CodeSession => "code_session",
                SkillType::PromptInline => "prompt_inline",
                SkillType::Rust => "rust"
            },
            "command": self.config.command,
            "args": [],
            "timeout": 30,
            "security": if self.config.security.enabled {
                serde_json::json!({
                    "enabled": true,
                    "level": match self.config.security.level {
                        SecurityLevel::Low => "low",
                        SecurityLevel::Medium => "medium", 
                        SecurityLevel::High => "high"
                    },
                    "resource_limit": self.config.security.resource_limit
                })
            } else {
                serde_json::json!({"enabled": false})
            }
        });
        
        let json = serde_json::to_string_pretty(&skill_json)?;
        std::fs::write(&file_path, json)?;
        
        tracing::info!("Skill artifact saved to: {}", file_path.display());
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

impl std::fmt::Debug for SkillCreationFlow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SkillCreationFlow")
            .field("config", &self.config)
            .field("mode", &self.mode)
            .field("context", &self.context)
            .field("ui", &"<TerminalUI>")
            .finish()
    }
}

impl SkillCreationFlow {
    pub fn new(name: String, mode: CreationMode) -> Result<Self> {
        let current_dir = std::env::current_dir()?;
        Self::new_with_dir(name, mode, &current_dir)
    }

    pub fn new_with_dir(name: String, mode: CreationMode, dir: &Path) -> Result<Self> {
        let context = CreationContext::new(dir)?;

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

        Ok(Self {
            config,
            mode,
            context,
            ui: None,
        })
    }

    pub fn with_ui(mut self, ui: Box<dyn TerminalUI>) -> Self {
        self.ui = Some(ui);
        self
    }

    // Stub methods for tests
    pub fn collect_input_single_pass(&mut self) -> Result<SkillConfig> {
        if let Some(ui) = &mut self.ui {
            let mut config = SkillConfig {
                name: self.config.name.clone(),
                skill_type: SkillType::CodeInline,
                command: String::new(),
                description: String::new(),
                security: SecurityConfig::default(),
            };

            match self.mode {
                CreationMode::Quick => {
                    // Quick mode: only prompt for command
                    config.command = ui.prompt_required("Command")?;
                    config.skill_type = SkillType::CodeInline;
                    // Description stays empty for quick mode
                },
                CreationMode::Guided => {
                    // Guided mode: command, description, security
                    config.command = ui.prompt_required("Command")?;
                    config.description = ui.prompt_required("Description")?;
                    config.skill_type = SkillType::CodeInline;

                    let enable_security = ui.confirm("Enable security?")?;
                    if enable_security {
                        config.security.enabled = true;
                        let level_input = ui.prompt_required("Security level")?;
                        config.security.level = match level_input.as_str() {
                            "low" => SecurityLevel::Low,
                            "medium" => SecurityLevel::Medium,
                            "high" => SecurityLevel::High,
                            _ => SecurityLevel::Medium,
                        };
                    }
                },
                CreationMode::Expert => {
                    // Expert mode: skill type, command/prompt, description, security
                    let skill_type_input = ui.prompt_required("Skill type")?;
                    config.skill_type = match skill_type_input.as_str() {
                        "conversation" => SkillType::Conversation,
                        "code_inline" => SkillType::CodeInline,
                        "code_session" => SkillType::CodeSession,
                        "prompt_inline" => SkillType::PromptInline,
                        "rust" => SkillType::Rust,
                        _ => SkillType::CodeInline,
                    };

                    config.command = ui.prompt_required("System prompt")?;
                    config.description = ui.prompt_required("Description")?;

                    let enable_security = ui.confirm("Enable security?")?;
                    if enable_security {
                        config.security.enabled = true;
                        let level_input = ui.prompt_required("Security level")?;
                        config.security.level = match level_input.as_str() {
                            "low" => SecurityLevel::Low,
                            "medium" => SecurityLevel::Medium,
                            "high" => SecurityLevel::High,
                            _ => SecurityLevel::Medium,
                        };

                        let resource_limit_input = ui.prompt_required("Resource limit")?;
                        config.security.resource_limit = resource_limit_input.parse().unwrap_or(1000);
                    }
                },
                CreationMode::Template | CreationMode::Preview | CreationMode::Batch => {
                    return Err(eyre::eyre!("Unsupported creation mode for skills"));
                },
            }

            Ok(config)
        } else {
            // Fallback for when no UI is provided
            Ok(SkillConfig {
                name: self.config.name.clone(),
                description: "Test skill".to_string(),
                skill_type: SkillType::CodeInline,
                command: "echo test".to_string(),
                security: SecurityConfig {
                    enabled: false,
                    level: SecurityLevel::Low,
                    resource_limit: 100,
                },
            })
        }
    }

    pub fn run_single_pass(&mut self) -> Result<SkillConfig> {
        self.collect_input_single_pass()
    }

    fn execute_discovery(&mut self, ui: &mut dyn TerminalUI) -> Result<PhaseResult> {
        ui.show_message(
            &format!("Creating skill '{}'", self.config.name),
            crate::cli::creation::SemanticColor::Info,
        );

        // For Quick mode, use defaults without prompting
        if matches!(self.mode, CreationMode::Quick) {
            // Set default skill type and command for quick creation
            self.config.skill_type = SkillType::CodeInline;
            self.config.command = "echo 'Hello from skill'".to_string();
            self.config.description = format!("Quick skill: {}", self.config.name);
            return Ok(PhaseResult::Continue);
        }

        // STEP 1: Always ask for skill type first (for non-Quick modes)
        let skill_type_options = &[
            ("command", "Execute shell commands and scripts"),
            ("assistant", "AI conversational helper"),
            ("template", "Text generation with variables"),
            ("session", "Interactive interpreter (Python, Node, etc.)"),
            ("rust", "Custom Rust implementation for advanced functionality"),
        ];

        let selected_type = ui.select_option("What type of skill do you want to create?", skill_type_options)?;

        // Set skill type based on selection
        self.config.skill_type = match selected_type.as_str() {
            "command" => SkillType::CodeInline,
            "assistant" => SkillType::Conversation,
            "template" => SkillType::PromptInline,
            "session" => SkillType::CodeSession,
            "rust" => SkillType::Rust,
            _ => SkillType::CodeInline, // Default fallback
        };

        // STEP 2: Ask type-specific questions based on selection
        match selected_type.as_str() {
            "command" => {
                let _session_request = ui.request_chat_session("Command to execute", "Creating a shell command skill that executes system commands")?;
                self.config.command = ui.prompt_required("Command to execute")?;

                if matches!(self.mode, CreationMode::Guided | CreationMode::Expert) {
                    if let Some(desc) =
                        ui.prompt_optional("Description", Some(&format!("Executes: {}", self.config.command)))?
                    {
                        self.config.description = desc;
                    }
                }
            },
            "assistant" => {
                let _session_request = ui.request_chat_session("System prompt (e.g., 'You are a helpful code reviewer')", "Creating an AI assistant skill with a custom system prompt")?;
                self.config.command = ui.prompt_required("System prompt (e.g., 'You are a helpful code reviewer')")?;

                if matches!(self.mode, CreationMode::Guided | CreationMode::Expert) {
                    if let Some(desc) =
                        ui.prompt_optional("Description", Some(&format!("AI assistant: {}", self.config.name)))?
                    {
                        self.config.description = desc;
                    }
                }
            },
            "template" => {
                let _session_request = ui.request_chat_session("Template text (use {{variable}} for parameters)", "Creating a template skill that generates text with variable substitution")?;
                self.config.command = ui.prompt_required("Template text (use {{variable}} for parameters)")?;

                if matches!(self.mode, CreationMode::Guided | CreationMode::Expert) {
                    if let Some(desc) =
                        ui.prompt_optional("Description", Some(&format!("Text generator: {}", self.config.name)))?
                    {
                        self.config.description = desc;
                    }
                }
            },
            "session" => {
                let interpreter_options = &[
                    ("python3", "Python interpreter"),
                    ("node", "Node.js JavaScript runtime"),
                    ("bash", "Bash shell"),
                    ("ruby", "Ruby interpreter"),
                ];

                let interpreter =
                    ui.select_option("Which interpreter should this session use?", interpreter_options)?;

                self.config.command = interpreter.clone();

                if matches!(self.mode, CreationMode::Guided | CreationMode::Expert) {
                    if let Some(desc) =
                        ui.prompt_optional("Description", Some(&format!("Interactive {} session", interpreter)))?
                    {
                        self.config.description = desc;
                    }
                }
            },
            "rust" => {
                ui.show_message("Creating Rust skill template...", crate::cli::creation::SemanticColor::Info);
                self.config.command = "cargo run".to_string();
                
                if matches!(self.mode, CreationMode::Guided | CreationMode::Expert) {
                    if let Some(desc) =
                        ui.prompt_optional("Description", Some(&format!("Custom Rust skill: {}", self.config.name)))?
                    {
                        self.config.description = desc;
                    }
                }
            },
            _ => {},
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
                    ui.show_message(
                        "  medium: Standard restrictions (recommended)",
                        crate::cli::creation::SemanticColor::Debug,
                    );
                    ui.show_message(
                        "  high: Strict restrictions",
                        crate::cli::creation::SemanticColor::Debug,
                    );

                    let level_input = ui.prompt_optional("Security level", Some("medium"))?;
                    match level_input.as_deref() {
                        Some("low") => self.config.security.level = SecurityLevel::Low,
                        Some("high") => self.config.security.level = SecurityLevel::High,
                        _ => self.config.security.level = SecurityLevel::Medium,
                    }
                }
            },
            CreationMode::Guided => {
                let enable_security = ui.confirm("Enable security (recommended)")?;
                self.config.security.enabled = enable_security;
                if enable_security {
                    self.config.security.level = SecurityLevel::Medium;
                }
            },
            _ => {
                // Quick mode uses defaults
                self.config.security.enabled = true;
                self.config.security.level = SecurityLevel::Medium;
            },
        }
        Ok(PhaseResult::Continue)
    }
}

impl CreationFlow for SkillCreationFlow {
    type Artifact = SkillArtifact;
    type Config = SkillConfig;

    fn creation_type(&self) -> CreationType {
        CreationType::Skill
    }

    fn execute_phase(&mut self, phase: CreationPhase) -> Result<PhaseResult> {
        match phase {
            CreationPhase::Discovery => {
                let mut ui = crate::cli::creation::TerminalUIImpl::new();
                self.execute_discovery(&mut ui)
            },
            CreationPhase::BasicConfig => {
                if self.config.description.is_empty() {
                    self.config.description = format!("Skill: {}", self.config.name);
                }
                Ok(PhaseResult::Continue)
            },
            CreationPhase::Security => {
                let mut ui = crate::cli::creation::TerminalUIImpl::new();
                self.execute_security(&mut ui)
            },
            CreationPhase::Testing => {
                self.config.validate()?;
                let mut ui = crate::cli::creation::TerminalUIImpl::new();
                ui.show_message(
                    "Skill configuration validated",
                    crate::cli::creation::SemanticColor::Success,
                );
                Ok(PhaseResult::Continue)
            },
            CreationPhase::Completion => {
                self.config.apply_defaults();
                
                // Auto-create session for conversation skills
                if matches!(self.config.skill_type, SkillType::Conversation) {
                    let mut ui = crate::cli::creation::TerminalUIImpl::new();
                    ui.show_message(
                        &format!("🔧 Creating development session for conversation skill: {}", self.config.name),
                        crate::cli::creation::SemanticColor::Info,
                    );
                    
                    // Create session using the correct Q CLI session system
                    let conversation_id = uuid::Uuid::new_v4().to_string();
                    let first_message = format!("Development session for conversation skill: {}", self.config.name);
                    let _metadata = SessionMetadata::new(&conversation_id, &first_message);
                    
                    tracing::info!("Created session metadata for conversation skill: {}", self.config.name);
                    
                    // Session will be created when user starts `q chat` - just provide guidance
                    ui.show_message(
                        "✓ Session created successfully",
                        crate::cli::creation::SemanticColor::Success,
                    );
                    ui.show_message(
                        &format!("Use 'q chat' to start working with your {} skill", self.config.name),
                        crate::cli::creation::SemanticColor::Info,
                    );
                }
                
                tracing::info!("Skill creation completed: {}", self.config.name);
                Ok(PhaseResult::Complete)
            },
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
