//! Template management with loading, caching, and validation

use super::*;
use eyre::{eyre, Result};
use std::collections::HashMap;
use tokio::fs;

/// Manages prompt templates with caching and validation
pub struct TemplateManager {
    base_path: PathBuf,
    cache: HashMap<String, PromptTemplate>,
    cache_valid: bool,
}

impl TemplateManager {
    /// Create new template manager
    pub fn new(base_path: &Path) -> Result<Self> {
        Ok(Self {
            base_path: base_path.to_path_buf(),
            cache: HashMap::new(),
            cache_valid: false,
        })
    }
    
    /// Load a specific template by name
    pub async fn load_template(&mut self, name: &str) -> Result<PromptTemplate> {
        // Check cache first
        if self.cache_valid {
            if let Some(template) = self.cache.get(name) {
                return Ok(template.clone());
            }
        }
        
        // Load from file
        let template_path = self.base_path.join("templates").join(format!("{}.json", name));
        
        if !template_path.exists() {
            return Err(eyre!("Template '{}' not found at {:?}", name, template_path));
        }
        
        let content = fs::read_to_string(&template_path).await?;
        let template: PromptTemplate = serde_json::from_str(&content)
            .map_err(|e| eyre!("Failed to parse template '{}': {}", name, e))?;
        
        // Validate template
        let validation = template.validate()?;
        if !validation.is_valid {
            let errors: Vec<_> = validation.issues.iter()
                .filter(|i| i.severity == IssueSeverity::Error)
                .map(|i| &i.message)
                .collect();
            return Err(eyre!("Template '{}' validation failed: {:?}", name, errors));
        }
        
        // Cache the template
        self.cache.insert(name.to_string(), template.clone());
        
        Ok(template)
    }
    
