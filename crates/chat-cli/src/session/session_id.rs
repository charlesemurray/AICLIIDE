use std::path::Path;

use crate::git::detect_git_context;

/// Resolve a session ID based on context
pub fn resolve_session_id(path: &Path, override_id: Option<&str>) -> String {
    // Layer 1: Explicit override
    if let Some(id) = override_id {
        return id.to_string();
    }

    // Layer 2: Git context (repo/branch)
    if let Ok(context) = detect_git_context(path) {
        let repo_name = context.repo_root.file_name().and_then(|n| n.to_str()).unwrap_or("repo");
        return format!("{}/{}", repo_name, context.branch_name);
    }

    // Layer 3: Fallback to path-based ID
    path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("session")
        .to_string()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_explicit_override() {
        let path = PathBuf::from("/tmp/test");
        let id = resolve_session_id(&path, Some("custom-id"));
        assert_eq!(id, "custom-id");
    }

    #[test]
    fn test_fallback_to_path() {
        let path = PathBuf::from("/tmp/my-project");
        let id = resolve_session_id(&path, None);
        assert_eq!(id, "my-project");
    }

    #[test]
    fn test_root_path_fallback() {
        let path = PathBuf::from("/");
        let id = resolve_session_id(&path, None);
        assert_eq!(id, "session");
    }
}
