#[cfg(test)]
mod interactive_tests {
    use crate::cli::creation::prompt_system::*;
    use crate::cli::creation::ui::MockTerminalUI;
    use eyre::Result;

    #[test]
    fn test_create_from_template_code_reviewer() -> Result<()> {
        let mut ui = MockTerminalUI::new(vec![
            "1".to_string(),           // Choose code_reviewer
            "".to_string(),            // Use default name
            "y".to_string(),           // Use default role
            "y".to_string(),           // Create
        ]);

        let mut builder = InteractivePromptBuilder::new(&mut ui);
        let template = builder.create_from_template()?;

        assert_eq!(template.name, "Code Reviewer");
        assert_eq!(template.category, TemplateCategory::CodeReviewer);
        Ok(())
    }

    #[test]
    fn test_create_custom_assistant() -> Result<()> {
        let mut ui = MockTerminalUI::new(vec![
            "5".to_string(),                    // Choose custom
            "Test Assistant".to_string(),       // Name
            "A test assistant".to_string(),     // Description
            "1".to_string(),                    // Code specialization
            "".to_string(),                     // Use default role
            "1,2".to_string(),                  // Capabilities
            "1,2".to_string(),                  // Constraints
            "1".to_string(),                    // Beginner difficulty
            "n".to_string(),                    // No example
            "y".to_string(),                    // Create
        ]);

        let mut builder = InteractivePromptBuilder::new(&mut ui);
        let template = builder.create_custom()?;

        assert_eq!(template.name, "Test Assistant");
        assert_eq!(template.description, "A test assistant");
        assert_eq!(template.difficulty, DifficultyLevel::Beginner);
        Ok(())
    }

    #[test]
    fn test_create_with_example() -> Result<()> {
        let mut ui = MockTerminalUI::new(vec![
            "My Assistant".to_string(),
            "Test description".to_string(),
            "4".to_string(),                    // General
            "".to_string(),                     // Use default role
            "".to_string(),                     // No capabilities
            "".to_string(),                     // No constraints
            "2".to_string(),                    // Intermediate
            "y".to_string(),                    // Add example
            "test input".to_string(),
            "test output".to_string(),
            "y".to_string(),                    // Create
        ]);

        let mut builder = InteractivePromptBuilder::new(&mut ui);
        let template = builder.create_custom()?;

        assert_eq!(template.examples.len(), 1);
        assert_eq!(template.examples[0].input, "test input");
        Ok(())
    }

    #[test]
    fn test_template_selection_options() -> Result<()> {
        let mut ui = MockTerminalUI::new(vec![
            "2".to_string(),           // Doc writer
            "".to_string(),            // Default name
            "y".to_string(),           // Default role
            "y".to_string(),           // Create
        ]);

        let mut builder = InteractivePromptBuilder::new(&mut ui);
        let template = builder.create_from_template()?;

        assert_eq!(template.category, TemplateCategory::DocumentationWriter);
        Ok(())
    }

    #[test]
    fn test_custom_role() -> Result<()> {
        let mut ui = MockTerminalUI::new(vec![
            "1".to_string(),                    // Code reviewer
            "Custom Reviewer".to_string(),      // Custom name
            "n".to_string(),                    // Don't use default role
            "You are a custom expert".to_string(), // Custom role
            "y".to_string(),                    // Create
        ]);

        let mut builder = InteractivePromptBuilder::new(&mut ui);
        let template = builder.create_from_template()?;

        assert_eq!(template.name, "Custom Reviewer");
        assert!(template.role.contains("custom expert"));
        Ok(())
    }
}
