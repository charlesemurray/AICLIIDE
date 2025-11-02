use std::collections::HashMap;

use eyre::Result;

use crate::cli::creation::enhanced_prompts::EnhancedPrompts;
use crate::cli::creation::template_loader::SimpleTemplateLoader;
use crate::cli::creation::{CreationType, SemanticColor, TerminalUI};

pub struct InteractiveCreationFlow<T: TerminalUI> {
    ui: T,
    template_loader: SimpleTemplateLoader,
}

impl<T: TerminalUI> InteractiveCreationFlow<T> {
    pub async fn new(ui: T) -> Result<Self> {
        Ok(Self {
            ui,
            template_loader: SimpleTemplateLoader::new(),
        })
    }

    pub async fn run(&mut self, creation_type: CreationType) -> Result<String> {
        match creation_type {
            CreationType::Skill => self.create_skill().await,
            CreationType::CustomCommand => self.create_command().await,
            CreationType::Agent => self.create_agent().await,
        }
    }

    async fn create_skill(&mut self) -> Result<String> {
        self.ui
            .show_message("🎯 Creating a new skill...\n", SemanticColor::Info);

        let name = EnhancedPrompts::prompt_skill_name(&mut self.ui)?;
        let description = EnhancedPrompts::prompt_description(&mut self.ui, "skill")?;
        let command = EnhancedPrompts::prompt_command(&mut self.ui)?;

        let mut params = HashMap::new();
        params.insert("name".to_string(), name.clone());
        params.insert("description".to_string(), description.unwrap_or_default());
        params.insert("command".to_string(), command);

        let rendered = self.template_loader.render_template("skill_basic", &params)?;

        if EnhancedPrompts::show_preview(&mut self.ui, &rendered, "skill")? {
            self.ui
                .show_message(&format!("✅ Created skill: {}\n", name), SemanticColor::Success);
            Ok(rendered)
        } else {
            Err(eyre::eyre!("Creation cancelled by user"))
        }
    }

    async fn create_command(&mut self) -> Result<String> {
        self.ui
            .show_message("⚡ Creating a new command...\n", SemanticColor::Info);

        let name = EnhancedPrompts::prompt_skill_name(&mut self.ui)?; // Reuse skill name validation
        let description = EnhancedPrompts::prompt_description(&mut self.ui, "command")?;
        let command = EnhancedPrompts::prompt_command(&mut self.ui)?;
        let args = self.ui.prompt_optional("Arguments (JSON array)", Some("[]"))?;

        let mut params = HashMap::new();
        params.insert("name".to_string(), name.clone());
        params.insert("description".to_string(), description.unwrap_or_default());
        params.insert("command".to_string(), command);
        params.insert("args".to_string(), args.unwrap_or_else(|| "[]".to_string()));

        let rendered = self.template_loader.render_template("command_basic", &params)?;

        if EnhancedPrompts::show_preview(&mut self.ui, &rendered, "command")? {
            self.ui
                .show_message(&format!("✅ Created command: {}\n", name), SemanticColor::Success);
            Ok(rendered)
        } else {
            Err(eyre::eyre!("Creation cancelled by user"))
        }
    }

    async fn create_agent(&mut self) -> Result<String> {
        self.ui
            .show_message("🤖 Creating a new agent...\n", SemanticColor::Info);

        let name = EnhancedPrompts::prompt_skill_name(&mut self.ui)?; // Reuse skill name validation
        let description = EnhancedPrompts::prompt_description(&mut self.ui, "agent")?;
        let role = EnhancedPrompts::prompt_agent_role(&mut self.ui)?;
        let capabilities = EnhancedPrompts::prompt_capabilities(&mut self.ui)?;

        let mut params = HashMap::new();
        params.insert("name".to_string(), name.clone());
        params.insert("description".to_string(), description.unwrap_or_default());
        params.insert("role".to_string(), role);

        // Format capabilities as JSON array
        let caps = if let Some(caps_str) = capabilities {
            caps_str
                .split(',')
                .map(|s| format!("\"{}\"", s.trim()))
                .collect::<Vec<_>>()
                .join(", ")
        } else {
            String::new()
        };
        params.insert("capabilities".to_string(), caps);

        let rendered = self.template_loader.render_template("agent_basic", &params)?;

        if EnhancedPrompts::show_preview(&mut self.ui, &rendered, "agent")? {
            self.ui
                .show_message(&format!("✅ Created agent: {}\n", name), SemanticColor::Success);
            Ok(rendered)
        } else {
            Err(eyre::eyre!("Creation cancelled by user"))
        }
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
    async fn test_skill_creation_with_preview_accept() {
        let ui = MockTerminalUI::new(vec![
            "test-skill".to_string(),
            "A test skill".to_string(),
            "echo hello".to_string(),
            "y".to_string(), // Accept preview
        ]);
        let mut flow = InteractiveCreationFlow::new(ui).await.unwrap();

        let result = flow.run(CreationType::Skill).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("test-skill"));
        assert!(output.contains("echo hello"));
    }

    #[tokio::test]
    async fn test_skill_creation_with_preview_reject() {
        let ui = MockTerminalUI::new(vec![
            "test-skill".to_string(),
            "A test skill".to_string(),
            "echo hello".to_string(),
            "n".to_string(), // Reject preview
        ]);
        let mut flow = InteractiveCreationFlow::new(ui).await.unwrap();

        let result = flow.run(CreationType::Skill).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cancelled"));
    }

    #[tokio::test]
    async fn test_command_creation_flow() {
        let ui = MockTerminalUI::new(vec![
            "test-command".to_string(),
            "A test command".to_string(),
            "ls -la".to_string(),
            "[\"--color\"]".to_string(),
            "y".to_string(), // Accept preview
        ]);
        let mut flow = InteractiveCreationFlow::new(ui).await.unwrap();

        let result = flow.run(CreationType::CustomCommand).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("test-command"));
        assert!(output.contains("ls -la"));
    }

    #[tokio::test]
    async fn test_agent_creation_flow() {
        let ui = MockTerminalUI::new(vec![
            "test-agent".to_string(),
            "A test agent".to_string(),
            "assistant".to_string(),
            "help, analyze, suggest".to_string(),
            "y".to_string(), // Accept preview
        ]);
        let mut flow = InteractiveCreationFlow::new(ui).await.unwrap();

        let result = flow.run(CreationType::Agent).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("test-agent"));
        assert!(output.contains("assistant"));
    }
}
