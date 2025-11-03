//! Resource cleanup for preventing memory leaks and file handle exhaustion

use std::sync::Arc;
use std::time::{
    Duration,
    Instant,
};

use tokio::sync::Mutex;

/// Resource usage statistics
#[derive(Debug, Clone, Default)]
pub struct ResourceStats {
    pub active_sessions: usize,
    pub total_buffer_bytes: usize,
    pub open_file_handles: usize,
    pub last_cleanup: Option<Instant>,
}

/// Manages resource cleanup for sessions
pub struct ResourceCleanupManager {
    stats: Arc<Mutex<ResourceStats>>,
    cleanup_interval: Duration,
}

impl ResourceCleanupManager {
    /// Create new cleanup manager
    pub fn new(cleanup_interval: Duration) -> Self {
        Self {
            stats: Arc::new(Mutex::new(ResourceStats::default())),
            cleanup_interval,
        }
    }

    /// Update resource statistics
    pub async fn update_stats(&self, active_sessions: usize, total_buffer_bytes: usize) {
        let mut stats = self.stats.lock().await;
        stats.active_sessions = active_sessions;
        stats.total_buffer_bytes = total_buffer_bytes;
    }

    /// Get current resource statistics
    pub async fn get_stats(&self) -> ResourceStats {
        self.stats.lock().await.clone()
    }

    /// Check if cleanup is needed
    pub async fn needs_cleanup(&self) -> bool {
        let stats = self.stats.lock().await;
        match stats.last_cleanup {
            None => true,
            Some(last) => last.elapsed() >= self.cleanup_interval,
        }
    }

    /// Mark cleanup as performed
    pub async fn mark_cleanup_done(&self) {
        let mut stats = self.stats.lock().await;
        stats.last_cleanup = Some(Instant::now());
    }

    /// Check for resource leaks
    pub async fn check_for_leaks(&self) -> Vec<String> {
        let stats = self.stats.lock().await;
        let mut warnings = Vec::new();

        // Check for excessive buffer usage (>100MB)
        if stats.total_buffer_bytes > 100 * 1024 * 1024 {
            warnings.push(format!(
                "High buffer usage: {} MB",
                stats.total_buffer_bytes / (1024 * 1024)
            ));
        }

        // Check for too many active sessions
        if stats.active_sessions > 20 {
            warnings.push(format!("High session count: {} active sessions", stats.active_sessions));
        }

        warnings
    }

    /// Get cleanup recommendations
    pub async fn get_recommendations(&self) -> Vec<String> {
        let stats = self.stats.lock().await;
        let mut recommendations = Vec::new();

        if stats.total_buffer_bytes > 50 * 1024 * 1024 {
            recommendations.push("Consider closing inactive sessions to free memory".to_string());
        }

        if stats.active_sessions > 10 {
            recommendations.push("Archive completed sessions to reduce resource usage".to_string());
        }

        recommendations
    }
}

impl Default for ResourceCleanupManager {
    fn default() -> Self {
        Self::new(Duration::from_secs(60))
    }
}

#[cfg(test)]
mod tests {
    use tokio::time::sleep;

    use super::*;

    #[tokio::test]
    async fn test_update_stats() {
        let manager = ResourceCleanupManager::default();

        manager.update_stats(5, 1024 * 1024).await;

        let stats = manager.get_stats().await;
        assert_eq!(stats.active_sessions, 5);
        assert_eq!(stats.total_buffer_bytes, 1024 * 1024);
    }

    #[tokio::test]
    async fn test_needs_cleanup_initially() {
        let manager = ResourceCleanupManager::default();

        assert!(manager.needs_cleanup().await);
    }

    #[tokio::test]
    async fn test_needs_cleanup_after_mark() {
        let manager = ResourceCleanupManager::default();

        manager.mark_cleanup_done().await;
        assert!(!manager.needs_cleanup().await);
    }

    #[tokio::test]
    async fn test_needs_cleanup_after_interval() {
        let manager = ResourceCleanupManager::new(Duration::from_millis(50));

        manager.mark_cleanup_done().await;
        assert!(!manager.needs_cleanup().await);

        sleep(Duration::from_millis(100)).await;
        assert!(manager.needs_cleanup().await);
    }

    #[tokio::test]
    async fn test_check_for_leaks_high_buffer() {
        let manager = ResourceCleanupManager::default();

        manager.update_stats(5, 150 * 1024 * 1024).await;

        let warnings = manager.check_for_leaks().await;
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("High buffer usage"));
    }

    #[tokio::test]
    async fn test_check_for_leaks_high_sessions() {
        let manager = ResourceCleanupManager::default();

        manager.update_stats(25, 1024).await;

        let warnings = manager.check_for_leaks().await;
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("High session count"));
    }

    #[tokio::test]
    async fn test_check_for_leaks_both() {
        let manager = ResourceCleanupManager::default();

        manager.update_stats(25, 150 * 1024 * 1024).await;

        let warnings = manager.check_for_leaks().await;
        assert_eq!(warnings.len(), 2);
    }

    #[tokio::test]
    async fn test_get_recommendations() {
        let manager = ResourceCleanupManager::default();

        manager.update_stats(15, 60 * 1024 * 1024).await;

        let recommendations = manager.get_recommendations().await;
        assert_eq!(recommendations.len(), 2);
    }

    #[tokio::test]
    async fn test_no_recommendations_when_healthy() {
        let manager = ResourceCleanupManager::default();

        manager.update_stats(5, 10 * 1024 * 1024).await;

        let recommendations = manager.get_recommendations().await;
        assert_eq!(recommendations.len(), 0);
    }
}
