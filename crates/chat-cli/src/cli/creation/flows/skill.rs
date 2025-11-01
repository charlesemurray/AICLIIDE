//! Skill creation flow - medium complexity

use crate::cli::creation::{
    CreationFlow, CreationConfig, CreationArtifact, CreationType, CreationPhase, PhaseResult,
    CreationMode, SkillType, SecurityLevel
};
use eyre::Result;
use serde::{Serialize, Deserialize};
use std::path::Path;

/// Skill creation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillConfig {
    pub name: String,
    pub skill_type: SkillType,
    pub command: String,
    pub description: String,
    pub security: SecurityConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub enabled: bool,
    pub level: SecurityLevel,
    pub resource_limit: u64,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            level: SecurityLevel::Medium,
            resource_limit: 1000,
        }
    }
}

impl CreationConfig for SkillConfig {
    fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(crate::cli::creation::CreationError::missing_required_field("name", "my-skill").into());
        }
        Ok(())
    }

    fn apply_defaults(&mut self) {
        if self.description.is_empty() {
            self.description = format!("Skill: {}", self.name);
        }
    }

    fn is_complete(&self) -> bool {
        !self.name.is_empty() && !self.command.is_empty()
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

/// Skill creation artifact
pub struct SkillArtifact {
    config: SkillConfig,
}

impl CreationArtifact for SkillArtifact {
    fn persist(&self, location: &Path) -> Result<()> {
        std::fs::create_dir_all(location)?;
        let file_path = location.join(format!("{}.json", self.config.name));
        let json = serde_json::to_string_pretty(&self.config)?;
        std::fs::write(file_path, json)?;
        Ok(())
    }

    fn validate_before_save(&self) -> Result<()> {
        self.config.validate()
    }

    fn get_name(&self) -> &str {
        &self.config.name
    }
}

/// Skill creation flow
pub struct SkillCreationFlow {
    config: SkillConfig,
    mode: CreationMode,
}

impl SkillCreationFlow {
    pub fn new(name: String, mode: CreationMode) -> Result<Self> {
        let config = SkillConfig {
            name,
            skill_type: SkillType::CodeInline,
            command: String::new(),
            description: String::new(),
            security: SecurityConfig::default(),
        };

        Ok(Self { config, mode })
    }
}

impl CreationFlow for SkillCreationFlow {
    type Config = SkillConfig;
    type Artifact = SkillArtifact;

    fn creation_type(&self) -> CreationType {
        CreationType::Skill
    }

    fn execute_phase(&mut self, phase: CreationPhase) -> Result<PhaseResult> {
        match phase {
            CreationPhase::Discovery => {
                match self.mode {
                    CreationMode::Quick => {
                        // Quick mode: auto-detect skill type and minimal config
                        self.config.skill_type = SkillType::CodeInline;
                        self.config.command = "echo 'Hello from skill'".to_string();
                    }
                    CreationMode::Guided => {
                        // Guided mode: step-by-step configuration
                        self.config.command = "python script.py".to_string();
                        self.config.skill_type = SkillType::CodeInline;
                    }
                    CreationMode::Expert => {
                        // Expert mode: full configuration options
                        self.config.skill_type = SkillType::Conversation;
                        self.config.command = "You are a helpful assistant".to_string();
                    }
                    _ => {}
                }
                Ok(PhaseResult::Continue)
            }
            CreationPhase::BasicConfig => {
                if self.config.description.is_empty() {
                    self.config.description = format!("Skill: {}", self.config.name);
                }
                Ok(PhaseResult::Continue)
            }
            CreationPhase::Security => {
                // Configure security settings based on mode
                match self.mode {
                    CreationMode::Expert => {
                        self.config.security.level = SecurityLevel::High;
                        self.config.security.resource_limit = 2000;
                    }
                    _ => {
                        self.config.security.level = SecurityLevel::Medium;
                    }
                }
                Ok(PhaseResult::Continue)
            }
            CreationPhase::Testing => {
                // Validate skill configuration
                self.config.validate()?;
                Ok(PhaseResult::Continue)
            }
            CreationPhase::Completion => {
                self.config.apply_defaults();
                Ok(PhaseResult::Complete)
            }
            _ => Ok(PhaseResult::Continue),
        }
    }

    fn create_artifact(&self) -> Result<Self::Artifact> {
        self.config.validate()?;
        Ok(SkillArtifact {
            config: self.config.clone(),
        })
    }

    fn get_config(&self) -> &Self::Config {
        &self.config
    }
}
