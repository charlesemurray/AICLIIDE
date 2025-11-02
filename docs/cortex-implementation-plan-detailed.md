# Cortex Rust Implementation - Detailed Execution Plan

## Principles

1. **Always compiles** - Every commit must compile
2. **Always tested** - Every feature has tests before moving on
3. **Small steps** - No placeholders, only working code
4. **Git commits** - Commit after each completed step
5. **Validation** - Verify against Python behavior at each phase
6. **Analysis** - Document what works and what's next

---

## Phase 0: Foundation (Days 1-3)

### Step 0.1: Error Types (30 minutes)
**Goal**: Define error handling

**Implementation**:
```rust
// crates/cortex-memory/src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CortexError {
    #[error("Memory not found: {0}")]
    NotFound(String),
    
    #[error("Embedding error: {0}")]
    EmbeddingError(String),
    
    #[error("Storage error: {0}")]
    StorageError(#[from] hnswlib::HnswError),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

pub type Result<T> = std::result::Result<T, CortexError>;
```

**Tests**:
```rust
#[test]
fn test_error_types() {
    let err = CortexError::NotFound("test".to_string());
    assert!(err.to_string().contains("not found"));
}
```

**Validation**:
- ✅ Compiles
- ✅ Tests pass
- ✅ Error messages are clear

**Git Commit**:
```bash
git add crates/cortex-memory/src/error.rs
git commit -m "feat(cortex): add error types"
```

**Analysis**: Error types complete, ready for data structures

---

### Step 0.2: MemoryNote Structure (1 hour)
**Goal**: Core data structure matching Python

**Implementation**:
```rust
// crates/cortex-memory/src/memory_note.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryNote {
    pub id: String,
    pub content: String,
    pub metadata: HashMap<String, Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl MemoryNote {
    pub fn new(id: String, content: String, metadata: HashMap<String, Value>) -> Self {
        let now = Utc::now();
        Self {
            id,
            content,
            metadata,
            created_at: now,
            updated_at: now,
        }
    }
    
    pub fn keywords(&self) -> Vec<String> {
        self.metadata
            .get("keywords")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default()
    }
    
    pub fn context(&self) -> String {
        self.metadata
            .get("context")
            .and_then(|v| v.as_str())
            .unwrap_or("General")
            .to_string()
    }
    
    pub fn tags(&self) -> Vec<String> {
        self.metadata
            .get("tags")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default()
    }
}
```

**Tests**:
```rust
#[test]
fn test_memory_note_creation() {
    let note = MemoryNote::new(
        "test-id".to_string(),
        "test content".to_string(),
        HashMap::new(),
    );
    
    assert_eq!(note.id, "test-id");
    assert_eq!(note.content, "test content");
    assert_eq!(note.context(), "General");
}

#[test]
fn test_memory_note_keywords() {
    let mut metadata = HashMap::new();
    metadata.insert("keywords".to_string(), json!(["rust", "memory"]));
    
    let note = MemoryNote::new("id".to_string(), "content".to_string(), metadata);
    
    assert_eq!(note.keywords(), vec!["rust", "memory"]);
}

#[test]
fn test_memory_note_serialization() {
    let note = MemoryNote::new("id".to_string(), "content".to_string(), HashMap::new());
    
    let json = serde_json::to_string(&note).unwrap();
    let deserialized: MemoryNote = serde_json::from_str(&json).unwrap();
    
    assert_eq!(note, deserialized);
}
```

**Validation**:
- ✅ Compiles
- ✅ All tests pass
- ✅ Matches Python MemoryNote structure
- ✅ Serialization works

**Git Commit**:
```bash
git add crates/cortex-memory/src/memory_note.rs
git commit -m "feat(cortex): add MemoryNote data structure"
```

**Analysis**: MemoryNote complete and tested, ready for ID mapping

---

### Step 0.3: ID Mapping Layer (2 hours)
**Goal**: Map String IDs ↔ usize for hnswlib

**Implementation**:
```rust
// crates/cortex-memory/src/id_mapper.rs
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug)]
pub struct IdMapper {
    string_to_usize: HashMap<String, usize>,
    usize_to_string: HashMap<usize, String>,
    next_id: AtomicUsize,
}

impl IdMapper {
    pub fn new() -> Self {
        Self {
            string_to_usize: HashMap::new(),
            usize_to_string: HashMap::new(),
            next_id: AtomicUsize::new(0),
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
}
```

**Tests**:
```rust
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
```

**Validation**:
- ✅ Compiles
- ✅ All tests pass
- ✅ Bidirectional mapping works
- ✅ Removal works
- ✅ Thread-safe ID generation

