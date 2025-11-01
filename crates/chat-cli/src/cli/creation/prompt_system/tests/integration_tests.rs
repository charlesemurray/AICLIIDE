//! Integration tests for prompt system components

use super::super::*;
use eyre::Result;
use tempfile::TempDir;

#[tokio::test]
async fn test_end_to_end_prompt_creation() -> Result<()> {
    // Create a complete prompt using the builder
    let template = PromptBuilder::new()
        .with_name("Code Review Assistant".to_string())
        .with_description("Assists with code reviews focusing on security and best practices".to_string())
        .with_role("You are an expert code reviewer with 10+ years of experience in software security and best practices".to_string())
        .add_capability("security vulnerability detection".to_string())
        .add_capability("performance optimization suggestions".to_string())
        .add_capability("code style and maintainability review".to_string())
        .add_constraint("always explain the reasoning behind suggestions".to_string())
        .add_constraint("provide specific examples when possible".to_string())
        .add_constraint("be constructive and helpful in tone".to_string())
        .with_example(
            "Review this function: def process_user_input(data): return eval(data)".to_string(),
            "This function has a critical security vulnerability. Using eval() on user input allows arbitrary code execution. Instead, use json.loads() for JSON data or implement proper input validation and parsing.".to_string()
        )
        .with_category(TemplateCategory::CodeReview)
        .with_difficulty(DifficultyLevel::Advanced)
        .with_tags(vec!["security".to_string(), "code-review".to_string(), "best-practices".to_string()])
        .build()?;
    
    // Validate the template
    let validation = template.validate()?;
    assert!(validation.is_valid);
    assert!(validation.score > 0.8); // Should be high quality
    
    // Generate the prompt
    let generated_prompt = template.generate_prompt()?;
    
    // Verify all components are included
    assert!(generated_prompt.contains("expert code reviewer"));
    assert!(generated_prompt.contains("security vulnerability detection"));
    assert!(generated_prompt.contains("always explain the reasoning"));
    assert!(generated_prompt.contains("eval(data)"));
    assert!(generated_prompt.contains("json.loads()"));
    
    Ok(())
}

#[tokio::test]
async fn test_template_manager_integration() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let mut manager = TemplateManager::new(temp_dir.path())?;
    
    // Create template using builder
    let template = PromptBuilder::new()
        .with_name("Documentation Assistant".to_string())
        .with_description("Helps write technical documentation".to_string())
        .with_role("You are a technical writing expert specializing in clear, comprehensive documentation".to_string())
        .add_capability("API documentation".to_string())
        .add_capability("user guides".to_string())
        .add_constraint("use clear, simple language".to_string())
        .with_category(TemplateCategory::Documentation)
        .build()?;
    
    // Save template
    manager.save_template("documentation-assistant", &template).await?;
    
    // Load it back
    let loaded_template = manager.load_template("documentation-assistant").await?;
    
    assert_eq!(loaded_template.name, template.name);
    assert_eq!(loaded_template.capabilities, template.capabilities);
    assert_eq!(loaded_template.metadata.category, TemplateCategory::Documentation);
    
    Ok(())
}

#[test]
fn test_validation_edge_cases() -> Result<()> {
    // Test very short role - should succeed but with warnings
    let template1 = PromptBuilder::new()
        .with_name("Short Role".to_string())
        .with_role("Help".to_string())
        .build();
    
    // Should succeed despite short role (warnings don't fail build)
    assert!(template1.is_ok());
    
    // Test minimal valid template
    let template2 = PromptBuilder::new()
        .with_name("Minimal Valid".to_string())
        .with_role("You are a helpful assistant with good knowledge and experience".to_string())
        .build()?;
    
    assert_eq!(template2.name, "Minimal Valid");
    
    Ok(())
}

#[test]
fn test_builder_immutability() -> Result<()> {
    let builder1 = PromptBuilder::new()
        .with_name("Original".to_string());
    
    let builder2 = builder1
        .with_name("Modified".to_string());
    
    // Original builder should be consumed, new builder should have modified name
    assert_eq!(builder2.template().name, "Modified");
    
    Ok(())
}

#[test]
fn test_complex_prompt_generation() -> Result<()> {
    let template = PromptBuilder::new()
        .with_name("Complex Assistant".to_string())
        .with_role("You are a multi-faceted AI assistant with expertise across domains".to_string())
        .with_capabilities(vec![
            "code analysis and review".to_string(),
            "technical writing and documentation".to_string(),
            "problem-solving and debugging".to_string(),
            "best practices recommendations".to_string(),
        ])
        .with_constraints(vec![
            "always provide detailed explanations".to_string(),
            "include relevant examples when helpful".to_string(),
            "maintain professional and helpful tone".to_string(),
            "ask clarifying questions when needed".to_string(),
        ])
        .with_example(
            "How do I optimize this slow database query?".to_string(),
            "To optimize your database query, I'll need to see the specific query and understand your database schema. However, here are common optimization strategies: 1) Add appropriate indexes, 2) Avoid SELECT *, 3) Use LIMIT when possible...".to_string()
        )
        .build()?;
    
    let prompt = template.generate_prompt()?;
    
    // Should contain all sections
    assert!(prompt.contains("multi-faceted AI assistant"));
    assert!(prompt.contains("Your key capabilities include:"));
    assert!(prompt.contains("code analysis and review"));
    assert!(prompt.contains("When responding, always:"));
    assert!(prompt.contains("provide detailed explanations"));
    assert!(prompt.contains("Example interaction:"));
    assert!(prompt.contains("optimize this slow database query"));
    
    // Check structure
    let role_pos = prompt.find("multi-faceted AI assistant").unwrap();
    let capabilities_pos = prompt.find("Your key capabilities include:").unwrap();
    let constraints_pos = prompt.find("When responding, always:").unwrap();
    let example_pos = prompt.find("Example interaction:").unwrap();
    
    // Verify order: role -> capabilities -> constraints -> example
    assert!(role_pos < capabilities_pos);
    assert!(capabilities_pos < constraints_pos);
    assert!(constraints_pos < example_pos);
    
    Ok(())
}
