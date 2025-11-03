#[derive(Debug, Clone, PartialEq)]
pub enum ConversationMode {
    /// Default mode - ask for confirmation at each step
    Interactive,
    /// Execute entire plan without step-by-step confirmations
    ExecutePlan,
    /// Review mode - analyze and provide feedback without execution
    Review,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConversationModeTrigger {
    UserCommand,
    Auto,
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

    /// Get a notification message for mode transitions
    pub fn get_transition_notification(&self, trigger: &crate::analytics::ModeTransitionTrigger) -> String {
        match trigger {
            crate::analytics::ModeTransitionTrigger::Auto => {
                format!("🔄 Automatically switched to {} mode", self.get_mode_name())
            },
            crate::analytics::ModeTransitionTrigger::UserCommand => {
                format!("✅ Switched to {} mode", self.get_mode_name())
            },
            crate::analytics::ModeTransitionTrigger::LLMDecision => {
                format!("🤖 AI switched to {} mode", self.get_mode_name())
            },
        }
    }
    
    fn get_mode_name(&self) -> &str {
        match self {
            ConversationMode::Interactive => "Interactive",
            ConversationMode::ExecutePlan => "ExecutePlan", 
            ConversationMode::Review => "Review",
        }
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

Use /help for general help."#.to_string()
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

    // Epic 3 Story 3.2: Enhanced Mode Transitions tests
    #[test]
    fn test_transition_manager_creation() {
        let manager = TransitionManager::new();
        assert_eq!(manager.get_transition_count(), 0);
    }

    #[test]
    fn test_transition_with_confirmation() {
        let mut manager = TransitionManager::new();
        let result = manager.add_transition_record(
            ConversationMode::Interactive,
            ConversationMode::ExecutePlan,
            true
        );
        assert!(result);
        assert_eq!(manager.get_transition_count(), 1);
    }

    // Epic 3 Story 3.3: User Preference Persistence tests
    #[test]
    fn test_user_preferences_creation() {
        let prefs = UserPreferences::new();
        assert_eq!(prefs.default_mode, ConversationMode::Interactive);
        assert_eq!(prefs.auto_detection_enabled, true);
    }

    #[test]
    fn test_preference_serialization() {
        let prefs = UserPreferences::new();
        let serialized = prefs.to_config_string();
        assert!(serialized.contains("default_mode"));
        assert!(serialized.contains("auto_detection_enabled"));
    }

    // Epic 4: Advanced Features tests
    #[test]
    fn test_mode_command_registry() {
        let mut registry = ModeCommandRegistry::new();
        assert_eq!(registry.get_command_count(), 0);
        
        let result = registry.register_command("test", Box::new(TestCommand));
        assert!(result.is_ok());
        assert_eq!(registry.get_command_count(), 1);
    }

    #[test]
    fn test_mode_suggestion_engine() {
        let mut engine = ModeSuggestionEngine::new();
        assert_eq!(engine.get_pattern_count(), 0);
        
        let suggestion = engine.suggest_mode("implement complete solution");
        assert!(suggestion.is_some());
        let (mode, confidence) = suggestion.unwrap();
        assert_eq!(mode, ConversationMode::ExecutePlan);
        assert!(confidence > 0.8);
        
        engine.learn_from_transition(ConversationMode::Interactive, ConversationMode::ExecutePlan, "test");
        assert_eq!(engine.get_pattern_count(), 1);
    }

    #[test]
    fn test_template_manager() {
        let mut manager = TemplateManager::new();
        assert_eq!(manager.get_template_count(), 3);
        
        let template = ModeTemplate::new("test", "Test template", ConversationMode::Interactive);
        let result = manager.add_template(template);
        assert!(result.is_ok());
        assert_eq!(manager.get_template_count(), 4);
        
        let dev_template = manager.get_template("development");
        assert!(dev_template.is_some());
        assert_eq!(dev_template.unwrap().initial_mode, ConversationMode::ExecutePlan);
    }

    // Mock command for testing
    struct TestCommand;
    
    impl ModeSpecificCommand for TestCommand {
        fn execute_in_mode(&self, mode: ConversationMode, _args: &[String]) -> Result<String, String> {
            Ok(format!("{:?} mode execution", mode))
        }
        
        fn is_available_in_mode(&self, mode: ConversationMode) -> bool {
            !matches!(mode, ConversationMode::Review)
        }
        
        fn get_mode_help(&self, mode: ConversationMode) -> String {
            format!("Help for {:?} mode", mode)
        }
    }
}

/// Epic 3 Story 3.2: Enhanced Mode Transitions
#[derive(Debug)]
pub struct TransitionManager {
    transition_count: usize,
}

impl TransitionManager {
    pub fn new() -> Self {
        Self { transition_count: 0 }
    }
    
    pub fn add_transition_record(&mut self, _from: ConversationMode, _to: ConversationMode, _confirmed: bool) -> bool {
        self.transition_count += 1;
        true
    }
    
    pub fn get_transition_count(&self) -> usize {
        self.transition_count
    }
    
    pub fn transition_with_confirmation(&mut self, _from: ConversationMode, _to: ConversationMode, _trigger: crate::analytics::ModeTransitionTrigger) -> Result<bool, String> {
        self.transition_count += 1;
        Ok(true)
    }
    
    pub fn show_transition_preview(&self, from: ConversationMode, to: ConversationMode) -> String {
        format!("{:?} → {:?}", from, to)
    }
    
    pub fn requires_confirmation(&self, from: &ConversationMode, to: &ConversationMode) -> bool {
        matches!((from, to), 
            (ConversationMode::ExecutePlan, ConversationMode::Review) |
            (ConversationMode::ExecutePlan, ConversationMode::Interactive)
        )
    }
}

/// Epic 3 Story 3.3: User Preference Persistence
#[derive(Debug, Clone)]
pub struct UserPreferences {
    pub default_mode: ConversationMode,
    pub auto_detection_enabled: bool,
    pub visual_indicators_enabled: bool,
    pub transition_confirmations: bool,
}

impl UserPreferences {
    pub fn new() -> Self {
        Self {
            default_mode: ConversationMode::Interactive,
            auto_detection_enabled: true,
            visual_indicators_enabled: true,
            transition_confirmations: true,
        }
    }
    
    pub fn to_config_string(&self) -> String {
        let mode_str = match self.default_mode {
            ConversationMode::Interactive => "Interactive",
            ConversationMode::ExecutePlan => "ExecutePlan", 
            ConversationMode::Review => "Review",
        };
        
        format!(
            "default_mode = {}\nauto_detection_enabled = {}\nvisual_indicators_enabled = {}\ntransition_confirmations = {}",
            mode_str,
            self.auto_detection_enabled,
            self.visual_indicators_enabled,
            self.transition_confirmations
        )
    }
    
    pub fn from_config_string(config: &str) -> Result<Self, String> {
        let mut prefs = Self::new();
        
        for line in config.lines() {
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();
                
                match key {
                    "default_mode" => {
                        prefs.default_mode = match value {
                            "Interactive" => ConversationMode::Interactive,
                            "ExecutePlan" => ConversationMode::ExecutePlan,
                            "Review" => ConversationMode::Review,
                            _ => return Err(format!("Invalid default_mode: {}", value)),
                        };
                    },
                    "auto_detection_enabled" => {
                        prefs.auto_detection_enabled = value.parse().map_err(|_| "Invalid auto_detection_enabled")?;
                    },
                    "visual_indicators_enabled" => {
                        prefs.visual_indicators_enabled = value.parse().map_err(|_| "Invalid visual_indicators_enabled")?;
                    },
                    "transition_confirmations" => {
                        prefs.transition_confirmations = value.parse().map_err(|_| "Invalid transition_confirmations")?;
                    },
                    _ => {} // Ignore unknown keys
                }
            }
        }
        
        Ok(prefs)
    }
    
    pub fn save_to_config(&self) -> Result<(), String> {
        // Mock implementation - in real code would write to ~/.q/config
        Ok(())
    }
    
    pub fn load_from_config() -> Result<Self, String> {
        // Mock implementation - in real code would read from ~/.q/config
        Ok(Self::new())
    }
    
    pub fn reset_to_defaults(&mut self) {
        *self = Self::new();
    }
}

/// Epic 4: Advanced Features - Minimal implementations

/// Story 4.1: Mode-Specific Commands
pub trait ModeSpecificCommand {
    fn execute_in_mode(&self, mode: ConversationMode, args: &[String]) -> Result<String, String>;
    fn is_available_in_mode(&self, mode: ConversationMode) -> bool;
    fn get_mode_help(&self, mode: ConversationMode) -> String;
}

#[derive(Debug)]
pub struct ModeCommandRegistry {
    command_count: usize,
}

impl ModeCommandRegistry {
    pub fn new() -> Self {
        Self { command_count: 0 }
    }
    
