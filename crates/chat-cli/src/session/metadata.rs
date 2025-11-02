use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use time::OffsetDateTime;

use super::error::SessionError;

/// Current metadata schema version
pub const METADATA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionStatus {
    Active,
    Background,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    /// Schema version for backwards compatibility
    #[serde(default = "default_version")]
    pub version: u32,

    /// Unique session identifier
    pub id: String,

    /// Current session status
    pub status: SessionStatus,

    /// When the session was created
    #[serde(with = "time::serde::rfc3339")]
    pub created: OffsetDateTime,

    /// Last activity timestamp
    #[serde(with = "time::serde::rfc3339")]
    pub last_active: OffsetDateTime,

    /// First message in the session
    pub first_message: String,

    /// Optional user-assigned name
    pub name: Option<String>,

    /// Number of files in session directory
    pub file_count: usize,

    /// Number of messages exchanged
    pub message_count: usize,

    /// Extensibility - custom fields for future use
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub custom_fields: HashMap<String, serde_json::Value>,
}

fn default_version() -> u32 {
    METADATA_VERSION
}

impl SessionMetadata {
    /// Create a new session metadata
    pub fn new(id: impl Into<String>, first_message: impl Into<String>) -> Self {
        let now = OffsetDateTime::now_utc();
        Self {
            version: METADATA_VERSION,
            id: id.into(),
            status: SessionStatus::Active,
            created: now,
            last_active: now,
            first_message: first_message.into(),
            name: None,
            file_count: 0,
            message_count: 0,
            custom_fields: HashMap::new(),
        }
    }

    /// Archive this session
    pub fn archive(&mut self) {
        self.status = SessionStatus::Archived;
        self.last_active = OffsetDateTime::now_utc();
    }

    /// Update last activity timestamp
    pub fn update_activity(&mut self) {
        self.last_active = OffsetDateTime::now_utc();
    }

    /// Set session name with validation
    pub fn set_name(&mut self, name: impl Into<String>) -> Result<(), SessionError> {
        let name = name.into();
        validate_session_name(&name)?;
        self.name = Some(name);
        self.update_activity();
        Ok(())
    }

    /// Migrate metadata to current version
    pub fn migrate(mut self) -> Result<Self, SessionError> {
        match self.version {
            0 => {
                // V0 -> V1: Add custom_fields
                self.custom_fields = HashMap::new();
                self.version = 1;
                Ok(self)
            },
            1 => {
                // Current version
                Ok(self)
            },
            v => Err(SessionError::InvalidMetadata(format!("Unknown schema version: {}", v))),
        }
    }
}

