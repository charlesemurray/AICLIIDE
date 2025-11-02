use std::time::Duration;

use async_trait::async_trait;
use serde::{
    Deserialize,
    Serialize,
};
use tokio::time::timeout;

pub mod builtin;
pub mod creation_assistant;
pub mod platform;
pub mod registry;
pub mod security;
pub mod security_logging;
pub mod security_testing;
pub mod security_tools;
pub mod tests;
pub mod types;
pub mod validation;

#[cfg(test)]
mod unit_tests;

pub use registry::SkillRegistry;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillResult {
    pub output: String,
    pub ui_updates: Option<Vec<UIUpdate>>,
    pub state_changes: Option<serde_json::Value>,
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
    #[error("Skill not found")]
    NotFound,
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
    async fn render_ui(&self) -> Result<SkillUI>;
    fn supports_interactive(&self) -> bool {
        false
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
