use std::io::Write;

use clap::Args;
use eyre::Result;

use crate::cli::chat::ChatState;
use crate::cli::chat::ConversationState;
use crate::os::Os;
use crate::theme::{ErrorDisplay, ErrorType};

/// Demonstrate colored error output formatting
#[derive(Debug, Args)]
pub struct ErrorDemoArgs {
    /// Type of error to demonstrate
    #[arg(value_enum)]
    pub error_type: Option<DemoErrorType>,
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum DemoErrorType {
    Auth,
    Network,
    File,
    Input,
    System,
    Tool,
}

impl ErrorDemoArgs {
    pub async fn execute(
        &self,
        _os: &mut Os,
        conversation_state: &mut ConversationState,
    ) -> Result<ChatState> {
        match self.error_type {
            Some(DemoErrorType::Auth) => {
                let error = ErrorDisplay::auth_error("Authentication token has expired");
                writeln!(conversation_state.stderr, "{}", error)?;
            },
            Some(DemoErrorType::Network) => {
                let error = ErrorDisplay::network_error("Failed to connect to API server")
                    .with_context("Endpoint: https://api.example.com");
                writeln!(conversation_state.stderr, "{}", error)?;
            },
            Some(DemoErrorType::File) => {
                let error = ErrorDisplay::file_error("Permission denied", Some("/etc/secure/config.json"));
                writeln!(conversation_state.stderr, "{}", error)?;
            },
            Some(DemoErrorType::Input) => {
                let error = ErrorDisplay::input_error("Invalid command syntax: missing required argument")
                    .with_context("Command: /example --missing-arg");
                writeln!(conversation_state.stderr, "{}", error)?;
            },
            Some(DemoErrorType::System) => {
                let error = ErrorDisplay::new(ErrorType::System, "Internal system error occurred")
                    .with_suggestion("Restart the application")
                    .with_suggestion("Check system logs")
                    .with_context("Component: session_manager");
                writeln!(conversation_state.stderr, "{}", error)?;
            },
            Some(DemoErrorType::Tool) => {
                let error = ErrorDisplay::tool_error("Tool execution timed out after 30 seconds", Some("git"));
                writeln!(conversation_state.stderr, "{}", error)?;
            },
            None => {
                // Show all error types
                use crate::theme::formatter;
                let fmt = formatter();
                
                writeln!(conversation_state.stdout, "{}", fmt.header("Error Display Demo"))?;
                writeln!(conversation_state.stdout)?;
                
                let error_types = [
                    ("auth", ErrorDisplay::auth_error("Sample authentication error")),
                    ("network", ErrorDisplay::network_error("Sample network error")),
                    ("file", ErrorDisplay::file_error("Sample file error", Some("/path/to/file"))),
                    ("input", ErrorDisplay::input_error("Sample input error")),
                    ("system", ErrorDisplay::new(ErrorType::System, "Sample system error")),
                    ("tool", ErrorDisplay::tool_error("Sample tool error", Some("example_tool"))),
                ];
                
                for (name, error) in error_types {
                    writeln!(conversation_state.stdout, "{}", fmt.emphasis(format!("{}:", name.to_uppercase())))?;
                    writeln!(conversation_state.stdout, "{}", error)?;
                }
                
                writeln!(conversation_state.stdout, "{}", fmt.info("Use --error-type <type> to see specific error examples"))?;
            },
        }
        
        Ok(ChatState::WaitingForInput)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_demo_args_creation() {
        let args = ErrorDemoArgs { error_type: None };
        assert!(args.error_type.is_none());
        
        let args = ErrorDemoArgs { error_type: Some(DemoErrorType::Auth) };
        assert!(matches!(args.error_type, Some(DemoErrorType::Auth)));
    }

    #[test]
    fn test_demo_error_types() {
        let types = [
            DemoErrorType::Auth,
            DemoErrorType::Network,
            DemoErrorType::File,
            DemoErrorType::Input,
            DemoErrorType::System,
            DemoErrorType::Tool,
        ];
        
        // Just verify all types can be created
        for error_type in types {
            let args = ErrorDemoArgs { error_type: Some(error_type) };
            assert!(args.error_type.is_some());
        }
    }
}
