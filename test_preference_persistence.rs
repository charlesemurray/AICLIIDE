#[cfg(test)]
mod test_preference_persistence {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_user_preferences_creation() {
        let prefs = UserPreferences::new();
        assert_eq!(prefs.default_mode, ConversationMode::Interactive);
        assert_eq!(prefs.auto_detection_enabled, true);
        assert_eq!(prefs.visual_indicators_enabled, true);
        assert_eq!(prefs.transition_confirmations, true);
    }

    #[test]
    fn test_user_preferences_with_defaults() {
        let prefs = UserPreferences::with_defaults();
        assert_eq!(prefs.default_mode, ConversationMode::Interactive);
        assert!(prefs.preferred_colors.contains_key(&ConversationMode::Interactive));
        assert_eq!(prefs.preferred_colors[&ConversationMode::Interactive], "blue");
    }

    #[test]
    fn test_preference_serialization() {
        let prefs = UserPreferences::new();
        let serialized = prefs.to_config_string();
        assert!(serialized.contains("default_mode"));
        assert!(serialized.contains("auto_detection_enabled"));
    }

    #[test]
    fn test_preference_deserialization() {
        let config_str = r#"
default_mode = "ExecutePlan"
auto_detection_enabled = false
visual_indicators_enabled = true
transition_confirmations = false
"#;
        let prefs = UserPreferences::from_config_string(config_str);
        assert!(prefs.is_ok());
        let prefs = prefs.unwrap();
        assert_eq!(prefs.default_mode, ConversationMode::ExecutePlan);
        assert_eq!(prefs.auto_detection_enabled, false);
        assert_eq!(prefs.transition_confirmations, false);
    }

    #[test]
    fn test_invalid_config_uses_defaults() {
        let invalid_config = "invalid config data";
        let prefs = UserPreferences::from_config_string(invalid_config);
        assert!(prefs.is_err());
        
        let default_prefs = UserPreferences::with_defaults();
        assert_eq!(default_prefs.default_mode, ConversationMode::Interactive);
    }

    #[test]
    fn test_reset_to_defaults() {
        let mut prefs = UserPreferences::new();
        prefs.default_mode = ConversationMode::ExecutePlan;
        prefs.auto_detection_enabled = false;
        
        prefs.reset_to_defaults();
        assert_eq!(prefs.default_mode, ConversationMode::Interactive);
        assert_eq!(prefs.auto_detection_enabled, true);
    }

    #[test]
    fn test_apply_to_session() {
        let mut prefs = UserPreferences::new();
        prefs.default_mode = ConversationMode::Review;
        prefs.auto_detection_enabled = false;
        
        let mut session = MockChatSession::new();
        prefs.apply_to_session(&mut session);
        
        assert_eq!(session.current_mode, ConversationMode::Review);
        assert_eq!(session.auto_detection_enabled, false);
    }

    #[test]
    fn test_preference_validation() {
        let mut prefs = UserPreferences::new();
        
        // Valid color
        let result = prefs.set_preferred_color(ConversationMode::Interactive, "red");
        assert!(result.is_ok());
        
        // Invalid color
        let result = prefs.set_preferred_color(ConversationMode::Interactive, "invalid_color");
        assert!(result.is_err());
    }

    #[test]
    fn test_config_file_operations() {
        let prefs = UserPreferences::new();
        
        // Test save (mock)
        let save_result = prefs.save_to_config();
        assert!(save_result.is_ok());
        
        // Test load (mock)
        let load_result = UserPreferences::load_from_config();
        assert!(load_result.is_ok());
    }

    // Mock session for testing
    struct MockChatSession {
        current_mode: ConversationMode,
        auto_detection_enabled: bool,
    }
    
    impl MockChatSession {
        fn new() -> Self {
            Self {
                current_mode: ConversationMode::Interactive,
                auto_detection_enabled: true,
            }
        }
    }
}
