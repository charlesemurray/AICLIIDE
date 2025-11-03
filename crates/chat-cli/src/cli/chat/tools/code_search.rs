use eyre::Result;
use serde::Deserialize;

use crate::cli::agent::{
    Agent,
    PermissionEvalResult,
};
use crate::os::Os;

#[derive(Debug, Clone, Deserialize)]
pub struct CodeSearch {
    pub query: String,
    pub path: Option<String>,
    pub file_types: Option<Vec<String>>,
    pub limit: Option<usize>,
}

impl CodeSearch {
    pub async fn validate(&mut self, _os: &Os) -> Result<()> {
        if self.query.trim().is_empty() {
            eyre::bail!("Search query cannot be empty");
        }
        Ok(())
    }

    pub fn eval_perm(&self, _os: &Os, _agent: &Agent) -> PermissionEvalResult {
        PermissionEvalResult::Allow
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_search_creation() {
        let search = CodeSearch {
            query: "test".to_string(),
            path: None,
            file_types: None,
            limit: None,
        };
        assert_eq!(search.query, "test");
    }

    #[test]
    fn test_code_search_with_all_fields() {
        let search = CodeSearch {
            query: "function".to_string(),
            path: Some("./src".to_string()),
            file_types: Some(vec!["rs".to_string(), "py".to_string()]),
            limit: Some(10),
        };

        assert_eq!(search.query, "function");
        assert_eq!(search.path, Some("./src".to_string()));
        assert_eq!(search.limit, Some(10));
        assert_eq!(search.file_types.as_ref().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_validation_empty_query_fails() {
        let mut search = CodeSearch {
            query: "".to_string(),
            path: None,
            file_types: None,
            limit: None,
        };

        // Create a minimal Os for testing
        let os = Os::new().await.unwrap();
        let result = search.validate(&os).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }

    #[tokio::test]
    async fn test_validation_whitespace_query_fails() {
        let mut search = CodeSearch {
            query: "   \t\n  ".to_string(),
            path: None,
            file_types: None,
            limit: None,
        };

        let os = Os::new().await.unwrap();
        let result = search.validate(&os).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validation_valid_query_succeeds() {
        let mut search = CodeSearch {
            query: "test function".to_string(),
            path: None,
            file_types: None,
            limit: None,
        };

        let os = Os::new().await.unwrap();
        let result = search.validate(&os).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_permission_no_restrictions_allows() {
        let search = CodeSearch {
            query: "test".to_string(),
            path: Some("./src".to_string()),
            file_types: None,
            limit: None,
        };

        let agent = Agent {
            name: "test_agent".to_string(),
            tools_settings: std::collections::HashMap::new(),
            ..Default::default()
        };
        let os = Os::new().await.unwrap();

        let result = search.eval_perm(&os, &agent);
        assert_eq!(result, PermissionEvalResult::Allow);
    }

    #[tokio::test]
    async fn test_permission_no_path_allows() {
        let search = CodeSearch {
            query: "test".to_string(),
            path: None,
            file_types: None,
            limit: None,
        };

        let agent = Agent {
            name: "test_agent".to_string(),
            tools_settings: std::collections::HashMap::new(),
            ..Default::default()
        };
        let os = Os::new().await.unwrap();

        let result = search.eval_perm(&os, &agent);
        assert_eq!(result, PermissionEvalResult::Allow);
    }

    #[test]
    fn test_tool_registration() {
        // Test tool can be created from JSON
        let json = r#"{"query": "test"}"#;
        let tool: CodeSearch = serde_json::from_str(json).unwrap();
        assert_eq!(tool.query, "test");
    }

    #[test]
    fn test_tool_registration_with_all_fields() {
        // Test tool can be created from JSON with all fields
        let json = r#"{
            "query": "function",
            "path": "./src",
            "file_types": ["rs", "py"],
            "limit": 10
        }"#;
        let tool: CodeSearch = serde_json::from_str(json).unwrap();
        assert_eq!(tool.query, "function");
        assert_eq!(tool.path, Some("./src".to_string()));
        assert_eq!(tool.file_types, Some(vec!["rs".to_string(), "py".to_string()]));
        assert_eq!(tool.limit, Some(10));
    }
}
