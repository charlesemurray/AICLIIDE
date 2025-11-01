//! CLI tests for Cisco-style command parsing and execution

use super::*;
use clap::Parser;
use crate::cli::creation::{CreateArgs, CreateCommand, SkillMode, CommandMode, AgentMode};

#[cfg(test)]
mod cisco_style_parsing {
    use super::*;

    #[test]
    fn test_create_skill_basic() {
        let args = CreateArgs::try_parse_from(&["create", "skill", "myskill"]).unwrap();
        match args.command {
            CreateCommand::Skill { name, mode } => {
                assert_eq!(name, "myskill");
                assert_eq!(mode, None); // Default mode
            }
            _ => panic!("Expected Skill command"),
        }
    }

    #[test]
    fn test_create_skill_with_modes() {
        // Test quick mode
        let args = CreateArgs::try_parse_from(&["create", "skill", "myskill", "quick"]).unwrap();
        match args.command {
            CreateCommand::Skill { name, mode } => {
                assert_eq!(name, "myskill");
                assert_eq!(mode, Some(SkillMode::Quick));
            }
            _ => panic!("Expected Skill command"),
        }

        // Test guided mode
        let args = CreateArgs::try_parse_from(&["create", "skill", "myskill", "guided"]).unwrap();
        match args.command {
            CreateCommand::Skill { name, mode } => {
                assert_eq!(name, "myskill");
                assert_eq!(mode, Some(SkillMode::Guided));
            }
            _ => panic!("Expected Skill command"),
        }

        // Test expert mode
        let args = CreateArgs::try_parse_from(&["create", "skill", "myskill", "expert"]).unwrap();
        match args.command {
            CreateCommand::Skill { name, mode } => {
                assert_eq!(name, "myskill");
                assert_eq!(mode, Some(SkillMode::Expert));
            }
            _ => panic!("Expected Skill command"),
        }
    }

    #[test]
    fn test_create_skill_template() {
        let args = CreateArgs::try_parse_from(&[
            "create", "skill", "myskill", "template", "existing-skill"
        ]).unwrap();
        
        match args.command {
            CreateCommand::Skill { name, mode } => {
                assert_eq!(name, "myskill");
                match mode {
                    Some(SkillMode::Template { source }) => {
                        assert_eq!(source, "existing-skill");
                    }
                    _ => panic!("Expected Template mode"),
                }
            }
            _ => panic!("Expected Skill command"),
        }
    }

    #[test]
    fn test_create_command_basic() {
        let args = CreateArgs::try_parse_from(&["create", "command", "mycmd"]).unwrap();
        match args.command {
            CreateCommand::Command { name, mode } => {
                assert_eq!(name, "mycmd");
                assert_eq!(mode, None);
            }
            _ => panic!("Expected Command command"),
        }
    }

    #[test]
    fn test_create_agent_basic() {
        let args = CreateArgs::try_parse_from(&["create", "agent", "myagent"]).unwrap();
        match args.command {
            CreateCommand::Agent { name, mode } => {
                assert_eq!(name, "myagent");
                assert_eq!(mode, None);
            }
            _ => panic!("Expected Agent command"),
        }
    }

    #[test]
    fn test_invalid_commands_rejected() {
        // Bash-style flags should be rejected
        assert!(CreateArgs::try_parse_from(&[
            "create", "skill", "myskill", "--interactive"
        ]).is_err());
        
        assert!(CreateArgs::try_parse_from(&[
            "create", "skill", "myskill", "--quick"
        ]).is_err());
        
        // Invalid subcommands should be rejected
        assert!(CreateArgs::try_parse_from(&[
            "create", "skill", "myskill", "invalid"
        ]).is_err());
    }
}

#[cfg(test)]
mod command_execution {
    use super::*;

