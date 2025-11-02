//! ID mapping layer for converting between String and usize IDs

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Maps between String IDs (used by Cortex) and usize IDs (used by HNSW)
#[derive(Debug)]
pub struct IdMapper {
    string_to_usize: HashMap<String, usize>,
    usize_to_string: HashMap<usize, String>,
    next_id: AtomicUsize,
}

impl IdMapper {
    /// Create a new ID mapper
    pub fn new() -> Self {
        Self {
            string_to_usize: HashMap::new(),
            usize_to_string: HashMap::new(),
            next_id: AtomicUsize::new(0),
        }
    }

    /// Get or create a numeric ID for a string ID
    pub fn get_or_create(&mut self, string_id: String) -> usize {
        if let Some(&numeric_id) = self.string_to_usize.get(&string_id) {
            return numeric_id;
        }

        let numeric_id = self.next_id.fetch_add(1, Ordering::SeqCst);
        self.string_to_usize.insert(string_id.clone(), numeric_id);
        self.usize_to_string.insert(numeric_id, string_id);
        numeric_id
    }

    /// Get the numeric ID for a string ID, if it exists
    pub fn get_numeric(&self, string_id: &str) -> Option<usize> {
        self.string_to_usize.get(string_id).copied()
    }

    /// Get the string ID for a numeric ID, if it exists
    pub fn get_string(&self, numeric_id: usize) -> Option<&String> {
        self.usize_to_string.get(&numeric_id)
    }

    /// Remove a mapping and return the numeric ID if it existed
    pub fn remove(&mut self, string_id: &str) -> Option<usize> {
        if let Some(numeric_id) = self.string_to_usize.remove(string_id) {
            self.usize_to_string.remove(&numeric_id);
            Some(numeric_id)
        } else {
            None
        }
    }

    /// Get the number of mappings
    pub fn len(&self) -> usize {
        self.string_to_usize.len()
    }

    /// Check if the mapper is empty
    pub fn is_empty(&self) -> bool {
        self.string_to_usize.is_empty()
    }
}

impl Default for IdMapper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_mapper_create() {
        let mut mapper = IdMapper::new();

        let id1 = mapper.get_or_create("uuid-1".to_string());
        let id2 = mapper.get_or_create("uuid-2".to_string());

        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
    }

    #[test]
    fn test_id_mapper_idempotent() {
        let mut mapper = IdMapper::new();

        let id1 = mapper.get_or_create("uuid-1".to_string());
        let id2 = mapper.get_or_create("uuid-1".to_string());

        assert_eq!(id1, id2);
        assert_eq!(mapper.len(), 1);
    }

    #[test]
    fn test_id_mapper_bidirectional() {
        let mut mapper = IdMapper::new();

        let numeric = mapper.get_or_create("uuid-1".to_string());

        assert_eq!(mapper.get_numeric("uuid-1"), Some(numeric));
        assert_eq!(mapper.get_string(numeric), Some(&"uuid-1".to_string()));
    }

    #[test]
    fn test_id_mapper_remove() {
        let mut mapper = IdMapper::new();

        let numeric = mapper.get_or_create("uuid-1".to_string());
        assert_eq!(mapper.len(), 1);

        let removed = mapper.remove("uuid-1");
        assert_eq!(removed, Some(numeric));
        assert_eq!(mapper.len(), 0);
        assert_eq!(mapper.get_numeric("uuid-1"), None);
        assert_eq!(mapper.get_string(numeric), None);
    }

    #[test]
    fn test_id_mapper_multiple_ids() {
        let mut mapper = IdMapper::new();

        let ids: Vec<String> = (0..10).map(|i| format!("uuid-{}", i)).collect();
        let numeric_ids: Vec<usize> = ids
            .iter()
            .map(|id| mapper.get_or_create(id.clone()))
            .collect();

        assert_eq!(mapper.len(), 10);

        for (i, id) in ids.iter().enumerate() {
            assert_eq!(mapper.get_numeric(id), Some(numeric_ids[i]));
            assert_eq!(mapper.get_string(numeric_ids[i]), Some(id));
        }
    }

    #[test]
    fn test_id_mapper_empty() {
        let mapper = IdMapper::new();
        assert!(mapper.is_empty());
        assert_eq!(mapper.len(), 0);
    }

    #[test]
    fn test_id_mapper_get_nonexistent() {
        let mapper = IdMapper::new();

        assert_eq!(mapper.get_numeric("nonexistent"), None);
        assert_eq!(mapper.get_string(999), None);
    }
}
