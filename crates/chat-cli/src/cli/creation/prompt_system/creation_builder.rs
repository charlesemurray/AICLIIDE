//! Shared foundation for creation builders

use eyre::Result;

/// Shared trait for all creation builders
pub trait CreationBuilder {
    type Output;
    
    /// Set the name (required for all creations)
    fn with_name(self, name: String) -> Self;
    
    /// Set the description (optional but recommended)
    fn with_description(self, description: String) -> Self;
    
    /// Validate the current configuration
    fn validate(&self) -> Result<ValidationResult>;
    
    /// Build the final result
    fn build(self) -> Result<Self::Output>;
}

/// Validation result shared across builders
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub issues: Vec<ValidationIssue>,
    pub score: f64,
}

#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub severity: IssueSeverity,
    pub message: String,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IssueSeverity {
    Error,
    Warning,
    Info,
}

impl ValidationResult {
    pub fn valid() -> Self {
        Self {
            is_valid: true,
            issues: Vec::new(),
            score: 1.0,
        }
    }
    
    pub fn invalid(issues: Vec<ValidationIssue>) -> Self {
        let error_count = issues.iter().filter(|i| i.severity == IssueSeverity::Error).count();
        let is_valid = error_count == 0;
        
        Self {
            is_valid,
            issues,
            score: if is_valid { 0.5 } else { 0.0 },
        }
    }
}
