use chat_cli::cli::chat::tools::{Tool, ToolOrigin};
use chat_cli::cli::skills::SkillRegistry;
use chat_cli::os::Os;

#[tokio::test]
async fn test_skill_invocation_via_natural_language() {
    // Simulate LLM requesting skill execution
    let os = Os::new().await.unwrap();
    let registry = SkillRegistry::with_builtins();

    // Get calculator skill as ToolSpec
    let toolspec = registry.get_toolspec("calculator");
    assert!(toolspec.is_some(), "Calculator skill should be available as ToolSpec");

    let toolspec = toolspec.unwrap();
    assert_eq!(toolspec.name, "calculator");
    assert!(matches!(toolspec.tool_origin, ToolOrigin::Skill(_)));

    // Verify schema is valid
    let schema = &toolspec.input_schema.0;
    assert!(schema["properties"]["a"].is_object());
    assert!(schema["properties"]["b"].is_object());
    assert!(schema["properties"]["op"].is_object());
}

#[tokio::test]
async fn test_skill_execution_through_tool_enum() {
    let os = Os::new().await.unwrap();
    let agents = chat_cli::cli::agent::Agents::default();

    // Create a skill tool
    let skill_tool = chat_cli::cli::chat::tools::skill_tool::SkillTool::new(
        "calculator".to_string(),
        serde_json::json!({
            "a": 10.0,
            "b": 5.0,
            "op": "add"
        }),
    );

    let tool = Tool::Skill(skill_tool);
    let mut output = Vec::new();
    let mut line_tracker = std::collections::HashMap::new();

    // Execute the tool
    let result = tool.invoke(&os, &mut output, &mut line_tracker, &agents).await;
    assert!(result.is_ok(), "Skill execution should succeed");

    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("15"), "Output should contain result");
}

#[tokio::test]
async fn test_skill_not_found_error_handling() {
    let os = Os::new().await.unwrap();
    let agents = chat_cli::cli::agent::Agents::default();

    let skill_tool = chat_cli::cli::chat::tools::skill_tool::SkillTool::new(
        "nonexistent-skill".to_string(),
        serde_json::json!({}),
    );

    let tool = Tool::Skill(skill_tool);
    let mut output = Vec::new();
    let mut line_tracker = std::collections::HashMap::new();

    let result = tool.invoke(&os, &mut output, &mut line_tracker, &agents).await;
    assert!(result.is_err(), "Should error for nonexistent skill");
    assert!(result.unwrap_err().to_string().contains("Skill not found"));
}

#[tokio::test]
async fn test_concurrent_skill_invocations() {
    let os = Os::new().await.unwrap();

    // Create multiple skill invocations
    let tasks: Vec<_> = (0..3)
        .map(|i| {
            let os_clone = os.clone();
            tokio::spawn(async move {
                let agents = chat_cli::cli::agent::Agents::default();
                let skill_tool = chat_cli::cli::chat::tools::skill_tool::SkillTool::new(
                    "calculator".to_string(),
                    serde_json::json!({
                        "a": i as f64,
                        "b": 1.0,
                        "op": "add"
                    }),
                );

                let tool = Tool::Skill(skill_tool);
                let mut output = Vec::new();
                let mut line_tracker = std::collections::HashMap::new();

                tool.invoke(&os_clone, &mut output, &mut line_tracker, &agents).await
            })
        })
        .collect();

    // Wait for all tasks
    for task in tasks {
        let result = task.await.unwrap();
        assert!(result.is_ok(), "Concurrent skill execution should succeed");
    }
}
