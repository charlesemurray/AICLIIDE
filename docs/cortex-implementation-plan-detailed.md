# Cortex Rust Implementation - Detailed Execution Plan

## Principles

1. **Always compiles** - Every commit must compile
2. **Always tested** - Every feature has tests before moving on
3. **Small steps** - No placeholders, only working code
4. **Git commits** - Commit after each completed step
5. **Validation** - Verify against Python behavior at each phase
6. **Analysis** - Document what works and what's next
7. **Python Reference** - Leverage existing Python Cortex implementation for verification

---

## Python Implementation Verification Strategy

### Overview
The Python Cortex implementation at `/local/workspace/q-cli/cortex` is our reference. We verify Rust implementation consistency by comparing behavior, not by porting Python code directly.

### Verification Methods

#### 1. Test Fixture Generation
**Goal**: Create golden datasets from Python Cortex

**Process**:
```bash
# Create script to generate test fixtures
cd /local/workspace/q-cli/cortex
python3 scripts/generate_test_fixtures.py
```

**Fixtures to Generate**:
- `fixtures/stm_operations.json` - STM add/search/evict operations with expected results
- `fixtures/ltm_operations.json` - LTM store/recall operations
- `fixtures/search_rankings.json` - Query → expected ranking order
- `fixtures/metadata_filters.json` - Filter operations → expected results
- `fixtures/edge_cases.json` - Empty queries, duplicates, large batches

**Fixture Format**:
```json
{
  "test_name": "stm_basic_search",
  "setup": {
    "capacity": 100,
    "items": [
      {"id": "1", "content": "rust programming", "embedding": [0.1, 0.2, ...], "metadata": {...}},
      {"id": "2", "content": "python programming", "embedding": [0.15, 0.25, ...], "metadata": {...}}
    ]
  },
  "operation": {
    "type": "search",
    "query_embedding": [0.1, 0.2, ...],
    "k": 5
  },
  "expected": {
    "result_ids": ["1", "2"],
    "min_similarity": 0.8
  }
}
```

**Rust Test Usage**:
```rust
#[test]
fn test_against_python_fixture() {
    let fixture: TestFixture = load_fixture("fixtures/stm_operations.json");
    
    let mut stm = ShortTermMemory::new(fixture.setup.capacity);
    
    // Apply setup
    for item in fixture.setup.items {
        stm.add(item.id, item.content, item.embedding, item.metadata);
    }
    
    // Execute operation
    let results = stm.search(&fixture.operation.query_embedding, fixture.operation.k);
    
    // Verify against expected
    assert_eq!(results.ids(), fixture.expected.result_ids);
}
```

#### 2. Python Test Analysis
**Goal**: Identify what Python tests cover and port to Rust

**Process**:
```bash
# Find all Python tests
find /local/workspace/q-cli/cortex -name "test_*.py" -o -name "*_test.py"

# Analyze test coverage
python3 -m pytest --collect-only /local/workspace/q-cli/cortex
```

**Test Categories to Port**:
- STM: add, search, eviction, capacity limits
- LTM: store, recall, metadata filtering
- Integration: STM→LTM promotion, cross-memory search
- Edge cases: empty inputs, duplicates, concurrent access
- Performance: large datasets, search speed

**Porting Checklist**:
```markdown
- [ ] test_stm_add_and_search → test_stm_add_and_search.rs
- [ ] test_stm_lru_eviction → test_stm_lru_eviction.rs
- [ ] test_ltm_metadata_filter → test_ltm_metadata_filter.rs
- [ ] test_recall_by_keywords → test_recall_by_keywords.rs
- [ ] test_empty_query → test_empty_query.rs
```

#### 3. Behavioral Comparison Tests
**Goal**: Run identical operations in both implementations

**Implementation**:
```rust
// crates/cortex-memory/tests/python_comparison.rs
#[test]
fn test_search_ranking_matches_python() {
    // Load Python-generated results
    let python_results = load_json("fixtures/search_rankings.json");
    
    // Run same query in Rust
    let mut stm = ShortTermMemory::new(100);
    setup_from_fixture(&mut stm, &python_results.setup);
    
    let rust_results = stm.search(&python_results.query, 10);
    
    // Compare rankings (allow small floating point differences)
    for (rust_item, python_item) in rust_results.iter().zip(python_results.expected.iter()) {
        assert_eq!(rust_item.id, python_item.id);
        assert!((rust_item.score - python_item.score).abs() < 0.001);
    }
}
```

#### 4. Integration Test Harness
**Goal**: Side-by-side execution comparison

