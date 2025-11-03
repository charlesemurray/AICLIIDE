/// Configuration for session performance tuning
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// Maximum number of sessions to process in bulk operations
    pub max_bulk_operations: usize,
    
    /// Maximum size of searchable preview (chars)
    pub max_preview_size: usize,
    
    /// Timeout for expensive operations (milliseconds)
    pub operation_timeout_ms: u64,
    
    /// Enable background optimization for hot sessions
    pub enable_background_optimization: bool,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            max_bulk_operations: 50,      // Reasonable limit for bulk ops
            max_preview_size: 500,        // Fast preview size
            operation_timeout_ms: 5000,   // 5 second timeout
            enable_background_optimization: false, // Conservative default
        }
    }
}

impl SessionConfig {
    /// Create config optimized for performance
    pub fn performance_optimized() -> Self {
        Self {
            max_bulk_operations: 20,      // Lower for better responsiveness
            max_preview_size: 200,        // Smaller for faster search
            operation_timeout_ms: 2000,   // Shorter timeout
            enable_background_optimization: true,
        }
    }
    
    /// Create config optimized for completeness
    pub fn completeness_optimized() -> Self {
        Self {
            max_bulk_operations: 100,     // Higher for power users
            max_preview_size: 1000,       // Larger for better search
            operation_timeout_ms: 10000,  // Longer timeout
            enable_background_optimization: true,
        }
    }
}
