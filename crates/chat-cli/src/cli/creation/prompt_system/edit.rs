//! Edit existing assistants

use eyre::Result;

use super::{DifficultyLevel, PromptTemplate, TemplateCategory};
use crate::cli::creation::{SemanticColor, TerminalUI};

pub struct AssistantEditor<'a, T: TerminalUI> {
    ui: &'a mut T,
    template: PromptTemplate,
}

impl<'a, T: TerminalUI> AssistantEditor<'a, T> {
    pub fn new(ui: &'a mut T, template: PromptTemplate) -> Self {
        Self { ui, template }
    }

    pub fn edit(mut self) -> Result<PromptTemplate> {
        self.ui.show_message(
            &format!("Editing: {} ({})", self.template.name, self.template.id),
            SemanticColor::Info,
        );

        loop {
            let choice = self.ui.select_option(
                "\nWhat would you like to edit?",
                &[
                    ("name", "Name"),
                    ("description", "Description"),
                    ("role", "Role"),
                    ("capabilities", "Capabilities (add/remove)"),
                    ("constraints", "Constraints (add/remove)"),
                    ("difficulty", "Difficulty level"),
                    ("done", "Done editing"),
                ],
            )?;

            match choice.as_str() {
                "name" => self.edit_name()?,
                "description" => self.edit_description()?,
                "role" => self.edit_role()?,
                "capabilities" => self.edit_capabilities()?,
                "constraints" => self.edit_constraints()?,
                "difficulty" => self.edit_difficulty()?,
                "done" => break,
                _ => {},
            }
        }

        if self.ui.confirm("Save changes")? {
            self.template.updated_at = chrono::Utc::now();
            Ok(self.template)
        } else {
            Err(eyre::eyre!("Edit cancelled"))
        }
    }

    fn edit_name(&mut self) -> Result<()> {
        self.ui
            .show_message(&format!("Current: {}", self.template.name), SemanticColor::Debug);
        let new_name = self.ui.prompt_required("New name")?;
        self.template.name = new_name;
        Ok(())
    }

    fn edit_description(&mut self) -> Result<()> {
        self.ui
            .show_message(&format!("Current: {}", self.template.description), SemanticColor::Debug);
        let new_desc = self.ui.prompt_required("New description")?;
        self.template.description = new_desc;
        Ok(())
    }

    fn edit_role(&mut self) -> Result<()> {
        self.ui
            .show_message(&format!("Current: {}", self.template.role), SemanticColor::Debug);
        let new_role = self.ui.prompt_required("New role")?;
        self.template.role = new_role;
        Ok(())
    }

    fn edit_capabilities(&mut self) -> Result<()> {
        loop {
            self.ui.show_message(
                &format!("Current: {:?}", self.template.capabilities),
                SemanticColor::Debug,
            );

            let action = self.ui.select_option(
                "Action:",
                &[
                    ("add", "Add capability"),
                    ("remove", "Remove capability"),
                    ("done", "Done"),
                ],
            )?;

            match action.as_str() {
                "add" => {
                    let cap = self.ui.prompt_required("New capability")?;
                    self.template.capabilities.push(cap);
                },
                "remove" => {
                    if self.template.capabilities.is_empty() {
                        self.ui
                            .show_message("No capabilities to remove", SemanticColor::Warning);
                        continue;
                    }
                    let options: Vec<_> = self
                        .template
                        .capabilities
                        .iter()
                        .map(|c| (c.as_str(), c.as_str()))
                        .collect();
                    let to_remove = self.ui.select_option("Remove which?", &options)?;
                    self.template.capabilities.retain(|c| c != &to_remove);
                },
                _ => break,
            }
        }
        Ok(())
    }

    fn edit_constraints(&mut self) -> Result<()> {
        loop {
            self.ui.show_message(
                &format!("Current: {:?}", self.template.constraints),
                SemanticColor::Debug,
            );

            let action = self.ui.select_option(
                "Action:",
                &[
                    ("add", "Add constraint"),
                    ("remove", "Remove constraint"),
                    ("done", "Done"),
                ],
            )?;

            match action.as_str() {
                "add" => {
                    let constraint = self.ui.prompt_required("New constraint")?;
                    self.template.constraints.push(constraint);
                },
                "remove" => {
                    if self.template.constraints.is_empty() {
                        self.ui.show_message("No constraints to remove", SemanticColor::Warning);
                        continue;
                    }
                    let options: Vec<_> = self
                        .template
                        .constraints
                        .iter()
                        .map(|c| (c.as_str(), c.as_str()))
                        .collect();
                    let to_remove = self.ui.select_option("Remove which?", &options)?;
                    self.template.constraints.retain(|c| c != &to_remove);
                },
                _ => break,
            }
        }
        Ok(())
    }

    fn edit_difficulty(&mut self) -> Result<()> {
        self.ui.show_message(
            &format!("Current: {:?}", self.template.difficulty),
            SemanticColor::Debug,
        );

        let choice = self.ui.select_option(
            "New difficulty:",
            &[
                ("beginner", "Beginner"),
                ("intermediate", "Intermediate"),
                ("advanced", "Advanced"),
            ],
        )?;

        self.template.difficulty = match choice.as_str() {
            "beginner" => DifficultyLevel::Beginner,
            "advanced" => DifficultyLevel::Advanced,
            _ => DifficultyLevel::Intermediate,
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::creation::prompt_system::PromptBuilder;
    use crate::cli::creation::prompt_system::creation_builder::CreationBuilder;
    use crate::cli::creation::ui::MockTerminalUI;

    #[test]
    fn test_edit_name() -> Result<()> {
        let template = PromptBuilder::new()
            .with_name("Old Name".to_string())
            .with_role("Role".to_string())
            .build()?;

        let mut ui = MockTerminalUI::new(vec![
            "1".to_string(),        // Edit name
            "New Name".to_string(), // New name
            "7".to_string(),        // Done
            "y".to_string(),        // Save
        ]);

        let editor = AssistantEditor::new(&mut ui, template);
        let result = editor.edit()?;

        assert_eq!(result.name, "New Name");
        Ok(())
    }

    #[test]
    fn test_edit_capabilities() -> Result<()> {
        let template = PromptBuilder::new()
            .with_name("Test".to_string())
            .with_role("Role".to_string())
            .add_capability("old".to_string())
            .build()?;

        let mut ui = MockTerminalUI::new(vec![
            "4".to_string(),   // Edit capabilities
            "1".to_string(),   // Add
            "new".to_string(), // New capability
            "3".to_string(),   // Done with capabilities
            "7".to_string(),   // Done editing
            "y".to_string(),   // Save
        ]);

        let editor = AssistantEditor::new(&mut ui, template);
        let result = editor.edit()?;

        assert!(result.capabilities.contains(&"new".to_string()));
        Ok(())
    }
}
