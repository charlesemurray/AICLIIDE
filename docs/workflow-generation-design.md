# AI-Powered Workflow Generation & Iteration Design

## Overview

AI-powered workflow generation is the **primary mechanism** for creating and iterating on workflows in Q CLI. Users describe what they want to accomplish in natural language, and the system generates or modifies workflows intelligently, including creating new skills if needed.

**Core Principle:** Workflows evolve through conversation, not replacement. The system makes surgical changes rather than wholesale rewrites, with safety rails and transparency at every step.

## User Experience

### Creation Flow

```bash
q workflow create "Fetch data from API, validate it, and save to database"
```

**System Response:**
```
Analyzing your request...

I'll create a workflow with these steps:
1. fetch-api-data - Fetch data from API endpoint
2. validate-data - Validate data structure and content  
3. save-to-db - Save validated data to database

Required skills:
✓ http-fetch (existing)
✗ data-validator (needs creation)
✗ db-writer (needs creation)

Would you like me to:
1. Show the proposed workflow
2. Create the missing skills
3. Save everything

[1/2/3/all/cancel]:
```

### Approval Flow

**User selects "all" or "1":**
```json
Proposed Workflow: data-pipeline.json
{
  "name": "data-pipeline",
  "version": "1.0.0",
  "description": "Fetch, validate, and save API data",
  "steps": [
    {
      "id": "fetch",
      "type": "skill",
      "skill": "http-fetch",
      "inputs": {
        "url": "context.api_url"
      },
      "outputs": ["data"]
    },
    {
      "id": "validate",
      "type": "skill",
      "skill": "data-validator",
      "inputs": {
        "data": "fetch.data",
        "schema": "context.schema"
      },
      "outputs": ["validated"]
    },
    {
      "id": "save",
      "type": "skill",
      "skill": "db-writer",
      "inputs": {
        "data": "validate.validated",
        "table": "context.table"
      }
    }
  ],
  "context": {
    "readonly": {
      "api_url": "https://api.example.com/data",
      "schema": "data-schema.json",
      "table": "api_data"
    }
  }
}

Accept this workflow? [y/n/edit]:
```

**User selects "2" or continues:**
```
Creating missing skills...

Skill: data-validator
Type: code_inline
Command: python3 validate.py
Description: Validate JSON data against schema

[Launches unified creation assistant for each skill]

Accept this skill? [y/n/edit]:
```

### Iteration Flow: Safe Modifications

**User wants to improve existing workflow:**
```bash
q workflow edit data-pipeline "add retry logic to the API call"
```

**System responds:**
```
Loading workflow: data-pipeline (v1)

Analyzing change request...

I'll modify the 'fetch' step to add retry logic.

Proposed changes:
  ~ Step 'fetch'
    + Add error_handling:
      - strategy: retry
      - max_attempts: 3
      - backoff: exponential

This will:
  • Keep all existing steps unchanged
  • Only add retry configuration to fetch
  • Maintain all data flow

Apply changes? [y/n/details]: y

✓ Workflow updated (v1 → v2)
✓ Previous version saved as backup
```

### Interactive Refinement

**User wants guided iteration:**
```bash
q workflow refine data-pipeline
```

**System enters interactive mode:**
```
Refining workflow: data-pipeline (v2)
Current: 3 steps, fetches and processes data with retry logic

What would you like to improve?
  [a]dd a step
  [m]odify a step
  [r]emove a step
  [e]rror handling
  [p]arallel execution
  [d]one refining

Choice: a

Where should I add the new step?
  1. Before 'fetch'
  2. After 'fetch'
  3. After 'validate'
  4. At the end

Choice: 2

What should this step do?
> log the response for debugging

Analyzing...

I'll add a logging step after 'fetch':
  
  Step: log-response
  Type: skill
  Skill: logger (existing)
  Inputs: 
    - message: fetch.response
    - level: debug

Add this step? [y/n/edit]: y

✓ Step added

Continue refining? [y/n]: n

✓ Workflow updated (v2 → v3)
```

### Scoped Modification

**User wants to change one specific thing:**
```bash
q workflow edit data-pipeline --step fetch "increase timeout to 30 seconds"
```

