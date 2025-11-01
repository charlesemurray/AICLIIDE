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
mod templates;
mod prompt_system;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod ui_integration_tests;

pub use types::*;
pub use errors::CreationError;
pub use ui::TerminalUIImpl;
#[cfg(test)]
pub use ui::MockTerminalUI;
pub use assistant::CreationAssistant;
pub use flows::*;
pub use context::CreationContext;
pub use templates::TemplateManager;

use clap::{Args, Subcommand, Parser, CommandFactory};
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
                    SkillMode::Template { source: _ } => CreationMode::Template,
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
                    CommandMode::Template { source: _ } => CreationMode::Template,
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
                    AgentMode::Template { source: _ } => CreationMode::Template,
                    AgentMode::Preview => CreationMode::Preview,
                    AgentMode::Edit => CreationMode::Guided,
                    AgentMode::Force => CreationMode::Guided,
                }).unwrap_or(CreationMode::Guided);

                let flow = AgentCreationFlow::new(name, creation_mode)?;
                CreationAssistant::new(flow).run().await
            },
        }
    }

    #[cfg(test)]
    pub async fn execute_test(self) -> Result<ExitCode> {
        // Test version that doesn't require Os parameter
        match self.command {
            CreateCommand::Skill { name, mode } => {
                let creation_mode = mode.map(|m| match m {
                    SkillMode::Quick => CreationMode::Quick,
                    SkillMode::Guided => CreationMode::Guided,
                    SkillMode::Expert => CreationMode::Expert,
                    SkillMode::Template { source: _ } => CreationMode::Template,
                    SkillMode::Preview => CreationMode::Preview,
                    SkillMode::Edit => CreationMode::Guided,
                    SkillMode::Force => CreationMode::Guided,
                }).unwrap_or(CreationMode::Guided);

                let flow = SkillCreationFlow::new(name, creation_mode)?;
                CreationAssistant::new(flow).run().await
            },
            CreateCommand::Command { name, mode } => {
                let creation_mode = mode.map(|m| match m {
                    CommandMode::Quick => CreationMode::Quick,
                    CommandMode::Guided => CreationMode::Guided,
                    CommandMode::Template { source: _ } => CreationMode::Template,
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
                    AgentMode::Template { source: _ } => CreationMode::Template,
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

impl Parser for CreateArgs {
    fn parse() -> Self {
        Self::parse_from(std::env::args_os())
    }

    fn try_parse() -> Result<Self, clap::Error> {
        Self::try_parse_from(std::env::args_os())
    }

    fn parse_from<I, T>(itr: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        Self::try_parse_from(itr).unwrap_or_else(|e| e.exit())
    }

    fn try_parse_from<I, T>(itr: I) -> Result<Self, clap::Error>
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        use clap::FromArgMatches;
        let cmd = Self::command();
        let matches = cmd.try_get_matches_from(itr)?;
        Self::from_arg_matches(&matches)
    }

    fn update_from<I, T>(&mut self, itr: I)
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        self.try_update_from(itr).unwrap_or_else(|e| e.exit())
    }

    fn try_update_from<I, T>(&mut self, itr: I) -> Result<(), clap::Error>
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        use clap::FromArgMatches;
        let cmd = Self::command();
        let matches = cmd.try_get_matches_from(itr)?;
        self.update_from_arg_matches(&matches)
    }
}

impl CommandFactory for CreateArgs {
    fn command() -> clap::Command {
        use clap::Subcommand;
        let mut cmd = clap::Command::new("create")
            .about("Create skills, commands, and agents")
            .subcommand_required(true);
        cmd = CreateCommand::augment_subcommands(cmd);
        cmd
    }

    fn command_for_update() -> clap::Command {
        Self::command()
    }
}
