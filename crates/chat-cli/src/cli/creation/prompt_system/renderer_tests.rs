#[cfg(test)]
mod renderer_tests {
    use std::collections::HashMap;
    use crate::cli::creation::prompt_system::template_manager::{SafeTemplateRenderer, TemplateRenderer};
    use crate::cli::creation::prompt_system::types::{
        PromptTemplate, TemplateCategory, DifficultyLevel, UsageStats, TemplateParameter, ParameterType
    };
    use chrono::Utc;

    fn create_test_template_with_params() -> PromptTemplate {
        PromptTemplate {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: "Test template".to_string(),
            version: 1,
            category: TemplateCategory::CodeReviewer,
            difficulty: DifficultyLevel::Beginner,
            tags: vec![],
            role: "You are an expert {{language}} developer.".to_string(),
            capabilities: vec!["Review {{language}} code".to_string()],
            constraints: vec!["Focus on {{focus_area}}".to_string()],
            context: Some("Reviewing {{language}} code".to_string()),
            parameters: vec![
                TemplateParameter {
                    name: "language".to_string(),
                    param_type: ParameterType::String { max_length: 50 },
                    description: "Programming language".to_string(),
                    default_value: Some("Rust".to_string()),
                    required: true,
                },
            ],
            examples: vec![],
            quality_indicators: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            usage_stats: UsageStats {
                success_rate: 0.0,
                avg_satisfaction: 0.0,
                usage_count: 0,
            },
        }
    }

    #[tokio::test]
    async fn test_renderer_substitutes_simple_params() {
        let renderer = SafeTemplateRenderer::new();
        let template = create_test_template_with_params();
        let mut params = HashMap::new();
        params.insert("language".to_string(), "Rust".to_string());
        
        let result = renderer.render(&template, &params).await.unwrap();
        
        assert!(result.contains("Rust"), "Result should contain 'Rust': {}", result);
        assert!(!result.contains("{{language}}"), "Result should not contain placeholder: {}", result);
    }

    #[tokio::test]
    async fn test_renderer_substitutes_multiple_occurrences() {
        let renderer = SafeTemplateRenderer::new();
        let template = create_test_template_with_params();
        let mut params = HashMap::new();
        params.insert("language".to_string(), "Python".to_string());
        
        let result = renderer.render(&template, &params).await.unwrap();
        
        // Should replace all occurrences
        assert!(result.contains("Python"));
        assert!(!result.contains("{{language}}"));
        assert_eq!(result.matches("Python").count(), 3, "Should replace all 3 occurrences");
    }

    #[tokio::test]
    async fn test_renderer_preserves_structure() {
        let renderer = SafeTemplateRenderer::new();
        let template = create_test_template_with_params();
        let mut params = HashMap::new();
        params.insert("language".to_string(), "Java".to_string());
        
        let result = renderer.render(&template, &params).await.unwrap();
        
        // Should preserve the template structure
        assert!(result.contains("You are an expert"));
        assert!(result.contains("developer"));
    }
}
