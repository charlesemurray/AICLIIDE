//! Creation system - Phase 0 implementation
//! 
//! Provides template management, test case generation, and quality validation
//! for AI assistant prompts and executable commands with specialized builders.

use eyre::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};

pub mod template_manager;
pub mod test_cases;
pub mod prompt_builder;
pub mod metrics;
pub mod examples;
pub mod creation_builder;
pub mod command_builder;

#[cfg(test)]
mod tests;

// Re-export main types
pub use template_manager::TemplateManager;
pub use test_cases::{TestCaseManager, PromptTestCase};
pub use prompt_builder::PromptBuilder;
pub use metrics::PromptMetrics;
pub use creation_builder::{CreationBuilder, ValidationResult, ValidationIssue, IssueSeverity};
pub use command_builder::{CommandBuilder, CommandConfig};

/// Core prompt template structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    pub name: String,
    pub description: String,
    pub role: String,
    pub capabilities: Vec<String>,
    pub constraints: Vec<String>,
    pub example_conversation: Option<ExampleConversation>,
    pub metadata: TemplateMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExampleConversation {
    pub input: String,
    pub output: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateMetadata {
    pub category: TemplateCategory,
    pub difficulty: DifficultyLevel,
    pub tags: Vec<String>,
    pub usage_stats: UsageStats,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TemplateCategory {
    CodeReview,
    Documentation,
    DomainExpert,
    GeneralAssistant,
    Custom,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DifficultyLevel {
    Beginner,
    Intermediate,
    Advanced,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    pub success_rate: f64,
    pub avg_satisfaction: f64,
    pub usage_count: u32,
}

impl Default for TemplateMetadata {
    fn default() -> Self {
        Self {
            category: TemplateCategory::GeneralAssistant,
            difficulty: DifficultyLevel::Beginner,
            tags: Vec::new(),
            usage_stats: UsageStats {
                success_rate: 0.0,
                avg_satisfaction: 0.0,
                usage_count: 0,
            },
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

impl PromptTemplate {
    /// Validate template structure and content
    pub fn validate(&self) -> Result<ValidationResult> {
        let mut issues = Vec::new();
        
        if self.name.is_empty() {
            issues.push(ValidationIssue {
                severity: IssueSeverity::Error,
                message: "Template name cannot be empty".to_string(),
                suggestion: Some("Provide a descriptive name like 'Code Reviewer'".to_string()),
            });
        }
        
        if self.role.len() < 20 {
            issues.push(ValidationIssue {
                severity: IssueSeverity::Warning,
                message: "Role definition is very short".to_string(),
                suggestion: Some("Add more context about the assistant's expertise and approach".to_string()),
            });
        }
        
        if self.capabilities.is_empty() {
            issues.push(ValidationIssue {
                severity: IssueSeverity::Warning,
                message: "No capabilities defined".to_string(),
                suggestion: Some("Add specific capabilities this assistant should have".to_string()),
            });
        }
        
        let error_count = issues.iter().filter(|i| i.severity == IssueSeverity::Error).count();
        let is_valid = error_count == 0;
        
        // Calculate score based on completeness and quality
        let mut score = 1.0f64;
        if self.role.len() < 50 { score -= 0.2; }
        if self.capabilities.is_empty() { score -= 0.3; }
        if self.constraints.is_empty() { score -= 0.2; }
        if self.example_conversation.is_none() { score -= 0.1; }
        
        score = score.max(0.0).min(1.0);
        
        Ok(ValidationResult {
            is_valid,
            issues,
            score,
        })
    }
    
    /// Generate a complete prompt from template
    pub fn generate_prompt(&self) -> Result<String> {
        let mut prompt = String::new();
        
        // Add role
        prompt.push_str(&self.role);
        
        // Add capabilities if any
        if !self.capabilities.is_empty() {
            prompt.push_str("\n\nYour key capabilities include:\n");
            for capability in &self.capabilities {
                prompt.push_str(&format!("- {}\n", capability));
            }
        }
        
        // Add constraints if any
        if !self.constraints.is_empty() {
            prompt.push_str("\nWhen responding, always:\n");
            for constraint in &self.constraints {
                prompt.push_str(&format!("- {}\n", constraint));
            }
        }
        
        // Add example if available
        if let Some(example) = &self.example_conversation {
            prompt.push_str(&format!("\nExample interaction:\nUser: {}\nAssistant: {}", 
                example.input, example.output));
        }
        
        Ok(prompt)
    }
}