/// Validate a session name
///
/// # Rules
/// - 1-100 characters
/// - Alphanumeric, dash, underscore only
///
/// # Errors
/// Returns `SessionError::InvalidName` if validation fails
pub fn validate_session_name(name: &str) -> Result<(), SessionError> {
    if name.is_empty() {
        return Err(SessionError::InvalidName("Name cannot be empty".into()));
    }

    if name.len() > 100 {
        return Err(SessionError::InvalidName("Name too long (max 100 characters)".into()));
    }

    if !name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err(SessionError::InvalidName(
            "Only alphanumeric, dash, and underscore allowed".into(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_creation() {
        let metadata = SessionMetadata::new("test-id", "First message");

        assert_eq!(metadata.id, "test-id");
        assert_eq!(metadata.version, METADATA_VERSION);
        assert_eq!(metadata.status, SessionStatus::Active);
        assert_eq!(metadata.first_message, "First message");
        assert_eq!(metadata.message_count, 0);
        assert_eq!(metadata.file_count, 0);
        assert!(metadata.name.is_none());
        assert!(metadata.custom_fields.is_empty());
    }

    #[test]
    fn test_metadata_serialization() {
        let metadata = SessionMetadata::new("test-id", "First message");
        let json = serde_json::to_string(&metadata).unwrap();
        let deserialized: SessionMetadata = serde_json::from_str(&json).unwrap();

        assert_eq!(metadata.id, deserialized.id);
        assert_eq!(metadata.version, deserialized.version);
        assert_eq!(metadata.first_message, deserialized.first_message);
        assert_eq!(metadata.status, deserialized.status);
    }

    #[test]
    fn test_status_transitions() {
        let mut metadata = SessionMetadata::new("test-id", "First");

        assert_eq!(metadata.status, SessionStatus::Active);

        metadata.archive();
        assert_eq!(metadata.status, SessionStatus::Archived);
    }

    #[test]
    fn test_update_activity() {
        let mut metadata = SessionMetadata::new("test-id", "First");
        let original_time = metadata.last_active;

        std::thread::sleep(std::time::Duration::from_millis(10));
        metadata.update_activity();

        assert!(metadata.last_active > original_time);
    }

    #[test]
    fn test_validate_name_valid() {
        assert!(validate_session_name("my-feature").is_ok());
        assert!(validate_session_name("feature_123").is_ok());
        assert!(validate_session_name("ABC-def_123").is_ok());
        assert!(validate_session_name("a").is_ok());
        assert!(validate_session_name(&"a".repeat(100)).is_ok());
    }

    #[test]
    fn test_validate_name_invalid() {
        assert!(validate_session_name("").is_err());
        assert!(validate_session_name(&"a".repeat(101)).is_err());
        assert!(validate_session_name("my feature").is_err()); // space
        assert!(validate_session_name("my/feature").is_err()); // slash
        assert!(validate_session_name("my.feature").is_err()); // dot
        assert!(validate_session_name("my@feature").is_err()); // special char
    }

    #[test]
    fn test_set_name_validation() {
        let mut metadata = SessionMetadata::new("test", "First");

        assert!(metadata.set_name("valid-name").is_ok());
        assert_eq!(metadata.name, Some("valid-name".to_string()));

        assert!(metadata.set_name("invalid name").is_err());
        assert_eq!(metadata.name, Some("valid-name".to_string())); // unchanged
    }

    #[test]
    fn test_schema_migration_v0_to_v1() {
        // Simulate V0 metadata (no custom_fields)
        let json = r#"{
            "version": 0,
            "id": "test",
            "status": "active",
            "created": "2025-01-01T00:00:00Z",
            "last_active": "2025-01-01T00:00:00Z",
            "first_message": "Test",
            "name": null,
            "file_count": 0,
            "message_count": 0
        }"#;

        let mut metadata: SessionMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(metadata.version, 0);

        metadata = metadata.migrate().unwrap();
        assert_eq!(metadata.version, 1);
        assert!(metadata.custom_fields.is_empty());
    }

    #[test]
    fn test_schema_migration_v1_no_change() {
        let metadata = SessionMetadata::new("test", "First");
        assert_eq!(metadata.version, 1);

        let migrated = metadata.migrate().unwrap();
        assert_eq!(migrated.version, 1);
    }

    #[test]
    fn test_schema_migration_unknown_version() {
        let mut metadata = SessionMetadata::new("test", "First");
        metadata.version = 999;

        let result = metadata.migrate();
        assert!(result.is_err());
        assert!(matches!(result, Err(SessionError::InvalidMetadata(_))));
    }

    #[test]
    fn test_archive_updates_timestamp() {
        let mut metadata = SessionMetadata::new("test", "First");
        let original_time = metadata.last_active;

        std::thread::sleep(std::time::Duration::from_millis(10));
        metadata.archive();

        assert!(metadata.last_active > original_time);
        assert_eq!(metadata.status, SessionStatus::Archived);
    }

    #[test]
    fn test_set_name_updates_timestamp() {
        let mut metadata = SessionMetadata::new("test", "First");
        let original_time = metadata.last_active;

        std::thread::sleep(std::time::Duration::from_millis(10));
        metadata.set_name("new-name").unwrap();

        assert!(metadata.last_active > original_time);
    }
}