**System responds:**
```
Modifying step: fetch

Current configuration:
  skill: http-fetch
  timeout: (default)

Proposed change:
  ~ timeout: default → 30

Apply? [y/n]: y

✓ Updated step 'fetch' (v3 → v4)
```

### Mistake Recovery

**User makes a potentially destructive change:**
```bash
q workflow edit data-pipeline "remove the validation step"
```

**System warns:**
```
⚠️  Warning: This will remove the 'validate' step

Impact analysis:
  - Step 'validate' will be removed
  - Step 'save' depends on 'validate.validated' output
  - This will break the workflow data flow

This seems problematic. Are you sure? [y/n/why]: why

The 'validate' step outputs data used by 'save'. Removing it means:
  • 'save' step will have no input
  • Data flow will be broken
  • Workflow will fail at runtime

Still remove? [y/n/modify instead]: n

Cancelled. Workflow unchanged.
```

### Version History & Rollback

**User wants to see what changed:**
```bash
q workflow history data-pipeline
```

**System shows:**
```
Workflow: data-pipeline

v4 (current) - 2 minutes ago
  Increased fetch timeout to 30s
  
v3 - 5 minutes ago
  Added logging step after fetch
  
v2 - 10 minutes ago
  Added retry logic to API call
  
v1 - 15 minutes ago
  Initial creation

View diff? [y/n]: y
Compare: [from] [to]: v1 v4

Changes from v1 to v4:
  + log-response step (after fetch)
  ~ fetch: added retry (3 attempts, exponential)
  ~ fetch: timeout 30s
```

**User wants to rollback:**
```bash
q workflow rollback data-pipeline --to v2
```

**System responds:**
```
⚠️  Rolling back from v4 to v2

This will undo:
  v4: Timeout increase
  v3: Logging step

Proceed? [y/n/save-current]: save-current

✓ Saved current version as: data-pipeline-backup-v4.json
✓ Rolled back to v2
```

## Iteration Strategies

### Strategy 1: Modification-Only Prompts

The system generates **modifications**, not full rewrites:

```rust
pub enum WorkflowModification {
    AddStep { after: String, step: Step },
    RemoveStep { id: String },
    ModifyStep { id: String, changes: StepChanges },
    AddErrorHandling { step_id: String, strategy: ErrorStrategy },
    AddParallel { step_ids: Vec<String> },
    ReorderSteps { new_order: Vec<String> },
}
```

LLM prompt explicitly instructs modification-only:
```
You are editing an EXISTING workflow. DO NOT rewrite the entire workflow.

Current workflow: {...}
User request: "add retry logic"

Generate ONLY the modifications needed:
{
  "modifications": [
    {"type": "modify_step", "id": "fetch", "changes": {...}}
  ]
}

Rules:
- Preserve existing steps unless explicitly asked to change
- Only modify what's necessary
- Maintain existing data flow
```

### Strategy 2: Automatic Versioning

Every change creates a new version automatically:

```
~/.aws/amazonq/workflows/
  data-pipeline.json          # Current (v4)
  .versions/
    data-pipeline.v1.json
    data-pipeline.v2.json
    data-pipeline.v3.json
```

No manual version management needed. System tracks:
- Version number
- Timestamp
- Change description
- Parent version

### Strategy 3: Diff Preview Before Apply

Always show what will change:

```rust
impl WorkflowEditor {
    pub async fn edit_workflow(
        &self,
        workflow_name: &str,
        edit_prompt: &str,
    ) -> Result<()> {
        let original = Workflow::load(workflow_name).await?;
        let modifications = self.generate_modifications(&original, edit_prompt).await?;
        
        // Show diff
        self.show_diff(&original, &modifications);
        
        // Require approval
        if !self.confirm("Apply these changes?")? {
            return Ok(());
        }
        
        // Save backup
        original.save_as_version()?;
        
        // Apply
        let updated = original.apply(modifications)?;
        updated.save()?;
        
        Ok(())
    }
}
```

### Strategy 4: Impact Analysis

System analyzes impact of changes:

```rust
pub struct ImpactAnalysis {
    pub steps_modified: Vec<String>,
    pub steps_added: Vec<String>,
    pub steps_removed: Vec<String>,
    pub data_flow_broken: Vec<String>,
    pub new_skills_needed: Vec<String>,
    pub warnings: Vec<String>,
}

impl WorkflowEditor {
    fn analyze_impact(&self, modifications: &[WorkflowModification]) -> ImpactAnalysis {
        // Check for broken references
        // Identify new dependencies
        // Detect potential issues
    }
}
```

