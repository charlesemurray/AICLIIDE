use std::fs;

/// Tests for skill loading feedback
///
/// Validates that users see clear feedback when skills are loaded
use chat_cli::cli::chat::skill_registry::SkillRegistry;
use tempfile::TempDir;

#[tokio::test]
async fn test_successful_skill_loading_shows_feedback() {
    let temp_dir = TempDir::new().unwrap();
    let skill_path = temp_dir.path().join("test.json");

    let skill_json = r#"{
        "name": "test-skill",
        "description": "A test skill",
        "skill_type": "code_inline"
    }"#;

    fs::write(&skill_path, skill_json).unwrap();

    let mut registry = SkillRegistry::new();
    let mut output = Vec::new();

    registry
        .load_from_directory_with_feedback(temp_dir.path(), &mut output)
        .await
        .unwrap();

    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("✓ Loaded skill: test-skill"));
    assert!(output_str.contains("Loaded 1 skill(s), 0 failed"));
}

#[tokio::test]
async fn test_failed_skill_loading_shows_error() {
    let temp_dir = TempDir::new().unwrap();
    let invalid_path = temp_dir.path().join("invalid.json");

    fs::write(&invalid_path, "not valid json").unwrap();

    let mut registry = SkillRegistry::new();
    let mut output = Vec::new();

    registry
        .load_from_directory_with_feedback(temp_dir.path(), &mut output)
        .await
        .unwrap();

    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("✗ Failed to load invalid.json"));
    assert!(output_str.contains("Loaded 0 skill(s), 1 failed"));
}

#[tokio::test]
async fn test_mixed_success_and_failure_feedback() {
    let temp_dir = TempDir::new().unwrap();

    // Valid skill
    let valid_path = temp_dir.path().join("valid.json");
    fs::write(
        &valid_path,
        r#"{"name": "good-skill", "description": "Valid", "skill_type": "code_inline"}"#,
    )
    .unwrap();

    // Invalid skill
    let invalid_path = temp_dir.path().join("bad.json");
    fs::write(&invalid_path, "{ bad json }").unwrap();

    let mut registry = SkillRegistry::new();
    let mut output = Vec::new();

    registry
        .load_from_directory_with_feedback(temp_dir.path(), &mut output)
        .await
        .unwrap();

    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("✓ Loaded skill: good-skill"));
    assert!(output_str.contains("✗ Failed to load bad.json"));
    assert!(output_str.contains("Loaded 1 skill(s), 1 failed"));
}

#[tokio::test]
async fn test_empty_directory_shows_no_feedback() {
    let temp_dir = TempDir::new().unwrap();

    let mut registry = SkillRegistry::new();
    let mut output = Vec::new();

    registry
        .load_from_directory_with_feedback(temp_dir.path(), &mut output)
        .await
        .unwrap();

    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.is_empty() || output_str.contains("Loaded 0 skill(s), 0 failed"));
}

#[tokio::test]
async fn test_multiple_successful_skills() {
    let temp_dir = TempDir::new().unwrap();

    for i in 1..=3 {
        let skill_path = temp_dir.path().join(format!("skill{}.json", i));
        let skill_json = format!(
            r#"{{"name": "skill-{}", "description": "Skill {}", "skill_type": "code_inline"}}"#,
            i, i
        );
        fs::write(&skill_path, skill_json).unwrap();
    }

    let mut registry = SkillRegistry::new();
    let mut output = Vec::new();

    registry
        .load_from_directory_with_feedback(temp_dir.path(), &mut output)
        .await
        .unwrap();

    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("✓ Loaded skill: skill-1"));
    assert!(output_str.contains("✓ Loaded skill: skill-2"));
    assert!(output_str.contains("✓ Loaded skill: skill-3"));
    assert!(output_str.contains("Loaded 3 skill(s), 0 failed"));
}
