//! Error types for Cortex memory system

use thiserror::Error;

/// Errors that can occur in the Cortex memory system
#[derive(Error, Debug)]
pub enum CortexError {
    /// Memory not found with the given ID
    #[error("Memory not found: {0}")]
    NotFound(String),

    /// Error generating or processing embeddings
    #[error("Embedding error: {0}")]
    EmbeddingError(String),

    /// Error in the underlying HNSW storage
    #[error("Storage error: {0}")]
    StorageError(#[from] hnswlib::HnswError),

    /// Invalid input provided
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// LLM processing error
    #[error("LLM error: {0}")]
    LlmError(String),
}

/// Result type for Cortex operations
pub type Result<T> = std::result::Result<T, CortexError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_not_found() {
        let err = CortexError::NotFound("test-id".to_string());
        assert!(err.to_string().contains("not found"));
        assert!(err.to_string().contains("test-id"));
    }

    #[test]
    fn test_error_embedding() {
        let err = CortexError::EmbeddingError("failed to generate".to_string());
        assert!(err.to_string().contains("Embedding error"));
    }

    #[test]
    fn test_error_invalid_input() {
        let err = CortexError::InvalidInput("wrong dimension".to_string());
        assert!(err.to_string().contains("Invalid input"));
    }
}
