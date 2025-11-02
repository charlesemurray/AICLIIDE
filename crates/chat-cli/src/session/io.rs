use std::path::Path;

use super::error::SessionError;
use super::metadata::SessionMetadata;

/// Save metadata to a session directory
///
/// Creates the directory if it doesn't exist and writes metadata.json atomically.
pub async fn save_metadata(session_dir: &Path, metadata: &SessionMetadata) -> Result<(), SessionError> {
    tokio::fs::create_dir_all(session_dir).await?;
    let metadata_path = session_dir.join("metadata.json");
    let json = serde_json::to_string_pretty(metadata)?;
    tokio::fs::write(metadata_path, json).await?;
    Ok(())
}

/// Load metadata from a session directory
///
/// Reads and deserializes metadata.json from the session directory.
pub async fn load_metadata(session_dir: &Path) -> Result<SessionMetadata, SessionError> {
    let metadata_path = session_dir.join("metadata.json");
    let json = tokio::fs::read_to_string(metadata_path).await?;
    let metadata = serde_json::from_str(&json)?;
    Ok(metadata)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_save_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let session_dir = temp_dir.path().join("test-session");

        let metadata = SessionMetadata::new("test-id", "First message");
        save_metadata(&session_dir, &metadata).await.unwrap();

        let metadata_path = session_dir.join("metadata.json");
        assert!(metadata_path.exists());
    }

    #[tokio::test]
    async fn test_load_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let session_dir = temp_dir.path().join("test-session");

        let original = SessionMetadata::new("test-id", "First message");
        save_metadata(&session_dir, &original).await.unwrap();

        let loaded = load_metadata(&session_dir).await.unwrap();
        assert_eq!(original.id, loaded.id);
        assert_eq!(original.first_message, loaded.first_message);
        assert_eq!(original.status, loaded.status);
    }

    #[tokio::test]
    async fn test_load_missing_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let session_dir = temp_dir.path().join("nonexistent");

        let result = load_metadata(&session_dir).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_save_creates_directory() {
        let temp_dir = TempDir::new().unwrap();
        let session_dir = temp_dir.path().join("nested").join("test-session");

        assert!(!session_dir.exists());

        let metadata = SessionMetadata::new("test-id", "First message");
        save_metadata(&session_dir, &metadata).await.unwrap();

        assert!(session_dir.exists());
        assert!(session_dir.join("metadata.json").exists());
    }

    #[tokio::test]
    async fn test_save_overwrites_existing() {
        let temp_dir = TempDir::new().unwrap();
        let session_dir = temp_dir.path().join("test-session");

        let mut metadata1 = SessionMetadata::new("test-id", "First message");
        save_metadata(&session_dir, &metadata1).await.unwrap();

        metadata1.message_count = 5;
        save_metadata(&session_dir, &metadata1).await.unwrap();

        let loaded = load_metadata(&session_dir).await.unwrap();
        assert_eq!(loaded.message_count, 5);
    }

    #[tokio::test]
    async fn test_load_corrupted_json() {
        let temp_dir = TempDir::new().unwrap();
        let session_dir = temp_dir.path().join("test-session");
        tokio::fs::create_dir_all(&session_dir).await.unwrap();

        let metadata_path = session_dir.join("metadata.json");
        tokio::fs::write(&metadata_path, "corrupted json{{{").await.unwrap();

        let result = load_metadata(&session_dir).await;
        assert!(result.is_err());
        assert!(matches!(result, Err(SessionError::Serialization(_))));
    }

    #[tokio::test]
    async fn test_roundtrip_preserves_all_fields() {
        let temp_dir = TempDir::new().unwrap();
        let session_dir = temp_dir.path().join("test-session");

        let mut original = SessionMetadata::new("test-id", "First message");
        original.message_count = 10;
        original.file_count = 3;
        original.set_name("my-session").unwrap();
        original.archive();

        save_metadata(&session_dir, &original).await.unwrap();
        let loaded = load_metadata(&session_dir).await.unwrap();

        assert_eq!(original.id, loaded.id);
        assert_eq!(original.status, loaded.status);
        assert_eq!(original.first_message, loaded.first_message);
        assert_eq!(original.name, loaded.name);
        assert_eq!(original.message_count, loaded.message_count);
        assert_eq!(original.file_count, loaded.file_count);
    }
}
