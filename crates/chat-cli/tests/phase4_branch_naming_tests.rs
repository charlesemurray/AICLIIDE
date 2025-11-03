use chat_cli::cli::chat::branch_naming::{
    generate_branch_name,
    generate_from_conversation,
    sanitize_branch_name,
};

#[test]
fn test_generate_from_conversation_with_type() {
    let msg = "Add user authentication with OAuth";
    let name = generate_from_conversation(msg, Some("feature"));
    assert_eq!(name, "feature/user-authentication-with-oauth");
}

#[test]
fn test_generate_from_conversation_default_type() {
    let msg = "Implement new dashboard";
    let name = generate_from_conversation(msg, None);
    assert_eq!(name, "feature/implement-dashboard");
}

#[test]
fn test_generate_from_short_message() {
    let msg = "Fix bug in login";
    let name = generate_from_conversation(msg, Some("fix"));
    assert!(name.starts_with("fix/"));
}

#[test]
fn test_sanitize_preserves_valid_chars() {
    assert_eq!(sanitize_branch_name("feature-123"), "feature-123");
}

#[test]
fn test_generate_with_prefix() {
    let name = generate_branch_name("test feature", Some("feat"));
    assert_eq!(name, "feat/test-feature");
}
