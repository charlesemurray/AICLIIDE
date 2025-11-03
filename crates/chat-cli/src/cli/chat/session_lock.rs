//! Session lock manager for preventing race conditions

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{
    Duration,
    Instant,
};

use eyre::{
    Result,
    bail,
};
use tokio::sync::Mutex;

/// Lock guard for a session
#[derive(Debug)]
pub struct SessionLockGuard {
    session_id: String,
    locks: Arc<Mutex<HashMap<String, LockInfo>>>,
}

impl Drop for SessionLockGuard {
    fn drop(&mut self) {
        // Release lock asynchronously
        let session_id = self.session_id.clone();
        let locks = self.locks.clone();
        tokio::spawn(async move {
            let mut locks = locks.lock().await;
            locks.remove(&session_id);
        });
    }
}

#[derive(Debug, Clone)]
struct LockInfo {
    acquired_at: Instant,
    holder: String,
}

/// Manages locks for concurrent session access
pub struct SessionLockManager {
    locks: Arc<Mutex<HashMap<String, LockInfo>>>,
    lock_timeout: Duration,
}

impl SessionLockManager {
    /// Create new lock manager
    pub fn new(lock_timeout: Duration) -> Self {
        Self {
            locks: Arc::new(Mutex::new(HashMap::new())),
            lock_timeout,
        }
    }

    /// Try to acquire lock for a session
    pub async fn try_lock(&self, session_id: &str, holder: &str) -> Result<SessionLockGuard> {
        let mut locks = self.locks.lock().await;

        // Check if already locked
        if let Some(lock_info) = locks.get(session_id) {
            // Check if lock is stale
            if lock_info.acquired_at.elapsed() > self.lock_timeout {
                eprintln!(
                    "Warning: Stale lock detected for session {} (held by {}), breaking lock",
                    session_id, lock_info.holder
                );
            } else {
                bail!("Session {} is locked by {}", session_id, lock_info.holder);
            }
        }

        // Acquire lock
        locks.insert(session_id.to_string(), LockInfo {
            acquired_at: Instant::now(),
            holder: holder.to_string(),
        });

        Ok(SessionLockGuard {
            session_id: session_id.to_string(),
            locks: self.locks.clone(),
        })
    }

    /// Check if session is locked
    pub async fn is_locked(&self, session_id: &str) -> bool {
        let locks = self.locks.lock().await;
        locks.contains_key(session_id)
    }

    /// Force release a lock (for cleanup)
    pub async fn force_unlock(&self, session_id: &str) {
        let mut locks = self.locks.lock().await;
        locks.remove(session_id);
    }

    /// Get all currently locked sessions
    pub async fn locked_sessions(&self) -> Vec<String> {
        let locks = self.locks.lock().await;
        locks.keys().cloned().collect()
    }

    /// Clean up stale locks
    pub async fn cleanup_stale_locks(&self) -> usize {
        let mut locks = self.locks.lock().await;
        let stale: Vec<_> = locks
            .iter()
            .filter(|(_, info)| info.acquired_at.elapsed() > self.lock_timeout)
            .map(|(id, _)| id.clone())
            .collect();

        let count = stale.len();
        for id in stale {
            locks.remove(&id);
        }
        count
    }
}

impl Default for SessionLockManager {
    fn default() -> Self {
        Self::new(Duration::from_secs(30))
    }
}

#[cfg(test)]
mod tests {
    use tokio::time::sleep;

    use super::*;

    #[tokio::test]
    async fn test_acquire_lock() {
        let manager = SessionLockManager::default();

        let guard = manager.try_lock("session-1", "user-1").await;
        assert!(guard.is_ok());
    }

    #[tokio::test]
    async fn test_double_lock_fails() {
        let manager = SessionLockManager::default();

        let _guard1 = manager.try_lock("session-1", "user-1").await.unwrap();
        let result = manager.try_lock("session-1", "user-2").await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("locked"));
    }

    #[tokio::test]
    async fn test_lock_released_on_drop() {
        let manager = SessionLockManager::default();

        {
            let _guard = manager.try_lock("session-1", "user-1").await.unwrap();
            assert!(manager.is_locked("session-1").await);
        }

        // Give async drop time to complete
        sleep(Duration::from_millis(10)).await;

        // Should be able to lock again
        let result = manager.try_lock("session-1", "user-2").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_is_locked() {
        let manager = SessionLockManager::default();

        assert!(!manager.is_locked("session-1").await);

        let _guard = manager.try_lock("session-1", "user-1").await.unwrap();
        assert!(manager.is_locked("session-1").await);
    }

    #[tokio::test]
    async fn test_force_unlock() {
        let manager = SessionLockManager::default();

        let _guard = manager.try_lock("session-1", "user-1").await.unwrap();
        assert!(manager.is_locked("session-1").await);

        manager.force_unlock("session-1").await;
        assert!(!manager.is_locked("session-1").await);
    }

    #[tokio::test]
    async fn test_locked_sessions() {
        let manager = SessionLockManager::default();

        let _guard1 = manager.try_lock("session-1", "user-1").await.unwrap();
        let _guard2 = manager.try_lock("session-2", "user-2").await.unwrap();

        let locked = manager.locked_sessions().await;
        assert_eq!(locked.len(), 2);
        assert!(locked.contains(&"session-1".to_string()));
        assert!(locked.contains(&"session-2".to_string()));
    }

    #[tokio::test]
    async fn test_stale_lock_cleanup() {
        let manager = SessionLockManager::new(Duration::from_millis(50));

        let _guard = manager.try_lock("session-1", "user-1").await.unwrap();

        // Wait for lock to become stale
        sleep(Duration::from_millis(100)).await;

        let count = manager.cleanup_stale_locks().await;
        assert_eq!(count, 1);
        assert!(!manager.is_locked("session-1").await);
    }

    #[tokio::test]
    async fn test_stale_lock_broken_on_new_acquire() {
        let manager = SessionLockManager::new(Duration::from_millis(50));

        let _guard1 = manager.try_lock("session-1", "user-1").await.unwrap();

        // Wait for lock to become stale
        sleep(Duration::from_millis(100)).await;

        // Should be able to acquire despite existing lock
        let result = manager.try_lock("session-1", "user-2").await;
        assert!(result.is_ok());
    }
}
