use eyre::Result;
use std::collections::HashMap;
use async_trait::async_trait;
use chrono::Utc;

use crate::cli::creation::prompt_system::types::*;
use crate::cli::creation::prompt_system::template_manager::TemplateStorage;

pub struct HybridTemplateStorage {
    embedded_templates: HashMap<String, PromptTemplate>,
}

impl HybridTemplateStorage {
    pub async fn new() -> Result<Self> {
        let mut storage = Self {
            embedded_templates: HashMap::new(),
        };
        
        storage.load_embedded_templates();
        Ok(storage)
    }

    fn load_embedded_templates(&mut self) {
        // Load basic embedded templates
        self.embedded_templates.insert(
            "code_reviewer".to_string(),
            self.create_code_reviewer_template()
        );
        
        self.embedded_templates.insert(
            "documentation_writer".to_string(),
            self.create_documentation_writer_template()
        );
        
        self.embedded_templates.insert(
            "conversation_assistant".to_string(),
            self.create_conversation_assistant_template()
        );
    }

    fn create_code_reviewer_template(&self) -> PromptTemplate {
        PromptTemplate {
            id: "code_reviewer".to_string(),
            name: "Code Reviewer".to_string(),
            description: "Expert code reviewer providing detailed feedback".to_string(),
            version: 1,
            category: TemplateCategory::CodeReviewer,
            difficulty: DifficultyLevel::Intermediate,
            tags: vec!["code".to_string(), "review".to_string(), "quality".to_string()],
            role: "You are an expert code reviewer with deep knowledge of software engineering best practices.".to_string(),
            capabilities: vec![
                "Analyze code for bugs and security vulnerabilities".to_string(),
                "Suggest performance improvements".to_string(),
                "Ensure code follows best practices".to_string(),
                "Provide constructive feedback".to_string(),
            ],
            constraints: vec![
                "Focus on actionable feedback".to_string(),
                "Explain the reasoning behind suggestions".to_string(),
                "Be respectful and constructive".to_string(),
            ],
            context: Some("Review the provided code thoroughly and provide detailed feedback.".to_string()),
            parameters: vec![
                TemplateParameter {
                    name: "language".to_string(),
                    param_type: ParameterType::Enum { 
                        options: vec!["rust".to_string(), "python".to_string(), "javascript".to_string(), "java".to_string()] 
                    },
                    description: "Programming language of the code".to_string(),
                    default_value: Some("rust".to_string()),
                    required: true,
                },
                TemplateParameter {
                    name: "focus_area".to_string(),
                    param_type: ParameterType::Enum { 
                        options: vec!["security".to_string(), "performance".to_string(), "maintainability".to_string(), "all".to_string()] 
                    },
                    description: "Primary focus area for the review".to_string(),
                    default_value: Some("all".to_string()),
                    required: false,
                },
            ],
            examples: vec![
                ExampleConversation {
                    input: "Please review this Rust function for potential issues.".to_string(),
                    output: "I'll analyze your Rust function focusing on safety, performance, and idiomatic patterns.".to_string(),
                }
            ],
            quality_indicators: vec![
                "Identifies specific issues".to_string(),
                "Provides actionable suggestions".to_string(),
                "Explains reasoning clearly".to_string(),
            ],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            usage_stats: UsageStats {
                success_rate: 0.92,
                avg_satisfaction: 4.3,
                usage_count: 156,
            },
        }
    }

