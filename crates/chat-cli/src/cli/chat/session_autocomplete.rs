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
            "/list",
            "/active",
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
        let types = vec![
            "debug",
            "planning",
            "development",
            "review",
            "feature",
            "hotfix",
            "refactor",
            "experiment",
        ];

        types
            .into_iter()
            .filter(|t| t.starts_with(partial))
            .map(|s| s.to_string())
            .collect()
    }

    /// Get context-aware completions based on command
    pub fn complete_context_aware(input: &str, available_sessions: &[String]) -> Vec<String> {
        let parts: Vec<&str> = input.split_whitespace().collect();

        if parts.is_empty() {
            return Self::complete_command("");
        }

        let command = parts[0];

        match command {
            "/switch" | "/s" | "/close" | "/rename" => {
                if parts.len() == 1 {
                    // Complete session name
                    Self::complete_session_name("", available_sessions)
                } else {
                    // Complete partial session name
                    Self::complete_session_name(parts[1], available_sessions)
                }
            },
            "/new" => {
                if parts.len() == 1 {
                    // Complete session type
                    Self::complete_session_type("")
                } else {
                    Self::complete_session_type(parts[1])
                }
            },
            _ => {
                // Complete command
                Self::complete_command(command)
            },
        }
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

    #[test]
    fn test_context_aware_switch_command() {
        let sessions = vec!["dev-session".to_string(), "debug-session".to_string()];
        let result = SessionAutocomplete::complete_context_aware("/switch ", &sessions);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_context_aware_switch_partial() {
        let sessions = vec!["dev-session".to_string(), "debug-session".to_string()];
        let result = SessionAutocomplete::complete_context_aware("/switch dev", &sessions);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "dev-session");
    }

    #[test]
    fn test_context_aware_new_command() {
        let sessions = vec![];
        let result = SessionAutocomplete::complete_context_aware("/new ", &sessions);
        assert!(result.len() > 0); // Should return session types
    }

    #[test]
    fn test_context_aware_command_completion() {
        let sessions = vec![];
        let result = SessionAutocomplete::complete_context_aware("/s", &sessions);
        assert!(result.len() >= 3); // /sessions, /switch, /s
    }
}
