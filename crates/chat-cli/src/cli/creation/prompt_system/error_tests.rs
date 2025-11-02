#[cfg(test)]
mod error_tests {
    use super::*;
    use crate::cli::creation::prompt_system::{PromptSystem, TemplateError};
    use crate::cli::creation::prompt_system::storage::HybridTemplateStorage;
    use crate::cli::creation::prompt_system::template_manager::{DefaultTemplateManager, TemplateStorage, TemplateManager};

    #[tokio::test]
    async fn test_template_error_types() {
        // Test NotFound error
        let error = TemplateError::NotFound { id: "test_id".to_string() };
        let error_string = error.to_string();
        assert!(error_string.contains("test_id"));
        assert!(error_string.contains("not found"));
        
        // Test ValidationFailed error
        let error = TemplateError::ValidationFailed { reason: "invalid format".to_string() };
        let error_string = error.to_string();
        assert!(error_string.contains("invalid format"));
        assert!(error_string.contains("validation failed"));
        
        // Test RenderingFailed error
        let error = TemplateError::RenderingFailed { reason: "missing parameter".to_string() };
        let error_string = error.to_string();
        assert!(error_string.contains("missing parameter"));
        assert!(error_string.contains("rendering failed"));
        
        // Test SecurityViolation error
        let error = TemplateError::SecurityViolation { violation: "unsafe content".to_string() };
        let error_string = error.to_string();
        assert!(error_string.contains("unsafe content"));
        assert!(error_string.contains("Security violation"));
    }

    #[tokio::test]
    async fn test_storage_error_handling() {
        let storage = HybridTemplateStorage::new().await.unwrap();
        
        // Test loading non-existent template
        let result = storage.load_template("definitely_does_not_exist").await;
        assert!(result.is_err());
        
        if let Err(e) = result {
            let error_string = e.to_string();
            assert!(error_string.contains("not found") || error_string.contains("NotFound"));
        }
    }

    #[tokio::test]
    async fn test_manager_fallback_behavior() {
        let manager = DefaultTemplateManager::new().await.unwrap();
        
        // Test that manager handles non-existent templates gracefully
        let result = manager.get_template("non_existent_template_xyz").await;
        
        // Should succeed due to fallback mechanism
        assert!(result.is_ok());
        
        let template = result.unwrap();
        // Should be one of the available templates
        assert!(["code_reviewer", "documentation_writer", "conversation_assistant"].contains(&template.id.as_str()));
    }

