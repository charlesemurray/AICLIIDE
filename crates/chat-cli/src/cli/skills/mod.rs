use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub mod builtin;
pub mod registry;
pub mod tests;
pub mod types;
pub mod validation;

pub use registry::SkillRegistry;
pub use types::*;
pub use validation::SkillValidator;

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
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
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
