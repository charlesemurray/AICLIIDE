#[cfg(test)]
mod cli_commands_tests {
    use std::process::ExitCode;

    use clap::{
        Parser,
        Subcommand,
    };
    use serde_json::json;

    use crate::cli::skills::{
        SkillError,
        SkillRegistry,
    };

    #[derive(Debug, PartialEq, Parser)]
    pub struct SkillsArgs {
        #[command(subcommand)]
        pub command: SkillsCommand,
    }

    #[derive(Debug, Subcommand, PartialEq)]
    pub enum SkillsCommand {
        /// List available skills
        List {
            /// Show detailed information
            #[arg(long)]
            detailed: bool,
        },
        /// Run a skill with parameters
        Run {
            /// Name of the skill to run
            skill_name: String,
            /// Parameters as JSON string
            #[arg(long)]
            params: Option<String>,
        },
        /// Show information about a specific skill
        Info {
            /// Name of the skill
            skill_name: String,
        },
        /// Install a skill from a file or URL
        Install {
            /// Path or URL to skill definition
            source: String,
        },
    }

    impl SkillsArgs {
        pub async fn execute(self, registry: &mut SkillRegistry) -> Result<ExitCode, SkillError> {
            match self.command {
                SkillsCommand::List { detailed } => {
                    let skills = registry.list();

                    if detailed {
                        for skill in skills {
                            println!("{}: {}", skill.name(), skill.description());
                            println!("  Interactive: {}", skill.supports_interactive());
                        }
                    } else {
                        for skill in skills {
                            println!("{}", skill.name());
                        }
                    }

                    Ok(ExitCode::SUCCESS)
                },
                SkillsCommand::Run { skill_name, params } => {
                    let params = match params {
                        Some(p) => serde_json::from_str(&p)
                            .map_err(|e| SkillError::InvalidInput(format!("Invalid JSON: {}", e)))?,
                        None => json!({}),
                    };

                    let result = registry.execute_skill(&skill_name, params).await?;
                    println!("{}", result.output);

                    Ok(ExitCode::SUCCESS)
                },
                SkillsCommand::Info { skill_name } => {
                    match registry.get(&skill_name) {
                        Some(skill) => {
                            println!("Name: {}", skill.name());
                            println!("Description: {}", skill.description());
                            println!("Interactive: {}", skill.supports_interactive());

                            let ui = skill.render_ui().await?;
                            if !ui.elements.is_empty() {
                                println!("UI Elements: {}", ui.elements.len());
                            }
                        },
                        None => {
                            return Err(SkillError::NotFound);
                        },
                    }

                    Ok(ExitCode::SUCCESS)
                },
                SkillsCommand::Install { source: _source } => {
                    // TODO: Implement skill installation
                    println!("Skill installation not yet implemented");
                    Ok(ExitCode::SUCCESS)
                },
            }
        }
    }

    #[tokio::test]
    async fn test_skills_list_command() {
        let mut registry = SkillRegistry::with_builtins();
        let args = SkillsArgs {
            command: SkillsCommand::List { detailed: false },
        };

        let result = args.execute(&mut registry).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ExitCode::SUCCESS);
    }

    #[tokio::test]
    async fn test_skills_list_detailed_command() {
        let mut registry = SkillRegistry::with_builtins();
        let args = SkillsArgs {
            command: SkillsCommand::List { detailed: true },
        };

        let result = args.execute(&mut registry).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ExitCode::SUCCESS);
    }

    #[tokio::test]
    async fn test_skills_run_command() {
        let mut registry = SkillRegistry::with_builtins();
        let args = SkillsArgs {
            command: SkillsCommand::Run {
                skill_name: "calculator".to_string(),
                params: Some(r#"{"op": "add", "a": 2, "b": 3}"#.to_string()),
            },
        };

        let result = args.execute(&mut registry).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ExitCode::SUCCESS);
    }

    #[tokio::test]
    async fn test_skills_run_command_no_params() {
        let mut registry = SkillRegistry::with_builtins();
        let args = SkillsArgs {
            command: SkillsCommand::Run {
                skill_name: "calculator".to_string(),
                params: None,
            },
        };

        let result = args.execute(&mut registry).await;
        // Should fail because calculator requires parameters
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_skills_run_command_invalid_json() {
        let mut registry = SkillRegistry::with_builtins();
        let args = SkillsArgs {
            command: SkillsCommand::Run {
                skill_name: "calculator".to_string(),
                params: Some("invalid json".to_string()),
            },
        };

        let result = args.execute(&mut registry).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            SkillError::InvalidInput(msg) => assert!(msg.contains("Invalid JSON")),
            _ => panic!("Expected InvalidInput error for invalid JSON"),
        }
    }

    #[tokio::test]
    async fn test_skills_run_command_nonexistent_skill() {
        let mut registry = SkillRegistry::with_builtins();
        let args = SkillsArgs {
            command: SkillsCommand::Run {
                skill_name: "nonexistent".to_string(),
                params: None,
            },
        };

        let result = args.execute(&mut registry).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            SkillError::NotFound => {}, // Expected
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn test_skills_info_command() {
        let mut registry = SkillRegistry::with_builtins();
        let args = SkillsArgs {
            command: SkillsCommand::Info {
                skill_name: "calculator".to_string(),
            },
        };

        let result = args.execute(&mut registry).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ExitCode::SUCCESS);
    }

    #[tokio::test]
    async fn test_skills_info_command_nonexistent() {
        let mut registry = SkillRegistry::with_builtins();
        let args = SkillsArgs {
            command: SkillsCommand::Info {
                skill_name: "nonexistent".to_string(),
            },
        };

        let result = args.execute(&mut registry).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            SkillError::NotFound => {}, // Expected
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn test_skills_install_command() {
        let mut registry = SkillRegistry::with_builtins();
        let args = SkillsArgs {
            command: SkillsCommand::Install {
                source: "test_skill.json".to_string(),
            },
        };

        let result = args.execute(&mut registry).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ExitCode::SUCCESS);
    }

    #[test]
    fn test_skills_args_parsing() {
        use clap::Parser;

        // Test list command
        let args = SkillsArgs::try_parse_from(&["skills", "list"]).unwrap();
        assert_eq!(args.command, SkillsCommand::List { detailed: false });

        // Test list detailed command
        let args = SkillsArgs::try_parse_from(&["skills", "list", "--detailed"]).unwrap();
        assert_eq!(args.command, SkillsCommand::List { detailed: true });

        // Test run command
        let args = SkillsArgs::try_parse_from(&["skills", "run", "calculator"]).unwrap();
        assert_eq!(args.command, SkillsCommand::Run {
            skill_name: "calculator".to_string(),
            params: None,
        });

        // Test run command with params
        let args =
            SkillsArgs::try_parse_from(&["skills", "run", "calculator", "--params", r#"{"op": "add"}"#]).unwrap();
        assert_eq!(args.command, SkillsCommand::Run {
            skill_name: "calculator".to_string(),
            params: Some(r#"{"op": "add"}"#.to_string()),
        });

        // Test info command
        let args = SkillsArgs::try_parse_from(&["skills", "info", "calculator"]).unwrap();
        assert_eq!(args.command, SkillsCommand::Info {
            skill_name: "calculator".to_string(),
        });

        // Test install command
        let args = SkillsArgs::try_parse_from(&["skills", "install", "skill.json"]).unwrap();
        assert_eq!(args.command, SkillsCommand::Install {
            source: "skill.json".to_string(),
        });
    }
}
