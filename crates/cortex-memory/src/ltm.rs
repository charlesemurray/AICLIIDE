use std::collections::HashMap;
use std::path::Path;

use rusqlite::{
    Connection,
    OptionalExtension,
    params,
};
use serde_json::Value;

use crate::{
    CortexError,
    HnswWrapper,
    MemoryNote,
    Result,
};

pub struct LongTermMemory {
    conn: Connection,
    hnsw: HnswWrapper,
}

impl LongTermMemory {
    pub fn new<P: AsRef<Path>>(db_path: P, dimensionality: usize) -> Result<Self> {
        let conn = Connection::open(db_path).map_err(|e| CortexError::StorageError(e.to_string()))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS memories (
                id TEXT PRIMARY KEY,
                content TEXT NOT NULL,
                metadata TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )
        .map_err(|e| CortexError::StorageError(e.to_string()))?;

        let hnsw = HnswWrapper::new(dimensionality, 10000)?;

        Ok(Self { conn, hnsw })
    }

    pub fn add(&mut self, note: MemoryNote, embedding: Vec<f32>) -> Result<()> {
        let metadata_json =
            serde_json::to_string(&note.metadata).map_err(|e| CortexError::StorageError(e.to_string()))?;

        self.conn
            .execute(
                "INSERT OR REPLACE INTO memories (id, content, metadata, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    &note.id,
                    &note.content,
                    &metadata_json,
                    note.created_at.to_rfc3339(),
                    note.updated_at.to_rfc3339(),
                ],
            )
            .map_err(|e| CortexError::StorageError(e.to_string()))?;

        self.hnsw.add(note.id, &embedding)?;
        Ok(())
    }

    pub fn get(&self, id: &str) -> Result<Option<MemoryNote>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, content, metadata, created_at, updated_at FROM memories WHERE id = ?1")
            .map_err(|e| CortexError::StorageError(e.to_string()))?;

        let result = stmt
            .query_row(params![id], |row| {
                let metadata_str: String = row.get(2)?;
                let metadata: HashMap<String, Value> = serde_json::from_str(&metadata_str)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

                let created_at: String = row.get(3)?;
                let updated_at: String = row.get(4)?;

                Ok(MemoryNote {
                    id: row.get(0)?,
                    content: row.get(1)?,
                    metadata,
                    created_at: created_at
                        .parse()
                        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
                    updated_at: updated_at
                        .parse()
                        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
                })
            })
            .optional()
            .map_err(|e| CortexError::StorageError(e.to_string()))?;

        Ok(result)
    }

    pub fn delete(&mut self, id: &str) -> Result<bool> {
        let deleted = self
            .conn
            .execute("DELETE FROM memories WHERE id = ?1", params![id])
            .map_err(|e| CortexError::StorageError(e.to_string()))?
            > 0;

        if deleted {
            self.hnsw.delete(id)?;
        }

        Ok(deleted)
    }

    pub fn search(
        &self,
        query_embedding: &[f32],
        k: usize,
        metadata_filter: Option<&HashMap<String, Value>>,
    ) -> Result<Vec<MemoryNote>> {
        let candidates = if let Some(filter) = metadata_filter {
            let filter_ids = self.filter_by_metadata(filter)?;
            self.hnsw.search(query_embedding, k * 2, Some(&filter_ids))?
        } else {
            self.hnsw.search(query_embedding, k, None)?
        };

        let mut results = Vec::new();
        for (id, _score) in candidates.iter().take(k) {
            if let Some(note) = self.get(id)? {
                results.push(note);
            }
        }

        Ok(results)
    }

    fn filter_by_metadata(&self, filter: &HashMap<String, Value>) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, metadata FROM memories")
            .map_err(|e| CortexError::StorageError(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| {
                let id: String = row.get(0)?;
                let metadata_str: String = row.get(1)?;
                Ok((id, metadata_str))
            })
            .map_err(|e| CortexError::StorageError(e.to_string()))?;

        let mut matching_ids = Vec::new();
        for row in rows {
            let (id, metadata_str) = row.map_err(|e| CortexError::StorageError(e.to_string()))?;
            let metadata: HashMap<String, Value> =
                serde_json::from_str(&metadata_str).map_err(|e| CortexError::StorageError(e.to_string()))?;

            if matches_filter(&metadata, filter) {
                matching_ids.push(id);
            }
        }

        Ok(matching_ids)
    }
}

fn matches_filter(metadata: &HashMap<String, Value>, filter: &HashMap<String, Value>) -> bool {
    for (key, filter_value) in filter {
        if let Some(meta_value) = metadata.get(key) {
            if meta_value != filter_value {
                return false;
            }
        } else {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use tempfile::NamedTempFile;

    use super::*;

    fn create_test_note(id: &str, content: &str) -> MemoryNote {
        MemoryNote::new(id.to_string(), content.to_string(), HashMap::new())
    }

    #[test]
    fn test_ltm_add_and_get() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut ltm = LongTermMemory::new(temp_file.path(), 3).unwrap();

        let note = create_test_note("1", "test content");
        let embedding = vec![1.0, 2.0, 3.0];

        ltm.add(note.clone(), embedding).unwrap();

        let retrieved = ltm.get("1").unwrap().unwrap();
        assert_eq!(retrieved.id, "1");
        assert_eq!(retrieved.content, "test content");
    }

    #[test]
    fn test_ltm_delete() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut ltm = LongTermMemory::new(temp_file.path(), 3).unwrap();

        let note = create_test_note("1", "test");
        ltm.add(note, vec![1.0, 0.0, 0.0]).unwrap();

        assert!(ltm.delete("1").unwrap());
        assert!(ltm.get("1").unwrap().is_none());
        assert!(!ltm.delete("1").unwrap());
    }

    #[test]
    fn test_ltm_search() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut ltm = LongTermMemory::new(temp_file.path(), 3).unwrap();

        ltm.add(create_test_note("1", "rust"), vec![1.0, 0.0, 0.0]).unwrap();
        ltm.add(create_test_note("2", "python"), vec![0.9, 0.1, 0.0]).unwrap();

        let results = ltm.search(&[1.0, 0.0, 0.0], 2, None).unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, "1");
    }

    #[test]
    fn test_ltm_metadata_filter() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut ltm = LongTermMemory::new(temp_file.path(), 3).unwrap();

        let mut metadata1 = HashMap::new();
        metadata1.insert("tag".to_string(), Value::String("rust".to_string()));
        let note1 = MemoryNote::new("1".to_string(), "rust content".to_string(), metadata1);

        let mut metadata2 = HashMap::new();
        metadata2.insert("tag".to_string(), Value::String("python".to_string()));
        let note2 = MemoryNote::new("2".to_string(), "python content".to_string(), metadata2);

        ltm.add(note1, vec![1.0, 0.0, 0.0]).unwrap();
        ltm.add(note2, vec![0.9, 0.1, 0.0]).unwrap();

        let mut filter = HashMap::new();
        filter.insert("tag".to_string(), Value::String("rust".to_string()));

        let results = ltm.search(&[1.0, 0.0, 0.0], 2, Some(&filter)).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "1");
    }
}
