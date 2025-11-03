# Code Search Tool - Test Specifications

## Test Strategy

### Test Pyramid
- **Unit Tests (70%)**: Fast, isolated, comprehensive coverage
- **Integration Tests (20%)**: Tool integration with Q CLI system  
- **End-to-End Tests (10%)**: Full workflow validation

### Test Categories

## Unit Tests

### 1. Structure & Validation Tests
```rust
#[cfg(test)]
mod structure_tests {
    use super::*;
    
    #[test]
    fn test_code_search_creation_valid() {
        let search = CodeSearch {
            query: "function".to_string(),
            path: Some("./src".to_string()),
            file_types: Some(vec!["rs".to_string(), "py".to_string()]),
            limit: Some(10),
        };
        
        assert_eq!(search.query, "function");
        assert_eq!(search.path, Some("./src".to_string()));
        assert_eq!(search.limit, Some(10));
    }
    
    #[test]
    fn test_code_search_defaults() {
        let search = CodeSearch {
            query: "test".to_string(),
            path: None,
            file_types: None,
            limit: None,
        };
        
        assert_eq!(search.query, "test");
        assert!(search.path.is_none());
        assert!(search.file_types.is_none());
        assert!(search.limit.is_none());
    }
    
    #[tokio::test]
    async fn test_validation_empty_query_fails() {
        let mut search = CodeSearch {
            query: "".to_string(),
            path: None,
            file_types: None,
            limit: None,
        };
        
        let os = create_test_os();
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
        
        let os = create_test_os();
        let result = search.validate(&os).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_validation_nonexistent_path_fails() {
        let mut search = CodeSearch {
            query: "test".to_string(),
            path: Some("/nonexistent/path".to_string()),
            file_types: None,
            limit: None,
        };
        
        let os = create_test_os();
        let result = search.validate(&os).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not exist"));
    }
    
    #[tokio::test]
    async fn test_validation_valid_path_succeeds() {
        let temp_dir = create_temp_dir();
        let mut search = CodeSearch {
            query: "test".to_string(),
            path: Some(temp_dir.path().to_string_lossy().to_string()),
            file_types: None,
            limit: None,
        };
        
        let os = create_test_os();
        let result = search.validate(&os).await;
        assert!(result.is_ok());
    }
}
```

### 2. Permission System Tests
```rust
#[cfg(test)]
mod permission_tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_permission_no_restrictions_allows() {
        let search = CodeSearch {
            query: "test".to_string(),
            path: Some("./src".to_string()),
            file_types: None,
            limit: None,
        };
        
        let agent = create_test_agent_with_settings(json!({}));
        let os = create_test_os();
        
        let result = search.eval_perm(&os, &agent);
        assert_eq!(result, PermissionEvalResult::Allow);
    }
    
    #[test]
    fn test_permission_allowed_paths_permits() {
        let search = CodeSearch {
            query: "test".to_string(),
            path: Some("./src".to_string()),
            file_types: None,
            limit: None,
        };
        
        let agent = create_test_agent_with_settings(json!({
            "allowedPaths": ["./src", "./docs"]
        }));
        let os = create_test_os();
        
        let result = search.eval_perm(&os, &agent);
        assert_eq!(result, PermissionEvalResult::Allow);
    }
    
    #[test]
    fn test_permission_allowed_paths_denies_others() {
        let search = CodeSearch {
            query: "test".to_string(),
            path: Some("./target".to_string()),
            file_types: None,
            limit: None,
        };
        
        let agent = create_test_agent_with_settings(json!({
            "allowedPaths": ["./src", "./docs"]
        }));
        let os = create_test_os();
        
        let result = search.eval_perm(&os, &agent);
        assert_eq!(result, PermissionEvalResult::Ask);
    }
    
    #[test]
    fn test_permission_denied_paths_blocks() {
        let search = CodeSearch {
            query: "test".to_string(),
            path: Some("./target".to_string()),
            file_types: None,
            limit: None,
        };
        
        let agent = create_test_agent_with_settings(json!({
            "deniedPaths": ["./target", "./node_modules"]
        }));
        let os = create_test_os();
        
        let result = search.eval_perm(&os, &agent);
        assert_eq!(result, PermissionEvalResult::Deny);
    }
    
    #[test]
    fn test_permission_denied_overrides_allowed() {
        let search = CodeSearch {
            query: "test".to_string(),
            path: Some("./src/target".to_string()),
            file_types: None,
            limit: None,
        };
        
        let agent = create_test_agent_with_settings(json!({
            "allowedPaths": ["./src"],
            "deniedPaths": ["./src/target"]
        }));
        let os = create_test_os();
        
        let result = search.eval_perm(&os, &agent);
        assert_eq!(result, PermissionEvalResult::Deny);
    }
}
```

