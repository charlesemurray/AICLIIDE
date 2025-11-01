//! UX tests for terminal user experience and interaction patterns

use super::*;
use crate::cli::creation::*;

#[cfg(test)]
mod terminal_native_ux {
    use super::*;

    #[test]
    fn test_no_emojis_in_output() {
        let mut ui = MockTerminalUI::new(vec!["test".to_string()]);
        ui.show_success("Command created successfully");
        ui.show_error("Invalid command name");
        ui.show_info("Creating skill...");
        
        // Verify no emojis in any output
        for output in &ui.outputs {
            assert!(!output.contains("🛠️"));
            assert!(!output.contains("✅"));
            assert!(!output.contains("❌"));
            assert!(!output.contains("📝"));
            assert!(!output.contains("🚀"));
        }
    }

    #[test]
    fn test_ansi_color_usage() {
        let mut ui = MockTerminalUI::new(vec![]);
        
        ui.show_success("Success message");
        ui.show_error("Error message");
        ui.show_warning("Warning message");
        ui.show_info("Info message");
        
        // Verify ANSI color codes are used
        assert!(ui.outputs.iter().any(|o| o.contains("\x1b[32m"))); // Green for success
        assert!(ui.outputs.iter().any(|o| o.contains("\x1b[31m"))); // Red for error
        assert!(ui.outputs.iter().any(|o| o.contains("\x1b[33m"))); // Yellow for warning
        assert!(ui.outputs.iter().any(|o| o.contains("\x1b[34m"))); // Blue for info
    }

    #[test]
    fn test_semantic_color_mapping() {
        let ui = TerminalUI::new();
        
        // Test semantic color mapping
        assert_eq!(ui.get_color(SemanticColor::Success), AnsiColor::Green);
        assert_eq!(ui.get_color(SemanticColor::Error), AnsiColor::Red);
        assert_eq!(ui.get_color(SemanticColor::Warning), AnsiColor::Yellow);
        assert_eq!(ui.get_color(SemanticColor::Info), AnsiColor::Blue);
        assert_eq!(ui.get_color(SemanticColor::Debug), AnsiColor::Cyan);
    }

    #[test]
    fn test_information_density() {
        let mut ui = MockTerminalUI::new(vec![]);
        
        let preview = "Creating: skill 'test' (code_inline)\nCommand: python script.py\nDescription: Test skill\nSecurity: enabled (medium)\nLocation: .q-skills/test.json";
        ui.show_preview(preview);
        
        // Verify compact, information-dense format
        let output = &ui.outputs[0];
        assert!(output.lines().count() <= 5); // Compact format
        assert!(output.contains("Creating:")); // Clear action
        assert!(output.contains("Location:")); // Practical info
        assert!(!output.contains("Welcome")); // No fluff
        assert!(!output.contains("Let's")); // No conversational tone
    }

    #[test]
    fn test_progress_indication() {
        let mut ui = MockTerminalUI::new(vec![]);
        
        ui.show_progress(2, 4, "Configuring skill");
        
        let output = &ui.outputs[0];
        assert!(output.contains("████░░░░")); // ASCII progress bar
        assert!(output.contains("50%")); // Percentage
        assert!(output.contains("2/4")); // Step indicator
        assert!(!output.contains("Step 2 of 4")); // Verbose format avoided
    }
}

#[cfg(test)]
mod single_pass_creation {
    use super::*;

    #[test]
    fn test_command_single_pass_flow() {
        let mut ui = MockTerminalUI::new(vec![
            "echo hello".to_string(),           // command
            "Test command".to_string(),         // description
            "y".to_string(),                    // confirm
        ]);
        
        let mut flow = CommandCreationFlow::new("test", &mut ui);
        let result = flow.run_single_pass().unwrap();
        
        // Verify single pass - no back and forth
        assert_eq!(ui.inputs.len(), 3); // Only 3 interactions
        assert!(result.is_complete());
        
        // Verify no "step 1 of N" messaging
        assert!(!ui.outputs.iter().any(|o| o.contains("Step")));
        assert!(!ui.outputs.iter().any(|o| o.contains("Next")));
    }

    #[test]
    fn test_skill_single_pass_guided() {
        let mut ui = MockTerminalUI::new(vec![
            "python script.py".to_string(),     // command
            "Test skill".to_string(),           // description
            "medium".to_string(),               // security level
            "y".to_string(),                    // confirm
        ]);
        
        let mut flow = SkillCreationFlow::new("test", SkillMode::Guided, &mut ui);
        let result = flow.run_single_pass().unwrap();
        
        // Even guided mode should be single pass
        assert_eq!(ui.inputs.len(), 4);
        assert!(result.is_complete());
        
        // Should collect all info upfront, then preview
        assert!(ui.outputs.iter().any(|o| o.contains("Preview:")));
    }

