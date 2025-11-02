use hnswlib::{
    HnswDistanceFunction,
    HnswIndex,
    HnswIndexInitConfig,
};

use crate::{
    CortexError,
    IdMapper,
    Result,
};

pub struct HnswWrapper {
    index: HnswIndex,
    id_mapper: IdMapper,
    dimensionality: usize,
}

impl HnswWrapper {
    pub fn new(dimensionality: usize, max_elements: usize) -> Result<Self> {
        let config = HnswIndexInitConfig {
            distance_function: HnswDistanceFunction::Cosine,
            dimensionality: dimensionality as i32,
            max_elements,
            m: 16,
            ef_construction: 200,
            ef_search: 100,
            random_seed: 0,
            persist_path: None,
        };

        let index = HnswIndex::init(config)?;

        Ok(Self {
            index,
            id_mapper: IdMapper::new(),
            dimensionality,
        })
    }

    pub fn add(&mut self, string_id: String, vector: &[f32]) -> Result<()> {
        if vector.len() != self.dimensionality {
            return Err(CortexError::InvalidInput(format!(
                "Expected {} dimensions, got {}",
                self.dimensionality,
                vector.len()
            )));
        }

        let numeric_id = self.id_mapper.get_or_create(string_id);
        self.index.add(numeric_id, vector)?;
        Ok(())
    }

    pub fn get(&self, string_id: &str) -> Result<Option<Vec<f32>>> {
        if let Some(numeric_id) = self.id_mapper.get_numeric(string_id) {
            Ok(self.index.get(numeric_id)?)
        } else {
            Ok(None)
        }
    }

    pub fn delete(&mut self, string_id: &str) -> Result<bool> {
        if let Some(numeric_id) = self.id_mapper.remove(string_id) {
            self.index.delete(numeric_id)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn search(&self, query: &[f32], k: usize, allowed_ids: Option<&[String]>) -> Result<Vec<(String, f32)>> {
        if query.len() != self.dimensionality {
            return Err(CortexError::InvalidInput(format!(
                "Expected {} dimensions, got {}",
                self.dimensionality,
                query.len()
            )));
        }

        let numeric_allowed: Vec<usize> = if let Some(ids) = allowed_ids {
            ids.iter().filter_map(|s| self.id_mapper.get_numeric(s)).collect()
        } else {
            vec![]
        };

        let (ids, distances) = self.index.query(query, k, &numeric_allowed, &[])?;

        let results: Vec<(String, f32)> = ids
            .iter()
            .zip(distances.iter())
            .filter_map(|(&id, &dist)| self.id_mapper.get_string(id).map(|s| (s.clone(), dist)))
            .collect();

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hnsw_wrapper_add_and_get() {
        let mut wrapper = HnswWrapper::new(3, 100).unwrap();

        let vec = vec![1.0, 2.0, 3.0];
        wrapper.add("doc1".to_string(), &vec).unwrap();

        let retrieved = wrapper.get("doc1").unwrap().unwrap();
        assert_eq!(retrieved, vec);
    }

    #[test]
    fn test_hnsw_wrapper_search() {
        let mut wrapper = HnswWrapper::new(3, 100).unwrap();

        wrapper.add("doc1".to_string(), &[1.0, 2.0, 3.0]).unwrap();
        wrapper.add("doc2".to_string(), &[1.1, 2.1, 3.1]).unwrap();

        let results = wrapper.search(&[1.0, 2.0, 3.0], 2, None).unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, "doc1");
    }

    #[test]
    fn test_hnsw_wrapper_delete() {
        let mut wrapper = HnswWrapper::new(3, 100).unwrap();

        wrapper.add("doc1".to_string(), &[1.0, 2.0, 3.0]).unwrap();
        assert!(wrapper.delete("doc1").unwrap());
        assert_eq!(wrapper.get("doc1").unwrap(), None);
    }

    #[test]
    fn test_hnsw_wrapper_filtered_search() {
        let mut wrapper = HnswWrapper::new(3, 100).unwrap();

        wrapper.add("doc1".to_string(), &[1.0, 2.0, 3.0]).unwrap();
        wrapper.add("doc2".to_string(), &[1.1, 2.1, 3.1]).unwrap();
        wrapper.add("doc3".to_string(), &[5.0, 6.0, 7.0]).unwrap();

        let allowed = vec!["doc1".to_string(), "doc3".to_string()];
        let results = wrapper.search(&[1.0, 2.0, 3.0], 3, Some(&allowed)).unwrap();

        assert!(!results.iter().any(|(id, _)| id == "doc2"));
    }

    #[test]
    fn test_hnsw_wrapper_dimension_validation() {
        let mut wrapper = HnswWrapper::new(3, 100).unwrap();

        let result = wrapper.add("doc1".to_string(), &[1.0, 2.0]);
        assert!(result.is_err());
    }

    #[test]
    fn test_hnsw_wrapper_search_dimension_validation() {
        let wrapper = HnswWrapper::new(3, 100).unwrap();

        let result = wrapper.search(&[1.0, 2.0], 5, None);
        assert!(result.is_err());
    }
}
