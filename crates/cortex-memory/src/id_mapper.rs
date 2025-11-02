use std::collections::HashMap;
use std::sync::atomic::{
    AtomicUsize,
    Ordering,
};

#[derive(Debug, Default)]
pub struct IdMapper {
    string_to_usize: HashMap<String, usize>,
    usize_to_string: HashMap<usize, String>,
    next_id: AtomicUsize,
}

impl IdMapper {
    pub fn new() -> Self {
        Self::default()
    }
    }

    pub fn get_or_create(&mut self, string_id: String) -> usize {
        if let Some(&numeric_id) = self.string_to_usize.get(&string_id) {
            return numeric_id;
        }

        let numeric_id = self.next_id.fetch_add(1, Ordering::SeqCst);
        self.string_to_usize.insert(string_id.clone(), numeric_id);
        self.usize_to_string.insert(numeric_id, string_id);
        numeric_id
    }

    pub fn get_numeric(&self, string_id: &str) -> Option<usize> {
        self.string_to_usize.get(string_id).copied()
    }

    pub fn get_string(&self, numeric_id: usize) -> Option<&String> {
        self.usize_to_string.get(&numeric_id)
    }

    pub fn remove(&mut self, string_id: &str) -> Option<usize> {
        if let Some(numeric_id) = self.string_to_usize.remove(string_id) {
            self.usize_to_string.remove(&numeric_id);
            Some(numeric_id)
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.string_to_usize.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.string_to_usize.is_empty()
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
    }
}
