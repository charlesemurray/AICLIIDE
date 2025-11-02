/// Autocomplete helper for session commands

pub struct SessionAutocomplete;

impl SessionAutocomplete {
    /// Get command completions for partial input
    pub fn complete_command(partial: &str) -> Vec<String> {
        let commands = vec![
            "/sessions",
            "/switch",
            "/s",
            "/new",
            "/close",
            "/rename",
            "/session-name",
        ];

        commands
            .into_iter()
            .filter(|cmd| cmd.starts_with(partial))
            .map(|s| s.to_string())
            .collect()
    }

    /// Get session name completions
    pub fn complete_session_name(partial: &str, available_sessions: &[String]) -> Vec<String> {
        available_sessions
            .iter()
            .filter(|name| name.starts_with(partial))
            .cloned()
            .collect()
    }

    /// Get session type completions
    pub fn complete_session_type(partial: &str) -> Vec<String> {
        let types = vec!["debug", "planning", "development", "review"];

        types
            .into_iter()
            .filter(|t| t.starts_with(partial))
            .map(|s| s.to_string())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complete_command_full_match() {
        let result = SessionAutocomplete::complete_command("/sessions");
        assert_eq!(result, vec!["/sessions"]);
    }

    #[test]
    fn test_complete_command_partial() {
        let result = SessionAutocomplete::complete_command("/s");
        assert_eq!(result.len(), 3); // /sessions, /switch, /s
    }

    #[test]
    fn test_complete_command_no_match() {
        let result = SessionAutocomplete::complete_command("/xyz");
        assert!(result.is_empty());
    }

    #[test]
    fn test_complete_session_name() {
        let sessions = vec![
            "debug-api".to_string(),
            "debug-test".to_string(),
            "planning-feature".to_string(),
        ];
        let result = SessionAutocomplete::complete_session_name("debug", &sessions);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_complete_session_name_no_match() {
        let sessions = vec!["debug-api".to_string()];
        let result = SessionAutocomplete::complete_session_name("xyz", &sessions);
        assert!(result.is_empty());
    }

    #[test]
    fn test_complete_session_type() {
        let result = SessionAutocomplete::complete_session_type("de");
        assert_eq!(result, vec!["debug", "development"]);
    }

    #[test]
    fn test_complete_session_type_no_match() {
        let result = SessionAutocomplete::complete_session_type("xyz");
        assert!(result.is_empty());
    }
}
