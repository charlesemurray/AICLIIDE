//! API rate limiting for multi-session support

use std::sync::Arc;
use std::time::{
    Duration,
    Instant,
};

use tokio::sync::Semaphore;

/// Rate limiter for API calls across sessions
pub struct ApiRateLimiter {
    /// Semaphore to limit concurrent API calls
    semaphore: Arc<Semaphore>,
    /// Maximum concurrent calls allowed
    max_concurrent: usize,
    /// Track active calls for monitoring
    active_calls: Arc<tokio::sync::Mutex<usize>>,
}

impl ApiRateLimiter {
    /// Create a new rate limiter
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            max_concurrent,
            active_calls: Arc::new(tokio::sync::Mutex::new(0)),
        }
    }

    /// Acquire a permit to make an API call
    /// Returns a guard that releases the permit when dropped
    pub async fn acquire(&self) -> RateLimitGuard {
        let permit = self.semaphore.clone().acquire_owned().await.expect("Semaphore closed");

        // Track active calls
        let mut active = self.active_calls.lock().await;
        *active += 1;

        RateLimitGuard {
            _permit: permit,
            active_calls: self.active_calls.clone(),
            acquired_at: Instant::now(),
        }
    }

    /// Try to acquire a permit without waiting
    /// Returns None if no permits available
    pub fn try_acquire(&self) -> Option<RateLimitGuard> {
        self.semaphore.clone().try_acquire_owned().ok().map(|permit| {
            // Note: Can't easily track active calls in sync context
            RateLimitGuard {
                _permit: permit,
                active_calls: self.active_calls.clone(),
                acquired_at: Instant::now(),
            }
        })
    }

    /// Get current number of active API calls
    pub async fn active_count(&self) -> usize {
        *self.active_calls.lock().await
    }

    /// Get available permits
    pub fn available_permits(&self) -> usize {
        self.semaphore.available_permits()
    }

    /// Get maximum concurrent calls
    pub fn max_concurrent(&self) -> usize {
        self.max_concurrent
    }
}

impl Clone for ApiRateLimiter {
    fn clone(&self) -> Self {
        Self {
            semaphore: self.semaphore.clone(),
            max_concurrent: self.max_concurrent,
            active_calls: self.active_calls.clone(),
        }
    }
}

/// Guard that releases rate limit permit when dropped
pub struct RateLimitGuard {
    _permit: tokio::sync::OwnedSemaphorePermit,
    active_calls: Arc<tokio::sync::Mutex<usize>>,
    acquired_at: Instant,
}

impl RateLimitGuard {
    /// Get how long this guard has been held
    pub fn duration(&self) -> Duration {
        self.acquired_at.elapsed()
    }
}

impl Drop for RateLimitGuard {
    fn drop(&mut self) {
        // Decrement active calls
        // Note: This is best-effort, may not execute if tokio runtime is shutting down
        let active_calls = self.active_calls.clone();
        tokio::spawn(async move {
            let mut active = active_calls.lock().await;
            *active = active.saturating_sub(1);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_basic() {
        let limiter = ApiRateLimiter::new(5);
        assert_eq!(limiter.max_concurrent(), 5);
        assert_eq!(limiter.available_permits(), 5);
    }

    #[tokio::test]
    async fn test_acquire_and_release() {
        let limiter = ApiRateLimiter::new(2);

        let _guard1 = limiter.acquire().await;
        assert_eq!(limiter.available_permits(), 1);

        let _guard2 = limiter.acquire().await;
        assert_eq!(limiter.available_permits(), 0);

        drop(_guard1);
        tokio::time::sleep(Duration::from_millis(10)).await;
        assert_eq!(limiter.available_permits(), 1);
    }

    #[tokio::test]
    async fn test_try_acquire() {
        let limiter = ApiRateLimiter::new(1);

        let _guard1 = limiter.try_acquire();
        assert!(_guard1.is_some());

        let guard2 = limiter.try_acquire();
        assert!(guard2.is_none());
    }

    #[tokio::test]
    async fn test_active_count() {
        let limiter = ApiRateLimiter::new(5);

        let _guard1 = limiter.acquire().await;
        tokio::time::sleep(Duration::from_millis(10)).await;
        assert_eq!(limiter.active_count().await, 1);

        let _guard2 = limiter.acquire().await;
        tokio::time::sleep(Duration::from_millis(10)).await;
        assert_eq!(limiter.active_count().await, 2);
    }

    #[tokio::test]
    async fn test_guard_duration() {
        let limiter = ApiRateLimiter::new(1);
        let guard = limiter.acquire().await;

        tokio::time::sleep(Duration::from_millis(50)).await;

        let duration = guard.duration();
        assert!(duration >= Duration::from_millis(50));
    }

    #[tokio::test]
    async fn test_concurrent_limit() {
        let limiter = ApiRateLimiter::new(2);

        let guard1 = limiter.acquire().await;
        let guard2 = limiter.acquire().await;

        // Third acquire should wait
        let limiter_clone = limiter.clone();
        let handle = tokio::spawn(async move {
            let _guard3 = limiter_clone.acquire().await;
            "acquired"
        });

        // Give it time to try
        tokio::time::sleep(Duration::from_millis(10)).await;
        assert!(!handle.is_finished());

        // Release one
        drop(guard1);

        // Now it should complete
        let result = tokio::time::timeout(Duration::from_millis(100), handle).await;
        assert!(result.is_ok());

        drop(guard2);
    }

    #[tokio::test]
    async fn test_clone() {
        let limiter = ApiRateLimiter::new(3);
        let limiter_clone = limiter.clone();

        let _guard1 = limiter.acquire().await;
        assert_eq!(limiter_clone.available_permits(), 2);
    }
}
