//! Agent creation flow - high complexity

use crate::cli::creation::{
    CreationFlow, CreationConfig, CreationArtifact, CreationType, CreationPhase, PhaseResult,
    CreationMode
};
use eyre::Result;
use serde::{Serialize, Deserialize};
use std::path::Path;

/// Agent creation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub basic: BasicAgentConfig,
    pub mcp: McpConfig,
    pub tools: ToolsConfig,
    pub resources: ResourcesConfig,
    pub hooks: HooksConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicAgentConfig {
    pub name: String,
    pub description: String,
    pub prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McpConfig {
    pub servers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolsConfig {
    pub enabled_tools: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourcesConfig {
    pub file_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HooksConfig {
    pub enabled_hooks: Vec<String>,
}

impl CreationConfig for AgentConfig {
    fn validate(&self) -> Result<()> {
        if self.basic.name.is_empty() {
            return Err(crate::cli::creation::CreationError::missing_required_field("name", "my-agent").into());
        }
        Ok(())
    }

    fn apply_defaults(&mut self) {
        if self.basic.description.is_empty() {
            self.basic.description = format!("Agent: {}", self.basic.name);
        }
    }

    fn is_complete(&self) -> bool {
        !self.basic.name.is_empty() && !self.basic.prompt.is_empty()
    }

    fn get_name(&self) -> &str {
        &self.basic.name
    }
}

/// Agent creation artifact
pub struct AgentArtifact {
    config: AgentConfig,
}

impl CreationArtifact for AgentArtifact {
    fn persist(&self, location: &Path) -> Result<()> {
        std::fs::create_dir_all(location)?;
        let file_path = location.join(format!("{}.json", self.config.basic.name));
        let json = serde_json::to_string_pretty(&self.config)?;
        std::fs::write(file_path, json)?;
        Ok(())
    }

    fn validate_before_save(&self) -> Result<()> {
        self.config.validate()
    }

    fn get_name(&self) -> &str {
        &self.config.basic.name
    }
}

/// Agent creation flow
pub struct AgentCreationFlow {
    config: AgentConfig,
    mode: CreationMode,
}

impl AgentCreationFlow {
    pub fn new(name: String, mode: CreationMode) -> Result<Self> {
        let config = AgentConfig {
            basic: BasicAgentConfig {
                name,
                description: String::new(),
                prompt: String::new(),
            },
            mcp: McpConfig::default(),
            tools: ToolsConfig::default(),
            resources: ResourcesConfig::default(),
            hooks: HooksConfig::default(),
        };

        Ok(Self { config, mode })
    }
}

impl CreationFlow for AgentCreationFlow {
    type Config = AgentConfig;
    type Artifact = AgentArtifact;

    fn creation_type(&self) -> CreationType {
        CreationType::Agent
    }

    fn execute_phase(&mut self, phase: CreationPhase) -> Result<PhaseResult> {
        match phase {
            CreationPhase::Discovery => {
                match self.mode {
                    CreationMode::Quick => {
                        self.config.basic.prompt = "You are a helpful assistant".to_string();
                    }
                    CreationMode::Guided => {
                        self.config.basic.prompt = "You are a coding assistant".to_string();
                        self.config.basic.description = "Coding helper".to_string();
                    }
                    CreationMode::Expert => {
                        self.config.basic.prompt = "You are an expert assistant".to_string();
                        self.config.basic.description = "Expert assistant".to_string();
                    }
                    _ => {}
                }
                Ok(PhaseResult::Continue)
            }
            CreationPhase::BasicConfig => {
                if self.config.basic.description.is_empty() {
                    self.config.basic.description = format!("Agent: {}", self.config.basic.name);
                }
                Ok(PhaseResult::Continue)
            }
            CreationPhase::AdvancedConfig => {
                match self.mode {
                    CreationMode::Expert => {
                        // Configure MCP servers for expert mode
                        self.config.mcp.servers.push("filesystem".to_string());
                        self.config.tools.enabled_tools.push("fs_read".to_string());
                        self.config.tools.enabled_tools.push("fs_write".to_string());
                    }
                    CreationMode::Guided => {
                        // Basic MCP setup for guided mode
                        self.config.mcp.servers.push("filesystem".to_string());
                    }
                    _ => {}
                }
                Ok(PhaseResult::Continue)
            }
            CreationPhase::Security => {
                // Agent security is handled by MCP server permissions
                Ok(PhaseResult::Continue)
            }
            CreationPhase::Testing => {
                // Validate agent configuration
                self.config.validate()?;
                Ok(PhaseResult::Continue)
            }
            CreationPhase::Completion => {
                self.config.apply_defaults();
                Ok(PhaseResult::Complete)
            }
        }
    }

    fn create_artifact(&self) -> Result<Self::Artifact> {
        self.config.validate()?;
        Ok(AgentArtifact {
            config: self.config.clone(),
        })
    }

    fn get_config(&self) -> &Self::Config {
        &self.config
    }
}
