//! Command creation flow - simplest creation type (LOW complexity)

use crate::cli::creation::{
    CreationFlow, CreationConfig, CreationArtifact, CreationType, CreationPhase, PhaseResult,
    CreationMode, TerminalUI, CreationContext, CommandType, CreationError
};
use crate::cli::custom_commands::{CustomCommand, CommandHandler};
use eyre::Result;
use serde::{Serialize, Deserialize};
use std::path::Path;

/// Command creation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandConfig {
    pub name: String,
    pub command: String,
    pub command_type: CommandType,
    pub description: String,
    pub parameters: Vec<CommandParameter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandParameter {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub default_value: Option<String>,
}

impl CreationConfig for CommandConfig {
    fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(CreationError::missing_required_field("name", "my-command").into());
        }
        if self.command.is_empty() {
            return Err(CreationError::missing_required_field("command", "echo hello").into());
        }
        Ok(())
    }

    fn apply_defaults(&mut self) {
        if self.description.is_empty() {
            self.description = format!("Custom command: {}", self.name);
        }
    }

    fn is_complete(&self) -> bool {
        !self.name.is_empty() && !self.command.is_empty()
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

/// Command creation artifact
pub struct CommandArtifact {
    config: CommandConfig,
}

impl CreationArtifact for CommandArtifact {
    fn persist(&self, location: &Path) -> Result<()> {
        std::fs::create_dir_all(location)?;
        
        let handler = match self.config.command_type {
            CommandType::Script => CommandHandler::Script {
                command: self.config.command.clone(),
                args: vec![],
            },
            CommandType::Alias => CommandHandler::Alias {
                target: self.config.command.clone(),
            },
            CommandType::Builtin => CommandHandler::Builtin {
                function_name: self.config.command.clone(),
            },
            CommandType::Executable => CommandHandler::Script {
                command: self.config.command.clone(),
                args: vec![],
            },
        };

        let custom_command = CustomCommand {
            name: self.config.name.clone(),
            description: self.config.description.clone(),
            handler,
            parameters: vec![], // Convert parameters if needed
            created_at: chrono::Utc::now().to_rfc3339(),
            usage_count: 0,
        };

        let file_path = location.join(format!("{}.json", self.config.name));
        let json = serde_json::to_string_pretty(&custom_command)?;
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

/// Command creation flow
pub struct CommandCreationFlow {
    config: CommandConfig,
    mode: CreationMode,
    context: CreationContext,
    current_phase: Option<CreationPhase>,
    ui: Option<Box<dyn TerminalUI>>,
}

impl CommandCreationFlow {
    pub fn new(name: String, mode: CreationMode) -> Result<Self> {
        let current_dir = std::env::current_dir()?;
        let context = CreationContext::new(&current_dir)?;
        
        // Validate name upfront
        let validation = context.validate_name(&name, &CreationType::CustomCommand);
        if !validation.is_valid {
            return Err(CreationError::invalid_name("command", &name).into());
        }

        let mut config = CommandConfig {
            name,
            command: String::new(),
            command_type: CommandType::Script,
            description: String::new(),
            parameters: Vec::new(),
        };

        // Apply smart defaults
        let defaults = context.suggest_defaults(&CreationType::CustomCommand);
        if let Some(cmd_type) = defaults.command_type {
            config.command_type = cmd_type;
        }
        if !defaults.description.is_empty() {
            config.description = defaults.description;
        }

        Ok(Self {
            config,
            mode,
            context,
            current_phase: None,
            ui: None,
        })
    }

    pub fn with_ui(mut self, ui: Box<dyn TerminalUI>) -> Self {
        self.ui = Some(ui);
        self
    }

    fn execute_discovery(&mut self, ui: &mut dyn TerminalUI) -> Result<PhaseResult> {
        ui.show_message(
            &format!("Creating command '{}'", self.config.name),
            crate::cli::creation::SemanticColor::Info
        );

        // STEP 1: Ask for command type first
        let command_type_options = &[
            ("executable", "Run a system program or script"),
            ("alias", "Shortcut to existing command with preset arguments"),
            ("script", "Multi-step script with several commands"),
        ];

        let selected_type = ui.select_option(
            "What type of command do you want to create?",
            command_type_options
        )?;

        // STEP 2: Ask type-specific questions
        match selected_type.as_str() {
            "executable" => {
                self.config.command = ui.prompt_required("Command to execute")?;
                self.config.command_type = CommandType::Executable;
            }
            "alias" => {
                self.config.command = ui.prompt_required("Base command")?;
                let args = ui.prompt_optional("Default arguments", None)?;
                if let Some(args) = args {
                    self.config.command = format!("{} {}", self.config.command, args);
                }
                self.config.command_type = CommandType::Alias;
            }
            "script" => {
                self.config.command = ui.prompt_required("Script commands (one per line or semicolon-separated)")?;
                self.config.command_type = CommandType::Script;
            }
            _ => {
                self.config.command = ui.prompt_required("Command")?;
                self.detect_command_type();
            }
        }

        // STEP 3: Description (for guided/expert modes)
        if matches!(self.mode, CreationMode::Guided | CreationMode::Expert) {
            let default_desc = match selected_type.as_str() {
                "executable" => format!("Executes: {}", self.config.command),
                "alias" => format!("Alias for: {}", self.config.command),
                "script" => format!("Script: {}", self.config.name),
                _ => format!("Command: {}", self.config.name),
            };

            if let Some(desc) = ui.prompt_optional("Description", Some(&default_desc))? {
                self.config.description = desc;
            }
        }

        Ok(PhaseResult::Continue)
    }

    fn execute_basic_config(&mut self, ui: &mut dyn TerminalUI) -> Result<PhaseResult> {
        // Detect and configure parameters
        self.detect_parameters();
        
        if !self.config.parameters.is_empty() {
            ui.show_message(
                &format!("Detected {} parameter(s)", self.config.parameters.len()),
                crate::cli::creation::SemanticColor::Info
            );
            
            for param in &self.config.parameters {
                ui.show_message(
                    &format!("  - {}: {}", param.name, if param.required { "required" } else { "optional" }),
                    crate::cli::creation::SemanticColor::Debug
                );
            }
        }

        Ok(PhaseResult::Continue)
    }

    fn detect_command_type(&mut self) {
        let cmd_lower = self.config.command.to_lowercase();
        
        // Simple heuristics for command type detection
        if cmd_lower.starts_with("alias ") || self.is_simple_alias() {
            self.config.command_type = CommandType::Alias;
        } else if self.is_builtin_function() {
            self.config.command_type = CommandType::Builtin;
        } else {
            self.config.command_type = CommandType::Script;
        }
    }

    fn is_simple_alias(&self) -> bool {
        // Check if it's a simple command alias (no pipes, redirects, etc.)
        let cmd = &self.config.command;
        !cmd.contains('|') && !cmd.contains('>') && !cmd.contains('<') && 
        !cmd.contains('&') && !cmd.contains(';') && cmd.split_whitespace().count() <= 3
    }

    fn is_builtin_function(&self) -> bool {
        // Check against known builtin functions
        let builtins = ["cd", "pwd", "echo", "exit", "help"];
        let first_word = self.config.command.split_whitespace().next().unwrap_or("");
        builtins.contains(&first_word)
    }

    fn detect_parameters(&mut self) {
        // Look for {{param}} patterns in the command
        let re = regex::Regex::new(r"\{\{(\w+)\}\}").unwrap();
        
        for cap in re.captures_iter(&self.config.command) {
            let param_name = cap[1].to_string();
            
            if !self.config.parameters.iter().any(|p| p.name == param_name) {
                self.config.parameters.push(CommandParameter {
                    name: param_name.clone(),
                    description: format!("Parameter: {}", param_name),
                    required: true,
                    default_value: None,
                });
            }
        }
    }
}

impl CreationFlow for CommandCreationFlow {
    type Config = CommandConfig;
    type Artifact = CommandArtifact;

    fn creation_type(&self) -> CreationType {
        CreationType::CustomCommand
    }

    fn execute_phase(&mut self, phase: CreationPhase) -> Result<PhaseResult> {
        self.current_phase = Some(phase.clone());
        
        match phase {
            CreationPhase::Discovery => {
                let mut ui = crate::cli::creation::TerminalUIImpl::new();
                self.execute_discovery(&mut ui)
            }
            CreationPhase::BasicConfig => {
                let mut ui = crate::cli::creation::TerminalUIImpl::new();
                self.execute_basic_config(&mut ui)
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
        Ok(CommandArtifact {
            config: self.config.clone(),
        })
    }

    fn get_config(&self) -> &Self::Config {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::creation::MockTerminalUI;
    use tempfile::TempDir;

    #[test]
    fn test_command_creation_flow_new() {
        let flow = CommandCreationFlow::new("test-cmd".to_string(), CreationMode::Quick);
        assert!(flow.is_ok());
        
        let flow = flow.unwrap();
        assert_eq!(flow.config.name, "test-cmd");
        assert_eq!(flow.creation_type(), CreationType::CustomCommand);
    }

    #[test]
    fn test_command_creation_flow_invalid_name() {
        let flow = CommandCreationFlow::new("invalid name!".to_string(), CreationMode::Quick);
        assert!(flow.is_err());
    }

    #[test]
    fn test_detect_command_type() {
        let mut flow = CommandCreationFlow::new("test".to_string(), CreationMode::Quick).unwrap();
        
        // Test script detection
        flow.config.command = "python script.py".to_string();
        flow.detect_command_type();
        assert_eq!(flow.config.command_type, CommandType::Script);
        
        // Test alias detection
        flow.config.command = "ls -la".to_string();
        flow.detect_command_type();
        assert_eq!(flow.config.command_type, CommandType::Alias);
        
        // Test builtin detection
        flow.config.command = "echo hello".to_string();
        flow.detect_command_type();
        assert_eq!(flow.config.command_type, CommandType::Builtin);
    }

    #[test]
    fn test_detect_parameters() {
        let mut flow = CommandCreationFlow::new("test".to_string(), CreationMode::Quick).unwrap();
        flow.config.command = "echo {{message}} and {{name}}".to_string();
        
        flow.detect_parameters();
        
        assert_eq!(flow.config.parameters.len(), 2);
        assert!(flow.config.parameters.iter().any(|p| p.name == "message"));
        assert!(flow.config.parameters.iter().any(|p| p.name == "name"));
    }

    #[test]
    fn test_command_config_validation() {
        let mut config = CommandConfig {
            name: "test".to_string(),
            command: "echo hello".to_string(),
            command_type: CommandType::Script,
            description: String::new(),
            parameters: Vec::new(),
        };

        assert!(config.validate().is_ok());
        assert!(config.is_complete());

        config.apply_defaults();
        assert!(!config.description.is_empty());

        // Test invalid config
        config.name = String::new();
        assert!(config.validate().is_err());
        assert!(!config.is_complete());
    }

    #[test]
    fn test_command_artifact_persistence() {
        let temp_dir = TempDir::new().unwrap();
        
        let config = CommandConfig {
            name: "test-cmd".to_string(),
            command: "echo hello".to_string(),
            command_type: CommandType::Script,
            description: "Test command".to_string(),
            parameters: Vec::new(),
        };

        let artifact = CommandArtifact { config };
        let result = artifact.persist(temp_dir.path());
        assert!(result.is_ok());

        // Verify file was created
        let file_path = temp_dir.path().join("test-cmd.json");
        assert!(file_path.exists());

        // Verify content
        let content = std::fs::read_to_string(file_path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed["name"], "test-cmd");
    }
}
