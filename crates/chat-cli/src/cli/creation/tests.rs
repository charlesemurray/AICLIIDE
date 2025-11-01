//! Basic integration tests for creation system

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_command_creation_basic() {
        let temp_dir = TempDir::new().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        // Test basic command creation flow
        let flow = CommandCreationFlow::new("test-cmd".to_string(), CreationMode::Quick);
        assert!(flow.is_ok());

        let flow = flow.unwrap();
        assert_eq!(flow.get_config().name, "test-cmd");
        assert_eq!(flow.creation_type(), CreationType::CustomCommand);
    }

    #[test]
    fn test_creation_context() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create a Python project
        std::fs::write(temp_dir.path().join("requirements.txt"), "requests==2.28.0").unwrap();
        
        let context = CreationContext::new(temp_dir.path()).unwrap();
        let defaults = context.suggest_defaults(&CreationType::Skill);
        
        assert_eq!(defaults.skill_type, Some(SkillType::CodeInline));
    }

    #[test]
    fn test_cisco_style_cli_parsing() {
        use clap::Parser;
        
        // Test basic command parsing
        let args = CreateArgs::try_parse_from(&["create", "command", "mycmd"]).unwrap();
        match args.command {
            CreateCommand::Command { name, mode } => {
                assert_eq!(name, "mycmd");
                assert_eq!(mode, None);
            }
            _ => panic!("Expected Command command"),
        }

        // Test with mode
        let args = CreateArgs::try_parse_from(&["create", "command", "mycmd", "quick"]).unwrap();
        match args.command {
            CreateCommand::Command { name, mode } => {
                assert_eq!(name, "mycmd");
                assert_eq!(mode, Some(CommandMode::Quick));
            }
            _ => panic!("Expected Command command"),
        }
    }

    #[tokio::test]
    async fn test_end_to_end_command_creation() {
        let temp_dir = TempDir::new().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        // Create .q-commands directory
        std::fs::create_dir_all(temp_dir.path().join(".q-commands")).unwrap();

        // Test command creation with mock UI
        let mut ui = MockTerminalUI::new(vec![
            "echo hello".to_string(),    // command
            "Test command".to_string(),  // description
            "y".to_string(),             // confirm
        ]);

        let flow = CommandCreationFlow::new("test-cmd".to_string(), CreationMode::Guided)
            .unwrap()
            .with_ui(Box::new(ui));

        let assistant = CreationAssistant::new(flow);
        let result = assistant.run().await;
        
        assert!(result.is_ok());

        // Verify file was created
        let cmd_file = temp_dir.path().join(".q-commands").join("test-cmd.json");
        assert!(cmd_file.exists());
    }
}
