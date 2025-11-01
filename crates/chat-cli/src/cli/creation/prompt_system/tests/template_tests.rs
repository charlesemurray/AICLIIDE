//! Tests for prompt template system

use super::super::*;
use eyre::Result;
use serde_json::json;
use tempfile::TempDir;

#[tokio::test]
async fn test_template_loading() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let templates_dir = temp_dir.path().join("templates");
    std::fs::create_dir_all(&templates_dir)?;
    
    // Create test template
    let template_data = json!({
        "name": "Code Reviewer",
        "description": "Reviews code for security and best practices",
        "role": "You are an expert code reviewer with 10+ years of experience",
        "capabilities": ["security analysis", "performance review"],
        "constraints": ["be constructive", "explain reasoning"],
        "example_conversation": {
            "input": "Review this function: def process_data(data): return data.upper()",
            "output": "This function works but could be improved..."
        }
    });
    
    std::fs::write(
        templates_dir.join("code-reviewer.json"),
        serde_json::to_string_pretty(&template_data)?
    )?;
    
    // Test loading
    let mut manager = TemplateManager::new(&temp_dir.path())?;
    let template = manager.load_template("code-reviewer").await?;
    
    assert_eq!(template.name, "Code Reviewer");
    assert_eq!(template.capabilities.len(), 2);
    assert!(template.capabilities.contains(&"security analysis".to_string()));
    
    Ok(())
}

#[tokio::test]
async fn test_template_not_found() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let mut manager = TemplateManager::new(&temp_dir.path())?;
    
    let result = manager.load_template("nonexistent").await;
    assert!(result.is_err());
    
    Ok(())
}

#[tokio::test]
async fn test_template_validation() -> Result<()> {
    let template = PromptTemplate {
        name: "Test Template".to_string(),
        description: "Test description".to_string(),
        role: "You are a test assistant".to_string(),
        capabilities: vec!["testing".to_string()],
        constraints: vec!["be helpful".to_string()],
        example_conversation: None,
        metadata: TemplateMetadata::default(),
    };
    
    let validation_result = template.validate()?;
    assert!(validation_result.is_valid);
    
    Ok(())
}

#[tokio::test]
async fn test_template_generation() -> Result<()> {
    let template = PromptTemplate {
        name: "Code Reviewer".to_string(),
        description: "Reviews code".to_string(),
        role: "You are an expert code reviewer".to_string(),
        capabilities: vec!["security analysis".to_string()],
        constraints: vec!["be constructive".to_string()],
        example_conversation: None,
        metadata: TemplateMetadata::default(),
    };
    
    let generated_prompt = template.generate_prompt()?;
    
    assert!(generated_prompt.contains("You are an expert code reviewer"));
    assert!(generated_prompt.contains("security analysis"));
    assert!(generated_prompt.contains("be constructive"));
    
    Ok(())
}

#[test]
fn test_template_metadata() {
    let metadata = TemplateMetadata {
        category: TemplateCategory::CodeReview,
        difficulty: DifficultyLevel::Beginner,
        tags: vec!["security".to_string(), "code".to_string()],
        usage_stats: UsageStats {
            success_rate: 0.85,
            avg_satisfaction: 4.2,
            usage_count: 150,
        },
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    assert_eq!(metadata.category, TemplateCategory::CodeReview);
    assert_eq!(metadata.difficulty, DifficultyLevel::Beginner);
    assert_eq!(metadata.tags.len(), 2);
}
