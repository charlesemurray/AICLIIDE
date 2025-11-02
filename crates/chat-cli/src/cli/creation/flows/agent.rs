//! Agent creation flow - high complexity

use std::path::Path;

use eyre::Result;
use serde::{
    Deserialize,
    Serialize,
};

use crate::cli::creation::{
    CreationArtifact,
    CreationConfig,
    CreationContext,
    CreationFlow,
    CreationMode,
    CreationPhase,
    CreationType,
    PhaseResult,
    TerminalUI,
};

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
    context: CreationContext,
    ui: Option<Box<dyn TerminalUI>>,
}

impl std::fmt::Debug for AgentCreationFlow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentCreationFlow")
            .field("config", &self.config)
            .field("mode", &self.mode)
            .field("context", &self.context)
            .field("ui", &"<TerminalUI>")
            .finish()
    }
}

impl AgentCreationFlow {
    pub fn new(name: String, mode: CreationMode) -> Result<Self> {
        let current_dir = std::env::current_dir()?;
        let context = CreationContext::new(&current_dir)?;

        // Validate name upfront
        let validation = context.validate_name(&name, &CreationType::Agent);
        if !validation.is_valid {
            return Err(crate::cli::creation::CreationError::invalid_name("agent", &name).into());
        }

        let mut config = AgentConfig {
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

        // Apply smart defaults
        let defaults = context.suggest_defaults(&CreationType::Agent);
        if !defaults.description.is_empty() {
            config.basic.description = defaults.description;
        }
        config.mcp.servers = defaults.mcp_servers;

        Ok(Self {
            config,
            mode,
            context,
            ui: None,
        })
    }

    pub fn with_ui(mut self, ui: Box<dyn TerminalUI>) -> Self {
        self.ui = Some(ui);
        self
    }

    // Stub methods for tests
    pub fn collect_input_single_pass(&mut self) -> Result<AgentConfig> {
        if let Some(ui) = &mut self.ui {
            let prompt = ui.prompt_required("Agent prompt")?;
            let description = ui.prompt_required("Description")?;

            // For expert mode, collect additional configuration
            let mut servers = vec![];
            let mut enabled_tools = vec![];
            let mut enabled_hooks = vec![];

            if self.mode == CreationMode::Expert {
                // MCP servers
                if ui.confirm("Enable MCP servers?")? {
                    let server_input = ui.prompt_required("MCP server name")?;
                    servers.push(server_input);
                }

                // Tools
                if ui.confirm("Enable tools?")? {
                    let tools_input = ui.prompt_required("Allowed tools (comma-separated)")?;
                    enabled_tools = tools_input.split(',').map(|s| s.trim().to_string()).collect();
                }

                // Hooks
                if ui.confirm("Enable hooks?")? {
                    let hook_input = ui.prompt_required("Hook type")?;
                    enabled_hooks.push(hook_input);
                }
            }

            Ok(AgentConfig {
                basic: BasicAgentConfig {
                    name: self.config.basic.name.clone(),
                    description,
                    prompt,
                },
                mcp: McpConfig { servers },
                tools: ToolsConfig { enabled_tools },
                resources: ResourcesConfig { file_paths: vec![] },
                hooks: HooksConfig { enabled_hooks },
            })
        } else {
            // Fallback for when no UI is available
            Ok(AgentConfig {
                basic: BasicAgentConfig {
                    name: self.config.basic.name.clone(),
                    description: "Test agent".to_string(),
                    prompt: "Test role".to_string(),
                },
                mcp: McpConfig { servers: vec![] },
                tools: ToolsConfig { enabled_tools: vec![] },
                resources: ResourcesConfig { file_paths: vec![] },
                hooks: HooksConfig { enabled_hooks: vec![] },
            })
        }
    }

    pub fn run_single_pass(&mut self) -> Result<AgentConfig> {
        self.collect_input_single_pass()
    }

    fn execute_discovery(&mut self, ui: &mut dyn TerminalUI) -> Result<PhaseResult> {
        ui.show_message(
            &format!("Creating agent '{}'", self.config.basic.name),
            crate::cli::creation::SemanticColor::Info,
        );

        match self.mode {
            CreationMode::Quick => {
                self.config.basic.prompt = ui.prompt_required("Agent prompt")?;
            },
            CreationMode::Guided => {
                self.config.basic.prompt =
                    ui.prompt_required("Agent prompt (e.g., 'You are a helpful coding assistant')")?;

                if let Some(desc) = ui.prompt_optional("Description", Some(&self.config.basic.description))? {
                    self.config.basic.description = desc;
                }
            },
            CreationMode::Expert => {
                self.config.basic.prompt = ui.prompt_required("Agent prompt")?;
                self.config.basic.description = ui.prompt_required("Description")?;
            },
            _ => {},
        }
        Ok(PhaseResult::Continue)
    }

    fn execute_advanced_config(&mut self, ui: &mut dyn TerminalUI) -> Result<PhaseResult> {
        match self.mode {
            CreationMode::Expert => {
                // MCP servers configuration
                let enable_mcp = ui.confirm("Enable MCP servers")?;
                if enable_mcp {
                    ui.show_message("Available MCP servers:", crate::cli::creation::SemanticColor::Info);
                    ui.show_message(
                        "  filesystem: File system operations",
                        crate::cli::creation::SemanticColor::Debug,
                    );
                    ui.show_message(
                        "  git: Git repository operations",
                        crate::cli::creation::SemanticColor::Debug,
                    );

                    let servers_input = ui.prompt_optional("MCP servers (comma-separated)", Some("filesystem"))?;
                    if let Some(servers) = servers_input {
                        self.config.mcp.servers = servers.split(',').map(|s| s.trim().to_string()).collect();
                    }
                }

                // Tools configuration
                let enable_tools = ui.confirm("Enable custom tools")?;
                if enable_tools {
                    let tools_input = ui.prompt_optional("Allowed tools (comma-separated)", None)?;
                    if let Some(tools) = tools_input {
                        self.config.tools.enabled_tools = tools.split(',').map(|s| s.trim().to_string()).collect();
                    }
                }

                // Hooks configuration
                let enable_hooks = ui.confirm("Enable lifecycle hooks")?;
                if enable_hooks {
                    ui.show_message("Available hooks:", crate::cli::creation::SemanticColor::Info);
                    ui.show_message(
                        "  agentSpawn: Called when agent starts",
                        crate::cli::creation::SemanticColor::Debug,
                    );
                    ui.show_message(
                        "  userPromptSubmit: Called on user input",
                        crate::cli::creation::SemanticColor::Debug,
                    );

                    let hooks_input = ui.prompt_optional("Hooks (comma-separated)", None)?;
                    if let Some(hooks) = hooks_input {
                        self.config.hooks.enabled_hooks = hooks.split(',').map(|s| s.trim().to_string()).collect();
                    }
                }
            },
            CreationMode::Guided => {
                // Simplified MCP setup
                let enable_mcp = ui.confirm("Enable file system access")?;
                if enable_mcp {
                    self.config.mcp.servers.push("filesystem".to_string());
                }
            },
            _ => {
                // Quick mode uses smart defaults
                if !self.config.mcp.servers.is_empty() {
                    ui.show_message(
                        &format!("Using MCP servers: {}", self.config.mcp.servers.join(", ")),
                        crate::cli::creation::SemanticColor::Info,
                    );
                }
            },
        }
        Ok(PhaseResult::Continue)
    }
}

impl CreationFlow for AgentCreationFlow {
    type Artifact = AgentArtifact;
    type Config = AgentConfig;

