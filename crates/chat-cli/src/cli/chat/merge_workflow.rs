use std::path::Path;
use std::process::Command;

use eyre::{
    Result,
    bail,
};

use crate::git::remove_worktree;
use crate::session::metadata::{MergeState, SessionMetadata};

/// Launch a chat session to help resolve merge conflicts
pub fn launch_conflict_resolution_chat(conflicts: &[String], branch: &str, target: &str) -> String {
    let conflict_list = conflicts.iter()
        .map(|f| format!("  - {}", f))
        .collect::<Vec<_>>()
        .join("\n");
    
    format!(
        "I need help resolving merge conflicts when merging branch '{}' into '{}'.\n\n\
        Conflicted files:\n{}\n\n\
        Please help me:\n\
        1. Understand what changes conflict\n\
        2. Decide how to resolve each conflict\n\
        3. Verify the resolution is correct\n\n\
        I'm in the worktree and ready to edit files.",
        branch, target, conflict_list
    )
}

/// Check if there are unresolved conflicts in the worktree
pub fn has_unresolved_conflicts(worktree_path: &Path) -> Result<bool> {
    let output = Command::new("git")
        .arg("-C")
        .arg(worktree_path)
        .arg("diff")
        .arg("--name-only")
        .arg("--diff-filter=U")
        .output()?;
    
    Ok(!output.stdout.is_empty())
}

/// Get list of conflicted files
pub fn get_conflicted_files(worktree_path: &Path) -> Result<Vec<String>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(worktree_path)
        .arg("diff")
        .arg("--name-only")
        .arg("--diff-filter=U")
        .output()?;
    
    let files = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|s| s.to_string())
        .collect();
    
    Ok(files)
}


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

/// Print conflict resolution guidance
pub fn print_conflict_resolution_guide(conflicts: &[String], branch: &str, target: &str) {
    println!("⚠️  Conflicts detected in {} file(s):", conflicts.len());
    for file in conflicts.iter().take(5) {
        println!("  • {}", file);
    }
    if conflicts.len() > 5 {
        println!("  ... and {} more", conflicts.len() - 5);
    }
    
    println!("\n📋 Resolution options:");
    println!("  1. Resolve manually:");
    println!("     git checkout {}", target);
    println!("     git merge {}", branch);
    println!("     # Fix conflicts, then:");
    println!("     git add .");
    println!("     git commit");
    println!("\n  2. Force merge (requires manual resolution):");
    println!("     /sessions merge --force");
    println!("\n  3. Cancel and continue working:");
    println!("     /sessions list");
}

/// Merge worktree branch back to target
/// Returns Ok(None) on success, Ok(Some(conflicts)) if conflicts detected, Err on failure
pub fn merge_branch(repo_root: &Path, branch: &str, target: &str, force: bool) -> Result<Option<Vec<String>>> {
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
        Ok(status) if status.success() => Ok(None),
        _ => {
            // Check if it's a conflict or other error
            let conflicts = get_conflicted_files(repo_root)?;
            
            if !conflicts.is_empty() && force {
                // User wants to resolve conflicts - leave in conflicted state
                Ok(Some(conflicts))
            } else {
                // Rollback: return to original branch
                let _ = checkout_branch(repo_root, &original_branch);
                if conflicts.is_empty() {
                    bail!("Merge failed. Returned to {}", original_branch)
                } else {
                    Ok(Some(conflicts))
                }
            }
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

/// Complete merge after conflicts have been resolved
pub fn complete_merge(worktree_path: &Path) -> Result<()> {
    // Verify no unresolved conflicts
    if has_unresolved_conflicts(worktree_path)? {
        bail!("Cannot complete merge: unresolved conflicts remain");
    }
    
    // Stage all resolved files
    Command::new("git")
        .arg("-C")
        .arg(worktree_path)
        .arg("add")
        .arg(".")
        .status()?;
    
    // Commit the merge
    let status = Command::new("git")
        .arg("-C")
        .arg(worktree_path)
        .arg("commit")
        .arg("--no-edit")
        .status()?;
    
    if !status.success() {
        bail!("Failed to commit merge");
    }
    
    Ok(())
}