**Script**: `scripts/verify_rust_implementation.py`
```python
#!/usr/bin/env python3
"""
Runs identical operations in Python Cortex and Rust cortex-memory,
compares results to verify consistency.
"""

import json
import subprocess
from cortex import ShortTermMemory as PythonSTM

def test_stm_operations():
    # Python execution
    py_stm = PythonSTM(capacity=100)
    py_stm.add("1", "content", embedding=[0.1, 0.2, 0.3], metadata={})
    py_results = py_stm.search([0.1, 0.2, 0.3], k=5)
    
    # Rust execution via CLI
    rust_output = subprocess.check_output([
        "cargo", "run", "--bin", "cortex-test-harness", "--",
        "--operation", "stm_search",
        "--fixture", "test_data.json"
    ])
    rust_results = json.loads(rust_output)
    
    # Compare
    assert py_results["ids"] == rust_results["ids"]
    assert_similar_scores(py_results["scores"], rust_results["scores"])

if __name__ == "__main__":
    test_stm_operations()
    print("✅ Rust implementation matches Python behavior")
```

**Usage**:
```bash
# Run after each phase
python3 scripts/verify_rust_implementation.py --phase stm
python3 scripts/verify_rust_implementation.py --phase ltm
python3 scripts/verify_rust_implementation.py --phase integration
```

#### 5. Behavioral Documentation
**Goal**: Document expected behavior from Python code

**Process**:
1. Read Python source: `/local/workspace/q-cli/cortex/cortex/memory/stm.py`
2. Document behavior: `docs/python_behavior_reference.md`
3. Check off as Rust matches

**Example Documentation**:
```markdown
# Python Cortex Behavior Reference

## ShortTermMemory

### add() method
**Python behavior**:
- Accepts: id (str), content (str), embedding (list[float]), metadata (dict)
- If capacity reached: evicts oldest item (FIFO)
- Updates `updated_at` timestamp
- Returns: None

**Rust verification**:
- [ ] Same parameters accepted
- [ ] LRU eviction matches FIFO behavior
- [ ] Timestamps updated correctly
- [ ] Returns Result<()>

### search() method
**Python behavior**:
- Brute-force cosine similarity
- Returns top-k results sorted by similarity (descending)
- Filters by metadata if provided
- Empty query returns empty results

**Rust verification**:
- [ ] Cosine similarity calculation matches (within 0.001)
- [ ] Ranking order identical
- [ ] Metadata filtering works same way
- [ ] Edge cases handled identically
```

#### 6. Continuous Verification
**Goal**: Automated checks during development

**CI Integration**:
```yaml
# .github/workflows/cortex-verification.yml
name: Verify Cortex Implementation

on: [push, pull_request]

jobs:
  verify:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      
      - name: Setup Python
        uses: actions/setup-python@v2
        with:
          python-version: '3.9'
      
      - name: Install Python Cortex
        run: |
          cd cortex
          pip install -e .
      
      - name: Generate Test Fixtures
        run: python3 scripts/generate_test_fixtures.py
      
      - name: Build Rust Implementation
        run: cargo build --release -p cortex-memory
      
      - name: Run Verification Tests
        run: |
          cargo test -p cortex-memory --test python_comparison
          python3 scripts/verify_rust_implementation.py
```

### Verification Schedule

**Phase 0 (Foundation)**:
- ✅ Data structures match Python equivalents
- ✅ Serialization format compatible

**Phase 1 (STM)**:
- Generate STM fixtures from Python
- Port Python STM tests to Rust
- Run side-by-side comparison
- Document any behavioral differences

**Phase 2 (LTM)**:
- Generate LTM fixtures from Python
- Port Python LTM tests to Rust
- Verify metadata filtering matches
- Compare search rankings

**Phase 3 (Integration)**:
- Generate end-to-end fixtures
- Test STM→LTM promotion behavior
- Verify cross-memory search
- Performance comparison

**Phase 4 (Q CLI Integration)**:
- Test with real Q CLI workflows
- Compare memory persistence
- Verify session isolation

### Success Criteria

For each phase, verification is complete when:
- ✅ All Python tests ported and passing in Rust
- ✅ Fixture-based tests passing (100% match)
- ✅ Side-by-side comparison script passes
- ✅ Behavioral documentation complete and checked off
- ✅ No unexplained differences in behavior
- ✅ Performance within acceptable range (document if slower)

### Handling Differences

If Rust behavior differs from Python:

1. **Document the difference** in `docs/rust_python_differences.md`
2. **Determine if intentional** (e.g., performance optimization)
3. **If bug**: Fix Rust to match Python
4. **If improvement**: Document why Rust approach is better
5. **Update tests** to reflect intentional differences

**Example**:
```markdown
## Difference: STM Eviction Strategy

**Python**: FIFO (First In, First Out)
**Rust**: LRU (Least Recently Used)

**Reason**: LRU is more appropriate for memory cache behavior
**Impact**: Items accessed recently stay in cache longer
**Validation**: Tested with access patterns, Rust performs better
**Status**: Intentional improvement ✅
```

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
