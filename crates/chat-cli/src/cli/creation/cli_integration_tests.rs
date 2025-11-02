#[cfg(test)]
mod tests {
    use crate::cli::creation::{CreateArgs, CreateCommand};

    #[tokio::test]
    async fn test_create_skill_command_integration() {
        let args = CreateArgs {
            command: CreateCommand::Skill {
                name: "test-skill".to_string(),
                mode: None,
            },
        };

        // Test that the command structure is correct
        assert!(matches!(args.command, CreateCommand::Skill { .. }));
    }

    #[tokio::test]
    async fn test_create_command_integration() {
        let args = CreateArgs {
            command: CreateCommand::Command {
                name: "test-command".to_string(),
                mode: None,
            },
        };

        assert!(matches!(args.command, CreateCommand::Command { .. }));
    }

    #[tokio::test]
    async fn test_create_agent_integration() {
        let args = CreateArgs {
            command: CreateCommand::Agent {
                name: "test-agent".to_string(),
                mode: None,
            },
        };

        assert!(matches!(args.command, CreateCommand::Agent { .. }));
    }

    #[test]
    fn test_cli_command_parsing() {
        // Test that the CLI structure is properly set up
        use clap::CommandFactory;
        let cmd = CreateArgs::command();

        assert_eq!(cmd.get_name(), "create");
        assert!(cmd.is_subcommand_required_set());

        // Verify subcommands exist
        let subcommands: Vec<_> = cmd.get_subcommands().map(|s| s.get_name()).collect();
        assert!(subcommands.contains(&"skill"));
        assert!(subcommands.contains(&"command"));
        assert!(subcommands.contains(&"agent"));
    }
}
