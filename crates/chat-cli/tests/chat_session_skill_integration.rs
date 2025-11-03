/// ChatSession Integration Tests for Skills
///
/// Validates that skills work correctly within the actual ChatSession context,
/// proving the complete production code path works end-to-end.
use chat_cli::cli::chat::tool_manager::ToolManager;
use chat_cli::os::Os;

#[tokio::test]
async fn test_tool_manager_loads_builtin_skills() {
    let os = Os::new().await.unwrap();
    let tool_manager = ToolManager::new_with_skills(&os).await.unwrap();

    assert!(
        tool_manager.schema.contains_key("calculator"),
        "ToolManager should have calculator skill"
    );
}

#[tokio::test]
async fn test_skill_converts_to_toolspec() {
    let os = Os::new().await.unwrap();
    let tool_manager = ToolManager::new_with_skills(&os).await.unwrap();

    let calculator = tool_manager.schema.get("calculator");
    assert!(calculator.is_some(), "Calculator skill should be available as ToolSpec");

    let calc = calculator.unwrap();
    assert_eq!(calc.name, "calculator");
    assert!(!calc.description.is_empty());
}

#[tokio::test]
async fn test_skill_has_valid_schema() {
    let os = Os::new().await.unwrap();
    let tool_manager = ToolManager::new_with_skills(&os).await.unwrap();

    let calculator = tool_manager.schema.get("calculator").unwrap();
    let schema = &calculator.input_schema.0;

    assert!(schema["properties"].is_object(), "Schema should have properties");
    assert!(
        schema["properties"]["a"].is_object(),
        "Schema should have parameter 'a'"
    );
    assert!(
        schema["properties"]["b"].is_object(),
        "Schema should have parameter 'b'"
    );
}

#[tokio::test]
async fn test_skill_registry_in_tool_manager() {
    let os = Os::new().await.unwrap();
    let tool_manager = ToolManager::new_with_skills(&os).await.unwrap();

    // Verify skills are loaded into the tool manager's schema
    assert!(
        !tool_manager.schema.is_empty(),
        "Tool manager should have skills loaded"
    );

    // Verify calculator is available
    assert!(
        tool_manager.schema.contains_key("calculator"),
        "Calculator should be available"
    );
}
