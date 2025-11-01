//! Performance tests for prompt builder

use super::super::*;
use eyre::Result;
use std::time::Instant;

#[test]
fn test_builder_performance() -> Result<()> {
    let start = Instant::now();
    
    // Create 100 templates to test performance
    for i in 0..100 {
        let _template = PromptBuilder::new()
            .with_name(format!("Test Assistant {}", i))
            .with_description(format!("Test description {}", i))
            .with_role("You are a test assistant with good knowledge and experience".to_string())
            .add_capability("testing".to_string())
            .add_capability("performance".to_string())
            .add_constraint("be efficient".to_string())
            .with_category(TemplateCategory::GeneralAssistant)
            .build()?;
    }
    
    let duration = start.elapsed();
    
    // Should complete in reasonable time (less than 100ms for 100 templates)
    assert!(duration.as_millis() < 100, "Builder took too long: {:?}", duration);
    
    Ok(())
}

#[test]
fn test_validation_performance() -> Result<()> {
    let template = PromptBuilder::new()
        .with_name("Performance Test".to_string())
        .with_description("Testing validation performance".to_string())
        .with_role("You are a performance testing assistant with extensive knowledge".to_string())
        .with_capabilities((0..50).map(|i| format!("capability {}", i)).collect())
        .with_constraints((0..20).map(|i| format!("constraint {}", i)).collect())
        .build()?;
    
    let start = Instant::now();
    
    // Run validation 1000 times
    for _ in 0..1000 {
        let _validation = template.validate()?;
    }
    
    let duration = start.elapsed();
    
    // Should complete in reasonable time (less than 50ms for 1000 validations)
    assert!(duration.as_millis() < 50, "Validation took too long: {:?}", duration);
    
    Ok(())
}

#[test]
fn test_prompt_generation_performance() -> Result<()> {
    let template = PromptBuilder::new()
        .with_name("Generation Test".to_string())
        .with_role("You are a prompt generation testing assistant".to_string())
        .with_capabilities((0..100).map(|i| format!("capability {}", i)).collect())
        .with_constraints((0..50).map(|i| format!("constraint {}", i)).collect())
        .with_example(
            "Test input with some content".to_string(),
            "Test output with detailed response and explanations".to_string()
        )
        .build()?;
    
    let start = Instant::now();
    
    // Generate prompts 1000 times
    for _ in 0..1000 {
        let _prompt = template.generate_prompt()?;
    }
    
    let duration = start.elapsed();
    
    // Should complete in reasonable time (less than 100ms for 1000 generations)
    assert!(duration.as_millis() < 100, "Generation took too long: {:?}", duration);
    
    Ok(())
}

#[test]
fn test_memory_usage() -> Result<()> {
    // Test that we don't have memory leaks with many builders
    let mut templates = Vec::new();
    
    for i in 0..1000 {
        let template = PromptBuilder::new()
            .with_name(format!("Memory Test {}", i))
            .with_role("You are a memory testing assistant".to_string())
            .add_capability("memory management".to_string())
            .build()?;
        
        templates.push(template);
    }
    
    // Verify all templates are created correctly
    assert_eq!(templates.len(), 1000);
    assert!(templates.iter().all(|t| t.name.starts_with("Memory Test")));
    
    Ok(())
}