    #[tokio::test]
    async fn test_system_error_propagation() {
        let system = PromptSystem::new().await.unwrap();
        
        // Test that system handles errors gracefully
        let result = system.get_template("invalid_template_id_12345").await;
        
        // Should succeed due to fallback strategy
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_parameter_validation_edge_cases() {
        let system = PromptSystem::new().await.unwrap();
        let template = system.get_template("code_reviewer").await.unwrap();
        
        // Test parameter extraction with valid template
        let params = system.get_template_parameters(&template);
        assert!(!params.is_empty());
        
        // Verify parameter structure
        for param in params {
            assert!(!param.name.is_empty());
            // Parameter type should be valid (no validation errors)
        }
    }

    #[tokio::test]
    async fn test_quality_score_validation() {
        let system = PromptSystem::new().await.unwrap();
        
        // Test various prompt types
        let long_prompt = "A".repeat(10000);
        let test_prompts = vec![
            "",
            "a",
            "You are a helpful assistant.",
            &long_prompt, // Very long prompt
            "🚀🎉 Unicode test 你好",
            "Special chars: @#$%^&*()",
        ];
        
        for prompt in test_prompts {
            let score = system.validate_prompt(&prompt);
            
            // Validate score bounds
            assert!(score.overall_score >= 0.0, "Score too low for prompt: '{}'", prompt);
            assert!(score.overall_score <= 5.0, "Score too high for prompt: '{}'", prompt);
            assert!(score.confidence >= 0.0, "Confidence too low for prompt: '{}'", prompt);
            assert!(score.confidence <= 1.0, "Confidence too high for prompt: '{}'", prompt);
        }
    }

    #[tokio::test]
    async fn test_suggestion_algorithm_robustness() {
        let system = PromptSystem::new().await.unwrap();
        
        // Test various use case inputs
        let test_cases = vec![
            "",
            "a",
            "code",
            "documentation writing help",
            "UPPERCASE TEXT",
            "mixed CaSe TeXt",
            "numbers 123 456",
            "special-chars_and.dots",
            "very long use case description that goes on and on and includes many words",
        ];
        
        for use_case in test_cases {
            let suggestions = system.suggest_templates_for_use_case(use_case).await.unwrap();
            
            // Should always return some suggestions
            assert!(!suggestions.is_empty(), "No suggestions for use case: '{}'", use_case);
            
            // Should not exceed maximum
            assert!(suggestions.len() <= 3, "Too many suggestions for use case: '{}'", use_case);
            
            // Each suggestion should be valid
            for suggestion in suggestions {
                assert!(!suggestion.id.is_empty());
                assert!(!suggestion.name.is_empty());
                assert!(suggestion.estimated_quality >= 0.0);
                assert!(suggestion.estimated_quality <= 5.0);
            }
        }
    }

    #[tokio::test]
    async fn test_concurrent_error_handling() {
        use std::sync::Arc;
        
        let system = Arc::new(PromptSystem::new().await.unwrap());
        
        // Test concurrent access with some invalid requests
        let handles = (0..10).map(|i| {
            let system_clone = Arc::clone(&system);
            tokio::spawn(async move {
                if i % 2 == 0 {
                    // Valid requests
                    let _templates = system_clone.list_templates().await.unwrap();
                    let _template = system_clone.get_template("conversation_assistant").await.unwrap();
                } else {
                    // Invalid requests that should be handled gracefully
                    let _result = system_clone.get_template(&format!("invalid_{}", i)).await;
                    // Should not panic, even if it returns an error or fallback
                }
            })
        }).collect::<Vec<_>>();
        
        // All tasks should complete without panicking
        for handle in handles {
            handle.await.unwrap();
        }
    }

    #[tokio::test]
    async fn test_template_structure_validation() {
        let system = PromptSystem::new().await.unwrap();
        let templates = system.list_templates().await.unwrap();
        
        for template_info in templates {
            let full_template = system.get_template(&template_info.id).await.unwrap();
            
            // Validate required fields are not empty
            assert!(!full_template.id.is_empty(), "Template ID is empty");
            assert!(!full_template.name.is_empty(), "Template name is empty");
            assert!(!full_template.description.is_empty(), "Template description is empty");
            assert!(!full_template.role.is_empty(), "Template role is empty");
            assert!(!full_template.capabilities.is_empty(), "Template capabilities are empty");
            
            // Validate version is positive
            assert!(full_template.version > 0, "Template version should be positive");
            
            // Validate usage stats are in valid ranges
            assert!(full_template.usage_stats.success_rate >= 0.0);
            assert!(full_template.usage_stats.success_rate <= 1.0);
            assert!(full_template.usage_stats.avg_satisfaction >= 0.0);
            assert!(full_template.usage_stats.avg_satisfaction <= 5.0);
            
            // Validate parameters if present
            for param in &full_template.parameters {
                assert!(!param.name.is_empty(), "Parameter name is empty");
                assert!(!param.description.is_empty(), "Parameter description is empty");
            }
        }
    }

    #[tokio::test]
    async fn test_rendering_error_handling() {
        let system = PromptSystem::new().await.unwrap();
        let template = system.get_template("code_reviewer").await.unwrap();
        
        // Test rendering with various parameter combinations
        let test_param_sets = vec![
            std::collections::HashMap::new(), // Empty params
            {
                let mut params = std::collections::HashMap::new();
                params.insert("language".to_string(), "rust".to_string());
                params
            },
            {
                let mut params = std::collections::HashMap::new();
                params.insert("invalid_param".to_string(), "value".to_string());
                params
            },
        ];
        
        for params in test_param_sets {
            let result = system.render_template(&template, &params).await;
            
            // Should not fail (current implementation is simple)
            assert!(result.is_ok(), "Rendering failed with params: {:?}", params);
            
            let rendered = result.unwrap();
            assert!(!rendered.is_empty(), "Rendered template is empty");
        }
    }
}
