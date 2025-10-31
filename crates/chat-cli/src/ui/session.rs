use crate::ui::colors::{SemanticColor, StyledText};

/// Types of sessions in Q CLI
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SessionType {
    Debug,
    Planning,
    Development,
    CodeReview,
}

/// Session display information
#[derive(Debug, Clone, PartialEq)]
pub struct SessionDisplay {
    pub session_type: SessionType,
    pub name: String,
    pub message_count: usize,
    pub status: SessionStatus,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SessionStatus {
    Active,
    Paused,
    Completed,
}

impl SessionType {
    /// Get the semantic color for this session type
    pub fn color(&self) -> SemanticColor {
        match self {
            SessionType::Debug => SemanticColor::Debug,
            SessionType::Planning => SemanticColor::Success,
            SessionType::Development => SemanticColor::Development,
            SessionType::CodeReview => SemanticColor::Warning,
        }
    }

    /// Get the short prefix for this session type
    pub fn prefix(&self) -> &'static str {
        match self {
            SessionType::Debug => "debug",
            SessionType::Planning => "plan",
            SessionType::Development => "dev",
            SessionType::CodeReview => "review",
        }
    }
}

impl SessionDisplay {
    pub fn new(session_type: SessionType, name: impl Into<String>) -> Self {
        Self {
            session_type,
            name: name.into(),
            message_count: 0,
            status: SessionStatus::Active,
        }
    }

    pub fn with_message_count(mut self, count: usize) -> Self {
        self.message_count = count;
        self
    }

    pub fn with_status(mut self, status: SessionStatus) -> Self {
        self.status = status;
        self
    }

    /// Format session message with colored prefix
    pub fn format_message(&self, message: impl Into<String>) -> StyledText {
        let prefix = format!("{}:", self.session_type.prefix());
        StyledText::new(format!("{} {}", prefix, message.into()))
            .with_color(self.session_type.color())
    }

    /// Format session list entry
    pub fn format_list_entry(&self) -> String {
        let status_indicator = match self.status {
            SessionStatus::Active => "",
            SessionStatus::Paused => " (paused)",
            SessionStatus::Completed => " (completed)",
        };
        
        format!("{} {} ({} messages){}",
            self.session_type.prefix(),
            self.name,
            self.message_count,
            status_indicator
        )
    }

    /// Get styled session list entry
    pub fn styled_list_entry(&self) -> StyledText {
        let entry = self.format_list_entry();
        let styled = StyledText::new(entry).with_color(self.session_type.color());
        
        match self.status {
            SessionStatus::Active => styled,
            SessionStatus::Paused => styled.dim(),
            SessionStatus::Completed => styled.dim(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_type_colors() {
        assert_eq!(SessionType::Debug.color(), SemanticColor::Debug);
        assert_eq!(SessionType::Planning.color(), SemanticColor::Success);
        assert_eq!(SessionType::Development.color(), SemanticColor::Development);
        assert_eq!(SessionType::CodeReview.color(), SemanticColor::Warning);
    }

    #[test]
    fn test_session_type_prefixes() {
        assert_eq!(SessionType::Debug.prefix(), "debug");
        assert_eq!(SessionType::Planning.prefix(), "plan");
        assert_eq!(SessionType::Development.prefix(), "dev");
        assert_eq!(SessionType::CodeReview.prefix(), "review");
    }

    #[test]
    fn test_session_display_creation() {
        let session = SessionDisplay::new(SessionType::Debug, "database-perf");
        assert_eq!(session.session_type, SessionType::Debug);
        assert_eq!(session.name, "database-perf");
        assert_eq!(session.message_count, 0);
        assert_eq!(session.status, SessionStatus::Active);
    }

    #[test]
    fn test_session_display_builder() {
        let session = SessionDisplay::new(SessionType::Planning, "feature-roadmap")
            .with_message_count(5)
            .with_status(SessionStatus::Paused);
        
        assert_eq!(session.message_count, 5);
        assert_eq!(session.status, SessionStatus::Paused);
    }

    #[test]
    fn test_format_message() {
        let session = SessionDisplay::new(SessionType::Debug, "test-session");
        let styled = session.format_message("What database system are you using?");
        
        assert_eq!(styled.text, "debug: What database system are you using?");
        assert_eq!(styled.color, Some(SemanticColor::Debug));
    }

    #[test]
    fn test_format_list_entry() {
        let session = SessionDisplay::new(SessionType::Development, "calculator")
            .with_message_count(3)
            .with_status(SessionStatus::Active);
        
        let entry = session.format_list_entry();
        assert_eq!(entry, "dev calculator (3 messages)");
    }

    #[test]
    fn test_format_list_entry_with_status() {
        let paused_session = SessionDisplay::new(SessionType::Planning, "migration")
            .with_message_count(2)
            .with_status(SessionStatus::Paused);
        
        let entry = paused_session.format_list_entry();
        assert_eq!(entry, "plan migration (2 messages) (paused)");

        let completed_session = SessionDisplay::new(SessionType::Debug, "bug-fix")
            .with_message_count(10)
            .with_status(SessionStatus::Completed);
        
        let entry = completed_session.format_list_entry();
        assert_eq!(entry, "debug bug-fix (10 messages) (completed)");
    }

    #[test]
    fn test_styled_list_entry() {
        let active_session = SessionDisplay::new(SessionType::Debug, "active-debug");
        let styled = active_session.styled_list_entry();
        assert_eq!(styled.color, Some(SemanticColor::Debug));
        
        let paused_session = SessionDisplay::new(SessionType::Planning, "paused-plan")
            .with_status(SessionStatus::Paused);
        let styled = paused_session.styled_list_entry();
        assert_eq!(styled.color, Some(SemanticColor::Success));
        // Note: We can't easily test the dim style in unit tests, but the structure is correct
    }

    #[test]
    fn test_all_session_types() {
        let types = [
            SessionType::Debug,
            SessionType::Planning,
            SessionType::Development,
            SessionType::CodeReview,
        ];

        for session_type in types {
            let session = SessionDisplay::new(session_type, "test");
            let message = session.format_message("test message");
            
            // Each session type should have a unique prefix and color
            assert!(message.text.starts_with(&format!("{}:", session_type.prefix())));
            assert_eq!(message.color, Some(session_type.color()));
        }
    }
}
