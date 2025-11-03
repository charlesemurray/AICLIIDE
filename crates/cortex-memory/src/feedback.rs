//! User feedback for memory quality

use std::path::Path;

use rusqlite::{
    Connection,
    Result,
};

/// Feedback on memory usefulness
#[derive(Debug, Clone)]
pub struct MemoryFeedback {
    pub memory_id: String,
    pub helpful: bool,
    pub timestamp: i64,
}

/// Manages user feedback on memories
pub struct FeedbackManager {
    conn: Connection,
}

impl FeedbackManager {
    /// Create new feedback manager
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let conn = Connection::open(db_path)?;

        // Create feedback table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS memory_feedback (
                memory_id TEXT PRIMARY KEY,
                helpful INTEGER NOT NULL,
                timestamp INTEGER NOT NULL
            )",
            [],
        )?;

        Ok(Self { conn })
    }

    /// Record feedback for a memory
    pub fn record_feedback(&self, memory_id: &str, helpful: bool) -> Result<()> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        self.conn.execute(
            "INSERT OR REPLACE INTO memory_feedback (memory_id, helpful, timestamp) VALUES (?1, ?2, ?3)",
            [memory_id, &(helpful as i64).to_string(), &timestamp.to_string()],
        )?;

        Ok(())
    }

    /// Get feedback for a memory
    pub fn get_feedback(&self, memory_id: &str) -> Result<Option<MemoryFeedback>> {
        let mut stmt = self.conn.prepare(
            "SELECT memory_id, helpful, timestamp FROM memory_feedback WHERE memory_id = ?1 ORDER BY timestamp DESC LIMIT 1"
        )?;

        let mut rows = stmt.query([memory_id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(MemoryFeedback {
                memory_id: row.get(0)?,
                helpful: row.get::<_, i64>(1)? != 0,
                timestamp: row.get(2)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Get feedback statistics
    pub fn get_stats(&self) -> Result<(usize, usize)> {
        let helpful: usize =
            self.conn
                .query_row("SELECT COUNT(*) FROM memory_feedback WHERE helpful = 1", [], |row| {
                    row.get(0)
                })?;

        let not_helpful: usize =
            self.conn
                .query_row("SELECT COUNT(*) FROM memory_feedback WHERE helpful = 0", [], |row| {
                    row.get(0)
                })?;

        Ok((helpful, not_helpful))
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn test_feedback_storage() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("feedback.db");
        let manager = FeedbackManager::new(&db_path).unwrap();

        manager.record_feedback("mem1", true).unwrap();
        manager.record_feedback("mem2", false).unwrap();

        let feedback1 = manager.get_feedback("mem1").unwrap().unwrap();
        assert!(feedback1.helpful);

        let feedback2 = manager.get_feedback("mem2").unwrap().unwrap();
        assert!(!feedback2.helpful);

        let (helpful, not_helpful) = manager.get_stats().unwrap();
        assert_eq!(helpful, 1);
        assert_eq!(not_helpful, 1);
    }

    #[test]
    fn test_duplicate_feedback_updates() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("feedback.db");
        let manager = FeedbackManager::new(&db_path).unwrap();

        // First feedback
        manager.record_feedback("mem1", true).unwrap();
        let feedback1 = manager.get_feedback("mem1").unwrap().unwrap();
        assert!(feedback1.helpful);

        // Second feedback for same ID should update
        manager.record_feedback("mem1", false).unwrap();
        let feedback2 = manager.get_feedback("mem1").unwrap().unwrap();
        assert!(!feedback2.helpful);

        // Should only have one entry
        let (helpful, not_helpful) = manager.get_stats().unwrap();
        assert_eq!(helpful, 0);
        assert_eq!(not_helpful, 1);
    }
}
