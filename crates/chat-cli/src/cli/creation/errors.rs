//! Error types for creation system with actionable messages

use std::path::PathBuf;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CreationError {
    #[error(
        "Invalid {creation_type} name '{name}'\nTry: {suggestion}\nValid names: alphanumeric, hyphens, underscores"
    )]
    InvalidName {
        creation_type: String,
        name: String,
        suggestion: String,
    },

    #[error(
        "{creation_type} '{name}' already exists\nUse: q create {creation_type} {name} force\nOr: q create {creation_type} {name} edit"
    )]
    AlreadyExists { creation_type: String, name: String },

    #[error("Template '{template}' not found\nAvailable templates: {available}")]
    TemplateNotFound { template: String, available: String },

    #[error("Invalid command: {command}\nReason: {reason}\nSuggestion: {suggestion}")]
    InvalidCommand {
        command: String,
        reason: String,
        suggestion: String,
    },

    #[error("Missing required field: {field}\nExample: {example}")]
    MissingRequiredField { field: String, example: String },

    #[error("Invalid skill type '{skill_type}'\nValid types: code_inline, code_session, conversation, prompt_inline")]
    InvalidSkillType { skill_type: String },

    #[error("Security validation failed: {reason}\nSuggestion: {suggestion}")]
    SecurityValidationFailed { reason: String, suggestion: String },

    #[error("MCP server '{server}' not available\nInstall: {install_command}\nOr use: {alternatives}")]
    McpServerNotAvailable {
        server: String,
        install_command: String,
        alternatives: String,
    },

    #[error("File system error: {message}\nPath: {path}\nSolution: {solution}")]
    FileSystemError {
        message: String,
        path: PathBuf,
        solution: String,
    },

    #[error("Creation interrupted\nResume with: q create resume\nOr start over: q create {creation_type} {name}")]
    CreationInterrupted { creation_type: String, name: String },

    #[error("Validation failed: {field} = '{value}'\nReason: {reason}\nTry: {suggestion}")]
    ValidationFailed {
        field: String,
        value: String,
        reason: String,
        suggestion: String,
    },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Generic error: {0}")]
    Generic(String),
}

impl CreationError {
    pub fn invalid_name(creation_type: &str, name: &str) -> Self {
        let suggestion = name
            .to_lowercase()
            .replace(' ', "-")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
            .collect();

        Self::InvalidName {
            creation_type: creation_type.to_string(),
            name: name.to_string(),
            suggestion,
        }
    }

    pub fn already_exists(creation_type: &str, name: &str) -> Self {
        Self::AlreadyExists {
            creation_type: creation_type.to_string(),
            name: name.to_string(),
        }
    }

    pub fn template_not_found(template: &str, available: Vec<String>) -> Self {
        Self::TemplateNotFound {
            template: template.to_string(),
            available: available.join(", "),
        }
    }

    pub fn invalid_command(command: &str, reason: &str, suggestion: &str) -> Self {
        Self::InvalidCommand {
            command: command.to_string(),
            reason: reason.to_string(),
            suggestion: suggestion.to_string(),
        }
    }

    pub fn missing_required_field(field: &str, example: &str) -> Self {
        Self::MissingRequiredField {
            field: field.to_string(),
            example: example.to_string(),
        }
    }

    pub fn security_validation_failed(reason: &str, suggestion: &str) -> Self {
        Self::SecurityValidationFailed {
            reason: reason.to_string(),
            suggestion: suggestion.to_string(),
        }
    }

    pub fn file_system_error(message: &str, path: PathBuf, solution: &str) -> Self {
        Self::FileSystemError {
            message: message.to_string(),
            path,
            solution: solution.to_string(),
        }
    }

    pub fn validation_failed(field: &str, value: &str, reason: &str, suggestion: &str) -> Self {
        Self::ValidationFailed {
            field: field.to_string(),
            value: value.to_string(),
            reason: reason.to_string(),
            suggestion: suggestion.to_string(),
        }
    }
}
