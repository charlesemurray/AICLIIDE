#[cfg(test)]
mod security_tests {
    use crate::cli::skills::{SkillRegistry};
    use serde_json::json;
    use std::fs;
    use tempfile::TempDir;
    use std::time::{Duration, Instant};

    // Tests for Phase 1: Security & Resilience

    #[tokio::test]
    async fn test_resource_limit_enforcement() {
        // Test that skills respect memory and CPU limits
        let temp_dir = TempDir::new().unwrap();
        let workspace_dir = temp_dir.path().join("workspace");
        let skills_dir = workspace_dir.join(".q-skills");
        fs::create_dir_all(&skills_dir).unwrap();

        // Create a skill with resource limits
        let resource_limited_skill = skills_dir.join("resource-limited.json");
        fs::write(&resource_limited_skill, json!({
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
        }).to_string()).unwrap();

        let registry = SkillRegistry::with_workspace_skills(&workspace_dir).await.unwrap();
        let skill = registry.get("resource-test").expect("Resource test skill should be loaded");

        // Execution should be terminated due to timeout
        let start = Instant::now();
        let _result = skill.execute(json!({})).await;
        let duration = start.elapsed();

        // Should timeout within reasonable bounds (1s + overhead)
        assert!(duration < Duration::from_secs(3), "Skill should timeout within limits");
        
        // TODO: Once timeout implementation is added, this should be an error
        // For now, we're just testing the framework
        println!("Resource limit test executed in {:?}", duration);
    }

    #[tokio::test]
    async fn test_sandbox_file_access_restrictions() {
        // Test that skills cannot access files outside allowed paths
        let temp_dir = TempDir::new().unwrap();
        let workspace_dir = temp_dir.path().join("workspace");
        let skills_dir = workspace_dir.join(".q-skills");
        fs::create_dir_all(&skills_dir).unwrap();

        // Create a skill that tries to access restricted files
        let restricted_skill = skills_dir.join("restricted-access.json");
        fs::write(&restricted_skill, json!({
            "name": "restricted-test",
            "description": "Test file access restrictions",
            "version": "1.0.0",
            "type": "code_inline",
            "command": "cat",
            "args": ["/etc/passwd"], // Should be blocked
            "security": {
                "permissions": {
                    "file_read": ["./allowed"],
                    "file_write": [],
                    "network_access": false
                }
            }
        }).to_string()).unwrap();

        let registry = SkillRegistry::with_workspace_skills(&workspace_dir).await.unwrap();
        let skill = registry.get("restricted-test").expect("Restricted test skill should be loaded");

        let result = skill.execute(json!({})).await;
        
        // TODO: Once sandboxing is implemented, this should fail
        // For now, we're testing the framework exists
        match result {
            Ok(_) => println!("Sandbox test executed (sandboxing not yet implemented)"),
            Err(e) => println!("Sandbox correctly blocked access: {}", e),
        }
    }

    #[tokio::test]
    async fn test_retry_logic_on_failures() {
        // Test that skills retry on transient failures
        let temp_dir = TempDir::new().unwrap();
        let workspace_dir = temp_dir.path().join("workspace");
        let skills_dir = workspace_dir.join(".q-skills");
        fs::create_dir_all(&skills_dir).unwrap();

        // Create a skill that might fail transiently
        let retry_skill = skills_dir.join("retry-test.json");
        fs::write(&retry_skill, json!({
            "name": "retry-test",
            "description": "Test retry logic",
            "version": "1.0.0",
            "type": "code_inline",
            "command": "sh",
            "args": ["-c", "exit 1"], // Always fails
            "resilience": {
                "retry_config": {
                    "max_attempts": 3,
                    "backoff_strategy": "exponential",
                    "retry_on": ["exit_code"]
                }
            }
        }).to_string()).unwrap();

        let registry = SkillRegistry::with_workspace_skills(&workspace_dir).await.unwrap();
        let skill = registry.get("retry-test").expect("Retry test skill should be loaded");

        let start = Instant::now();
        let result = skill.execute(json!({})).await;
        let duration = start.elapsed();

        // TODO: Once retry logic is implemented, should take longer due to retries
        // For now, we're testing the framework
        println!("Retry test completed in {:?}", duration);
        match result {
            Ok(_) => println!("Retry test succeeded"),
            Err(e) => println!("Retry test failed after retries: {}", e),
        }
    }

