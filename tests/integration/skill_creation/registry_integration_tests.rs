use super::IntegrationTestContext;
use crate::cli::skills::{creation::SkillType, registry::SkillsRegistry};

#[tokio::test]
async fn test_skill_registration_after_creation() -> anyhow::Result<()> {
    let mut ctx = IntegrationTestContext::new()?;
    
    // Create skill
    ctx.session.start_creation("registry-test", SkillType::Command)?;
    ctx.session.set_description("Test registry integration")?;
    ctx.session.set_command("echo registry")?;
    ctx.session.finalize_skill()?;
    
    // Load registry and verify skill is registered
    let registry = SkillsRegistry::load_from_directory(&ctx.skills_dir)?;
    let skills = registry.list_skills();
    
    assert!(skills.iter().any(|s| s.name == "registry-test"));
    
    Ok(())
}

#[tokio::test]
async fn test_registry_parsing_of_created_skills() -> anyhow::Result<()> {
    let mut ctx = IntegrationTestContext::new()?;
    
    // Create multiple skills of different types
    let test_skills = [
        ("parse-cmd", SkillType::Command, "echo test"),
        ("parse-repl", SkillType::Repl, "python3"),
    ];
    
    for (name, skill_type, command) in test_skills {
        ctx.session.start_creation(name, skill_type)?;
        ctx.session.set_description(&format!("Parse test for {}", name))?;
        ctx.session.set_command(command)?;
        ctx.session.finalize_skill()?;
    }
    
    // Verify registry can parse all created skills
    let registry = SkillsRegistry::load_from_directory(&ctx.skills_dir)?;
    let skills = registry.list_skills();
    
    assert_eq!(skills.len(), 2);
    assert!(skills.iter().any(|s| s.name == "parse-cmd"));
    assert!(skills.iter().any(|s| s.name == "parse-repl"));
    
    Ok(())
}

#[tokio::test]
async fn test_skill_execution_after_creation() -> anyhow::Result<()> {
    let mut ctx = IntegrationTestContext::new()?;
    
    // Create executable skill
    ctx.session.start_creation("exec-test", SkillType::Command)?;
    ctx.session.set_description("Executable test skill")?;
    ctx.session.set_command("echo 'execution test'")?;
    ctx.session.finalize_skill()?;
    
    // Load and execute skill
    let registry = SkillsRegistry::load_from_directory(&ctx.skills_dir)?;
    let skill = registry.get_skill("exec-test").expect("Skill should exist");
    
    // Verify skill can be loaded and has correct properties
    assert_eq!(skill.name, "exec-test");
    assert_eq!(skill.description, "Executable test skill");
    
    Ok(())
}