**Git Commit**:
```bash
git add crates/cortex-memory/src/id_mapper.rs
git commit -m "feat(cortex): add ID mapping layer for String<->usize conversion"
```

**Analysis**: ID mapping complete, ready for HNSW wrapper

---

### Step 0.4: Update lib.rs (15 minutes)
**Goal**: Export new modules

**Implementation**:
```rust
// crates/cortex-memory/src/lib.rs
pub mod error;
pub mod memory_note;
pub mod id_mapper;

pub use error::{CortexError, Result};
pub use memory_note::MemoryNote;
pub use id_mapper::IdMapper;

#[cfg(test)]
mod tests {
    // Keep existing hnswlib tests
}
```

**Validation**:
- ✅ Compiles
- ✅ All tests still pass
- ✅ Public API is clean

**Git Commit**:
```bash
git add crates/cortex-memory/src/lib.rs
git commit -m "feat(cortex): export foundation modules"
```

**Phase 0 Complete**: Foundation ready for STM implementation

---

## Phase 1: Short-Term Memory (Days 4-7)

### Step 1.1: HNSW Wrapper (3 hours)
**Goal**: Wrap hnswlib with String ID support

**Implementation**:
```rust
// crates/cortex-memory/src/hnsw_wrapper.rs
use crate::{IdMapper, Result, CortexError};
use hnswlib::{HnswIndex, HnswIndexInitConfig, HnswDistanceFunction};

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
        
        let index = HnswIndex::init(config)
            .map_err(|e| CortexError::StorageError(e))?;
        
        Ok(Self {
            index,
            id_mapper: IdMapper::new(),
            dimensionality,
        })
    }
    
    pub fn add(&mut self, string_id: String, vector: &[f32]) -> Result<()> {
        if vector.len() != self.dimensionality {
            return Err(CortexError::InvalidInput(
                format!("Expected {} dimensions, got {}", self.dimensionality, vector.len())
            ));
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
    
    pub fn search(
        &self,
        query: &[f32],
        k: usize,
        allowed_ids: Option<&[String]>,
    ) -> Result<Vec<(String, f32)>> {
        if query.len() != self.dimensionality {
            return Err(CortexError::InvalidInput(
                format!("Expected {} dimensions, got {}", self.dimensionality, query.len())
            ));
        }
        
        // Convert string IDs to numeric
        let numeric_allowed: Vec<usize> = if let Some(ids) = allowed_ids {
            ids.iter()
                .filter_map(|s| self.id_mapper.get_numeric(s))
                .collect()
        } else {
            vec![]
        };
        
        let allowed_slice = if numeric_allowed.is_empty() {
            &[]
        } else {
            &numeric_allowed
        };
        
        // Query HNSW
        let (ids, distances) = self.index.query(query, k, allowed_slice, &[])?;
        
        // Convert back to string IDs
        let results: Vec<(String, f32)> = ids.iter()
            .zip(distances.iter())
            .filter_map(|(&id, &dist)| {
                self.id_mapper.get_string(id).map(|s| (s.clone(), dist))
            })
            .collect();
        
        Ok(results)
    }
}
```

**Tests**:
```rust
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
    
    wrapper.add("doc1".to_string(), &vec![1.0, 2.0, 3.0]).unwrap();
    wrapper.add("doc2".to_string(), &vec![1.1, 2.1, 3.1]).unwrap();
    
    let results = wrapper.search(&vec![1.0, 2.0, 3.0], 2, None).unwrap();
    
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].0, "doc1");
}

#[test]
fn test_hnsw_wrapper_delete() {
    let mut wrapper = HnswWrapper::new(3, 100).unwrap();
    
    wrapper.add("doc1".to_string(), &vec![1.0, 2.0, 3.0]).unwrap();
    assert!(wrapper.delete("doc1").unwrap());
    assert_eq!(wrapper.get("doc1").unwrap(), None);
}

#[test]
fn test_hnsw_wrapper_filtered_search() {
    let mut wrapper = HnswWrapper::new(3, 100).unwrap();
    
    wrapper.add("doc1".to_string(), &vec![1.0, 2.0, 3.0]).unwrap();
    wrapper.add("doc2".to_string(), &vec![1.1, 2.1, 3.1]).unwrap();
    wrapper.add("doc3".to_string(), &vec![5.0, 6.0, 7.0]).unwrap();
    
    let allowed = vec!["doc1".to_string(), "doc3".to_string()];
    let results = wrapper.search(&vec![1.0, 2.0, 3.0], 3, Some(&allowed)).unwrap();
    
    assert!(!results.iter().any(|(id, _)| id == "doc2"));
}

#[test]
fn test_hnsw_wrapper_dimension_validation() {
    let mut wrapper = HnswWrapper::new(3, 100).unwrap();
    
    let result = wrapper.add("doc1".to_string(), &vec![1.0, 2.0]);
    assert!(result.is_err());
}
```

