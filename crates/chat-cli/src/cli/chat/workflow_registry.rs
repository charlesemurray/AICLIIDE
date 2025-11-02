//! Workflow registry for managing workflow definitions

use std::collections::HashMap;

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
}
