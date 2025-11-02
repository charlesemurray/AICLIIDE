//! Memory note data structure

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A memory note containing content and metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryNote {
    /// Unique identifier for this memory
    pub id: String,
    /// The actual content of the memory
    pub content: String,
    /// Metadata associated with this memory
    pub metadata: HashMap<String, Value>,
    /// When this memory was created
    pub created_at: DateTime<Utc>,
    /// When this memory was last updated
    pub updated_at: DateTime<Utc>,
}

impl MemoryNote {
    /// Create a new memory note
    pub fn new(id: String, content: String, metadata: HashMap<String, Value>) -> Self {
        let now = Utc::now();
        Self {
            id,
            content,
            metadata,
            created_at: now,
            updated_at: now,
        }
    }

    /// Get keywords from metadata
    pub fn keywords(&self) -> Vec<String> {
        self.metadata
            .get("keywords")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default()
    }

    /// Get context from metadata, defaults to "General"
    pub fn context(&self) -> String {
        self.metadata
            .get("context")
            .and_then(|v| v.as_str())
            .unwrap_or("General")
            .to_string()
    }

    /// Get tags from metadata
    pub fn tags(&self) -> Vec<String> {
        self.metadata
            .get("tags")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default()
    }

    /// Get category from metadata
    pub fn category(&self) -> Option<String> {
        self.metadata.get("category").and_then(|v| v.as_str()).map(String::from)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_memory_note_creation() {
        let note = MemoryNote::new("test-id".to_string(), "test content".to_string(), HashMap::new());

        assert_eq!(note.id, "test-id");
        assert_eq!(note.content, "test content");
        assert_eq!(note.context(), "General");
        assert!(note.keywords().is_empty());
        assert!(note.tags().is_empty());
        assert!(note.category().is_none());
    }

    #[test]
    fn test_memory_note_with_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("keywords".to_string(), json!(["rust", "memory"]));
        metadata.insert("context".to_string(), json!("programming"));
        metadata.insert("tags".to_string(), json!(["important", "work"]));
        metadata.insert("category".to_string(), json!("work.programming.rust"));

        let note = MemoryNote::new("id".to_string(), "content".to_string(), metadata);

        assert_eq!(note.keywords(), vec!["rust", "memory"]);
        assert_eq!(note.context(), "programming");
        assert_eq!(note.tags(), vec!["important", "work"]);
        assert_eq!(note.category(), Some("work.programming.rust".to_string()));
    }

    #[test]
    fn test_memory_note_serialization() {
        let note = MemoryNote::new("id".to_string(), "content".to_string(), HashMap::new());

        let json = serde_json::to_string(&note).unwrap();
        let deserialized: MemoryNote = serde_json::from_str(&json).unwrap();

        assert_eq!(note, deserialized);
    }

    #[test]
    fn test_memory_note_timestamps() {
        let note = MemoryNote::new("id".to_string(), "content".to_string(), HashMap::new());

        assert_eq!(note.created_at, note.updated_at);
        assert!(note.created_at <= Utc::now());
    }

    #[test]
    fn test_memory_note_empty_arrays() {
        let mut metadata = HashMap::new();
        metadata.insert("keywords".to_string(), json!([]));
        metadata.insert("tags".to_string(), json!([]));

        let note = MemoryNote::new("id".to_string(), "content".to_string(), metadata);

        assert!(note.keywords().is_empty());
        assert!(note.tags().is_empty());
    }

    #[test]
    fn test_memory_note_invalid_metadata_types() {
        let mut metadata = HashMap::new();
        metadata.insert("keywords".to_string(), json!("not an array"));
        metadata.insert("tags".to_string(), json!(123));

        let note = MemoryNote::new("id".to_string(), "content".to_string(), metadata);

        assert!(note.keywords().is_empty());
        assert!(note.tags().is_empty());
    }
}
