#[cfg(test)]
mod storage_tests {
    use super::*;
    use crate::cli::creation::prompt_system::storage::HybridTemplateStorage;
    use crate::cli::creation::prompt_system::template_manager::TemplateStorage;

    #[tokio::test]
    async fn test_storage_initialization() {
        let storage = HybridTemplateStorage::new().await;
        assert!(storage.is_ok());
    }

    #[tokio::test]
    async fn test_embedded_templates_loaded() {
        let storage = HybridTemplateStorage::new().await.unwrap();
        let templates = storage.list_all_templates().await.unwrap();
        
        assert_eq!(templates.len(), 3);
        
        let ids: Vec<&str> = templates.iter().map(|t| t.id.as_str()).collect();
        assert!(ids.contains(&"code_reviewer"));
        assert!(ids.contains(&"documentation_writer"));
        assert!(ids.contains(&"conversation_assistant"));
    }

    #[tokio::test]
    async fn test_load_existing_template() {
        let storage = HybridTemplateStorage::new().await.unwrap();
        
        let template = storage.load_template("code_reviewer").await;
        assert!(template.is_ok());
        
        let template = template.unwrap();
        assert_eq!(template.id, "code_reviewer");
        assert_eq!(template.name, "Code Reviewer");
        assert!(!template.parameters.is_empty());
    }

    #[tokio::test]
    async fn test_load_nonexistent_template() {
        let storage = HybridTemplateStorage::new().await.unwrap();
        
        let result = storage.load_template("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_template_structure_validity() {
        let storage = HybridTemplateStorage::new().await.unwrap();
        let templates = storage.list_all_templates().await.unwrap();
        
        for template in templates {
            // Validate required fields
            assert!(!template.id.is_empty());
            assert!(!template.name.is_empty());
            assert!(!template.description.is_empty());
            assert!(!template.role.is_empty());
            assert!(!template.capabilities.is_empty());
            
            // Validate usage stats
            assert!(template.usage_stats.success_rate >= 0.0);
            assert!(template.usage_stats.success_rate <= 1.0);
            assert!(template.usage_stats.avg_satisfaction >= 0.0);
            assert!(template.usage_stats.avg_satisfaction <= 5.0);
        }
    }

    #[tokio::test]
    async fn test_code_reviewer_template_specifics() {
        let storage = HybridTemplateStorage::new().await.unwrap();
        let template = storage.load_template("code_reviewer").await.unwrap();
        
        assert_eq!(template.category, crate::cli::creation::prompt_system::types::TemplateCategory::CodeReviewer);
        assert_eq!(template.difficulty, crate::cli::creation::prompt_system::types::DifficultyLevel::Intermediate);
        
        // Should have language parameter
        let has_language_param = template.parameters.iter().any(|p| p.name == "language");
        assert!(has_language_param);
        
        // Should have focus_area parameter
        let has_focus_param = template.parameters.iter().any(|p| p.name == "focus_area");
        assert!(has_focus_param);
    }

    #[tokio::test]
    async fn test_documentation_writer_template_specifics() {
        let storage = HybridTemplateStorage::new().await.unwrap();
        let template = storage.load_template("documentation_writer").await.unwrap();
        
        assert_eq!(template.category, crate::cli::creation::prompt_system::types::TemplateCategory::DocumentationWriter);
        assert_eq!(template.difficulty, crate::cli::creation::prompt_system::types::DifficultyLevel::Beginner);
        
        // Should have doc_type parameter
        let has_doc_type = template.parameters.iter().any(|p| p.name == "doc_type");
        assert!(has_doc_type);
    }

    #[tokio::test]
    async fn test_conversation_assistant_template_specifics() {
        let storage = HybridTemplateStorage::new().await.unwrap();
        let template = storage.load_template("conversation_assistant").await.unwrap();
        
        assert_eq!(template.category, crate::cli::creation::prompt_system::types::TemplateCategory::ConversationAssistant);
        assert_eq!(template.difficulty, crate::cli::creation::prompt_system::types::DifficultyLevel::Beginner);
        
        // Should have tone parameter
        let has_tone_param = template.parameters.iter().any(|p| p.name == "tone");
        assert!(has_tone_param);
    }
}
