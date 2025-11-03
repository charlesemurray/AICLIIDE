use std::path::Path;

use crate::git::{
    GitError,
    list_worktrees,
};

pub fn sanitize_branch_name(input: &str) -> String {
    input
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| match c {
            'a'..='z' | '0'..='9' => c,
            ' ' | '_' | '-' | '/' => '-',
            _ => '-',
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
        .chars()
        .take(50)
        .collect()
}

pub fn generate_branch_name(context: &str, prefix: Option<&str>) -> String {
    let sanitized = sanitize_branch_name(context);
    if let Some(p) = prefix {
        format!("{}/{}", sanitize_branch_name(p), sanitized)
    } else {
        sanitized
    }
}

/// Generate a branch name from conversation context
pub fn generate_from_conversation(first_message: &str, session_type: Option<&str>) -> String {
    let words: Vec<&str> = first_message
        .split_whitespace()
        .filter(|w| w.len() > 3)
        .take(4)
        .collect();

    let context = words.join(" ");
    let prefix = session_type.or(Some("feature"));
    generate_branch_name(&context, prefix)
}

pub fn check_branch_conflict(repo_root: &Path, branch_name: &str) -> Result<bool, GitError> {
    let worktrees = list_worktrees(repo_root)?;
    Ok(worktrees.iter().any(|wt| wt.branch == branch_name))
}

pub fn ensure_unique_branch_name(repo_root: &Path, base_name: &str) -> Result<String, GitError> {
    let mut name = base_name.to_string();
    let mut counter = 1;
    while check_branch_conflict(repo_root, &name)? {
        name = format!("{}-{}", base_name, counter);
        counter += 1;
        if counter > 100 {
            return Err(GitError::CommandFailed("Too many conflicts".to_string()));
        }
    }
    Ok(name)
}
