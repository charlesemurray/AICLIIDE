//! Assistant management commands

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{
    Args,
    Subcommand,
};
use eyre::Result;

use crate::cli::creation::TerminalUIImpl;
use crate::cli::creation::prompt_system::{
    AssistantEditor,
    ConflictStrategy,
    InteractivePromptBuilder,
    delete_template,
    export_all_assistants,
    export_assistant,
    import_assistant,
    list_templates,
    load_template,
    save_template,
};

#[derive(Debug, Args, PartialEq)]
pub struct AssistantArgs {
    #[command(subcommand)]
    pub command: AssistantCommand,
}

#[derive(Debug, Subcommand, PartialEq)]
pub enum AssistantCommand {
    /// Create a new assistant
    Create {
        #[command(subcommand)]
        mode: Option<CreateMode>,
    },
    /// List all saved assistants
    List,
    /// Edit an existing assistant
    Edit {
        /// ID of the assistant to edit
        id: String,
    },
    /// Delete an assistant
    Delete {
        /// ID of the assistant to delete
        id: String,
    },
    /// Export an assistant to a file
    Export {
        /// ID of the assistant
        id: String,
        /// Output file path
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Export all assistants to a directory
    ExportAll {
        /// Output directory
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Import an assistant from a file
    Import {
        /// Input file path
        path: PathBuf,
        /// Conflict strategy: skip, overwrite, or rename
        #[arg(short, long, default_value = "rename")]
        strategy: String,
    },
}

#[derive(Debug, Subcommand, PartialEq)]
pub enum CreateMode {
    /// Use a pre-built template
    Template,
    /// Build from scratch
    Custom,
}

impl AssistantArgs {
    pub async fn execute(self) -> Result<ExitCode> {
        match self.command {
            AssistantCommand::Create { mode } => {
                let mut ui = TerminalUIImpl::new();
                let mut builder = InteractivePromptBuilder::new(&mut ui);

                let template = match mode {
                    Some(CreateMode::Custom) => builder.create_custom()?,
                    _ => builder.create_from_template()?,
                };

                let path = save_template(&template)?;

                println!("\n✓ Created assistant: {}", template.name);
                println!("  Category: {:?}", template.category);
                println!("  Difficulty: {:?}", template.difficulty);
                println!("  Capabilities: {}", template.capabilities.len());
                println!("  Saved to: {}", path.display());

                Ok(ExitCode::SUCCESS)
            },
            AssistantCommand::List => {
                let templates = list_templates()?;

                if templates.is_empty() {
                    println!("No assistants found. Create one with: q assistant create");
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
            AssistantCommand::Edit { id } => {
                let template = load_template(&id)?;

                let mut ui = TerminalUIImpl::new();
                let editor = AssistantEditor::new(&mut ui, template);
                let updated = editor.edit()?;

                save_template(&updated)?;

                println!("\n✓ Updated assistant: {}", updated.name);
                println!("  Saved to: ~/.q-skills/{}.json", updated.id);

                Ok(ExitCode::SUCCESS)
            },
            AssistantCommand::Delete { id } => {
                delete_template(&id)?;
                println!("✓ Deleted assistant: {}", id);

                Ok(ExitCode::SUCCESS)
            },
            AssistantCommand::Export { id, output } => {
                let path = export_assistant(&id, &output)?;
                println!("✓ Exported: {}", id);
                println!("  To: {}", path.display());

                Ok(ExitCode::SUCCESS)
            },
            AssistantCommand::ExportAll { output } => {
                let paths = export_all_assistants(&output)?;
                println!("✓ Exported {} assistants to {}", paths.len(), output.display());
                for path in paths {
                    println!("  - {}", path.file_name().unwrap().to_string_lossy());
                }

                Ok(ExitCode::SUCCESS)
            },
            AssistantCommand::Import { path, strategy } => {
                let conflict_strategy = match strategy.as_str() {
                    "skip" => ConflictStrategy::Skip,
                    "overwrite" => ConflictStrategy::Overwrite,
                    _ => ConflictStrategy::Rename,
                };

                let id = import_assistant(&path, conflict_strategy)?;
                println!("✓ Imported as: {}", id);

                Ok(ExitCode::SUCCESS)
            },
        }
    }
}
