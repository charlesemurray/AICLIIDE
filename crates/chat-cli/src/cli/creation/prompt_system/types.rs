use eyre::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: u32,
    
    pub category: TemplateCategory,
    pub difficulty: DifficultyLevel,
    pub tags: Vec<String>,
    
    pub role: String,
    pub capabilities: Vec<String>,
    pub constraints: Vec<String>,
    pub context: Option<String>,
    
    pub parameters: Vec<TemplateParameter>,
    pub examples: Vec<ExampleConversation>,
    pub quality_indicators: Vec<String>,
    
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub usage_stats: UsageStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateParameter {
    pub name: String,
    pub param_type: ParameterType,
    pub description: String,
    pub default_value: Option<String>,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    String { max_length: usize },
    Enum { options: Vec<String> },
    Number { min: f64, max: f64 },
    Boolean,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TemplateCategory {
    CodeReviewer,
    DocumentationWriter,
    DomainExpert,
    ConversationAssistant,
    TaskAutomator,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DifficultyLevel {
    Beginner,
    Intermediate,
    Advanced,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExampleConversation {
    pub input: String,
    pub output: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    pub success_rate: f64,
    pub avg_satisfaction: f64,
    pub usage_count: u32,
}

#[derive(Debug, Clone)]
pub struct QualityScore {
    pub overall_score: f64,
    pub component_scores: HashMap<String, f64>,
    pub feedback: Vec<QualityFeedback>,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub struct QualityFeedback {
    pub severity: FeedbackSeverity,
    pub message: String,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone)]
pub enum FeedbackSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub struct TemplateInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: TemplateCategory,
    pub difficulty: DifficultyLevel,
    pub estimated_quality: f64,
    pub usage_stats: UsageStats,
}

#[derive(Debug, thiserror::Error)]
pub enum TemplateError {
    #[error("Template not found: {id}")]
    NotFound { id: String },
    
    #[error("Template validation failed: {reason}")]
    ValidationFailed { reason: String },
    
    #[error("Template rendering failed: {reason}")]
    RenderingFailed { reason: String },
    
    #[error("Security violation: {violation}")]
    SecurityViolation { violation: String },
    
    #[error("System error: {source}")]
    SystemError { 
        #[from]
        source: eyre::Error
    },
}
