use std::path::PathBuf;
use tempfile::TempDir;
use crate::cli::skills::creation::SkillCreationSession;

pub struct IntegrationTestContext {
    pub temp_dir: TempDir,
    pub skills_dir: PathBuf,
    pub session: SkillCreationSession,
}

impl IntegrationTestContext {
    pub fn new() -> anyhow::Result<Self> {
        let temp_dir = TempDir::new()?;
        let skills_dir = temp_dir.path().join(".q-skills");
        std::fs::create_dir_all(&skills_dir)?;
        
        let session = SkillCreationSession::new(skills_dir.clone())?;
        
        Ok(Self {
            temp_dir,
            skills_dir,
            session,
        })
    }
    
    pub fn verify_skill_created(&self, skill_name: &str) -> anyhow::Result<()> {
        let skill_file = self.skills_dir.join(format!("{}.json", skill_name));
        assert!(skill_file.exists(), "Skill JSON file should exist");
        
        let content = std::fs::read_to_string(&skill_file)?;
        let _: serde_json::Value = serde_json::from_str(&content)?;
        
        Ok(())
    }
}

pub mod workflow_tests;
pub mod file_operations_tests;
pub mod registry_integration_tests;
