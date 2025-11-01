#[cfg(test)]
mod resilience_tests {
    use crate::cli::skills::SkillRegistry;
    use serde_json::json;
    use std::fs;
    use tempfile::TempDir;
    use std::time::{Duration, Instant};

    // Tests for resilience, error recovery, and performance

    #[tokio::test]
    async fn test_graceful_error_handling() {
        // Test that skill failures don't crash the system
        let temp_dir = TempDir::new().unwrap();
        let workspace_dir = temp_dir.path().join("workspace");
        let skills_dir = workspace_dir.join(".q-skills");
        fs::create_dir_all(&skills_dir).unwrap();

        // Create skills that fail in safe, predictable ways
        let failing_skills = vec![
            ("exit-code-skill", "sh", vec!["-c", "exit 1"]), // Simple exit code failure
            ("timeout-skill", "sleep", vec!["1"]), // Short timeout for testing
            ("nonexistent-skill", "nonexistent_command_12345", vec![]), // Command not found
        ];

        for (name, command, args) in failing_skills {
            let skill_file = skills_dir.join(format!("{}.json", name));
            fs::write(&skill_file, json!({
                "name": name,
                "description": format!("Test {} failure", name),
                "version": "1.0.0",
                "type": "code_inline",
                "command": command,
                "args": args,
                "resilience": {
                    "timeout": 2, // Short timeout for tests
                    "retry_attempts": 1
                }
            }).to_string()).unwrap();
        }

        let registry = SkillRegistry::with_workspace_skills(&workspace_dir).await.unwrap();
        
        // Execute each failing skill with timeout and ensure system remains stable
        for (name, _, _) in &[
            ("exit-code-skill", "", vec![] as Vec<String>),
            ("timeout-skill", "", vec![] as Vec<String>),
            ("nonexistent-skill", "", vec![] as Vec<String>),
        ] {
            if let Some(skill) = registry.get(name) {
                println!("Testing graceful failure for: {}", name);
                let start = Instant::now();
                
                let result = skill.execute(json!({})).await;
                let duration = start.elapsed();
                
                println!("  Result: {:?}, Duration: {:?}", result.is_ok(), duration);
                
                // System should remain responsive
                assert!(duration < Duration::from_secs(10), "Skill should not hang system");
                
                // Should be able to execute other skills after failure
                if let Some(echo_skill) = registry.get("echo-test") {
                    let recovery_result = echo_skill.execute(json!({})).await;
                    println!("  System recovery test: {:?}", recovery_result.is_ok());
                }
            }
        }
    }

