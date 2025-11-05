use clap::ValueEnum;
use crossterm::style::Color;
use serde::{
    Deserialize,
    Serialize,
};

/// Types of sessions in Q CLI
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ValueEnum)]
pub enum SessionType {
    Debug,
    Planning,
    Development,
    CodeReview,
    Feature,
    Hotfix,
    Refactor,
    Experiment,
}

impl SessionType {
    pub fn requires_worktree(&self) -> bool {
        matches!(
            self,
            SessionType::Feature | SessionType::Refactor | SessionType::Experiment
        )
    }

    pub fn is_interactive(&self) -> bool {
        !matches!(self, SessionType::Development)
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            SessionType::Debug => "Debug",
            SessionType::Planning => "Planning",
            SessionType::Development => "Development",
            SessionType::CodeReview => "Code Review",
            SessionType::Feature => "Feature",
            SessionType::Hotfix => "Hotfix",
            SessionType::Refactor => "Refactor",
            SessionType::Experiment => "Experiment",
        }
    }
}

/// Session display information
#[derive(Debug, Clone, PartialEq)]
pub struct SessionDisplay {
    pub session_type: SessionType,
    pub name: String,
    pub message_count: usize,
    pub status: SessionStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SessionStatus {
    Active,
    WaitingForInput,
    Processing,
    Paused,
    Completed,
}

impl SessionStatus {
    /// Check if transition to new state is valid
    pub fn can_transition_to(&self, new_state: &SessionStatus) -> bool {
        use SessionStatus::*;
        match (self, new_state) {
            // Active can transition to any state
            (Active, _) => true,
            // WaitingForInput can become Active or Paused
            (WaitingForInput, Active | Paused | Completed) => true,
            // Processing can become WaitingForInput or Paused
            (Processing, WaitingForInput | Paused | Completed) => true,
            // Paused can become Active or Completed
            (Paused, Active | Completed) => true,
            // Completed is terminal
            (Completed, _) => false,
            // All other transitions invalid
            _ => false,
        }
    }
}

/// Colors for different session types
#[derive(Debug, Clone)]
pub struct SessionColors {
    /// Debug sessions - blue
    pub debug: Color,
    /// Planning sessions - green  
    pub planning: Color,
    /// Development sessions - purple/magenta
    pub development: Color,
    /// Code review sessions - yellow
    pub code_review: Color,
}

impl Default for SessionColors {
    fn default() -> Self {
        Self {
            debug: Color::Blue,
            planning: Color::Green,
            development: Color::Magenta,
            code_review: Color::Yellow,
        }
    }
}

impl SessionType {
    /// Get the color for this session type from the theme
    pub fn color(&self, colors: &SessionColors) -> Color {
        match self {
            SessionType::Debug => colors.debug,
            SessionType::Planning => colors.planning,
            SessionType::Development => colors.development,
            SessionType::CodeReview => colors.code_review,
            SessionType::Feature => colors.development,
            SessionType::Hotfix => colors.debug,
            SessionType::Refactor => colors.code_review,
            SessionType::Experiment => colors.planning,
        }
    }

    /// Get the short prefix for this session type
    pub fn prefix(&self) -> &'static str {
        match self {
            SessionType::Debug => "debug",
            SessionType::Planning => "plan",
            SessionType::Development => "dev",
            SessionType::CodeReview => "review",
            SessionType::Feature => "feature",
            SessionType::Hotfix => "hotfix",
            SessionType::Refactor => "refactor",
            SessionType::Experiment => "experiment",
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

    /// Format session message with colored prefix using crossterm
    pub fn format_message(&self, message: impl Into<String>, colors: &SessionColors) -> String {
        use crossterm::style::Stylize;

        let prefix = format!("{}:", self.session_type.prefix());
        let color = self.session_type.color(colors);

        format!("{} {}", prefix.with(color), message.into())
    }

    /// Format session list entry
    pub fn format_list_entry(&self) -> String {
        let status_indicator = match self.status {
            SessionStatus::Active => "",
            SessionStatus::WaitingForInput => " ⏎",
            SessionStatus::Processing => " ⏳",
            SessionStatus::Paused => " (paused)",
            SessionStatus::Completed => " (completed)",
        };

        format!(
            "{} {} ({} messages){}",
            self.session_type.prefix(),
            self.name,
            self.message_count,
            status_indicator
        )
    }

    /// Get colored session list entry using crossterm
    pub fn colored_list_entry(&self, colors: &SessionColors) -> String {
        use crossterm::style::Stylize;

        let entry = self.format_list_entry();
        let color = self.session_type.color(colors);

        match self.status {
            SessionStatus::Active => entry.with(color).to_string(),
            SessionStatus::WaitingForInput => entry.with(color).bold().to_string(),
            SessionStatus::Processing => entry.with(color).to_string(),
            SessionStatus::Paused | SessionStatus::Completed => entry.with(color).dim().to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crossterm::style::Color;

    use super::*;

    #[test]
    fn test_session_type_colors() {
        let colors = SessionColors::default();
        assert_eq!(SessionType::Debug.color(&colors), Color::Blue);
        assert_eq!(SessionType::Planning.color(&colors), Color::Green);
        assert_eq!(SessionType::Development.color(&colors), Color::Magenta);
        assert_eq!(SessionType::CodeReview.color(&colors), Color::Yellow);
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
    fn test_all_session_types() {
        let colors = SessionColors::default();
        let types = [
            SessionType::Debug,
            SessionType::Planning,
            SessionType::Development,
            SessionType::CodeReview,
        ];

        for session_type in types {
            let session = SessionDisplay::new(session_type, "test");
            let message = session.format_message("test message", &colors);

            // Each session type should have a unique prefix
            assert!(message.contains(&format!("{}:", session_type.prefix())));
        }
    }
}
