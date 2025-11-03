use std::path::Path;

use eyre::{Result, bail};

use crate::git::{
    GitError,
    list_worktrees,
};

// Branch naming constants
const MIN_WORD_LENGTH: usize = 3;
const MAX_CONTEXT_WORDS: usize = 4;
const MAX_BRANCH_NAME_LENGTH: usize = 50;
const MAX_CONFLICT_RETRIES: u32 = 100;

pub fn sanitize_branch_name(input: &str) -> Result<String> {
    // Validate input
    if input.trim().is_empty() {
        bail!("Branch name cannot be empty");
    }
    
    let sanitized = input
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
        .take(MAX_BRANCH_NAME_LENGTH)
        .collect::<String>();
    
    // Validate result
    if sanitized.is_empty() {
        bail!("Branch name '{}' contains no valid characters", input);
    }
    if sanitized.starts_with('-') || sanitized.ends_with('-') {
        bail!("Branch name cannot start or end with '-'");
    }
    
    Ok(sanitized)
}

pub fn generate_branch_name(context: &str, prefix: Option<&str>) -> Result<String> {
    let sanitized = sanitize_branch_name(context)?;
    if let Some(p) = prefix {
        Ok(format!("{}/{}", sanitize_branch_name(p)?, sanitized))
    } else {
        Ok(sanitized)
    }
}

/// Generate a branch name from conversation context
pub fn generate_from_conversation(first_message: &str, session_type: Option<&str>) -> Result<String> {
    let words: Vec<&str> = first_message
        .split_whitespace()
        .filter(|w| w.len() > MIN_WORD_LENGTH)
        .take(MAX_CONTEXT_WORDS)
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
        if counter > MAX_CONFLICT_RETRIES {
            return Err(GitError::CommandFailed(
                format!("Too many conflicts (tried {} names)", MAX_CONFLICT_RETRIES)
            ));
        }
    }
    Ok(name)
}
