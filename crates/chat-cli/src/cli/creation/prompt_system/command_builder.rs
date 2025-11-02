//! Command builder for creating executable commands

use std::collections::HashMap;

use eyre::Result;

use super::creation_builder::{CreationBuilder, IssueSeverity, ValidationIssue, ValidationResult};

/// Builder for creating executable commands
pub struct CommandBuilder {
    name: String,
    description: String,
    command: String,
    parameters: Vec<String>,
    working_directory: Option<String>,
    timeout: Option<u64>,
    environment: HashMap<String, String>,
}

/// Command configuration result
#[derive(Debug, Clone)]
pub struct CommandConfig {
    pub name: String,
    pub description: String,
    pub command: String,
    pub parameters: Vec<String>,
    pub working_directory: Option<String>,
    pub timeout: Option<u64>,
    pub environment: HashMap<String, String>,
}

impl CommandBuilder {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            command: String::new(),
            parameters: Vec::new(),
            working_directory: None,
            timeout: None,
            environment: HashMap::new(),
        }
    }

    /// Set the executable command
    pub fn with_command(mut self, command: String) -> Self {
        self.command = command;
        self
    }

    /// Add a command parameter/flag
    pub fn add_parameter(mut self, parameter: String) -> Self {
        self.parameters.push(parameter);
        self
    }

    /// Set all parameters at once
    pub fn with_parameters(mut self, parameters: Vec<String>) -> Self {
        self.parameters = parameters;
        self
    }

    /// Set working directory
    pub fn with_working_directory(mut self, dir: String) -> Self {
        self.working_directory = Some(dir);
        self
    }

    /// Set execution timeout in seconds
    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout = Some(seconds);
        self
    }

    /// Add environment variable
    pub fn with_environment(mut self, key: String, value: String) -> Self {
        self.environment.insert(key, value);
        self
    }

    /// Preview what the command would execute
    pub fn preview(&self) -> String {
        let mut preview = self.command.clone();
        for param in &self.parameters {
            preview.push(' ');
            preview.push_str(param);
        }
        preview
    }
}

impl CreationBuilder for CommandBuilder {
    type Output = CommandConfig;

    fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    fn validate(&self) -> Result<ValidationResult> {
        let mut issues = Vec::new();

        // Name validation
        if self.name.is_empty() {
            issues.push(ValidationIssue {
                severity: IssueSeverity::Error,
                message: "Command name cannot be empty".to_string(),
                suggestion: Some("Provide a descriptive name like 'git-status'".to_string()),
            });
        }

        // Command validation
        if self.command.is_empty() {
            issues.push(ValidationIssue {
                severity: IssueSeverity::Error,
                message: "Command executable cannot be empty".to_string(),
                suggestion: Some("Specify the command to execute like 'git status'".to_string()),
            });
        }

        // Timeout validation
        if let Some(timeout) = self.timeout {
            if timeout == 0 || timeout > 3600 {
                issues.push(ValidationIssue {
                    severity: IssueSeverity::Warning,
                    message: "Timeout should be between 1-3600 seconds".to_string(),
                    suggestion: Some("Use a reasonable timeout like 30 or 300 seconds".to_string()),
                });
            }
        }

        let error_count = issues.iter().filter(|i| i.severity == IssueSeverity::Error).count();
        let is_valid = error_count == 0;

        // Calculate score
        let mut score = 1.0f64;
        if self.description.is_empty() {
            score -= 0.2;
        }
        if self.parameters.is_empty() {
            score -= 0.1;
        }
        if self.timeout.is_none() {
            score -= 0.1;
        }

        score = score.max(0.0).min(1.0);

        Ok(ValidationResult {
            is_valid,
            issues,
            score,
        })
    }

    fn build(self) -> Result<CommandConfig> {
        let validation = self.validate()?;
        if !validation.is_valid {
            let errors: Vec<_> = validation
                .issues
                .iter()
                .filter(|i| i.severity == IssueSeverity::Error)
                .map(|i| i.message.clone())
                .collect();
            return Err(eyre::eyre!("Command validation failed: {}", errors.join(", ")));
        }

        Ok(CommandConfig {
            name: self.name,
            description: self.description,
            command: self.command,
            parameters: self.parameters,
            working_directory: self.working_directory,
            timeout: self.timeout,
            environment: self.environment,
        })
    }
}
