use std::fs;
use std::io::{
    self,
    Write,
};
use std::process::ExitCode;

use clap::{
    Args,
    Subcommand,
};
use eyre::Result;

use crate::cli::chat::workflow_registry::WorkflowRegistry;
use crate::os::Os;

#[derive(Debug, Args, PartialEq)]
pub struct WorkflowsArgs {
    #[command(subcommand)]
    pub command: WorkflowsCommand,
}

#[derive(Debug, Subcommand, PartialEq)]
pub enum WorkflowsCommand {
    /// List available workflows
    List,
    /// Show information about a specific workflow
    Show {
        /// Name of the workflow
        workflow_name: String,
    },
    /// Add a workflow from a file
    Add {
        /// Path to workflow definition file
        source: String,
    },
    /// Remove a workflow
    Remove {
        /// Name of the workflow to remove
        workflow_name: String,
    },
}

impl WorkflowsArgs {
    pub async fn execute(self, os: &Os) -> Result<ExitCode> {
        match self.command {
            WorkflowsCommand::List => self.list_workflows(os).await,
            WorkflowsCommand::Show { workflow_name } => self.show_workflow(os, &workflow_name).await,
            WorkflowsCommand::Add { source } => self.add_workflow(os, &source).await,
            WorkflowsCommand::Remove { workflow_name } => self.remove_workflow(os, &workflow_name).await,
        }
    }

    async fn list_workflows(&self, os: &Os) -> Result<ExitCode> {
        let registry = WorkflowRegistry::with_all_workflows(&os.env.current_dir()?).await?;

        if registry.is_empty() {
            println!("No workflows found.");
            return Ok(ExitCode::SUCCESS);
        }

        println!("Available workflows:\n");
        for workflow in registry.list_workflows() {
            println!("  {} (v{})", workflow.name, workflow.version);
            println!("    {}", workflow.description);
            println!("    Steps: {}", workflow.steps.len());
            println!();
        }

        Ok(ExitCode::SUCCESS)
    }

    async fn show_workflow(&self, os: &Os, name: &str) -> Result<ExitCode> {
        let registry = WorkflowRegistry::with_all_workflows(&os.env.current_dir()?).await?;

        let workflow = registry.get(name).ok_or_else(|| eyre::eyre!("Workflow '{}' not found", name))?;

        println!("Workflow: {}", workflow.name);
        println!("Version: {}", workflow.version);
        println!("Description: {}", workflow.description);
        println!("\nSteps ({}):", workflow.steps.len());

        for (i, step) in workflow.steps.iter().enumerate() {
            println!("  {}. {} (tool: {})", i + 1, step.name, step.tool);
        }

        Ok(ExitCode::SUCCESS)
    }

    async fn add_workflow(&self, os: &Os, source: &str) -> Result<ExitCode> {
        let current_dir = os.env.current_dir()?;
        let workflows_dir = current_dir.join(".q-workflows");

        // Create workflows directory if it doesn't exist
        if !workflows_dir.exists() {
            fs::create_dir_all(&workflows_dir)?;
        }

        // Read and validate workflow file
        let content = fs::read_to_string(source)?;
        let workflow: crate::cli::chat::tools::workflow::WorkflowDefinition = serde_json::from_str(&content)?;

        // Copy to workflows directory
        let dest = workflows_dir.join(format!("{}.json", workflow.name));
        if dest.exists() {
            print!("Workflow '{}' already exists. Overwrite? (y/N): ", workflow.name);
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            if input.trim().to_lowercase() != "y" {
                println!("Cancelled");
                return Ok(ExitCode::SUCCESS);
            }
        }

        fs::copy(source, &dest)?;
        println!("✓ Added workflow '{}'", workflow.name);

        Ok(ExitCode::SUCCESS)
    }

    async fn remove_workflow(&self, os: &Os, name: &str) -> Result<ExitCode> {
        let current_dir = os.env.current_dir()?;
        let workflows_dir = current_dir.join(".q-workflows");

        if !workflows_dir.exists() {
            return Err(eyre::eyre!("No workflows directory found"));
        }

        let workflow_file = workflows_dir.join(format!("{}.json", name));
        if !workflow_file.exists() {
            return Err(eyre::eyre!("Workflow '{}' not found", name));
        }

        // Confirm removal
        print!("Remove workflow '{}'? (y/N): ", name);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if input.trim().to_lowercase() != "y" {
            println!("Cancelled");
            return Ok(ExitCode::SUCCESS);
        }

        fs::remove_file(&workflow_file)?;
        println!("✓ Removed workflow '{}'", name);

        Ok(ExitCode::SUCCESS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflows_command_parse() {
        use clap::Parser;

        #[derive(Parser)]
        struct TestCli {
            #[command(subcommand)]
            command: WorkflowsCommand,
        }

        let cli = TestCli::parse_from(["test", "list"]);
        assert_eq!(cli.command, WorkflowsCommand::List);

        let cli = TestCli::parse_from(["test", "show", "my-workflow"]);
        assert_eq!(
            cli.command,
            WorkflowsCommand::Show {
                workflow_name: "my-workflow".to_string()
            }
        );

        let cli = TestCli::parse_from(["test", "add", "workflow.json"]);
        assert_eq!(
            cli.command,
            WorkflowsCommand::Add {
                source: "workflow.json".to_string()
            }
        );

        let cli = TestCli::parse_from(["test", "remove", "my-workflow"]);
        assert_eq!(
            cli.command,
            WorkflowsCommand::Remove {
                workflow_name: "my-workflow".to_string()
            }
        );
    }
}
