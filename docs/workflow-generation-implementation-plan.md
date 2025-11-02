# Workflow Generation & Iteration Implementation Plan

## Overview

Implementation plan for AI-powered workflow generation and iteration system. This builds on the base workflow execution engine and adds intelligent creation and modification capabilities.

## Prerequisites

- Base workflow execution engine (from workflow-implementation-plan-v2.md Phase 1-3)
- Skills system integration
- Agent system integration
- Q Developer API access for LLM calls

## Phase 1: Resource Discovery (Week 1)

### Goals
- Discover available skills, agents, and MCP servers
- Build resource catalog for LLM context
- Format resources for AI consumption

### Tasks

#### 1.1 Discovery Module Setup

**Create `workflow/discovery.rs`:**
```rust
use crate::cli::skills::SkillRegistry;
use crate::cli::agent::Agent;
use serde::{Serialize, Deserialize};

pub struct ResourceDiscovery {
    skill_registry: SkillRegistry,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredResources {
    pub skills: Vec<SkillInfo>,
    pub agents: Vec<AgentInfo>,
    pub mcp_servers: Vec<McpServerInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInfo {
    pub name: String,
    pub description: String,
    pub skill_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub name: String,
    pub description: String,
    pub tools: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerInfo {
    pub name: String,
    pub command: String,
}
```

#### 1.2 Skill Discovery

```rust
impl ResourceDiscovery {
    pub async fn new() -> Result<Self, WorkflowError> {
        let current_dir = std::env::current_dir()?;
        let skill_registry = SkillRegistry::with_all_skills(&current_dir)
            .await
            .unwrap_or_else(|_| SkillRegistry::with_builtins());
        
        Ok(Self { skill_registry })
    }
    
    pub async fn discover_all(&self) -> DiscoveredResources {
        DiscoveredResources {
            skills: self.discover_skills().await,
            agents: self.discover_agents().await,
            mcp_servers: self.discover_mcp_servers().await,
        }
    }
    
    async fn discover_skills(&self) -> Vec<SkillInfo> {
        self.skill_registry.list()
            .iter()
            .map(|skill| SkillInfo {
                name: skill.name().to_string(),
                description: skill.description().to_string(),
                skill_type: self.infer_skill_type(skill),
            })
            .collect()
    }
    
    fn infer_skill_type(&self, skill: &dyn crate::cli::skills::Skill) -> String {
        // Infer from skill implementation
        "code_inline".to_string() // Simplified
    }
}
```

#### 1.3 Agent Discovery

```rust
impl ResourceDiscovery {
    async fn discover_agents(&self) -> Vec<AgentInfo> {
        let mut agents = vec![];
        
        // Global agents
        if let Ok(global_dir) = crate::util::paths::global_agents_dir() {
            agents.extend(self.load_agents_from_dir(&global_dir).await);
        }
        
        // Workspace agents
        if let Ok(workspace_dir) = crate::util::paths::workspace_agents_dir() {
            agents.extend(self.load_agents_from_dir(&workspace_dir).await);
        }
        
        agents
    }
    
    async fn load_agents_from_dir(&self, dir: &Path) -> Vec<AgentInfo> {
        let mut agents = vec![];
        
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                if entry.path().extension() == Some(std::ffi::OsStr::new("json")) {
                    if let Ok(agent) = Agent::load_from_path(&entry.path()).await {
                        agents.push(AgentInfo {
                            name: agent.name.clone(),
                            description: agent.description.clone().unwrap_or_default(),
                            tools: agent.tools.clone(),
                        });
                    }
                }
            }
        }
        
        agents
    }
}
```

#### 1.4 MCP Discovery

```rust
impl ResourceDiscovery {
    async fn discover_mcp_servers(&self) -> Vec<McpServerInfo> {
        let mut servers = vec![];
        
        // Load from global mcp.json
        if let Ok(config) = self.load_mcp_config_global().await {
            servers.extend(self.parse_mcp_config(config));
        }
        
        // Load from workspace mcp.json
        if let Ok(config) = self.load_mcp_config_workspace().await {
            servers.extend(self.parse_mcp_config(config));
        }
        
        servers
    }
    
    async fn load_mcp_config_global(&self) -> Result<serde_json::Value, WorkflowError> {
        let path = crate::util::paths::global_mcp_config()?;
        let content = tokio::fs::read_to_string(path).await?;
        Ok(serde_json::from_str(&content)?)
    }
}
```

