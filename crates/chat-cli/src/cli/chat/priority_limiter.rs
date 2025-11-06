//! Priority-based rate limiting with timeout fallback
//!
//! Foreground (active session) tries priority pool first, falls back to shared pool.
//! Background always uses shared pool.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;

/// Metrics for priority limiter
#[derive(Default)]
pub struct PriorityMetrics {
    /// Foreground calls that used priority pool
    pub foreground_priority_count: AtomicU64,
    /// Foreground calls that fell back to shared pool
    pub foreground_fallback_count: AtomicU64,
    /// Background calls
    pub background_count: AtomicU64,
    /// Total time spent waiting for priority (microseconds)
    pub priority_wait_time_us: AtomicU64,
    /// Total time spent waiting for shared (microseconds)
    pub shared_wait_time_us: AtomicU64,
}

impl PriorityMetrics {
    /// Get foreground priority hit rate (0.0 to 1.0)
    pub fn priority_hit_rate(&self) -> f64 {
        let priority = self.foreground_priority_count.load(Ordering::Relaxed) as f64;
        let fallback = self.foreground_fallback_count.load(Ordering::Relaxed) as f64;
        let total = priority + fallback;
        if total == 0.0 { 0.0 } else { priority / total }
    }
    
    /// Get average priority wait time (microseconds)
    pub fn avg_priority_wait_us(&self) -> f64 {
        let total_us = self.priority_wait_time_us.load(Ordering::Relaxed) as f64;
        let count = self.foreground_priority_count.load(Ordering::Relaxed) as f64;
        if count == 0.0 { 0.0 } else { total_us / count }
    }
    
    /// Get average shared wait time (microseconds)
    pub fn avg_shared_wait_us(&self) -> f64 {
        let total_us = self.shared_wait_time_us.load(Ordering::Relaxed) as f64;
        let count = (self.foreground_fallback_count.load(Ordering::Relaxed) + 
                     self.background_count.load(Ordering::Relaxed)) as f64;
        if count == 0.0 { 0.0 } else { total_us / count }
    }
    
    /// Print metrics summary
    pub fn print_summary(&self) {
        let priority_count = self.foreground_priority_count.load(Ordering::Relaxed);
        let fallback_count = self.foreground_fallback_count.load(Ordering::Relaxed);
        let background_count = self.background_count.load(Ordering::Relaxed);
        
        eprintln!("[METRICS] Priority Limiter Stats:");
        eprintln!("[METRICS]   Foreground (priority): {}", priority_count);
        eprintln!("[METRICS]   Foreground (fallback): {}", fallback_count);
        eprintln!("[METRICS]   Background: {}", background_count);
        eprintln!("[METRICS]   Priority hit rate: {:.1}%", self.priority_hit_rate() * 100.0);
        eprintln!("[METRICS]   Avg priority wait: {:.0}µs", self.avg_priority_wait_us());
        eprintln!("[METRICS]   Avg shared wait: {:.0}µs", self.avg_shared_wait_us());
    }
}

/// Priority-based rate limiter
#[derive(Clone)]
pub struct PriorityLimiter {
    /// Small pool reserved for foreground (priority)
    priority_semaphore: Arc<Semaphore>,
    /// Large pool shared by all (fallback)
    shared_semaphore: Arc<Semaphore>,
    /// Timeout for priority acquisition
    priority_timeout: Duration,
    /// Metrics
    pub metrics: Arc<PriorityMetrics>,
}

impl PriorityLimiter {
    /// Create new priority limiter
    ///
    /// # Arguments
    /// * `priority_permits` - Reserved for foreground (e.g., 5)
    /// * `shared_permits` - Shared by all (e.g., 15)
    /// * `priority_timeout` - How long foreground waits for priority (e.g., 100ms)
    pub fn new(priority_permits: usize, shared_permits: usize, priority_timeout: Duration) -> Self {
        Self {
            priority_semaphore: Arc::new(Semaphore::new(priority_permits)),
            shared_semaphore: Arc::new(Semaphore::new(shared_permits)),
            priority_timeout,
            metrics: Arc::new(PriorityMetrics::default()),
        }
    }
    
    /// Acquire permit for foreground (high priority)
    /// Tries priority pool first, falls back to shared pool if timeout
    pub async fn acquire_foreground(&self) -> PriorityPermit {
        let start = Instant::now();
        
        // Try priority pool with timeout
        match tokio::time::timeout(
            self.priority_timeout,
            self.priority_semaphore.acquire()
        ).await {
            Ok(Ok(permit)) => {
                let wait_us = start.elapsed().as_micros() as u64;
                self.metrics.foreground_priority_count.fetch_add(1, Ordering::Relaxed);
                self.metrics.priority_wait_time_us.fetch_add(wait_us, Ordering::Relaxed);
                eprintln!("[PRIORITY] Foreground acquired priority permit ({}µs)", wait_us);
                PriorityPermit::Priority(permit)
            },
            _ => {
                // Timeout or error, fall back to shared
                eprintln!("[PRIORITY] Foreground timeout, using shared pool");
                let permit = self.shared_semaphore.acquire().await.unwrap();
                let wait_us = start.elapsed().as_micros() as u64;
                self.metrics.foreground_fallback_count.fetch_add(1, Ordering::Relaxed);
                self.metrics.shared_wait_time_us.fetch_add(wait_us, Ordering::Relaxed);
                PriorityPermit::Shared(permit)
            }
        }
    }
    
    /// Acquire permit for background (low priority)
    /// Always uses shared pool
    pub async fn acquire_background(&self) -> PriorityPermit {
        let start = Instant::now();
        let permit = self.shared_semaphore.acquire().await.unwrap();
        let wait_us = start.elapsed().as_micros() as u64;
        self.metrics.background_count.fetch_add(1, Ordering::Relaxed);
        self.metrics.shared_wait_time_us.fetch_add(wait_us, Ordering::Relaxed);
        PriorityPermit::Shared(permit)
    }
    
    /// Get available permits in priority pool
    pub fn priority_available(&self) -> usize {
        self.priority_semaphore.available_permits()
    }
    
    /// Get available permits in shared pool
    pub fn shared_available(&self) -> usize {
        self.shared_semaphore.available_permits()
    }
}

/// Permit from priority limiter
pub enum PriorityPermit<'a> {
    Priority(tokio::sync::SemaphorePermit<'a>),
    Shared(tokio::sync::SemaphorePermit<'a>),
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_foreground_gets_priority() {
        let limiter = PriorityLimiter::new(2, 3, Duration::from_millis(100));
        
        // Foreground should get priority permit
        let _permit = limiter.acquire_foreground().await;
        assert_eq!(limiter.priority_available(), 1);
        assert_eq!(limiter.shared_available(), 3);
    }
    
    #[tokio::test]
    async fn test_foreground_falls_back() {
        let limiter = PriorityLimiter::new(1, 3, Duration::from_millis(50));
        
        // Use up priority permit
        let _p1 = limiter.acquire_foreground().await;
        
        // Next foreground should timeout and use shared
        let _p2 = limiter.acquire_foreground().await;
        assert_eq!(limiter.shared_available(), 2);
    }
    
    #[tokio::test]
    async fn test_background_uses_shared() {
        let limiter = PriorityLimiter::new(2, 3, Duration::from_millis(100));
        
        // Background always uses shared
        let _permit = limiter.acquire_background().await;
        assert_eq!(limiter.priority_available(), 2);
        assert_eq!(limiter.shared_available(), 2);
    }
}
