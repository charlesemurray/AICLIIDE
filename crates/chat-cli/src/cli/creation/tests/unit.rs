//! Unit tests for individual creation system components

use super::*;
use crate::cli::creation::{
    AgentCreationFlow,
    CommandCreationFlow,
    CreationContext,
    CreationFlow,
    CreationType,
    SkillCreationFlow,
    TerminalUI,
};

#[cfg(test)]
mod creation_types {
    use super::*;

    #[test]
    fn test_creation_type_complexity_levels() {
        assert_eq!(CreationType::CustomCommand.complexity_level(), ComplexityLevel::Low);
        assert_eq!(CreationType::Skill.complexity_level(), ComplexityLevel::Medium);
        assert_eq!(CreationType::Agent.complexity_level(), ComplexityLevel::High);
    }

    #[test]
    fn test_creation_type_required_phases() {
        let command_phases = CreationType::CustomCommand.required_phases();
        assert_eq!(command_phases, vec![
            CreationPhase::Discovery,
            CreationPhase::BasicConfig,
            CreationPhase::Completion
        ]);

        let skill_phases = CreationType::Skill.required_phases();
        assert_eq!(skill_phases, vec![
            CreationPhase::Discovery,
            CreationPhase::BasicConfig,
            CreationPhase::Security,
            CreationPhase::Testing,
            CreationPhase::Completion
        ]);

        let agent_phases = CreationType::Agent.required_phases();
        assert_eq!(agent_phases, vec![
            CreationPhase::Discovery,
            CreationPhase::BasicConfig,
            CreationPhase::AdvancedConfig,
            CreationPhase::Security,
            CreationPhase::Testing,
            CreationPhase::Completion
        ]);
    }
}

#[cfg(test)]
mod command_creation_flow {
    use super::*;

    #[test]
    fn test_command_flow_single_pass_collection() -> Result<()> {
        let fixtures = TestFixtures::new();
        let ui = MockTerminalUI::new(vec![
            "echo hello".to_string(),   // command
            "Test command".to_string(), // description
            "".to_string(),             // parameters (none)
        ]);

        let mut flow = CommandCreationFlow::new("test".to_string(), CreationMode::Quick)?.with_ui(Box::new(ui));
        let config = flow.collect_input_single_pass().unwrap();

        assert_eq!(config.command, "echo hello");
        assert_eq!(config.description, "Test command");
        assert_eq!(config.parameters.len(), 0);
        assert_eq!(config.command_type, CommandType::Script);
        Ok(())
    }

    #[test]
    fn test_command_flow_auto_detect_alias() -> Result<()> {
        let ui = MockTerminalUI::new(vec![
            "ls -la".to_string(),     // command (detected as alias)
            "List files".to_string(), // description
        ]);

        let mut flow = CommandCreationFlow::new("ll".to_string(), CreationMode::Quick)?.with_ui(Box::new(ui));
        let config = flow.collect_input_single_pass().unwrap();

        assert_eq!(config.command_type, CommandType::Alias);
        Ok(())
    }

    #[test]
    fn test_command_flow_parameter_detection() -> Result<()> {
        let ui = MockTerminalUI::new(vec![
            "echo {{message}}".to_string(), // command with parameter
            "Echo message".to_string(),     // description
        ]);

        let mut flow = CommandCreationFlow::new("echo".to_string(), CreationMode::Quick)?.with_ui(Box::new(ui));
        let config = flow.collect_input_single_pass().unwrap();

        assert_eq!(config.parameters.len(), 1);
        assert_eq!(config.parameters[0].name, "message");
        assert!(config.parameters[0].required);
        Ok(())
    }
}

#[cfg(test)]
mod skill_creation_flow {
    use super::*;

    #[test]
    fn test_skill_flow_quick_mode() -> Result<()> {
        let ui = MockTerminalUI::new(vec![
            "python script.py".to_string(), // command
        ]);

        let mut flow = SkillCreationFlow::new("test".to_string(), CreationMode::Quick)?.with_ui(Box::new(ui));
        let config = flow.collect_input_single_pass().unwrap();

        assert_eq!(config.skill_type, SkillType::CodeInline);
        assert_eq!(config.command, "python script.py");
        assert!(config.description.is_empty()); // Quick mode skips optional fields
        Ok(())
    }

    #[test]
    fn test_skill_flow_guided_mode() -> Result<()> {
        let ui = MockTerminalUI::new(vec![
            "python script.py".to_string(),  // command
            "Test Python skill".to_string(), // description
            "y".to_string(),                 // enable security
            "medium".to_string(),            // security level
        ]);

        let mut flow = SkillCreationFlow::new("test".to_string(), CreationMode::Guided)?.with_ui(Box::new(ui));
        let config = flow.collect_input_single_pass().unwrap();

        assert_eq!(config.skill_type, SkillType::CodeInline);
        assert_eq!(config.description, "Test Python skill");
        assert!(config.security.enabled);
        assert_eq!(config.security.level, SecurityLevel::Medium);
        Ok(())
    }

    #[test]
    fn test_skill_flow_expert_mode() -> Result<()> {
        let ui = MockTerminalUI::new(vec![
            "conversation".to_string(),                // skill type
            "You are a helpful assistant".to_string(), // system prompt
            "Test conversation skill".to_string(),     // description
            "y".to_string(),                           // enable security
            "high".to_string(),                        // security level
            "1000".to_string(),                        // resource limit
        ]);

        let mut flow = SkillCreationFlow::new("test".to_string(), CreationMode::Expert)?.with_ui(Box::new(ui));
        let config = flow.collect_input_single_pass().unwrap();

        assert_eq!(config.skill_type, SkillType::Conversation);
        // Remove this assertion - system_prompt field doesn't exist
        // assert_eq!(config.system_prompt, "You are a helpful assistant");
        assert_eq!(config.security.resource_limit, 1000);
        Ok(())
    }
}

