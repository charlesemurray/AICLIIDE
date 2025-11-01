//! Interactive prompt builder for step-by-step creation

use super::{PromptTemplate, TemplateMetadata, ExampleConversation, TemplateCategory, DifficultyLevel};
use super::creation_builder::{CreationBuilder, ValidationResult, ValidationIssue, IssueSeverity};
use eyre::Result;

/// Interactive prompt builder for step-by-step creation
pub struct PromptBuilder {
    template: PromptTemplate,
}

impl PromptBuilder {
    pub fn new() -> Self {
        Self {
            template: PromptTemplate {
                name: String::new(),
                description: String::new(),
                role: String::new(),
                capabilities: Vec::new(),
                constraints: Vec::new(),
                example_conversation: None,
                metadata: TemplateMetadata::default(),
            },
        }
    }
    
    /// Set the role definition
    pub fn with_role(mut self, role: String) -> Self {
        self.template.role = role;
        self
    }
    
    /// Add a capability
    pub fn add_capability(mut self, capability: String) -> Self {
        self.template.capabilities.push(capability);
        self
    }
    
    /// Add multiple capabilities
    pub fn with_capabilities(mut self, capabilities: Vec<String>) -> Self {
        self.template.capabilities = capabilities;
        self
    }
    
    /// Add a constraint
    pub fn add_constraint(mut self, constraint: String) -> Self {
        self.template.constraints.push(constraint);
        self
    }
    
    /// Add multiple constraints
    pub fn with_constraints(mut self, constraints: Vec<String>) -> Self {
        self.template.constraints = constraints;
        self
    }
    
    /// Set example conversation
    pub fn with_example(mut self, input: String, output: String) -> Self {
        self.template.example_conversation = Some(ExampleConversation { input, output });
        self
    }
    
    /// Set template category
    pub fn with_category(mut self, category: TemplateCategory) -> Self {
        self.template.metadata.category = category;
        self
    }
    
    /// Set difficulty level
    pub fn with_difficulty(mut self, difficulty: DifficultyLevel) -> Self {
        self.template.metadata.difficulty = difficulty;
        self
    }
    
    /// Add tags
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.template.metadata.tags = tags;
        self
    }
    
    /// Get current template (for inspection)
    pub fn template(&self) -> &PromptTemplate {
        &self.template
    }
    
    /// Generate preview of the prompt
    pub fn preview(&self) -> Result<String> {
        self.template.generate_prompt()
    }
}

impl CreationBuilder for PromptBuilder {
    type Output = PromptTemplate;
    
    fn with_name(mut self, name: String) -> Self {
        self.template.name = name;
        self
    }
    
    fn with_description(mut self, description: String) -> Self {
        self.template.description = description;
        self
    }
    
    fn validate(&self) -> Result<ValidationResult> {
        let mut issues = Vec::new();
        
        if self.template.name.is_empty() {
            issues.push(ValidationIssue {
                severity: IssueSeverity::Error,
                message: "Template name cannot be empty".to_string(),
                suggestion: Some("Provide a descriptive name like 'Code Reviewer'".to_string()),
            });
        }
        
        if self.template.role.len() < 20 {
            issues.push(ValidationIssue {
                severity: IssueSeverity::Warning,
                message: "Role definition is very short".to_string(),
                suggestion: Some("Add more context about the assistant's expertise and approach".to_string()),
            });
        }
        
        if self.template.capabilities.is_empty() {
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
        if self.template.role.len() < 50 { score -= 0.2; }
        if self.template.capabilities.is_empty() { score -= 0.3; }
        if self.template.constraints.is_empty() { score -= 0.2; }
        if self.template.example_conversation.is_none() { score -= 0.1; }
        
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
        Ok(self.template)
    }
}
