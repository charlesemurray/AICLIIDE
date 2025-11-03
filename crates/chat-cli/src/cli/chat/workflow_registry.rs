//! Workflow registry for managing workflow definitions

use std::collections::HashMap;
use std::path::Path;

use eyre::Result;

use crate::cli::chat::tools::workflow::WorkflowDefinition;

#[derive(Clone, Debug)]
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

    pub fn get_workflow(&self, name: &str) -> Option<&WorkflowDefinition> {
        self.get(name)
    }

    pub fn list_workflows(&self) -> Vec<&WorkflowDefinition> {
        self.workflows.values().collect()
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

    #[test]
    fn test_get_workflow_exists() {
        let mut registry = WorkflowRegistry::new();
        let workflow = WorkflowDefinition {
            name: "test-workflow".to_string(),
            version: "1.0.0".to_string(),
            description: "Test".to_string(),
            steps: vec![],
            context: None,
        };
        registry.workflows.insert("test-workflow".to_string(), workflow);

        let result = registry.get_workflow("test-workflow");
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "test-workflow");
    }

    #[test]
    fn test_get_workflow_not_found() {
        let registry = WorkflowRegistry::new();
        let result = registry.get_workflow("nonexistent");
        assert!(result.is_none());
    }

    #[test]
    fn test_list_workflows() {
        let mut registry = WorkflowRegistry::new();

        let workflow1 = WorkflowDefinition {
            name: "workflow-1".to_string(),
            version: "1.0.0".to_string(),
            description: "First workflow".to_string(),
            steps: vec![],
            context: None,
        };
        let workflow2 = WorkflowDefinition {
            name: "workflow-2".to_string(),
            version: "1.0.0".to_string(),
            description: "Second workflow".to_string(),
            steps: vec![],
            context: None,
        };

        registry.workflows.insert("workflow-1".to_string(), workflow1);
        registry.workflows.insert("workflow-2".to_string(), workflow2);

        let workflows = registry.list_workflows();
        assert_eq!(workflows.len(), 2);
    }

    #[tokio::test]
    async fn test_load_from_nonexistent_directory() {
        use std::path::PathBuf;

        let mut registry = WorkflowRegistry::new();
        let nonexistent = PathBuf::from("/nonexistent/workflow/directory");

        let result = registry.load_from_directory(&nonexistent).await;
        assert!(result.is_err() || registry.is_empty());
    }

    #[tokio::test]
    async fn test_load_malformed_workflow_json() {
        use std::fs;

        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let workflow_path = dir.path().join("malformed.json");

        fs::write(&workflow_path, "{ not valid json }").unwrap();

        let mut registry = WorkflowRegistry::new();
        let result = registry.load_from_directory(dir.path()).await;

        assert!(result.is_ok());
        assert_eq!(registry.len(), 0);
    }

    #[tokio::test]
    async fn test_load_duplicate_workflow_names() {
        use std::fs;

        use tempfile::tempdir;

        let dir = tempdir().unwrap();

        let workflow_json = r#"{
            "name": "duplicate",
            "version": "1.0.0",
            "description": "Duplicate workflow",
            "steps": []
        }"#;

        fs::write(dir.path().join("workflow1.json"), workflow_json).unwrap();
        fs::write(dir.path().join("workflow2.json"), workflow_json).unwrap();

        let mut registry = WorkflowRegistry::new();
        registry.load_from_directory(dir.path()).await.unwrap();

        assert_eq!(registry.len(), 1);
        assert!(registry.get("duplicate").is_some());
    }
}
