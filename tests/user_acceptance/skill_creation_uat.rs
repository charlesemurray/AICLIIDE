use std::path::PathBuf;
use tempfile::TempDir;
use crate::cli::skills::creation::{SkillCreationSession, SkillType};

pub struct UserAcceptanceTestContext {
    temp_dir: TempDir,
    skills_dir: PathBuf,
}

impl UserAcceptanceTestContext {
    pub fn new() -> anyhow::Result<Self> {
        let temp_dir = TempDir::new()?;
        let skills_dir = temp_dir.path().join(".q-skills");
        std::fs::create_dir_all(&skills_dir)?;
        Ok(Self { temp_dir, skills_dir })
    }
}

#[tokio::test]
async fn uat_001_developer_creates_first_command_skill() -> anyhow::Result<()> {
    let ctx = UserAcceptanceTestContext::new()?;
    let mut session = SkillCreationSession::new(ctx.skills_dir.clone())?;
    
    // Simulate user creating git-status skill
    session.start_creation("git-status", SkillType::Command)?;
    session.set_description("Show git repository status")?;
    session.set_command("git status --porcelain")?;
    
    let skill_path = session.finalize_skill()?;
    
    // Verify skill is immediately usable
    assert!(skill_path.exists());
    let content = std::fs::read_to_string(&skill_path)?;
    let json: serde_json::Value = serde_json::from_str(&content)?;
    
    assert_eq!(json["name"], "git-status");
    assert_eq!(json["type"], "command");
    assert_eq!(json["command"], "git status --porcelain");
    
    Ok(())
}

#[tokio::test]
async fn uat_002_data_scientist_creates_python_repl() -> anyhow::Result<()> {
    let ctx = UserAcceptanceTestContext::new()?;
    let mut session = SkillCreationSession::new(ctx.skills_dir.clone())?;
    
    // Create data analysis REPL skill
    session.start_creation("data-analysis", SkillType::Repl)?;
    session.set_description("Python environment with data science libraries")?;
    session.set_command("python3")?;
    
    // Create startup script
    session.create_supporting_file(
        "startup.py", 
        "import pandas as pd\nimport numpy as np\nprint('Data science environment ready')"
    )?;
    
    let skill_path = session.finalize_skill()?;
    
    // Verify REPL skill structure
    assert!(skill_path.exists());
    let startup_file = ctx.skills_dir.join("data-analysis").join("startup.py");
    assert!(startup_file.exists());
    
    Ok(())
}

#[tokio::test]
async fn uat_003_technical_writer_creates_doc_assistant() -> anyhow::Result<()> {
    let ctx = UserAcceptanceTestContext::new()?;
    let mut session = SkillCreationSession::new(ctx.skills_dir.clone())?;
    
    // Create documentation assistant
    session.start_creation("doc-formatter", SkillType::Assistant)?;
    session.set_description("Assistant for formatting technical documentation")?;
    session.set_prompt(
        "Format this text as technical documentation:\n\n{{text}}\n\nUse proper headings, bullet points, and code blocks."
    )?;
    
    // Test prompt with sample input
    let test_result = session.test_template(&[("text", "API endpoint returns user data")])?;
    assert!(test_result.contains("Format this text"));
    assert!(test_result.contains("API endpoint returns user data"));
    
    session.finalize_skill()?;
    
    Ok(())
}

#[tokio::test]
async fn uat_004_devops_creates_infrastructure_template() -> anyhow::Result<()> {
    let ctx = UserAcceptanceTestContext::new()?;
    let mut session = SkillCreationSession::new(ctx.skills_dir.clone())?;
    
    // Create S3 bucket template
    session.start_creation("aws-s3-bucket", SkillType::Template)?;
    session.set_description("CloudFormation template for S3 bucket")?;
    session.set_prompt(
        "Resources:\n  {{bucket_name}}:\n    Type: AWS::S3::Bucket\n    Properties:\n      BucketName: {{bucket_name}}\n      PublicAccessBlockConfiguration:\n        BlockPublicAcls: true"
    )?;
    
    // Test template with parameters
    let test_result = session.test_template(&[
        ("bucket_name", "my-test-bucket")
    ])?;
    
    assert!(test_result.contains("my-test-bucket"));
    assert!(test_result.contains("AWS::S3::Bucket"));
    
    session.finalize_skill()?;
    
    Ok(())
}

#[tokio::test]
async fn uat_005_user_handles_creation_errors_gracefully() -> anyhow::Result<()> {
    let ctx = UserAcceptanceTestContext::new()?;
    let mut session = SkillCreationSession::new(ctx.skills_dir.clone())?;
    
    // Test invalid skill name
    let result = session.start_creation("invalid name with spaces", SkillType::Command);
    assert!(result.is_err());
    
    // Test empty description
    session.start_creation("valid-name", SkillType::Command)?;
    let result = session.set_description("");
    assert!(result.is_err());
    
    // Test missing required fields
    let result = session.finalize_skill();
    assert!(result.is_err());
    
    Ok(())
}
