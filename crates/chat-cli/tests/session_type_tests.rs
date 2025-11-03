use chat_cli::theme::session::SessionType;

#[test]
fn test_worktree_required_types() {
    assert!(SessionType::Feature.requires_worktree());
    assert!(SessionType::Refactor.requires_worktree());
    assert!(SessionType::Experiment.requires_worktree());

    assert!(!SessionType::Debug.requires_worktree());
    assert!(!SessionType::Planning.requires_worktree());
    assert!(!SessionType::Development.requires_worktree());
    assert!(!SessionType::CodeReview.requires_worktree());
    assert!(!SessionType::Hotfix.requires_worktree());
}

#[test]
fn test_interactive_types() {
    assert!(SessionType::Debug.is_interactive());
    assert!(SessionType::Planning.is_interactive());
    assert!(SessionType::CodeReview.is_interactive());
    assert!(SessionType::Feature.is_interactive());
    assert!(SessionType::Hotfix.is_interactive());
    assert!(SessionType::Refactor.is_interactive());
    assert!(SessionType::Experiment.is_interactive());

    assert!(!SessionType::Development.is_interactive());
}

#[test]
fn test_display_names() {
    assert_eq!(SessionType::Feature.display_name(), "Feature");
    assert_eq!(SessionType::Hotfix.display_name(), "Hotfix");
    assert_eq!(SessionType::Refactor.display_name(), "Refactor");
    assert_eq!(SessionType::Experiment.display_name(), "Experiment");
}
