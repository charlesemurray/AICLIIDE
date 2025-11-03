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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_status_display() {
        let mode = ConversationMode::ExecutePlan;
        let status = mode.get_status_display();
        assert!(status.contains("ExecutePlan"));
        assert!(status.contains("🚀")); // Visual indicator
    }

    #[test]
    fn test_mode_prompt_indicator() {
        let mode = ConversationMode::Review;
        let prompt = mode.get_prompt_indicator();
        assert!(prompt.contains("REVIEW"));
        assert!(!prompt.is_empty());
    }

    #[test]
    fn test_all_modes_have_indicators() {
        let modes = vec![
            ConversationMode::Interactive,
            ConversationMode::ExecutePlan,
            ConversationMode::Review,
        ];
        
        for mode in modes {
            let status = mode.get_status_display();
            let prompt = mode.get_prompt_indicator();
            assert!(!status.is_empty(), "Mode {:?} should have status display", mode);
            assert!(!prompt.is_empty(), "Mode {:?} should have prompt indicator", mode);
        }
    }
}
