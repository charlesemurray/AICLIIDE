use super::IntegrationTestContext;
use crate::cli::skills::creation::SkillType;

#[tokio::test]
async fn test_create_command_skill_end_to_end() -> anyhow::Result<()> {
    let mut ctx = IntegrationTestContext::new()?;
    
    // Simulate complete command skill creation
    ctx.session.start_creation("test-command", SkillType::Command)?;
    ctx.session.set_description("Test command skill")?;
    ctx.session.set_command("echo 'hello world'")?;
    
    let skill_path = ctx.session.finalize_skill()?;
    
    // Verify skill was created correctly
    ctx.verify_skill_created("test-command")?;
    assert!(skill_path.exists());
    
    Ok(())
}

#[tokio::test]
async fn test_create_assistant_skill_with_prompt_testing() -> anyhow::Result<()> {
    let mut ctx = IntegrationTestContext::new()?;
    
    ctx.session.start_creation("test-assistant", SkillType::Assistant)?;
    ctx.session.set_description("Test assistant skill")?;
    ctx.session.set_prompt("You are a helpful assistant. Answer: {{question}}")?;
    
    // Test prompt with parameters
    let test_result = ctx.session.test_template(&[("question", "What is 2+2?")])?;
    assert!(test_result.contains("You are a helpful assistant"));
    
    let skill_path = ctx.session.finalize_skill()?;
    ctx.verify_skill_created("test-assistant")?;
    
    Ok(())
}

#[tokio::test]
async fn test_create_all_skill_types() -> anyhow::Result<()> {
    let mut ctx = IntegrationTestContext::new()?;
    
    let skill_types = [
        ("cmd-skill", SkillType::Command),
        ("repl-skill", SkillType::Repl),
        ("assistant-skill", SkillType::Assistant),
        ("template-skill", SkillType::Template),
    ];
    
    for (name, skill_type) in skill_types {
        ctx.session.start_creation(name, skill_type)?;
        ctx.session.set_description(&format!("Test {} skill", name))?;
        
        match skill_type {
            SkillType::Command => ctx.session.set_command("echo test")?,
            SkillType::Repl => ctx.session.set_command("python3")?,
            SkillType::Assistant => ctx.session.set_prompt("Test prompt")?,
            SkillType::Template => ctx.session.set_prompt("Template: {{param}}")?,
        }
        
        ctx.session.finalize_skill()?;
        ctx.verify_skill_created(name)?;
    }
    
    Ok(())
}
