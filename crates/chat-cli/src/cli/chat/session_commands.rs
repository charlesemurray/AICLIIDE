use eyre::Result;
use crate::cli::chat::input_router::SessionCommand;
use crate::theme::session::SessionStatus;

pub struct SessionCommandHandler;

impl SessionCommandHandler {
    pub fn execute(command: SessionCommand) -> Result<String> {
        match command {
            SessionCommand::List { all, waiting } => {
                Self::handle_list(all, waiting)
            },
            SessionCommand::Switch(name) => {
                Self::handle_switch(&name)
            },
            SessionCommand::New { session_type, name } => {
                Self::handle_new(session_type, name)
            },
            SessionCommand::Close(name) => {
                Self::handle_close(name)
            },
            SessionCommand::Rename(name) => {
                Self::handle_rename(&name)
            },
            SessionCommand::SessionName(name) => {
                Self::handle_session_name(name)
            },
        }
    }

    fn handle_list(all: bool, waiting: bool) -> Result<String> {
        let filter = if waiting {
            "waiting"
        } else if all {
            "all"
        } else {
            "active"
        };
        Ok(format!("Listing {} sessions", filter))
    }

    fn handle_switch(name: &str) -> Result<String> {
        Ok(format!("Switching to session: {}", name))
    }

    fn handle_new(session_type: Option<crate::theme::session::SessionType>, name: Option<String>) -> Result<String> {
        let type_str = session_type.map(|t| format!("{:?}", t)).unwrap_or_else(|| "Development".to_string());
        let name_str = name.unwrap_or_else(|| "auto-generated".to_string());
        Ok(format!("Creating new {} session: {}", type_str, name_str))
    }

    fn handle_close(name: Option<String>) -> Result<String> {
        let target = name.unwrap_or_else(|| "current".to_string());
        Ok(format!("Closing session: {}", target))
    }

    fn handle_rename(name: &str) -> Result<String> {
        Ok(format!("Renaming current session to: {}", name))
    }

    fn handle_session_name(name: Option<String>) -> Result<String> {
        match name {
            Some(n) => Ok(format!("Setting session name to: {}", n)),
            None => Ok("Viewing current session name".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::session::SessionType;

    #[test]
    fn test_handle_list_active() {
        let result = SessionCommandHandler::execute(SessionCommand::List {
            all: false,
            waiting: false,
        }).unwrap();
        assert!(result.contains("active"));
    }

    #[test]
    fn test_handle_list_all() {
        let result = SessionCommandHandler::execute(SessionCommand::List {
            all: true,
            waiting: false,
        }).unwrap();
        assert!(result.contains("all"));
    }

    #[test]
    fn test_handle_list_waiting() {
        let result = SessionCommandHandler::execute(SessionCommand::List {
            all: false,
            waiting: true,
        }).unwrap();
        assert!(result.contains("waiting"));
    }

    #[test]
    fn test_handle_switch() {
        let result = SessionCommandHandler::execute(
            SessionCommand::Switch("my-session".to_string())
        ).unwrap();
        assert!(result.contains("my-session"));
    }

    #[test]
    fn test_handle_new_no_args() {
        let result = SessionCommandHandler::execute(SessionCommand::New {
            session_type: None,
            name: None,
        }).unwrap();
        assert!(result.contains("Development"));
        assert!(result.contains("auto-generated"));
    }

    #[test]
    fn test_handle_new_with_type() {
        let result = SessionCommandHandler::execute(SessionCommand::New {
            session_type: Some(SessionType::Debug),
            name: None,
        }).unwrap();
        assert!(result.contains("Debug"));
    }

    #[test]
    fn test_handle_new_with_name() {
        let result = SessionCommandHandler::execute(SessionCommand::New {
            session_type: None,
            name: Some("my-session".to_string()),
        }).unwrap();
        assert!(result.contains("my-session"));
    }

    #[test]
    fn test_handle_close_current() {
        let result = SessionCommandHandler::execute(
            SessionCommand::Close(None)
        ).unwrap();
        assert!(result.contains("current"));
    }

    #[test]
    fn test_handle_close_named() {
        let result = SessionCommandHandler::execute(
            SessionCommand::Close(Some("my-session".to_string()))
        ).unwrap();
        assert!(result.contains("my-session"));
    }

    #[test]
    fn test_handle_rename() {
        let result = SessionCommandHandler::execute(
            SessionCommand::Rename("new-name".to_string())
        ).unwrap();
        assert!(result.contains("new-name"));
    }

    #[test]
    fn test_handle_session_name_view() {
        let result = SessionCommandHandler::execute(
            SessionCommand::SessionName(None)
        ).unwrap();
        assert!(result.contains("Viewing"));
    }

    #[test]
    fn test_handle_session_name_set() {
        let result = SessionCommandHandler::execute(
            SessionCommand::SessionName(Some("new-name".to_string()))
        ).unwrap();
        assert!(result.contains("new-name"));
    }
}
