use super::IntegrationTestContext;
use crate::cli::skills::creation::SkillType;

#[tokio::test]
async fn test_skill_file_structure() -> anyhow::Result<()> {
    let mut ctx = IntegrationTestContext::new()?;
    
    ctx.session.start_creation("file-test", SkillType::Command)?;
    ctx.session.set_description("Test file structure")?;
    ctx.session.set_command("ls -la")?;
    
    let skill_path = ctx.session.finalize_skill()?;
    
    // Verify JSON file exists and is valid
    assert!(skill_path.exists());
    let content = std::fs::read_to_string(&skill_path)?;
    let json: serde_json::Value = serde_json::from_str(&content)?;
    
    // Verify required fields
    assert_eq!(json["name"], "file-test");
    assert_eq!(json["description"], "Test file structure");
    assert_eq!(json["type"], "command");
    
    Ok(())
}

#[tokio::test]
async fn test_duplicate_skill_name_handling() -> anyhow::Result<()> {
    let mut ctx = IntegrationTestContext::new()?;
    
    // Create first skill
    ctx.session.start_creation("duplicate", SkillType::Command)?;
    ctx.session.set_description("First skill")?;
    ctx.session.set_command("echo first")?;
    ctx.session.finalize_skill()?;
    
    // Attempt to create duplicate - should handle gracefully
    let result = ctx.session.start_creation("duplicate", SkillType::Command);
    assert!(result.is_err(), "Should reject duplicate skill names");
    
    Ok(())
}

#[tokio::test]
async fn test_supporting_files_creation() -> anyhow::Result<()> {
    let mut ctx = IntegrationTestContext::new()?;
    
    ctx.session.start_creation("with-files", SkillType::Assistant)?;
    ctx.session.set_description("Skill with supporting files")?;
    ctx.session.set_prompt("Test prompt")?;
    
    // Add supporting file
    ctx.session.create_supporting_file("helper.txt", "Helper content")?;
    
    ctx.session.finalize_skill()?;
    
    // Verify supporting file exists
    let helper_file = ctx.skills_dir.join("with-files").join("helper.txt");
    assert!(helper_file.exists());
    assert_eq!(std::fs::read_to_string(helper_file)?, "Helper content");
    
    Ok(())
}
