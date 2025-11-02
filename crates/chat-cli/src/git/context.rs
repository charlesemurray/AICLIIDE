use std::path::{Path, PathBuf};
use std::process::Command;

use super::error::{GitError, Result};

#[derive(Debug, Clone)]
pub struct GitContext {
    pub repo_root: PathBuf,
    pub repo_name: String,
    pub branch_name: String,
    pub is_worktree: bool,
    pub is_main_branch: bool,
    pub worktree_path: Option<PathBuf>,
}

impl GitContext {
    pub fn is_main_branch(&self) -> bool {
        self.is_main_branch
    }
}

pub fn is_git_installed() -> bool {
    Command::new("git")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

pub fn detect_git_context(path: &Path) -> Result<GitContext> {
    if !is_git_installed() {
        return Err(GitError::NotInstalled);
    }

    let repo_root = get_repo_root(path)?;
    let branch_name = get_current_branch(path)?;
    let is_worktree = is_worktree(path)?;
    let is_main_branch = is_main_branch(&branch_name);
    let worktree_path = if is_worktree {
        Some(path.to_path_buf())
    } else {
        None
    };

    let repo_name = repo_root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    Ok(GitContext {
        repo_root,
        repo_name,
        branch_name,
        is_worktree,
        is_main_branch,
        worktree_path,
    })
}

pub fn get_repo_root(path: &Path) -> Result<PathBuf> {
    let output = Command::new("git")
        .current_dir(path)
        .args(&["rev-parse", "--show-toplevel"])
        .output()?;

    if !output.status.success() {
        return Err(GitError::NotARepository);
    }

    let root = String::from_utf8_lossy(&output.stdout)
        .trim()
        .to_string();

    Ok(PathBuf::from(root))
}

pub fn get_current_branch(path: &Path) -> Result<String> {
    let output = Command::new("git")
        .current_dir(path)
        .args(&["rev-parse", "--abbrev-ref", "HEAD"])
        .output()?;

    if !output.status.success() {
        return Err(GitError::CommandFailed(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn is_worktree(path: &Path) -> Result<bool> {
    let output = Command::new("git")
        .current_dir(path)
        .args(&["rev-parse", "--git-dir"])
        .output()?;

    if !output.status.success() {
        return Ok(false);
    }

    let git_dir = String::from_utf8_lossy(&output.stdout).trim().to_string();
    
    // Worktrees have .git file pointing to worktrees directory
    Ok(git_dir.contains("worktrees"))
}

pub fn is_main_branch(branch: &str) -> bool {
    matches!(branch, "main" | "master")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_main_branch() {
        assert!(is_main_branch("main"));
        assert!(is_main_branch("master"));
        assert!(!is_main_branch("feature/test"));
        assert!(!is_main_branch("develop"));
    }

    #[test]
    fn test_is_git_installed() {
        // This will pass if git is installed on the system
        let installed = is_git_installed();
        println!("Git installed: {}", installed);
    }
}
