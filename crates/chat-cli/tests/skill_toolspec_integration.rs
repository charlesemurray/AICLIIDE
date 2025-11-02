use chat_cli::cli::chat::tools::ToolOrigin;
use chat_cli::cli::skills::SkillRegistry;

#[tokio::test]
async fn test_skill_to_toolspec_conversion() {
    // Test that skills can be converted to ToolSpecs
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
async fn test_all_skills_export_toolspecs() {
    let registry = SkillRegistry::with_builtins();
    let toolspecs = registry.get_all_toolspecs();

    assert!(!toolspecs.is_empty(), "Should have at least one skill");

    // Verify all toolspecs have required fields
    for toolspec in toolspecs {
        assert!(!toolspec.name.is_empty(), "ToolSpec should have a name");
        assert!(!toolspec.description.is_empty(), "ToolSpec should have a description");
    }
}

#[tokio::test]
async fn test_skill_toolspec_schema_validity() {
    let registry = SkillRegistry::with_builtins();
    let toolspecs = registry.get_all_toolspecs();

    for toolspec in toolspecs {
        let schema = &toolspec.input_schema.0;

        // Verify schema has required structure
        assert!(schema["type"].is_string(), "Schema should have type");
        assert!(schema["properties"].is_object(), "Schema should have properties");
    }
}
