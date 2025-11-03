use std::path::Path;
use std::process::Command;

use eyre::{
    Result,
    bail,
};

use crate::git::remove_worktree;
use crate::session::metadata::SessionMetadata;

/// Get the current branch name
fn get_current_branch(repo_root: &Path) -> Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .arg("branch")
        .arg("--show-current")
        .output()?;
    
    if !output.status.success() {
        bail!("Failed to get current branch");
    }
    
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Checkout a branch
fn checkout_branch(repo_root: &Path, branch: &str) -> Result<()> {
    let status = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .arg("checkout")
        .arg(branch)
        .status()?;
    
    if !status.success() {
        bail!("Failed to checkout {}", branch);
    }
    
    Ok(())
}

/// Check if worktree has uncommitted changes
/// 
/// # Arguments
/// * `worktree_path` - Path to the worktree directory
/// 
/// # Returns
/// `true` if there are uncommitted changes, `false` otherwise
pub fn has_uncommitted_changes(worktree_path: &Path) -> Result<bool> {
    let output = Command::new("git")
        .arg("-C")
        .arg(worktree_path)
        .arg("status")
        .arg("--porcelain")
        .output()?;

    Ok(!output.stdout.is_empty())
}

/// Detect merge conflicts
pub fn detect_conflicts(repo_root: &Path, branch: &str, target: &str) -> Result<Vec<String>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .arg("merge-tree")
        .arg(target)
        .arg(branch)
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let conflicts: Vec<String> = stdout
        .lines()
        .filter(|line| line.starts_with("changed in both"))
        .map(|line| line.split_whitespace().last().unwrap_or("").to_string())
        .collect();

    Ok(conflicts)
}

/// Merge worktree branch back to target
pub fn merge_branch(repo_root: &Path, branch: &str, target: &str) -> Result<()> {
    // Save current branch for rollback
    let original_branch = get_current_branch(repo_root)?;
    
    // Switch to target branch
    if let Err(e) = checkout_branch(repo_root, target) {
        return Err(e);
    }
    
    // Attempt merge
    let merge_result = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .arg("merge")
        .arg(branch)
        .arg("--no-ff")
        .arg("-m")
        .arg(format!("Merge branch '{}'", branch))
        .status();
    
    match merge_result {
        Ok(status) if status.success() => Ok(()),
        _ => {
            // Rollback: return to original branch
            let _ = checkout_branch(repo_root, &original_branch);
            bail!("Merge failed - conflicts need resolution. Returned to {}", original_branch)
        }
    }
}

/// Prepare worktree for merge
pub fn prepare_merge(session: &SessionMetadata) -> Result<()> {
    let wt = session
        .worktree_info
        .as_ref()
        .ok_or_else(|| eyre::eyre!("Not a worktree session"))?;

    // Check for uncommitted changes
    if has_uncommitted_changes(&wt.path)? {
        bail!("Worktree has uncommitted changes. Commit or stash them first.");
    }

    Ok(())
}

/// Clean up after successful merge
pub fn cleanup_after_merge(session: &SessionMetadata) -> Result<()> {
    let wt = session
        .worktree_info
        .as_ref()
        .ok_or_else(|| eyre::eyre!("Not a worktree session"))?;

    // Remove worktree
    remove_worktree(&wt.path)?;

    // Delete branch
    Command::new("git")
        .arg("-C")
        .arg(&wt.repo_root)
        .arg("branch")
        .arg("-d")
        .arg(&wt.branch)
        .status()?;

    Ok(())
}