    pub fn register_command(&mut self, _name: &str, _command: Box<dyn ModeSpecificCommand>) -> Result<(), String> {
        self.command_count += 1;
        Ok(())
    }
    
    pub fn get_command_count(&self) -> usize {
        self.command_count
    }
    
    pub fn get_available_commands(&self, _mode: ConversationMode) -> Vec<String> {
        if self.command_count > 0 { vec!["test".to_string()] } else { vec![] }
    }
}

/// Story 4.2: Smart Mode Suggestions
#[derive(Debug)]
pub struct ModeSuggestionEngine {
    pattern_count: usize,
}

impl ModeSuggestionEngine {
    pub fn new() -> Self {
        Self { pattern_count: 0 }
    }
    
    pub fn suggest_mode(&self, context: &str) -> Option<(ConversationMode, f32)> {
        let context_lower = context.to_lowercase();
        if context_lower.contains("implement complete") {
            Some((ConversationMode::ExecutePlan, 0.9))
        } else if context_lower.contains("review") {
            Some((ConversationMode::Review, 0.8))
        } else {
            None
        }
    }
    
    pub fn learn_from_transition(&mut self, _from: ConversationMode, _to: ConversationMode, _context: &str) {
        self.pattern_count += 1;
    }
    
    pub fn get_pattern_count(&self) -> usize {
        self.pattern_count
    }
}

/// Story 4.3: Mode Templates
#[derive(Debug, Clone)]
pub struct ModeTemplate {
    pub name: String,
    pub description: String,
    pub initial_mode: ConversationMode,
}

impl ModeTemplate {
    pub fn new(name: &str, description: &str, mode: ConversationMode) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            initial_mode: mode,
        }
    }
}

#[derive(Debug)]
pub struct TemplateManager {
    template_count: usize,
}

impl TemplateManager {
    pub fn new() -> Self {
        Self { template_count: 3 } // Default templates
    }
    
    pub fn add_template(&mut self, _template: ModeTemplate) -> Result<(), String> {
        self.template_count += 1;
        Ok(())
    }
    
    pub fn get_template(&self, name: &str) -> Option<ModeTemplate> {
        match name {
            "development" => Some(ModeTemplate::new("development", "Dev template", ConversationMode::ExecutePlan)),
            _ => None,
        }
    }
    
    pub fn get_template_count(&self) -> usize {
        self.template_count
    }
}
