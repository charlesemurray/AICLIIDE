//! Fair scheduler for background LLM processing
//!
//! Ensures each background session gets fair access to rate-limited API calls
//! while preserving priority for the active session.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Fair scheduler for distributing API permits among background sessions
pub struct FairScheduler {
    /// Queue of sessions waiting for permits (round-robin)
    waiting_queue: Arc<Mutex<VecDeque<String>>>,
    /// Track which sessions currently have permits
    active_sessions: Arc<Mutex<HashMap<String, usize>>>,
    /// Last session that was served (for fairness)
    last_served: Arc<Mutex<Option<String>>>,
}

impl FairScheduler {
    pub fn new() -> Self {
        Self {
            waiting_queue: Arc::new(Mutex::new(VecDeque::new())),
            active_sessions: Arc::new(Mutex::new(HashMap::new())),
            last_served: Arc::new(Mutex::new(None)),
        }
    }
    
    /// Register a session as waiting for a permit
    /// Returns true if session was added, false if already waiting
    pub async fn register_waiting(&self, session_id: String) -> bool {
        let mut queue = self.waiting_queue.lock().await;
        
        // Don't add if already in queue
        if queue.contains(&session_id) {
            return false;
        }
        
        // Add to end of queue
        queue.push_back(session_id);
        true
    }
    
    /// Get next session that should receive a permit (fair round-robin)
    /// Returns None if no sessions waiting
    pub async fn next_session(&self) -> Option<String> {
        let mut queue = self.waiting_queue.lock().await;
        let mut last_served = self.last_served.lock().await;
        
        // If queue empty, no one waiting
        if queue.is_empty() {
            return None;
        }
        
        // Round-robin: if last served is still in queue, start after it
        if let Some(ref last) = *last_served {
            if let Some(pos) = queue.iter().position(|s| s == last) {
                // Rotate queue so we start after last served
                queue.rotate_left(pos + 1);
            }
        }
        
        // Take first session from queue
        let session_id = queue.pop_front()?;
        *last_served = Some(session_id.clone());
        
        Some(session_id)
    }
    
    /// Mark session as having acquired a permit
    pub async fn mark_active(&self, session_id: String) {
        let mut active = self.active_sessions.lock().await;
        *active.entry(session_id).or_insert(0) += 1;
    }
    
    /// Mark session as having released a permit
    pub async fn mark_inactive(&self, session_id: String) {
        let mut active = self.active_sessions.lock().await;
        if let Some(count) = active.get_mut(&session_id) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                active.remove(&session_id);
            }
        }
    }
    
    /// Check if session currently has a permit
    pub async fn is_active(&self, session_id: &str) -> bool {
        let active = self.active_sessions.lock().await;
        active.contains_key(session_id)
    }
    
    /// Get number of sessions currently active
    pub async fn active_count(&self) -> usize {
        let active = self.active_sessions.lock().await;
        active.len()
    }
    
    /// Get number of sessions waiting
    pub async fn waiting_count(&self) -> usize {
        let queue = self.waiting_queue.lock().await;
        queue.len()
    }
    
    /// Remove session from waiting queue (e.g., if cancelled)
    pub async fn remove_waiting(&self, session_id: &str) {
        let mut queue = self.waiting_queue.lock().await;
        queue.retain(|s| s != session_id);
    }
}

impl Default for FairScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for FairScheduler {
    fn clone(&self) -> Self {
        Self {
            waiting_queue: self.waiting_queue.clone(),
            active_sessions: self.active_sessions.clone(),
            last_served: self.last_served.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_round_robin() {
        let scheduler = FairScheduler::new();
        
        // Register 3 sessions
        scheduler.register_waiting("session1".to_string()).await;
        scheduler.register_waiting("session2".to_string()).await;
        scheduler.register_waiting("session3".to_string()).await;
        
        // Should serve in order
        assert_eq!(scheduler.next_session().await, Some("session1".to_string()));
        assert_eq!(scheduler.next_session().await, Some("session2".to_string()));
        assert_eq!(scheduler.next_session().await, Some("session3".to_string()));
        
        // Queue should be empty
        assert_eq!(scheduler.next_session().await, None);
    }
    
    #[tokio::test]
    async fn test_fairness_with_requeue() {
        let scheduler = FairScheduler::new();
        
        // Register sessions
        scheduler.register_waiting("session1".to_string()).await;
        scheduler.register_waiting("session2".to_string()).await;
        
        // Serve session1
        let s1 = scheduler.next_session().await.unwrap();
        assert_eq!(s1, "session1");
        
        // Session1 re-registers (still processing, wants another permit)
        scheduler.register_waiting("session1".to_string()).await;
        
        // Should serve session2 next (fairness)
        assert_eq!(scheduler.next_session().await, Some("session2".to_string()));
        
        // Then session1 again
        assert_eq!(scheduler.next_session().await, Some("session1".to_string()));
    }
    
    #[tokio::test]
    async fn test_active_tracking() {
        let scheduler = FairScheduler::new();
        
        scheduler.mark_active("session1".to_string()).await;
        assert!(scheduler.is_active("session1").await);
        assert_eq!(scheduler.active_count().await, 1);
        
        scheduler.mark_inactive("session1".to_string()).await;
        assert!(!scheduler.is_active("session1").await);
        assert_eq!(scheduler.active_count().await, 0);
    }
    
    #[tokio::test]
    async fn test_no_duplicate_waiting() {
        let scheduler = FairScheduler::new();
        
        assert!(scheduler.register_waiting("session1".to_string()).await);
        assert!(!scheduler.register_waiting("session1".to_string()).await);
        
        assert_eq!(scheduler.waiting_count().await, 1);
    }
}
