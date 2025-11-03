use crate::git::{detect_git_context, remove_worktree};
use crate::session::metadata::SessionMetadata;
use eyre::{Result, bail};
use std::path::Path;
use std::process::Command;

/// Check if worktree has uncommitted changes
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
    // Switch to target branch
    let status = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .arg("checkout")
        .arg(target)
        .status()?;
    
    if !status.success() {
        bail!("Failed to checkout {}", target);
    }
    
    // Merge branch
    let status = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .arg("merge")
        .arg(branch)
        .arg("--no-ff")
        .arg("-m")
        .arg(format!("Merge branch '{}'", branch))
        .status()?;
    
    if !status.success() {
        bail!("Merge failed - conflicts need resolution");
    }
    
    Ok(())
}

/// Prepare worktree for merge
pub fn prepare_merge(session: &SessionMetadata) -> Result<()> {
    let wt = session.worktree_info.as_ref()
        .ok_or_else(|| eyre::eyre!("Not a worktree session"))?;
    
    // Check for uncommitted changes
    if has_uncommitted_changes(&wt.path)? {
        bail!("Worktree has uncommitted changes. Commit or stash them first.");
    }
    
    Ok(())
}

/// Clean up after successful merge
pub fn cleanup_after_merge(session: &SessionMetadata) -> Result<()> {
    let wt = session.worktree_info.as_ref()
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
