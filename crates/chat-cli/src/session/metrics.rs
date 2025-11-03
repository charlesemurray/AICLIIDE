use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Metrics for session operations
#[derive(Clone)]
pub struct SessionMetrics {
    pub list_calls: Arc<AtomicU64>,
    pub list_duration_ms: Arc<AtomicU64>,
    pub archive_calls: Arc<AtomicU64>,
    pub name_calls: Arc<AtomicU64>,
    pub errors: Arc<AtomicU64>,
    pub active_sessions: Arc<AtomicU64>,
}

impl SessionMetrics {
    pub fn new() -> Self {
        Self {
            list_calls: Arc::new(AtomicU64::new(0)),
            list_duration_ms: Arc::new(AtomicU64::new(0)),
            archive_calls: Arc::new(AtomicU64::new(0)),
            name_calls: Arc::new(AtomicU64::new(0)),
            errors: Arc::new(AtomicU64::new(0)),
            active_sessions: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn record_list(&self, duration_ms: u64, count: usize) {
        self.list_calls.fetch_add(1, Ordering::Relaxed);
        self.list_duration_ms.fetch_add(duration_ms, Ordering::Relaxed);
        self.active_sessions.store(count as u64, Ordering::Relaxed);
    }

    pub fn record_archive(&self) {
        self.archive_calls.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_name(&self) {
        self.name_calls.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_error(&self) {
        self.errors.fetch_add(1, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> MetricsSnapshot {
        let list_calls = self.list_calls.load(Ordering::Relaxed);
        MetricsSnapshot {
            list_calls,
            avg_list_duration_ms: if list_calls > 0 {
                self.list_duration_ms.load(Ordering::Relaxed) / list_calls
            } else {
                0
            },
            archive_calls: self.archive_calls.load(Ordering::Relaxed),
            name_calls: self.name_calls.load(Ordering::Relaxed),
            errors: self.errors.load(Ordering::Relaxed),
            active_sessions: self.active_sessions.load(Ordering::Relaxed),
        }
    }
}

impl Default for SessionMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub list_calls: u64,
    pub avg_list_duration_ms: u64,
    pub archive_calls: u64,
    pub name_calls: u64,
    pub errors: u64,
    pub active_sessions: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_recording() {
        let metrics = SessionMetrics::new();
        
        metrics.record_list(100, 5);
        metrics.record_archive();
        metrics.record_name();
        
        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.list_calls, 1);
        assert_eq!(snapshot.avg_list_duration_ms, 100);
        assert_eq!(snapshot.archive_calls, 1);
        assert_eq!(snapshot.name_calls, 1);
        assert_eq!(snapshot.active_sessions, 5);
    }

    #[test]
    fn test_average_duration() {
        let metrics = SessionMetrics::new();
        
        metrics.record_list(100, 5);
        metrics.record_list(200, 10);
        
        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.list_calls, 2);
        assert_eq!(snapshot.avg_list_duration_ms, 150); // (100 + 200) / 2
    }
}
