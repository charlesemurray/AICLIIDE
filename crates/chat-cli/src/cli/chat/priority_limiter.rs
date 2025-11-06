//! Priority-based rate limiting with timeout fallback
//!
//! Foreground (active session) tries priority pool first, falls back to shared pool.
//! Background always uses shared pool.

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;

/// Priority-based rate limiter
#[derive(Clone)]
pub struct PriorityLimiter {
    /// Small pool reserved for foreground (priority)
    priority_semaphore: Arc<Semaphore>,
    /// Large pool shared by all (fallback)
    shared_semaphore: Arc<Semaphore>,
    /// Timeout for priority acquisition
    priority_timeout: Duration,
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
        }
    }
    
    /// Acquire permit for foreground (high priority)
    /// Tries priority pool first, falls back to shared pool if timeout
    pub async fn acquire_foreground(&self) -> PriorityPermit {
        // Try priority pool with timeout
        match tokio::time::timeout(
            self.priority_timeout,
            self.priority_semaphore.acquire()
        ).await {
            Ok(Ok(permit)) => {
                eprintln!("[PRIORITY] Foreground acquired priority permit");
                PriorityPermit::Priority(permit)
            },
            _ => {
                // Timeout or error, fall back to shared
                eprintln!("[PRIORITY] Foreground timeout, using shared pool");
                let permit = self.shared_semaphore.acquire().await.unwrap();
                PriorityPermit::Shared(permit)
            }
        }
    }
    
    /// Acquire permit for background (low priority)
    /// Always uses shared pool
    pub async fn acquire_background(&self) -> PriorityPermit {
        let permit = self.shared_semaphore.acquire().await.unwrap();
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
