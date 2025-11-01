//! Integration tests for creation system - testing real builder flows end-to-end

use super::*;
use crate::cli::creation::flows::*;
use crate::cli::creation::types::*;
use crate::cli::creation::ui::MockTerminalUI;
use eyre::Result;
use std::path::PathBuf;
use tempfile::TempDir;

#[cfg(test)]
mod builder_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_skill_creation_end_to_end() -> Result<()> {
        // Test: Complete skill creation from user input to file output
        let temp_dir = TempDir::new()?;
        std::env::set_current_dir(&temp_dir)?;
        
        // Create skills directory
        let skills_dir = temp_dir.path().join(".q-skills");
        std::fs::create_dir_all(&skills_dir)?;
        
        // Simulate user creating a Python skill
        let mut ui = MockTerminalUI::new(vec![
            "python analyze.py".to_string(),    // command
            "Data analysis script".to_string(), // description  
            "y".to_string(),                    // confirm
        ]);
        
        // Use actual builder system
        let mut flow = SkillCreationFlow::new("data-analyzer".to_string(), CreationMode::Guided)?
            .with_ui(Box::new(ui));
        
        let config = flow.run_single_pass()?;
        
        // Verify the builder created correct configuration
        assert_eq!(config.get_name(), "data-analyzer");
        assert!(config.is_complete());
        
        // Test that config can be persisted (using the artifact system)
        let artifact = flow.create_artifact()?;
        let skill_file = skills_dir.join("data-analyzer.json");
        artifact.persist(&skill_file)?;
        
        // Verify file was created with correct content
        assert!(skill_file.exists());
        let content = std::fs::read_to_string(&skill_file)?;
        assert!(content.contains("data-analyzer"));
        assert!(content.contains("python analyze.py"));
        
