use std::path::Path;

use crate::{
    LongTermMemory,
    MemoryNote,
    Result,
    ShortTermMemory,
};

pub struct MemoryManager {
    stm: ShortTermMemory,
    ltm: LongTermMemory,
    stm_capacity: usize,
}

impl MemoryManager {
    pub fn new<P: AsRef<Path>>(db_path: P, dimensionality: usize, stm_capacity: usize) -> Result<Self> {
        let stm = ShortTermMemory::new(stm_capacity);
        let ltm = LongTermMemory::new(db_path, dimensionality)?;

        Ok(Self { stm, ltm, stm_capacity })
    }

    pub fn add(&mut self, note: MemoryNote, embedding: Vec<f32>) -> Result<()> {
        self.stm.add(note, embedding)?;
        Ok(())
    }

    pub fn get(&mut self, id: &str) -> Result<Option<MemoryNote>> {
        if let Some(note) = self.stm.get(id) {
            return Ok(Some(note.clone()));
        }

        self.ltm.get(id)
    }

    pub fn delete(&mut self, id: &str) -> Result<bool> {
        let stm_deleted = self.stm.delete(id);
        let ltm_deleted = self.ltm.delete(id)?;
        Ok(stm_deleted || ltm_deleted)
    }

    pub fn search(&self, query_embedding: &[f32], k: usize) -> Vec<(String, f32)> {
        self.stm.search(query_embedding, k)
    }

    pub fn promote_to_ltm(&mut self, id: &str, embedding: Vec<f32>) -> Result<bool> {
        if let Some(note) = self.stm.get(id) {
            let note_clone = note.clone();
            self.ltm.add(note_clone, embedding)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn stm_len(&self) -> usize {
        self.stm.len()
    }

    pub fn stm_capacity(&self) -> usize {
        self.stm_capacity
    }

    pub fn get_ltm(&self) -> &LongTermMemory {
        &self.ltm
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use tempfile::NamedTempFile;

    use super::*;

    fn create_test_note(id: &str, content: &str) -> MemoryNote {
        MemoryNote::new(id.to_string(), content.to_string(), HashMap::new())
    }

    #[test]
    fn test_memory_manager_add_and_get() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut manager = MemoryManager::new(temp_file.path(), 3, 10).unwrap();

        let note = create_test_note("1", "test content");
        let embedding = vec![1.0, 2.0, 3.0];

        manager.add(note, embedding).unwrap();

        let retrieved = manager.get("1").unwrap().unwrap();
        assert_eq!(retrieved.id, "1");
        assert_eq!(retrieved.content, "test content");
    }

    #[test]
    fn test_memory_manager_search() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut manager = MemoryManager::new(temp_file.path(), 3, 10).unwrap();

        manager.add(create_test_note("1", "rust"), vec![1.0, 0.0, 0.0]).unwrap();
        manager
            .add(create_test_note("2", "python"), vec![0.9, 0.1, 0.0])
            .unwrap();

        let results = manager.search(&[1.0, 0.0, 0.0], 2);

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, "1");
    }

    #[test]
    fn test_memory_manager_promote_to_ltm() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut manager = MemoryManager::new(temp_file.path(), 3, 10).unwrap();

        let note = create_test_note("1", "test");
        let embedding = vec![1.0, 0.0, 0.0];

        manager.add(note, embedding.clone()).unwrap();
        assert!(manager.promote_to_ltm("1", embedding).unwrap());

        let from_ltm = manager.ltm.get("1").unwrap().unwrap();
        assert_eq!(from_ltm.id, "1");
    }

    #[test]
    fn test_memory_manager_delete() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut manager = MemoryManager::new(temp_file.path(), 3, 10).unwrap();

        manager.add(create_test_note("1", "test"), vec![1.0, 0.0, 0.0]).unwrap();

        assert!(manager.delete("1").unwrap());
        assert!(manager.get("1").unwrap().is_none());
    }

    #[test]
    fn test_memory_manager_get_from_ltm() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut manager = MemoryManager::new(temp_file.path(), 3, 10).unwrap();

        let note = create_test_note("1", "test");
        let embedding = vec![1.0, 0.0, 0.0];

        manager.add(note, embedding.clone()).unwrap();
        manager.promote_to_ltm("1", embedding).unwrap();
        manager.stm.delete("1");

        let retrieved = manager.get("1").unwrap().unwrap();
        assert_eq!(retrieved.id, "1");
    }
}