### 3. Search Logic Tests
```rust
#[cfg(test)]
mod search_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_ripgrep_basic_search() {
        let temp_dir = create_test_workspace();
        create_test_file(&temp_dir, "test.rs", "fn hello() { println!(\"world\"); }");
        create_test_file(&temp_dir, "main.rs", "fn main() { hello(); }");
        
        let search = CodeSearch {
            query: "hello".to_string(),
            path: Some(temp_dir.path().to_string_lossy().to_string()),
            file_types: None,
            limit: None,
        };
        
        let os = create_test_os();
        let results = search.ripgrep_search(temp_dir.path(), &os).await.unwrap();
        
        assert_eq!(results.len(), 2);
        assert!(results.iter().any(|r| r.file_path.ends_with("test.rs")));
        assert!(results.iter().any(|r| r.file_path.ends_with("main.rs")));
    }
    
    #[tokio::test]
    async fn test_ripgrep_file_type_filter() {
        let temp_dir = create_test_workspace();
        create_test_file(&temp_dir, "test.rs", "fn hello() {}");
        create_test_file(&temp_dir, "test.py", "def hello(): pass");
        create_test_file(&temp_dir, "test.js", "function hello() {}");
        
        let search = CodeSearch {
            query: "hello".to_string(),
            path: Some(temp_dir.path().to_string_lossy().to_string()),
            file_types: Some(vec!["rs".to_string()]),
            limit: None,
        };
        
        let os = create_test_os();
        let results = search.ripgrep_search(temp_dir.path(), &os).await.unwrap();
        
        assert_eq!(results.len(), 1);
        assert!(results[0].file_path.ends_with("test.rs"));
    }
    
    #[tokio::test]
    async fn test_ripgrep_limit_results() {
        let temp_dir = create_test_workspace();
        for i in 0..10 {
            create_test_file(&temp_dir, &format!("test{}.rs", i), "fn hello() {}");
        }
        
        let search = CodeSearch {
            query: "hello".to_string(),
            path: Some(temp_dir.path().to_string_lossy().to_string()),
            file_types: None,
            limit: Some(5),
        };
        
        let os = create_test_os();
        let results = search.ripgrep_search(temp_dir.path(), &os).await.unwrap();
        
        assert!(results.len() <= 5);
    }
    
    #[tokio::test]
    async fn test_ripgrep_no_results() {
        let temp_dir = create_test_workspace();
        create_test_file(&temp_dir, "test.rs", "fn goodbye() {}");
        
        let search = CodeSearch {
            query: "hello".to_string(),
            path: Some(temp_dir.path().to_string_lossy().to_string()),
            file_types: None,
            limit: None,
        };
        
        let os = create_test_os();
        let results = search.ripgrep_search(temp_dir.path(), &os).await.unwrap();
        
        assert_eq!(results.len(), 0);
    }
    
    #[tokio::test]
    async fn test_ripgrep_missing_command_error() {
        // Test graceful handling when ripgrep is not installed
        // This test should be conditional based on environment
    }
}
```

### 4. Output Formatting Tests
```rust
#[cfg(test)]
mod formatting_tests {
    use super::*;
    
    #[test]
    fn test_format_results_empty() {
        let results = vec![];
        let search = CodeSearch {
            query: "test".to_string(),
            path: None,
            file_types: None,
            limit: None,
        };
        
        let formatted = search.format_results(results);
        assert!(formatted.contains("No results found"));
        assert!(formatted.contains("test"));
    }
    
    #[test]
    fn test_format_results_single() {
        let results = vec![SearchResult {
            file_path: PathBuf::from("src/main.rs"),
            line_number: 42,
            line_content: "fn hello() { println!(\"world\"); }".to_string(),
            match_context: None,
        }];
        
        let search = CodeSearch {
            query: "hello".to_string(),
            path: None,
            file_types: None,
            limit: None,
        };
        
        let formatted = search.format_results(results);
        assert!(formatted.contains("Found 1 results"));
        assert!(formatted.contains("src/main.rs:42"));
        assert!(formatted.contains("fn hello()"));
    }
    
    #[test]
    fn test_format_results_multiple() {
        let results = vec![
            SearchResult {
                file_path: PathBuf::from("src/main.rs"),
                line_number: 42,
                line_content: "fn hello() {}".to_string(),
                match_context: None,
            },
            SearchResult {
                file_path: PathBuf::from("src/lib.rs"),
                line_number: 15,
                line_content: "pub fn hello() {}".to_string(),
                match_context: None,
            },
        ];
        
        let search = CodeSearch {
            query: "hello".to_string(),
            path: None,
            file_types: None,
            limit: None,
        };
        
        let formatted = search.format_results(results);
        assert!(formatted.contains("Found 2 results"));
        assert!(formatted.contains("src/main.rs:42"));
        assert!(formatted.contains("src/lib.rs:15"));
    }
    
    #[test]
    fn test_format_results_with_limit() {
        let results = (0..25).map(|i| SearchResult {
            file_path: PathBuf::from(format!("src/test{}.rs", i)),
            line_number: i + 1,
            line_content: "fn hello() {}".to_string(),
            match_context: None,
        }).collect();
        
        let search = CodeSearch {
            query: "hello".to_string(),
            path: None,
            file_types: None,
            limit: Some(20),
        };
        
        let formatted = search.format_results(results);
        assert!(formatted.contains("... and 5 more results"));
    }
}
```

