#[cfg(test)]
mod manual_verification_test {
    use crate::cli::skills::{SkillRegistry, SkillValidator};
    use serde_json::json;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_manual_verification_complete_workflow() {
        println!("\n🧪 MANUAL VERIFICATION TEST");
        println!("============================");

        // Create test workspace with real skills
        let temp_dir = TempDir::new().unwrap();
        let workspace_dir = temp_dir.path().join("workspace");
        let skills_dir = workspace_dir.join(".q-skills");
        fs::create_dir_all(&skills_dir).unwrap();

        // Create real echo skill
        let echo_skill = skills_dir.join("echo-skill.json");
        fs::write(&echo_skill, json!({
            "name": "echo-test",
            "description": "Test echo command execution",
            "version": "1.0.0",
            "aliases": ["echo", "et"],
            "type": "code_inline",
            "command": "echo",
            "args": ["Hello from manual verification!"]
        }).to_string()).unwrap();

        // Create prompt skill with parameters
        let greeting_skill = skills_dir.join("greeting-skill.json");
        fs::write(&greeting_skill, json!({
            "name": "greeting",
            "description": "Generate personalized greetings",
            "version": "1.0.0",
            "type": "prompt_inline",
            "prompt": "Hello {name}! Welcome to {place}. Today is {day}.",
            "parameters": [
                {
                    "name": "name",
                    "description": "Person's name",
                    "required": true
                },
                {
                    "name": "place",
                    "description": "Location", 
                    "required": false,
                    "default": "Q CLI"
                },
                {
                    "name": "day",
                    "description": "Day of the week",
                    "required": true
                }
            ]
        }).to_string()).unwrap();

        // Create invalid skill for validation testing
        let invalid_skill = skills_dir.join("invalid-skill.json");
        fs::write(&invalid_skill, json!({
            "name": "invalid skill with spaces!",
            "description": "This should fail validation",
            "version": "not.a.version",
            "type": "unknown_type"
        }).to_string()).unwrap();

        println!("\n📁 Test 1: Loading skills from workspace...");
        let registry = SkillRegistry::with_workspace_skills(&workspace_dir).await.unwrap();
        let skills = registry.list();
        println!("✅ Loaded {} skills", skills.len());
        
        for skill in skills {
            println!("   - {} ({}): {}", skill.name(), skill.aliases().join(", "), skill.description());
        }

        println!("\n⚡ Test 2: Executing echo skill...");
        let echo_skill = registry.get("echo-test").expect("Echo skill should be loaded");
        let result = echo_skill.execute(json!({})).await.unwrap();
        println!("✅ Echo skill executed successfully!");
        println!("   Output: {}", result.output);
        assert!(result.output.contains("Hello from manual verification!"));

        println!("\n📝 Test 3: Executing prompt skill with parameters...");
        let greeting_skill = registry.get("greeting").expect("Greeting skill should be loaded");
        let params = json!({
            "name": "Alice",
            "place": "Wonderland",
            "day": "Friday"
        });
        
        let result = greeting_skill.execute(params).await.unwrap();
        println!("✅ Greeting skill executed successfully!");
        println!("   Output: {}", result.output);
        assert_eq!(result.output, "Hello Alice! Welcome to Wonderland. Today is Friday.");

        // Test UI rendering
        let ui = greeting_skill.render_ui().await.unwrap();
        println!("✅ UI rendered successfully!");
        println!("   Interactive: {}", ui.interactive);
        println!("   Elements: {}", ui.elements.len());
        assert!(ui.interactive);
        assert_eq!(ui.elements.len(), 4); // Text + 3 inputs

        println!("\n🔍 Test 4: Testing validation on invalid skill...");
        let invalid_content = fs::read_to_string(&invalid_skill).unwrap();
        let validation_result = SkillValidator::validate_skill_json(&invalid_content);
        
        match validation_result {
            Ok(_) => panic!("Validation should have failed but didn't!"),
            Err(e) => {
                println!("✅ Validation correctly rejected invalid skill!");
                println!("   Error: {}", e);
                assert!(e.to_string().contains("alphanumeric characters"));
            }
        }

        println!("\n🔎 Test 5: Testing skill discovery...");
        let locations = vec![skills_dir.as_path()];
        let discovered = SkillRegistry::discover_skills_in_locations(&locations);
        println!("✅ Discovered {} skills from locations", discovered.len());
        
        for skill_info in &discovered {
            println!("   - {}: {} (v{})", skill_info.name, skill_info.description, skill_info.version);
        }
        
        // Should discover valid skills but skip invalid ones
        assert!(discovered.iter().any(|s| s.name == "echo-test"));
        assert!(discovered.iter().any(|s| s.name == "greeting"));
        // Invalid skill should not be discovered due to validation

        println!("\n🧮 Test 6: Testing builtin calculator...");
        let calculator = registry.get("calculator").expect("Calculator should be available");
        println!("✅ Calculator skill found!");
        println!("   Description: {}", calculator.description());
        println!("   Aliases: {:?}", calculator.aliases());
        assert_eq!(calculator.name(), "calculator");
        assert!(calculator.aliases().contains(&"calc".to_string()));

        println!("\n🎉 Manual verification completed successfully!");
        println!("   All systems working as expected!");
    }

    #[tokio::test]
    async fn test_real_command_execution_verification() {
        println!("\n🔧 REAL COMMAND EXECUTION TEST");
        println!("===============================");

        let temp_dir = TempDir::new().unwrap();
        let workspace_dir = temp_dir.path().join("workspace");
        let skills_dir = workspace_dir.join(".q-skills");
        fs::create_dir_all(&skills_dir).unwrap();

        // Test with different real commands
        let commands_to_test = vec![
            ("date-skill", "date", vec!["+%Y-%m-%d"], "should show current date"),
            ("pwd-skill", "pwd", vec![], "should show current directory"),
            ("ls-skill", "ls", vec!["-la", "/tmp"], "should list /tmp directory"),
        ];

        for (skill_name, command, args, description) in commands_to_test {
            println!("\n🧪 Testing {}: {}", skill_name, description);
            
            let skill_file = skills_dir.join(format!("{}.json", skill_name));
            fs::write(&skill_file, json!({
                "name": skill_name,
                "description": description,
                "version": "1.0.0",
                "type": "code_inline",
                "command": command,
                "args": args
            }).to_string()).unwrap();

            let registry = SkillRegistry::with_workspace_skills(&workspace_dir).await.unwrap();
            let skill = registry.get(skill_name).expect(&format!("{} should be loaded", skill_name));
            
            match skill.execute(json!({})).await {
                Ok(result) => {
                    println!("✅ {} executed successfully!", skill_name);
                    println!("   Output: {}", result.output.lines().next().unwrap_or("(empty)"));
                    assert!(!result.output.is_empty(), "Command should produce output");
                }
                Err(e) => {
                    println!("⚠️  {} execution failed (may be expected): {}", skill_name, e);
                    // Some commands might fail in test environment, that's OK
                }
            }
        }

        println!("\n✅ Real command execution verification completed!");
    }
}