**Validation**:
- ✅ Compiles
- ✅ All tests pass
- ✅ String IDs work
- ✅ Filtered search works
- ✅ Dimension validation works

**Git Commit**:
```bash
git add crates/cortex-memory/src/hnsw_wrapper.rs
git commit -m "feat(cortex): add HNSW wrapper with String ID support"
```

**Analysis**: HNSW wrapper complete, ready for STM

---

### Step 1.2: Short-Term Memory (4 hours)
**Goal**: LRU cache with search

**Implementation**: (See design doc for full code)

**Tests**: (Comprehensive tests for add, search, eviction, etc.)

**Validation**:
- ✅ Compiles
- ✅ All tests pass
- ✅ LRU eviction works correctly
- ✅ Search returns correct results
- ✅ Matches Python STM behavior

**Git Commit**:
```bash
git add crates/cortex-memory/src/stm.rs
git commit -m "feat(cortex): implement short-term memory with LRU"
```

**Analysis**: STM complete, ready for LTM

---

## Commit Message Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types**:
- `feat`: New feature
- `fix`: Bug fix
- `test`: Adding tests
- `refactor`: Code refactoring
- `docs`: Documentation
- `chore`: Maintenance

**Example**:
```
feat(cortex): add short-term memory implementation

- Implements LRU cache with configurable capacity
- Brute-force cosine similarity search
- Automatic eviction of oldest items
- Tested against Python Cortex STM behavior

Closes #123
```

---

## Validation Checklist (Every Step)

Before committing, verify:

- [ ] Code compiles: `cargo build`
- [ ] Tests pass: `cargo test`
- [ ] No warnings: `cargo clippy`
- [ ] Formatted: `cargo fmt`
- [ ] Documentation: Public items have doc comments
- [ ] No placeholders: All functions fully implemented
- [ ] Git status clean: No untracked files

---

## Progress Tracking

After each phase, create analysis document:

```markdown
# Phase X Complete - Analysis

## What Was Built
- Component A: Description
- Component B: Description

## Tests Added
- X unit tests
- Y integration tests
- All passing

## Validation Results
- ✅ Compiles without warnings
- ✅ All tests pass
- ✅ Matches Python behavior for [specific cases]

## Performance
- Operation A: Xms
- Operation B: Yms

## What's Next
- Next component to build
- Dependencies needed
- Estimated time

## Blockers
- None / List any issues
```

---

## Daily Workflow

### Start of Day
1. Pull latest: `git pull`
2. Check what's next in plan
3. Review previous day's analysis

### During Implementation
1. Read step in plan
2. Implement minimal working code
3. Write tests
4. Run validation checklist
5. Commit
6. Update progress doc

### End of Day
1. Ensure everything compiles
2. All tests passing
3. Write analysis of what's complete
4. Commit analysis
5. Plan next day's work

---

## Example Session

```bash
# Step 1: Implement error types
vim crates/cortex-memory/src/error.rs
cargo test
cargo clippy
cargo fmt
git add crates/cortex-memory/src/error.rs
git commit -m "feat(cortex): add error types"

# Step 2: Implement MemoryNote
vim crates/cortex-memory/src/memory_note.rs
cargo test
cargo clippy
git add crates/cortex-memory/src/memory_note.rs
git commit -m "feat(cortex): add MemoryNote data structure"

# Step 3: Update lib.rs
vim crates/cortex-memory/src/lib.rs
cargo test
git add crates/cortex-memory/src/lib.rs
git commit -m "feat(cortex): export foundation modules"

# End of session
cargo build --release  # Final check
git log --oneline -5   # Review commits
```

---

## Anti-Patterns to Avoid

### ❌ Don't Do This
```rust
pub fn search(&self, query: &str) -> Vec<SearchResult> {
    // TODO: implement search
    vec![]
}
```

### ✅ Do This Instead
```rust
// Don't add the function until you can implement it fully
// OR implement a minimal working version:
pub fn search(&self, query: &str) -> Vec<SearchResult> {
    // Brute force search (will optimize later)
    self.items.iter()
        .filter(|item| item.content.contains(query))
        .cloned()
        .collect()
}
```

---

## Summary

This plan ensures:
- ✅ Always compiles
- ✅ Always tested
- ✅ Small, complete steps
- ✅ Git commits after each step
- ✅ Validation at each step
- ✅ Progress analysis
- ✅ No placeholders

Ready to start Step 0.1?
