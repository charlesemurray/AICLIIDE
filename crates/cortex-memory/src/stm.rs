use crate::{CortexError, MemoryNote, Result};
use std::collections::{HashMap, VecDeque};

pub struct ShortTermMemory {
    capacity: usize,
    memories: HashMap<String, MemoryNote>,
    embeddings: HashMap<String, Vec<f32>>,
    access_order: VecDeque<String>,
}

impl ShortTermMemory {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            memories: HashMap::new(),
            embeddings: HashMap::new(),
            access_order: VecDeque::new(),
        }
    }

    pub fn add(&mut self, note: MemoryNote, embedding: Vec<f32>) -> Result<()> {
        if self.memories.len() >= self.capacity && !self.memories.contains_key(&note.id) {
            if let Some(oldest_id) = self.access_order.pop_front() {
                self.memories.remove(&oldest_id);
                self.embeddings.remove(&oldest_id);
            }
        }

        let id = note.id.clone();
        self.memories.insert(id.clone(), note);
        self.embeddings.insert(id.clone(), embedding);
        self.access_order.push_back(id);
        Ok(())
    }

    pub fn get(&mut self, id: &str) -> Option<&MemoryNote> {
        if self.memories.contains_key(id) {
            self.access_order.retain(|x| x != id);
            self.access_order.push_back(id.to_string());
        }
        self.memories.get(id)
    }

    pub fn delete(&mut self, id: &str) -> bool {
        self.access_order.retain(|x| x != id);
        self.embeddings.remove(id);
        self.memories.remove(id).is_some()
    }

    pub fn search(&self, query_embedding: &[f32], k: usize) -> Vec<(String, f32)> {
        let mut results: Vec<(String, f32)> = self
            .embeddings
            .iter()
            .map(|(id, emb)| {
                let similarity = cosine_similarity(query_embedding, emb);
                (id.clone(), similarity)
            })
            .collect();

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        results.truncate(k);
        results
    }

    pub fn len(&self) -> usize {
        self.memories.len()
    }

    pub fn is_empty(&self) -> bool {
        self.memories.is_empty()
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    dot / (norm_a * norm_b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_note(id: &str, content: &str) -> MemoryNote {
        MemoryNote::new(id.to_string(), content.to_string(), HashMap::new())
    }

    #[test]
    fn test_stm_add_and_get() {
        let mut stm = ShortTermMemory::new(10);
        let note = create_test_note("1", "test content");
        let embedding = vec![1.0, 2.0, 3.0];

        stm.add(note.clone(), embedding).unwrap();

        let retrieved = stm.get("1").unwrap();
        assert_eq!(retrieved.id, "1");
        assert_eq!(retrieved.content, "test content");
    }

    #[test]
    fn test_stm_lru_eviction() {
        let mut stm = ShortTermMemory::new(2);

        stm.add(create_test_note("1", "first"), vec![1.0, 0.0, 0.0])
            .unwrap();
        stm.add(create_test_note("2", "second"), vec![0.0, 1.0, 0.0])
            .unwrap();
        stm.add(create_test_note("3", "third"), vec![0.0, 0.0, 1.0])
            .unwrap();

        assert_eq!(stm.len(), 2);
        assert!(stm.get("1").is_none());
        assert!(stm.get("2").is_some());
        assert!(stm.get("3").is_some());
    }

    #[test]
    fn test_stm_search() {
        let mut stm = ShortTermMemory::new(10);

        stm.add(create_test_note("1", "rust"), vec![1.0, 0.0, 0.0])
            .unwrap();
        stm.add(create_test_note("2", "python"), vec![0.9, 0.1, 0.0])
            .unwrap();
        stm.add(create_test_note("3", "java"), vec![0.0, 1.0, 0.0])
            .unwrap();

        let results = stm.search(&[1.0, 0.0, 0.0], 2);

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, "1");
        assert!(results[0].1 > results[1].1);
    }

    #[test]
    fn test_stm_delete() {
        let mut stm = ShortTermMemory::new(10);

        stm.add(create_test_note("1", "test"), vec![1.0, 0.0, 0.0])
            .unwrap();

        assert!(stm.delete("1"));
        assert!(stm.get("1").is_none());
        assert!(!stm.delete("1"));
    }

    #[test]
    fn test_stm_lru_access_updates() {
        let mut stm = ShortTermMemory::new(2);

        stm.add(create_test_note("1", "first"), vec![1.0, 0.0, 0.0])
            .unwrap();
        stm.add(create_test_note("2", "second"), vec![0.0, 1.0, 0.0])
            .unwrap();

        stm.get("1");

        stm.add(create_test_note("3", "third"), vec![0.0, 0.0, 1.0])
            .unwrap();

        assert!(stm.get("1").is_some());
        assert!(stm.get("2").is_none());
        assert!(stm.get("3").is_some());
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.001);

        let c = vec![1.0, 0.0, 0.0];
        let d = vec![0.0, 1.0, 0.0];
        assert!(cosine_similarity(&c, &d).abs() < 0.001);
    }
}
