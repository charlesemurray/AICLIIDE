use crate::git::GitContext;

/// Strategy for worktree creation/usage
#[derive(Debug, Clone, PartialEq)]
pub enum WorktreeStrategy {
    /// Create a worktree with the specified name
    Create(String),
    /// Create a temporary worktree with auto-generated name
    CreateTemp,
    /// Use existing worktree (already in one)
    UseExisting,
    /// Never create a worktree
    Never,
    /// Ask the user what to do
    Ask,
}

/// Resolve worktree strategy based on CLI args and context
pub fn resolve_worktree_strategy(
    worktree_arg: Option<&String>,
    no_worktree: bool,
    git_context: Option<&GitContext>,
) -> WorktreeStrategy {
    // Layer 1: Explicit flags take precedence
    if let Some(name) = worktree_arg {
        return WorktreeStrategy::Create(name.clone());
    }

    if no_worktree {
        return WorktreeStrategy::Never;
    }

    // Layer 2: Check if already in a worktree
    if let Some(context) = git_context {
        if context.is_worktree {
            return WorktreeStrategy::UseExisting;
        }
    }

    // Layer 3: Default - ask user
    WorktreeStrategy::Ask
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_explicit_worktree_name() {
        let name = "feature-branch".to_string();
        let strategy = resolve_worktree_strategy(Some(&name), false, None);
        assert_eq!(strategy, WorktreeStrategy::Create("feature-branch".to_string()));
    }

    #[test]
    fn test_no_worktree_flag() {
        let strategy = resolve_worktree_strategy(None, true, None);
        assert_eq!(strategy, WorktreeStrategy::Never);
    }

    #[test]
    fn test_explicit_name_overrides_no_worktree() {
        let name = "feature".to_string();
        let strategy = resolve_worktree_strategy(Some(&name), true, None);
        assert_eq!(strategy, WorktreeStrategy::Create("feature".to_string()));
    }

    #[test]
    fn test_already_in_worktree() {
        let context = GitContext {
            repo_root: "/tmp/repo".into(),
            repo_name: "repo".to_string(),
            branch_name: "feature".to_string(),
            is_worktree: true,
            is_main_branch: false,
            worktree_path: None,
        };
        let strategy = resolve_worktree_strategy(None, false, Some(&context));
        assert_eq!(strategy, WorktreeStrategy::UseExisting);
    }

    #[test]
    fn test_default_asks_user() {
        let strategy = resolve_worktree_strategy(None, false, None);
        assert_eq!(strategy, WorktreeStrategy::Ask);
    }

    #[test]
    fn test_not_in_worktree_asks() {
        let context = GitContext {
            repo_root: "/tmp/repo".into(),
            repo_name: "repo".to_string(),
            branch_name: "main".to_string(),
            is_worktree: false,
            is_main_branch: true,
            worktree_path: None,
        };
        let strategy = resolve_worktree_strategy(None, false, Some(&context));
        assert_eq!(strategy, WorktreeStrategy::Ask);
    }
}
