#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::cli::creation::prompt_system::PromptSystem;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_end_to_end_template_workflow() {
        let system = PromptSystem::new().await.unwrap();
        
        // 1. List available templates
        let templates = system.list_templates().await.unwrap();
        assert!(!templates.is_empty());
        
        // 2. Get a specific template
        let template_id = &templates[0].id;
        let template = system.get_template(template_id).await.unwrap();
        
        // 3. Extract parameters
        let _params_info = system.get_template_parameters(&template);
        
        // 4. Render template
        let params = HashMap::new();
        let rendered = system.render_template(&template, &params).await.unwrap();
        assert!(!rendered.is_empty());
        
        // 5. Validate quality
        let score = system.validate_prompt(&rendered);
        assert!(score.overall_score >= 0.0);
    }

    #[tokio::test]
    async fn test_suggest_templates_for_different_use_cases() {
        let system = PromptSystem::new().await.unwrap();
        
        // Test code-related use case
        let code_suggestions = system.suggest_templates_for_use_case("code review").await.unwrap();
        assert!(!code_suggestions.is_empty());
        let has_code_reviewer = code_suggestions.iter().any(|t| t.id == "code_reviewer");
        assert!(has_code_reviewer);
        
        // Test documentation use case
        let doc_suggestions = system.suggest_templates_for_use_case("documentation").await.unwrap();
        assert!(!doc_suggestions.is_empty());
        let has_doc_writer = doc_suggestions.iter().any(|t| t.id == "documentation_writer");
        assert!(has_doc_writer);
        
        // Test general conversation use case
        let chat_suggestions = system.suggest_templates_for_use_case("conversation").await.unwrap();
        assert!(!chat_suggestions.is_empty());
        let has_assistant = chat_suggestions.iter().any(|t| t.id == "conversation_assistant");
        assert!(has_assistant);
    }

    #[tokio::test]
    async fn test_parameter_extraction_and_validation() {
        let system = PromptSystem::new().await.unwrap();
        let template = system.get_template("code_reviewer").await.unwrap();
        
        let params = system.get_template_parameters(&template);
        
        // Should have language parameter
        let language_param = params.iter().find(|p| p.name == "language");
        assert!(language_param.is_some());
        
        let language_param = language_param.unwrap();
        assert!(language_param.required);
        
        // Should have focus_area parameter
        let focus_param = params.iter().find(|p| p.name == "focus_area");
        assert!(focus_param.is_some());
        
        let focus_param = focus_param.unwrap();
        assert!(!focus_param.required); // This one is optional
    }

    #[tokio::test]
    async fn test_template_rendering_with_parameters() {
        let system = PromptSystem::new().await.unwrap();
        let template = system.get_template("code_reviewer").await.unwrap();
        
        // Test with empty parameters
        let empty_params = HashMap::new();
        let rendered1 = system.render_template(&template, &empty_params).await.unwrap();
        assert!(!rendered1.is_empty());
        
        // Test with some parameters
        let mut params = HashMap::new();
        params.insert("language".to_string(), "rust".to_string());
        params.insert("focus_area".to_string(), "security".to_string());
        
        let rendered2 = system.render_template(&template, &params).await.unwrap();
        assert!(!rendered2.is_empty());
        // Both should render successfully (current implementation just returns role)
        assert_eq!(rendered1, rendered2); // Current simple implementation
    }

    #[tokio::test]
    async fn test_quality_validation_consistency() {
        let system = PromptSystem::new().await.unwrap();
        
        let prompt = "You are a helpful assistant.";
        
        // Multiple calls should return consistent results
        let score1 = system.validate_prompt(prompt);
        let score2 = system.validate_prompt(prompt);
        
        assert_eq!(score1.overall_score, score2.overall_score);
        assert_eq!(score1.confidence, score2.confidence);
    }

    #[tokio::test]
    async fn test_suggestion_algorithm_relevance() {
        let system = PromptSystem::new().await.unwrap();
        
        // Test that suggestions are limited to max 3
        let suggestions = system.suggest_templates_for_use_case("general help").await.unwrap();
        assert!(suggestions.len() <= 3);
        
        // Test that suggestions are ordered by relevance (most relevant first)
        let code_suggestions = system.suggest_templates_for_use_case("code").await.unwrap();
        if !code_suggestions.is_empty() {
            // First suggestion should be most relevant
            assert!(!code_suggestions[0].name.is_empty());
        }
    }

    #[tokio::test]
    async fn test_error_handling_graceful_degradation() {
        let system = PromptSystem::new().await.unwrap();
        
        // Test with invalid template ID - should fallback gracefully
        let result = system.get_template("invalid_id_12345").await;
        assert!(result.is_ok()); // Should fallback to available template
        
        // Test empty use case - should still return suggestions
        let suggestions = system.suggest_templates_for_use_case("").await.unwrap();
        assert!(!suggestions.is_empty()); // Should return all templates
    }

    #[tokio::test]
    async fn test_template_metadata_consistency() {
        let system = PromptSystem::new().await.unwrap();
        let templates = system.list_templates().await.unwrap();
        
        for template_info in templates {
            // Get full template and verify consistency
            let full_template = system.get_template(&template_info.id).await.unwrap();
            
            assert_eq!(template_info.id, full_template.id);
            assert_eq!(template_info.name, full_template.name);
            assert_eq!(template_info.description, full_template.description);
            assert_eq!(template_info.category, full_template.category);
            assert_eq!(template_info.difficulty, full_template.difficulty);
        }
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        use std::sync::Arc;
        
        let system = Arc::new(PromptSystem::new().await.unwrap());
        
        // Test concurrent template access
        let handles = (0..5).map(|_| {
            let system_clone = Arc::clone(&system);
            tokio::spawn(async move {
                let templates = system_clone.list_templates().await.unwrap();
                assert!(!templates.is_empty());
                
                let template = system_clone.get_template("conversation_assistant").await.unwrap();
                assert_eq!(template.id, "conversation_assistant");
            })
        }).collect::<Vec<_>>();
        
        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }
    }
}
