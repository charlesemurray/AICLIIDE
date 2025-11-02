#[cfg(test)]
mod performance_tests {
    use super::*;
    use crate::cli::creation::prompt_system::PromptSystem;
    use std::time::Instant;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_system_initialization_performance() {
        let start = Instant::now();
        let system = PromptSystem::new().await.unwrap();
        let duration = start.elapsed();
        
        // Should initialize quickly (under 100ms)
        assert!(duration.as_millis() < 100);
        
        // Verify system is functional
        let templates = system.list_templates().await.unwrap();
        assert!(!templates.is_empty());
    }

    #[tokio::test]
    async fn test_template_listing_performance() {
        let system = PromptSystem::new().await.unwrap();
        
        let start = Instant::now();
        let templates = system.list_templates().await.unwrap();
        let duration = start.elapsed();
        
        // Should list templates quickly (under 10ms)
        assert!(duration.as_millis() < 10);
        assert_eq!(templates.len(), 3);
    }

    #[tokio::test]
    async fn test_template_retrieval_performance() {
        let system = PromptSystem::new().await.unwrap();
        
        let start = Instant::now();
        let template = system.get_template("code_reviewer").await.unwrap();
        let duration = start.elapsed();
        
        // Should retrieve template quickly (under 5ms)
        assert!(duration.as_millis() < 5);
        assert_eq!(template.id, "code_reviewer");
    }

    #[tokio::test]
    async fn test_repeated_access_performance() {
        let system = PromptSystem::new().await.unwrap();
        
        // First access (cache miss)
        let start = Instant::now();
        let _template1 = system.get_template("conversation_assistant").await.unwrap();
        let first_duration = start.elapsed();
        
        // Second access (cache hit)
        let start = Instant::now();
        let _template2 = system.get_template("conversation_assistant").await.unwrap();
        let second_duration = start.elapsed();
        
        // Second access should be faster or similar (caching effect)
        assert!(second_duration <= first_duration + std::time::Duration::from_millis(1));
    }

    #[tokio::test]
    async fn test_suggestion_algorithm_performance() {
        let system = PromptSystem::new().await.unwrap();
        
        let start = Instant::now();
        let suggestions = system.suggest_templates_for_use_case("code review analysis").await.unwrap();
        let duration = start.elapsed();
        
        // Should generate suggestions quickly (under 5ms)
        assert!(duration.as_millis() < 5);
        assert!(!suggestions.is_empty());
        assert!(suggestions.len() <= 3);
    }

    #[tokio::test]
    async fn test_quality_validation_performance() {
        let system = PromptSystem::new().await.unwrap();
        
        let prompt = "You are an expert software engineer with extensive experience in Rust, Python, and JavaScript.";
        
        let start = Instant::now();
        let score = system.validate_prompt(prompt);
        let duration = start.elapsed();
        
        // Should validate quickly (under 1ms)
        assert!(duration.as_millis() < 1);
        assert!(score.overall_score >= 0.0);
    }

    #[tokio::test]
    async fn test_bulk_operations_performance() {
        let system = PromptSystem::new().await.unwrap();
        
        let start = Instant::now();
        
        // Perform multiple operations
        for _ in 0..10 {
            let _templates = system.list_templates().await.unwrap();
            let _template = system.get_template("code_reviewer").await.unwrap();
            let _suggestions = system.suggest_templates_for_use_case("help").await.unwrap();
        }
        
        let duration = start.elapsed();
        
        // Should handle bulk operations efficiently (under 50ms total)
        assert!(duration.as_millis() < 50);
    }

    #[tokio::test]
    async fn test_memory_usage_stability() {
        let system = PromptSystem::new().await.unwrap();
        
        // Perform many operations to test for memory leaks
        for i in 0..100 {
            let templates = system.list_templates().await.unwrap();
            assert_eq!(templates.len(), 3);
            
            let template = system.get_template("conversation_assistant").await.unwrap();
            assert_eq!(template.id, "conversation_assistant");
            
            let suggestions = system.suggest_templates_for_use_case(&format!("test case {}", i)).await.unwrap();
            assert!(!suggestions.is_empty());
            
            // Validate some prompts
            let score = system.validate_prompt(&format!("Test prompt {}", i));
            assert!(score.overall_score >= 0.0);
        }
        
        // If we get here without running out of memory, test passes
        assert!(true);
    }

    #[tokio::test]
    async fn test_edge_case_empty_strings() {
        let system = PromptSystem::new().await.unwrap();
        
        // Test empty use case
        let suggestions = system.suggest_templates_for_use_case("").await.unwrap();
        assert!(!suggestions.is_empty()); // Should return all templates
        
        // Test empty prompt validation
        let score = system.validate_prompt("");
        assert!(score.overall_score >= 0.0);
        
        // Test whitespace-only strings
        let score = system.validate_prompt("   \n\t  ");
        assert!(score.overall_score >= 0.0);
    }

    #[tokio::test]
    async fn test_edge_case_very_long_strings() {
        let system = PromptSystem::new().await.unwrap();
        
        // Test very long use case
        let long_use_case = "a".repeat(1000);
        let suggestions = system.suggest_templates_for_use_case(&long_use_case).await.unwrap();
        assert!(!suggestions.is_empty());
        
        // Test very long prompt
        let long_prompt = "You are a helpful assistant. ".repeat(100);
        let score = system.validate_prompt(&long_prompt);
        assert!(score.overall_score >= 0.0);
    }

    #[tokio::test]
    async fn test_edge_case_special_characters() {
        let system = PromptSystem::new().await.unwrap();
        
        // Test use case with special characters
        let special_use_case = "code-review & documentation! @#$%^&*()";
        let suggestions = system.suggest_templates_for_use_case(special_use_case).await.unwrap();
        assert!(!suggestions.is_empty());
        
        // Test prompt with special characters
        let special_prompt = "You are a helpful assistant! @#$%^&*()_+-=[]{}|;':\",./<>?";
        let score = system.validate_prompt(special_prompt);
        assert!(score.overall_score >= 0.0);
    }

    #[tokio::test]
    async fn test_edge_case_unicode_characters() {
        let system = PromptSystem::new().await.unwrap();
        
        // Test with Unicode characters
        let unicode_use_case = "código revisión 代码审查 コードレビュー";
        let suggestions = system.suggest_templates_for_use_case(unicode_use_case).await.unwrap();
        assert!(!suggestions.is_empty());
        
        // Test prompt with Unicode
        let unicode_prompt = "You are a helpful assistant. 你好世界 こんにちは世界 🚀🎉";
        let score = system.validate_prompt(unicode_prompt);
        assert!(score.overall_score >= 0.0);
    }
}
