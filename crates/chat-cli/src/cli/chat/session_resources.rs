//! Resource management for sessions (rate limiting, memory, cleanup)

use std::sync::Arc;

use crate::cli::chat::memory_monitor::MemoryMonitor;
use crate::cli::chat::rate_limiter::ApiRateLimiter;
use crate::cli::chat::resource_cleanup::ResourceCleanupManager;

/// Resource manager for sessions
pub struct SessionResources {
    rate_limiter: ApiRateLimiter,
    memory_monitor: MemoryMonitor,
    cleanup_manager: ResourceCleanupManager,
    dropped_events: Arc<std::sync::atomic::AtomicUsize>,
}

impl SessionResources {
    pub fn new(max_concurrent_api_calls: usize) -> Self {
        Self {
            rate_limiter: ApiRateLimiter::new(max_concurrent_api_calls),
            memory_monitor: MemoryMonitor::new(),
            cleanup_manager: ResourceCleanupManager::default(),
            dropped_events: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        }
    }

    pub fn rate_limiter(&self) -> ApiRateLimiter {
        self.rate_limiter.clone()
    }

    pub fn memory_monitor(&self) -> MemoryMonitor {
        self.memory_monitor.clone()
    }

    pub fn cleanup_manager(&self) -> &ResourceCleanupManager {
        &self.cleanup_manager
    }

    pub fn dropped_events_count(&self) -> usize {
        self.dropped_events.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn increment_dropped_events(&self) {
        self.dropped_events.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
}