### Strategy 5: Scoped Edits

Allow targeting specific parts:

```bash
# Scope to specific step
q workflow edit name --step fetch "change"

# Scope to error handling
q workflow edit name --error-handling "add retries"

# Scope to context
q workflow edit name --context "add new variable"
```

### Strategy 6: Interactive Refinement

Multi-turn conversation for complex changes:

```rust
pub struct RefinementSession {
    workflow: Workflow,
    modifications: Vec<WorkflowModification>,
}

impl RefinementSession {
    pub async fn run(&mut self) -> Result<()> {
        loop {
            let choice = self.prompt_action()?;
            
            match choice {
                Action::AddStep => self.add_step_interactive().await?,
                Action::ModifyStep => self.modify_step_interactive().await?,
                Action::Done => break,
            }
        }
        
        self.apply_all_modifications().await
    }
}
```

### Strategy 7: Branching for Experiments

Try changes without affecting main workflow:

```bash
q workflow branch data-pipeline experimental
q workflow edit data-pipeline-experimental "try parallel execution"
# If good:
q workflow merge experimental into data-pipeline
# If bad:
q workflow delete data-pipeline-experimental
```

## Alternative Approaches Considered

### Alternative 1: File-Based Editing (Traditional)

**Approach:** User edits JSON directly in editor

**Pros:**
- Full control
- Works with any editor
- Familiar to developers

**Cons:**
- No guidance or suggestions
- Easy to break data flow
- No automatic versioning
- Steep learning curve
- No understanding of intent

**Why not chosen:** Too error-prone, no iteration support

### Alternative 2: Declarative Config Only (No AI)

**Approach:** User writes complete workflow JSON manually

**Pros:**
- Predictable
- No AI unpredictability
- Full control

**Cons:**
- Requires knowing all skills/agents
- No discovery of existing resources
- Manual wiring of inputs/outputs
- Time-consuming

**Why not chosen:** Doesn't leverage AI capabilities, poor UX

### Alternative 3: Visual Workflow Builder (GUI)

**Approach:** Graphical drag-and-drop interface

**Pros:**
- Visual representation
- Easy to understand flow
- Intuitive editing

**Cons:**
- Not CLI-native
- Requires separate UI
- Slower for power users
- Doesn't fit Q CLI philosophy

**Why not chosen:** Not terminal-native, breaks CLI workflow

### Alternative 4: Template-Based (No Generation)

**Approach:** User picks from predefined templates

**Pros:**
- Fast for common patterns
- Predictable structure
- No AI needed

**Cons:**
- Limited to predefined templates
- Not flexible for unique needs
- Still requires manual editing
- Doesn't learn or improve

**Why not chosen:** Too limiting, doesn't handle custom workflows

### Alternative 5: Imperative API (Code-Based)

**Approach:** User writes workflows in Rust code

**Pros:**
- Type-safe
- IDE support
- Programmatic control

**Cons:**
- Requires Rust knowledge
- Not accessible to non-developers
- Compilation required
- Overkill for simple workflows

**Why not chosen:** Too complex, not accessible

### Alternative 6: Hybrid Manual + AI Assist

**Approach:** User edits manually, AI suggests improvements

**Pros:**
- User control + AI help
- Non-destructive suggestions
- Learn as you go

**Cons:**
- Still requires JSON knowledge
- Slower than pure AI generation
- Suggestions might be ignored

**Why not chosen:** Doesn't make AI primary, still requires manual editing

### Alternative 7: Conversation-Only (No Commands)

**Approach:** Everything through chat interface

**Pros:**
- Natural conversation
- No command syntax to learn
- Contextual

**Cons:**
- Ambiguous (which workflow?)
- No clear workflow management
- Mixed with other chat topics
- Hard to track versions

**Why not chosen:** Lacks structure, hard to manage multiple workflows

### Why Our Approach is Better

The chosen design combines the best aspects:

1. **AI Generation** (fast, discovers resources) + **Human Approval** (safety)
2. **Natural Language** (accessible) + **Structured Commands** (clear intent)
3. **Automatic Versioning** (safety net) + **Explicit Edits** (control)
4. **Diff Preview** (transparency) + **Iterative Mode** (guidance)
5. **File-Based Storage** (portable) + **CLI Management** (convenient)

