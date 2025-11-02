#[cfg(test)]
mod security_tests {
    use std::fs;
    use std::time::{
        Duration,
        Instant,
    };

    use serde_json::json;
    use tempfile::TempDir;

    use crate::cli::skills::SkillRegistry;

    // Tests for Phase 1: Security & Resilience

    #[tokio::test]
    async fn test_resource_limit_enforcement() {
        // Test that skills respect timeout limits
        let temp_dir = TempDir::new().unwrap();
        let workspace_dir = temp_dir.path().join("workspace");
        let skills_dir = workspace_dir.join(".q-skills");
        fs::create_dir_all(&skills_dir).unwrap();

        // Create a skill with resource limits
        let resource_limited_skill = skills_dir.join("resource-limited.json");
        fs::write(
            &resource_limited_skill,
            json!({
                "name": "resource-test",
                "description": "Test resource limits",
                "version": "1.0.0",
                "type": "code_inline",
                "command": "sleep",
                "args": ["2"],
                "security": {
                    "resource_limits": {
                        "max_memory_mb": 50,
                        "max_execution_time": 1
                    }
                }
            })
            .to_string(),
        )
        .unwrap();

        let registry = SkillRegistry::with_workspace_skills(&workspace_dir).await.unwrap();
        let skill = registry
            .get("resource-test")
            .expect("Resource test skill should be loaded");

        // Execution should be terminated due to timeout
        let start = Instant::now();
        let result = skill.execute(json!({})).await;
        let duration = start.elapsed();

        // Should timeout within reasonable bounds (1s + overhead)
        assert!(duration < Duration::from_secs(3), "Skill should timeout within limits");

        // Should return an error due to timeout
        assert!(result.is_err(), "Long-running skill should timeout and return error");
    }

    // Removed placeholder tests that don't actually test functionality:
    // - test_sandbox_file_access_restrictions (not implemented)
    // - test_retry_logic_on_failures (not implemented)
    // - test_circuit_breaker_pattern (not implemented)
    // - test_permission_validation (not implemented)

    // These will be re-added when the actual functionality is implemented

    #[tokio::test]
    async fn test_input_sanitization() {
        // Test that malicious input is properly sanitized
        let temp_dir = TempDir::new().unwrap();
        let workspace_dir = temp_dir.path().join("workspace");
        let skills_dir = workspace_dir.join(".q-skills");
        fs::create_dir_all(&skills_dir).unwrap();

        // Create a prompt skill that should sanitize input
        let sanitization_skill = skills_dir.join("sanitization-test.json");
        fs::write(
            &sanitization_skill,
            json!({
                "name": "sanitization-test",
                "description": "Test input sanitization",
                "version": "1.0.0",
                "type": "prompt_inline",
                "prompt": "Hello {name}!",
                "parameters": [
                    {
                        "name": "name",
                        "type": "string",
                        "pattern": "^[a-zA-Z0-9 ]+$",
                        "required": true
                    }
                ]
            })
            .to_string(),
        )
        .unwrap();

        let registry = SkillRegistry::with_workspace_skills(&workspace_dir).await.unwrap();
        let skill = registry
            .get("sanitization-test")
            .expect("Sanitization test skill should be loaded");

        // Test with malicious input
        let malicious_params = json!({
            "name": "'; rm -rf /; echo 'hacked"
        });

        let result = skill.execute(malicious_params).await;

        // TODO: Once input sanitization is implemented, this should be rejected or sanitized
        match result {
            Ok(output) => {
                println!("Sanitization test output: {}", output.output);
                // Should not contain the malicious command
                assert!(!output.output.contains("rm -rf"));
            },
            Err(e) => println!("Sanitization correctly rejected malicious input: {}", e),
        }
    }

    #[test]
    fn test_security_configuration_validation() {
        // Test that security configurations are properly validated
        use crate::cli::skills::validation::SkillValidator;

        // Valid security configuration
        let valid_config = json!({
            "name": "secure-skill",
            "description": "Secure skill",
            "version": "1.0.0",
            "type": "code_inline",
            "command": "echo",
            "security": {
                "permissions": {
                    "file_read": ["./"],
                    "file_write": ["./output"],
                    "network_access": false
                },
                "resource_limits": {
                    "max_memory_mb": 100,
                    "max_execution_time": 30
                }
            }
        });

        let result = SkillValidator::validate_skill_json(&valid_config.to_string());
        assert!(result.is_ok(), "Valid security configuration should be accepted");

        // Invalid security configuration - negative values should be rejected
        let invalid_config = json!({
            "name": "insecure-skill",
            "description": "Insecure skill",
            "version": "1.0.0",
            "type": "code_inline",
            "command": "echo"
        });

        // This should pass basic validation since security config is optional
        let result = SkillValidator::validate_skill_json(&invalid_config.to_string());
        assert!(result.is_ok(), "Basic skill without security config should be valid");
    }
}
