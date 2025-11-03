use std::path::{
    Path,
    PathBuf,
};
use std::process::Command;

use super::error::{
    GitError,
    Result,
};

/// Information about a git worktree from `git worktree list`
#[derive(Debug, Clone)]
pub struct GitWorktreeInfo {
    /// Absolute path to the worktree directory
    pub path: PathBuf,
    /// Branch name checked out in this worktree
    pub branch: String,
    /// Current commit SHA in this worktree
    pub commit: String,
}

impl GitWorktreeInfo {
    /// Convert to session WorktreeInfo with additional context
    /// 
    /// # Arguments
    /// * `repo_root` - Root path of the main repository
    /// * `merge_target` - Target branch for eventual merge (usually "main")
    pub fn to_session_info(&self, repo_root: PathBuf, merge_target: String) -> crate::session::metadata::WorktreeInfo {
        crate::session::metadata::WorktreeInfo {
            path: self.path.clone(),
            branch: self.branch.clone(),
            repo_root,
            is_temporary: false,
            merge_target,
        }
    }
}

/// List all worktrees in a git repository
/// 
/// # Arguments
/// * `repo_root` - Path to the git repository root
/// 
/// # Returns
/// Vector of worktree information for all worktrees in the repository
/// 
/// # Errors
/// * `GitError::CommandFailed` - If git command fails or repo is invalid
pub fn list_worktrees(repo_root: &Path) -> Result<Vec<GitWorktreeInfo>> {
    let output = Command::new("git")
        .current_dir(repo_root)
        .args(&["worktree", "list", "--porcelain"])
        .output()?;

    if !output.status.success() {
        return Err(GitError::CommandFailed(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    parse_worktree_list(&String::from_utf8_lossy(&output.stdout))
}

/// Parse output from `git worktree list --porcelain`
fn parse_worktree_list(output: &str) -> Result<Vec<GitWorktreeInfo>> {
    let mut worktrees = Vec::new();
    let mut current_path = None;
    let mut current_commit = None;
    let mut current_branch = None;

    for line in output.lines() {
        if line.starts_with("worktree ") {
            if let Some(path) = line.strip_prefix("worktree ") {
                current_path = Some(path.to_string());
            }
        } else if line.starts_with("HEAD ") {
            if let Some(commit) = line.strip_prefix("HEAD ") {
                current_commit = Some(commit.to_string());
            }
        } else if line.starts_with("branch ") {
            if let Some(branch_ref) = line.strip_prefix("branch ") {
                let branch = branch_ref
                    .strip_prefix("refs/heads/")
                    .unwrap_or(branch_ref);
                current_branch = Some(branch.to_string());
            }
        } else if line.is_empty() {
            if let (Some(path), Some(commit), Some(branch)) =
                (current_path.take(), current_commit.take(), current_branch.take())
            {
                worktrees.push(GitWorktreeInfo {
                    path: PathBuf::from(path),
                    branch,
                    commit,
                });
            }
        }
    }

    // Handle last entry if no trailing newline
    if let (Some(path), Some(commit), Some(branch)) = (current_path, current_commit, current_branch) {
        worktrees.push(GitWorktreeInfo {
            path: PathBuf::from(path),
            branch,
            commit,
        });
    }

    Ok(worktrees)
}

/// Create a new git worktree with a new branch
/// 
/// # Arguments
/// * `repo_root` - Path to the main repository
/// * `name` - Name for the new branch
/// * `base_branch` - Branch to base the new worktree on
/// * `path` - Optional custom path (defaults to `{repo_root}-{name}`)
/// 
/// # Returns
/// Path to the created worktree directory
/// 
/// # Errors
/// * `GitError::BranchExists` - If branch already exists
/// * `GitError::WorktreeExists` - If worktree path already exists
/// * `GitError::CommandFailed` - If git command fails or path is invalid
pub fn create_worktree(repo_root: &Path, name: &str, base_branch: &str, path: Option<PathBuf>) -> Result<PathBuf> {
    // Check if branch already exists
    if branch_exists(repo_root, name)? {
        return Err(GitError::BranchExists(name.to_string()));
    }

    // Determine worktree path
    let worktree_path = if let Some(p) = path {
        p
    } else {
        let parent = repo_root.parent().unwrap_or(repo_root);
        let repo_name = repo_root
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| GitError::CommandFailed("Invalid repo path".to_string()))?;
        parent.join(format!("{}-{}", repo_name, name))
    };

    // Check if worktree already exists
    if worktree_path.exists() {
        return Err(GitError::WorktreeExists(worktree_path.display().to_string()));
    }

    // Create worktree
    let worktree_path_str = worktree_path
        .to_str()
        .ok_or_else(|| GitError::CommandFailed("Invalid worktree path".to_string()))?;
    
    let output = Command::new("git")
        .current_dir(repo_root)
        .args(&[
            "worktree",
            "add",
            worktree_path_str,
            "-b",
            name,
            base_branch,
        ])
        .output()?;

    if !output.status.success() {
        return Err(GitError::CommandFailed(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    Ok(worktree_path)
}

/// Remove a git worktree
/// 
/// # Arguments
/// * `path` - Path to the worktree directory to remove
/// 
/// # Errors
/// * `GitError::CommandFailed` - If git command fails or path is invalid
pub fn remove_worktree(path: &Path) -> Result<()> {
    let path_str = path
        .to_str()
        .ok_or_else(|| GitError::CommandFailed("Invalid worktree path".to_string()))?;
    
    let output = Command::new("git")
        .args(&["worktree", "remove", path_str])
        .output()?;

    if !output.status.success() {
        return Err(GitError::CommandFailed(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    Ok(())
}

pub fn worktree_exists(repo_root: &Path, name: &str) -> bool {
    list_worktrees(repo_root)
        .ok()
        .and_then(|worktrees| worktrees.iter().find(|wt| wt.branch == name).map(|_| true))
        .unwrap_or(false)
}

pub fn branch_exists(repo_root: &Path, name: &str) -> Result<bool> {
    let output = Command::new("git")
        .current_dir(repo_root)
        .args(&["branch", "--list", name])
        .output()?;

    if !output.status.success() {
        return Err(GitError::CommandFailed(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    Ok(!output.stdout.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_worktree_list() {
        let output = "worktree /path/to/repo\nHEAD abc123\nbranch refs/heads/main\n\nworktree /path/to/worktree\nHEAD def456\nbranch refs/heads/feature\n\n";

        let worktrees = parse_worktree_list(output).unwrap();

        assert_eq!(worktrees.len(), 2);
        assert_eq!(worktrees[0].branch, "main");
        assert_eq!(worktrees[1].branch, "feature");
    }
}