**Key differentiator:** Treats workflows as living documents that evolve through conversation, with safety rails and transparency at every step.

## Complete Customer Journeys

### Journey 1: First-Time Creation

```bash
$ q workflow create "Monitor API health and alert on failures"

Analyzing your request...

I'll create a workflow that:
1. Checks API endpoint health
2. Evaluates response status
3. Sends alert if unhealthy

Available resources:
✓ http-fetch (existing skill)
✗ health-checker (will create)
✗ alert-sender (will create)

Creating workflow: api-health-monitor

Proposed workflow has 3 steps:
  1. fetch-health → Check API endpoint
  2. evaluate → Determine if healthy
  3. alert → Send notification if needed

Would you like to:
  [v]iew full workflow
  [c]reate missing skills
  [s]ave and finish
  [q]uit

Choice: s

Creating missing skills...

Skill: health-checker
  [Shows definition, user approves]
✓ Created

Skill: alert-sender
  [Shows definition, user approves]
✓ Created

✓ Workflow saved: api-health-monitor.json (v1)
```

### Journey 2: Quick Iteration

```bash
$ q workflow edit api-health-monitor "add retry logic to the API call"

Loading workflow: api-health-monitor (v1)

Proposed changes:
  ~ Step 'fetch-health'
    + Add error_handling: retry (3 attempts, exponential)

Apply? [y/n]: y

✓ Workflow updated (v1 → v2)
```

### Journey 3: Complex Multi-Step Change

```bash
$ q workflow edit api-health-monitor "add parallel health checks for multiple endpoints"

This is a significant change. Would you like:
  [i]terative mode (step by step)
  [p]roceed with all changes
  [c]ancel

Choice: i

Step 1/4: Modify context for multiple URLs
  ~ context.api_url → context.api_urls (array)
Apply? [y/n]: y

Step 2/4: Add parallel health checks
  + parallel-health-checks group (3 steps)
Apply? [y/n]: y

Step 3/4: Modify evaluate step
  ~ Handle array of responses
Apply? [y/n]: y

Step 4/4: Update alert condition
  ~ Check if any endpoint failed
Apply? [y/n]: y

✓ All changes applied (v2 → v3)
```

### Journey 4: Mistake Recovery

```bash
$ q workflow edit api-health-monitor "remove the alert step"

⚠️  Warning: This will remove the 'alert' step

Impact analysis:
  - No alerts will be sent on failure
  - Core notification mechanism will be lost

Still remove? [y/n/modify instead]: n

Cancelled.
```

### Journey 5: History & Rollback

```bash
$ q workflow history api-health-monitor

v3 (current) - Parallel health checks
v2 - Added retry logic  
v1 - Initial creation

$ q workflow rollback api-health-monitor --to v2

⚠️  This will undo: Parallel health checks
Proceed? [y/n/save-current]: save-current

✓ Saved v3 as backup
✓ Rolled back to v2
```

## CLI Commands

```bash
# Creation
q workflow create "description"                    # Create new workflow

# Iteration  
q workflow edit <name> "change"                    # Edit existing workflow
q workflow edit <name> --step <id> "change"        # Edit specific step
q workflow refine <name>                           # Interactive refinement

# History & Recovery
q workflow history <name>                          # Show version history
q workflow diff <name> --from v1 --to v3          # Compare versions
q workflow rollback <name> --to v2                 # Rollback to version

# Branching
q workflow branch <name> <branch-name>             # Create experimental branch
q workflow merge <branch> into <name>              # Merge branch back

# Management
q workflow list                                    # List all workflows
q workflow show <name>                             # Display workflow
q workflow validate <name>                         # Validate workflow
q workflow delete <name>                           # Delete workflow

# Sharing
q workflow export <name>                           # Export as package
q workflow import <package>                        # Import package
```

## Architecture

### Module Structure

```
crates/chat-cli/src/cli/workflow/
  mod.rs                # CLI commands and public API
  generator.rs          # AI-powered workflow generation
  editor.rs             # Workflow modification engine
  discovery.rs          # Resource discovery (skills/agents/MCP)
  approval.rs           # User review and approval flow
  versioning.rs         # Automatic version control
  diff.rs               # Diff generation and display
  refine.rs             # Interactive refinement mode
  types.rs              # Data structures
  error.rs              # Error types
```

