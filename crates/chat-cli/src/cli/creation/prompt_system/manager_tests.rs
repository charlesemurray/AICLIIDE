#[cfg(test)]
mod manager_tests {
    use std::collections::HashMap;

    use super::*;
    use crate::cli::creation::prompt_system::template_manager::{DefaultTemplateManager, TemplateManager};

    #[tokio::test]
    async fn test_manager_initialization() {
        let manager = DefaultTemplateManager::new().await;
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_list_templates_returns_template_info() {
        let manager = DefaultTemplateManager::new().await.unwrap();
        let templates = manager.list_templates().await.unwrap();

        assert_eq!(templates.len(), 3);

        for template_info in templates {
            assert!(!template_info.id.is_empty());
            assert!(!template_info.name.is_empty());
            assert!(!template_info.description.is_empty());
            assert!(template_info.estimated_quality >= 0.0);
            assert!(template_info.estimated_quality <= 5.0);
        }
    }

    #[tokio::test]
    async fn test_get_template_with_caching() {
        let manager = DefaultTemplateManager::new().await.unwrap();

        // First call - should load from storage
        let template1 = manager.get_template("code_reviewer").await.unwrap();
        assert_eq!(template1.id, "code_reviewer");

        // Second call - should use cache (same result)
        let template2 = manager.get_template("code_reviewer").await.unwrap();
        assert_eq!(template2.id, "code_reviewer");
        assert_eq!(template1.name, template2.name);
    }

    #[tokio::test]
    async fn test_get_nonexistent_template_fallback() {
        let manager = DefaultTemplateManager::new().await.unwrap();

        let result = manager.get_template("nonexistent_template").await;
        assert!(result.is_ok()); // Should fallback to available template

        let template = result.unwrap();
        // Should be one of the embedded templates
        assert!(["code_reviewer", "documentation_writer", "conversation_assistant"].contains(&template.id.as_str()));
    }

    #[tokio::test]
    async fn test_render_template_basic() {
        let manager = DefaultTemplateManager::new().await.unwrap();
        let template = manager.get_template("conversation_assistant").await.unwrap();

        let params = HashMap::new();
        let result = manager.render_template(&template, &params).await;

        assert!(result.is_ok());
        let rendered = result.unwrap();
        assert!(!rendered.is_empty());
        // Current simple implementation just returns the role
        assert!(rendered.contains("helpful assistant") || rendered.contains("You are"));
    }

    #[tokio::test]
    async fn test_validate_quality_returns_score() {
        let manager = DefaultTemplateManager::new().await.unwrap();

        let score = manager.validate_quality("You are a helpful assistant.");

        assert!(score.overall_score >= 0.0);
        assert!(score.overall_score <= 5.0);
        assert!(score.confidence >= 0.0);
        assert!(score.confidence <= 1.0);
    }

    #[tokio::test]
    async fn test_validate_quality_different_prompts() {
        let manager = DefaultTemplateManager::new().await.unwrap();

        let good_prompt = "You are an expert software engineer with 10 years of experience in Rust development.";
        let basic_prompt = "Help me.";

        let good_score = manager.validate_quality(good_prompt);
        let basic_score = manager.validate_quality(basic_prompt);

        // Both should be valid scores
        assert!(good_score.overall_score >= 0.0);
        assert!(basic_score.overall_score >= 0.0);
    }

    #[tokio::test]
    async fn test_fallback_strategy_with_empty_storage() {
        let manager = DefaultTemplateManager::new().await.unwrap();

        // Try to get a template that doesn't exist
        let result = manager.get_template("definitely_not_exists").await;

        // Should still succeed due to fallback strategy
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_template_info_quality_estimation() {
        let manager = DefaultTemplateManager::new().await.unwrap();
        let templates = manager.list_templates().await.unwrap();

        for template_info in templates {
            // Quality should be estimated for each template
            assert!(template_info.estimated_quality > 0.0);
            assert!(template_info.estimated_quality <= 5.0);

            // Usage stats should be present
            assert!(template_info.usage_stats.success_rate >= 0.0);
            assert!(template_info.usage_stats.success_rate <= 1.0);
        }
    }
}
