use std::fmt;

use crate::theme::formatter;

/// Enhanced error display with colored output and suggestions
#[derive(Debug)]
pub struct ErrorDisplay {
    pub error_type: ErrorType,
    pub message: String,
    pub suggestions: Vec<String>,
    pub context: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ErrorType {
    /// Authentication errors
    Auth,
    /// Network/API errors
    Network,
    /// File system errors
    FileSystem,
    /// User input errors
    Input,
    /// System/internal errors
    System,
    /// Tool execution errors
    Tool,
}

impl ErrorDisplay {
    pub fn new(error_type: ErrorType, message: impl Into<String>) -> Self {
        Self {
            error_type,
            message: message.into(),
            suggestions: Vec::new(),
            context: None,
        }
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestions.push(suggestion.into());
        self
    }

    pub fn with_suggestions(mut self, suggestions: Vec<String>) -> Self {
        self.suggestions = suggestions;
        self
    }

    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    /// Format the error with colors and suggestions
    pub fn format_colored(&self) -> String {
        let fmt = formatter();
        let mut output = String::new();

        // Error header with type indicator
        let type_indicator = match self.error_type {
            ErrorType::Auth => "AUTH",
            ErrorType::Network => "NETWORK",
            ErrorType::FileSystem => "FILE",
            ErrorType::Input => "INPUT",
            ErrorType::System => "SYSTEM",
            ErrorType::Tool => "TOOL",
        };

        output.push_str(&fmt.status_error(format!("{}: {}", type_indicator, self.message)));
        output.push('\n');

        // Add context if provided
        if let Some(context) = &self.context {
            output.push_str(&fmt.secondary(format!("Context: {}", context)));
            output.push('\n');
        }

        // Add suggestions if any
        if !self.suggestions.is_empty() {
            output.push('\n');
            output.push_str(&fmt.info("Suggestions:"));
            output.push('\n');
            for suggestion in &self.suggestions {
                output.push_str(&fmt.list_item(suggestion));
                output.push('\n');
            }
        }

        output
    }

    /// Create an auth error with common suggestions
    pub fn auth_error(message: impl Into<String>) -> Self {
        Self::new(ErrorType::Auth, message).with_suggestions(vec![
            "Run 'q login' to authenticate".to_string(),
            "Check your internet connection".to_string(),
            "Verify your AWS credentials".to_string(),
        ])
    }

    /// Create a network error with common suggestions
    pub fn network_error(message: impl Into<String>) -> Self {
        Self::new(ErrorType::Network, message).with_suggestions(vec![
            "Check your internet connection".to_string(),
            "Try again in a few moments".to_string(),
            "Verify the service is available".to_string(),
        ])
    }

    /// Create a file system error with common suggestions
    pub fn file_error(message: impl Into<String>, path: Option<&str>) -> Self {
        let mut error = Self::new(ErrorType::FileSystem, message).with_suggestions(vec![
            "Check if the file exists".to_string(),
            "Verify file permissions".to_string(),
            "Ensure the directory is accessible".to_string(),
        ]);

        if let Some(path) = path {
            error = error.with_context(format!("Path: {}", path));
        }

        error
    }

    /// Create an input error with suggestions
    pub fn input_error(message: impl Into<String>) -> Self {
        Self::new(ErrorType::Input, message).with_suggestions(vec![
            "Check the command syntax".to_string(),
            "Use --help for usage information".to_string(),
            "Verify all required parameters are provided".to_string(),
        ])
    }

    /// Create a tool error with suggestions
    pub fn tool_error(message: impl Into<String>, tool_name: Option<&str>) -> Self {
        let mut error = Self::new(ErrorType::Tool, message).with_suggestions(vec![
            "Check tool permissions".to_string(),
            "Verify tool dependencies are installed".to_string(),
            "Try running the tool manually".to_string(),
        ]);

        if let Some(tool_name) = tool_name {
            error = error.with_context(format!("Tool: {}", tool_name));
        }

        error
    }
}

impl fmt::Display for ErrorDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_colored())
    }
}

