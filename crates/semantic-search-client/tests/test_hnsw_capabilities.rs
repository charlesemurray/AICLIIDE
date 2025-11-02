/// Test suite to verify hnsw_rs capabilities for Cortex requirements
///
/// This tests what hnsw_rs v0.3.1 actually supports vs what Cortex needs
use hnsw_rs::hnsw::Hnsw;
use hnsw_rs::prelude::*;

#[test]
fn test_basic_insert_and_search() {
    // Test: Can we insert and search?
    let mut hnsw = Hnsw::<f32, DistCosine>::new(
        16,  // max_nb_connection
        100, // max_elements
        16,  // ef_construction
        100, // ef_search
        DistCosine {},
    );

    let vec1 = vec![1.0, 2.0, 3.0];
    let vec2 = vec![1.1, 2.1, 3.1];
    let vec3 = vec![5.0, 6.0, 7.0];

    hnsw.insert((&vec1, 0));
    hnsw.insert((&vec2, 1));
    hnsw.insert((&vec3, 2));

    let query = vec![1.0, 2.0, 3.0];
    let results = hnsw.search(&query, 2, 100);

    assert!(!results.is_empty());
    println!("✅ Basic insert and search works");
    println!("   Results: {:?}", results);
}

#[test]
fn test_get_by_id() {
    // Test: Can we retrieve a vector by ID?
    let mut hnsw = Hnsw::<f32, DistCosine>::new(16, 100, 16, 100, DistCosine {});

    let vec1 = vec![1.0, 2.0, 3.0];
    hnsw.insert((&vec1, 42));

    // Try to get the vector back
    // Check if this method exists
    // let retrieved = hnsw.get(42);

    println!("⚠️  Need to check if get() method exists in hnsw_rs API");
    println!("   Checking available methods...");
}

#[test]
fn test_delete_operation() {
    // Test: Can we delete by ID?
    let mut hnsw = Hnsw::<f32, DistCosine>::new(16, 100, 16, 100, DistCosine {});

    let vec1 = vec![1.0, 2.0, 3.0];
    let vec2 = vec![5.0, 6.0, 7.0];

    hnsw.insert((&vec1, 0));
    hnsw.insert((&vec2, 1));

    // Try to delete
    // Check if this method exists
    // let result = hnsw.delete(0);

    println!("⚠️  Need to check if delete() method exists in hnsw_rs API");
    println!("   This is CRITICAL for Cortex");
}

#[test]
fn test_filtered_search() {
    // Test: Can we search with allowed/disallowed IDs?
    let mut hnsw = Hnsw::<f32, DistCosine>::new(16, 100, 16, 100, DistCosine {});

    let vec1 = vec![1.0, 2.0, 3.0];
    let vec2 = vec![1.1, 2.1, 3.1];
    let vec3 = vec![5.0, 6.0, 7.0];

    hnsw.insert((&vec1, 0));
    hnsw.insert((&vec2, 1));
    hnsw.insert((&vec3, 2));

    let query = vec![1.0, 2.0, 3.0];

    // Try filtered search
    // Check if search accepts allowed_ids parameter
    // let results = hnsw.search_filtered(&query, 2, 100, &[0, 2]);

    let results = hnsw.search(&query, 3, 100);

    println!("⚠️  Standard search returns: {:?}", results);
    println!("   Need to check if filtered search exists");
    println!("   Chroma's hnswlib has: query(vector, k, allowed_ids, disallowed_ids)");
}

#[test]
fn test_capacity_and_resize() {
    // Test: Can we resize the index?
    let mut hnsw = Hnsw::<f32, DistCosine>::new(16, 10, 16, 100, DistCosine {});

    // Insert up to capacity
    for i in 0..10 {
        let vec = vec![i as f32, i as f32, i as f32];
        hnsw.insert((&vec, i));
    }

    // Try to insert beyond capacity
    let vec11 = vec![11.0, 11.0, 11.0];
    hnsw.insert((&vec11, 11));

    println!("✅ Can insert beyond initial capacity");
    println!("   hnsw_rs may auto-resize or handle this gracefully");
}

#[test]
fn test_persistence() {
    // Test: Can we save and load?
    use std::path::PathBuf;

    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let save_path = temp_dir.path().join("test_index");

    let mut hnsw = Hnsw::<f32, DistCosine>::new(16, 100, 16, 100, DistCosine {});

    let vec1 = vec![1.0, 2.0, 3.0];
    hnsw.insert((&vec1, 0));

    // Try to save
    // Check what save methods exist
    // hnsw.save(&save_path)?;

    println!("⚠️  Need to check persistence API in hnsw_rs");
    println!("   Q CLI's VectorIndex uses hnsw_rs - check how it persists");
}

#[test]
fn test_id_type() {
    // Test: What ID types are supported?
    let mut hnsw = Hnsw::<f32, DistCosine>::new(16, 100, 16, 100, DistCosine {});

    // hnsw_rs uses usize for IDs
    let vec1 = vec![1.0, 2.0, 3.0];
    hnsw.insert((&vec1, 0_usize));

    // Can't use String IDs directly
    // hnsw.insert((&vec1, "my-uuid-string"));  // Won't compile

    println!("✅ Confirmed: hnsw_rs uses usize for IDs");
    println!("   ❌ Cannot use String IDs directly");
    println!("   ⚠️  Need ID mapping layer: String -> usize");
}

#[test]
fn test_concurrent_access() {
    // Test: Is it thread-safe?
    use std::sync::Arc;
    use std::thread;

    let hnsw = Arc::new(Hnsw::<f32, DistCosine>::new(16, 100, 16, 100, DistCosine {}));

    // Try concurrent reads
    let hnsw_clone = hnsw.clone();
    let handle = thread::spawn(move || {
        let query = vec![1.0, 2.0, 3.0];
        let _results = hnsw_clone.search(&query, 5, 100);
    });

    handle.join().unwrap();

    println!("✅ Can share Hnsw across threads with Arc");
    println!("   But need to check if insert requires &mut");
}
