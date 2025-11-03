use std::path::{
    Path,
    PathBuf,
};
use std::process::Command;

use super::error::{
    GitError,
    Result,
};

#[derive(Debug, Clone)]
pub struct GitWorktreeInfo {
    pub path: PathBuf,
    pub branch: String,
    pub commit: String,
}

impl GitWorktreeInfo {
    /// Convert to session WorktreeInfo with additional context
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

fn parse_worktree_list(output: &str) -> Result<Vec<GitWorktreeInfo>> {
    let mut worktrees = Vec::new();
    let mut current_path = None;
    let mut current_commit = None;
    let mut current_branch = None;

    for line in output.lines() {
        if line.starts_with("worktree ") {
            current_path = Some(line.strip_prefix("worktree ").unwrap().to_string());
        } else if line.starts_with("HEAD ") {
            current_commit = Some(line.strip_prefix("HEAD ").unwrap().to_string());
        } else if line.starts_with("branch ") {
            current_branch = Some(
                line.strip_prefix("branch ")
                    .unwrap()
                    .strip_prefix("refs/heads/")
                    .unwrap_or(line.strip_prefix("branch ").unwrap())
                    .to_string(),
            );
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

pub fn create_worktree(repo_root: &Path, name: &str, base_branch: &str, path: Option<PathBuf>) -> Result<PathBuf> {
    // Check if branch already exists
    if branch_exists(repo_root, name)? {
        return Err(GitError::BranchExists(name.to_string()));
    }

    // Determine worktree path
    let worktree_path = if let Some(p) = path {
        p
    } else {
        repo_root.parent().unwrap_or(repo_root).join(format!(
            "{}-{}",
            repo_root.file_name().unwrap().to_str().unwrap(),
            name
        ))
    };

    // Check if worktree already exists
    if worktree_path.exists() {
        return Err(GitError::WorktreeExists(worktree_path.display().to_string()));
    }

    // Create worktree
    let output = Command::new("git")
        .current_dir(repo_root)
        .args(&[
            "worktree",
            "add",
            worktree_path.to_str().unwrap(),
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

pub fn remove_worktree(path: &Path) -> Result<()> {
    let output = Command::new("git")
        .args(&["worktree", "remove", path.to_str().unwrap()])
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
