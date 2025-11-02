use cortex_memory::{MemoryManager, MemoryNote, ShortTermMemory};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use tempfile::NamedTempFile;

#[derive(Debug, Deserialize)]
struct Operation {
    #[serde(rename = "type")]
    op_type: String,
    #[serde(default)]
    id: String,
    #[serde(default)]
    content: String,
    #[serde(default)]
    embedding: Vec<f32>,
    #[serde(default)]
    metadata: HashMap<String, Value>,
    #[serde(default)]
    query_embedding: Vec<f32>,
    #[serde(default)]
    k: usize,
    #[serde(default)]
    expected_order: Vec<String>,
    #[serde(default)]
    expected: Option<HashMap<String, Value>>,
    #[serde(default)]
    expected_ids: Vec<String>,
    #[serde(default)]
    filter: HashMap<String, Value>,
}

#[derive(Debug, Deserialize)]
struct TestFixture {
    description: String,
    #[serde(default)]
    capacity: usize,
    #[serde(default)]
    dimensionality: usize,
    #[serde(default)]
    stm_capacity: usize,
    operations: Vec<Operation>,
}

fn load_fixtures(filename: &str) -> HashMap<String, TestFixture> {
    let path = format!("tests/fixtures/{}", filename);
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("Failed to read fixture file: {}", path));
    serde_json::from_str(&content).expect("Failed to parse fixture JSON")
}

#[test]
fn test_stm_basic_operations_fixture() {
    let fixtures = load_fixtures("stm_fixtures.json");
    let fixture = &fixtures["stm_basic_operations"];

    let mut stm = ShortTermMemory::new(fixture.capacity);

    for op in &fixture.operations {
        match op.op_type.as_str() {
            "add" => {
                let note = MemoryNote::new(op.id.clone(), op.content.clone(), op.metadata.clone());
                stm.add(note, op.embedding.clone()).unwrap();
            }
            "search" => {
                let results = stm.search(&op.query_embedding, op.k);
                let result_ids: Vec<String> = results.iter().map(|(id, _)| id.clone()).collect();
                assert_eq!(result_ids, op.expected_order, "Search order mismatch");
            }
            _ => panic!("Unknown operation type: {}", op.op_type),
        }
    }
}

#[test]
fn test_stm_lru_eviction_fixture() {
    let fixtures = load_fixtures("stm_fixtures.json");
    let fixture = &fixtures["stm_lru_eviction"];

    let mut stm = ShortTermMemory::new(fixture.capacity);

    for op in &fixture.operations {
        match op.op_type.as_str() {
            "add" => {
                let note = MemoryNote::new(op.id.clone(), op.content.clone(), op.metadata.clone());
                stm.add(note, op.embedding.clone()).unwrap();
            }
            "get" => {
                let result = stm.get(&op.id);
                if let Some(expected) = &op.expected {
                    assert!(result.is_some(), "Expected to find id: {}", op.id);
                    let note = result.unwrap();
                    assert_eq!(note.id, expected.get("id").unwrap().as_str().unwrap());
                    assert_eq!(
                        note.content,
                        expected.get("content").unwrap().as_str().unwrap()
                    );
                } else {
                    assert!(result.is_none(), "Expected id {} to be evicted", op.id);
                }
            }
            _ => panic!("Unknown operation type: {}", op.op_type),
        }
    }
}

#[test]
fn test_stm_lru_access_order_fixture() {
    let fixtures = load_fixtures("stm_fixtures.json");
    let fixture = &fixtures["stm_lru_access_order"];

    let mut stm = ShortTermMemory::new(fixture.capacity);

    for op in &fixture.operations {
        match op.op_type.as_str() {
            "add" => {
                let note = MemoryNote::new(op.id.clone(), op.content.clone(), op.metadata.clone());
                stm.add(note, op.embedding.clone()).unwrap();
            }
            "get" => {
                let result = stm.get(&op.id);
                if let Some(expected) = &op.expected {
                    assert!(result.is_some(), "Expected to find id: {}", op.id);
                    let note = result.unwrap();
                    assert_eq!(note.id, expected.get("id").unwrap().as_str().unwrap());
                } else {
                    assert!(result.is_none(), "Expected id {} to not exist", op.id);
                }
            }
            _ => panic!("Unknown operation type: {}", op.op_type),
        }
    }
}