#### 1.5 Resource Formatting

```rust
impl ResourceDiscovery {
    pub fn format_for_llm(&self, resources: &DiscoveredResources) -> String {
        let mut output = String::new();
        
        output.push_str("Available Skills:\n");
        for skill in &resources.skills {
            output.push_str(&format!(
                "- {} ({}): {}\n",
                skill.name, skill.skill_type, skill.description
            ));
        }
        
        output.push_str("\nAvailable Agents:\n");
        for agent in &resources.agents {
            output.push_str(&format!(
                "- {}: {}\n",
                agent.name, agent.description
            ));
        }
        
        output.push_str("\nAvailable MCP Servers:\n");
        for server in &resources.mcp_servers {
            output.push_str(&format!(
                "- {}: {}\n",
                server.name, server.command
            ));
        }
        
        output
    }
}
```

### Deliverables
- [ ] Resource discovery module
- [ ] Skill discovery working
- [ ] Agent discovery working
- [ ] MCP server discovery working
- [ ] LLM-friendly formatting

### Testing
```rust
#[tokio::test]
async fn test_discover_skills() {
    let discovery = ResourceDiscovery::new().await.unwrap();
    let resources = discovery.discover_all().await;
    assert!(!resources.skills.is_empty());
}
```

## Phase 2: Basic Workflow Generation (Week 2)

### Goals
- LLM integration for workflow generation
- Parse LLM responses into workflow JSON
- Basic approval flow

### Tasks

#### 2.1 Generator Module

**Create `workflow/generator.rs`:**
```rust
use crate::api_client::ApiClient;
use super::discovery::{ResourceDiscovery, DiscoveredResources};
use super::types::*;

pub struct WorkflowGenerator {
    api_client: ApiClient,
    discovery: ResourceDiscovery,
}

impl WorkflowGenerator {
    pub async fn new() -> Result<Self, WorkflowError> {
        Ok(Self {
            api_client: ApiClient::new()?,
            discovery: ResourceDiscovery::new().await?,
        })
    }
    
    pub async fn generate_from_prompt(
        &self,
        prompt: &str,
    ) -> Result<GeneratedWorkflow, WorkflowError> {
        println!("Analyzing your request...\n");
        
        // Discover resources
        let resources = self.discovery.discover_all().await;
        
        // Build system prompt
        let system_prompt = self.build_generation_prompt(&resources);
        
        // Call LLM
        let response = self.api_client
            .send_conversation_message(&system_prompt, prompt)
            .await?;
        
        // Parse response
        let generated = self.parse_generation_response(&response)?;
        
        Ok(generated)
    }
}
```

#### 2.2 LLM Prompt Construction

```rust
impl WorkflowGenerator {
    fn build_generation_prompt(&self, resources: &DiscoveredResources) -> String {
        format!(r#"You are a workflow generation assistant for Q CLI.

{}

Generate a workflow JSON that accomplishes the user's goal.

Rules:
1. Reuse existing skills/agents when possible
2. Identify missing skills that need to be created
3. Use appropriate step types (skill, agent, parallel, conditional)
4. Wire inputs/outputs between steps correctly
5. Add context for configuration values

Output JSON format:
{{
  "workflow": {{
    "name": "workflow-name",
    "version": "1.0.0",
    "description": "what it does",
    "steps": [...]
  }},
  "new_skills": [
    {{
      "name": "skill-name",
      "type": "code_inline",
      "description": "what it does",
      "command": "command to run",
      "reasoning": "why needed"
    }}
  ],
  "reasoning": "explanation of workflow design"
}}
"#,
            self.discovery.format_for_llm(resources)
        )
    }
}
```

#### 2.3 Response Parsing

