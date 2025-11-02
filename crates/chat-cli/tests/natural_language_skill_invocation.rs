use chat_cli::cli::chat::tool_manager::ToolManager;
use chat_cli::os::Os;

#[tokio::test]
async fn test_tool_manager_with_skills_initialization() {
    // Verify ToolManager can be initialized with skills
    let os = Os::new().await.unwrap();
    let result = ToolManager::new_with_skills(&os).await;

    assert!(result.is_ok(), "ToolManager should initialize with skills successfully");
}

#[tokio::test]
async fn test_tool_manager_default_initialization() {
    // Test that default ToolManager works
    let _tool_manager = ToolManager::default();

    // Default initialization should succeed
    assert!(true);
}

#[tokio::test]
async fn test_multiple_tool_manager_instances() {
    // Verify multiple ToolManager instances can coexist
    let os = Os::new().await.unwrap();
    let tm1 = ToolManager::new_with_skills(&os).await;
    let tm2 = ToolManager::new_with_skills(&os).await;

    assert!(tm1.is_ok());
    assert!(tm2.is_ok());
}
