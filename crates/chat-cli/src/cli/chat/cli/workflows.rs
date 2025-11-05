use clap::Subcommand;
use std::path::PathBuf;

use crate::cli::chat::{ChatError, ChatSession, ChatState};
use crate::cli::workflows::{WorkflowRegistry, validate_workflow};
use crate::cli::chat::tools::workflow::WorkflowDefinition;
use crate::os::Os;

#[derive(Debug, PartialEq, Subcommand)]
pub enum WorkflowsSubcommand {
    /// List available workflows
    List {
        /// Show workflows from specific scope (workspace, global, all)
        #[arg(long, default_value = "all")]
        scope: String,
    },
    /// Show information about a specific workflow
    Info {
        /// Name of the workflow
        workflow_name: String,
    },
    /// Create a new workflow interactively
    Create {
        /// Name of the workflow to create
        name: String,
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
    /// Run a workflow
    Run {
        /// Name of the workflow to run
        workflow_name: String,
        /// Parameters as JSON string
        #[arg(long)]
        params: Option<String>,
    },
}

impl WorkflowsSubcommand {
    pub fn name(&self) -> &'static str {
        match self {
            Self::List { .. } => "list",
            Self::Info { .. } => "info",
            Self::Create { .. } => "create",
            Self::Add { .. } => "add",
            Self::Remove { .. } => "remove",
            Self::Run { .. } => "run",
        }
    }

    pub async fn execute(
        self,
        _session: &mut ChatSession,
        os: &mut Os,
    ) -> Result<ChatState, ChatError> {
        use crossterm::{execute, style};
        
        let current_dir = std::env::current_dir()
            .map_err(|e| ChatError::Custom(format!("Failed to get current directory: {}", e).into()))?;
        
        let workflow_dir = current_dir.join(".q-workflows");
        
        let message = match self {
            Self::List { scope } => {
                let mut registry = WorkflowRegistry::new(workflow_dir.clone());
                registry.load_from_directory(&workflow_dir).await
                    .map_err(|e| ChatError::Custom(format!("Failed to load workflows: {}", e).into()))?;
                
                let workflows = registry.list_workflows();
                
                if workflows.is_empty() {
                    format!("No workflows found in {}\n\nCreate one with: /workflows create <name>", workflow_dir.display())
                } else {
                    let mut output = format!("Available workflows ({} scope):\n\n", scope);
                    for workflow in workflows {
                        output.push_str(&format!("  • {} (v{})\n    {}\n\n", 
                            workflow.name, 
                            workflow.version,
                            workflow.description
                        ));
                    }
                    output
                }
            }
            Self::Info { workflow_name } => {
                let mut registry = WorkflowRegistry::new(workflow_dir.clone());
                registry.load_from_directory(&workflow_dir).await
                    .map_err(|e| ChatError::Custom(format!("Failed to load workflows: {}", e).into()))?;
                
                match registry.get(&workflow_name) {
                    Some(workflow) => {
                        let mut output = format!("Workflow: {}\n", workflow.name);
                        output.push_str(&format!("Version: {}\n", workflow.version));
                        output.push_str(&format!("Description: {}\n\n", workflow.description));
                        output.push_str(&format!("Steps ({}):\n", workflow.steps.len()));
                        for (i, step) in workflow.steps.iter().enumerate() {
                            output.push_str(&format!("  {}. {} (tool: {})\n", i + 1, step.name, step.tool));
                        }
                        output
                    }
                    None => format!("Workflow '{}' not found", workflow_name)
                }
            }
            Self::Create { name } => {
                use crate::cli::workflows::creation_assistant::WorkflowCreationAssistant;
                
                let mut assistant = WorkflowCreationAssistant::new(&name);
                assistant.start_discovery()
            }
            Self::Add { source } => {
                let content = std::fs::read_to_string(&source)
                    .map_err(|e| ChatError::Custom(format!("Failed to read file: {}", e).into()))?;
                
                let workflow: WorkflowDefinition = serde_json::from_str(&content)
                    .map_err(|e| ChatError::Custom(format!("Invalid workflow JSON: {}", e).into()))?;
                
                validate_workflow(&workflow)
                    .map_err(|e| ChatError::Custom(format!("Invalid workflow: {}", e).into()))?;
                
                let mut registry = WorkflowRegistry::new(workflow_dir.clone());
                let saved_path = registry.save_workflow(&workflow).await
                    .map_err(|e| ChatError::Custom(format!("Failed to save workflow: {}", e).into()))?;
                
                format!("✅ Workflow '{}' added successfully\nSaved to: {}", workflow.name, saved_path.display())
            }
            Self::Remove { workflow_name } => {
                let mut registry = WorkflowRegistry::new(workflow_dir.clone());
                registry.load_from_directory(&workflow_dir).await
                    .map_err(|e| ChatError::Custom(format!("Failed to load workflows: {}", e).into()))?;
                
                registry.delete_workflow(&workflow_name).await
                    .map_err(|e| ChatError::Custom(format!("Failed to remove workflow: {}", e).into()))?;
                
                format!("✅ Workflow '{}' removed successfully", workflow_name)
            }
            Self::Run { workflow_name, params } => {
                let mut registry = WorkflowRegistry::new(workflow_dir.clone());
                registry.load_from_directory(&workflow_dir).await
                    .map_err(|e| ChatError::Custom(format!("Failed to load workflows: {}", e).into()))?;
                
                match registry.get(&workflow_name) {
                    Some(workflow) => {
                        use crate::cli::chat::tools::workflow::WorkflowTool;
                        use std::collections::HashMap;
                        
                        let params_map: HashMap<String, serde_json::Value> = if let Some(params_str) = params {
                            serde_json::from_str(&params_str)
                                .map_err(|e| ChatError::Custom(format!("Invalid JSON parameters: {}", e).into()))?
                        } else {
                            HashMap::new()
                        };
                        
                        let tool = WorkflowTool::from_definition(workflow);
                        
                        match tool.invoke_with_definition_and_manager(workflow, params_map, Some(&mut _session.conversation.tool_manager)).await {
                            Ok(result) => format!("🔄 Workflow '{}' completed\n\n{}", workflow_name, result),
                            Err(e) => format!("❌ Workflow '{}' failed: {}", workflow_name, e),
                        }
                    }
                    None => format!("❌ Workflow '{}' not found", workflow_name)
                }
            }
        };

        execute!(_session.stderr, style::Print(&message), style::Print("\n"))?;
        
        Ok(ChatState::PromptUser {
            skip_printing_tools: false,
        })
    }
}