```rust
impl WorkflowGenerator {
    fn parse_generation_response(
        &self,
        response: &str,
    ) -> Result<GeneratedWorkflow, WorkflowError> {
        // Extract JSON from markdown code blocks if present
        let json = self.extract_json(response)?;
        
        // Parse
        let parsed: GenerationResponse = serde_json::from_str(&json)
            .map_err(|e| WorkflowError::InvalidGeneration(e.to_string()))?;
        
        Ok(GeneratedWorkflow {
            workflow: parsed.workflow,
            new_skills: parsed.new_skills,
            reasoning: parsed.reasoning,
        })
    }
    
    fn extract_json(&self, text: &str) -> Result<String, WorkflowError> {
        // Look for JSON in code blocks
        if let Some(start) = text.find("```json") {
            if let Some(end) = text[start..].find("```") {
                let json = &text[start + 7..start + end];
                return Ok(json.trim().to_string());
            }
        }
        
        // Try to find raw JSON
        if let Some(start) = text.find('{') {
            if let Some(end) = text.rfind('}') {
                return Ok(text[start..=end].to_string());
            }
        }
        
        Err(WorkflowError::InvalidGeneration("No JSON found".to_string()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedWorkflow {
    pub workflow: Workflow,
    pub new_skills: Vec<NewSkillSpec>,
    pub reasoning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewSkillSpec {
    pub name: String,
    #[serde(rename = "type")]
    pub skill_type: String,
    pub description: String,
    pub command: Option<String>,
    pub reasoning: String,
}

#[derive(Debug, Deserialize)]
struct GenerationResponse {
    workflow: Workflow,
    new_skills: Vec<NewSkillSpec>,
    reasoning: String,
}
```

### Deliverables
- [ ] Generator module created
- [ ] LLM integration working
- [ ] Response parsing functional
- [ ] Basic generation end-to-end

### Testing
```rust
#[tokio::test]
async fn test_generate_workflow() {
    let generator = WorkflowGenerator::new().await.unwrap();
    let result = generator.generate_from_prompt(
        "fetch data from API and save to file"
    ).await.unwrap();
    
    assert!(!result.workflow.steps.is_empty());
}
```

## Phase 3: Approval Flow (Week 3)

### Goals
- Interactive approval UI
- Workflow preview
- Skill creation integration

### Tasks

#### 3.1 Approval Module

**Create `workflow/approval.rs`:**
```rust
use std::io::{self, Write};
use super::types::*;
use super::generator::{GeneratedWorkflow, NewSkillSpec};

pub struct ApprovalFlow;

impl ApprovalFlow {
    pub fn show_summary(generated: &GeneratedWorkflow) {
        println!("\nI'll create a workflow with these steps:");
        for (i, step) in generated.workflow.steps.iter().enumerate() {
            println!("{}. {} - {}", 
                i + 1, 
                step.id(), 
                Self::describe_step(step)
            );
        }
        
        println!("\nRequired skills:");
        let existing = Self::find_existing_skills(&generated.workflow);
        for skill in existing {
            println!("✓ {} (existing)", skill);
        }
        
        for skill in &generated.new_skills {
            println!("✗ {} (needs creation)", skill.name);
        }
    }
    
    pub fn prompt_choice() -> Result<UserChoice, io::Error> {
        println!("\nWould you like me to:");
        println!("  [v]iew full workflow");
        println!("  [c]reate missing skills");
        println!("  [s]ave and finish");
        println!("  [q]uit");
        
        print!("\nChoice: ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        match input.trim() {
            "v" => Ok(UserChoice::ViewWorkflow),
            "c" => Ok(UserChoice::CreateSkills),
            "s" => Ok(UserChoice::SaveAll),
            "q" => Ok(UserChoice::Quit),
            _ => Ok(UserChoice::ViewWorkflow),
        }
    }
    
    pub fn show_workflow(workflow: &Workflow) -> Result<bool, io::Error> {
        println!("\nProposed Workflow: {}.json", workflow.name);
        println!("{}", serde_json::to_string_pretty(workflow).unwrap());
        
        print!("\nAccept? [y/n]: ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        Ok(input.trim() == "y")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum UserChoice {
    ViewWorkflow,
    CreateSkills,
    SaveAll,
    Quit,
}
```

#### 3.2 CLI Integration

**Update `workflow/mod.rs`:**
```rust
#[derive(Debug, Subcommand, PartialEq)]
pub enum WorkflowCommand {
    /// Create a workflow from natural language
    Create {
        /// Description of what the workflow should do
        prompt: String,
    },
    // ... other commands
}

impl WorkflowArgs {
    pub async fn execute(self, _os: &mut Os) -> Result<ExitCode> {
        match self.command {
            WorkflowCommand::Create { prompt } => {
                self.create_workflow(&prompt).await
            }
            // ... other commands
        }
    }
    
    async fn create_workflow(&self, prompt: &str) -> Result<ExitCode> {
        let generator = WorkflowGenerator::new().await?;
        let generated = generator.generate_from_prompt(prompt).await?;
        
        // Show summary
        ApprovalFlow::show_summary(&generated);
        
        // Get user choice
        loop {
            match ApprovalFlow::prompt_choice()? {
                UserChoice::ViewWorkflow => {
                    if ApprovalFlow::show_workflow(&generated.workflow)? {
                        break;
                    }
                }
                UserChoice::CreateSkills => {
                    self.create_skills(&generated.new_skills).await?;
                }
                UserChoice::SaveAll => {
                    self.create_skills(&generated.new_skills).await?;
                    self.save_workflow(&generated.workflow).await?;
                    break;
                }
                UserChoice::Quit => {
                    return Ok(ExitCode::FAILURE);
                }
            }
        }
        
        Ok(ExitCode::SUCCESS)
    }
}
```

### Deliverables
- [ ] Approval flow module
- [ ] Interactive prompts working
- [ ] CLI integration complete
- [ ] End-to-end creation flow

## Phase 4: Skill Creation Integration (Week 4)

### Goals
- Integrate with unified creation assistant
- Generate skills from specs
- Skill approval flow

### Tasks

#### 4.1 Skill Creation

```rust
impl WorkflowArgs {
    async fn create_skills(
        &self,
        specs: &[NewSkillSpec],
    ) -> Result<(), WorkflowError> {
        println!("\nCreating missing skills...\n");
        
        for spec in specs {
            println!("Skill: {}", spec.name);
            println!("Type: {}", spec.skill_type);
            println!("Description: {}", spec.description);
            println!("Reasoning: {}", spec.reasoning);
            
            // Generate skill JSON
            let skill_json = self.generate_skill_json(spec)?;
            
            // Show for approval
            println!("\nProposed skill definition:");
            println!("{}", serde_json::to_string_pretty(&skill_json)?);
            
            print!("\nAccept? [y/n]: ");
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            
            if input.trim() == "y" {
                self.save_skill(&spec.name, &skill_json).await?;
                println!("✓ Created {}\n", spec.name);
            } else {
                println!("Skipped {}\n", spec.name);
            }
        }
        
        Ok(())
    }
    
    fn generate_skill_json(&self, spec: &NewSkillSpec) -> Result<serde_json::Value, WorkflowError> {
        Ok(serde_json::json!({
            "name": spec.name,
            "description": spec.description,
            "type": spec.skill_type,
            "command": spec.command,
        }))
    }
    
    async fn save_skill(
        &self,
        name: &str,
        skill_json: &serde_json::Value,
    ) -> Result<(), WorkflowError> {
        let skills_dir = crate::util::paths::workspace_skills_dir()?;
        std::fs::create_dir_all(&skills_dir)?;
        
        let path = skills_dir.join(format!("{}.json", name));
        let json = serde_json::to_string_pretty(skill_json)?;
        tokio::fs::write(path, json).await?;
        
        Ok(())
    }
}
```

### Deliverables
- [ ] Skill creation from specs
- [ ] Skill approval flow
- [ ] Skill saving to disk
- [ ] Integration with workflow creation

## Phase 5-8: See Next Document

Phases 5-8 cover iteration features and will be in a separate implementation plan document.

## Timeline Summary

| Phase | Duration | Deliverable |
|-------|----------|-------------|
| 1 | Week 1 | Resource discovery |
| 2 | Week 2 | Basic generation |
| 3 | Week 3 | Approval flow |
| 4 | Week 4 | Skill creation |

**MVP: 4 weeks** - Users can create workflows from natural language with approval