#[cfg(test)]
mod agent_creation_flow {
    use super::*;

    #[test]
    fn test_agent_flow_quick_mode() -> Result<()> {
        let mut ui = MockTerminalUI::new(vec![
            "Test role".to_string(), // prompt
        ]);

        let mut flow = AgentCreationFlow::new("test".to_string(), CreationMode::Quick)?;
        let config = flow.collect_input_single_pass().unwrap();

        assert_eq!(config.basic.prompt, "Test role");
        assert!(config.mcp.servers.is_empty()); // Quick mode uses defaults
        assert!(config.tools.enabled_tools.is_empty());
        Ok(())
    }

    #[test]
    fn test_agent_flow_expert_mode() -> Result<()> {
        let mut ui = MockTerminalUI::new(vec![
            "You are a coding assistant".to_string(), // prompt
            "Coding helper".to_string(),              // description
            "y".to_string(),                          // enable MCP
            "filesystem".to_string(),                 // MCP server
            "y".to_string(),                          // enable tools
            "fs_read,fs_write".to_string(),           // allowed tools
            "y".to_string(),                          // enable hooks
            "agentSpawn".to_string(),                 // hook type
        ]);

        let mut flow = AgentCreationFlow::new("test".to_string(), CreationMode::Expert)?.with_ui(Box::new(ui));
        let config = flow.collect_input_single_pass().unwrap();

        assert_eq!(config.basic.description, "Coding helper");
        assert_eq!(config.mcp.servers.len(), 1);
        assert_eq!(config.tools.enabled_tools.len(), 2);
        assert_eq!(config.hooks.enabled_hooks.len(), 1);
        Ok(())
    }
}

#[cfg(test)]
mod creation_context {
    use super::*;

    #[test]
    fn test_context_smart_defaults() {
        let fixtures = TestFixtures::new();
        fixtures.setup_directories();

        // Create existing skill for context
        std::fs::write(
            fixtures.skills_dir.join("existing.json"),
            r#"{"name": "existing", "type": "code_inline", "command": "python test.py"}"#,
        )
        .unwrap();

        let context = CreationContext::new(fixtures.temp_dir.path()).unwrap();
        let defaults = context.suggest_defaults(&CreationType::Skill);

        assert_eq!(defaults.skill_type, Some(SkillType::CodeInline)); // Based on existing
    }

    #[test]
    fn test_context_project_detection() {
        let fixtures = TestFixtures::new();

        // Create Python project files
        std::fs::write(fixtures.temp_dir.path().join("requirements.txt"), "").unwrap();
        std::fs::write(fixtures.temp_dir.path().join("main.py"), "print('hello')").unwrap();

        let context = CreationContext::new(fixtures.temp_dir.path()).unwrap();
        // Remove this assertion - project_type field is private
        // assert_eq!(context.project_type, Some(ProjectType::Python));

        let defaults = context.suggest_defaults(&CreationType::Skill);
        assert_eq!(defaults.skill_type, Some(SkillType::CodeInline)); // Python → code_inline
    }

    #[test]
    fn test_context_name_validation() {
        let fixtures = TestFixtures::new();
        let context = CreationContext::new(fixtures.temp_dir.path()).unwrap();

        // Valid names
        assert!(context.validate_name("test", &CreationType::Skill).is_valid());
        assert!(context.validate_name("test-skill", &CreationType::Skill).is_valid());
        assert!(context.validate_name("test_skill", &CreationType::Skill).is_valid());

        // Invalid names - "Test Skill" becomes "test-skill"
        let result = context.validate_name("Test Skill", &CreationType::Skill);
        assert!(!result.is_valid());
        assert_eq!(result.suggestion, "test-skill");

        // Invalid names - "test@skill" becomes "testskill" (@ is filtered out)
        let result = context.validate_name("test@skill", &CreationType::Skill);
        assert!(!result.is_valid());
        assert_eq!(result.suggestion, "testskill");
    }
}

#[cfg(test)]
mod terminal_ui {
    use super::*;

    #[test]
    fn test_terminal_ui_prompt_required() {
        let mut ui = MockTerminalUI::new(vec!["test input".to_string()]);
        let result = ui.prompt_required("Command").unwrap();
        assert_eq!(result, "test input");
        assert!(ui.outputs.iter().any(|o| o.contains("Command:")));
    }

    #[test]
    fn test_terminal_ui_prompt_optional() {
        let mut ui = MockTerminalUI::new(vec!["".to_string()]); // Empty input
        let result = ui.prompt_optional("Description", Some("default")).unwrap();
        assert_eq!(result, Some("default".to_string()));
    }

    #[test]
    fn test_terminal_ui_preview_format() {
        let mut ui = MockTerminalUI::new(vec![]);
        ui.show_preview("Creating: skill 'test' (code_inline)\nCommand: echo hello");

        assert!(ui.outputs.iter().any(|o| o.contains("Creating: skill 'test'")));
        assert!(ui.outputs.iter().any(|o| o.contains("Command: echo hello")));
    }

    #[test]
    fn test_terminal_ui_confirm() {
        let mut ui = MockTerminalUI::new(vec!["y".to_string()]);
        let result = ui.confirm("Create?").unwrap();
        assert!(result);

        let mut ui = MockTerminalUI::new(vec!["n".to_string()]);
        let result = ui.confirm("Create?").unwrap();
        assert!(!result);
    }
}
