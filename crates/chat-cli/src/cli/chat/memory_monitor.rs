//! Memory monitoring for multi-session support

use std::sync::Arc;

use tokio::sync::Mutex;

/// Memory statistics for a session
#[derive(Debug, Clone, Default)]
pub struct SessionMemoryStats {
    /// Output buffer size in bytes
    pub buffer_size: usize,
    /// Estimated conversation state size in bytes
    pub conversation_size: usize,
    /// Total estimated memory usage
    pub total_bytes: usize,
}

impl SessionMemoryStats {
    /// Create new stats
    pub fn new(buffer_size: usize, conversation_size: usize) -> Self {
        Self {
            buffer_size,
            conversation_size,
            total_bytes: buffer_size + conversation_size,
        }
    }

    /// Check if memory usage exceeds threshold
    pub fn exceeds_threshold(&self, threshold_bytes: usize) -> bool {
        self.total_bytes > threshold_bytes
    }

    /// Get memory usage in MB
    pub fn total_mb(&self) -> f64 {
        self.total_bytes as f64 / (1024.0 * 1024.0)
    }
}

/// Memory monitor for tracking session memory usage
pub struct MemoryMonitor {
    /// Memory stats per session
    stats: Arc<Mutex<std::collections::HashMap<String, SessionMemoryStats>>>,
    /// Warning threshold in bytes (default: 50 MB per session)
    warning_threshold: usize,
    /// Critical threshold in bytes (default: 75 MB per session)
    critical_threshold: usize,
}

impl MemoryMonitor {
    /// Create a new memory monitor
    pub fn new() -> Self {
        Self {
            stats: Arc::new(Mutex::new(std::collections::HashMap::new())),
            warning_threshold: 50 * 1024 * 1024,  // 50 MB
            critical_threshold: 75 * 1024 * 1024, // 75 MB
        }
    }

    /// Create with custom thresholds
    pub fn with_thresholds(warning_mb: usize, critical_mb: usize) -> Self {
        Self {
            stats: Arc::new(Mutex::new(std::collections::HashMap::new())),
            warning_threshold: warning_mb * 1024 * 1024,
            critical_threshold: critical_mb * 1024 * 1024,
        }
    }

    /// Update memory stats for a session
    pub async fn update_session(&self, session_id: &str, buffer_size: usize, conversation_size: usize) {
        let mut stats = self.stats.lock().await;
        stats.insert(
            session_id.to_string(),
            SessionMemoryStats::new(buffer_size, conversation_size),
        );
    }

    /// Remove session from monitoring
    pub async fn remove_session(&self, session_id: &str) {
        let mut stats = self.stats.lock().await;
        stats.remove(session_id);
    }

    /// Get memory stats for a session
    pub async fn get_session_stats(&self, session_id: &str) -> Option<SessionMemoryStats> {
        let stats = self.stats.lock().await;
        stats.get(session_id).cloned()
    }

    /// Get total memory usage across all sessions
    pub async fn total_memory_usage(&self) -> usize {
        let stats = self.stats.lock().await;
        stats.values().map(|s| s.total_bytes).sum()
    }

    /// Get total memory usage in MB
    pub async fn total_memory_mb(&self) -> f64 {
        let total = self.total_memory_usage().await;
        total as f64 / (1024.0 * 1024.0)
    }

    /// Check if any session exceeds warning threshold
    pub async fn check_warnings(&self) -> Vec<(String, SessionMemoryStats)> {
        let stats = self.stats.lock().await;
        stats
            .iter()
            .filter(|(_, s)| s.exceeds_threshold(self.warning_threshold))
            .map(|(id, s)| (id.clone(), s.clone()))
            .collect()
    }

    /// Check if any session exceeds critical threshold
    pub async fn check_critical(&self) -> Vec<(String, SessionMemoryStats)> {
        let stats = self.stats.lock().await;
        stats
            .iter()
            .filter(|(_, s)| s.exceeds_threshold(self.critical_threshold))
            .map(|(id, s)| (id.clone(), s.clone()))
            .collect()
    }

    /// Get sessions that should be hibernated (exceed critical threshold)
    pub async fn sessions_to_hibernate(&self) -> Vec<String> {
        self.check_critical().await.into_iter().map(|(id, _)| id).collect()
    }

