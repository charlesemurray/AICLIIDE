//! Integration of prompt builder with skill creation

use eyre::Result;

use crate::cli::creation::TerminalUI;
use crate::cli::creation::prompt_system::{
    InteractivePromptBuilder,
    PromptTemplate,
};

/// Create a skill using the interactive prompt builder
pub fn create_skill_with_prompt_builder<T: TerminalUI>(ui: &mut T) -> Result<PromptTemplate> {
    let mut builder = InteractivePromptBuilder::new(ui);
    builder.create_from_template()
}

/// Create a custom skill from scratch
pub fn create_custom_skill<T: TerminalUI>(ui: &mut T) -> Result<PromptTemplate> {
    let mut builder = InteractivePromptBuilder::new(ui);
    builder.create_custom()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::creation::prompt_system::TemplateCategory;
    use crate::cli::creation::ui::MockTerminalUI;

    #[test]
    fn test_create_skill_with_prompt_builder() -> Result<()> {
        let mut ui = MockTerminalUI::new(vec![
            "1".to_string(), // Select code_reviewer
            "".to_string(),  // Use default name
            "y".to_string(), // Use default role
            "y".to_string(), // Create
        ]);

        let template = create_skill_with_prompt_builder(&mut ui)?;
        assert_eq!(template.category, TemplateCategory::CodeReviewer);
        Ok(())
    }

    #[test]
    fn test_create_custom_skill() -> Result<()> {
        let mut ui = MockTerminalUI::new(vec![
            "My Skill".to_string(),
            "Test skill".to_string(),
            "1".to_string(), // Code
            "".to_string(),  // Default role
            "1".to_string(), // Capability
            "1".to_string(), // Constraint
            "1".to_string(), // Beginner
            "n".to_string(), // No example
            "y".to_string(), // Create
        ]);

        let template = create_custom_skill(&mut ui)?;
        assert_eq!(template.name, "My Skill");
        Ok(())
    }
}
