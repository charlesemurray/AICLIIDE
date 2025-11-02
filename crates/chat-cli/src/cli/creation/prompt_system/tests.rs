#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::cli::creation::prompt_system::{
        DifficultyLevel, ParameterType, PromptSystem, PromptTemplate, TemplateCategory, TemplateError,
        TemplateParameter, UsageStats,
    };

    #[tokio::test]
    async fn test_prompt_system_creation() {
        let system = PromptSystem::new().await;
        assert!(system.is_ok());
    }

    #[tokio::test]
    async fn test_list_templates_returns_embedded_templates() {
        let system = PromptSystem::new().await.unwrap();
        let templates = system.list_templates().await.unwrap();
        assert_eq!(templates.len(), 3);

        let template_ids: Vec<&str> = templates.iter().map(|t| t.id.as_str()).collect();
        assert!(template_ids.contains(&"code_reviewer"));
        assert!(template_ids.contains(&"documentation_writer"));
        assert!(template_ids.contains(&"conversation_assistant"));
    }

    #[tokio::test]
    async fn test_get_nonexistent_template_uses_fallback() {
        let system = PromptSystem::new().await.unwrap();
        let result = system.get_template("nonexistent").await;

        // Should get first available template as fallback
        assert!(result.is_ok());
        let template = result.unwrap();
        // Should be one of the embedded templates
        assert!(["code_reviewer", "documentation_writer", "conversation_assistant"].contains(&template.id.as_str()));
    }

    #[tokio::test]
    async fn test_quality_validation_returns_score() {
        let system = PromptSystem::new().await.unwrap();
        let score = system.validate_prompt("You are a helpful assistant.");

        assert!(score.overall_score >= 0.0);
        assert!(score.overall_score <= 5.0);
        assert!(score.confidence >= 0.0);
        assert!(score.confidence <= 1.0);
    }

    #[tokio::test]
    async fn test_template_rendering_basic() {
        let system = PromptSystem::new().await.unwrap();
        let template = system.get_template("conversation_assistant").await.unwrap();
        let params = HashMap::new();

        let rendered = system.render_template(&template, &params).await;
        assert!(rendered.is_ok());
        assert!(!rendered.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_suggest_templates_for_use_case() {
        let system = PromptSystem::new().await.unwrap();
        let suggestions = system.suggest_templates_for_use_case("code review").await.unwrap();

        // Should return suggestions from embedded templates
        assert!(suggestions.len() > 0);
        assert!(suggestions.len() <= 3); // Max 3 suggestions

        // Should include code reviewer template for "code review" use case
        let has_code_reviewer = suggestions.iter().any(|t| t.id == "code_reviewer");
        assert!(has_code_reviewer);
    }

    #[tokio::test]
    async fn test_template_parameter_extraction() {
        let template = create_test_template();
        let system = PromptSystem::new().await.unwrap();
        let params = system.get_template_parameters(&template);

        assert_eq!(params.len(), 1);
        assert_eq!(params[0].name, "language");
        assert_eq!(params[0].required, true);
    }

    #[test]
    fn test_error_types_are_properly_defined() {
        let error = TemplateError::NotFound { id: "test".to_string() };
        assert!(error.to_string().contains("test"));

        let error = TemplateError::ValidationFailed {
            reason: "bad template".to_string(),
        };
        assert!(error.to_string().contains("bad template"));
    }

    // Helper function to create test template
    fn create_test_template() -> PromptTemplate {
        PromptTemplate {
            id: "test".to_string(),
            name: "Test Template".to_string(),
            description: "A test template".to_string(),
            version: 1,
            category: TemplateCategory::ConversationAssistant,
            difficulty: DifficultyLevel::Beginner,
            tags: vec!["test".to_string()],
            role: "You are a test assistant.".to_string(),
            capabilities: vec!["Testing".to_string()],
            constraints: vec!["Be helpful".to_string()],
            context: None,
            parameters: vec![TemplateParameter {
                name: "language".to_string(),
                param_type: ParameterType::String { max_length: 50 },
                description: "Programming language".to_string(),
                default_value: Some("Python".to_string()),
                required: true,
            }],
            examples: vec![],
            quality_indicators: vec![],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            usage_stats: UsageStats {
                success_rate: 0.8,
                avg_satisfaction: 4.0,
                usage_count: 10,
            },
        }
    }
}