    /// Get memory summary
    pub async fn summary(&self) -> MemorySummary {
        let stats = self.stats.lock().await;
        let total = stats.values().map(|s| s.total_bytes).sum();
        let session_count = stats.len();
        let avg = if session_count > 0 { total / session_count } else { 0 };

        MemorySummary {
            total_bytes: total,
            session_count,
            average_bytes: avg,
            warning_count: stats
                .values()
                .filter(|s| s.exceeds_threshold(self.warning_threshold))
                .count(),
            critical_count: stats
                .values()
                .filter(|s| s.exceeds_threshold(self.critical_threshold))
                .count(),
        }
    }
}

impl Clone for MemoryMonitor {
    fn clone(&self) -> Self {
        Self {
            stats: self.stats.clone(),
            warning_threshold: self.warning_threshold,
            critical_threshold: self.critical_threshold,
        }
    }
}

impl Default for MemoryMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Memory usage summary
#[derive(Debug, Clone)]
pub struct MemorySummary {
    pub total_bytes: usize,
    pub session_count: usize,
    pub average_bytes: usize,
    pub warning_count: usize,
    pub critical_count: usize,
}

impl MemorySummary {
    /// Get total memory in MB
    pub fn total_mb(&self) -> f64 {
        self.total_bytes as f64 / (1024.0 * 1024.0)
    }

    /// Get average memory per session in MB
    pub fn average_mb(&self) -> f64 {
        self.average_bytes as f64 / (1024.0 * 1024.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_memory_stats() {
        let stats = SessionMemoryStats::new(1024, 2048);
        assert_eq!(stats.buffer_size, 1024);
        assert_eq!(stats.conversation_size, 2048);
        assert_eq!(stats.total_bytes, 3072);
    }

    #[tokio::test]
    async fn test_memory_monitor_update() {
        let monitor = MemoryMonitor::new();
        monitor.update_session("test-1", 1024, 2048).await;

        let stats = monitor.get_session_stats("test-1").await;
        assert!(stats.is_some());
        assert_eq!(stats.unwrap().total_bytes, 3072);
    }

    #[tokio::test]
    async fn test_total_memory_usage() {
        let monitor = MemoryMonitor::new();
        monitor.update_session("test-1", 1024, 2048).await;
        monitor.update_session("test-2", 2048, 4096).await;

        let total = monitor.total_memory_usage().await;
        assert_eq!(total, 3072 + 6144);
    }

    #[tokio::test]
    async fn test_warning_threshold() {
        let monitor = MemoryMonitor::with_thresholds(1, 2); // 1 MB warning, 2 MB critical

        // Below threshold
        monitor.update_session("test-1", 512 * 1024, 0).await;
        let warnings = monitor.check_warnings().await;
        assert_eq!(warnings.len(), 0);

        // Above threshold
        monitor.update_session("test-2", 2 * 1024 * 1024, 0).await;
        let warnings = monitor.check_warnings().await;
        assert_eq!(warnings.len(), 1);
    }

    #[tokio::test]
    async fn test_critical_threshold() {
        let monitor = MemoryMonitor::with_thresholds(1, 2);

        monitor.update_session("test-1", 3 * 1024 * 1024, 0).await;
        let critical = monitor.check_critical().await;
        assert_eq!(critical.len(), 1);
    }

    #[tokio::test]
    async fn test_remove_session() {
        let monitor = MemoryMonitor::new();
        monitor.update_session("test-1", 1024, 2048).await;

        assert!(monitor.get_session_stats("test-1").await.is_some());

        monitor.remove_session("test-1").await;
        assert!(monitor.get_session_stats("test-1").await.is_none());
    }

    #[tokio::test]
    async fn test_memory_summary() {
        let monitor = MemoryMonitor::with_thresholds(1, 2);
        monitor.update_session("test-1", 512 * 1024, 512 * 1024).await; // 1 MB
        monitor.update_session("test-2", 2 * 1024 * 1024, 0).await; // 2 MB (warning)
        monitor.update_session("test-3", 3 * 1024 * 1024, 0).await; // 3 MB (critical)

        let summary = monitor.summary().await;
        assert_eq!(summary.session_count, 3);
        assert_eq!(summary.warning_count, 2); // test-2 and test-3
        assert_eq!(summary.critical_count, 1); // test-3
    }

    #[tokio::test]
    async fn test_sessions_to_hibernate() {
        let monitor = MemoryMonitor::with_thresholds(1, 2);
        monitor.update_session("test-1", 512 * 1024, 0).await;
        monitor.update_session("test-2", 3 * 1024 * 1024, 0).await;

        let to_hibernate = monitor.sessions_to_hibernate().await;
        assert_eq!(to_hibernate.len(), 1);
        assert!(to_hibernate.contains(&"test-2".to_string()));
    }
}