### Core Components

#### 1. Resource Discovery

```rust
// In workflow/discovery.rs
use crate::cli::skills::SkillRegistry;
use crate::cli::agent::Agent;
use crate::mcp_client::McpClient;

pub struct ResourceDiscovery {
    skill_registry: SkillRegistry,
}

impl ResourceDiscovery {
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
                skill_type: self.infer_type(skill),
            })
            .collect()
    }
    
    async fn discover_agents(&self) -> Vec<AgentInfo> {
        // Load from ~/.aws/amazonq/agents/ and .amazonq/agents/
        let global_agents = Agent::list_global().await.unwrap_or_default();
        let workspace_agents = Agent::list_workspace().await.unwrap_or_default();
        
        global_agents.into_iter()
            .chain(workspace_agents)
            .map(|agent| AgentInfo {
                name: agent.name.clone(),
                description: agent.description.clone().unwrap_or_default(),
                tools: agent.tools.clone(),
            })
            .collect()
    }
    
    async fn discover_mcp_servers(&self) -> Vec<McpServerInfo> {
        // Load from mcp.json files
        let global_mcp = McpClient::load_global_config().await.unwrap_or_default();
        let workspace_mcp = McpClient::load_workspace_config().await.unwrap_or_default();
        
        // Merge and return server info
        vec![]
    }
}

#[derive(Debug, Clone)]
pub struct DiscoveredResources {
    pub skills: Vec<SkillInfo>,
    pub agents: Vec<AgentInfo>,
    pub mcp_servers: Vec<McpServerInfo>,
}

#[derive(Debug, Clone)]
pub struct SkillInfo {
    pub name: String,
    pub description: String,
    pub skill_type: String,
}

#[derive(Debug, Clone)]
pub struct AgentInfo {
    pub name: String,
    pub description: String,
    pub tools: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct McpServerInfo {
    pub name: String,
    pub command: String,
    pub available_tools: Vec<String>,
}
```

#### 2. Workflow Generator

```rust
// In workflow/generator.rs
use crate::api_client::ApiClient;
use crate::cli::creation::CreationAssistant;

pub struct WorkflowGenerator {
    api_client: ApiClient,
    discovery: ResourceDiscovery,
    creation_assistant: CreationAssistant,
}

impl WorkflowGenerator {
    pub async fn generate_from_prompt(
        &self,
        prompt: &str,
    ) -> Result<GeneratedWorkflow, WorkflowError> {
        // 1. Discover available resources
        let resources = self.discovery.discover_all().await;
        
        // 2. Build generation prompt
        let system_prompt = self.build_system_prompt(&resources);
        
        // 3. Call LLM to generate workflow
        let response = self.api_client
            .send_message(&system_prompt, prompt)
            .await?;
        
        // 4. Parse LLM response
        let generated = self.parse_generation_response(&response)?;
        
        // 5. Validate workflow
        self.validate_generated_workflow(&generated)?;
        
        Ok(generated)
    }
    
    fn build_system_prompt(&self, resources: &DiscoveredResources) -> String {
        format!(
            r#"You are a workflow generation assistant for Q CLI.

Available Skills:
{}

Available Agents:
{}

Available MCP Servers:
{}

Generate a workflow JSON that accomplishes the user's goal.

Rules:
1. Reuse existing skills/agents when possible
2. Identify missing skills that need to be created
3. Use appropriate step types (skill, agent, parallel, conditional)
4. Wire inputs/outputs between steps
5. Add context for configuration values

Output JSON format:
{{
  "workflow": {{ ... workflow definition ... }},
  "new_skills": [
    {{
      "name": "skill-name",
      "type": "code_inline|code_session|conversation|prompt_inline",
      "description": "what it does",
      "command": "command to run",
      "reasoning": "why this skill is needed"
    }}
  ],
  "reasoning": "explanation of the workflow design"
}}
"#,
            self.format_skills(&resources.skills),
            self.format_agents(&resources.agents),
            self.format_mcp_servers(&resources.mcp_servers),
        )
    }
    
    fn format_skills(&self, skills: &[SkillInfo]) -> String {
        skills.iter()
            .map(|s| format!("- {} ({}): {}", s.name, s.skill_type, s.description))
            .collect::<Vec<_>>()
            .join("\n")
    }
    
    fn parse_generation_response(
        &self,
        response: &str,
    ) -> Result<GeneratedWorkflow, WorkflowError> {
        // Extract JSON from LLM response
        let json = self.extract_json_from_response(response)?;
        
        // Parse into structured format
        let parsed: GenerationResponse = serde_json::from_str(&json)?;
        
        Ok(GeneratedWorkflow {
            workflow: parsed.workflow,
            new_skills: parsed.new_skills,
            reasoning: parsed.reasoning,
        })
    }
    
    pub async fn create_missing_skills(
        &self,
        skills: Vec<NewSkillSpec>,
    ) -> Result<Vec<String>, WorkflowError> {
        let mut created = vec![];
        
        for skill_spec in skills {
            println!("\nCreating skill: {}", skill_spec.name);
            println!("Type: {}", skill_spec.skill_type);
            println!("Description: {}", skill_spec.description);
            println!("Reasoning: {}", skill_spec.reasoning);
            
            // Use unified creation assistant
            let skill_json = self.creation_assistant
                .create_skill_from_spec(&skill_spec)
                .await?;
            
            // Show for approval
            println!("\nProposed skill definition:");
            println!("{}", serde_json::to_string_pretty(&skill_json)?);
            
            if self.prompt_approval("Accept this skill?")? {
                // Save skill
                let path = self.save_skill(&skill_spec.name, &skill_json)?;
                created.push(skill_spec.name.clone());
                println!("✓ Skill saved: {}", path.display());
            } else {
                println!("Skipped skill: {}", skill_spec.name);
            }
        }
        
        Ok(created)
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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GenerationResponse {
    workflow: Workflow,
    new_skills: Vec<NewSkillSpec>,
    reasoning: String,
}
```

