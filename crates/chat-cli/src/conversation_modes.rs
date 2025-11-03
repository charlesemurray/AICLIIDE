#[derive(Debug, Clone, PartialEq)]
pub enum ConversationMode {
    /// Default mode - ask for confirmation at each step
    Interactive,
    /// Execute entire plan without step-by-step confirmations
    ExecutePlan,
    /// Review mode - analyze and provide feedback without execution
    Review,
}

impl ConversationMode {
    pub fn system_prompt_suffix(&self) -> &'static str {
        match self {
            ConversationMode::Interactive => "",
            ConversationMode::ExecutePlan => {
                "\n\nIMPORTANT: Execute the entire plan without asking for step-by-step confirmation. Only ask questions if you need clarification about requirements. Report what you've done after completion."
            },
            ConversationMode::Review => {
                "\n\nIMPORTANT: Analyze and provide detailed feedback on the request. Do not execute any tools or make changes. Focus on reviewing, suggesting improvements, and identifying potential issues."
            },
        }
    }

    pub fn from_user_input(input: &str) -> Option<Self> {
        // Handle empty or whitespace-only input
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return None;
        }

        let input_lower = trimmed.to_lowercase();
        match input_lower.as_str() {
            "/execute" | "/plan" | "/auto" | "/exec" | "/e" => Some(ConversationMode::ExecutePlan),
            "/review" | "/analyze" | "/rev" | "/r" => Some(ConversationMode::Review),
            "/interactive" | "/step" | "/int" | "/i" => Some(ConversationMode::Interactive),
            _ => None,
        }
    }

    pub fn detect_from_context(input: &str) -> Self {
        // Handle empty input gracefully
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return ConversationMode::Interactive;
        }

        let input_lower = trimmed.to_lowercase();

        // Look for plan execution indicators - prioritize complete implementation requests
        if (input_lower.contains("implement")
            && (input_lower.contains("entire") || input_lower.contains("complete") || input_lower.contains("full")))
            || input_lower.contains("build everything")
            || input_lower.contains("create complete")
            || input_lower.contains("develop full")
            || input_lower.contains("execute entire")
        {
            return ConversationMode::ExecutePlan;
        }

        // Look for review indicators - analysis and examination requests
        if input_lower.contains("review")
            || input_lower.contains("analyze")
            || input_lower.contains("feedback")
            || input_lower.contains("examine")
            || input_lower.contains("check")
            || input_lower.contains("audit")
            || input_lower.contains("inspect")
            || input_lower.contains("evaluate")
        {
            return ConversationMode::Review;
        }

        // Default to interactive mode for unclear requests
        ConversationMode::Interactive
    }

    pub fn to_analytics_mode(&self) -> crate::analytics::ConversationMode {
        match self {
            ConversationMode::Interactive => crate::analytics::ConversationMode::Planning,
            ConversationMode::ExecutePlan => crate::analytics::ConversationMode::Implementation,
            ConversationMode::Review => crate::analytics::ConversationMode::Review,
        }
    }

    fn get_mode_name(&self) -> &str {
        match self {
            ConversationMode::Interactive => "Interactive",
            ConversationMode::ExecutePlan => "ExecutePlan", 
            ConversationMode::Review => "Review",
        }
    }

    /// Get a notification message for mode transitions
    pub fn get_transition_notification(&self, trigger: &ConversationModeTrigger) -> String {
        match trigger {
            ConversationModeTrigger::Auto => {
                format!("🔄 Automatically switched to {} mode", self.get_mode_name())
            },
            ConversationModeTrigger::UserCommand => {
                format!("✅ Switched to {} mode", self.get_mode_name())
            },
        }
    }

    /// Handle mode-related commands (/mode, /status)
    pub fn handle_mode_command(command: &str, current_mode: &ConversationMode, mode_history: &[(ConversationMode, ConversationModeTrigger, String)]) -> String {
        match command {
            "/mode" | "/status" => Self::get_mode_status_display(current_mode, mode_history),
            _ => "Unknown command. Use /mode or /status to see current mode.".to_string(),
        }
    }
    
    /// Get detailed status display with current mode and history
    pub fn get_mode_status_display(current_mode: &ConversationMode, mode_history: &[(ConversationMode, ConversationModeTrigger, String)]) -> String {
        let mut status = format!("Current mode: {}\n", current_mode.get_status_display());
        
        if mode_history.is_empty() {
            status.push_str("No recent transitions");
        } else {
            status.push_str("Recent transitions:\n");
            for (mode, trigger, timestamp) in mode_history.iter().rev().take(3) {
                let trigger_text = match trigger {
                    ConversationModeTrigger::Auto => "auto",
                    ConversationModeTrigger::UserCommand => "manual",
                };
                status.push_str(&format!("  {} - {} ({})\n", timestamp, mode.get_mode_name(), trigger_text));
            }
        }
        
        status
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_status_display() {
        let mode = ConversationMode::ExecutePlan;
        let status = mode.get_status_display();
        assert!(status.contains("ExecutePlan"));
        assert!(status.contains("🚀"));
    }

    #[test]
    fn test_mode_prompt_indicator() {
        let mode = ConversationMode::Review;
        let prompt = mode.get_prompt_indicator();
        assert!(prompt.contains("REVIEW"));
        assert!(!prompt.is_empty());
    }

    #[test]
    fn test_auto_transition_notification() {
        let mode = ConversationMode::ExecutePlan;
        let notification = mode.get_transition_notification(&ConversationModeTrigger::Auto);
        assert!(notification.contains("ExecutePlan"));
        assert!(notification.contains("Automatically"));
        assert!(!notification.is_empty());
    }

    #[test]
    fn test_manual_transition_notification() {
        let mode = ConversationMode::Review;
        let notification = mode.get_transition_notification(&ConversationModeTrigger::UserCommand);
        assert!(notification.contains("Review"));
        assert!(notification.contains("Switched"));
        assert!(!notification.is_empty());
    }

    #[test]
    fn test_mode_status_command() {
        let current_mode = ConversationMode::ExecutePlan;
        let history = vec![];
        
        let response = ConversationMode::handle_mode_command("/mode", &current_mode, &history);
        assert!(response.contains("ExecutePlan"));
        assert!(response.contains("Current mode"));
        assert!(response.contains("No recent transitions"));
    }

    #[test]
    fn test_status_command_with_history() {
        let current_mode = ConversationMode::Review;
        let history = vec![
            (ConversationMode::Interactive, ConversationModeTrigger::UserCommand, "2025-11-03 03:40:00".to_string()),
            (ConversationMode::ExecutePlan, ConversationModeTrigger::Auto, "2025-11-03 03:41:00".to_string()),
        ];
        
        let response = ConversationMode::handle_mode_command("/status", &current_mode, &history);
        assert!(response.contains("Review"));
        assert!(response.contains("Recent transitions"));
        assert!(response.contains("ExecutePlan"));
        assert!(response.contains("auto"));
    }

    #[test]
    fn test_invalid_mode_command() {
        let current_mode = ConversationMode::Interactive;
        let history = vec![];
        
        let response = ConversationMode::handle_mode_command("/invalid", &current_mode, &history);
        assert!(response.contains("Unknown command"));
    }
}
