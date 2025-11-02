//! Input routing for multi-session support

use eyre::{Result, bail};

use crate::theme::session::SessionType;

/// Session command parsed from user input
#[derive(Debug, Clone, PartialEq)]
pub enum SessionCommand {
    /// List all sessions
    List {
        /// Show all sessions including completed
        all: bool,
        /// Show only waiting sessions
        waiting: bool,
    },
    /// Switch to a session
    Switch(String), // session name
    /// Create new session
    New {
        /// Session type
        session_type: Option<SessionType>,
        /// Session name
        name: Option<String>,
    },
    /// Close a session
    Close(Option<String>), // session name (None = current)
    /// Rename current session
    Rename(String), // new name
    /// View or set session name
    SessionName(Option<String>), // new name (None = view only)
}

/// Input router for multi-session support
pub struct InputRouter;

impl InputRouter {
    /// Parse input to determine if it's a session command or chat input
    pub fn parse(input: &str) -> Result<Option<SessionCommand>> {
        let input = input.trim();

        if !input.starts_with('/') {
            // Not a command, regular chat input
            return Ok(None);
        }

        // Parse command
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(None);
        }

        match parts[0] {
            "/sessions" => {
                let mut all = false;
                let mut waiting = false;

                for part in &parts[1..] {
                    match *part {
                        "--all" => all = true,
                        "--waiting" => waiting = true,
                        _ => bail!("Unknown flag: {}", part),
                    }
                }

                Ok(Some(SessionCommand::List { all, waiting }))
            }
            "/switch" | "/s" => {
                if parts.len() < 2 {
                    bail!("Usage: /switch <session-name>");
                }
                Ok(Some(SessionCommand::Switch(parts[1].to_string())))
            }
            "/new" => {
                let mut session_type = None;
                let mut name = None;

                if parts.len() > 1 {
                    // Try to parse session type
                    session_type = match parts[1] {
                        "debug" => Some(SessionType::Debug),
                        "planning" | "plan" => Some(SessionType::Planning),
                        "dev" | "development" => Some(SessionType::Development),
                        "review" | "code-review" => Some(SessionType::CodeReview),
                        _ => {
                            // Not a type, treat as name
                            name = Some(parts[1].to_string());
                            None
                        }
                    };

                    // If we parsed a type and there's another part, it's the name
                    if session_type.is_some() && parts.len() > 2 {
                        name = Some(parts[2].to_string());
                    }
                }

                Ok(Some(SessionCommand::New { session_type, name }))
            }
            "/close" => {
                let name = parts.get(1).map(|s| s.to_string());
                Ok(Some(SessionCommand::Close(name)))
            }
            "/rename" => {
                if parts.len() < 2 {
                    bail!("Usage: /rename <new-name>");
                }
                Ok(Some(SessionCommand::Rename(parts[1].to_string())))
            }
            "/session-name" => {
                let name = parts.get(1).map(|s| s.to_string());
                Ok(Some(SessionCommand::SessionName(name)))
            }
            _ => {
                // Not a session command, let it pass through
                Ok(None)
            }
        }
    }

    /// Validate session name format
    pub fn validate_session_name(name: &str) -> Result<()> {
        if name.is_empty() {
            bail!("Session name cannot be empty");
        }

        if name.len() > 20 {
            bail!("Session name must be 20 characters or less");
        }

        // Check for valid characters (alphanumeric, dash, underscore)
        if !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            bail!("Session name can only contain letters, numbers, dashes, and underscores");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_regular_input() {
        let result = InputRouter::parse("Hello, how are you?").unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_sessions_list() {
        let result = InputRouter::parse("/sessions").unwrap();
        assert_eq!(
            result,
            Some(SessionCommand::List {
                all: false,
                waiting: false
            })
        );
    }

    #[test]
    fn test_parse_sessions_list_all() {
        let result = InputRouter::parse("/sessions --all").unwrap();
        assert_eq!(
            result,
            Some(SessionCommand::List {
                all: true,
                waiting: false
            })
        );
    }

    #[test]
    fn test_parse_sessions_list_waiting() {
        let result = InputRouter::parse("/sessions --waiting").unwrap();
        assert_eq!(
            result,
            Some(SessionCommand::List {
                all: false,
                waiting: true
            })
        );
    }

    #[test]
    fn test_parse_switch() {
        let result = InputRouter::parse("/switch my-session").unwrap();
        assert_eq!(result, Some(SessionCommand::Switch("my-session".to_string())));
    }

    #[test]
    fn test_parse_switch_alias() {
        let result = InputRouter::parse("/s my-session").unwrap();
        assert_eq!(result, Some(SessionCommand::Switch("my-session".to_string())));
    }

    #[test]
    fn test_parse_switch_no_name() {
        let result = InputRouter::parse("/switch");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_new_no_args() {
        let result = InputRouter::parse("/new").unwrap();
        assert_eq!(
            result,
            Some(SessionCommand::New {
                session_type: None,
                name: None
            })
        );
    }

    #[test]
    fn test_parse_new_with_type() {
        let result = InputRouter::parse("/new debug").unwrap();
        assert_eq!(
            result,
            Some(SessionCommand::New {
                session_type: Some(SessionType::Debug),
                name: None
            })
        );
    }

    #[test]
    fn test_parse_new_with_type_and_name() {
        let result = InputRouter::parse("/new debug my-debug-session").unwrap();
        assert_eq!(
            result,
            Some(SessionCommand::New {
                session_type: Some(SessionType::Debug),
                name: Some("my-debug-session".to_string())
            })
        );
    }

    #[test]
    fn test_parse_new_with_name_only() {
        let result = InputRouter::parse("/new my-session").unwrap();
        assert_eq!(
            result,
            Some(SessionCommand::New {
                session_type: None,
                name: Some("my-session".to_string())
            })
        );
    }

    #[test]
    fn test_parse_close_no_name() {
        let result = InputRouter::parse("/close").unwrap();
        assert_eq!(result, Some(SessionCommand::Close(None)));
    }

    #[test]
    fn test_parse_close_with_name() {
        let result = InputRouter::parse("/close my-session").unwrap();
        assert_eq!(
            result,
            Some(SessionCommand::Close(Some("my-session".to_string())))
        );
    }

    #[test]
    fn test_parse_rename() {
        let result = InputRouter::parse("/rename new-name").unwrap();
        assert_eq!(result, Some(SessionCommand::Rename("new-name".to_string())));
    }

    #[test]
    fn test_parse_rename_no_name() {
        let result = InputRouter::parse("/rename");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_session_name_view() {
        let result = InputRouter::parse("/session-name").unwrap();
        assert_eq!(result, Some(SessionCommand::SessionName(None)));
    }

    #[test]
    fn test_parse_session_name_set() {
        let result = InputRouter::parse("/session-name new-name").unwrap();
        assert_eq!(
            result,
            Some(SessionCommand::SessionName(Some("new-name".to_string())))
        );
    }

    #[test]
    fn test_validate_session_name_valid() {
        assert!(InputRouter::validate_session_name("my-session").is_ok());
        assert!(InputRouter::validate_session_name("session_123").is_ok());
        assert!(InputRouter::validate_session_name("debug-1").is_ok());
    }

    #[test]
    fn test_validate_session_name_empty() {
        assert!(InputRouter::validate_session_name("").is_err());
    }

    #[test]
    fn test_validate_session_name_too_long() {
        assert!(InputRouter::validate_session_name("this-is-a-very-long-session-name").is_err());
    }

    #[test]
    fn test_validate_session_name_invalid_chars() {
        assert!(InputRouter::validate_session_name("my session").is_err());
        assert!(InputRouter::validate_session_name("my@session").is_err());
        assert!(InputRouter::validate_session_name("my.session").is_err());
    }

    #[test]
    fn test_parse_unknown_command() {
        let result = InputRouter::parse("/unknown-command").unwrap();
        assert_eq!(result, None); // Pass through to regular command handling
    }
}
