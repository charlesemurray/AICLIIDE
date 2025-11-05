use serde::{Deserialize, Serialize};

/// Result of workflow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowResult {
    pub success: bool,
    pub output: String,
    pub step_results: Vec<StepResult>,
}

/// Result of a single workflow step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub step_name: String,
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
}

/// Workflow execution state
#[derive(Debug, Clone, PartialEq)]
pub enum WorkflowState {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Workflow error types
#[derive(Debug, thiserror::Error)]
pub enum WorkflowError {
    #[error("Workflow not found: {0}")]
    NotFound(String),
    
    #[error("Invalid workflow definition: {0}")]
    InvalidDefinition(String),
    
    #[error("Step execution failed: {0}")]
    StepFailed(String),
    
    #[error("Workflow execution timeout")]
    Timeout,
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}
