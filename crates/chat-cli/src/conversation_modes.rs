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

    /// Get a user-friendly status display for the current mode
    pub fn get_status_display(&self) -> String {
        match self {
            ConversationMode::Interactive => "💬 Interactive Mode".to_string(),
            ConversationMode::ExecutePlan => "🚀 ExecutePlan Mode".to_string(),
            ConversationMode::Review => "🔍 Review Mode".to_string(),
        }
    }
    
    /// Get a compact prompt indicator for the current mode
    pub fn get_prompt_indicator(&self) -> String {
        match self {
            ConversationMode::Interactive => "[INTERACTIVE]".to_string(),
            ConversationMode::ExecutePlan => "[EXECUTE]".to_string(),
            ConversationMode::Review => "[REVIEW]".to_string(),
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
    pub fn get_transition_notification(&self, trigger: &crate::analytics::ModeTransitionTrigger) -> String {
        match trigger {
            crate::analytics::ModeTransitionTrigger::Auto => {
                format!("🔄 Automatically switched to {} mode", self.get_mode_name())
            },
            crate::analytics::ModeTransitionTrigger::UserCommand => {
                format!("✅ Switched to {} mode", self.get_mode_name())
            },
        }
    }

    /// Handle mode-related commands (/mode, /status)
    pub fn handle_mode_command(command: &str, current_mode: &ConversationMode, mode_history: &[(ConversationMode, crate::analytics::ModeTransitionTrigger, String)]) -> String {
        match command {
            "/mode" | "/status" => Self::get_mode_status_display(current_mode, mode_history),
            _ => "Unknown command. Use /mode or /status to see current mode.".to_string(),
        }
    }
    
    /// Get detailed status display with current mode and history
    pub fn get_mode_status_display(current_mode: &ConversationMode, mode_history: &[(ConversationMode, crate::analytics::ModeTransitionTrigger, String)]) -> String {
        let mut status = format!("Current mode: {}\n", current_mode.get_status_display());
        
        if mode_history.is_empty() {
            status.push_str("No recent transitions");
        } else {
            status.push_str("Recent transitions:\n");
            for (mode, trigger, timestamp) in mode_history.iter().rev().take(3) {
                let trigger_text = match trigger {
                    crate::analytics::ModeTransitionTrigger::Auto => "auto",
                    crate::analytics::ModeTransitionTrigger::UserCommand => "manual",
                };
                status.push_str(&format!("  {} - {} ({})\n", timestamp, mode.get_mode_name(), trigger_text));
            }
        }
        
        status
    }

    /// Get comprehensive help text for conversation modes
    pub fn get_help_text() -> String {
        r#"Conversation Modes Help

Available Modes:
• Interactive - Default mode with step-by-step confirmations
• ExecutePlan - Execute entire plan without confirmation prompts  
• Review - Analyze and provide analysis without making changes

Manual Commands:
• /execute - Switch to ExecutePlan mode
• /review - Switch to Review mode
• /interactive - Switch to Interactive mode
• /mode or /status - Show current mode and history

Auto-Detection Examples:
• "implement complete solution" → ExecutePlan mode
• "review this code" → Review mode
• "analyze the architecture" → Review mode

Use /help for general help or /modes for quick reference."#.to_string()
    }
    
    /// Handle help-related commands
    pub fn handle_help_command(command: &str) -> String {
        match command {
            "/help modes" => Self::get_help_text(),
            "/help" => "Available help topics and commands:\n• /help modes - Conversation modes guide\n• /modes - Quick reference".to_string(),
            _ => "Unknown help topic. Available: /help modes".to_string(),
        }
    }
    
    /// Get quick reference for conversation modes
    pub fn get_quick_reference() -> String {
        r#"Quick Reference:
/execute - ExecutePlan mode
/review - Review mode  
/interactive - Interactive mode
/mode - Show current mode
/status - Show mode status"#.to_string()
    }

    /// Handle override commands for cancelling automatic mode transitions
    pub fn handle_override_command(
        command: &str, 
        current_mode: &mut ConversationMode,
        previous_mode: &Option<ConversationMode>,
        last_trigger: &Option<crate::analytics::ModeTransitionTrigger>,
        can_override: &mut bool
    ) -> String {
        match command {
            "/cancel" | "/undo" | "/revert" => {
                if !Self::can_override_transition(last_trigger, *can_override) {
                    if last_trigger.is_none() {
                        return "No recent transition to cancel.".to_string();
                    } else {
                        return "Cannot override manual mode transitions.".to_string();
                    }
                }
                
                // Revert to previous mode
                if let Some(prev_mode) = previous_mode {
                    *current_mode = prev_mode.clone();
                    *can_override = false;
                    "Cancelled automatic mode transition. Reverted to previous mode.".to_string()
                } else {
                    "No previous mode to revert to.".to_string()
                }
            },
            _ => format!("Unknown override command: {}. Use /cancel, /undo, or /revert.", command),
        }
    }
    
    /// Check if the last transition can be overridden
    pub fn can_override_transition(last_trigger: &Option<crate::analytics::ModeTransitionTrigger>, can_override: bool) -> bool {
        can_override && matches!(last_trigger, Some(crate::analytics::ModeTransitionTrigger::Auto))
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
        let notification = mode.get_transition_notification(&crate::analytics::ModeTransitionTrigger::Auto);
        assert!(notification.contains("ExecutePlan"));
        assert!(notification.contains("Automatically"));
        assert!(!notification.is_empty());
    }

    #[test]
    fn test_manual_transition_notification() {
        let mode = ConversationMode::Review;
        let notification = mode.get_transition_notification(&crate::analytics::ModeTransitionTrigger::UserCommand);
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
            (ConversationMode::Interactive, crate::analytics::ModeTransitionTrigger::UserCommand, "2025-11-03 03:40:00".to_string()),
            (ConversationMode::ExecutePlan, crate::analytics::ModeTransitionTrigger::Auto, "2025-11-03 03:41:00".to_string()),
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

    #[test]
    fn test_help_modes_command() {
        let help_text = ConversationMode::handle_help_command("/help modes");
        assert!(help_text.contains("Conversation Modes"));
        assert!(help_text.contains("Interactive"));
        assert!(help_text.contains("ExecutePlan"));
        assert!(help_text.contains("Review"));
        assert!(help_text.contains("/execute"));
    }

    #[test]
    fn test_help_command_alias() {
        let help_text = ConversationMode::handle_help_command("/help");
        assert!(help_text.contains("modes"));
        assert!(help_text.contains("commands"));
    }

    #[test]
    fn test_quick_reference() {
        let reference = ConversationMode::get_quick_reference();
        assert!(reference.contains("/execute"));
        assert!(reference.contains("/review"));
        assert!(reference.contains("/interactive"));
        assert!(reference.contains("/mode"));
        assert!(reference.len() < 500);
    }

    #[test]
    fn test_can_override_auto_transition() {
        let trigger = Some(crate::analytics::ModeTransitionTrigger::Auto);
        let can_override = true;
        assert!(ConversationMode::can_override_transition(&trigger, can_override));
    }

    #[test]
    fn test_cannot_override_manual_transition() {
        let trigger = Some(crate::analytics::ModeTransitionTrigger::UserCommand);
        let can_override = true;
        assert!(!ConversationMode::can_override_transition(&trigger, can_override));
    }

    #[test]
    fn test_override_command_with_no_previous_mode() {
        let mut current_mode = ConversationMode::ExecutePlan;
        let previous_mode = None;
        let trigger = Some(crate::analytics::ModeTransitionTrigger::Auto);
        let mut can_override = true;
        
        let response = ConversationMode::handle_override_command(
            "/cancel", &mut current_mode, &previous_mode, &trigger, &mut can_override
        );
        assert!(response.contains("No previous mode"));
    }

    #[test]
    fn test_successful_override() {
        let mut current_mode = ConversationMode::ExecutePlan;
        let previous_mode = Some(ConversationMode::Interactive);
        let trigger = Some(crate::analytics::ModeTransitionTrigger::Auto);
        let mut can_override = true;
        
        let response = ConversationMode::handle_override_command(
            "/undo", &mut current_mode, &previous_mode, &trigger, &mut can_override
        );
        assert!(response.contains("Cancelled"));
        assert_eq!(current_mode, ConversationMode::Interactive);
        assert!(!can_override);
    }
}