    fn creation_type(&self) -> CreationType {
        CreationType::Agent
    }

    fn execute_phase(&mut self, phase: CreationPhase) -> Result<PhaseResult> {
        match phase {
            CreationPhase::Discovery => {
                let mut ui = crate::cli::creation::TerminalUIImpl::new();
                self.execute_discovery(&mut ui)
            },
            CreationPhase::Planning => {
                // Planning phase - prepare for configuration
                Ok(PhaseResult::Continue)
            },
            CreationPhase::BasicConfig => {
                if self.config.basic.description.is_empty() {
                    self.config.basic.description = format!("Agent: {}", self.config.basic.name);
                }
                Ok(PhaseResult::Continue)
            },
            CreationPhase::AdvancedConfig => {
                let mut ui = crate::cli::creation::TerminalUIImpl::new();
                self.execute_advanced_config(&mut ui)
            },
            CreationPhase::Security => {
                let mut ui = crate::cli::creation::TerminalUIImpl::new();
                ui.show_message(
                    "Agent security managed by MCP server permissions",
                    crate::cli::creation::SemanticColor::Info,
                );
                Ok(PhaseResult::Continue)
            },
            CreationPhase::Testing => {
                self.config.validate()?;
                let mut ui = crate::cli::creation::TerminalUIImpl::new();
                ui.show_message(
                    "Agent configuration validated",
                    crate::cli::creation::SemanticColor::Success,
                );
                Ok(PhaseResult::Continue)
            },
            CreationPhase::Completion => {
                self.config.apply_defaults();
                Ok(PhaseResult::Complete)
            },
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
