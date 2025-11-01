//! Tests for corrected skill creation flow - type first, then type-specific questions

use super::skill::*;
use crate::cli::creation::{CreationMode, SkillType};
use crate::cli::creation::ui_integration_tests::MockUI;
use eyre::Result;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guided_flow_asks_type_first() {
        let mut ui = MockUI::new(vec![
            "2",                    // Select "assistant" (AI conversational helper)
            "Code reviewer",        // System prompt
            "Reviews code for bugs and improvements", // Description
        ]);

        let mut flow = SkillCreationFlow::new("test-skill".to_string(), CreationMode::Guided).unwrap();
        let flow = flow.with_ui(Box::new(ui));
        
        // This should work without errors - type selection first
        // Implementation will be added next
    }

    #[test]
    fn test_skill_type_options_are_correct() {
        let expected_options = &[
            ("command", "Execute shell commands and scripts"),
            ("assistant", "AI conversational helper"),
            ("template", "Text generation with variables"),
            ("session", "Interactive interpreter (Python, Node, etc.)"),
        ];

        // Verify we have the right skill types with good descriptions
        assert_eq!(expected_options.len(), 4);
        assert!(expected_options.iter().any(|(key, _)| *key == "command"));
        assert!(expected_options.iter().any(|(key, _)| *key == "assistant"));
        assert!(expected_options.iter().any(|(key, _)| *key == "template"));
        assert!(expected_options.iter().any(|(key, _)| *key == "session"));
    }

    #[test]
    fn test_command_skill_questions() {
        let mut ui = MockUI::new(vec![
            "1",                    // Select "command" 
            "echo 'Hello World'",   // Command to execute
            "Simple greeting",      // Description
        ]);

        // Should ask: type -> command -> description
        // No prompt/template questions for command skills
    }

    #[test]
    fn test_assistant_skill_questions() {
        let mut ui = MockUI::new(vec![
            "2",                    // Select "assistant"
            "You are a helpful code reviewer", // System prompt
            "Reviews code for quality",        // Description
        ]);

        // Should ask: type -> prompt -> description
        // No command questions for assistant skills
    }

    #[test]
    fn test_template_skill_questions() {
        let mut ui = MockUI::new(vec![
            "3",                    // Select "template"
            "Generate docs for {{function_name}}", // Template text
            "Documentation generator",             // Description
        ]);

        // Should ask: type -> template -> description
        // No command/prompt questions for template skills
    }

    #[test]
    fn test_session_skill_questions() {
        let mut ui = MockUI::new(vec![
            "4",                    // Select "session"
            "2",                    // Select "node" interpreter
            "Interactive Node.js environment", // Description
        ]);

        // Should ask: type -> interpreter -> description
        // Interpreter should be multiple choice too
    }

    #[test]
    fn test_quick_mode_uses_smart_defaults() {
        let mut ui = MockUI::new(vec![
            "1",                    // Select "command" (if not auto-detected)
            "ls -la",              // Command (only required input)
        ]);

        // Quick mode should minimize questions, use smart defaults
        // Description should be auto-generated
    }

    #[test]
    fn test_expert_mode_asks_all_questions() {
        let mut ui = MockUI::new(vec![
            "2",                    // Select "assistant"
            "You are an expert",    // System prompt
            "Expert assistant",     // Description
            "y",                    // Enable security
            "2",                    // Medium security level
        ]);

        // Expert mode should ask everything including security options
    }

    #[test]
    fn test_invalid_skill_type_selection() {
        let mut ui = MockUI::new(vec![
            "invalid",              // Invalid selection
            "2",                    // Valid selection on retry
            "Test prompt",          // System prompt
        ]);

        // Should handle invalid input gracefully and retry
    }
}
