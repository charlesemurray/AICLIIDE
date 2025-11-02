//! Unified Creation Assistant
//!
//! Provides consistent, terminal-native creation experiences for Skills, Custom Commands, and
//! Agents. Follows Cisco-style CLI patterns, Rust best practices, and senior engineering standards.

mod assistant;
mod context;
mod enhanced_prompts;
mod errors;
mod flows;
mod prompt_system;
mod template_loader;
mod templates;
mod types;
mod ui;

#[cfg(test)]
mod cli_integration_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod ui_integration_tests;

use std::process::ExitCode;

pub use assistant::CreationAssistant;
use clap::{
    Args,
    CommandFactory,
    Parser,
    Subcommand,
};
pub use context::CreationContext;
pub use errors::CreationError;
use eyre::Result;
pub use flows::*;
pub use templates::TemplateManager;
pub use types::*;
#[cfg(test)]
pub use ui::MockTerminalUI;
pub use ui::TerminalUIImpl;

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
    /// Create an AI assistant (interactive prompt builder)
    Assistant {
        #[command(subcommand)]
        mode: Option<AssistantMode>,
    },
    /// List saved assistants
    ListAssistants,
    /// Edit an existing assistant
    EditAssistant {
        /// ID of the assistant to edit
        id: String,
    },
    /// Delete an assistant
    DeleteAssistant {
        /// ID of the assistant to delete
        id: String,
    },
    /// Export an assistant to a file
    ExportAssistant {
        /// ID of the assistant
        id: String,
        /// Output file path
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Export all assistants to a directory
    ExportAssistants {
        /// Output directory
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Import an assistant from a file
    ImportAssistant {
        /// Input file path
        path: PathBuf,
        /// Conflict strategy: skip, overwrite, or rename
        #[arg(short, long, default_value = "rename")]
        strategy: String,
    },
}

use std::path::PathBuf;

/// Assistant creation modes
#[derive(Debug, Subcommand, PartialEq)]
pub enum AssistantMode {
    /// Use a pre-built template
    Template,
    /// Build from scratch
    Custom,
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
        use crate::cli::creation::TerminalUIImpl;
        use crate::cli::creation::flows::InteractiveCreationFlow;

        match self.command {
            CreateCommand::Assistant { mode } => {
                use crate::cli::creation::prompt_system::{
                    InteractivePromptBuilder,
                    save_template,
                };

                let mut ui = TerminalUIImpl::new();
                let mut builder = InteractivePromptBuilder::new(&mut ui);

                let template = match mode {
                    Some(AssistantMode::Custom) => builder.create_custom()?,
                    _ => builder.create_from_template()?,
                };

                // Save to disk
                let path = save_template(&template)?;

                println!("\n✓ Created assistant: {}", template.name);
                println!("  Category: {:?}", template.category);
                println!("  Difficulty: {:?}", template.difficulty);
                println!("  Capabilities: {}", template.capabilities.len());
                println!("  Saved to: {}", path.display());

                Ok(ExitCode::SUCCESS)
            },
            CreateCommand::ListAssistants => {
                use crate::cli::creation::prompt_system::{
                    list_templates,
                    load_template,
                };

                let templates = list_templates()?;

                if templates.is_empty() {
                    println!("No assistants found. Create one with: q create assistant");
                    return Ok(ExitCode::SUCCESS);
                }

                println!("Saved assistants:\n");
                for id in templates {
                    if let Ok(template) = load_template(&id) {
                        println!("  {} - {}", id, template.name);
                        println!(
                            "    Category: {:?}, Difficulty: {:?}",
                            template.category, template.difficulty
                        );
                    }
                }

                Ok(ExitCode::SUCCESS)
            },
            CreateCommand::EditAssistant { id } => {
                use crate::cli::creation::prompt_system::{
                    AssistantEditor,
                    load_template,
                    save_template,
                };

                let template = load_template(&id)?;

                let mut ui = TerminalUIImpl::new();
                let editor = AssistantEditor::new(&mut ui, template);
                let updated = editor.edit()?;

                save_template(&updated)?;

                println!("\n✓ Updated assistant: {}", updated.name);
                println!("  Saved to: ~/.q-skills/{}.json", updated.id);

                Ok(ExitCode::SUCCESS)
            },
            CreateCommand::DeleteAssistant { id } => {
                use crate::cli::creation::prompt_system::delete_template;

                delete_template(&id)?;
                println!("✓ Deleted assistant: {}", id);

                Ok(ExitCode::SUCCESS)
            },
            CreateCommand::ExportAssistant { id, output } => {
                use crate::cli::creation::prompt_system::export_assistant;

                let path = export_assistant(&id, &output)?;
                println!("✓ Exported: {}", id);
                println!("  To: {}", path.display());

                Ok(ExitCode::SUCCESS)
            },
            CreateCommand::ExportAssistants { output } => {
                use crate::cli::creation::prompt_system::export_all_assistants;

                let paths = export_all_assistants(&output)?;
                println!("✓ Exported {} assistants to {}", paths.len(), output.display());
                for path in paths {
                    println!("  - {}", path.file_name().unwrap().to_string_lossy());
                }

                Ok(ExitCode::SUCCESS)
            },
            CreateCommand::ImportAssistant { path, strategy } => {
                use crate::cli::creation::prompt_system::{
                    ConflictStrategy,
                    import_assistant,
                };

                let conflict_strategy = match strategy.as_str() {
                    "skip" => ConflictStrategy::Skip,
                    "overwrite" => ConflictStrategy::Overwrite,
                    _ => ConflictStrategy::Rename,
                };

                let id = import_assistant(&path, conflict_strategy)?;
                println!("✓ Imported as: {}", id);

                Ok(ExitCode::SUCCESS)
            },
            CreateCommand::Skill { name: _, mode: _ } => {
                let ui = TerminalUIImpl::new();
                let mut flow = InteractiveCreationFlow::new(ui).await?;
                let result = flow.run(CreationType::Skill).await?;
                println!("Created skill:\n{}", result);
                Ok(ExitCode::SUCCESS)
            },
            CreateCommand::Command { name: _, mode: _ } => {
                let ui = TerminalUIImpl::new();
                let mut flow = InteractiveCreationFlow::new(ui).await?;
                let result = flow.run(CreationType::CustomCommand).await?;
                println!("Created command:\n{}", result);
                Ok(ExitCode::SUCCESS)
            },
            CreateCommand::Agent { name: _, mode: _ } => {
                let ui = TerminalUIImpl::new();
                let mut flow = InteractiveCreationFlow::new(ui).await?;
                let result = flow.run(CreationType::Agent).await?;
                println!("Created agent:\n{}", result);
                Ok(ExitCode::SUCCESS)
            },
        }
    }

    #[cfg(test)]
    pub async fn execute_test(self) -> Result<ExitCode> {
        // Test version that doesn't require Os parameter
        match self.command {
            CreateCommand::Assistant { mode: _ } => {
                // Not implemented in tests yet
                Ok(ExitCode::SUCCESS)
            },
            CreateCommand::Skill { name, mode } => {
                let creation_mode = mode
                    .map(|m| match m {
                        SkillMode::Quick => CreationMode::Quick,
                        SkillMode::Guided => CreationMode::Guided,
                        SkillMode::Expert => CreationMode::Expert,
                        SkillMode::Template { source: _ } => CreationMode::Template,
                        SkillMode::Preview => CreationMode::Preview,
                        SkillMode::Edit => CreationMode::Guided,
                        SkillMode::Force => CreationMode::Guided,
                    })
                    .unwrap_or(CreationMode::Guided);

                let flow = SkillCreationFlow::new(name, creation_mode)?;
                CreationAssistant::new(flow).run().await
            },
            CreateCommand::Command { name, mode } => {
                let creation_mode = mode
                    .map(|m| match m {
                        CommandMode::Quick => CreationMode::Quick,
                        CommandMode::Guided => CreationMode::Guided,
                        CommandMode::Template { source: _ } => CreationMode::Template,
                        CommandMode::Preview => CreationMode::Preview,
                        CommandMode::Edit => CreationMode::Guided,
                        CommandMode::Force => CreationMode::Guided,
                    })
                    .unwrap_or(CreationMode::Guided);

                let flow = CommandCreationFlow::new(name, creation_mode)?;
                CreationAssistant::new(flow).run().await
            },
            CreateCommand::Agent { name, mode } => {
                let creation_mode = mode
                    .map(|m| match m {
                        AgentMode::Quick => CreationMode::Quick,
                        AgentMode::Guided => CreationMode::Guided,
                        AgentMode::Expert => CreationMode::Expert,
                        AgentMode::Template { source: _ } => CreationMode::Template,
                        AgentMode::Preview => CreationMode::Preview,
                        AgentMode::Edit => CreationMode::Guided,
                        AgentMode::Force => CreationMode::Guided,
                    })
                    .unwrap_or(CreationMode::Guided);

                let flow = AgentCreationFlow::new(name, creation_mode)?;
                CreationAssistant::new(flow).run().await
            },
            CreateCommand::ListAssistants => {
                // Not implemented in tests yet
                Ok(ExitCode::SUCCESS)
            },
            CreateCommand::DeleteAssistant { id: _ } => {
                // Not implemented in tests yet
                Ok(ExitCode::SUCCESS)
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
