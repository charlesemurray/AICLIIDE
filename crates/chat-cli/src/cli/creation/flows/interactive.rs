use eyre::Result;
use crate::cli::creation::{CreationType, TerminalUI, SemanticColor};
use crate::cli::creation::prompt_system::{PromptSystem, TemplateInfo};

pub struct InteractiveCreationFlow<T: TerminalUI> {
    ui: T,
    prompt_system: PromptSystem,
}

impl<T: TerminalUI> InteractiveCreationFlow<T> {
    pub async fn new(ui: T) -> Result<Self> {
        let prompt_system = PromptSystem::new().await?;
        Ok(Self { ui, prompt_system })
    }

    pub async fn run(&mut self, creation_type: CreationType) -> Result<String> {
        match creation_type {
            CreationType::Skill => self.create_skill().await,
            CreationType::CustomCommand => self.create_command().await,
            CreationType::Agent => self.create_agent().await,
        }
    }

    async fn create_skill(&mut self) -> Result<String> {
        self.ui.show_message("🎯 Creating a new skill...\n", SemanticColor::Info);
        
        // Get available templates
        let templates = self.prompt_system.list_templates().await?;
        let template = self.select_template(&templates, "skill creation")?;
        
        // Get basic info
        let name = self.ui.prompt_required("Skill name")?;
        let description = self.ui.prompt_optional("Description", None)?;
        
        // Get template and render with parameters
        let template_obj = self.prompt_system.get_template(&template.id).await?;
        let mut params = std::collections::HashMap::new();
        params.insert("name".to_string(), name.clone());
        if let Some(desc) = description {
            params.insert("description".to_string(), desc);
        }
        
        let rendered = self.prompt_system.render_template(&template_obj, &params).await?;
        
        self.ui.show_message(&format!("✅ Created skill: {}\n", name), SemanticColor::Success);
        Ok(rendered)
    }

    async fn create_command(&mut self) -> Result<String> {
        self.ui.show_message("⚡ Creating a new command...\n", SemanticColor::Info);
        
        let templates = self.prompt_system.list_templates().await?;
        let template = self.select_template(&templates, "command creation")?;
        
        let name = self.ui.prompt_required("Command name")?;
        let command = self.ui.prompt_required("Command to execute")?;
        
        let template_obj = self.prompt_system.get_template(&template.id).await?;
        let mut params = std::collections::HashMap::new();
        params.insert("name".to_string(), name.clone());
        params.insert("command".to_string(), command);
        
        let rendered = self.prompt_system.render_template(&template_obj, &params).await?;
        
        self.ui.show_message(&format!("✅ Created command: {}\n", name), SemanticColor::Success);
        Ok(rendered)
    }

    async fn create_agent(&mut self) -> Result<String> {
        self.ui.show_message("🤖 Creating a new agent...\n", SemanticColor::Info);
        
        let templates = self.prompt_system.list_templates().await?;
        let template = self.select_template(&templates, "agent creation")?;
        
        let name = self.ui.prompt_required("Agent name")?;
        let role = self.ui.prompt_required("Agent role")?;
        
        let template_obj = self.prompt_system.get_template(&template.id).await?;
        let mut params = std::collections::HashMap::new();
        params.insert("name".to_string(), name.clone());
        params.insert("role".to_string(), role);
        
        let rendered = self.prompt_system.render_template(&template_obj, &params).await?;
        
        self.ui.show_message(&format!("✅ Created agent: {}\n", name), SemanticColor::Success);
        Ok(rendered)
    }

    fn select_template(&mut self, templates: &[TemplateInfo], use_case: &str) -> Result<TemplateInfo> {
        if templates.is_empty() {
            return Err(eyre::eyre!("No templates available"));
        }

        if templates.len() == 1 {
            // Auto-select if only one template
            return Ok(templates[0].clone());
        }

        // Show template options
        self.ui.show_message("Available templates:\n", SemanticColor::Info);
        let options: Vec<(&str, &str)> = templates.iter()
            .map(|t| (t.name.as_str(), t.description.as_str()))
            .collect();

        let selected = self.ui.select_option("Select template", &options)?;
        
        templates.iter()
            .find(|t| t.name == selected)
            .cloned()
            .ok_or_else(|| eyre::eyre!("Invalid template selection"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::creation::tests::MockTerminalUI;

    #[tokio::test]
    async fn test_interactive_flow_creation() {
        let ui = MockTerminalUI::new(vec![]);
        let flow = InteractiveCreationFlow::new(ui).await;
        assert!(flow.is_ok());
    }

    #[tokio::test]
    async fn test_skill_creation_flow() {
        let ui = MockTerminalUI::new(vec![
            "test_skill".to_string(),
            "".to_string(), // Empty description
        ]);
        let mut flow = InteractiveCreationFlow::new(ui).await.unwrap();
        
        let result = flow.run(CreationType::Skill).await;
        assert!(result.is_ok());
    }
}
