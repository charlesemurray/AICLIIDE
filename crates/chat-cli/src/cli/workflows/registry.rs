use std::collections::HashMap;
use std::path::{Path, PathBuf};

use eyre::{Result, eyre};
use serde_json;

use crate::cli::chat::tools::workflow::WorkflowDefinition;

/// Registry for managing workflows
#[derive(Debug, Clone)]
pub struct WorkflowRegistry {
    workflows: HashMap<String, WorkflowDefinition>,
    workflow_dir: PathBuf,
}

impl WorkflowRegistry {
    /// Create a new workflow registry
    pub fn new(workflow_dir: PathBuf) -> Self {
        Self {
            workflows: HashMap::new(),
            workflow_dir,
        }
    }

    /// Load workflows from directory
    pub async fn load_from_directory(&mut self, dir: &Path) -> Result<()> {
        if !dir.exists() {
            std::fs::create_dir_all(dir)?;
            return Ok(());
        }

        let entries = std::fs::read_dir(dir)?;
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                match self.load_workflow_from_file(&path).await {
                    Ok(workflow) => {
                        self.workflows.insert(workflow.name.clone(), workflow);
                    }
                    Err(e) => {
                        eprintln!("Failed to load workflow from {:?}: {}", path, e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Load a single workflow from file
    async fn load_workflow_from_file(&self, path: &Path) -> Result<WorkflowDefinition> {
        let content = std::fs::read_to_string(path)?;
        let workflow: WorkflowDefinition = serde_json::from_str(&content)?;
        Ok(workflow)
    }

    /// Register a workflow
    pub fn register(&mut self, workflow: WorkflowDefinition) {
        self.workflows.insert(workflow.name.clone(), workflow);
    }

    /// Get a workflow by name
    pub fn get(&self, name: &str) -> Option<&WorkflowDefinition> {
        self.workflows.get(name)
    }

    /// List all workflows
    pub fn list_workflows(&self) -> Vec<&WorkflowDefinition> {
        self.workflows.values().collect()
    }

    /// Remove a workflow
    pub fn remove(&mut self, name: &str) -> Option<WorkflowDefinition> {
        self.workflows.remove(name)
    }

    /// Save a workflow to file
    pub async fn save_workflow(&self, workflow: &WorkflowDefinition) -> Result<PathBuf> {
        let file_path = self.workflow_dir.join(format!("{}.json", workflow.name));
        let json = serde_json::to_string_pretty(workflow)?;
        std::fs::write(&file_path, json)?;
        Ok(file_path)
    }

    /// Delete workflow file
    pub async fn delete_workflow(&mut self, name: &str) -> Result<()> {
        self.workflows.remove(name);
        let file_path = self.workflow_dir.join(format!("{}.json", name));
        if file_path.exists() {
            std::fs::remove_file(file_path)?;
        }
        Ok(())
    }

    /// Check if workflow exists
    pub fn exists(&self, name: &str) -> bool {
        self.workflows.contains_key(name)
    }

    /// Get workflow directory
    pub fn workflow_dir(&self) -> &Path {
        &self.workflow_dir
    }

    /// Get number of workflows
    pub fn len(&self) -> usize {
        self.workflows.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.workflows.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_workflow_registry_new() {
        let dir = tempdir().unwrap();
        let registry = WorkflowRegistry::new(dir.path().to_path_buf());
        assert_eq!(registry.list_workflows().len(), 0);
    }

    #[tokio::test]
    async fn test_register_and_get_workflow() {
        let dir = tempdir().unwrap();
        let mut registry = WorkflowRegistry::new(dir.path().to_path_buf());

        let workflow = WorkflowDefinition {
            name: "test-workflow".to_string(),
            version: "1.0.0".to_string(),
            description: "Test workflow".to_string(),
            steps: vec![],
            context: None,
        };

        registry.register(workflow.clone());
        
        let retrieved = registry.get("test-workflow");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "test-workflow");
    }

    #[tokio::test]
    async fn test_save_and_load_workflow() {
        let dir = tempdir().unwrap();
        let mut registry = WorkflowRegistry::new(dir.path().to_path_buf());

        let workflow = WorkflowDefinition {
            name: "test-workflow".to_string(),
            version: "1.0.0".to_string(),
            description: "Test workflow".to_string(),
            steps: vec![],
            context: None,
        };

        // Save
        registry.save_workflow(&workflow).await.unwrap();

        // Load
        registry.load_from_directory(dir.path()).await.unwrap();
        
        let loaded = registry.get("test-workflow");
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().description, "Test workflow");
    }
}