    #[tokio::test]
    async fn test_create_skill_execution() {
        let fixtures = TestFixtures::new();
        fixtures.setup_directories();
        
        let args = CreateArgs {
            command: CreateCommand::Skill {
                name: "test-skill".to_string(),
                mode: Some(SkillMode::Quick),
            }
        };
        
        // Mock the execution environment
        std::env::set_current_dir(&fixtures.temp_dir).unwrap();
        
        let result = args.execute_test().await;
        assert!(result.is_ok());
        
        // Verify skill was created
        let skill_file = fixtures.skills_dir.join("test-skill.json");
        assert!(skill_file.exists());
    }

    #[tokio::test]
    async fn test_create_command_execution() {
        let fixtures = TestFixtures::new();
        fixtures.setup_directories();
        
        let args = CreateArgs {
            command: CreateCommand::Command {
                name: "test-cmd".to_string(),
                mode: Some(CommandMode::Quick),
            }
        };
        
        std::env::set_current_dir(&fixtures.temp_dir).unwrap();
        
        let result = args.execute_test().await;
        assert!(result.is_ok());
        
        // Verify command was created
        let cmd_file = fixtures.commands_dir.join("test-cmd.json");
        assert!(cmd_file.exists());
    }

    #[tokio::test]
    async fn test_create_agent_execution() {
        let fixtures = TestFixtures::new();
        fixtures.setup_directories();
        
        let args = CreateArgs {
            command: CreateCommand::Agent {
                name: "test-agent".to_string(),
                mode: Some(AgentMode::Quick),
            }
        };
        
        std::env::set_current_dir(&fixtures.temp_dir).unwrap();
        
        let result = args.execute_test().await;
        assert!(result.is_ok());
        
        // Verify agent was created
        let agent_file = fixtures.agents_dir.join("test-agent.json");
        assert!(agent_file.exists());
    }
}

#[cfg(test)]
mod error_handling {
    use super::*;

    #[test]
    fn test_invalid_name_error() {
        let args = CreateArgs::try_parse_from(&["create", "skill", "Invalid Name"]).unwrap();
        
        // Should parse but fail during execution with helpful error
        let result = futures::executor::block_on(args.execute_test());
        assert!(result.is_err());
        
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Invalid skill name"));
        assert!(error.to_string().contains("invalid-name")); // Suggestion
    }

    #[test]
    fn test_existing_name_error() {
        let fixtures = TestFixtures::new();
        fixtures.setup_directories();
        
        // Create existing skill
        std::fs::write(
            fixtures.skills_dir.join("existing.json"),
            r#"{"name": "existing"}"#
        ).unwrap();
        
        std::env::set_current_dir(&fixtures.temp_dir).unwrap();
        
        let args = CreateArgs {
            command: CreateCommand::Skill {
                name: "existing".to_string(),
                mode: None,
            }
        };
        
        let result = futures::executor::block_on(args.execute_test());
        assert!(result.is_err());
        
        let error = result.unwrap_err();
        assert!(error.to_string().contains("already exists"));
        assert!(error.to_string().contains("force")); // Suggest force mode
    }

    #[test]
    fn test_missing_template_error() {
        let args = CreateArgs {
            command: CreateCommand::Skill {
                name: "test".to_string(),
                mode: Some(SkillMode::Template { 
                    source: "nonexistent".to_string() 
                }),
            }
        };
        
        let result = futures::executor::block_on(args.execute_test());
        assert!(result.is_err());
        
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Template 'nonexistent' not found"));
    }
}

#[cfg(test)]
mod help_output {
    use super::*;

    #[test]
    fn test_create_help_output() {
        let result = CreateArgs::try_parse_from(&["create", "--help"]);
        assert!(result.is_err()); // Help causes parse to fail, but with help output
        
        // The help output should contain our subcommands
        let help = format!("{}", CreateArgs::command().render_help());
        assert!(help.contains("skill"));
        assert!(help.contains("command"));
        assert!(help.contains("agent"));
    }

    #[test]
    fn test_skill_help_output() {
        let help = format!("{}", CreateArgs::command().render_help());
        assert!(help.contains("quick"));
        assert!(help.contains("guided"));
        assert!(help.contains("expert"));
        assert!(help.contains("template"));
        assert!(help.contains("preview"));
        assert!(help.contains("edit"));
        assert!(help.contains("force"));
    }
}
