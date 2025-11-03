use std::path::PathBuf;

use chat_cli::cli::chat::worktree_strategy::{
    WorktreeStrategy,
    resolve_worktree_strategy,
};
use chat_cli::git::GitContext;
use chat_cli::theme::session::SessionType;

#[test]
fn test_worktree_strategy_with_explicit_flag() {
    let name = "my-feature".to_string();
    let strategy = resolve_worktree_strategy(Some(&name), false, None);
    assert_eq!(strategy, WorktreeStrategy::Create("my-feature".to_string()));
}

#[test]
fn test_worktree_strategy_no_worktree_flag() {
    let strategy = resolve_worktree_strategy(None, true, None);
    assert_eq!(strategy, WorktreeStrategy::Never);
}

#[test]
fn test_worktree_strategy_in_existing_worktree() {
    let context = GitContext {
        repo_root: PathBuf::from("/tmp/repo"),
        repo_name: "repo".to_string(),
        branch_name: "feature/test".to_string(),
        is_worktree: true,
        is_main_branch: false,
        worktree_path: None,
    };
    let strategy = resolve_worktree_strategy(None, false, Some(&context));
    assert_eq!(strategy, WorktreeStrategy::UseExisting);
}

#[test]
fn test_session_type_worktree_requirements() {
    assert!(SessionType::Feature.requires_worktree());
    assert!(SessionType::Refactor.requires_worktree());
    assert!(SessionType::Experiment.requires_worktree());

    assert!(!SessionType::Debug.requires_worktree());
    assert!(!SessionType::Hotfix.requires_worktree());
}

#[test]
fn test_session_type_interactivity() {
    assert!(SessionType::Feature.is_interactive());
    assert!(SessionType::Debug.is_interactive());
    assert!(!SessionType::Development.is_interactive());
}

#[test]
fn test_skill_worktree_requirement() {
    use chat_cli::cli::skills::types::JsonSkill;
    use serde_json::json;

    let skill_json = json!({
        "name": "test-skill",
        "type": "command",
        "command": "echo test",
        "requires_worktree": true
    });

    let skill = JsonSkill::from_json(skill_json).unwrap();
    assert!(skill.requires_worktree());
}

#[test]
fn test_skill_without_worktree_requirement() {
    use chat_cli::cli::skills::types::JsonSkill;
    use serde_json::json;

    let skill_json = json!({
        "name": "test-skill",
        "type": "command",
        "command": "echo test"
    });

    let skill = JsonSkill::from_json(skill_json).unwrap();
    assert!(!skill.requires_worktree());
}
