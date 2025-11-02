//! Interactive prompt builder for step-by-step creation

use super::{PromptTemplate, ExampleConversation, TemplateCategory, DifficultyLevel, UsageStats};
use super::creation_builder::{CreationBuilder, ValidationResult, ValidationIssue, IssueSeverity};
use eyre::Result;
use chrono::Utc;

/// Interactive prompt builder for step-by-step creation
pub struct PromptBuilder {
    id: String,
    name: String,
    description: String,
    role: String,
    capabilities: Vec<String>,
    constraints: Vec<String>,
    category: TemplateCategory,
    difficulty: DifficultyLevel,
    tags: Vec<String>,
    examples: Vec<ExampleConversation>,
}

impl PromptBuilder {
    pub fn new() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            description: String::new(),
            role: String::new(),
            capabilities: Vec::new(),
            constraints: Vec::new(),
            category: TemplateCategory::ConversationAssistant,
            difficulty: DifficultyLevel::Beginner,
            tags: Vec::new(),
            examples: Vec::new(),
        }
    }
    
    /// Set the role definition
    pub fn with_role(mut self, role: String) -> Self {
        self.role = role;
        self
    }
    
    /// Add a capability
    pub fn add_capability(mut self, capability: String) -> Self {
        self.capabilities.push(capability);
        self
    }
    
    /// Add multiple capabilities
    pub fn with_capabilities(mut self, capabilities: Vec<String>) -> Self {
        self.capabilities = capabilities;
        self
    }
    
    /// Add a constraint
    pub fn add_constraint(mut self, constraint: String) -> Self {
        self.constraints.push(constraint);
        self
    }
    
    /// Add multiple constraints
    pub fn with_constraints(mut self, constraints: Vec<String>) -> Self {
        self.constraints = constraints;
        self
    }
    
    /// Set example conversation
    pub fn with_example(mut self, input: String, output: String) -> Self {
        self.examples.push(ExampleConversation { input, output });
        self
    }
    
    /// Set template category
    pub fn with_category(mut self, category: TemplateCategory) -> Self {
        self.category = category;
        self
    }
    
    /// Set difficulty level
    pub fn with_difficulty(mut self, difficulty: DifficultyLevel) -> Self {
        self.difficulty = difficulty;
        self
    }
    
    /// Add tags
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
    
    /// Generate preview of the prompt
    pub fn preview(&self) -> String {
        let mut preview = format!("Role: {}\n\n", self.role);
        
        if !self.capabilities.is_empty() {
            preview.push_str("Capabilities:\n");
            for cap in &self.capabilities {
                preview.push_str(&format!("- {}\n", cap));
            }
            preview.push('\n');
        }
        
        if !self.constraints.is_empty() {
            preview.push_str("Constraints:\n");
            for constraint in &self.constraints {
                preview.push_str(&format!("- {}\n", constraint));
            }
        }
        
        preview
    }
}

impl CreationBuilder for PromptBuilder {
    type Output = PromptTemplate;
    
    fn with_name(mut self, name: String) -> Self {
        self.name = name.clone();
        self.id = name.to_lowercase().replace(' ', "_");
        self
    }
    
    fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }
    
    fn validate(&self) -> Result<ValidationResult> {
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
        if self.examples.is_empty() { score -= 0.1; }
        
        score = score.max(0.0).min(1.0);
        
        Ok(ValidationResult {
            is_valid,
            issues,
            score,
        })
    }
    
    fn build(self) -> Result<PromptTemplate> {
        let validation = self.validate()?;
        if !validation.is_valid {
            let errors: Vec<_> = validation.issues
                .iter()
                .filter(|i| i.severity == IssueSeverity::Error)
                .map(|i| i.message.clone())
                .collect();
            return Err(eyre::eyre!("Template validation failed: {}", errors.join(", ")));
        }
        
        let now = Utc::now();
        Ok(PromptTemplate {
            id: self.id,
            name: self.name,
            description: self.description,
            version: 1,
            category: self.category,
            difficulty: self.difficulty,
            tags: self.tags,
            role: self.role,
            capabilities: self.capabilities,
            constraints: self.constraints,
            context: None,
            parameters: Vec::new(),
            examples: self.examples,
            quality_indicators: Vec::new(),
            created_at: now,
            updated_at: now,
            usage_stats: UsageStats {
                success_rate: 0.0,
                avg_satisfaction: 0.0,
                usage_count: 0,
            },
        })
    }
}
