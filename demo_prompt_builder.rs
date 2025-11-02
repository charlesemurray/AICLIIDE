#!/usr/bin/env rust-script
//! Demo of the interactive prompt builder
//! 
//! Run with: cargo run --example demo_prompt_builder

use chat_cli::cli::creation::prompt_system::*;
use chat_cli::cli::creation::ui::MockTerminalUI;

fn main() -> eyre::Result<()> {
    println!("=== Interactive Prompt Builder Demo ===\n");

    // Demo 1: Template-based creation
    println!("Demo 1: Creating from template (Code Reviewer)");
    println!("-----------------------------------------------");
    
    let mut ui = MockTerminalUI::new(vec![
        "1".to_string(),      // Select code_reviewer
        "".to_string(),       // Use default name
        "y".to_string(),      // Use default role
        "y".to_string(),      // Create
    ]);

    let mut builder = InteractivePromptBuilder::new(&mut ui);
    let template = builder.create_from_template()?;

    println!("\n✓ Created assistant: {}", template.name);
    println!("  ID: {}", template.id);
    println!("  Category: {:?}", template.category);
    println!("  Difficulty: {:?}", template.difficulty);
    println!("  Role: {}", template.role);
    println!("  Capabilities: {}", template.capabilities.len());
    println!("  Constraints: {}", template.constraints.len());
    println!("  Examples: {}", template.examples.len());

    // Demo 2: Custom creation
    println!("\n\nDemo 2: Creating custom assistant");
    println!("-----------------------------------------------");
    
    let mut ui2 = MockTerminalUI::new(vec![
        "Python Helper".to_string(),
        "Helps with Python coding".to_string(),
        "1".to_string(),      // Code
        "".to_string(),       // Default role
        "1,2".to_string(),    // Capabilities
        "1".to_string(),      // Constraint
        "2".to_string(),      // Intermediate
        "n".to_string(),      // No example
        "y".to_string(),      // Create
    ]);

    let mut builder2 = InteractivePromptBuilder::new(&mut ui2);
    let template2 = builder2.create_custom()?;

    println!("\n✓ Created assistant: {}", template2.name);
    println!("  ID: {}", template2.id);
    println!("  Description: {}", template2.description);
    println!("  Category: {:?}", template2.category);
    println!("  Difficulty: {:?}", template2.difficulty);
    println!("  Capabilities: {:?}", template2.capabilities);
    println!("  Constraints: {:?}", template2.constraints);

    // Demo 3: Validation
    println!("\n\nDemo 3: Builder validation");
    println!("-----------------------------------------------");
    
    let builder3 = PromptBuilder::new()
        .with_name("Test Assistant".to_string())
        .with_description("A test".to_string())
        .with_role("You are a helpful test assistant".to_string())
        .add_capability("testing".to_string())
        .add_constraint("be helpful".to_string());

    let validation = builder3.validate()?;
    println!("Valid: {}", validation.is_valid);
    println!("Score: {:.2}/1.0", validation.score);
    println!("Issues: {}", validation.issues.len());
    
    for issue in &validation.issues {
        println!("  - {:?}: {}", issue.severity, issue.message);
    }

    let template3 = builder3.build()?;
    println!("\n✓ Built: {}", template3.name);

    println!("\n=== All demos completed successfully! ===");
    Ok(())
}
