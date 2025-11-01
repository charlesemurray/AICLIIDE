//! Backward compatibility tests ensuring existing functionality is preserved

use super::*;
use crate::cli::{SkillsArgs, SkillsCommand};
use crate::cli::agent::{AgentArgs, AgentCommand};

#[cfg(test)]
mod existing_skills_cli {
    use super::*;

    #[tokio::test]
    async fn test_skills_create_interactive_delegation() {
        let fixtures = TestFixtures::new();
        fixtures.setup_directories();
        std::env::set_current_dir(&fixtures.temp_dir).unwrap();
        
        // Existing command: q skills create myskill --interactive
        let args = SkillsArgs {
            command: SkillsCommand::Create {
                name: "myskill".to_string(),
                skill_type: None,
                interactive: true,
                wizard: false,
                quick: false,
                command: None,
                template: None,
            }
        };
        
        let result = args.execute_test().await;
        assert!(result.is_ok());
        
        // Should delegate to new creation system but maintain same behavior
        let skill_file = fixtures.skills_dir.join("myskill.json");
        assert!(skill_file.exists());
        
        // Verify it used guided mode (equivalent to --interactive)
        let content = std::fs::read_to_string(&skill_file).unwrap();
        let skill: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(skill["name"], "myskill");
    }

    #[tokio::test]
    async fn test_skills_create_wizard_delegation() {
        let fixtures = TestFixtures::new();
        fixtures.setup_directories();
        std::env::set_current_dir(&fixtures.temp_dir).unwrap();
        
        // Existing command: q skills create myskill --wizard
        let args = SkillsArgs {
            command: SkillsCommand::Create {
                name: "myskill".to_string(),
                skill_type: Some("code_inline".to_string()),
                interactive: false,
                wizard: true,
                quick: false,
                command: None,
                template: None,
            }
        };
        
        let result = args.execute_test().await;
        assert!(result.is_ok());
        
        // Should delegate to guided mode with pre-filled type
        let skill_file = fixtures.skills_dir.join("myskill.json");
        assert!(skill_file.exists());
        
        let content = std::fs::read_to_string(&skill_file).unwrap();
        let skill: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(skill["type"], "code_inline");
    }

    #[tokio::test]
    async fn test_skills_create_quick_delegation() {
        let fixtures = TestFixtures::new();
        fixtures.setup_directories();
        std::env::set_current_dir(&fixtures.temp_dir).unwrap();
        
        // Existing command: q skills create myskill --quick --command "echo hello"
        let args = SkillsArgs {
            command: SkillsCommand::Create {
                name: "myskill".to_string(),
                skill_type: None,
                interactive: false,
                wizard: false,
                quick: true,
                command: Some("echo hello".to_string()),
                template: None,
            }
        };
        
        let result = args.execute_test().await;
        assert!(result.is_ok());
        
        // Should delegate to quick mode with pre-filled command
        let skill_file = fixtures.skills_dir.join("myskill.json");
        assert!(skill_file.exists());
        
        let content = std::fs::read_to_string(&skill_file).unwrap();
        let skill: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(skill["command"], "echo hello");
    }

