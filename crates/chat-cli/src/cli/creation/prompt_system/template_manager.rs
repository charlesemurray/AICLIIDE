use eyre::Result;
use std::collections::HashMap;
use async_trait::async_trait;

use crate::cli::creation::prompt_system::types::*;
use crate::cli::creation::prompt_system::storage::HybridTemplateStorage;

#[async_trait]
pub trait TemplateManager: Send + Sync {
    async fn list_templates(&self) -> Result<Vec<TemplateInfo>>;
    async fn get_template(&self, id: &str) -> Result<PromptTemplate>;
    async fn render_template(&self, template: &PromptTemplate, params: &HashMap<String, String>) -> Result<String>;
    fn validate_quality(&self, prompt: &str) -> QualityScore;
}

pub struct DefaultTemplateManager {
    storage: Box<dyn TemplateStorage>,
    validator: Box<dyn QualityValidator>,
    renderer: Box<dyn TemplateRenderer>,
    cache: Box<dyn CacheManager>,
}

impl DefaultTemplateManager {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            storage: Box::new(HybridTemplateStorage::new().await?),
            validator: Box::new(MultiDimensionalValidator::new()),
            renderer: Box::new(SafeTemplateRenderer::new()),
            cache: Box::new(TwoTierCacheManager::new()),
        })
    }
}

#[async_trait]
impl TemplateManager for DefaultTemplateManager {
    async fn list_templates(&self) -> Result<Vec<TemplateInfo>> {
        let templates = self.storage.list_all_templates().await?;
        Ok(templates.into_iter().map(|t| {
            let rendered = self.render_template_internal(&t);
            let quality = self.validate_quality(&rendered).overall_score;
            TemplateInfo {
                id: t.id,
                name: t.name,
                description: t.description,
                category: t.category,
                difficulty: t.difficulty,
                estimated_quality: quality,
                usage_stats: t.usage_stats,
            }
        }).collect())
    }

    async fn get_template(&self, id: &str) -> Result<PromptTemplate> {
        // Check cache first
        if let Some(template) = self.cache.get(id).await? {
            return Ok(template);
        }

        // Load from storage with fallback
        let template = self.load_with_fallback(id).await?;
        
        // Cache for future use
        self.cache.put(id, &template).await?;
        
        Ok(template)
    }

    async fn render_template(&self, template: &PromptTemplate, params: &HashMap<String, String>) -> Result<String> {
        self.renderer.render(template, params).await
    }

    fn validate_quality(&self, prompt: &str) -> QualityScore {
        self.validator.validate(prompt)
    }
}

impl DefaultTemplateManager {
    async fn load_with_fallback(&self, id: &str) -> Result<PromptTemplate> {
        // Primary: Load from storage
        match self.storage.load_template(id).await {
            Ok(template) => return Ok(template),
            Err(e) => {
                // Check if it's a "not found" error by examining the error message
                if e.to_string().contains("not found") || e.to_string().contains("NotFound") {
                    // Fallback 1: Try similar template
                    if let Ok(similar) = self.find_similar_template(id).await {
                        return Ok(similar);
                    }
                }
                // Continue to final fallback for any error
            }
        }
        
        // Final fallback: Emergency template
        Ok(self.create_emergency_template())
    }

    async fn find_similar_template(&self, _id: &str) -> Result<PromptTemplate> {
        // Simple implementation: return first available template
        let templates = self.storage.list_all_templates().await?;
        templates.into_iter().next()
            .ok_or_else(|| TemplateError::NotFound { id: "any".to_string() }.into())
    }

    fn create_emergency_template(&self) -> PromptTemplate {
        PromptTemplate {
            id: "emergency".to_string(),
            name: "Basic Assistant".to_string(),
            description: "Emergency fallback template".to_string(),
            version: 1,
            category: TemplateCategory::ConversationAssistant,
            difficulty: DifficultyLevel::Beginner,
            tags: vec!["fallback".to_string()],
            role: "You are a helpful assistant.".to_string(),
            capabilities: vec!["Answer questions".to_string()],
            constraints: vec!["Be helpful and accurate".to_string()],
            context: None,
            parameters: vec![],
            examples: vec![],
            quality_indicators: vec![],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            usage_stats: UsageStats {
                success_rate: 0.5,
                avg_satisfaction: 3.0,
                usage_count: 0,
            },
        }
    }

    fn render_template_internal(&self, template: &PromptTemplate) -> String {
        // Simple rendering for quality estimation
        format!("{}\n\nCapabilities:\n{}\n\nConstraints:\n{}", 
            template.role,
            template.capabilities.join("\n- "),
            template.constraints.join("\n- ")
        )
    }
}

// Trait definitions for components
#[async_trait]
pub trait TemplateStorage: Send + Sync {
    async fn load_template(&self, id: &str) -> Result<PromptTemplate>;
    async fn list_all_templates(&self) -> Result<Vec<PromptTemplate>>;
}

pub trait QualityValidator: Send + Sync {
    fn validate(&self, prompt: &str) -> QualityScore;
}

#[async_trait]
pub trait TemplateRenderer: Send + Sync {
    async fn render(&self, template: &PromptTemplate, params: &HashMap<String, String>) -> Result<String>;
}

#[async_trait]
pub trait CacheManager: Send + Sync {
    async fn get(&self, id: &str) -> Result<Option<PromptTemplate>>;
    async fn put(&self, id: &str, template: &PromptTemplate) -> Result<()>;
}

// Placeholder implementations (to be implemented in subsequent steps)
pub struct MultiDimensionalValidator;
pub struct SafeTemplateRenderer;
pub struct TwoTierCacheManager;

impl MultiDimensionalValidator {
    pub fn new() -> Self {
        Self
    }
}

impl SafeTemplateRenderer {
    pub fn new() -> Self {
        Self
    }
}

impl TwoTierCacheManager {
    pub fn new() -> Self {
        Self
    }
}

// Temporary implementations - will be replaced with real implementations
impl QualityValidator for MultiDimensionalValidator {
    fn validate(&self, _prompt: &str) -> QualityScore {
        QualityScore {
            overall_score: 3.5,
            component_scores: HashMap::new(),
            feedback: vec![],
            confidence: 0.8,
        }
    }
}

#[async_trait]
impl TemplateRenderer for SafeTemplateRenderer {
    async fn render(&self, template: &PromptTemplate, _params: &HashMap<String, String>) -> Result<String> {
        Ok(template.role.clone())
    }
}

#[async_trait]
impl CacheManager for TwoTierCacheManager {
    async fn get(&self, _id: &str) -> Result<Option<PromptTemplate>> {
        Ok(None)
    }

    async fn put(&self, _id: &str, _template: &PromptTemplate) -> Result<()> {
        Ok(())
    }
}
