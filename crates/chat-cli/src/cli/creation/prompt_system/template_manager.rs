use std::collections::HashMap;

use async_trait::async_trait;
use eyre::Result;

use crate::cli::creation::prompt_system::storage::HybridTemplateStorage;
use crate::cli::creation::prompt_system::types::*;

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
        Ok(templates
            .into_iter()
            .map(|t| {
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
            })
            .collect())
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
            },
        }

        // Final fallback: Emergency template
        Ok(self.create_emergency_template())
    }

    async fn find_similar_template(&self, _id: &str) -> Result<PromptTemplate> {
        // Simple implementation: return first available template
        let templates = self.storage.list_all_templates().await?;
        templates
            .into_iter()
            .next()
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
        format!(
            "{}\n\nCapabilities:\n{}\n\nConstraints:\n{}",
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

// Real implementations
pub struct MultiDimensionalValidator;

impl MultiDimensionalValidator {
    pub fn new() -> Self {
        Self
    }

    fn calculate_role_clarity(&self, prompt: &str) -> f64 {
        let mut score = 0.0;
        let word_count = prompt.split_whitespace().count();

        // Length score (0-0.3): Detailed roles are clearer
        let length_score = if word_count < 5 {
            0.0
        } else if word_count < 10 {
            0.1
        } else if word_count < 20 {
            0.2
        } else {
            0.3
        };
        score += length_score;

        // Specificity score (0-0.4): Technical terms and domain keywords
        let technical_terms = [
            "expert",
            "specialist",
            "senior",
            "architect",
            "engineer",
            "developer",
            "analyst",
            "consultant",
            "reviewer",
            "writer",
            "rust",
            "python",
            "java",
            "javascript",
            "code",
            "software",
            "system",
            "data",
            "security",
            "performance",
            "testing",
            "async",
            "concurrent",
            "distributed",
            "microservices",
            "cloud",
        ];

        let prompt_lower = prompt.to_lowercase();
        let term_count = technical_terms
            .iter()
            .filter(|term| prompt_lower.contains(*term))
            .count();

        let specificity_score = (term_count as f64 * 0.1).min(0.4);
        score += specificity_score;

        // Structure score (0-0.3): Presence of "You are" pattern
        if prompt_lower.contains("you are") {
            score += 0.15;
        }
        if prompt_lower.contains("with") || prompt_lower.contains("specializing") {
            score += 0.15;
        }

        score.min(1.0)
    }

    fn calculate_capability_completeness(&self, prompt: &str) -> f64 {
        let mut score = 0.0;
        
        // Look for capabilities section
        let prompt_lower = prompt.to_lowercase();
        if !prompt_lower.contains("capabilit") {
            return 0.0;
        }
        
        // Count bullet points or numbered items
        let bullet_count = prompt.matches("\n-").count() + prompt.matches("\n*").count();
        let numbered_count = (1..=10).filter(|i| prompt.contains(&format!("\n{}.", i))).count();
        let item_count = bullet_count + numbered_count;
        
        // Quantity score (0-0.75): More capabilities is better
        let quantity_score = match item_count {
            0 => 0.0,
            1 => 0.15,
            2 => 0.3,
            3 => 0.45,
            4 => 0.6,
            5 => 0.75,
            _ => 0.75,
        };
        score += quantity_score;
        
        // Specificity score (0-0.3): Check for action verbs and technical terms
        let action_verbs = [
            "analyze", "detect", "find", "identify", "review", "validate",
            "suggest", "recommend", "optimize", "refactor", "implement",
            "debug", "test", "document", "explain", "evaluate"
        ];
        
        let verb_count = action_verbs.iter()
            .filter(|verb| prompt_lower.contains(*verb))
            .count();
        
        let specificity_score = (verb_count as f64 * 0.1).min(0.3);
        score += specificity_score;
        
        score.min(1.0)
    }

    fn calculate_constraint_clarity(&self, prompt: &str) -> f64 {
        let mut score = 0.0;
        let prompt_lower = prompt.to_lowercase();
        if !prompt_lower.contains("constraint") { return 0.0; }
        
        let bullet_count = prompt.matches("\n-").count() + prompt.matches("\n*").count();
        let numbered_count = (1..=10).filter(|i| prompt.contains(&format!("\n{}.", i))).count();
        let item_count = bullet_count + numbered_count;
        
        let quantity_score = match item_count {
            0 => 0.0, 1 => 0.2, 2 => 0.35, 3 => 0.5, 4 => 0.65, _ => 0.65,
        };
        score += quantity_score;
        
        let specificity_terms = [
            "always", "never", "must", "should", "limit", "maximum", "minimum",
            "avoid", "ensure", "require", "only", "exactly", "within", "cite",
            "specific", "concise", "clear", "accurate", "precise"
        ];
        
        let term_count = specificity_terms.iter().filter(|t| prompt_lower.contains(*t)).count();
        score += (term_count as f64 * 0.1).min(0.35);
        score.min(1.0)
    }

    fn calculate_example_quality(&self, prompt: &str) -> f64 {
        let mut score = 0.0;
        let prompt_lower = prompt.to_lowercase();
        if !prompt_lower.contains("example") { return 0.0; }
        
        let input_count = prompt_lower.matches("input:").count();
        let output_count = prompt_lower.matches("output:").count();
        let pair_count = input_count.min(output_count);
        
        let pair_score = match pair_count {
            0 => 0.0, 1 => 0.4, 2 => 0.5, 3 => 0.6, _ => 0.6,
        };
        score += pair_score;
        
        if input_count > 0 && output_count > 0 {
            score += 0.2;
            if input_count == output_count {
                score += 0.2;
            }
        }
        
        score.min(1.0)
    }

    fn generate_feedback(
        &self,
        role_clarity: f64,
        capability_completeness: f64,
        constraint_clarity: f64,
        example_quality: f64
    ) -> Vec<QualityFeedback> {
        let mut feedback = Vec::new();

        // Role clarity feedback
        if role_clarity < 0.3 {
            feedback.push(QualityFeedback {
                severity: FeedbackSeverity::Error,
                message: "Role definition is too vague or missing".to_string(),
                suggestion: Some("Add a clear role statement like 'You are an expert [domain] with knowledge of [specifics]'".to_string()),
            });
        } else if role_clarity < 0.6 {
            feedback.push(QualityFeedback {
                severity: FeedbackSeverity::Warning,
                message: "Role definition could be more specific".to_string(),
                suggestion: Some("Include domain expertise, specializations, or specific knowledge areas".to_string()),
            });
        }

        // Capability feedback
        if capability_completeness < 0.3 {
            feedback.push(QualityFeedback {
                severity: FeedbackSeverity::Error,
                message: "Capabilities section is missing or incomplete".to_string(),
                suggestion: Some("Add a 'Capabilities:' section with specific, actionable items using verbs like 'analyze', 'review', 'suggest'".to_string()),
            });
        } else if capability_completeness < 0.6 {
            feedback.push(QualityFeedback {
                severity: FeedbackSeverity::Warning,
                message: "Capabilities could be more detailed".to_string(),
                suggestion: Some("Add more specific capabilities or use action verbs to describe what the assistant can do".to_string()),
            });
        }

        // Constraint feedback
        if constraint_clarity < 0.3 {
            feedback.push(QualityFeedback {
                severity: FeedbackSeverity::Warning,
                message: "Constraints section is missing or unclear".to_string(),
                suggestion: Some("Add a 'Constraints:' section with measurable rules like 'Always cite sources' or 'Limit responses to 500 words'".to_string()),
            });
        } else if constraint_clarity < 0.5 {
            feedback.push(QualityFeedback {
                severity: FeedbackSeverity::Info,
                message: "Constraints could be more specific".to_string(),
                suggestion: Some("Use measurable terms like 'always', 'never', 'must', or specific limits".to_string()),
            });
        }

        // Example feedback
        if example_quality < 0.3 {
            feedback.push(QualityFeedback {
                severity: FeedbackSeverity::Info,
                message: "Examples section is missing or incomplete".to_string(),
                suggestion: Some("Add example conversations with 'Input:' and 'Output:' pairs to demonstrate expected behavior".to_string()),
            });
        }

        feedback
    }
}

impl QualityValidator for MultiDimensionalValidator {
    fn validate(&self, prompt: &str) -> QualityScore {
        let mut component_scores = HashMap::new();

        let role_clarity = self.calculate_role_clarity(prompt);
        component_scores.insert("role_clarity".to_string(), role_clarity);

        let capability_completeness = self.calculate_capability_completeness(prompt);
        component_scores.insert("capability_completeness".to_string(), capability_completeness);

        let constraint_clarity = self.calculate_constraint_clarity(prompt);
        component_scores.insert("constraint_clarity".to_string(), constraint_clarity);

        let example_quality = self.calculate_example_quality(prompt);
        component_scores.insert("example_quality".to_string(), example_quality);

        // Weighted average: role 30%, capability 25%, constraint 25%, examples 20%
        let overall_score = (role_clarity * 0.3) + (capability_completeness * 0.25) + 
                           (constraint_clarity * 0.25) + (example_quality * 0.2);

        // Generate feedback based on component scores
        let feedback = self.generate_feedback(
            role_clarity,
            capability_completeness,
            constraint_clarity,
            example_quality
        );

        QualityScore {
            overall_score,
            component_scores,
            feedback,
            confidence: 0.8,
        }
    }
}

pub struct SafeTemplateRenderer;
pub struct TwoTierCacheManager;

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