#### 2. Workflow Editor (Iteration Engine)

```rust
// In workflow/editor.rs
pub struct WorkflowEditor {
    api_client: ApiClient,
    version_manager: VersionManager,
}

impl WorkflowEditor {
    pub async fn edit_workflow(
        &self,
        workflow_name: &str,
        edit_prompt: &str,
    ) -> Result<(), WorkflowError> {
        let original = Workflow::load(workflow_name).await?;
        let modifications = self.generate_modifications(&original, edit_prompt).await?;
        
        self.show_diff(&original, &modifications);
        
        if !self.confirm("Apply changes?")? {
            return Ok(());
        }
        
        self.version_manager.save_version(&original)?;
        let updated = original.apply_modifications(modifications)?;
        updated.save().await?;
        
        Ok(())
    }
    
    fn build_edit_prompt(&self, workflow: &Workflow) -> String {
        format!(r#"
You are editing an EXISTING workflow. DO NOT rewrite the entire workflow.

Current workflow: {}

Generate ONLY the modifications needed.

Rules:
- Preserve existing steps unless explicitly asked to change
- Only modify what's necessary
- Maintain existing data flow
"#, serde_json::to_string_pretty(workflow).unwrap())
    }
}

#[derive(Debug, Clone)]
pub enum WorkflowModification {
    AddStep { after: String, step: Step },
    RemoveStep { id: String },
    ModifyStep { id: String, changes: StepChanges },
    AddErrorHandling { step_id: String, strategy: ErrorStrategy },
}
```

#### 3. Version Manager

```rust
// In workflow/versioning.rs
pub struct VersionManager {
    versions_dir: PathBuf,
}

impl VersionManager {
    pub fn save_version(&self, workflow: &Workflow) -> Result<(), WorkflowError> {
        let version_file = self.versions_dir.join(format!(
            "{}.v{}.json",
            workflow.name,
            workflow.version_number()
        ));
        
        let json = serde_json::to_string_pretty(workflow)?;
        std::fs::write(version_file, json)?;
        Ok(())
    }
    
    pub fn rollback(
        &self,
        workflow_name: &str,
        to_version: u32,
        save_current: bool,
    ) -> Result<(), WorkflowError> {
        let current = Workflow::load(workflow_name).await?;
        
        if save_current {
            let backup = format!("{}-backup-v{}", workflow_name, current.version_number());
            current.save_as(&backup).await?;
        }
        
        let target = self.load_version(workflow_name, to_version)?;
        target.save().await?;
        
        Ok(())
    }
}
```