    #[test]
    fn test_agent_single_pass_expert() {
        let mut ui = MockTerminalUI::new(vec![
            "You are helpful".to_string(),      // prompt
            "Helper agent".to_string(),         // description
            "filesystem".to_string(),           // MCP server
            "fs_read,fs_write".to_string(),     // tools
            "y".to_string(),                    // confirm
        ]);
        
        let mut flow = AgentCreationFlow::new("test", AgentMode::Expert, &mut ui);
        let result = flow.run_single_pass().unwrap();
        
        // Even complex agent creation should be single pass
        assert_eq!(ui.inputs.len(), 5);
        assert!(result.is_complete());
    }
}

#[cfg(test)]
mod power_user_efficiency {
    use super::*;

    #[test]
    fn test_quick_mode_minimal_prompts() {
        let mut ui = MockTerminalUI::new(vec![
            "python script.py".to_string(),     // Only required input
        ]);
        
        let mut flow = SkillCreationFlow::new("test", SkillMode::Quick, &mut ui);
        let result = flow.run_single_pass().unwrap();
        
        // Quick mode should minimize interactions
        assert_eq!(ui.inputs.len(), 1);
        assert!(result.is_complete());
        
        // Should use smart defaults for everything else
        let config = result.get_config();
        assert!(!config.description.is_empty()); // Auto-generated
        assert_eq!(config.skill_type, SkillType::CodeInline); // Auto-detected
    }

    #[test]
    fn test_template_mode_efficiency() {
        let fixtures = TestFixtures::new();
        fixtures.setup_directories();
        
        // Create template skill
        std::fs::write(
            fixtures.skills_dir.join("template.json"),
            r#"{"name": "template", "type": "code_inline", "command": "python {{script}}", "description": "Python runner"}"#
        ).unwrap();
        
        let mut ui = MockTerminalUI::new(vec![
            "main.py".to_string(),              // Only parameter value needed
        ]);
        
        let mut flow = SkillCreationFlow::new_from_template("test", "template", &mut ui);
        let result = flow.run_single_pass().unwrap();
        
        // Template mode should be very efficient
        assert_eq!(ui.inputs.len(), 1);
        
        let config = result.get_config();
        assert_eq!(config.command, "python main.py"); // Template applied
        assert_eq!(config.description, "Python runner"); // Inherited
    }

    #[test]
    fn test_preview_mode_no_creation() {
        let mut ui = MockTerminalUI::new(vec![
            "echo hello".to_string(),
        ]);
        
        let mut flow = CommandCreationFlow::new("test", &mut ui);
        let result = flow.run_preview_only().unwrap();
        
        // Preview mode should show what would be created without creating
        assert!(ui.outputs.iter().any(|o| o.contains("Would create:")));
        assert!(ui.outputs.iter().any(|o| o.contains("echo hello")));
        assert!(!result.was_created()); // Nothing actually created
    }

    #[test]
    fn test_batch_mode_scriptability() {
        let config_json = r#"{
            "name": "batch-skill",
            "type": "code_inline", 
            "command": "python batch.py",
            "description": "Batch created skill"
        }"#;
        
        let mut flow = SkillCreationFlow::new_from_json("batch-skill", config_json).unwrap();
        let result = flow.run_batch().unwrap();
        
        // Batch mode should require no user interaction
        assert!(result.is_complete());
        assert_eq!(result.get_config().name, "batch-skill");
    }
}

#[cfg(test)]
mod error_messaging {
    use super::*;

    #[test]
    fn test_actionable_error_messages() {
        let context = CreationContext::new(std::env::current_dir().unwrap()).unwrap();
        
        // Test invalid name error
        let validation = context.validate_name("Invalid Name!", &CreationType::Skill);
        assert!(!validation.is_valid());
        
        let error_msg = validation.error_message();
        assert!(error_msg.contains("Invalid skill name")); // Clear problem
        assert!(error_msg.contains("invalid-name")); // Suggested fix
        assert!(error_msg.contains("Valid names:")); // Explanation of rules
        assert!(!error_msg.contains("please try again")); // No vague messaging
    }

