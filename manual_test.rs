use std::path::Path;
use serde_json::json;

// Import the skills system
use chat_cli::cli::skills::{SkillRegistry, SkillValidator};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 MANUAL SKILLS SYSTEM TEST");
    println!("============================\n");

    let workspace_path = Path::new("/tmp/test-workspace");
    
    // Test 1: Load skills from workspace
    println!("📁 Test 1: Loading skills from workspace...");
    match SkillRegistry::with_workspace_skills(workspace_path).await {
        Ok(registry) => {
            let skills = registry.list();
            println!("✅ Loaded {} skills", skills.len());
            
            for skill in skills {
                println!("   - {} ({}): {}", skill.name(), skill.aliases().join(", "), skill.description());
            }
        }
        Err(e) => {
            println!("❌ Failed to load skills: {}", e);
            return Err(e.into());
        }
    }
    
    println!();

    // Test 2: Execute echo skill
    println!("⚡ Test 2: Executing echo skill...");
    let registry = SkillRegistry::with_workspace_skills(workspace_path).await?;
    
    if let Some(echo_skill) = registry.get("echo-test") {
        match echo_skill.execute(json!({})).await {
            Ok(result) => {
                println!("✅ Echo skill executed successfully!");
                println!("   Output: {}", result.output);
            }
            Err(e) => {
                println!("❌ Echo skill execution failed: {}", e);
            }
        }
    } else {
        println!("❌ Echo skill not found!");
    }
    
    println!();

    // Test 3: Execute prompt skill with parameters
    println!("📝 Test 3: Executing prompt skill with parameters...");
    
    if let Some(greeting_skill) = registry.get("greeting") {
        let params = json!({
            "name": "Alice",
            "place": "Wonderland",
            "day": "Friday"
        });
        
        match greeting_skill.execute(params).await {
            Ok(result) => {
                println!("✅ Greeting skill executed successfully!");
                println!("   Output: {}", result.output);
            }
            Err(e) => {
                println!("❌ Greeting skill execution failed: {}", e);
            }
        }
        
        // Test UI rendering
        match greeting_skill.render_ui().await {
            Ok(ui) => {
                println!("✅ UI rendered successfully!");
                println!("   Interactive: {}", ui.interactive);
                println!("   Elements: {}", ui.elements.len());
            }
            Err(e) => {
                println!("❌ UI rendering failed: {}", e);
            }
        }
    } else {
        println!("❌ Greeting skill not found!");
    }
    
    println!();

    // Test 4: Test validation on invalid skill
    println!("🔍 Test 4: Testing validation on invalid skill...");
    let invalid_skill_content = std::fs::read_to_string("/tmp/test-workspace/.q-skills/invalid-skill.json")?;
    
    match SkillValidator::validate_skill_json(&invalid_skill_content) {
        Ok(_) => {
            println!("❌ Validation should have failed but didn't!");
        }
        Err(e) => {
            println!("✅ Validation correctly rejected invalid skill!");
            println!("   Error: {}", e);
        }
    }
    
    println!();

    // Test 5: Test skill discovery
    println!("🔎 Test 5: Testing skill discovery...");
    let skills_dir = workspace_path.join(".q-skills");
    let locations = vec![skills_dir.as_path()];
    let discovered = SkillRegistry::discover_skills_in_locations(&locations);
    
    println!("✅ Discovered {} skills from locations", discovered.len());
    for skill_info in discovered {
        println!("   - {}: {} (v{})", skill_info.name, skill_info.description, skill_info.version);
    }
    
    println!();

    // Test 6: Test builtin calculator override
    println!("🧮 Test 6: Testing builtin calculator...");
    if let Some(calculator) = registry.get("calculator") {
        println!("✅ Calculator skill found!");
        println!("   Description: {}", calculator.description());
        println!("   Aliases: {:?}", calculator.aliases());
    } else {
        println!("❌ Calculator skill not found!");
    }
    
    println!();
    println!("🎉 Manual testing completed!");
    
    Ok(())
}
