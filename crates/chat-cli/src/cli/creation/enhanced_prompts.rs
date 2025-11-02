use eyre::Result;

use crate::cli::creation::{SemanticColor, TerminalUI};

pub struct EnhancedPrompts;

impl EnhancedPrompts {
    pub fn prompt_skill_name<T: TerminalUI>(ui: &mut T) -> Result<String> {
        ui.show_message(
            "💡 Skill names should be descriptive and use kebab-case (e.g., 'git-status', 'deploy-app')\n",
            SemanticColor::Info,
        );

        loop {
            let name = ui.prompt_required("Skill name")?;
            if Self::validate_skill_name(&name) {
                return Ok(name);
            }
            ui.show_message(
                "❌ Invalid name. Use lowercase letters, numbers, and hyphens only.\n",
                SemanticColor::Error,
            );
        }
    }

    pub fn prompt_command<T: TerminalUI>(ui: &mut T) -> Result<String> {
        ui.show_message(
            "⚡ Enter the command that this skill will execute (e.g., 'git status', 'docker ps')\n",
            SemanticColor::Info,
        );

        loop {
            let command = ui.prompt_required("Command to execute")?;
            if Self::validate_command(&command) {
                return Ok(command);
            }
            ui.show_message(
                "❌ Command cannot be empty or contain only whitespace.\n",
                SemanticColor::Error,
            );
        }
    }

    pub fn prompt_description<T: TerminalUI>(ui: &mut T, item_type: &str) -> Result<Option<String>> {
        ui.show_message(
            &format!("📝 Provide a brief description of what this {} does\n", item_type),
            SemanticColor::Info,
        );
        Ok(ui.prompt_optional("Description", None)?)
    }

    pub fn prompt_agent_role<T: TerminalUI>(ui: &mut T) -> Result<String> {
        ui.show_message(
            "🤖 Define the agent's role (e.g., 'code reviewer', 'documentation writer', 'assistant')\n",
            SemanticColor::Info,
        );

        loop {
            let role = ui.prompt_required("Agent role")?;
            if Self::validate_role(&role) {
                return Ok(role);
            }
            ui.show_message(
                "❌ Role cannot be empty or contain only whitespace.\n",
                SemanticColor::Error,
            );
        }
    }

    pub fn prompt_capabilities<T: TerminalUI>(ui: &mut T) -> Result<Option<String>> {
        ui.show_message("🔧 List the agent's capabilities, separated by commas (e.g., 'analyze code, suggest improvements, write tests')\n", SemanticColor::Info);
        Ok(ui.prompt_optional("Capabilities (comma-separated)", None)?)
    }

    pub fn show_preview<T: TerminalUI>(ui: &mut T, content: &str, item_type: &str) -> Result<bool> {
        ui.show_message(&format!("\n📋 Preview of your {}:\n", item_type), SemanticColor::Info);
        ui.show_message(&format!("```json\n{}\n```\n", content), SemanticColor::Success);

        ui.confirm("Create this item?")
    }

    fn validate_skill_name(name: &str) -> bool {
        !name.trim().is_empty()
            && name
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
            && !name.starts_with('-')
            && !name.ends_with('-')
    }

    fn validate_command(command: &str) -> bool {
        !command.trim().is_empty()
    }

    fn validate_role(role: &str) -> bool {
        !role.trim().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::creation::tests::MockTerminalUI;

    #[test]
    fn test_skill_name_validation() {
        assert!(EnhancedPrompts::validate_skill_name("valid-name"));
        assert!(EnhancedPrompts::validate_skill_name("test123"));
        assert!(!EnhancedPrompts::validate_skill_name("Invalid-Name"));
        assert!(!EnhancedPrompts::validate_skill_name("-invalid"));
        assert!(!EnhancedPrompts::validate_skill_name("invalid-"));
        assert!(!EnhancedPrompts::validate_skill_name(""));
    }

    #[test]
    fn test_command_validation() {
        assert!(EnhancedPrompts::validate_command("git status"));
        assert!(EnhancedPrompts::validate_command("ls -la"));
        assert!(!EnhancedPrompts::validate_command(""));
        assert!(!EnhancedPrompts::validate_command("   "));
    }

    #[test]
    fn test_enhanced_skill_name_prompt() {
        let mut ui = MockTerminalUI::new(vec![
            "invalid-Name".to_string(), // First invalid input
            "valid-name".to_string(),   // Valid input
        ]);

        let result = EnhancedPrompts::prompt_skill_name(&mut ui).unwrap();
        assert_eq!(result, "valid-name");
    }
}
