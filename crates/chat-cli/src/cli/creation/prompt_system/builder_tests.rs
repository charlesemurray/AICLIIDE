#[cfg(test)]
mod builder_tests {
    use crate::cli::creation::prompt_system::*;
    use eyre::Result;

    #[test]
    fn test_prompt_builder_basic() -> Result<()> {
        let template = PromptBuilder::new()
            .with_name("Test Assistant".to_string())
            .with_description("A test assistant".to_string())
            .with_role("You are a helpful test assistant".to_string())
            .add_capability("testing".to_string())
            .add_constraint("be helpful".to_string())
            .build()?;

        assert_eq!(template.name, "Test Assistant");
        assert_eq!(template.role, "You are a helpful test assistant");
        assert_eq!(template.capabilities.len(), 1);
        assert_eq!(template.constraints.len(), 1);
        Ok(())
    }

    #[test]
    fn test_prompt_builder_validation() -> Result<()> {
        let builder = PromptBuilder::new()
            .with_name("Test".to_string())
            .with_role("Short".to_string());

        let validation = builder.validate()?;
        assert!(validation.is_valid); // Should be valid even with warnings
        assert!(!validation.issues.is_empty()); // But should have warnings
        Ok(())
    }

    #[test]
    fn test_prompt_builder_preview() {
        let builder = PromptBuilder::new()
            .with_role("You are a code reviewer".to_string())
            .add_capability("security analysis".to_string())
            .add_constraint("be constructive".to_string());

        let preview = builder.preview();
        assert!(preview.contains("You are a code reviewer"));
        assert!(preview.contains("security analysis"));
        assert!(preview.contains("be constructive"));
    }

    #[test]
    fn test_command_builder_basic() -> Result<()> {
        let command = CommandBuilder::new()
            .with_name("test-cmd".to_string())
            .with_description("A test command".to_string())
            .with_command("echo".to_string())
            .add_parameter("hello".to_string())
            .with_timeout(30)
            .build()?;

        assert_eq!(command.name, "test-cmd");
        assert_eq!(command.command, "echo");
        assert_eq!(command.parameters.len(), 1);
        assert_eq!(command.timeout, Some(30));
        Ok(())
    }

    #[test]
    fn test_command_builder_validation() -> Result<()> {
        let builder = CommandBuilder::new()
            .with_name("test".to_string())
            .with_command("ls".to_string());

        let validation = builder.validate()?;
        assert!(validation.is_valid);
        Ok(())
    }

    #[test]
    fn test_command_builder_preview() {
        let builder = CommandBuilder::new()
            .with_command("git".to_string())
            .add_parameter("status".to_string())
            .add_parameter("--short".to_string());

        let preview = builder.preview();
        assert_eq!(preview, "git status --short");
    }

    #[test]
    fn test_builder_validation_errors() {
        let builder = PromptBuilder::new(); // No name
        let validation = builder.validate().unwrap();
        assert!(!validation.is_valid);
        
        let errors: Vec<_> = validation.issues
            .iter()
            .filter(|i| i.severity == IssueSeverity::Error)
            .collect();
        assert!(!errors.is_empty());
    }
}
