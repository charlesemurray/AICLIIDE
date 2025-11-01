//! Tests for prompt builder functionality

use super::super::*;
use eyre::Result;

#[test]
fn test_prompt_builder_basic() -> Result<()> {
    let builder = PromptBuilder::new()
        .with_name("Test Assistant".to_string())
        .with_description("A test assistant".to_string())
        .with_role("You are a helpful test assistant".to_string());
    
    assert_eq!(builder.template().name, "Test Assistant");
    assert_eq!(builder.template().description, "A test assistant");
    assert_eq!(builder.template().role, "You are a helpful test assistant");
    
    Ok(())
}

#[test]
fn test_prompt_builder_capabilities() -> Result<()> {
    let builder = PromptBuilder::new()
        .with_name("Code Helper".to_string())
        .add_capability("code review".to_string())
        .add_capability("bug fixing".to_string());
    
    assert_eq!(builder.template().capabilities.len(), 2);
    assert!(builder.template().capabilities.contains(&"code review".to_string()));
    assert!(builder.template().capabilities.contains(&"bug fixing".to_string()));
    
    Ok(())
}

#[test]
fn test_prompt_builder_with_capabilities() -> Result<()> {
    let capabilities = vec!["testing".to_string(), "debugging".to_string()];
    let builder = PromptBuilder::new()
        .with_name("Test Helper".to_string())
        .with_capabilities(capabilities.clone());
    
    assert_eq!(builder.template().capabilities, capabilities);
    
    Ok(())
}

#[test]
fn test_prompt_builder_constraints() -> Result<()> {
    let builder = PromptBuilder::new()
        .with_name("Careful Assistant".to_string())
        .add_constraint("be precise".to_string())
        .add_constraint("explain reasoning".to_string());
    
    assert_eq!(builder.template().constraints.len(), 2);
    assert!(builder.template().constraints.contains(&"be precise".to_string()));
    
    Ok(())
}

#[test]
fn test_prompt_builder_example() -> Result<()> {
    let builder = PromptBuilder::new()
        .with_name("Example Assistant".to_string())
        .with_example(
            "What is 2+2?".to_string(),
            "2+2 equals 4".to_string()
        );
    
    let example = builder.template().example_conversation.as_ref().unwrap();
    assert_eq!(example.input, "What is 2+2?");
    assert_eq!(example.output, "2+2 equals 4");
    
    Ok(())
}

#[test]
fn test_prompt_builder_metadata() -> Result<()> {
    let tags = vec!["code".to_string(), "review".to_string()];
    let builder = PromptBuilder::new()
        .with_name("Code Reviewer".to_string())
        .with_category(TemplateCategory::CodeReview)
        .with_difficulty(DifficultyLevel::Advanced)
        .with_tags(tags.clone());
    
    assert_eq!(builder.template().metadata.category, TemplateCategory::CodeReview);
    assert_eq!(builder.template().metadata.difficulty, DifficultyLevel::Advanced);
    assert_eq!(builder.template().metadata.tags, tags);
    
    Ok(())
}

#[test]
fn test_prompt_builder_validation_success() -> Result<()> {
    let builder = PromptBuilder::new()
        .with_name("Valid Assistant".to_string())
        .with_description("A valid test assistant".to_string())
        .with_role("You are a helpful assistant with extensive knowledge".to_string())
        .add_capability("answering questions".to_string())
        .add_constraint("be helpful".to_string());
    
    let validation = builder.validate()?;
    assert!(validation.is_valid);
    assert!(validation.score > 0.5);
    
    Ok(())
}

#[test]
fn test_prompt_builder_validation_failure() -> Result<()> {
    let builder = PromptBuilder::new(); // Empty template
    
    let validation = builder.validate()?;
    assert!(!validation.is_valid);
    assert!(!validation.issues.is_empty());
    
    // Should have error for empty name
    let has_name_error = validation.issues
        .iter()
        .any(|issue| issue.severity == IssueSeverity::Error && 
                    issue.message.contains("name cannot be empty"));
    assert!(has_name_error);
    
    Ok(())
}

#[test]
fn test_prompt_builder_build_success() -> Result<()> {
    let template = PromptBuilder::new()
        .with_name("Working Assistant".to_string())
        .with_description("A working test assistant".to_string())
        .with_role("You are a helpful assistant with good knowledge".to_string())
        .add_capability("helping users".to_string())
        .build()?;
    
    assert_eq!(template.name, "Working Assistant");
    assert!(!template.capabilities.is_empty());
    
    Ok(())
}

#[test]
fn test_prompt_builder_build_failure() {
    let result = PromptBuilder::new().build(); // Empty template should fail
    assert!(result.is_err());
}

#[test]
fn test_prompt_builder_preview() -> Result<()> {
    let builder = PromptBuilder::new()
        .with_name("Preview Assistant".to_string())
        .with_role("You are a preview assistant".to_string())
        .add_capability("generating previews".to_string())
        .add_constraint("be concise".to_string())
        .with_example(
            "Show me a preview".to_string(),
            "Here's your preview".to_string()
        );
    
    let preview = builder.preview()?;
    
    assert!(preview.contains("You are a preview assistant"));
    assert!(preview.contains("generating previews"));
    assert!(preview.contains("be concise"));
    assert!(preview.contains("Show me a preview"));
    assert!(preview.contains("Here's your preview"));
    
    Ok(())
}

#[test]
fn test_prompt_builder_chaining() -> Result<()> {
    let template = PromptBuilder::new()
        .with_name("Chained Assistant".to_string())
        .with_description("Built with method chaining".to_string())
        .with_role("You are a chained assistant".to_string())
        .add_capability("method chaining".to_string())
        .add_capability("fluent interface".to_string())
        .add_constraint("maintain fluency".to_string())
        .with_category(TemplateCategory::GeneralAssistant)
        .with_difficulty(DifficultyLevel::Intermediate)
        .with_tags(vec!["chaining".to_string(), "fluent".to_string()])
        .build()?;
    
    assert_eq!(template.name, "Chained Assistant");
    assert_eq!(template.capabilities.len(), 2);
    assert_eq!(template.constraints.len(), 1);
    assert_eq!(template.metadata.category, TemplateCategory::GeneralAssistant);
    assert_eq!(template.metadata.difficulty, DifficultyLevel::Intermediate);
    assert_eq!(template.metadata.tags.len(), 2);
    
    Ok(())
}