#[test]
fn test_ltm_basic_operations_fixture() {
    let fixtures = load_fixtures("ltm_fixtures.json");
    let fixture = &fixtures["ltm_basic_operations"];

    let temp_file = NamedTempFile::new().unwrap();
    let mut ltm =
        cortex_memory::LongTermMemory::new(temp_file.path(), fixture.dimensionality).unwrap();

    for op in &fixture.operations {
        match op.op_type.as_str() {
            "add" => {
                let note = MemoryNote::new(op.id.clone(), op.content.clone(), op.metadata.clone());
                ltm.add(note, op.embedding.clone()).unwrap();
            }
            "get" => {
                let result = ltm.get(&op.id).unwrap();
                if let Some(expected) = &op.expected {
                    assert!(result.is_some(), "Expected to find id: {}", op.id);
                    let note = result.unwrap();
                    assert_eq!(note.id, expected.get("id").unwrap().as_str().unwrap());
                    assert_eq!(
                        note.content,
                        expected.get("content").unwrap().as_str().unwrap()
                    );
                } else {
                    assert!(result.is_none(), "Expected id {} to not exist", op.id);
                }
            }
            "delete" => {
                let result = ltm.delete(&op.id).unwrap();
                assert!(result, "Expected delete to succeed");
            }
            _ => panic!("Unknown operation type: {}", op.op_type),
        }
    }
}

#[test]
fn test_ltm_metadata_filtering_fixture() {
    let fixtures = load_fixtures("ltm_fixtures.json");
    let fixture = &fixtures["ltm_metadata_filtering"];

    let temp_file = NamedTempFile::new().unwrap();
    let mut ltm =
        cortex_memory::LongTermMemory::new(temp_file.path(), fixture.dimensionality).unwrap();

    for op in &fixture.operations {
        match op.op_type.as_str() {
            "add" => {
                let note = MemoryNote::new(op.id.clone(), op.content.clone(), op.metadata.clone());
                ltm.add(note, op.embedding.clone()).unwrap();
            }
            "search" => {
                let filter = if !op.filter.is_empty() {
                    Some(&op.filter)
                } else {
                    None
                };
                let results = ltm.search(&op.query_embedding, op.k, filter).unwrap();
                let result_ids: Vec<String> = results.iter().map(|note| note.id.clone()).collect();
                assert_eq!(result_ids, op.expected_ids, "Filtered search mismatch");
            }
            _ => panic!("Unknown operation type: {}", op.op_type),
        }
    }
}

#[test]
fn test_manager_stm_to_ltm_promotion_fixture() {
    let fixtures = load_fixtures("manager_fixtures.json");
    let fixture = &fixtures["manager_stm_to_ltm_promotion"];

    let temp_file = NamedTempFile::new().unwrap();
    let mut manager = MemoryManager::new(
        temp_file.path(),
        fixture.dimensionality,
        fixture.stm_capacity,
    )
    .unwrap();

    for op in &fixture.operations {
        match op.op_type.as_str() {
            "add" => {
                let note = MemoryNote::new(op.id.clone(), op.content.clone(), op.metadata.clone());
                manager.add(note, op.embedding.clone()).unwrap();
            }
            "promote" => {
                let result = manager.promote_to_ltm(&op.id, op.embedding.clone()).unwrap();
                if let Some(expected) = &op.expected {
                    assert_eq!(result, expected.as_bool().unwrap());
                }
            }
            "get_from_ltm" => {
                let result = manager.ltm.get(&op.id).unwrap();
                if let Some(expected) = &op.expected {
                    assert!(result.is_some(), "Expected to find id in LTM: {}", op.id);
                    let note = result.unwrap();
                    assert_eq!(note.id, expected.get("id").unwrap().as_str().unwrap());
                    assert_eq!(
                        note.content,
                        expected.get("content").unwrap().as_str().unwrap()
                    );
                }
            }
            _ => panic!("Unknown operation type: {}", op.op_type),
        }
    }
}
