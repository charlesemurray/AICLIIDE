use std::time::Duration;

use async_trait::async_trait;
use serde::{
    Deserialize,
    Serialize,
};
use tokio::time::timeout;

use crate::cli::chat::tools::ToolSpec;

pub mod builtin;
pub mod creation_assistant;
pub mod error_recovery;
pub mod onboarding;
pub mod platform;
pub mod registry;
pub mod security;
pub mod security_logging;
pub mod security_testing;
pub mod security_tools;
pub mod templates;
pub mod tests;
pub mod toolspec_conversion;
pub mod types;
pub mod validation;

#[cfg(test)]
mod unit_tests;

pub use registry::SkillRegistry;
pub use toolspec_conversion::{
    ConversionError,
    ToToolSpec,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillResult {
    pub output: String,
    pub ui_updates: Option<Vec<UIUpdate>>,
    pub state_changes: Option<serde_json::Value>,
    /// Request to create a new chat session for this skill
    pub create_session: Option<SessionRequest>,
    /// Request to switch to an existing session
    pub switch_to_session: Option<String>,
    /// Request to close a session
    pub close_session: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRequest {
    pub name: String,
    pub session_type: crate::theme::session::SessionType,
    pub initial_prompt: Option<String>,
}

impl SkillResult {
    /// Create a result that requests a new session
    pub fn with_session(output: String, name: String, session_type: crate::theme::session::SessionType) -> Self {
        Self {
            output,
            ui_updates: None,
            state_changes: None,
            create_session: Some(SessionRequest {
                name,
                session_type,
                initial_prompt: None,
            }),
            switch_to_session: None,
            close_session: None,
        }
    }
    
    /// Create a result that switches to an existing session
    pub fn switch_session(output: String, session_name: String) -> Self {
        Self {
            output,
            ui_updates: None,
            state_changes: None,
            create_session: None,
            switch_to_session: Some(session_name),
            close_session: None,
        }
    }
    
    /// Create a result that closes a session
    pub fn close_session(output: String, session_name: String) -> Self {
        Self {
            output,
            ui_updates: None,
            state_changes: None,
            create_session: None,
            switch_to_session: None,
            close_session: Some(session_name),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIUpdate {
    pub element_id: String,
    pub update_type: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillUI {
    pub elements: Vec<UIElement>,
    pub interactive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UIElement {
    Text(String),
    Input { id: String, placeholder: String },
    Button { id: String, label: String },
    List { id: String, items: Vec<String> },
}

#[derive(Debug, thiserror::Error)]
pub enum SkillError {
    #[error("Skill '{0}' not found")]
    NotFound(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Execution timeout after {0}s")]
    Timeout(u64),
    #[error("Resource limit exceeded: {0}")]
    ResourceLimit(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[async_trait]
pub trait Skill: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn aliases(&self) -> Vec<String> {
        vec![]
    }
    async fn execute(&self, params: serde_json::Value) -> Result<SkillResult>;
    
    /// Execute skill with security context (default implementation delegates to execute)
    async fn execute_with_security(
        &self,
        params: serde_json::Value,
        _security_context: &security::SecurityContext,
    ) -> Result<SkillResult> {
        // Default: just execute (security checks will be added in later steps)
        self.execute(params).await
    }
    
    async fn render_ui(&self) -> Result<SkillUI>;
    fn supports_interactive(&self) -> bool {
        false
    }
    fn to_toolspec(&self) -> std::result::Result<ToolSpec, toolspec_conversion::ConversionError> {
        Err(toolspec_conversion::ConversionError::InvalidSchema(
            "Skill does not support ToolSpec conversion".to_string(),
        ))
    }
}

pub type Result<T> = std::result::Result<T, SkillError>;

#[derive(Debug, Clone)]
pub struct ResourceLimits {
    pub timeout_seconds: u64,
    pub max_memory_mb: Option<u64>,
    pub max_cpu_percent: Option<u64>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            timeout_seconds: 30,
            max_memory_mb: Some(512),
            max_cpu_percent: Some(80),
        }
    }
}

pub async fn execute_with_timeout<T>(
    future: impl std::future::Future<Output = Result<T>>,
    limits: &ResourceLimits,
) -> Result<T> {
    match timeout(Duration::from_secs(limits.timeout_seconds), future).await {
        Ok(result) => result,
        Err(_) => Err(SkillError::Timeout(limits.timeout_seconds)),
    }
}
