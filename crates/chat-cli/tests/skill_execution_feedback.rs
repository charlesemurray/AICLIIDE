/// Tests for skill execution feedback
///
/// Validates that users see clear feedback during skill execution
use chat_cli::cli::chat::tools::skill_tool::SkillTool;
use chat_cli::cli::skills::SkillRegistry;

#[tokio::test]
async fn test_successful_execution_shows_feedback() {
    let registry = SkillRegistry::with_builtins();
    let tool = SkillTool::new(
        "calculator".to_string(),
        serde_json::json!({
            "a": 5.0,
            "b": 3.0,
            "op": "add"
        }),
    );
    let mut output = Vec::new();

    let result = tool.invoke_with_feedback(&registry, &mut output, true).await;
    assert!(result.is_ok());

    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("🔧 Executing skill: calculator"));
    assert!(output_str.contains("✓ Skill completed in"));
    assert!(output_str.contains("8"));
}

#[tokio::test]
async fn test_failed_execution_shows_error_feedback() {
    let registry = SkillRegistry::with_builtins();
    let tool = SkillTool::new(
        "calculator".to_string(),
        serde_json::json!({
            "a": 10.0,
            "b": 0.0,
            "op": "divide"
        }),
    );
    let mut output = Vec::new();

    let result = tool.invoke_with_feedback(&registry, &mut output, true).await;
    assert!(result.is_err());

    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("🔧 Executing skill: calculator"));
    assert!(output_str.contains("✗ Skill failed after"));
}

#[tokio::test]
async fn test_execution_without_feedback() {
    let registry = SkillRegistry::with_builtins();
    let tool = SkillTool::new(
        "calculator".to_string(),
        serde_json::json!({
            "a": 5.0,
            "b": 3.0,
            "op": "add"
        }),
    );
    let mut output = Vec::new();

    let result = tool.invoke_with_feedback(&registry, &mut output, false).await;
    assert!(result.is_ok());

    let output_str = String::from_utf8(output).unwrap();
    assert!(!output_str.contains("🔧 Executing skill"));
    assert!(!output_str.contains("✓ Skill completed"));
    assert!(output_str.contains("8")); // Still shows result
}

#[tokio::test]
async fn test_execution_timing_is_shown() {
    let registry = SkillRegistry::with_builtins();
    let tool = SkillTool::new(
        "calculator".to_string(),
        serde_json::json!({
            "a": 100.0,
            "b": 50.0,
            "op": "multiply"
        }),
    );
    let mut output = Vec::new();

    let result = tool.invoke_with_feedback(&registry, &mut output, true).await;
    assert!(result.is_ok());

    let output_str = String::from_utf8(output).unwrap();
    // Check that timing is shown with format like "0.00s" or "0.01s"
    assert!(output_str.contains("s")); // Contains seconds indicator
    assert!(output_str.contains("✓ Skill completed in"));
}

#[tokio::test]
async fn test_skill_not_found_shows_feedback() {
    let registry = SkillRegistry::new();
    let tool = SkillTool::new("nonexistent".to_string(), serde_json::json!({}));
    let mut output = Vec::new();

    let result = tool.invoke_with_feedback(&registry, &mut output, true).await;
    assert!(result.is_err());

    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("🔧 Executing skill: nonexistent"));
    // Note: Skill not found happens before execution, so no completion message
}

#[tokio::test]
async fn test_default_invoke_shows_feedback() {
    let registry = SkillRegistry::with_builtins();
    let tool = SkillTool::new(
        "calculator".to_string(),
        serde_json::json!({
            "a": 7.0,
            "b": 2.0,
            "op": "add"
        }),
    );
    let mut output = Vec::new();

    // Default invoke() should show feedback
    let result = tool.invoke(&registry, &mut output).await;
    assert!(result.is_ok());

    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("🔧 Executing skill: calculator"));
    assert!(output_str.contains("✓ Skill completed in"));
}
