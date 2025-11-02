# Workflow Iteration Implementation Plan

## Overview

Implementation plan for workflow iteration features: editing, versioning, rollback, and interactive refinement.

## Prerequisites

- Workflow generation working (phases 1-4 from workflow-generation-implementation-plan.md)
- Base workflow execution engine
- Skills and agents integration

## Phase 5: Workflow Editing (Week 5)

### Goals
- Edit existing workflows with AI
- Generate modifications (not full rewrites)
- Diff preview before applying

### Tasks

#### 5.1 Editor Module

**Create `workflow/editor.rs`:**
```rust
use super::types::*;
use super::discovery::ResourceDiscovery;
use crate::api_client::ApiClient;

pub struct WorkflowEditor {
    api_client: ApiClient,
    discovery: ResourceDiscovery,
}

impl WorkflowEditor {
    pub async fn new() -> Result<Self, WorkflowError> {
        Ok(Self {
            api_client: ApiClient::new()?,
            discovery: ResourceDiscovery::new().await?,
        })
    }
    
    pub async fn edit_workflow(
        &self,
        workflow_name: &str,
        edit_prompt: &str,
        scope: Option<EditScope>,
    ) -> Result<(), WorkflowError> {
        println!("Loading workflow: {} ...\n", workflow_name);
        
        // Load existing workflow
        let original = Workflow::load(workflow_name).await?;
        
        println!("Analyzing change request...\n");
        
        // Generate modifications
        let modifications = self.generate_modifications(
            &original,
            edit_prompt,
            scope,
        ).await?;
        
        // Show what will change
        self.show_modifications(&original, &modifications);
        
        // Get approval
        print!("\nApply changes? [y/n/details]: ");
        std::io::stdout().flush()?;
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        
        match input.trim() {
            "y" => {
                // Apply modifications
                let updated = self.apply_modifications(original, modifications)?;
                updated.save().await?;
                
                println!("\n✓ Workflow updated");
                Ok(())
            }
            "details" => {
                self.show_detailed_diff(&original, &modifications);
                self.edit_workflow(workflow_name, edit_prompt, scope).await
            }
            _ => {
                println!("Cancelled.");
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum EditScope {
    Step(String),
    ErrorHandling,
    Context,
}
```

#### 5.2 Modification Generation

```rust
impl WorkflowEditor {
    async fn generate_modifications(
        &self,
        workflow: &Workflow,
        edit_prompt: &str,
        scope: Option<EditScope>,
    ) -> Result<Vec<WorkflowModification>, WorkflowError> {
        let system_prompt = self.build_edit_prompt(workflow, scope);
        
        let response = self.api_client
            .send_conversation_message(&system_prompt, edit_prompt)
            .await?;
        
        self.parse_modifications(&response)
    }
    
    fn build_edit_prompt(&self, workflow: &Workflow, scope: Option<EditScope>) -> String {
        let scope_instruction = match scope {
            Some(EditScope::Step(step_id)) => {
                format!("ONLY modify step '{}'. Do not change other steps.", step_id)
            }
            Some(EditScope::ErrorHandling) => {
                "ONLY modify error handling configuration.".to_string()
            }
            Some(EditScope::Context) => {
                "ONLY modify workflow context.".to_string()
            }
            None => "Modify only what's necessary.".to_string(),
        };
        
        format!(r#"You are editing an EXISTING workflow. DO NOT rewrite the entire workflow.

Current workflow:
{}

{}

User request: "{}"

Generate ONLY the modifications needed:
{{
  "modifications": [
    {{"type": "add_step", "after": "step-id", "step": {{...}}}},
    {{"type": "modify_step", "id": "step-id", "changes": {{...}}}},
    {{"type": "remove_step", "id": "step-id"}},
    {{"type": "add_error_handling", "step_id": "step-id", "strategy": {{...}}}}
  ],
  "reasoning": "why these specific changes"
}}

Rules:
- Preserve existing steps unless explicitly asked to change
- Only modify what's necessary
- Maintain existing data flow
- Don't break existing references
"#,
            serde_json::to_string_pretty(workflow).unwrap(),
            scope_instruction,
            edit_prompt
        )
    }
    
    fn parse_modifications(&self, response: &str) -> Result<Vec<WorkflowModification>, WorkflowError> {
        let json = self.extract_json(response)?;
        let parsed: ModificationResponse = serde_json::from_str(&json)?;
        Ok(parsed.modifications)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WorkflowModification {
    AddStep {
        after: String,
        step: Step,
    },
    RemoveStep {
        id: String,
    },
    ModifyStep {
        id: String,
        changes: serde_json::Value,
    },
    AddErrorHandling {
        step_id: String,
        strategy: ErrorStrategy,
    },
}

#[derive(Debug, Deserialize)]
struct ModificationResponse {
    modifications: Vec<WorkflowModification>,
    reasoning: String,
}
```

