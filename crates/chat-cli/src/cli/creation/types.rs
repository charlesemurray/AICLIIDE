//! Core types and traits for the unified creation system

use serde::{Deserialize, Serialize};
use std::path::Path;
use eyre::Result;

/// Creation complexity levels determine UI flow and feature availability
#[derive(Debug, Clone, PartialEq)]
pub enum ComplexityLevel {
    Low,        // Custom Commands: simple linear flow
    Medium,     // Skills: guided with security/testing
    High,       // Agents: expert with MCP/tools/hooks
}

/// Creation phases that different types go through
#[derive(Debug, Clone, PartialEq)]
pub enum CreationPhase {
    Discovery,      // Understand what user wants to create
    BasicConfig,    // Essential configuration
    AdvancedConfig, // Complex configuration (agents only)
    Security,       // Security settings (skills/agents)
    Testing,        // Validation and testing (skills/agents)
    Completion,     // Final review and save
}

/// Types of creations supported
#[derive(Debug, Clone, PartialEq)]
pub enum CreationType {
    CustomCommand,
    Skill,
    Agent,
}

impl CreationType {
    pub fn complexity_level(&self) -> ComplexityLevel {
        match self {
            CreationType::CustomCommand => ComplexityLevel::Low,
            CreationType::Skill => ComplexityLevel::Medium,
            CreationType::Agent => ComplexityLevel::High,
        }
    }

    pub fn required_phases(&self) -> Vec<CreationPhase> {
        match self {
            CreationType::CustomCommand => vec![
                CreationPhase::Discovery,
                CreationPhase::BasicConfig,
                CreationPhase::Completion,
            ],
            CreationType::Skill => vec![
                CreationPhase::Discovery,
                CreationPhase::BasicConfig,
                CreationPhase::Security,
                CreationPhase::Testing,
                CreationPhase::Completion,
            ],
            CreationType::Agent => vec![
                CreationPhase::Discovery,
                CreationPhase::BasicConfig,
                CreationPhase::AdvancedConfig,
                CreationPhase::Security,
                CreationPhase::Testing,
                CreationPhase::Completion,
            ],
        }
    }
}

/// Result of executing a creation phase
#[derive(Debug)]
pub enum PhaseResult {
    Continue,
    Complete,
    Retry(String), // Error message for retry
}

/// Core trait for all creation flows
pub trait CreationFlow {
    type Config: CreationConfig;
    type Artifact: CreationArtifact;

    fn creation_type(&self) -> CreationType;
    fn execute_phase(&mut self, phase: CreationPhase) -> Result<PhaseResult>;
    fn create_artifact(&self) -> Result<Self::Artifact>;
    fn get_config(&self) -> &Self::Config;
}

/// Configuration validation and defaults
pub trait CreationConfig {
    fn validate(&self) -> Result<()>;
    fn apply_defaults(&mut self);
    fn is_complete(&self) -> bool;
    fn get_name(&self) -> &str;
}

/// Artifact persistence and validation
pub trait CreationArtifact {
    fn persist(&self, location: &Path) -> Result<()>;
    fn validate_before_save(&self) -> Result<()>;
    fn get_name(&self) -> &str;
}

/// Semantic colors for terminal output
#[derive(Debug, Clone, PartialEq)]
pub enum SemanticColor {
    Success,  // Green
    Error,    // Red
    Warning,  // Yellow
    Info,     // Blue
    Debug,    // Cyan
}

/// Terminal UI abstraction
pub trait TerminalUI {
    fn prompt_required(&mut self, field: &str) -> Result<String>;
    fn prompt_optional(&mut self, field: &str, default: Option<&str>) -> Result<Option<String>>;
    fn confirm(&mut self, message: &str) -> Result<bool>;
    fn show_preview(&mut self, content: &str);
    fn show_progress(&mut self, current: usize, total: usize, message: &str);
    fn show_message(&mut self, message: &str, color: SemanticColor);
    
    // New multiple choice methods
    fn select_option(&mut self, prompt: &str, options: &[(&str, &str)]) -> Result<String>;
    fn select_multiple(&mut self, prompt: &str, options: &[(&str, &str)], allow_other: bool) -> Result<Vec<String>>;
}

/// Smart defaults and suggestions based on context
#[derive(Debug, Default)]
pub struct CreationDefaults {
    pub skill_type: Option<SkillType>,
    pub command_type: Option<CommandType>,
    pub command: Option<String>,
    pub description: String,
    pub mcp_servers: Vec<String>,
}

/// Project type detection for smart defaults
#[derive(Debug, Clone, PartialEq)]
pub enum ProjectType {
    Python,
    JavaScript,
    Rust,
    Go,
    Generic,
}

/// Validation result with suggestions
#[derive(Debug)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub error_message: String,
    pub suggestion: String,
}

impl ValidationResult {
    pub fn valid() -> Self {
        Self {
            is_valid: true,
            error_message: String::new(),
            suggestion: String::new(),
        }
    }

    pub fn invalid(error: &str, suggestion: &str) -> Self {
        Self {
            is_valid: false,
            error_message: error.to_string(),
            suggestion: suggestion.to_string(),
        }
    }
}

/// Skill types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SkillType {
    #[serde(rename = "code_inline")]
    CodeInline,
    #[serde(rename = "code_session")]
    CodeSession,
    #[serde(rename = "conversation")]
    Conversation,
    #[serde(rename = "prompt_inline")]
    PromptInline,
}

/// Command types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CommandType {
    Script,
    Alias,
    Builtin,
}

/// Security configuration levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SecurityLevel {
    Low,
    Medium,
    High,
}

/// Creation modes for different user preferences
#[derive(Debug, Clone, PartialEq)]
pub enum CreationMode {
    Quick,      // Minimal prompts, smart defaults
    Guided,     // Step-by-step with explanations
    Expert,     // Full control, all options
    Template,   // Copy from existing
    Preview,    // Show what would be created
    Batch,      // Non-interactive from config
}
