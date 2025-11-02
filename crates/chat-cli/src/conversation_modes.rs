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
            ConversationMode::ExecutePlan => "\n\nIMPORTANT: Execute the entire plan without asking for step-by-step confirmation. Only ask questions if you need clarification about requirements. Report what you've done after completion.",
            ConversationMode::Review => "\n\nIMPORTANT: Analyze and provide detailed feedback on the request. Do not execute any tools or make changes. Focus on reviewing, suggesting improvements, and identifying potential issues.",
        }
    }

    pub fn from_user_input(input: &str) -> Option<Self> {
        let input = input.trim().to_lowercase();
        match input.as_str() {
            "/execute" | "/plan" | "/auto" => Some(ConversationMode::ExecutePlan),
            "/review" | "/analyze" => Some(ConversationMode::Review),
            "/interactive" | "/step" => Some(ConversationMode::Interactive),
            _ => None,
        }
    }

    pub fn detect_from_context(input: &str) -> Self {
        let input = input.to_lowercase();
        
        // Look for plan execution indicators
        if input.contains("implement") && (input.contains("entire") || input.contains("complete") || input.contains("full")) {
            return ConversationMode::ExecutePlan;
        }
        
        // Look for review indicators  
        if input.contains("review") || input.contains("analyze") || input.contains("feedback") {
            return ConversationMode::Review;
        }
        
        // Default to interactive
        ConversationMode::Interactive
    }

    pub fn to_analytics_mode(&self) -> crate::analytics::ConversationMode {
        match self {
            ConversationMode::Interactive => crate::analytics::ConversationMode::Planning,
            ConversationMode::ExecutePlan => crate::analytics::ConversationMode::Implementation,
            ConversationMode::Review => crate::analytics::ConversationMode::Review,
        }
    }
}
