use std::path::{Path, PathBuf};

/// Resolves a path that may contain the @session/ prefix
///
/// If the path starts with @session/, it will be resolved to:
/// .amazonq/sessions/{conversation_id}/{remaining_path}
///
/// Otherwise, returns the path as-is
pub fn resolve_session_path(path: &str, conversation_id: &str, current_dir: &Path) -> PathBuf {
    if let Some(stripped) = path.strip_prefix("@session/") {
        current_dir
            .join(".amazonq/sessions")
            .join(conversation_id)
            .join(stripped)
    } else {
        PathBuf::from(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_path_resolution() {
        let cwd = PathBuf::from("/workspace");
        let conv_id = "abc-123";

        let result = resolve_session_path("@session/analysis.md", conv_id, &cwd);
        assert_eq!(
            result,
            PathBuf::from("/workspace/.amazonq/sessions/abc-123/analysis.md")
        );
    }

    #[test]
    fn test_regular_path_unchanged() {
        let cwd = PathBuf::from("/workspace");
        let conv_id = "abc-123";

        let result = resolve_session_path("./regular.md", conv_id, &cwd);
        assert_eq!(result, PathBuf::from("./regular.md"));
    }
}