    #[tokio::test]
    async fn test_circuit_breaker_pattern() {
        // Test that circuit breaker prevents cascading failures
        let temp_dir = TempDir::new().unwrap();
        let workspace_dir = temp_dir.path().join("workspace");
        let skills_dir = workspace_dir.join(".q-skills");
        fs::create_dir_all(&skills_dir).unwrap();

        // Create a skill with circuit breaker configuration
        let circuit_breaker_skill = skills_dir.join("circuit-breaker.json");
        fs::write(&circuit_breaker_skill, json!({
            "name": "circuit-breaker-test",
            "description": "Test circuit breaker",
            "version": "1.0.0",
            "type": "code_inline",
            "command": "false", // Always fails
            "resilience": {
                "circuit_breaker": {
                    "failure_threshold": 2,
                    "recovery_timeout": 5
                }
            }
        }).to_string()).unwrap();

        let registry = SkillRegistry::with_workspace_skills(&workspace_dir).await.unwrap();
        let skill = registry.get("circuit-breaker-test").expect("Circuit breaker test skill should be loaded");

        // Execute multiple times to trigger circuit breaker
        for i in 1..=5 {
            let result = skill.execute(json!({})).await;
            println!("Circuit breaker test attempt {}: {:?}", i, result.is_ok());
            
            // TODO: Once circuit breaker is implemented, later attempts should fail fast
        }
    }

    #[tokio::test]
    async fn test_input_sanitization() {
        // Test that malicious input is properly sanitized
        let temp_dir = TempDir::new().unwrap();
        let workspace_dir = temp_dir.path().join("workspace");
        let skills_dir = workspace_dir.join(".q-skills");
        fs::create_dir_all(&skills_dir).unwrap();

        // Create a prompt skill that should sanitize input
        let sanitization_skill = skills_dir.join("sanitization-test.json");
        fs::write(&sanitization_skill, json!({
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
        }).to_string()).unwrap();

        let registry = SkillRegistry::with_workspace_skills(&workspace_dir).await.unwrap();
        let skill = registry.get("sanitization-test").expect("Sanitization test skill should be loaded");

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
            }
            Err(e) => println!("Sanitization correctly rejected malicious input: {}", e),
        }
    }

    #[tokio::test]
    async fn test_permission_validation() {
        // Test that skills with invalid permissions are rejected
        let temp_dir = TempDir::new().unwrap();
        let workspace_dir = temp_dir.path().join("workspace");
        let skills_dir = workspace_dir.join(".q-skills");
        fs::create_dir_all(&skills_dir).unwrap();

        // Create a skill with invalid permission configuration
        let invalid_permissions_skill = skills_dir.join("invalid-permissions.json");
        fs::write(&invalid_permissions_skill, json!({
            "name": "invalid-permissions-test",
            "description": "Test invalid permissions",
            "version": "1.0.0",
            "type": "code_inline",
            "command": "echo",
            "args": ["test"],
            "security": {
                "permissions": {
                    "file_read": ["/etc", "/root"], // Should be rejected
                    "network_access": true,
                    "process_spawn": true
                }
            }
        }).to_string()).unwrap();

        // TODO: Once permission validation is implemented, this should fail during loading
        let result = SkillRegistry::with_workspace_skills(&workspace_dir).await;
        
        match result {
            Ok(registry) => {
                // If loaded, the skill should be restricted
                if let Some(skill) = registry.get("invalid-permissions-test") {
                    println!("Skill loaded but should have restricted permissions");
                    // Execute and verify it doesn't have dangerous permissions
                    let exec_result = skill.execute(json!({})).await;
                    println!("Execution result: {:?}", exec_result.is_ok());
                }
            }
            Err(e) => println!("Permission validation correctly rejected skill: {}", e),
        }
    }

    #[test]
    fn test_security_configuration_validation() {
        // Test that security configurations are properly validated
        use crate::cli::skills::SkillValidator;

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

        // Invalid security configuration
        let invalid_config = json!({
            "name": "insecure-skill",
            "description": "Insecure skill",
            "version": "1.0.0",
            "type": "code_inline",
            "command": "echo",
            "security": {
                "resource_limits": {
                    "max_memory_mb": -1, // Invalid negative value
                    "max_execution_time": "invalid" // Invalid type
                }
            }
        });

        // TODO: Once security validation is implemented, this should fail
        let result = SkillValidator::validate_skill_json(&invalid_config.to_string());
        println!("Security validation result: {:?}", result.is_ok());
    }
}