## Integration Tests

### 1. Tool Registration Tests
```rust
// tests/integration/code_search_registration.rs
#[tokio::test]
async fn test_tool_registration_in_system() {
    let tool_json = r#"{
        "query": "test function",
        "path": "./src",
        "file_types": ["rs", "py"],
        "limit": 10
    }"#;
    
    // Test tool can be deserialized from JSON
    let tool: CodeSearch = serde_json::from_str(tool_json).unwrap();
    assert_eq!(tool.query, "test function");
    
    // Test tool appears in tool manager
    // This requires integration with actual tool manager
}

#[tokio::test]
async fn test_tool_schema_validation() {
    // Test that tool schema in tool_index.json is valid
    // Test required/optional parameters work correctly
}
```

### 2. Agent Integration Tests
```rust
// tests/integration/code_search_agent.rs
#[tokio::test]
async fn test_agent_configuration_respected() {
    let agent_config = r#"{
        "toolsSettings": {
            "code_search": {
                "allowedPaths": ["./src"],
                "deniedPaths": ["./target"],
                "maxResults": 5
            }
        }
    }"#;
    
    // Test that agent configuration is properly loaded and respected
}

#[tokio::test]
async fn test_tool_in_chat_session() {
    // Test tool works in actual chat session context
    // This is the most important integration test
}
```

## End-to-End Tests

### 1. Workflow Tests
```rust
// tests/e2e/code_search_workflow.rs
#[tokio::test]
async fn test_complete_search_workflow() {
    // 1. Create test repository
    let test_repo = create_test_repository();
    
    // 2. Start chat session
    let mut session = create_test_chat_session();
    
    // 3. Execute search via LLM
    let response = session.send_message("Find all functions named 'authenticate'").await;
    
    // 4. Verify search was executed and results returned
    assert!(response.contains("authenticate"));
    assert!(response.contains("📁")); // File indicator
}

#[tokio::test]
async fn test_permission_workflow() {
    // Test complete permission checking workflow
}

#[tokio::test]
async fn test_error_handling_workflow() {
    // Test error scenarios end-to-end
}
```

## Performance Tests

### 1. Benchmark Tests
```rust
// benches/code_search_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_search_small_repo(c: &mut Criterion) {
    let test_repo = create_small_test_repo(); // ~100 files
    
    c.bench_function("search_small_repo", |b| {
        b.iter(|| {
            let search = CodeSearch {
                query: black_box("function".to_string()),
                path: Some(test_repo.path().to_string_lossy().to_string()),
                file_types: None,
                limit: Some(20),
            };
            // Benchmark search execution
        })
    });
}

fn bench_search_large_repo(c: &mut Criterion) {
    let test_repo = create_large_test_repo(); // ~1000 files
    
    c.bench_function("search_large_repo", |b| {
        b.iter(|| {
            // Benchmark search on larger repository
        })
    });
}

criterion_group!(benches, bench_search_small_repo, bench_search_large_repo);
criterion_main!(benches);
```

## Test Utilities

### Helper Functions
```rust
// tests/common/mod.rs
use tempfile::TempDir;
use std::fs;
use std::path::Path;

pub fn create_test_os() -> Os {
    // Create test OS instance
}

pub fn create_test_agent_with_settings(settings: serde_json::Value) -> Agent {
    // Create test agent with specific tool settings
}

pub fn create_test_workspace() -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    // Set up basic workspace structure
    temp_dir
}

pub fn create_test_file(dir: &TempDir, filename: &str, content: &str) {
    let file_path = dir.path().join(filename);
    fs::write(file_path, content).unwrap();
}

pub fn create_test_repository() -> TempDir {
    let temp_dir = create_test_workspace();
    
    // Create realistic test repository structure
    create_test_file(&temp_dir, "src/main.rs", r#"
        fn main() {
            println!("Hello, world!");
        }
        
        fn authenticate(token: &str) -> bool {
            !token.is_empty()
        }
    "#);
    
    create_test_file(&temp_dir, "src/lib.rs", r#"
        pub fn authenticate_user(username: &str, password: &str) -> Result<User, AuthError> {
            // Authentication logic
        }
    "#);
    
    create_test_file(&temp_dir, "tests/auth_tests.rs", r#"
        #[test]
        fn test_authenticate() {
            assert!(authenticate("valid_token"));
        }
    "#);
    
    temp_dir
}
```

## Test Execution Strategy

### Continuous Testing
```bash
# Run during development
cargo test code_search --lib
cargo test --test integration
cargo clippy -- -D warnings

# Pre-commit validation
cargo test --all
cargo bench --no-run  # Ensure benchmarks compile
cargo doc --no-deps
```

### Coverage Requirements
- **Unit Tests**: >90% line coverage
- **Integration Tests**: All major workflows covered
- **Edge Cases**: All error conditions tested
- **Performance**: Regression tests for response time

### Test Data Management
- Use temporary directories for all file operations
- Clean up test data automatically
- Use realistic but minimal test cases
- Avoid external dependencies in tests

This comprehensive test specification ensures every aspect of the code search tool is thoroughly validated before deployment.
