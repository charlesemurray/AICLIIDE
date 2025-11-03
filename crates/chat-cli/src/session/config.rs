/// Configuration for session performance tuning
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// Maximum number of sessions to process in bulk operations
    pub max_bulk_operations: usize,
    
    /// Maximum size of searchable preview (chars)
    pub max_preview_size: usize,
    
    /// Timeout for expensive operations (milliseconds)
    pub operation_timeout_ms: u64,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            max_bulk_operations: 50,      // Reasonable limit for bulk ops
            max_preview_size: 500,        // Fast preview size
            operation_timeout_ms: 5000,   // 5 second timeout
        }
    }
}