#### 5.3 Modification Display

```rust
impl WorkflowEditor {
    fn show_modifications(&self, original: &Workflow, modifications: &[WorkflowModification]) {
        println!("Proposed changes:");
        
        for modification in modifications {
            match modification {
                WorkflowModification::AddStep { after, step } => {
                    println!("  + Add step '{}' after '{}'", step.id(), after);
                }
                WorkflowModification::RemoveStep { id } => {
                    println!("  - Remove step '{}'", id);
                }
                WorkflowModification::ModifyStep { id, .. } => {
                    println!("  ~ Modify step '{}'", id);
                }
                WorkflowModification::AddErrorHandling { step_id, strategy } => {
                    println!("  ~ Add error handling to '{}': {:?}", step_id, strategy);
                }
            }
        }
        
        println!("\nThis will:");
        println!("  • Keep all other steps unchanged");
        println!("  • Maintain existing data flow");
    }
}
```

#### 5.4 Modification Application

```rust
impl WorkflowEditor {
    fn apply_modifications(
        &self,
        mut workflow: Workflow,
        modifications: Vec<WorkflowModification>,
    ) -> Result<Workflow, WorkflowError> {
        for modification in modifications {
            workflow = self.apply_single_modification(workflow, modification)?;
        }
        
        // Increment version
        workflow.version = self.increment_version(&workflow.version);
        
        Ok(workflow)
    }
    
    fn apply_single_modification(
        &self,
        mut workflow: Workflow,
        modification: WorkflowModification,
    ) -> Result<Workflow, WorkflowError> {
        match modification {
            WorkflowModification::AddStep { after, step } => {
                let position = workflow.steps.iter()
                    .position(|s| s.id() == after)
                    .ok_or_else(|| WorkflowError::StepNotFound(after))?;
                
                workflow.steps.insert(position + 1, step);
            }
            WorkflowModification::RemoveStep { id } => {
                workflow.steps.retain(|s| s.id() != id);
            }
            WorkflowModification::ModifyStep { id, changes } => {
                // Find and modify step
                for step in &mut workflow.steps {
                    if step.id() == id {
                        self.apply_step_changes(step, changes)?;
                        break;
                    }
                }
            }
            WorkflowModification::AddErrorHandling { step_id, strategy } => {
                // Add error handling to workflow config
                workflow.error_handling
                    .per_step
                    .insert(step_id, strategy);
            }
        }
        
        Ok(workflow)
    }
}
```

#### 5.5 CLI Integration

**Update `workflow/mod.rs`:**
```rust
#[derive(Debug, Subcommand, PartialEq)]
pub enum WorkflowCommand {
    Create { prompt: String },
    
    /// Edit an existing workflow
    Edit {
        /// Workflow name
        name: String,
        /// What to change
        prompt: String,
        /// Scope to specific step
        #[arg(long)]
        step: Option<String>,
    },
}

impl WorkflowArgs {
    async fn edit_workflow(
        &self,
        name: &str,
        prompt: &str,
        step: Option<String>,
    ) -> Result<ExitCode> {
        let editor = WorkflowEditor::new().await?;
        
        let scope = step.map(EditScope::Step);
        
        editor.edit_workflow(name, prompt, scope).await?;
        
        Ok(ExitCode::SUCCESS)
    }
}
```

### Deliverables
- [ ] Editor module created
- [ ] Modification generation working
- [ ] Diff display functional
- [ ] `q workflow edit` command working

## Phase 6: Versioning & Rollback (Week 6)

### Goals
- Automatic version control
- Version history display
- Rollback capability

### Tasks

#### 6.1 Versioning Module

