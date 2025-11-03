#[derive(Debug, Clone, PartialEq)]
pub enum ConversationMode {
    Interactive,
    ExecutePlan,
    Review,
}

// User preferences for conversation modes
#[derive(Debug, Clone)]
pub struct ModePreferences {
    pub auto_detection_enabled: bool,
}

impl ModePreferences {
    pub fn new() -> Self {
        Self {
            auto_detection_enabled: true, // Default to enabled
        }
    }
    
    // Method that doesn't exist yet - test should fail
    pub fn toggle_auto_detection(&mut self) -> String {
        self.auto_detection_enabled = !self.auto_detection_enabled;
        if self.auto_detection_enabled {
            "Auto-detection enabled.".to_string()
        } else {
            "Auto-detection disabled.".to_string()
        }
    }
    
    // Method that doesn't exist yet - test should fail
    pub fn set_auto_detection(&mut self, enabled: bool) -> String {
        self.auto_detection_enabled = enabled;
        if enabled {
            "Auto-detection enabled.".to_string()
        } else {
            "Auto-detection disabled.".to_string()
        }
    }
    
    // Method that doesn't exist yet - test should fail
    pub fn get_preferences_status(&self) -> String {
        format!(
            "Auto-detection: {}",
            if self.auto_detection_enabled { "enabled" } else { "disabled" }
        )
    }
}

impl ConversationMode {
    pub fn detect_from_context(input: &str) -> Self {
        let input_lower = input.to_lowercase();
        if input_lower.contains("review") {
            ConversationMode::Review
        } else if input_lower.contains("implement complete") {
            ConversationMode::ExecutePlan
        } else {
            ConversationMode::Interactive
        }
    }
    
    // Method that doesn't exist yet - test should fail
    pub fn detect_with_preferences(input: &str, preferences: &ModePreferences) -> Option<Self> {
        if !preferences.auto_detection_enabled {
            return None;
        }
        
        let detected = Self::detect_from_context(input);
        // Only return Some if it's not Interactive (i.e., actually detected something)
        if detected != ConversationMode::Interactive {
            Some(detected)
        } else {
            None
        }
    }
    
    // Method that doesn't exist yet - test should fail
    pub fn handle_preferences_command(command: &str, preferences: &mut ModePreferences) -> String {
        match command {
            "/toggle-auto" => preferences.toggle_auto_detection(),
            "/auto-on" => preferences.set_auto_detection(true),
            "/auto-off" => preferences.set_auto_detection(false),
            "/auto-status" => preferences.get_preferences_status(),
            _ => "Unknown preferences command. Available: /toggle-auto, /auto-on, /auto-off, /auto-status".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_detection_enabled_by_default() {
        let preferences = ModePreferences::new();
        assert!(preferences.auto_detection_enabled);
    }

    #[test]
    fn test_toggle_auto_detection() {
        let mut preferences = ModePreferences::new();
        
        // Should be enabled by default
        assert!(preferences.auto_detection_enabled);
        
        // Toggle off
        let response = preferences.toggle_auto_detection();
        assert!(!preferences.auto_detection_enabled);
        assert!(response.contains("disabled") || response.contains("off"));
        
        // Toggle back on
        let response = preferences.toggle_auto_detection();
        assert!(preferences.auto_detection_enabled);
        assert!(response.contains("enabled") || response.contains("on"));
    }

    #[test]
    fn test_set_auto_detection_explicitly() {
        let mut preferences = ModePreferences::new();
        
        let response = preferences.set_auto_detection(false);
        assert!(!preferences.auto_detection_enabled);
        assert!(response.contains("disabled"));
        
        let response = preferences.set_auto_detection(true);
        assert!(preferences.auto_detection_enabled);
        assert!(response.contains("enabled"));
    }

    #[test]
    fn test_detect_with_preferences_enabled() {
        let preferences = ModePreferences::new(); // Auto-detection enabled
        
        let result = ConversationMode::detect_with_preferences("review this code", &preferences);
        assert_eq!(result, Some(ConversationMode::Review));
        
        let result = ConversationMode::detect_with_preferences("implement complete solution", &preferences);
        assert_eq!(result, Some(ConversationMode::ExecutePlan));
    }

    #[test]
    fn test_detect_with_preferences_disabled() {
        let mut preferences = ModePreferences::new();
        preferences.auto_detection_enabled = false;
        
        let result = ConversationMode::detect_with_preferences("review this code", &preferences);
        assert_eq!(result, None); // Should not detect when disabled
        
        let result = ConversationMode::detect_with_preferences("implement complete solution", &preferences);
        assert_eq!(result, None); // Should not detect when disabled
    }

    #[test]
    fn test_preferences_status() {
        let mut preferences = ModePreferences::new();
        
        let status = preferences.get_preferences_status();
        assert!(status.contains("Auto-detection"));
        assert!(status.contains("enabled") || status.contains("on"));
        
        preferences.auto_detection_enabled = false;
        let status = preferences.get_preferences_status();
        assert!(status.contains("disabled") || status.contains("off"));
    }

    #[test]
    fn test_preferences_commands() {
        let mut preferences = ModePreferences::new();
        
        // Test toggle command
        let response = ConversationMode::handle_preferences_command("/toggle-auto", &mut preferences);
        assert!(!preferences.auto_detection_enabled);
        assert!(response.contains("disabled"));
        
        // Test enable command
        let response = ConversationMode::handle_preferences_command("/auto-on", &mut preferences);
        assert!(preferences.auto_detection_enabled);
        assert!(response.contains("enabled"));
        
        // Test disable command
        let response = ConversationMode::handle_preferences_command("/auto-off", &mut preferences);
        assert!(!preferences.auto_detection_enabled);
        assert!(response.contains("disabled"));
        
        // Test status command
        let response = ConversationMode::handle_preferences_command("/auto-status", &mut preferences);
        assert!(response.contains("Auto-detection"));
    }

    #[test]
    fn test_invalid_preferences_command() {
        let mut preferences = ModePreferences::new();
        
        let response = ConversationMode::handle_preferences_command("/invalid", &mut preferences);
        assert!(response.contains("Unknown") || response.contains("Available"));
    }
}

fn main() {
    println!("Running auto-detection toggle tests...");
    // Tests will fail because methods are unimplemented
}
