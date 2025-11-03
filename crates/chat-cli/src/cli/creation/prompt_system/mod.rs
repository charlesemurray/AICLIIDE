use std::collections::HashMap;

use eyre::Result;

pub mod command_builder;
pub mod creation_builder;
pub mod edit;
pub mod examples;
pub mod export_import;
pub mod interactive;
pub mod persistence;
pub mod prompt_builder;
pub mod storage;
pub mod template_manager;
pub mod types;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod storage_tests;

#[cfg(test)]
mod manager_tests;

#[cfg(test)]
mod integration_tests;

#[cfg(test)]
mod performance_tests;

#[cfg(test)]
mod error_tests;

#[cfg(test)]
mod builder_tests;

#[cfg(test)]
mod interactive_tests;

#[cfg(test)]
mod e2e_test;

#[cfg(test)]
mod persistence_test;

#[cfg(test)]
mod quality_validator_tests;

#[cfg(test)]
mod renderer_tests;

pub use command_builder::{
    CommandBuilder,
    CommandConfig,
};
pub use creation_builder::{
    CreationBuilder,
    IssueSeverity,
    ValidationIssue,
    ValidationResult,
};
pub use edit::AssistantEditor;
pub use export_import::{
    ConflictStrategy,
    export_all_assistants,
    export_assistant,
    import_all_assistants,
    import_assistant,
};
pub use interactive::InteractivePromptBuilder;
pub use persistence::{
    delete_template,
    get_assistants_dir,
    list_templates,
    load_template,
    save_template,
};
pub use prompt_builder::PromptBuilder;
pub use template_manager::{
    DefaultTemplateManager,
    TemplateManager,
};
pub use types::*;

/// Main entry point for the prompt building system
pub struct PromptSystem {
    manager: Box<dyn TemplateManager>,
}

impl PromptSystem {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            manager: Box::new(DefaultTemplateManager::new().await?),
        })
    }

    pub async fn list_templates(&self) -> Result<Vec<TemplateInfo>> {
        self.manager.list_templates().await
    }

    pub async fn get_template(&self, id: &str) -> Result<PromptTemplate> {
        self.manager.get_template(id).await
    }

    pub async fn render_template(&self, template: &PromptTemplate, params: &HashMap<String, String>) -> Result<String> {
        self.manager.render_template(template, params).await
    }

    pub fn validate_prompt(&self, prompt: &str) -> QualityScore {
        self.manager.validate_quality(prompt)
    }

    pub async fn suggest_templates_for_use_case(&self, use_case: &str) -> Result<Vec<TemplateInfo>> {
        let all_templates = self.list_templates().await?;

        // Simple keyword matching for now
        let keywords = use_case.to_lowercase();
        let mut scored_templates: Vec<(TemplateInfo, f64)> = all_templates
            .into_iter()
            .map(|template| {
                let score = self.calculate_relevance_score(&template, &keywords);
                (template, score)
            })
            .collect();

        // Sort by relevance score
        scored_templates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Return top 3 suggestions
        Ok(scored_templates
            .into_iter()
            .take(3)
            .map(|(template, _)| template)
            .collect())
    }

    pub fn get_template_parameters(&self, template: &PromptTemplate) -> Vec<ParameterInfo> {
        template
            .parameters
            .iter()
            .map(|param| ParameterInfo {
                name: param.name.clone(),
                param_type: param.param_type.clone(),
                description: param.description.clone(),
                default_value: param.default_value.clone(),
                required: param.required,
            })
            .collect()
    }

    fn calculate_relevance_score(&self, template: &TemplateInfo, keywords: &str) -> f64 {
        let mut score = 0.0;

        // Check name and description
        if template.name.to_lowercase().contains(keywords) {
            score += 2.0;
        }
        if template.description.to_lowercase().contains(keywords) {
            score += 1.0;
        }

        // Boost score based on quality and usage
        score += template.estimated_quality * 0.2;
        score += (template.usage_stats.success_rate * 0.5);

        score
    }
}

#[derive(Debug, Clone)]
pub struct ParameterInfo {
    pub name: String,
    pub param_type: ParameterType,
    pub description: String,
    pub default_value: Option<String>,
    pub required: bool,
}