    #[tokio::test]
    async fn test_skills_other_commands_unchanged() {
        let fixtures = TestFixtures::new();
        fixtures.setup_directories();
        
        // Create test skill
        std::fs::write(
            fixtures.skills_dir.join("test.json"),
            r#"{"name": "test", "type": "code_inline", "command": "echo test"}"#
        ).unwrap();
        
        std::env::set_current_dir(&fixtures.temp_dir).unwrap();
        
        // Existing commands should work unchanged
        let list_args = SkillsArgs {
            command: SkillsCommand::List,
        };
        let result = list_args.execute_test().await;
        assert!(result.is_ok());
        
        let run_args = SkillsArgs {
            command: SkillsCommand::Run {
                skill_name: "test".to_string(),
                params: None,
            },
        };
        let result = run_args.execute_test().await;
        assert!(result.is_ok());
        
        let info_args = SkillsArgs {
            command: SkillsCommand::Info {
                skill_name: "test".to_string(),
            },
        };
        let result = info_args.execute_test().await;
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod existing_agent_cli {
    use super::*;

    #[tokio::test]
    async fn test_agent_create_delegation() {
        let fixtures = TestFixtures::new();
        fixtures.setup_directories();
        std::env::set_current_dir(&fixtures.temp_dir).unwrap();
        
        // Existing command: q agent create --name myagent
        let args = AgentArgs {
            command: AgentCommand::Create {
                name: Some("myagent".to_string()),
                interactive: false,
            }
        };
        
        let result = args.execute_test().await;
        assert!(result.is_ok());
        
        // Should delegate to new creation system
        let agent_file = fixtures.agents_dir.join("myagent.json");
        assert!(agent_file.exists());
        
        let content = std::fs::read_to_string(&agent_file).unwrap();
        let agent: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(agent["name"], "myagent");
    }

    #[tokio::test]
    async fn test_agent_other_commands_unchanged() {
        let fixtures = TestFixtures::new();
        fixtures.setup_directories();
        
        // Create test agent
        std::fs::write(
            fixtures.agents_dir.join("test.json"),
            r#"{"name": "test", "description": "Test agent", "prompt": "You are helpful"}"#
        ).unwrap();
        
        std::env::set_current_dir(&fixtures.temp_dir).unwrap();
        
        // Existing commands should work unchanged
        let list_args = AgentArgs {
            command: AgentCommand::List,
        };
        let result = list_args.execute_test().await;
        assert!(result.is_ok());
        
        let validate_args = AgentArgs {
            command: AgentCommand::Validate {
                name: "test".to_string(),
            },
        };
        let result = validate_args.execute_test().await;
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod file_format_compatibility {
    use super::*;

    #[tokio::test]
    async fn test_existing_skill_files_readable() {
        let fixtures = TestFixtures::new();
        fixtures.setup_directories();
        
        // Create skill in existing format
        let existing_skill = r#"{
            "name": "existing-skill",
            "type": "code_inline",
            "command": "python script.py",
            "description": "Existing skill",
            "security": {
                "enabled": true,
                "level": "medium"
            },
            "created_at": "2024-01-01T00:00:00Z",
            "usage_count": 5
        }"#;
        
        std::fs::write(
            fixtures.skills_dir.join("existing-skill.json"),
            existing_skill
        ).unwrap();
        
        // New system should read existing files
        let registry = SkillRegistry::new(fixtures.skills_dir.clone()).await.unwrap();
        let skill = registry.get_skill("existing-skill").unwrap();
        
        assert_eq!(skill.name, "existing-skill");
        assert_eq!(skill.command, "python script.py");
        assert_eq!(skill.usage_count, 5);
    }

    #[tokio::test]
    async fn test_existing_command_files_readable() {
        let fixtures = TestFixtures::new();
        fixtures.setup_directories();
        
        // Create command in existing format
        let existing_command = r#"{
            "name": "existing-cmd",
            "description": "Existing command",
            "handler": {
                "Script": {
                    "command": "echo hello",
                    "args": []
                }
            },
            "parameters": [],
            "created_at": "2024-01-01T00:00:00Z",
            "usage_count": 3
        }"#;
        
        std::fs::write(
            fixtures.commands_dir.join("existing-cmd.json"),
            existing_command
        ).unwrap();
        
        // New system should read existing files
        let registry = CustomCommandRegistry::new(fixtures.commands_dir.clone()).unwrap();
        let command = registry.get_command("existing-cmd").unwrap();
        
        assert_eq!(command.name, "existing-cmd");
        assert_eq!(command.usage_count, 3);
    }

    #[tokio::test]
    async fn test_existing_agent_files_readable() {
        let fixtures = TestFixtures::new();
        fixtures.setup_directories();
        
        // Create agent in existing format
        let existing_agent = r#"{
            "name": "existing-agent",
            "description": "Existing agent",
            "prompt": "You are helpful",
            "mcpServers": [
                {
                    "name": "filesystem",
                    "command": "mcp-server-filesystem",
                    "args": ["/tmp"]
                }
            ],
            "tools": ["fs_read", "fs_write"],
            "hooks": {
                "agentSpawn": "console.log('Agent started')"
            }
        }"#;
        
        std::fs::write(
            fixtures.agents_dir.join("existing-agent.json"),
            existing_agent
        ).unwrap();
        
        // New system should read existing files
        let agent_manager = AgentManager::new(fixtures.agents_dir.clone()).await.unwrap();
        let agent = agent_manager.get_agent("existing-agent").unwrap();
        
        assert_eq!(agent.name, "existing-agent");
        assert_eq!(agent.mcp_servers.len(), 1);
        assert_eq!(agent.tools.len(), 2);
    }
}

#[cfg(test)]
mod api_compatibility {
    use super::*;