    fn create_documentation_writer_template(&self) -> PromptTemplate {
        PromptTemplate {
            id: "documentation_writer".to_string(),
            name: "Documentation Writer".to_string(),
            description: "Technical writer specializing in clear, comprehensive documentation".to_string(),
            version: 1,
            category: TemplateCategory::DocumentationWriter,
            difficulty: DifficultyLevel::Beginner,
            tags: vec!["documentation".to_string(), "writing".to_string(), "technical".to_string()],
            role: "You are a technical writer who creates clear, comprehensive documentation.".to_string(),
            capabilities: vec![
                "Write clear API documentation".to_string(),
                "Create user guides and tutorials".to_string(),
                "Explain complex concepts simply".to_string(),
                "Structure information logically".to_string(),
            ],
            constraints: vec![
                "Use clear, concise language".to_string(),
                "Include practical examples".to_string(),
                "Structure content with headers and lists".to_string(),
            ],
            context: Some("Create documentation that helps users understand and use the subject effectively.".to_string()),
            parameters: vec![
                TemplateParameter {
                    name: "doc_type".to_string(),
                    param_type: ParameterType::Enum { 
                        options: vec!["api".to_string(), "tutorial".to_string(), "guide".to_string(), "reference".to_string()] 
                    },
                    description: "Type of documentation to create".to_string(),
                    default_value: Some("guide".to_string()),
                    required: true,
                },
                TemplateParameter {
                    name: "audience".to_string(),
                    param_type: ParameterType::Enum { 
                        options: vec!["beginner".to_string(), "intermediate".to_string(), "expert".to_string()] 
                    },
                    description: "Target audience level".to_string(),
                    default_value: Some("intermediate".to_string()),
                    required: false,
                },
            ],
            examples: vec![
                ExampleConversation {
                    input: "Create documentation for this API endpoint.".to_string(),
                    output: "I'll create comprehensive API documentation with examples and usage patterns.".to_string(),
                }
            ],
            quality_indicators: vec![
                "Clear structure and organization".to_string(),
                "Practical examples included".to_string(),
                "Appropriate for target audience".to_string(),
            ],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            usage_stats: UsageStats {
                success_rate: 0.88,
                avg_satisfaction: 4.1,
                usage_count: 89,
            },
        }
    }

    fn create_conversation_assistant_template(&self) -> PromptTemplate {
        PromptTemplate {
            id: "conversation_assistant".to_string(),
            name: "Conversation Assistant".to_string(),
            description: "Helpful assistant for general conversations and questions".to_string(),
            version: 1,
            category: TemplateCategory::ConversationAssistant,
            difficulty: DifficultyLevel::Beginner,
            tags: vec!["conversation".to_string(), "assistant".to_string(), "general".to_string()],
            role: "You are a helpful, knowledgeable assistant ready to help with various tasks and questions.".to_string(),
            capabilities: vec![
                "Answer questions on various topics".to_string(),
                "Provide explanations and guidance".to_string(),
                "Help with problem-solving".to_string(),
                "Engage in natural conversation".to_string(),
            ],
            constraints: vec![
                "Be helpful and accurate".to_string(),
                "Admit when uncertain".to_string(),
                "Ask clarifying questions when needed".to_string(),
            ],
            context: Some("Assist the user with their questions and tasks in a helpful, friendly manner.".to_string()),
            parameters: vec![
                TemplateParameter {
                    name: "tone".to_string(),
                    param_type: ParameterType::Enum { 
                        options: vec!["formal".to_string(), "casual".to_string(), "friendly".to_string(), "professional".to_string()] 
                    },
                    description: "Tone of the conversation".to_string(),
                    default_value: Some("friendly".to_string()),
                    required: false,
                },
            ],
            examples: vec![
                ExampleConversation {
                    input: "Can you help me understand this concept?".to_string(),
                    output: "I'd be happy to help explain that concept! Let me break it down for you.".to_string(),
                }
            ],
            quality_indicators: vec![
                "Provides helpful responses".to_string(),
                "Maintains appropriate tone".to_string(),
                "Asks clarifying questions when needed".to_string(),
            ],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            usage_stats: UsageStats {
                success_rate: 0.85,
                avg_satisfaction: 4.0,
                usage_count: 234,
            },
        }
    }
}

#[async_trait]
impl TemplateStorage for HybridTemplateStorage {
    async fn load_template(&self, id: &str) -> Result<PromptTemplate> {
        // Try embedded templates first
        if let Some(template) = self.embedded_templates.get(id) {
            return Ok(template.clone());
        }
        
        // TODO: Try file-based storage
        // For now, return not found error
        Err(TemplateError::NotFound { id: id.to_string() }.into())
    }

    async fn list_all_templates(&self) -> Result<Vec<PromptTemplate>> {
        let mut templates = Vec::new();
        
        // Add embedded templates
        for template in self.embedded_templates.values() {
            templates.push(template.clone());
        }
        
        // TODO: Add file-based templates
        
        Ok(templates)
    }
}