    #[test]
    fn test_context_aware_suggestions() {
        let fixtures = TestFixtures::new();
        fixtures.setup_directories();
        
        // Create existing skill
        std::fs::write(
            fixtures.skills_dir.join("existing.json"),
            r#"{"name": "existing"}"#
        ).unwrap();
        
        let context = CreationContext::new(fixtures.temp_dir.path()).unwrap();
        let validation = context.validate_name("existing", &CreationType::Skill);
        
        assert!(!validation.is_valid());
        let error_msg = validation.error_message();
        assert!(error_msg.contains("already exists"));
        assert!(error_msg.contains("force")); // Suggest force mode
        assert!(error_msg.contains("edit")); // Suggest edit mode
        assert!(error_msg.contains("existing-2")); // Suggest alternative name
    }

    #[test]
    fn test_immediate_validation_feedback() {
        let mut ui = MockTerminalUI::new(vec![
            "invalid!name".to_string(),         // Invalid input
            "valid-name".to_string(),           // Corrected input
        ]);
        
        let result = ui.prompt_with_validation("Skill name", |name| {
            if name.contains('!') {
                Err("Invalid character '!' in name. Try: valid-name".to_string())
            } else {
                Ok(())
            }
        }).unwrap();
        
        assert_eq!(result, "valid-name");
        
        // Should show error immediately, not after completion
        assert!(ui.outputs.iter().any(|o| o.contains("Invalid character")));
        assert!(ui.outputs.iter().any(|o| o.contains("Try: valid-name")));
    }

    #[test]
    fn test_no_generic_error_messages() {
        let mut ui = MockTerminalUI::new(vec![]);
        
        let error = CreationError::InvalidName {
            name: "bad name".to_string(),
            suggestion: "bad-name".to_string(),
        };
        
        ui.show_error(&error);
        
        let error_output = &ui.outputs[0];
        
        // Should not contain generic messages
        assert!(!error_output.contains("Something went wrong"));
        assert!(!error_output.contains("Please try again"));
        assert!(!error_output.contains("An error occurred"));
        
        // Should contain specific, actionable information
        assert!(error_output.contains("Invalid skill name 'bad name'"));
        assert!(error_output.contains("Try: bad-name"));
    }
}

#[cfg(test)]
mod cognitive_load_management {
    use super::*;

    #[test]
    fn test_progressive_disclosure_by_mode() {
        // Quick mode - minimal options
        let quick_prompts = SkillCreationFlow::get_prompts(SkillMode::Quick);
        assert_eq!(quick_prompts.len(), 1); // Only command
        
        // Guided mode - essential options
        let guided_prompts = SkillCreationFlow::get_prompts(SkillMode::Guided);
        assert_eq!(guided_prompts.len(), 3); // command, description, security
        
        // Expert mode - all options
        let expert_prompts = SkillCreationFlow::get_prompts(SkillMode::Expert);
        assert!(expert_prompts.len() >= 5); // All configuration options
    }

    #[test]
    fn test_smart_defaults_reduce_decisions() {
        let fixtures = TestFixtures::new();
        
        // Create Python project context
        std::fs::write(fixtures.temp_dir.path().join("main.py"), "print('hello')").unwrap();
        
        let context = CreationContext::new(fixtures.temp_dir.path()).unwrap();
        let defaults = context.suggest_defaults(&CreationType::Skill);
        
        // Should pre-fill obvious choices
        assert_eq!(defaults.skill_type, Some(SkillType::CodeInline)); // Python → code_inline
        assert!(defaults.command.as_ref().unwrap().contains("python")); // Python command
        assert!(!defaults.description.is_empty()); // Auto-generated description
        
        // User only needs to confirm/modify, not decide everything
    }

    #[test]
    fn test_contextual_help_not_tutorials() {
        let mut ui = MockTerminalUI::new(vec!["help".to_string()]);
        
        let help_output = ui.show_contextual_help("skill_type");
        
        // Should be concise, contextual help
        assert!(help_output.contains("code_inline: Run shell commands"));
        assert!(help_output.contains("conversation: Chat assistant"));
        assert!(!help_output.contains("Welcome to skill creation")); // No tutorial intro
        assert!(!help_output.contains("Let's learn about")); // No educational tone
        assert!(help_output.len() < 200); // Concise
    }

    #[test]
    fn test_consistent_interaction_patterns() {
        let mut ui = MockTerminalUI::new(vec![
            "test".to_string(),
            "y".to_string(),
            "n".to_string(),
        ]);
        
        // All prompts should follow consistent patterns
        let name = ui.prompt_required("Name");
        let confirm1 = ui.confirm("Enable security?");
        let confirm2 = ui.confirm("Add parameters?");
        
        // Verify consistent formatting
        assert!(ui.outputs.iter().all(|o| o.ends_with(": ") || o.ends_with("? ")));
        assert!(ui.outputs.iter().all(|o| !o.contains("Please enter")));
        assert!(ui.outputs.iter().all(|o| !o.contains("Would you like to")));
    }
}