#### 3. Approval Flow

```rust
// In workflow/approval.rs
use std::io::{self, Write};

pub struct ApprovalFlow;

impl ApprovalFlow {
    pub fn show_workflow_summary(workflow: &Workflow, new_skills: &[NewSkillSpec]) {
        println!("\nI'll create a workflow with these steps:");
        for (i, step) in workflow.steps.iter().enumerate() {
            println!("{}. {} - {}", i + 1, step.id(), self.describe_step(step));
        }
        
        println!("\nRequired skills:");
        let existing_skills = self.find_existing_skills(workflow);
        for skill in existing_skills {
            println!("✓ {} (existing)", skill);
        }
        
        for skill in new_skills {
            println!("✗ {} (needs creation)", skill.name);
        }
    }
    
    pub fn prompt_user_choice() -> Result<UserChoice, io::Error> {
        println!("\nWould you like me to:");
        println!("1. Show the proposed workflow");
        println!("2. Create the missing skills");
        println!("3. Save everything");
        println!("all. Do all of the above");
        println!("cancel. Cancel workflow creation");
        
        print!("\n[1/2/3/all/cancel]: ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        match input.trim() {
            "1" => Ok(UserChoice::ShowWorkflow),
            "2" => Ok(UserChoice::CreateSkills),
            "3" => Ok(UserChoice::SaveAll),
            "all" => Ok(UserChoice::All),
            "cancel" => Ok(UserChoice::Cancel),
            _ => Ok(UserChoice::ShowWorkflow),
        }
    }
    
    pub fn show_workflow_json(workflow: &Workflow) -> Result<bool, io::Error> {
        println!("\nProposed Workflow: {}.json", workflow.name);
        println!("{}", serde_json::to_string_pretty(workflow).unwrap());
        
        print!("\nAccept this workflow? [y/n/edit]: ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        match input.trim() {
            "y" | "yes" => Ok(true),
            "edit" => {
                // TODO: Open in editor
                Ok(false)
            }
            _ => Ok(false),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum UserChoice {
    ShowWorkflow,
    CreateSkills,
    SaveAll,
    All,
    Cancel,
}
```

#### 4. CLI Integration

```rust
// In workflow/mod.rs
#[derive(Debug, Subcommand, PartialEq)]
pub enum WorkflowCommand {
    /// Create a workflow from natural language description
    Create {
        /// Description of what the workflow should do
        prompt: String,
        
        /// Skip approval and auto-create
        #[arg(long)]
        auto_approve: bool,
        
        /// Save to specific location
        #[arg(long)]
        output: Option<PathBuf>,
    },
    // ... other commands
}

impl WorkflowArgs {
    pub async fn execute(self, os: &mut Os) -> Result<ExitCode> {
        match self.command {
            WorkflowCommand::Create { prompt, auto_approve, output } => {
                let generator = WorkflowGenerator::new().await?;
                
                println!("Analyzing your request...\n");
                
                // Generate workflow
                let generated = generator.generate_from_prompt(&prompt).await?;
                
                if !auto_approve {
                    // Show summary
                    ApprovalFlow::show_workflow_summary(
                        &generated.workflow,
                        &generated.new_skills,
                    );
                    
                    // Get user choice
                    loop {
                        match ApprovalFlow::prompt_user_choice()? {
                            UserChoice::ShowWorkflow => {
                                if ApprovalFlow::show_workflow_json(&generated.workflow)? {
                                    break;
                                }
                            }
                            UserChoice::CreateSkills => {
                                generator.create_missing_skills(generated.new_skills.clone()).await?;
                            }
                            UserChoice::SaveAll | UserChoice::All => {
                                // Create skills first
                                generator.create_missing_skills(generated.new_skills.clone()).await?;
                                
                                // Save workflow
                                let path = self.save_workflow(&generated.workflow, output)?;
                                println!("\n✓ Workflow saved: {}", path.display());
                                break;
                            }
                            UserChoice::Cancel => {
                                println!("Cancelled.");
                                return Ok(ExitCode::FAILURE);
                            }
                        }
                    }
                } else {
                    // Auto-approve mode
                    generator.create_missing_skills(generated.new_skills).await?;
                    let path = self.save_workflow(&generated.workflow, output)?;
                    println!("✓ Workflow saved: {}", path.display());
                }
                
                Ok(ExitCode::SUCCESS)
            }
            // ... other commands
        }
    }
}
```