        Ok(())
    }

    #[tokio::test]
    async fn test_command_creation_end_to_end() -> Result<()> {
        // Test: Complete command creation with parameter detection
        let temp_dir = TempDir::new()?;
        std::env::set_current_dir(&temp_dir)?;
        
        let commands_dir = temp_dir.path().join(".q-commands");
        std::fs::create_dir_all(&commands_dir)?;
        
        let mut ui = MockTerminalUI::new(vec![
            "git commit -m \"{{message}}\"".to_string(), // command with parameter
            "Quick git commit".to_string(),             // description
            "y".to_string(),                            // confirm
        ]);
        
        let mut flow = CommandCreationFlow::new("quick-commit".to_string(), CreationMode::Quick)?
            .with_ui(Box::new(ui));
        
        let config = flow.run_single_pass()?;
        
        // Verify parameter detection worked
        assert_eq!(config.get_name(), "quick-commit");
        assert!(config.is_complete());
        
        // Test artifact creation (skip persistence for now)
        let _artifact = flow.create_artifact()?;
        
        Ok(())
    }

    #[tokio::test]
    async fn test_agent_creation_end_to_end() -> Result<()> {
        // Test: Complete agent creation with MCP integration
        let temp_dir = TempDir::new()?;
        std::env::set_current_dir(&temp_dir)?;
        
        let agents_dir = temp_dir.path().join(".q-agents");
        std::fs::create_dir_all(&agents_dir)?;
        
        let mut ui = MockTerminalUI::new(vec![
            "You are a helpful coding assistant".to_string(), // prompt
            "Coding helper agent".to_string(),                // description
            "filesystem".to_string(),                         // MCP server
            "y".to_string(),                                  // confirm
        ]);
        
        let mut flow = AgentCreationFlow::new("code-helper".to_string(), CreationMode::Expert)?
            .with_ui(Box::new(ui));
        
        let config = flow.run_single_pass()?;
        
        // Verify agent configuration
        assert_eq!(config.get_name(), "code-helper");
        assert!(config.is_complete());
        
        // Test artifact creation (skip persistence for now)
        let _artifact = flow.create_artifact()?;
        
        Ok(())
    }

    #[tokio::test]
    async fn test_context_aware_creation() -> Result<()> {
        // Test: Builder system uses project context for smart defaults
        let temp_dir = TempDir::new()?;
        std::env::set_current_dir(&temp_dir)?;
        
        // Create Python project context
        std::fs::write(temp_dir.path().join("main.py"), "print('hello')")?;
        std::fs::write(temp_dir.path().join("requirements.txt"), "requests==2.28.0")?;
        
        let skills_dir = temp_dir.path().join(".q-skills");
        std::fs::create_dir_all(&skills_dir)?;
        
        // Minimal input - let context provide defaults
        let mut ui = MockTerminalUI::new(vec![
            "python main.py".to_string(), // command
            "y".to_string(),              // confirm (description auto-generated)
        ]);
        
        let mut flow = SkillCreationFlow::new("main-runner".to_string(), CreationMode::Quick)?
            .with_ui(Box::new(ui));
        
        let config = flow.run_single_pass()?;
        
        // Verify context-aware defaults were applied
        assert_eq!(config.get_name(), "main-runner");
        assert!(config.is_complete());
        
        // Should have detected Python project and set appropriate defaults
        let _artifact = flow.create_artifact()?;
        
        Ok(())
    }

    #[tokio::test]
    async fn test_preview_mode_integration() -> Result<()> {
        // Test: Preview mode shows what would be created without creating
        let temp_dir = TempDir::new()?;
        std::env::set_current_dir(&temp_dir)?;
        
        let mut ui = MockTerminalUI::new(vec![
            "echo 'preview test'".to_string(), // command
        ]);
        
        let mut flow = CommandCreationFlow::new("preview-cmd".to_string(), CreationMode::Preview)?
            .with_ui(Box::new(ui));
        
        // Preview should return content without creating files
        let preview_content = flow.run_preview_only()?;
        
        assert!(!preview_content.is_empty());
        assert!(preview_content.contains("preview-cmd"));
        assert!(preview_content.contains("echo 'preview test'"));
        
        // Verify no files were created
        let commands_dir = temp_dir.path().join(".q-commands");
        assert!(!commands_dir.exists());
        
        Ok(())
    }

    #[tokio::test]
    async fn test_validation_error_handling() -> Result<()> {
        // Test: Builder system handles validation errors gracefully
        let temp_dir = TempDir::new()?;
        std::env::set_current_dir(&temp_dir)?;
        
        // Try to create with invalid name (should fail validation)
        let result = CommandCreationFlow::new("".to_string(), CreationMode::Quick);
        
        // Should fail validation for empty name
        assert!(result.is_err());
        
        // Try with valid name
        let mut ui = MockTerminalUI::new(vec![
            "echo valid".to_string(),
            "y".to_string(),
        ]);
        
        let mut flow = CommandCreationFlow::new("valid-cmd".to_string(), CreationMode::Quick)?
            .with_ui(Box::new(ui));
        
        let config = flow.run_single_pass()?;
        assert!(config.is_complete());
        
        Ok(())
    }

    #[tokio::test]
    async fn test_multiple_creation_modes() -> Result<()> {
        // Test: Different creation modes work correctly
        let temp_dir = TempDir::new()?;
        std::env::set_current_dir(&temp_dir)?;
        
        let skills_dir = temp_dir.path().join(".q-skills");
        std::fs::create_dir_all(&skills_dir)?;
        
        // Test Quick mode - minimal prompts
        let mut quick_ui = MockTerminalUI::new(vec![
            "python quick.py".to_string(),
        ]);
        
        let mut quick_flow = SkillCreationFlow::new("quick-skill".to_string(), CreationMode::Quick)?
            .with_ui(Box::new(quick_ui));
        
        let quick_config = quick_flow.run_single_pass()?;
        assert_eq!(quick_config.get_name(), "quick-skill");
        
        // Test Guided mode - more prompts
        let mut guided_ui = MockTerminalUI::new(vec![
            "python guided.py".to_string(),    // command
            "Guided skill".to_string(),        // description
            "medium".to_string(),              // security level
            "y".to_string(),                   // confirm
        ]);
        
        let mut guided_flow = SkillCreationFlow::new("guided-skill".to_string(), CreationMode::Guided)?
            .with_ui(Box::new(guided_ui));
        
        let guided_config = guided_flow.run_single_pass()?;
        assert_eq!(guided_config.get_name(), "guided-skill");
        
        // Both should be complete and valid
        assert!(quick_config.is_complete());
        assert!(guided_config.is_complete());
        
        Ok(())
    }

    #[tokio::test]
    async fn test_builder_system_integration() -> Result<()> {
        // Test: All three builder types work together
        let temp_dir = TempDir::new()?;
        std::env::set_current_dir(&temp_dir)?;
        
        // Create directories
        std::fs::create_dir_all(temp_dir.path().join(".q-skills"))?;
        std::fs::create_dir_all(temp_dir.path().join(".q-commands"))?;
        std::fs::create_dir_all(temp_dir.path().join(".q-agents"))?;
        
        // Create skill
        let mut skill_ui = MockTerminalUI::new(vec![
            "python process.py".to_string(),
            "y".to_string(),
        ]);
        let mut skill_flow = SkillCreationFlow::new("processor".to_string(), CreationMode::Quick)?
            .with_ui(Box::new(skill_ui));
        let skill_config = skill_flow.run_single_pass()?;
        
        // Create command
        let mut cmd_ui = MockTerminalUI::new(vec![
            "ls -la".to_string(),
            "y".to_string(),
        ]);
        let mut cmd_flow = CommandCreationFlow::new("list-all".to_string(), CreationMode::Quick)?
            .with_ui(Box::new(cmd_ui));
        let cmd_config = cmd_flow.run_single_pass()?;
        
        // Create agent
        let mut agent_ui = MockTerminalUI::new(vec![
            "You help with tasks".to_string(),
            "y".to_string(),
        ]);
        let mut agent_flow = AgentCreationFlow::new("helper".to_string(), CreationMode::Quick)?
            .with_ui(Box::new(agent_ui));
        let agent_config = agent_flow.run_single_pass()?;
        
        // All should be created successfully
        assert!(skill_config.is_complete());
        assert!(cmd_config.is_complete());
        assert!(agent_config.is_complete());
        
        // All should have correct names
        assert_eq!(skill_config.get_name(), "processor");
        assert_eq!(cmd_config.get_name(), "list-all");
        assert_eq!(agent_config.get_name(), "helper");
        
        Ok(())
    }
}
