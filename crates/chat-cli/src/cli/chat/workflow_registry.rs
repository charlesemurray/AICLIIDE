//! Workflow registry for managing workflow definitions

use std::collections::HashMap;
use std::path::Path;

use eyre::Result;

use crate::cli::chat::tools::workflow::WorkflowDefinition;

pub struct WorkflowRegistry {
    workflows: HashMap<String, WorkflowDefinition>,
}

impl WorkflowRegistry {
    pub fn new() -> Self {
        Self {
            workflows: HashMap::new(),
        }
    }

    pub async fn load_from_directory(&mut self, path: &Path) -> Result<()> {
        let mut entries = tokio::fs::read_dir(path).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let content = tokio::fs::read_to_string(&path).await?;
                let workflow: WorkflowDefinition = serde_json::from_str(&content)?;
                self.workflows.insert(workflow.name.clone(), workflow);
            }
        }

        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&WorkflowDefinition> {
        self.workflows.get(name)
    }

    pub fn len(&self) -> usize {
        self.workflows.len()
    }

    pub fn is_empty(&self) -> bool {
        self.workflows.is_empty()
    }
}

impl Default for WorkflowRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_registry_creation() {
        let registry = WorkflowRegistry::new();
        assert_eq!(registry.len(), 0);
    }

    #[tokio::test]
    async fn test_load_workflows_from_directory() {
        use std::fs;

        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let workflow_path = dir.path().join("test_workflow.json");

        let workflow_json = r#"{
            "name": "test-workflow",
            "version": "1.0.0",
            "description": "A test workflow",
            "steps": []
        }"#;

        fs::write(&workflow_path, workflow_json).unwrap();

        let mut registry = WorkflowRegistry::new();
        registry.load_from_directory(dir.path()).await.unwrap();

        assert_eq!(registry.len(), 1);
        assert!(registry.get("test-workflow").is_some());
    }
}
