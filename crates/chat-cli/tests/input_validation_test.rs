use chat_cli::cli::chat::branch_naming::sanitize_branch_name;

#[test]
fn test_sanitize_rejects_empty_name() {
    let result = sanitize_branch_name("");
    assert!(result.is_err(), "Should reject empty branch name");
}

#[test]
fn test_sanitize_rejects_whitespace_only() {
    let result = sanitize_branch_name("   ");
    assert!(result.is_err(), "Should reject whitespace-only name");
}

#[test]
fn test_sanitize_rejects_invalid_chars_only() {
    let result = sanitize_branch_name("!!!@@@###");
    assert!(result.is_err(), "Should reject name with no valid characters");
}

#[test]
fn test_sanitize_rejects_leading_dash() {
    let result = sanitize_branch_name("-feature");
    // After sanitization, if it starts with dash, should be rejected
    if let Ok(name) = result {
        assert!(!name.starts_with('-'), "Should not start with dash");
    }
}

#[test]
fn test_sanitize_rejects_trailing_dash() {
    let result = sanitize_branch_name("feature-");
    // After sanitization, if it ends with dash, should be rejected
    if let Ok(name) = result {
        assert!(!name.ends_with('-'), "Should not end with dash");
    }
}

#[test]
fn test_sanitize_accepts_valid_name() {
    let result = sanitize_branch_name("feature-branch");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "feature-branch");
}

#[test]
fn test_sanitize_converts_spaces() {
    let result = sanitize_branch_name("my feature");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "my-feature");
}

#[test]
fn test_sanitize_handles_mixed_case() {
    let result = sanitize_branch_name("MyFeature");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "myfeature");
}