**Create `workflow/versioning.rs`:**
```rust
use std::path::PathBuf;
use super::types::*;

pub struct VersionManager {
    versions_dir: PathBuf,
}

impl VersionManager {
    pub fn new() -> Result<Self, WorkflowError> {
        let versions_dir = crate::util::paths::workflow_versions_dir()?;
        std::fs::create_dir_all(&versions_dir)?;
        Ok(Self { versions_dir })
    }
    
    pub fn save_version(&self, workflow: &Workflow) -> Result<(), WorkflowError> {
        let version_num = self.parse_version_number(&workflow.version);
        let version_file = self.versions_dir.join(format!(
            "{}.v{}.json",
            workflow.name,
            version_num
        ));
        
        let json = serde_json::to_string_pretty(workflow)?;
        std::fs::write(version_file, json)?;
        
        Ok(())
    }
    
    pub fn load_version(
        &self,
        workflow_name: &str,
        version: u32,
    ) -> Result<Workflow, WorkflowError> {
        let version_file = self.versions_dir.join(format!(
            "{}.v{}.json",
            workflow_name,
            version
        ));
        
        let json = std::fs::read_to_string(version_file)?;
        let workflow = serde_json::from_str(&json)?;
        
        Ok(workflow)
    }
    
    pub fn list_versions(&self, workflow_name: &str) -> Result<Vec<VersionInfo>, WorkflowError> {
        let mut versions = vec![];
        
        for entry in std::fs::read_dir(&self.versions_dir)? {
            let entry = entry?;
            let filename = entry.file_name().to_string_lossy().to_string();
            
            if filename.starts_with(workflow_name) && filename.ends_with(".json") {
                if let Some(version) = self.extract_version_number(&filename) {
                    let metadata = entry.metadata()?;
                    versions.push(VersionInfo {
                        version,
                        timestamp: metadata.modified()?,
                        size: metadata.len(),
                    });
                }
            }
        }
        
        versions.sort_by(|a, b| b.version.cmp(&a.version));
        Ok(versions)
    }
    
    fn parse_version_number(&self, version: &str) -> u32 {
        version.split('.').next()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1)
    }
    
    fn extract_version_number(&self, filename: &str) -> Option<u32> {
        // Extract from "name.v3.json" format
        filename.split('.').nth(1)
            .and_then(|s| s.strip_prefix('v'))
            .and_then(|s| s.parse().ok())
    }
}

#[derive(Debug, Clone)]
pub struct VersionInfo {
    pub version: u32,
    pub timestamp: std::time::SystemTime,
    pub size: u64,
}
```

#### 6.2 Auto-Save on Edit

**Update `workflow/editor.rs`:**
```rust
use super::versioning::VersionManager;

impl WorkflowEditor {
    pub async fn edit_workflow(
        &self,
        workflow_name: &str,
        edit_prompt: &str,
        scope: Option<EditScope>,
    ) -> Result<(), WorkflowError> {
        let original = Workflow::load(workflow_name).await?;
        let modifications = self.generate_modifications(&original, edit_prompt, scope).await?;
        
        self.show_modifications(&original, &modifications);
        
        if self.confirm_changes()? {
            // Save current version before modifying
            let version_manager = VersionManager::new()?;
            version_manager.save_version(&original)?;
            
            // Apply modifications
            let updated = self.apply_modifications(original, modifications)?;
            updated.save().await?;
            
            println!("\n✓ Workflow updated ({} → {})",
                self.get_version_number(&original.version),
                self.get_version_number(&updated.version)
            );
            println!("✓ Previous version saved as backup");
        }
        
        Ok(())
    }
}
```

#### 6.3 History Command

**Update `workflow/mod.rs`:**
```rust
#[derive(Debug, Subcommand, PartialEq)]
pub enum WorkflowCommand {
    Create { prompt: String },
    Edit { name: String, prompt: String, step: Option<String> },
    
    /// Show workflow version history
    History {
        /// Workflow name
        name: String,
    },
}

impl WorkflowArgs {
    async fn show_history(&self, name: &str) -> Result<ExitCode> {
        let version_manager = VersionManager::new()?;
        let versions = version_manager.list_versions(name)?;
        
        println!("Workflow: {}\n", name);
        
        for (i, version_info) in versions.iter().enumerate() {
            let marker = if i == 0 { " (current)" } else { "" };
            let timestamp = self.format_timestamp(version_info.timestamp);
            
            println!("v{}{} - {}",
                version_info.version,
                marker,
                timestamp
            );
        }
        
        Ok(ExitCode::SUCCESS)
    }
}
```

