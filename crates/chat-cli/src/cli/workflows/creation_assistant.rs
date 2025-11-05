use std::path::PathBuf;

use crate::cli::chat::tools::workflow::{WorkflowDefinition, WorkflowStep};

#[derive(Debug, Clone, PartialEq)]
pub enum CreationState {
    Discovery,
    AddingSteps,
    Review,
    Complete,
}

pub struct WorkflowCreationSession {
    workflow_name: String,
    state: CreationState,
    description: Option<String>,
    steps: Vec<WorkflowStep>,
}

impl WorkflowCreationSession {
    pub fn new(workflow_name: &str) -> Self {
        Self {
            workflow_name: workflow_name.to_string(),
            state: CreationState::Discovery,
            description: None,
            steps: Vec::new(),
        }
    }

    pub fn workflow_name(&self) -> &str {
        &self.workflow_name
    }

    pub fn state(&self) -> &CreationState {
        &self.state
    }

    pub fn set_description(&mut self, description: String) {
        self.description = Some(description);
        self.state = CreationState::AddingSteps;
    }

    pub fn add_step(&mut self, name: String, tool: String, parameters: serde_json::Value) {
        self.steps.push(WorkflowStep {
            name,
            tool,
            parameters,
        });
    }

    pub fn steps(&self) -> &[WorkflowStep] {
        &self.steps
    }

    pub fn move_to_review(&mut self) {
        self.state = CreationState::Review;
    }

    pub fn complete(&mut self) {
        self.state = CreationState::Complete;
    }

    pub fn to_definition(&self) -> WorkflowDefinition {
        WorkflowDefinition {
            name: self.workflow_name.clone(),
            version: "1.0.0".to_string(),
            description: self.description.clone().unwrap_or_default(),
            steps: self.steps.clone(),
            context: None,
        }
    }
}

pub struct WorkflowCreationAssistant {
    session: WorkflowCreationSession,
}

impl WorkflowCreationAssistant {
    pub fn new(workflow_name: &str) -> Self {
        Self {
            session: WorkflowCreationSession::new(workflow_name),
        }
    }

    pub fn session(&self) -> &WorkflowCreationSession {
        &self.session
    }

    pub fn session_mut(&mut self) -> &mut WorkflowCreationSession {
        &mut self.session
    }

    pub fn start_discovery(&mut self) -> String {
        format!(
            "🔄 Workflow Creation Assistant\nCreating workflow: {}\n\nWhat does this workflow do? Describe the sequence of tasks.",
            self.session.workflow_name()
        )
    }

    pub fn handle_discovery_response(&mut self, user_input: &str) -> String {
        self.session.set_description(user_input.to_string());
        
        "Great! Now let's add steps to your workflow.\n\n\
        Each step needs:\n\
        - A name (e.g., 'read_file')\n\
        - A tool to use (e.g., 'fs_read', 'execute_bash', or a skill name)\n\
        - Parameters for that tool\n\n\
        Tell me about the first step, or say 'done' if you want to review.".to_string()
    }

    pub fn handle_step_input(&mut self, user_input: &str) -> String {
        if user_input.trim().to_lowercase() == "done" {
            self.session.move_to_review();
            return self.show_review();
        }

        "I'll help you add that step. What's the step name?".to_string()
    }

    pub fn add_step(&mut self, name: String, tool: String, parameters: serde_json::Value) {
        self.session.add_step(name, tool, parameters);
    }

    pub fn show_review(&self) -> String {
        let mut review = format!(
            "📋 Workflow Review\n\nName: {}\nDescription: {}\n\nSteps:\n",
            self.session.workflow_name(),
            self.session.description.as_deref().unwrap_or("(no description)")
        );

        for (i, step) in self.session.steps().iter().enumerate() {
            review.push_str(&format!(
                "{}. {} (using {})\n",
                i + 1,
                step.name,
                step.tool
            ));
        }

        review.push_str("\nSay 'save' to create the workflow, or 'cancel' to discard.");
        review
    }

    pub fn save_workflow(&mut self, workflows_dir: &PathBuf) -> Result<PathBuf, String> {
        let definition = self.session.to_definition();
        let json = serde_json::to_string_pretty(&definition)
            .map_err(|e| format!("Failed to serialize workflow: {}", e))?;

        let file_path = workflows_dir.join(format!("{}.json", self.session.workflow_name()));
        std::fs::write(&file_path, json)
            .map_err(|e| format!("Failed to write workflow file: {}", e))?;

        self.session.complete();
        Ok(file_path)
    }
}