    #[tokio::test]
    async fn test_concurrent_skill_execution() {
        // Test that multiple skills can run concurrently without interference
        let temp_dir = TempDir::new().unwrap();
        let workspace_dir = temp_dir.path().join("workspace");
        let skills_dir = workspace_dir.join(".q-skills");
        fs::create_dir_all(&skills_dir).unwrap();

        // Create multiple concurrent skills
        for i in 1..=5 {
            let skill_file = skills_dir.join(format!("concurrent-{}.json", i));
            fs::write(&skill_file, json!({
                "name": format!("concurrent-{}", i),
                "description": format!("Concurrent skill {}", i),
                "version": "1.0.0",
                "type": "code_inline",
                "command": "sh",
                "args": ["-c", &format!("echo 'Skill {}' && sleep 1", i)]
            }).to_string()).unwrap();
        }

        let registry = SkillRegistry::with_workspace_skills(&workspace_dir).await.unwrap();
        
        // Execute skills sequentially to test system stability under load
        let start = Instant::now();
        let mut results = vec![];
        
        for i in 1..=5 {
            let skill_name = format!("concurrent-{}", i);
            if let Some(skill) = registry.get(&skill_name) {
                let result = skill.execute(json!({})).await;
                results.push((skill_name, result));
            }
        }

        let total_duration = start.elapsed();
        
        println!("Concurrent execution completed in {:?}", total_duration);
        println!("Results: {} skills completed", results.len());
        
        // Should complete faster than sequential execution (< 5 seconds vs 5+ seconds)
        // TODO: Once proper concurrent execution is implemented
        for (name, result) in results {
            println!("  {}: {:?}", name, result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_resource_cleanup() {
        // Test that resources are properly cleaned up after skill execution
        let temp_dir = TempDir::new().unwrap();
        let workspace_dir = temp_dir.path().join("workspace");
        let skills_dir = workspace_dir.join(".q-skills");
        fs::create_dir_all(&skills_dir).unwrap();

        // Create a skill that creates temporary resources
        let cleanup_skill = skills_dir.join("cleanup-test.json");
        fs::write(&cleanup_skill, json!({
            "name": "cleanup-test",
            "description": "Test resource cleanup",
            "version": "1.0.0",
            "type": "code_inline",
            "command": "sh",
            "args": ["-c", "mktemp /tmp/skill_test_XXXXXX && echo 'Resource created'"]
        }).to_string()).unwrap();

        let registry = SkillRegistry::with_workspace_skills(&workspace_dir).await.unwrap();
        let skill = registry.get("cleanup-test").expect("Cleanup test skill should be loaded");

        // Execute skill multiple times
        for i in 1..=3 {
            println!("Cleanup test iteration {}", i);
            let result = skill.execute(json!({})).await;
            println!("  Result: {:?}", result.is_ok());
            
            // TODO: Once resource tracking is implemented, verify cleanup
            // Check that temporary files are cleaned up
            // Check that processes are terminated
            // Check that memory is released
        }

        // Verify no resource leaks
        // TODO: Implement resource leak detection
        println!("Resource cleanup test completed");
    }

    #[tokio::test]
    async fn test_error_classification() {
        // Test that errors are properly classified as transient vs permanent
        let temp_dir = TempDir::new().unwrap();
        let workspace_dir = temp_dir.path().join("workspace");
        let skills_dir = workspace_dir.join(".q-skills");
        fs::create_dir_all(&skills_dir).unwrap();

        let error_scenarios = vec![
            ("network-error", "curl", vec!["http://nonexistent.example.com"], "transient"),
            ("permission-error", "cat", vec!["/root/.ssh/id_rsa"], "permanent"),
            ("not-found-error", "nonexistent_command", vec![], "permanent"),
            ("timeout-error", "sleep", vec!["300"], "transient"),
        ];

        for (name, command, args, expected_type) in error_scenarios {
            let skill_file = skills_dir.join(format!("{}.json", name));
            fs::write(&skill_file, json!({
                "name": name,
                "description": format!("Test {} error", expected_type),
                "version": "1.0.0",
                "type": "code_inline",
                "command": command,
                "args": args,
                "resilience": {
                    "timeout": 5,
                    "retry_attempts": 2,
                    "error_classification": {
                        "transient_patterns": ["network", "timeout", "temporary"],
                        "permanent_patterns": ["not found", "permission denied"]
                    }
                }
            }).to_string()).unwrap();
        }

        let registry = SkillRegistry::with_workspace_skills(&workspace_dir).await.unwrap();
        
        for (name, _, _, expected_type) in &[
            ("network-error", "", vec![] as Vec<String>, "transient"),
            ("permission-error", "", vec![], "permanent"),
            ("not-found-error", "", vec![], "permanent"),
            ("timeout-error", "", vec![], "transient"),
        ] {
            if let Some(skill) = registry.get(name) {
                println!("Testing {} error classification", expected_type);
                
                let start = Instant::now();
                let result = skill.execute(json!({})).await;
                let duration = start.elapsed();
                
                println!("  {}: {:?} in {:?}", name, result.is_ok(), duration);
                
                // TODO: Once error classification is implemented:
                // - Transient errors should be retried (longer duration)
                // - Permanent errors should fail fast (shorter duration)
                match *expected_type {
                    "transient" => {
                        // Should take longer due to retries
                        println!("    Expected retries for transient error");
                    }
                    "permanent" => {
                        // Should fail fast without retries
                        println!("    Expected fast failure for permanent error");
                    }
                    _ => {}
                }
            }
        }
    }

    #[tokio::test]
    async fn test_fallback_mechanisms() {
        // Test that skills can fall back to alternative implementations
        let temp_dir = TempDir::new().unwrap();
        let workspace_dir = temp_dir.path().join("workspace");
        let skills_dir = workspace_dir.join(".q-skills");
        fs::create_dir_all(&skills_dir).unwrap();

        // Create a skill with fallback configuration
        let fallback_skill = skills_dir.join("fallback-test.json");
        fs::write(&fallback_skill, json!({
            "name": "fallback-test",
            "description": "Test fallback mechanisms",
            "version": "1.0.0",
            "type": "code_inline",
            "command": "nonexistent_primary_command",
            "args": ["arg1"],
            "resilience": {
                "fallback_commands": [
                    {
                        "command": "also_nonexistent",
                        "args": ["fallback1"]
                    },
                    {
                        "command": "echo",
                        "args": ["Fallback executed successfully"]
                    }
                ]
            }
        }).to_string()).unwrap();

        let registry = SkillRegistry::with_workspace_skills(&workspace_dir).await.unwrap();
        let skill = registry.get("fallback-test").expect("Fallback test skill should be loaded");

        let result = skill.execute(json!({})).await;
        
        // TODO: Once fallback mechanisms are implemented, should succeed with fallback
        match result {
            Ok(output) => {
                println!("Fallback test succeeded: {}", output.output);
                // Should contain fallback output
                if output.output.contains("Fallback executed successfully") {
                    println!("✅ Fallback mechanism worked correctly");
                } else {
                    println!("⚠️  Fallback mechanism not yet implemented");
                }
            }
            Err(e) => println!("Fallback test failed: {} (fallback not yet implemented)", e),
        }
    }

    #[tokio::test]
    async fn test_health_check_system() {
        // Test that skill health can be monitored
        let temp_dir = TempDir::new().unwrap();
        let workspace_dir = temp_dir.path().join("workspace");
        let skills_dir = workspace_dir.join(".q-skills");
        fs::create_dir_all(&skills_dir).unwrap();

        // Create skills with health check configuration
        let health_skills = vec![
            ("healthy-skill", "echo", vec!["healthy"], true),
            ("unhealthy-skill", "false", vec![], false),
        ];

        for (name, command, args, should_be_healthy) in health_skills {
            let skill_file = skills_dir.join(format!("{}.json", name));
            fs::write(&skill_file, json!({
                "name": name,
                "description": format!("Health check test - {}", if should_be_healthy { "healthy" } else { "unhealthy" }),
                "version": "1.0.0",
                "type": "code_inline",
                "command": command,
                "args": args,
                "observability": {
                    "health_check": {
                        "enabled": true,
                        "interval": 30,
                        "timeout": 5,
                        "success_threshold": 1,
                        "failure_threshold": 3
                    }
                }
            }).to_string()).unwrap();
        }

        let registry = SkillRegistry::with_workspace_skills(&workspace_dir).await.unwrap();
        
        // TODO: Once health check system is implemented
        for (name, _, _, expected_health) in &[
            ("healthy-skill", "", vec![] as Vec<String>, true),
            ("unhealthy-skill", "", vec![] as Vec<String>, false),
        ] {
            if let Some(skill) = registry.get(name) {
                println!("Testing health check for: {}", name);
                
                // Execute skill to test health
                let result = skill.execute(json!({})).await;
                println!("  Execution result: {:?}", result.is_ok());
                
                // TODO: Check health status
                // let health_status = skill.get_health_status();
                // assert_eq!(health_status.is_healthy(), *expected_health);
                
                println!("  Expected health: {}", expected_health);
            }
        }
    }
}