## Integration with Unified Creation Assistant

### Reusing Skill Creation

```rust
// In workflow/generator.rs
impl WorkflowGenerator {
    async fn create_skill_from_spec(
        &self,
        spec: &NewSkillSpec,
    ) -> Result<serde_json::Value, WorkflowError> {
        // Use existing unified creation assistant
        let assistant = CreationAssistant::new();
        
        // Convert spec to creation request
        let request = SkillCreationRequest {
            name: spec.name.clone(),
            skill_type: spec.skill_type.clone(),
            description: spec.description.clone(),
            command: spec.command.clone(),
            // Pre-fill from spec
            auto_configure: true,
        };
        
        // Let creation assistant handle the details
        assistant.create_skill(request).await
    }
}
```

## Example Prompts & Outputs

### Example 1: Simple Linear Workflow

**Prompt:**
```
"Check if a file exists, and if it does, process it with Python"
```

**Generated:**
```json
{
  "workflow": {
    "name": "conditional-file-processor",
    "version": "1.0.0",
    "steps": [
      {
        "id": "check-file",
        "type": "skill",
        "skill": "file-exists",
        "inputs": {"path": "context.file_path"},
        "outputs": ["exists"]
      },
      {
        "id": "conditional",
        "type": "conditional",
        "condition": "check-file.exists == true",
        "then": [
          {
            "id": "process",
            "type": "skill",
            "skill": "python-processor",
            "inputs": {"file": "context.file_path"}
          }
        ]
      }
    ]
  },
  "new_skills": [
    {
      "name": "file-exists",
      "type": "code_inline",
      "command": "test -f",
      "reasoning": "Check file existence"
    },
    {
      "name": "python-processor",
      "type": "code_inline",
      "command": "python3 process.py",
      "reasoning": "Process file with Python"
    }
  ]
}
```

### Example 2: Agent-Based Workflow

**Prompt:**
```
"Analyze code quality, then have an agent fix any issues found"
```

**Generated:**
```json
{
  "workflow": {
    "name": "code-quality-fixer",
    "version": "1.0.0",
    "steps": [
      {
        "id": "analyze",
        "type": "skill",
        "skill": "code-analyzer",
        "inputs": {"path": "context.code_path"},
        "outputs": ["issues"]
      },
      {
        "id": "fix",
        "type": "agent",
        "agent": "code-fixer",
        "inputs": {"issues": "analyze.issues"},
        "async": false,
        "guardrails": {
          "allowed_tools": ["fs_read", "fs_write"],
          "max_iterations": 5
        }
      }
    ]
  },
  "new_skills": [
    {
      "name": "code-analyzer",
      "type": "code_inline",
      "command": "pylint",
      "reasoning": "Analyze Python code quality"
    }
  ]
}
```

## Implementation Phases

### Phase 1: Basic Generation (Week 1)
- [ ] Resource discovery (skills, agents, MCP)
- [ ] LLM prompt construction
- [ ] Basic workflow generation
- [ ] JSON parsing and validation

### Phase 2: Approval Flow (Week 2)
- [ ] Interactive approval UI
- [ ] Workflow preview
- [ ] Edit capability
- [ ] Save workflow

### Phase 3: Skill Creation (Week 3)
- [ ] Integration with unified creation assistant
- [ ] Skill generation from specs
- [ ] Skill approval flow
- [ ] Auto-registration in registry

### Phase 4: Advanced Features (Week 4)
- [ ] Context analysis (current directory, project type)
- [ ] Iterative refinement (user feedback loop)
- [ ] Template library (common workflow patterns)
- [ ] Workflow validation and testing

## Success Criteria

- User can create workflow from natural language in < 2 minutes
- 80%+ of generated workflows are valid on first try
- Existing skills are reused when appropriate
- New skills integrate seamlessly with creation assistant
- User approval flow is clear and efficient

## Future Enhancements

- Learn from user edits to improve generation
- Suggest workflow optimizations
- Generate test cases for workflows
- Workflow templates from common patterns
- Multi-turn conversation for complex workflows
