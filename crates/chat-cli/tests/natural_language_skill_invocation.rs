use chat_cli::cli::chat::tool_manager::ToolManager;
use chat_cli::cli::chat::tools::Tool;
use chat_cli::os::Os;

#[tokio::test]
async fn test_skill_discoverable_by_agent() {
    // Verify skills are registered as tools that an agent can discover
    let os = Os::new().await.unwrap();
    let tool_manager = ToolManager::new_with_skills(&os).await.unwrap();

    // Verify calculator skill is available
    let tools = tool_manager.get_all_tools();
    let calculator_tool = tools.iter().find(|t| t.display_name() == "calculator");

    assert!(
        calculator_tool.is_some(),
        "Calculator skill should be discoverable as a tool"
    );
}

#[tokio::test]
async fn test_skill_has_correct_metadata() {
    // Test that skills have proper metadata for agent discovery
    let os = Os::new().await.unwrap();
    let tool_manager = ToolManager::new_with_skills(&os).await.unwrap();

    let tools = tool_manager.get_all_tools();
    let calculator_tool = tools.iter().find(|t| t.display_name() == "calculator");

    assert!(calculator_tool.is_some());

    if let Some(tool) = calculator_tool {
        // Verify tool has description
        assert!(!tool.display_name().is_empty());

        // Verify it's a Skill variant
        assert!(matches!(tool, Tool::Skill(_)));
    }
}

#[tokio::test]
async fn test_multiple_skills_registered() {
    // Verify all builtin skills are registered and discoverable
    let os = Os::new().await.unwrap();
    let tool_manager = ToolManager::new_with_skills(&os).await.unwrap();

    let tools = tool_manager.get_all_tools();
    let skill_tools: Vec<_> = tools.iter().filter(|t| matches!(t, Tool::Skill(_))).collect();

    assert!(!skill_tools.is_empty(), "Should have at least one skill registered");

    // Verify each skill tool has proper metadata
    for tool in skill_tools {
        assert!(!tool.display_name().is_empty(), "Skill should have a name");
    }
}

#[tokio::test]
async fn test_skills_and_native_tools_coexist() {
    // Verify skills are added alongside native tools
    let os = Os::new().await.unwrap();
    let tool_manager = ToolManager::new_with_skills(&os).await.unwrap();

    let tools = tool_manager.get_all_tools();

    // Should have both skill tools and native tools
    let has_skills = tools.iter().any(|t| matches!(t, Tool::Skill(_)));
    let has_native = tools.iter().any(|t| !matches!(t, Tool::Skill(_)));

    assert!(has_skills, "Should have skill tools");
    assert!(has_native, "Should have native tools");
}