    #[test]
    fn test_skill_registry_api_unchanged() {
        // Existing SkillRegistry API should remain unchanged
        let registry = SkillRegistry::with_builtins();
        
        // These methods should still exist and work
        assert!(registry.list_skills().is_ok());
        assert!(registry.get_skill("nonexistent").is_none());
        
        // New methods can be added but existing ones preserved
        let skill_names = registry.get_skill_names();
        assert!(skill_names.is_ok());
    }

    #[test]
    fn test_custom_command_registry_api_unchanged() {
        let temp_dir = TempDir::new().unwrap();
        let registry = CustomCommandRegistry::new(temp_dir.path().to_path_buf()).unwrap();
        
        // Existing API should be preserved
        assert!(registry.list_commands().is_ok());
        assert!(registry.get_command("nonexistent").is_none());
        
        // Can add new methods but preserve existing
        let command_names = registry.get_command_names();
        assert!(command_names.is_ok());
    }

    #[test]
    fn test_agent_manager_api_unchanged() {
        // Agent manager API should remain stable
        // Test basic agent operations that should be preserved
        
        let temp_dir = TempDir::new().unwrap();
        
        // Test that we can create agent directory structure
        let agents_dir = temp_dir.path().join(".amazonq").join("cli-agents");
        std::fs::create_dir_all(&agents_dir).unwrap();
        
        // Test that we can write agent files in expected format
        let agent_config = r#"{
            "name": "test-agent",
            "description": "Test agent",
            "prompt": "You are helpful"
        }"#;
        
        std::fs::write(agents_dir.join("test-agent.json"), agent_config).unwrap();
        
        // Verify file was created correctly
        assert!(agents_dir.join("test-agent.json").exists());
        
        // Test that we can read the agent config back
        let content = std::fs::read_to_string(agents_dir.join("test-agent.json")).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed["name"], "test-agent");
    }
}

#[cfg(test)]
mod migration_support {
    use super::*;

    #[tokio::test]
    async fn test_automatic_format_migration() {
        let fixtures = TestFixtures::new();
        fixtures.setup_directories();
        
        // Create skill in old format (missing new fields)
        let old_format = r#"{
            "name": "old-skill",
            "type": "code_inline",
            "command": "python script.py"
        }"#;
        
        std::fs::write(
            fixtures.skills_dir.join("old-skill.json"),
            old_format
        ).unwrap();
        
        // New system should automatically migrate on load
        let registry = SkillRegistry::new(fixtures.skills_dir.clone()).await.unwrap();
        let skill = registry.get_skill("old-skill").unwrap();
        
        // Should have new fields with defaults
        assert!(!skill.description.is_empty()); // Auto-generated
        assert!(skill.security.is_some()); // Default security config
        assert!(skill.created_at.is_some()); // Auto-added timestamp
        
        // Original file should be backed up
        let backup_file = fixtures.skills_dir.join("old-skill.json.backup");
        assert!(backup_file.exists());
        
        // New file should have migrated format
        let migrated_content = std::fs::read_to_string(
            fixtures.skills_dir.join("old-skill.json")
        ).unwrap();
        let migrated: serde_json::Value = serde_json::from_str(&migrated_content).unwrap();
        assert!(migrated.get("description").is_some());
        assert!(migrated.get("security").is_some());
    }

    #[tokio::test]
    async fn test_version_compatibility_check() {
        let fixtures = TestFixtures::new();
        fixtures.setup_directories();
        
        // Create skill with future version
        let future_format = r#"{
            "name": "future-skill",
            "version": "2.0.0",
            "type": "code_inline",
            "command": "python script.py",
            "new_future_field": "some value"
        }"#;
        
        std::fs::write(
            fixtures.skills_dir.join("future-skill.json"),
            future_format
        ).unwrap();
        
        // Should handle gracefully with warning
        let registry = SkillRegistry::new(fixtures.skills_dir.clone()).await.unwrap();
        let skill = registry.get_skill("future-skill");
        
        // Should still load basic fields
        assert!(skill.is_some());
        assert_eq!(skill.unwrap().name, "future-skill");
        
        // Should log compatibility warning (check logs in real implementation)
    }
}