#### 6.4 Rollback Command

```rust
#[derive(Debug, Subcommand, PartialEq)]
pub enum WorkflowCommand {
    // ... existing commands
    
    /// Rollback to a previous version
    Rollback {
        /// Workflow name
        name: String,
        /// Version to rollback to
        #[arg(long)]
        to: u32,
        /// Save current version before rollback
        #[arg(long)]
        save_current: bool,
    },
}

impl WorkflowArgs {
    async fn rollback_workflow(
        &self,
        name: &str,
        to_version: u32,
        save_current: bool,
    ) -> Result<ExitCode> {
        let version_manager = VersionManager::new()?;
        
        // Load current
        let current = Workflow::load(name).await?;
        let current_version = self.parse_version(&current.version);
        
        println!("⚠️  Rolling back from v{} to v{}\n", current_version, to_version);
        
        // Show what will be lost
        self.show_rollback_impact(name, current_version, to_version)?;
        
        print!("\nProceed? [y/n/save-current]: ");
        std::io::stdout().flush()?;
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        
        match input.trim() {
            "y" | "save-current" => {
                let should_save = input.trim() == "save-current" || save_current;
                
                if should_save {
                    let backup_name = format!("{}-backup-v{}", name, current_version);
                    current.save_as(&backup_name).await?;
                    println!("✓ Saved current as: {}.json", backup_name);
                }
                
                // Load target version
                let target = version_manager.load_version(name, to_version)?;
                target.save().await?;
                
                println!("✓ Rolled back to v{}", to_version);
                Ok(ExitCode::SUCCESS)
            }
            _ => {
                println!("Cancelled.");
                Ok(ExitCode::FAILURE)
            }
        }
    }
}
```

### Deliverables
- [ ] Automatic versioning on edit
- [ ] Version history display
- [ ] Rollback command working
- [ ] Backup before rollback

## Phase 7: Interactive Refinement (Week 7)

### Goals
- Multi-turn interactive editing
- Guided step-by-step changes
- Menu-driven interface

### Tasks

#### 7.1 Refinement Module

**Create `workflow/refine.rs`:**
```rust
use std::io::{self, Write};
use super::types::*;
use super::editor::WorkflowEditor;

pub struct RefinementSession {
    workflow: Workflow,
    editor: WorkflowEditor,
    modifications: Vec<WorkflowModification>,
}

impl RefinementSession {
    pub async fn run(workflow_name: &str) -> Result<(), WorkflowError> {
        let workflow = Workflow::load(workflow_name).await?;
        let editor = WorkflowEditor::new().await?;
        
        let mut session = Self {
            workflow,
            editor,
            modifications: vec![],
        };
        
        session.show_intro();
        
        loop {
            match session.prompt_action()? {
                Action::AddStep => session.add_step_interactive().await?,
                Action::ModifyStep => session.modify_step_interactive().await?,
                Action::RemoveStep => session.remove_step_interactive().await?,
                Action::ErrorHandling => session.configure_error_handling().await?,
                Action::Done => break,
            }
            
            if !session.prompt_continue()? {
                break;
            }
        }
        
        if !session.modifications.is_empty() {
            session.apply_all().await?;
        }
        
        Ok(())
    }
    
    fn show_intro(&self) {
        println!("Refining workflow: {} (v{})",
            self.workflow.name,
            self.workflow.version
        );
        println!("Current: {} steps\n", self.workflow.steps.len());
    }
    
    fn prompt_action(&self) -> Result<Action, io::Error> {
        println!("\nWhat would you like to improve?");
        println!("  [a]dd a step");
        println!("  [m]odify a step");
        println!("  [r]emove a step");
        println!("  [e]rror handling");
        println!("  [d]one refining");
        
        print!("\nChoice: ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        match input.trim() {
            "a" => Ok(Action::AddStep),
            "m" => Ok(Action::ModifyStep),
            "r" => Ok(Action::RemoveStep),
            "e" => Ok(Action::ErrorHandling),
            "d" => Ok(Action::Done),
            _ => Ok(Action::Done),
        }
    }
}

enum Action {
    AddStep,
    ModifyStep,
    RemoveStep,
    ErrorHandling,
    Done,
}
```