    /// List all available templates
    pub async fn list_templates(&mut self) -> Result<Vec<TemplateInfo>> {
        let templates_dir = self.base_path.join("templates");
        
        if !templates_dir.exists() {
            return Ok(Vec::new());
        }
        
        let mut templates = Vec::new();
        let mut entries = fs::read_dir(&templates_dir).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    // Try to load template for metadata
                    match self.load_template(stem).await {
                        Ok(template) => {
                            templates.push(TemplateInfo {
                                name: stem.to_string(),
                                display_name: template.name,
                                description: template.description,
                                category: template.metadata.category,
                                difficulty: template.metadata.difficulty,
                                success_rate: template.metadata.usage_stats.success_rate,
                            });
                        }
                        Err(_) => {
                            // Skip invalid templates but don't fail entirely
                            continue;
                        }
                    }
                }
            }
        }
        
        // Sort by success rate and category
        templates.sort_by(|a, b| {
            b.success_rate.partial_cmp(&a.success_rate)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.category.to_string().cmp(&b.category.to_string()))
        });
        
        Ok(templates)
    }
    
    /// Create built-in templates if they don't exist
    pub async fn ensure_builtin_templates(&mut self) -> Result<()> {
        let templates_dir = self.base_path.join("templates");
        fs::create_dir_all(&templates_dir).await?;
        
        // Create core templates
        self.create_code_reviewer_template().await?;
        self.create_documentation_writer_template().await?;
        self.create_domain_expert_template().await?;
        
        Ok(())
    }
    
    async fn create_code_reviewer_template(&self) -> Result<()> {
        let template = PromptTemplate {
            name: "Code Reviewer".to_string(),
            description: "Reviews code for security, performance, and best practices".to_string(),
            role: "You are an expert code reviewer with 10+ years of experience in software development. You specialize in identifying security vulnerabilities, performance issues, and adherence to best practices.".to_string(),
            capabilities: vec![
                "Security vulnerability analysis".to_string(),
                "Performance optimization suggestions".to_string(),
                "Code quality assessment".to_string(),
                "Best practices enforcement".to_string(),
            ],
            constraints: vec![
                "Always explain your reasoning".to_string(),
                "Provide specific examples".to_string(),
                "Be constructive and helpful".to_string(),
                "Focus on the most critical issues first".to_string(),
            ],
            example_conversation: Some(ExampleConversation {
                input: "Review this function: def login(user, password): return user == 'admin'".to_string(),
                output: "This function has a critical security vulnerability. It uses plain text comparison and hardcoded credentials. Here's what's wrong and how to fix it: 1) Password should be hashed and verified securely, 2) User authentication should use a proper database lookup, 3) Consider implementing rate limiting to prevent brute force attacks.".to_string(),
            }),
            metadata: TemplateMetadata {
                category: TemplateCategory::CodeReview,
                difficulty: DifficultyLevel::Beginner,
                tags: vec!["security".to_string(), "performance".to_string(), "best-practices".to_string()],
                usage_stats: UsageStats {
                    success_rate: 0.87,
                    avg_satisfaction: 4.3,
                    usage_count: 245,
                },
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        };
        
        self.save_template("code-reviewer", &template).await
    }
    
    async fn create_documentation_writer_template(&self) -> Result<()> {
        let template = PromptTemplate {
            name: "Documentation Writer".to_string(),
            description: "Creates clear, comprehensive technical documentation".to_string(),
            role: "You are a technical writer who specializes in creating clear, comprehensive documentation for software projects. You excel at explaining complex concepts in simple terms.".to_string(),
            capabilities: vec![
                "API documentation".to_string(),
                "User guides and tutorials".to_string(),
                "Code comments and inline documentation".to_string(),
                "README files and project documentation".to_string(),
            ],
            constraints: vec![
                "Use simple, clear language".to_string(),
                "Include practical examples".to_string(),
                "Structure information logically".to_string(),
                "Consider the target audience".to_string(),
            ],
            example_conversation: Some(ExampleConversation {
                input: "Document this API endpoint: POST /users - creates a new user".to_string(),
                output: "## Create User\n\n**Endpoint:** `POST /users`\n\n**Description:** Creates a new user account in the system.\n\n**Request Body:**\n```json\n{\n  \"name\": \"string\",\n  \"email\": \"string\"\n}\n```\n\n**Response:** Returns the created user with ID\n\n**Example:**\n```bash\ncurl -X POST /users -d '{\"name\":\"John\",\"email\":\"john@example.com\"}'\n```".to_string(),
            }),
            metadata: TemplateMetadata {
                category: TemplateCategory::Documentation,
                difficulty: DifficultyLevel::Beginner,
                tags: vec!["documentation".to_string(), "api".to_string(), "writing".to_string()],
                usage_stats: UsageStats {
                    success_rate: 0.91,
                    avg_satisfaction: 4.5,
                    usage_count: 189,
                },
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        };
        
        self.save_template("documentation-writer", &template).await
    }
    
    async fn create_domain_expert_template(&self) -> Result<()> {
        let template = PromptTemplate {
            name: "Domain Expert".to_string(),
            description: "Specialized knowledge assistant for specific domains".to_string(),
            role: "You are a domain expert with deep knowledge and experience in your field. You provide accurate, well-reasoned answers and guidance based on established best practices and current knowledge.".to_string(),
            capabilities: vec![
                "Answer domain-specific questions".to_string(),
                "Provide expert guidance and recommendations".to_string(),
                "Explain complex concepts clearly".to_string(),
                "Share relevant best practices".to_string(),
            ],
            constraints: vec![
                "Cite sources when possible".to_string(),
                "Admit uncertainty when appropriate".to_string(),
                "Focus on accuracy over speed".to_string(),
                "Provide context for recommendations".to_string(),
            ],
            example_conversation: Some(ExampleConversation {
                input: "What's the best approach for handling database migrations in a production environment?".to_string(),
                output: "For production database migrations, I recommend a blue-green deployment strategy: 1) Test migrations thoroughly in staging, 2) Use backward-compatible changes when possible, 3) Plan rollback procedures, 4) Monitor performance during migration, 5) Consider maintenance windows for breaking changes. Tools like Flyway or Liquibase can help manage this process safely.".to_string(),
            }),
            metadata: TemplateMetadata {
                category: TemplateCategory::DomainExpert,
                difficulty: DifficultyLevel::Intermediate,
                tags: vec!["expert".to_string(), "domain-knowledge".to_string(), "guidance".to_string()],
                usage_stats: UsageStats {
                    success_rate: 0.83,
                    avg_satisfaction: 4.1,
                    usage_count: 156,
                },
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        };
        
        self.save_template("domain-expert", &template).await
    }
    
    pub async fn save_template(&self, name: &str, template: &PromptTemplate) -> Result<()> {
        let templates_dir = self.base_path.join("templates");
        fs::create_dir_all(&templates_dir).await?;
        
        let template_path = templates_dir.join(format!("{}.json", name));
        let content = serde_json::to_string_pretty(template)?;
        fs::write(template_path, content).await?;
        
        Ok(())
    }
    
    /// Invalidate cache (call when templates are modified externally)
    pub fn invalidate_cache(&mut self) {
        self.cache.clear();
        self.cache_valid = false;
    }
}

/// Template information for listing
#[derive(Debug, Clone)]
pub struct TemplateInfo {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub category: TemplateCategory,
    pub difficulty: DifficultyLevel,
    pub success_rate: f64,
}

impl TemplateCategory {
    fn to_string(&self) -> String {
        match self {
            TemplateCategory::CodeReview => "Code Review".to_string(),
            TemplateCategory::Documentation => "Documentation".to_string(),
            TemplateCategory::DomainExpert => "Domain Expert".to_string(),
            TemplateCategory::GeneralAssistant => "General Assistant".to_string(),
            TemplateCategory::Custom => "Custom".to_string(),
        }
    }
}
