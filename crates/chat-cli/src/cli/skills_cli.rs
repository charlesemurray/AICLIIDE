use crate::cli::skills::{SkillRegistry, SkillError};
use crate::os::Os;
use clap::{Args, Subcommand};
use eyre::Result;
use serde_json::json;
use std::process::ExitCode;

#[derive(Debug, Args, PartialEq)]
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

// Separate enum for slash commands
#[derive(Debug, Subcommand, PartialEq)]
pub enum SkillsSlashCommand {
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
}

impl SkillsArgs {
    pub async fn execute(self, _os: &mut Os) -> Result<ExitCode> {
        let registry = SkillRegistry::with_builtins();
        
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
                
                let result = registry.execute_skill(&skill_name, params).await
                    .map_err(|e| eyre::eyre!("Skill execution failed: {}", e))?;
                println!("{}", result.output);
                
                Ok(ExitCode::SUCCESS)
            },
            SkillsCommand::Info { skill_name } => {
                match registry.get(&skill_name) {
                    Some(skill) => {
                        println!("Name: {}", skill.name());
                        println!("Description: {}", skill.description());
                        println!("Interactive: {}", skill.supports_interactive());
                        
                        let ui = skill.render_ui().await
                            .map_err(|e| eyre::eyre!("Failed to render UI: {}", e))?;
                        if !ui.elements.is_empty() {
                            println!("UI Elements: {}", ui.elements.len());
                        }
                    },
                    None => {
                        return Err(eyre::eyre!("Skill '{}' not found", skill_name));
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

impl SkillsSlashCommand {
    pub async fn execute(self, _os: &mut Os) -> Result<ExitCode> {
        let registry = SkillRegistry::with_builtins();
        
        match self {
            Self::List { detailed } => {
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
            Self::Run { skill_name, params } => {
                let params = match params {
                    Some(p) => serde_json::from_str(&p)
                        .map_err(|e| SkillError::InvalidInput(format!("Invalid JSON: {}", e)))?,
                    None => json!({}),
                };
                
                let result = registry.execute_skill(&skill_name, params).await
                    .map_err(|e| eyre::eyre!("Skill execution failed: {}", e))?;
                println!("{}", result.output);
                
                Ok(ExitCode::SUCCESS)
            },
            Self::Info { skill_name } => {
                match registry.get(&skill_name) {
                    Some(skill) => {
                        println!("Name: {}", skill.name());
                        println!("Description: {}", skill.description());
                        println!("Interactive: {}", skill.supports_interactive());
                        
                        let ui = skill.render_ui().await
                            .map_err(|e| eyre::eyre!("Failed to render UI: {}", e))?;
                        if !ui.elements.is_empty() {
                            println!("UI Elements: {}", ui.elements.len());
                        }
                    },
                    None => {
                        return Err(eyre::eyre!("Skill '{}' not found", skill_name));
                    },
                }
                
                Ok(ExitCode::SUCCESS)
            },
        }
    }
}