#### 7.2 Interactive Step Addition

```rust
impl RefinementSession {
    async fn add_step_interactive(&mut self) -> Result<(), WorkflowError> {
        // Show current steps
        println!("\nCurrent steps:");
        for (i, step) in self.workflow.steps.iter().enumerate() {
            println!("  {}. {}", i + 1, step.id());
        }
        
        // Ask where to add
        println!("\nWhere should I add the new step?");
        for (i, step) in self.workflow.steps.iter().enumerate() {
            println!("  {}. After '{}'", i + 1, step.id());
        }
        
        print!("\nChoice: ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let position: usize = input.trim().parse().unwrap_or(1);
        
        // Ask what it should do
        print!("\nWhat should this step do?\n> ");
        io::stdout().flush()?;
        
        let mut description = String::new();
        io::stdin().read_line(&mut description)?;
        
        // Generate step using AI
        println!("\nAnalyzing...");
        let step = self.editor.generate_step_from_description(&description.trim()).await?;
        
        println!("\nI'll add this step:");
        println!("  Step: {}", step.id());
        println!("  Type: {}", step.type_name());
        
        print!("\nAdd this step? [y/n]: ");
        io::stdout().flush()?;
        
        let mut confirm = String::new();
        io::stdin().read_line(&mut confirm)?;
        
        if confirm.trim() == "y" {
            let after = self.workflow.steps[position - 1].id().to_string();
            self.modifications.push(WorkflowModification::AddStep { after, step });
            println!("✓ Step queued");
        }
        
        Ok(())
    }
}
```

#### 7.3 CLI Integration

```rust
#[derive(Debug, Subcommand, PartialEq)]
pub enum WorkflowCommand {
    // ... existing commands
    
    /// Interactively refine a workflow
    Refine {
        /// Workflow name
        name: String,
    },
}

impl WorkflowArgs {
    async fn refine_workflow(&self, name: &str) -> Result<ExitCode> {
        RefinementSession::run(name).await?;
        Ok(ExitCode::SUCCESS)
    }
}
```

### Deliverables
- [ ] Interactive refinement mode
- [ ] Step-by-step guided changes
- [ ] Menu-driven interface
- [ ] `q workflow refine` command

## Phase 8: Diff & Validation (Week 8)

### Goals
- Visual diff between versions
- Impact analysis
- Validation before apply

### Tasks

#### 8.1 Diff Module

**Create `workflow/diff.rs`:**
```rust
pub struct DiffGenerator;

impl DiffGenerator {
    pub fn generate(original: &Workflow, modified: &Workflow) -> WorkflowDiff {
        WorkflowDiff {
            steps_added: Self::find_added_steps(original, modified),
            steps_removed: Self::find_removed_steps(original, modified),
            steps_modified: Self::find_modified_steps(original, modified),
            context_changes: Self::find_context_changes(original, modified),
        }
    }
    
    pub fn display(diff: &WorkflowDiff) {
        println!("\nChanges:");
        
        for step_id in &diff.steps_added {
            println!("  + Added step: {}", step_id);
        }
        
        for step_id in &diff.steps_removed {
            println!("  - Removed step: {}", step_id);
        }
        
        for step_id in &diff.steps_modified {
            println!("  ~ Modified step: {}", step_id);
        }
    }
}

pub struct WorkflowDiff {
    pub steps_added: Vec<String>,
    pub steps_removed: Vec<String>,
    pub steps_modified: Vec<String>,
    pub context_changes: Vec<String>,
}
```

### Deliverables
- [ ] Diff generation
- [ ] Visual diff display
- [ ] `q workflow diff` command

## Timeline Summary

| Phase | Duration | Deliverable |
|-------|----------|-------------|
| 5 | Week 5 | Workflow editing |
| 6 | Week 6 | Versioning & rollback |
| 7 | Week 7 | Interactive refinement |
| 8 | Week 8 | Diff & validation |

**Total: 8 weeks** for complete generation + iteration system
