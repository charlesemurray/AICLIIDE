//! Priority-aware semaphore for LLM rate limiting
//!
//! Ensures high-priority (active session) requests always go first

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::Semaphore;

/// Priority-aware semaphore wrapper
#[derive(Clone)]
pub struct PrioritySemaphore {
    semaphore: Arc<Semaphore>,
    high_priority_waiting: Arc<AtomicUsize>,
}

impl PrioritySemaphore {
    pub fn new(permits: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(permits)),
            high_priority_waiting: Arc::new(AtomicUsize::new(0)),
        }
    }
    
    /// Acquire permit with high priority (active session)
    /// Always goes first, even if low priority waiting
    pub async fn acquire_high_priority(&self) -> tokio::sync::SemaphorePermit<'_> {
        self.high_priority_waiting.fetch_add(1, Ordering::SeqCst);
        let permit = self.semaphore.acquire().await.unwrap();
        self.high_priority_waiting.fetch_sub(1, Ordering::SeqCst);
        permit
    }
    
    /// Try to acquire permit with low priority (background session)
    /// Only succeeds if no high priority waiting
    pub async fn acquire_low_priority(&self) -> tokio::sync::SemaphorePermit<'_> {
        loop {
            // Check if high priority waiting
            if self.high_priority_waiting.load(Ordering::SeqCst) > 0 {
                // High priority waiting, yield and retry
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                continue;
            }
            
            // Try to acquire
            if let Ok(permit) = self.semaphore.try_acquire() {
                return permit;
            }
            
            // No permits available, wait and retry
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
    }
    
    /// Get available permits
    pub fn available_permits(&self) -> usize {
        self.semaphore.available_permits()
    }
}
