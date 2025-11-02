//! Cortex Memory System - Advanced memory for AI agents

pub mod error;

pub use error::{CortexError, Result};

#[cfg(test)]
mod tests {
    use hnswlib::{HnswDistanceFunction, HnswIndex, HnswIndexInitConfig};

    #[test]
    fn test_hnswlib_basic() {
        println!("\n🧪 Testing hnswlib basic functionality...");

        let config = HnswIndexInitConfig {
            distance_function: HnswDistanceFunction::Cosine,
            dimensionality: 3,
            max_elements: 100,
            m: 16,
            ef_construction: 200,
            ef_search: 100,
            random_seed: 0,
            persist_path: None,
        };

        let index = HnswIndex::init(config).expect("Failed to create index");

        println!("✅ Index created successfully");

        let vec1 = vec![1.0, 2.0, 3.0];
        let vec2 = vec![1.1, 2.1, 3.1];
        let vec3 = vec![5.0, 6.0, 7.0];

        index.add(0, &vec1).expect("Failed to add vec1");
        index.add(1, &vec2).expect("Failed to add vec2");
        index.add(2, &vec3).expect("Failed to add vec3");

        println!("✅ Added 3 vectors");

        let query = vec![1.0, 2.0, 3.0];
        let (ids, distances) = index.query(&query, 2, &[], &[]).expect("Failed to query");

        println!("✅ Search results:");
        for (id, dist) in ids.iter().zip(distances.iter()) {
            println!("   ID: {}, Distance: {:.6}", id, dist);
        }

        assert_eq!(ids.len(), 2);
        assert_eq!(ids[0], 0);

        println!("✅ Basic search works!\n");
    }

    #[test]
    fn test_hnswlib_delete() {
        println!("\n🧪 Testing hnswlib delete functionality...");

        let config = HnswIndexInitConfig {
            distance_function: HnswDistanceFunction::Cosine,
            dimensionality: 3,
            max_elements: 100,
            m: 16,
            ef_construction: 200,
            ef_search: 100,
            random_seed: 0,
            persist_path: None,
        };

        let index = HnswIndex::init(config).expect("Failed to create index");

        index.add(0, &vec![1.0, 2.0, 3.0]).unwrap();
        index.add(1, &vec![1.1, 2.1, 3.1]).unwrap();

        println!("✅ Added 2 vectors");

        index.delete(0).expect("Failed to delete");

        println!("✅ Deleted vector 0");

        let (ids, _) = index.query(&vec![1.0, 2.0, 3.0], 2, &[], &[]).unwrap();

        println!("✅ Search after delete: {:?}", ids);
        assert!(!ids.contains(&0), "Deleted vector should not appear in results");

        println!("✅ Delete works!\n");
    }

    #[test]
    fn test_hnswlib_get() {
        println!("\n🧪 Testing hnswlib get functionality...");

        let config = HnswIndexInitConfig {
            distance_function: HnswDistanceFunction::Cosine,
            dimensionality: 3,
            max_elements: 100,
            m: 16,
            ef_construction: 200,
            ef_search: 100,
            random_seed: 0,
            persist_path: None,
        };

        let index = HnswIndex::init(config).expect("Failed to create index");

        let vec1 = vec![1.0, 2.0, 3.0];
        index.add(42, &vec1).unwrap();

        println!("✅ Added vector with ID 42");

        let retrieved = index.get(42).expect("Failed to get vector");

        println!("✅ Retrieved vector: {:?}", retrieved);
        assert_eq!(retrieved, Some(vec1));

        let missing = index.get(999);
        assert!(missing.is_err() || missing.unwrap().is_none());

        println!("✅ Get by ID works!\n");
    }

    #[test]
    fn test_hnswlib_filtered_search() {
        println!("\n🧪 Testing hnswlib filtered search...");

        let config = HnswIndexInitConfig {
            distance_function: HnswDistanceFunction::Cosine,
            dimensionality: 3,
            max_elements: 100,
            m: 16,
            ef_construction: 200,
            ef_search: 100,
            random_seed: 0,
            persist_path: None,
        };

        let index = HnswIndex::init(config).expect("Failed to create index");

        index.add(0, &vec![1.0, 2.0, 3.0]).unwrap();
        index.add(1, &vec![1.1, 2.1, 3.1]).unwrap();
        index.add(2, &vec![5.0, 6.0, 7.0]).unwrap();

        println!("✅ Added 3 vectors");

        let allowed_ids = vec![0, 2];
        let (ids, distances) = index.query(&vec![1.0, 2.0, 3.0], 3, &allowed_ids, &[]).unwrap();

        println!("✅ Filtered search results: {:?}", ids);
        println!("   Distances: {:?}", distances);

        assert!(!ids.contains(&1), "ID 1 should be filtered out");
        assert!(ids.contains(&0) || ids.contains(&2));

        println!("✅ Filtered search works!\n");
    }

    #[test]
    fn test_hnswlib_all_features() {
        println!("\n🧪 Testing all hnswlib features together...");

        let config = HnswIndexInitConfig {
            distance_function: HnswDistanceFunction::Cosine,
            dimensionality: 3,
            max_elements: 100,
            m: 16,
            ef_construction: 200,
            ef_search: 100,
            random_seed: 0,
            persist_path: None,
        };

        let index = HnswIndex::init(config).expect("Failed to create index");

        index.add(0, &vec![1.0, 2.0, 3.0]).unwrap();
        println!("✅ Add works");

        let vec = index.get(0).unwrap();
        assert!(vec.is_some());
        println!("✅ Get works");

        let (ids, _) = index.query(&vec![1.0, 2.0, 3.0], 1, &[], &[]).unwrap();
        assert_eq!(ids[0], 0);
        println!("✅ Search works");

        index.delete(0).unwrap();
        println!("✅ Delete works");

        println!("✅ All features confirmed working!\n");
        println!("🎉 hnswlib is ready for Cortex integration!");
    }
}
