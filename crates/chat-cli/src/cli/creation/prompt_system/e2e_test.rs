#[cfg(test)]
mod e2e_tests {
    use eyre::Result;

    use crate::cli::creation::flows::*;
    use crate::cli::creation::prompt_system::*;
    use crate::cli::creation::ui::MockTerminalUI;

    #[test]
    fn test_end_to_end_template_creation() -> Result<()> {
        // Simulate user creating an assistant from template
        let mut ui = MockTerminalUI::new(vec![
            "1".to_string(), // Select code_reviewer template
            "".to_string(),  // Use default name
            "y".to_string(), // Use default role
            "y".to_string(), // Create
        ]);

        let template = create_skill_with_prompt_builder(&mut ui)?;

        // Verify the template was created correctly
        assert_eq!(template.name, "Code Reviewer");
        assert_eq!(template.category, TemplateCategory::CodeReviewer);
        assert_eq!(template.difficulty, DifficultyLevel::Advanced);
        assert!(!template.capabilities.is_empty());
        assert!(!template.constraints.is_empty());

        Ok(())
    }

    #[test]
    fn test_end_to_end_custom_creation() -> Result<()> {
        // Simulate user creating a custom assistant
        let mut ui = MockTerminalUI::new(vec![
            "Python Helper".to_string(),     // Name
            "Helps with Python".to_string(), // Description
            "1".to_string(),                 // Code specialization
            "".to_string(),                  // Use default role
            "1,2".to_string(),               // Select capabilities
            "1".to_string(),                 // Select constraint
            "2".to_string(),                 // Intermediate difficulty
            "n".to_string(),                 // No example
            "y".to_string(),                 // Create
        ]);

        let template = create_custom_skill(&mut ui)?;

        // Verify the template
        assert_eq!(template.name, "Python Helper");
        assert_eq!(template.description, "Helps with Python");
        assert_eq!(template.difficulty, DifficultyLevel::Intermediate);

        Ok(())
    }

    #[test]
    fn test_builder_validation_flow() -> Result<()> {
        // Test that validation works in the flow
        let builder = PromptBuilder::new()
            .with_name("Test".to_string())
            .with_description("Test assistant".to_string())
            .with_role("You are a test assistant".to_string())
            .add_capability("testing".to_string());

        let validation = builder.validate()?;
        assert!(validation.is_valid);
        assert!(validation.score > 0.0);

        let template = builder.build()?;
        assert_eq!(template.name, "Test");

        Ok(())
    }
}
