use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomCommand {
    pub name: String,
    pub description: String,
    pub handler: CommandHandler,
    pub parameters: Vec<CommandParameter>,
    pub created_at: String,
    pub usage_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandHandler {
    Script { command: String, args: Vec<String> },
    Alias { target: String },
    Builtin { function_name: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    #[serde(rename = "string")]
    String,
    #[serde(rename = "number")]
    Number,
    #[serde(rename = "boolean")]
    Boolean,
    #[serde(rename = "enum")]
    Enum,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandParameter {
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: ParameterType, // NEW: Enum type for validation
    pub required: bool,                // KEEP: Existing functionality
    pub default_value: Option<String>, // KEEP: Existing functionality
    pub description: Option<String>,   // CHANGE: Make optional
    pub values: Option<Vec<String>>,   // NEW: For enum validation
    pub pattern: Option<String>,       // NEW: For security validation (regex)
}

#[derive(Debug)]
pub struct CustomCommandRegistry {
    commands: HashMap<String, CustomCommand>,
    commands_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub struct CommandExecution {
    pub command_name: String,
    pub arguments: HashMap<String, String>,
}

#[derive(Debug)]
pub enum CommandError {
    NotFound(String),
    InvalidParameter(String),
    ExecutionFailed(String),
    RegistryError(String),
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandError::NotFound(name) => write!(f, "Command '{}' not found", name),
            CommandError::InvalidParameter(msg) => write!(f, "Invalid parameter: {}", msg),
            CommandError::ExecutionFailed(msg) => write!(f, "Execution failed: {}", msg),
            CommandError::RegistryError(msg) => write!(f, "Registry error: {}", msg),
        }
    }
}

impl std::error::Error for CommandError {}

impl CustomCommand {
    pub fn new_script(name: String, description: String, command: String) -> Self {
        Self {
            name,
            description,
            handler: CommandHandler::Script { command, args: vec![] },
            parameters: vec![],
            created_at: chrono::Utc::now().to_rfc3339(),
            usage_count: 0,
        }
    }

    pub fn new_alias(name: String, description: String, target: String) -> Self {
        Self {
            name,
            description,
            handler: CommandHandler::Alias { target },
            parameters: vec![],
            created_at: chrono::Utc::now().to_rfc3339(),
            usage_count: 0,
        }
    }

    pub fn add_parameter(&mut self, param: CommandParameter) {
        self.parameters.push(param);
    }

    pub fn increment_usage(&mut self) {
        self.usage_count += 1;
    }

    pub fn validate_parameters(&self, args: &HashMap<String, String>) -> Result<(), CommandError> {
        for param in &self.parameters {
            if param.required && !args.contains_key(&param.name) {
                return Err(CommandError::InvalidParameter(format!(
                    "Required parameter '{}' is missing",
                    param.name
                )));
            }

            // Validate parameter value if provided
            if let Some(value) = args.get(&param.name) {
                if let Err(validation_error) = param.validate(value) {
                    return Err(CommandError::InvalidParameter(format!(
                        "Parameter '{}': {}",
                        param.name, validation_error
                    )));
                }
            }
        }
        Ok(())
    }
}

impl CommandParameter {
    pub fn new(name: String, param_type: ParameterType) -> Self {
        Self {
            name,
            param_type,
            required: true,
            default_value: None,
            description: None,
            values: None,
            pattern: None,
        }
    }

    pub fn optional(name: String, param_type: ParameterType, default: Option<String>) -> Self {
        Self {
            name,
            param_type,
            required: false,
            default_value: default,
            description: None,
            values: None,
            pattern: None,
        }
    }

    pub fn enum_param(name: String, values: Vec<String>) -> Self {
        Self {
            name,
            param_type: ParameterType::Enum,
            required: true,
            default_value: None,
            description: None,
            values: Some(values),
            pattern: None,
        }
    }

    pub fn with_pattern(mut self, pattern: String) -> Self {
        self.pattern = Some(pattern);
        self
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn validate(&self, value: &str) -> Result<(), String> {
        // Type validation
        match self.param_type {
            ParameterType::String => {
                // Basic string validation - check for command injection
                if value.contains(';') || value.contains('|') || value.contains('&') {
                    return Err("Invalid characters detected in string parameter".to_string());
                }
            },
            ParameterType::Number => {
                if value.parse::<f64>().is_err() {
                    return Err(format!("'{}' is not a valid number", value));
                }
            },
            ParameterType::Boolean => {
                if !matches!(
                    value.to_lowercase().as_str(),
                    "true" | "false" | "1" | "0" | "yes" | "no"
                ) {
                    return Err(format!("'{}' is not a valid boolean", value));
                }
            },
            ParameterType::Enum => {
                if let Some(ref allowed_values) = self.values {
                    if !allowed_values.contains(&value.to_string()) {
                        return Err(format!("'{}' is not in allowed values: {:?}", value, allowed_values));
                    }
                } else {
                    return Err("Enum parameter missing allowed values".to_string());
                }
            },
        }

        // Pattern validation
        if let Some(ref pattern) = self.pattern {
            use regex::Regex;
            match Regex::new(pattern) {
                Ok(regex) => {
                    if !regex.is_match(value) {
                        return Err(format!("Value '{}' does not match required pattern", value));
                    }
                },
                Err(_) => {
                    return Err("Invalid regex pattern in parameter definition".to_string());
                },
            }
        }

        Ok(())
    }
}

impl CustomCommandRegistry {
    pub fn new(commands_dir: PathBuf) -> Result<Self, CommandError> {
        fs::create_dir_all(&commands_dir)
            .map_err(|e| CommandError::RegistryError(format!("Failed to create commands directory: {}", e)))?;

        let mut registry = Self {
            commands: HashMap::new(),
            commands_dir,
        };

        registry.load_commands()?;
        Ok(registry)
    }

    pub fn add_command(&mut self, command: CustomCommand) -> Result<(), CommandError> {
        if self.commands.contains_key(&command.name) {
            return Err(CommandError::RegistryError(format!(
                "Command '{}' already exists",
                command.name
            )));
        }

        self.save_command(&command)?;
        self.commands.insert(command.name.clone(), command);
        Ok(())
    }

    pub fn get_command(&self, name: &str) -> Option<&CustomCommand> {
        self.commands.get(name)
    }

    pub fn get_command_mut(&mut self, name: &str) -> Option<&mut CustomCommand> {
        self.commands.get_mut(name)
    }

    pub fn remove_command(&mut self, name: &str) -> Result<(), CommandError> {
        if !self.commands.contains_key(name) {
            return Err(CommandError::NotFound(name.to_string()));
        }

        let file_path = self.commands_dir.join(format!("{}.json", name));
        fs::remove_file(&file_path)
            .map_err(|e| CommandError::RegistryError(format!("Failed to delete command file: {}", e)))?;

        self.commands.remove(name);
        Ok(())
    }

    pub fn list_commands(&self) -> Vec<&CustomCommand> {
        self.commands.values().collect()
    }

    pub fn command_exists(&self, name: &str) -> bool {
        self.commands.contains_key(name)
    }

    fn load_commands(&mut self) -> Result<(), CommandError> {
        let entries = fs::read_dir(&self.commands_dir)
            .map_err(|e| CommandError::RegistryError(format!("Failed to read commands directory: {}", e)))?;

        for entry in entries {
            let entry =
                entry.map_err(|e| CommandError::RegistryError(format!("Failed to read directory entry: {}", e)))?;

            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                match self.load_command_from_file(&path) {
                    Ok(command) => {
                        self.commands.insert(command.name.clone(), command);
                    },
                    Err(e) => {
                        eprintln!("Warning: Failed to load command from {:?}: {}", path, e);
                    },
                }
            }
        }

        Ok(())
    }

    fn load_command_from_file(&self, path: &PathBuf) -> Result<CustomCommand, CommandError> {
        let content = fs::read_to_string(path)
            .map_err(|e| CommandError::RegistryError(format!("Failed to read command file: {}", e)))?;

        let command: CustomCommand = serde_json::from_str(&content)
            .map_err(|e| CommandError::RegistryError(format!("Failed to parse command JSON: {}", e)))?;

        Ok(command)
    }

    fn save_command(&self, command: &CustomCommand) -> Result<(), CommandError> {
        let file_path = self.commands_dir.join(format!("{}.json", command.name));
        let json = serde_json::to_string_pretty(command)
            .map_err(|e| CommandError::RegistryError(format!("Failed to serialize command: {}", e)))?;

        fs::write(&file_path, json)
            .map_err(|e| CommandError::RegistryError(format!("Failed to write command file: {}", e)))?;

        Ok(())
    }

    pub fn update_command(&mut self, command: CustomCommand) -> Result<(), CommandError> {
        if !self.commands.contains_key(&command.name) {
            return Err(CommandError::NotFound(command.name.clone()));
        }

        self.save_command(&command)?;
        self.commands.insert(command.name.clone(), command);
        Ok(())
    }
}