/// Helper function to create error displays from common error types
pub fn format_chat_error(error: &crate::cli::chat::ChatError) -> ErrorDisplay {
    use crate::cli::chat::ChatError;

    match error {
        ChatError::Auth(_) => ErrorDisplay::auth_error(error.to_string()),
        ChatError::Client(_) | ChatError::SendMessage(_) | ChatError::ResponseStream(_) => {
            ErrorDisplay::network_error(error.to_string())
        },
        ChatError::Std(_) => ErrorDisplay::file_error(error.to_string(), None),
        ChatError::Custom(msg) => {
            // Try to categorize custom errors based on content
            let msg_lower = msg.to_lowercase();
            if msg_lower.contains("file") || msg_lower.contains("path") {
                ErrorDisplay::file_error(msg.to_string(), None)
            } else if msg_lower.contains("network") || msg_lower.contains("connection") {
                ErrorDisplay::network_error(msg.to_string())
            } else if msg_lower.contains("tool") {
                ErrorDisplay::tool_error(msg.to_string(), None)
            } else {
                ErrorDisplay::new(ErrorType::System, msg.to_string())
            }
        },
        ChatError::GetPromptError(_) => ErrorDisplay::input_error(error.to_string()),
        _ => ErrorDisplay::new(ErrorType::System, error.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_creation() {
        let error = ErrorDisplay::new(ErrorType::Auth, "Authentication failed");
        assert_eq!(error.error_type, ErrorType::Auth);
        assert_eq!(error.message, "Authentication failed");
        assert!(error.suggestions.is_empty());
        assert!(error.context.is_none());
    }

    #[test]
    fn test_error_display_with_suggestions() {
        let error = ErrorDisplay::new(ErrorType::Network, "Connection timeout")
            .with_suggestion("Check internet connection")
            .with_suggestion("Try again later");

        assert_eq!(error.suggestions.len(), 2);
        assert_eq!(error.suggestions[0], "Check internet connection");
        assert_eq!(error.suggestions[1], "Try again later");
    }

    #[test]
    fn test_error_display_with_context() {
        let error =
            ErrorDisplay::new(ErrorType::FileSystem, "File not found").with_context("Path: /nonexistent/file.txt");

        assert_eq!(error.context, Some("Path: /nonexistent/file.txt".to_string()));
    }

    #[test]
    fn test_auth_error_helper() {
        let error = ErrorDisplay::auth_error("Token expired");
        assert_eq!(error.error_type, ErrorType::Auth);
        assert_eq!(error.message, "Token expired");
        assert!(!error.suggestions.is_empty());
        assert!(error.suggestions.iter().any(|s| s.contains("q login")));
    }

    #[test]
    fn test_network_error_helper() {
        let error = ErrorDisplay::network_error("API unavailable");
        assert_eq!(error.error_type, ErrorType::Network);
        assert_eq!(error.message, "API unavailable");
        assert!(!error.suggestions.is_empty());
        assert!(error.suggestions.iter().any(|s| s.contains("internet connection")));
    }

    #[test]
    fn test_file_error_helper() {
        let error = ErrorDisplay::file_error("Permission denied", Some("/etc/config"));
        assert_eq!(error.error_type, ErrorType::FileSystem);
        assert_eq!(error.message, "Permission denied");
        assert!(!error.suggestions.is_empty());
        assert!(error.context.is_some());
        assert!(error.context.as_ref().unwrap().contains("/etc/config"));
    }

    #[test]
    fn test_input_error_helper() {
        let error = ErrorDisplay::input_error("Invalid command syntax");
        assert_eq!(error.error_type, ErrorType::Input);
        assert_eq!(error.message, "Invalid command syntax");
        assert!(!error.suggestions.is_empty());
        assert!(error.suggestions.iter().any(|s| s.contains("--help")));
    }

    #[test]
    fn test_tool_error_helper() {
        let error = ErrorDisplay::tool_error("Tool execution failed", Some("git"));
        assert_eq!(error.error_type, ErrorType::Tool);
        assert_eq!(error.message, "Tool execution failed");
        assert!(!error.suggestions.is_empty());
        assert!(error.context.is_some());
        assert!(error.context.as_ref().unwrap().contains("git"));
    }

    #[test]
    fn test_format_colored_basic() {
        let error = ErrorDisplay::new(ErrorType::System, "Test error");
        let formatted = error.format_colored();
        assert!(formatted.contains("SYSTEM"));
        assert!(formatted.contains("Test error"));
    }

    #[test]
    fn test_format_colored_with_suggestions() {
        let error = ErrorDisplay::new(ErrorType::Input, "Invalid input")
            .with_suggestion("Try this")
            .with_suggestion("Or this");

        let formatted = error.format_colored();
        assert!(formatted.contains("INPUT"));
        assert!(formatted.contains("Invalid input"));
        assert!(formatted.contains("Suggestions:"));
        assert!(formatted.contains("Try this"));
        assert!(formatted.contains("Or this"));
    }

    #[test]
    fn test_format_colored_with_context() {
        let error = ErrorDisplay::new(ErrorType::FileSystem, "Access denied").with_context("File: /secure/file.txt");

        let formatted = error.format_colored();
        assert!(formatted.contains("FILE"));
        assert!(formatted.contains("Access denied"));
        assert!(formatted.contains("Context:"));
        assert!(formatted.contains("/secure/file.txt"));
    }

    #[test]
    fn test_display_trait() {
        let error = ErrorDisplay::new(ErrorType::Network, "Connection failed");
        let display_output = format!("{}", error);
        assert!(display_output.contains("NETWORK"));
        assert!(display_output.contains("Connection failed"));
    }

    #[test]
    fn test_error_type_indicators() {
        let types = [
            (ErrorType::Auth, "AUTH"),
            (ErrorType::Network, "NETWORK"),
            (ErrorType::FileSystem, "FILE"),
            (ErrorType::Input, "INPUT"),
            (ErrorType::System, "SYSTEM"),
            (ErrorType::Tool, "TOOL"),
        ];

        for (error_type, expected_indicator) in types {
            let error = ErrorDisplay::new(error_type, "Test message");
            let formatted = error.format_colored();
            assert!(formatted.contains(expected_indicator));
        }
    }
}
