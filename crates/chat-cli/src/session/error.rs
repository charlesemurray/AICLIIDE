use thiserror::Error;

#[derive(Debug, Error)]
pub enum SessionError {
    #[error("Session not found: {0}")]
    NotFound(String),

    #[error("Session already exists: {0}")]
    AlreadyExists(String),

    #[error("Invalid session metadata: {0}")]
    InvalidMetadata(String),

    #[error("Corrupted session data: {0}")]
    Corrupted(String),

    #[error("Concurrent modification detected")]
    ConcurrentModification,

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Storage error: {0}")]
    Storage(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Invalid session name: {0}")]
    InvalidName(String),
}

impl SessionError {
    /// Get user-friendly error message with actionable guidance
    pub fn user_message(&self) -> String {
        match self {
            SessionError::NotFound(id) => {
                format!(
                    "Session '{}' not found.\n\
                     Use '/sessions list' to see available sessions.",
                    id
                )
            }
            SessionError::AlreadyExists(id) => {
                format!(
                    "Session '{}' already exists.\n\
                     Use '/sessions list' to see existing sessions.",
                    id
                )
            }
            SessionError::InvalidMetadata(msg) => {
                format!("Invalid session data: {}", msg)
            }
            SessionError::Corrupted(id) => {
                format!(
                    "Session '{}' data is corrupted.\n\
                     Attempting automatic recovery...",
                    id
                )
            }
            SessionError::ConcurrentModification => {
                "Another process is modifying this session.\n\
                 Please try again in a moment."
                    .to_string()
            }
            SessionError::PermissionDenied(path) => {
                format!(
                    "Permission denied accessing: {}\n\
                     Check file permissions in .amazonq/sessions/",
                    path
                )
            }
            SessionError::Storage(e) => {
                format!(
                    "Storage error: {}\nPlease check disk space and permissions.",
                    e
                )
            }
            SessionError::Serialization(e) => {
                format!("Data format error: {}", e)
            }
            SessionError::InvalidName(msg) => {
                format!(
                    "Invalid session name: {}\n\
                     Names must be 1-100 characters, alphanumeric with dash/underscore only.",
                    msg
                )
            }
        }
    }

    /// Check if error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            SessionError::ConcurrentModification | SessionError::Corrupted(_)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_user_messages() {
        let err = SessionError::NotFound("test-123".to_string());
        let msg = err.user_message();
        assert!(msg.contains("test-123"));
        assert!(msg.contains("/sessions list"));
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let session_err = SessionError::from(io_err);

        assert!(matches!(session_err, SessionError::Storage(_)));
        assert!(session_err.user_message().contains("Permission denied") || session_err.user_message().contains("Storage error"));
    }

    #[test]
    fn test_error_display() {
        let err = SessionError::Corrupted("test-123".to_string());
        let display = format!("{}", err);
        assert!(display.contains("Corrupted"));
        assert!(display.contains("test-123"));
    }

    #[test]
    fn test_error_debug() {
        let err = SessionError::ConcurrentModification;
        let debug = format!("{:?}", err);
        assert!(debug.contains("ConcurrentModification"));
    }

    #[test]
    fn test_is_recoverable() {
        assert!(SessionError::ConcurrentModification.is_recoverable());
        assert!(SessionError::Corrupted("test".to_string()).is_recoverable());
        assert!(!SessionError::NotFound("test".to_string()).is_recoverable());
        assert!(!SessionError::InvalidName("test".to_string()).is_recoverable());
    }

    #[test]
    fn test_all_error_variants_have_user_messages() {
        let errors = vec![
            SessionError::NotFound("id".to_string()),
            SessionError::AlreadyExists("id".to_string()),
            SessionError::InvalidMetadata("msg".to_string()),
            SessionError::Corrupted("id".to_string()),
            SessionError::ConcurrentModification,
            SessionError::PermissionDenied("path".to_string()),
            SessionError::InvalidName("msg".to_string()),
        ];

        for err in errors {
            let msg = err.user_message();
            assert!(!msg.is_empty(), "Error {:?} has empty user message", err);
        }
    }
}
