//! Unified Creation Assistant
//! 
//! Provides consistent, terminal-native creation experiences for Skills, Custom Commands, and Agents.
//! Follows Cisco-style CLI patterns, Rust best practices, and senior engineering standards.

mod types;
mod errors;
mod ui;
mod assistant;
mod flows;
mod context;

#[cfg(test)]
mod tests;

pub use types::*;
pub use errors::CreationError;
pub use ui::{TerminalUIImpl, TerminalUI};
#[cfg(test)]
pub use ui::MockTerminalUI;
pub use assistant::CreationAssistant;
pub use flows::*;
pub use context::CreationContext;

use clap::{Args, Subcommand};
use eyre::Result;
use std::process::ExitCode;
use crate::os::Os;

/// Creation command arguments following Cisco-style CLI patterns
#[derive(Debug, Args, PartialEq)]
pub struct CreateArgs {
    #[command(subcommand)]
    pub command: CreateCommand,
}

/// Cisco-style creation subcommands (no --flags)
#[derive(Debug, Subcommand, PartialEq)]
pub enum CreateCommand {
    /// Create a new skill
    Skill {
        /// Name of the skill
        name: String,
        #[command(subcommand)]
        mode: Option<SkillMode>,
    },
    /// Create a new custom command
    Command {
        /// Name of the command
        name: String,
        #[command(subcommand)]
        mode: Option<CommandMode>,
    },
    /// Create a new agent
    Agent {
        /// Name of the agent
        name: String,
        #[command(subcommand)]
        mode: Option<AgentMode>,
    },
}

/// Skill creation modes
#[derive(Debug, Subcommand, PartialEq)]
pub enum SkillMode {
    /// Quick creation with minimal prompts
    Quick,
    /// Step-by-step guided creation
    Guided,
    /// Expert mode with full configuration
    Expert,
    /// Create from existing template
    Template { source: String },
    /// Preview what would be created
    Preview,
    /// Edit existing skill
    Edit,
    /// Force overwrite existing
    Force,
}

/// Command creation modes
#[derive(Debug, Subcommand, PartialEq)]
pub enum CommandMode {
    /// Quick creation with minimal prompts
    Quick,
    /// Step-by-step guided creation
    Guided,
    /// Create from existing template
    Template { source: String },
    /// Preview what would be created
    Preview,
    /// Edit existing command
    Edit,
    /// Force overwrite existing
    Force,
}

/// Agent creation modes
#[derive(Debug, Subcommand, PartialEq)]
pub enum AgentMode {
    /// Quick creation with minimal prompts
    Quick,
    /// Step-by-step guided creation
    Guided,
    /// Expert mode with full configuration
    Expert,
    /// Create from existing template
    Template { source: String },
    /// Preview what would be created
    Preview,
    /// Edit existing agent
    Edit,
    /// Force overwrite existing
    Force,
}

impl CreateArgs {
    pub async fn execute(self, _os: &mut Os) -> Result<ExitCode> {
        match self.command {
            CreateCommand::Skill { name, mode } => {
                let creation_mode = mode.map(|m| match m {
                    SkillMode::Quick => CreationMode::Quick,
                    SkillMode::Guided => CreationMode::Guided,
                    SkillMode::Expert => CreationMode::Expert,
                    SkillMode::Template { source } => CreationMode::Template,
                    SkillMode::Preview => CreationMode::Preview,
                    SkillMode::Edit => CreationMode::Guided, // Edit as guided
                    SkillMode::Force => CreationMode::Guided, // Force as guided
                }).unwrap_or(CreationMode::Guided); // Default to guided

                let flow = SkillCreationFlow::new(name, creation_mode)?;
                CreationAssistant::new(flow).run().await
            },
            CreateCommand::Command { name, mode } => {
                let creation_mode = mode.map(|m| match m {
                    CommandMode::Quick => CreationMode::Quick,
                    CommandMode::Guided => CreationMode::Guided,
                    CommandMode::Template { source } => CreationMode::Template,
                    CommandMode::Preview => CreationMode::Preview,
                    CommandMode::Edit => CreationMode::Guided,
                    CommandMode::Force => CreationMode::Guided,
                }).unwrap_or(CreationMode::Guided);

                let flow = CommandCreationFlow::new(name, creation_mode)?;
                CreationAssistant::new(flow).run().await
            },
            CreateCommand::Agent { name, mode } => {
                let creation_mode = mode.map(|m| match m {
                    AgentMode::Quick => CreationMode::Quick,
                    AgentMode::Guided => CreationMode::Guided,
                    AgentMode::Expert => CreationMode::Expert,
                    AgentMode::Template { source } => CreationMode::Template,
                    AgentMode::Preview => CreationMode::Preview,
                    AgentMode::Edit => CreationMode::Guided,
                    AgentMode::Force => CreationMode::Guided,
                }).unwrap_or(CreationMode::Guided);

                let flow = AgentCreationFlow::new(name, creation_mode)?;
                CreationAssistant::new(flow).run().await
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_cisco_style_skill_parsing() {
        // Basic skill creation
        let args = CreateArgs::try_parse_from(&["create", "skill", "myskill"]).unwrap();
        match args.command {
            CreateCommand::Skill { name, mode } => {
                assert_eq!(name, "myskill");
                assert_eq!(mode, None);
            }
            _ => panic!("Expected Skill command"),
        }

        // Skill with quick mode
        let args = CreateArgs::try_parse_from(&["create", "skill", "myskill", "quick"]).unwrap();
        match args.command {
            CreateCommand::Skill { name, mode } => {
                assert_eq!(name, "myskill");
                assert_eq!(mode, Some(SkillMode::Quick));
            }
            _ => panic!("Expected Skill command"),
        }

        // Skill with template
        let args = CreateArgs::try_parse_from(&[
            "create", "skill", "myskill", "template", "existing"
        ]).unwrap();
        match args.command {
            CreateCommand::Skill { name, mode } => {
                assert_eq!(name, "myskill");
                match mode {
                    Some(SkillMode::Template { source }) => {
                        assert_eq!(source, "existing");
                    }
                    _ => panic!("Expected Template mode"),
                }
            }
            _ => panic!("Expected Skill command"),
        }
    }

    #[test]
    fn test_cisco_style_command_parsing() {
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
    fn test_cisco_style_agent_parsing() {
        let args = CreateArgs::try_parse_from(&["create", "agent", "myagent", "expert"]).unwrap();
        match args.command {
            CreateCommand::Agent { name, mode } => {
                assert_eq!(name, "myagent");
                assert_eq!(mode, Some(AgentMode::Expert));
            }
            _ => panic!("Expected Agent command"),
        }
    }

    #[test]
    fn test_bash_style_flags_rejected() {
        // Should reject bash-style --flags
        assert!(CreateArgs::try_parse_from(&[
            "create", "skill", "myskill", "--interactive"
        ]).is_err());
        
        assert!(CreateArgs::try_parse_from(&[
            "create", "skill", "myskill", "--quick"
        ]).is_err());
    }
}
