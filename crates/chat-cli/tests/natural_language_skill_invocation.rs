use chat_cli::cli::chat::tools::{
    Tool,
    ToolManager,
};
use chat_cli::cli::skills::SkillRegistry;
use serde_json::json;

#[tokio::test]
async fn test_skill_discoverable_by_agent() {
    // Verify skills are registered as tools that an agent can discover
    let registry = SkillRegistry::with_builtins();
    let mut tool_manager = ToolManager::default();
    tool_manager.register_skills(&registry);

    // Verify calculator skill is available
    let tools = tool_manager.get_all_tools();
    let calculator_tool = tools.iter().find(|t| t.display_name() == "calculator");

    assert!(
        calculator_tool.is_some(),
        "Calculator skill should be discoverable as a tool"
    );
}

#[tokio::test]
async fn test_skill_invocation_through_tool_interface() {
    // Test that a skill can be invoked through the Tool interface
    let registry = SkillRegistry::with_builtins();
    let mut tool_manager = ToolManager::default();
    tool_manager.register_skills(&registry);

    // Simulate agent invoking calculator skill
    let input = json!({
        "a": 5,
        "b": 3,
        "op": "add"
    });

    let tools = tool_manager.get_all_tools();
    let calculator_tool = tools.iter().find(|t| t.display_name() == "calculator");

    if let Some(Tool::Skill(skill_tool)) = calculator_tool {
        let result = skill_tool.invoke(input).await;
        assert!(result.is_ok(), "Skill invocation should succeed");

        let output = result.unwrap();
        assert!(
            output.to_string().contains("8"),
            "Calculator should return correct result"
        );
    } else {
        panic!("Calculator tool not found or wrong type");
    }
}

#[tokio::test]
async fn test_multiple_skills_registered() {
    // Verify all builtin skills are registered and discoverable
    let registry = SkillRegistry::with_builtins();
    let mut tool_manager = ToolManager::default();
    tool_manager.register_skills(&registry);

    let tools = tool_manager.get_all_tools();
    let skill_tools: Vec<_> = tools.iter().filter(|t| matches!(t, Tool::Skill(_))).collect();

    assert!(!skill_tools.is_empty(), "Should have at least one skill registered");

    // Verify each skill tool has proper metadata
    for tool in skill_tools {
        assert!(!tool.display_name().is_empty(), "Skill should have a name");
    }
}
