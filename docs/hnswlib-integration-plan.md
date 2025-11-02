# hnswlib Integration Plan

## How Cargo Handles C++ Automatically

### Step 1: Add Dependency (1 line)

```toml
# Cargo.toml
[dependencies]
hnswlib = { version = "0.8.2", git = "https://github.com/chroma-core/hnswlib.git", branch = "master" }
```

**That's it!** Cargo automatically:
1. ✅ Downloads hnswlib source
2. ✅ Runs build.rs (compiles C++)
3. ✅ Links into your binary
4. ✅ Handles all platforms

### Step 2: Use It (Just Like Any Rust Crate)

```rust
use hnswlib::{HnswIndex, HnswIndexConfig};

// Use it like normal Rust code
let index = HnswIndex::init(config)?;
index.add(id, &vector, false)?;
let results = index.query(&query, k, &[], &[])?;
```

**No manual C++ compilation needed!**

---

## Implementation Plan

### Phase 1: Add Dependency & Test (Day 1)

**Goal**: Verify hnswlib builds on your system

```bash
# 1. Add to Cargo.toml
cd crates/semantic-search-client
# Add hnswlib dependency

# 2. Test build
cargo build

# 3. Create simple test
cargo test test_hnswlib_basic
```

**Expected**: Builds successfully, C++ compiles automatically

---

### Phase 2: Create Wrapper (Days 2-3)

**Goal**: Wrap hnswlib with Cortex-friendly API

```rust
// crates/cortex-memory/src/hnsw_wrapper.rs

use hnswlib::{HnswIndex, HnswIndexConfig, HnswDistanceFunction};
use std::collections::HashMap;

pub struct CortexHnswIndex {
    index: HnswIndex,
    // String ID ↔ usize mapping
    string_to_id: HashMap<String, usize>,
    id_to_string: HashMap<usize, String>,
    next_id: usize,
}

impl CortexHnswIndex {
    pub fn new(dim: usize, max_elements: usize) -> Result<Self> {
        let config = HnswIndexConfig {
            distance_function: HnswDistanceFunction::Cosine,
            dimensionality: dim,
            max_elements,
            m: 16,
            ef_construction: 200,
            random_seed: 0,
            allow_replace_deleted: true,
            is_persistent: false,
            path: None,
        };
        
        let index = HnswIndex::init(config)?;
        
        Ok(Self {
            index,
            string_to_id: HashMap::new(),
            id_to_string: HashMap::new(),
            next_id: 0,
        })
    }
    
    pub fn add(&mut self, string_id: String, vector: &[f32]) -> Result<()> {
        // Get or create numeric ID
        let numeric_id = if let Some(&id) = self.string_to_id.get(&string_id) {
            id
        } else {
            let id = self.next_id;
            self.next_id += 1;
            self.string_to_id.insert(string_id.clone(), id);
            self.id_to_string.insert(id, string_id);
            id
        };
        
        // Add to HNSW
        self.index.add(numeric_id, vector, false)?;
        Ok(())
    }
    
    pub fn delete(&mut self, string_id: &str) -> Result<bool> {
        if let Some(&numeric_id) = self.string_to_id.get(string_id) {
            self.index.delete(numeric_id)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    pub fn get(&self, string_id: &str) -> Result<Option<Vec<f32>>> {
        if let Some(&numeric_id) = self.string_to_id.get(string_id) {
            let vector = self.index.get(numeric_id)?;
            Ok(Some(vector))
        } else {
            Ok(None)
        }
    }
    
    pub fn search(
        &self,
        query: &[f32],
        k: usize,
        filter_ids: Option<&[String]>,
    ) -> Result<Vec<(String, f32)>> {
        // Convert string IDs to numeric for filtering
        let allowed_ids: Vec<usize> = if let Some(ids) = filter_ids {
            ids.iter()
                .filter_map(|s| self.string_to_id.get(s).copied())
                .collect()
        } else {
            vec![]
        };
        
        let allowed_slice = if allowed_ids.is_empty() {
            &[]
        } else {
            &allowed_ids
        };
        
        // Query HNSW
        let (ids, distances) = self.index.query(query, k, allowed_slice, &[])?;
        
        // Convert back to string IDs
        let results: Vec<(String, f32)> = ids.iter()
            .zip(distances.iter())
            .filter_map(|(&id, &dist)| {
                self.id_to_string.get(&id).map(|s| (s.clone(), dist))
            })
            .collect();
        
        Ok(results)
    }
}
```

---

### Phase 3: Integrate with Cortex (Days 4-7)

**Goal**: Replace Q CLI's VectorIndex with hnswlib wrapper

```rust
// crates/cortex-memory/src/ltm.rs

use crate::hnsw_wrapper::CortexHnswIndex;

pub struct LongTermMemory {
    index: CortexHnswIndex,
    documents: HashMap<String, Document>,
}

impl LongTermMemory {
    pub async fn add(
        &mut self,
        id: String,
        content: String,
        metadata: HashMap<String, Value>,
        embedding: Vec<f32>,
    ) -> Result<()> {
        // Add to HNSW
        self.index.add(id.clone(), &embedding)?;
        
        // Store document
        self.documents.insert(id, Document {
            content,
            metadata,
            embedding,
        });
        
        Ok(())
    }
    
    pub async fn get(&self, id: &str) -> Result<Option<Document>> {
        Ok(self.documents.get(id).cloned())
    }
    
    pub async fn delete(&mut self, id: &str) -> Result<bool> {
        self.index.delete(id)?;
        Ok(self.documents.remove(id).is_some())
    }
    
    pub async fn search(
        &self,
        query_embedding: &[f32],
        limit: usize,
        filter: Option<&Filter>,
    ) -> Result<Vec<SearchResult>> {
        // Apply metadata filter to get allowed IDs
        let allowed_ids: Option<Vec<String>> = if let Some(f) = filter {
            let ids: Vec<String> = self.documents
                .iter()
                .filter(|(_, doc)| matches_filter(&doc.metadata, f))
                .map(|(id, _)| id.clone())
                .collect();
            Some(ids)
        } else {
            None
        };
        
        // Search HNSW with pre-filtering
        let results = self.index.search(
            query_embedding,
            limit,
            allowed_ids.as_deref(),
        )?;
        
        // Build search results
        let search_results: Vec<SearchResult> = results
            .into_iter()
            .filter_map(|(id, distance)| {
                self.documents.get(&id).map(|doc| SearchResult {
                    id: id.clone(),
                    content: doc.content.clone(),
                    score: 1.0 - distance,
                    distance,
                    metadata: doc.metadata.clone(),
                })
            })
            .collect();
        
        Ok(search_results)
    }
}
```

---

### Phase 4: Testing (Days 8-10)

**Unit Tests**:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_add_and_get() {
        let mut index = CortexHnswIndex::new(3, 100).unwrap();
        let vec = vec![1.0, 2.0, 3.0];
        
        index.add("doc1".to_string(), &vec).unwrap();
        let retrieved = index.get("doc1").unwrap().unwrap();
        
        assert_eq!(retrieved, vec);
    }
    
    #[test]
    fn test_delete() {
        let mut index = CortexHnswIndex::new(3, 100).unwrap();
        index.add("doc1".to_string(), &vec![1.0, 2.0, 3.0]).unwrap();
        
        assert!(index.delete("doc1").unwrap());
        assert!(index.get("doc1").unwrap().is_none());
    }
    
    #[test]
    fn test_search_with_filter() {
        let mut index = CortexHnswIndex::new(3, 100).unwrap();
        
        index.add("doc1".to_string(), &vec![1.0, 2.0, 3.0]).unwrap();
        index.add("doc2".to_string(), &vec![1.1, 2.1, 3.1]).unwrap();
        index.add("doc3".to_string(), &vec![5.0, 6.0, 7.0]).unwrap();
        
        // Search with filter
        let results = index.search(
            &vec![1.0, 2.0, 3.0],
            2,
            Some(&["doc1".to_string(), "doc2".to_string()]),
        ).unwrap();
        
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|(id, _)| id == "doc1" || id == "doc2"));
    }
}
```

**Integration Tests**:
```rust
#[tokio::test]
async fn test_cortex_memory_system() {
    let mut ltm = LongTermMemory::new().await.unwrap();
    
    // Add memories
    ltm.add(
        "mem1".to_string(),
        "Rust is great".to_string(),
        HashMap::new(),
        vec![1.0, 2.0, 3.0],
    ).await.unwrap();
    
    // Search
    let results = ltm.search(&vec![1.0, 2.0, 3.0], 5, None).await.unwrap();
    assert!(!results.is_empty());
    
    // Delete
    assert!(ltm.delete("mem1").await.unwrap());
    assert!(ltm.get("mem1").await.unwrap().is_none());
}
```

---

### Phase 5: Documentation (Days 11-12)

**Build Instructions**:
```markdown
# Building Q CLI with Cortex

## Prerequisites

### Linux
```bash
sudo apt install build-essential
```

### macOS
```bash
xcode-select --install
```

### Windows
Install Visual Studio Build Tools

## Build
```bash
cargo build --release
```

That's it! Cargo handles C++ compilation automatically.
```

---

## Timeline

| Phase | Days | Deliverable |
|-------|------|-------------|
| 1. Add dependency & test | 1 | hnswlib builds |
| 2. Create wrapper | 2-3 | CortexHnswIndex working |
| 3. Integrate with Cortex | 4-7 | LTM using hnswlib |
| 4. Testing | 8-10 | All tests passing |
| 5. Documentation | 11-12 | Build docs complete |
| **Buffer** | 13-20 | Bug fixes, polish |

**Total**: 3-4 weeks

---

## Cargo Handles Everything

### What Cargo Does Automatically

```toml
# You just add this:
[dependencies]
hnswlib = { git = "..." }
```

**Cargo automatically**:
1. ✅ Downloads source
2. ✅ Runs build.rs
3. ✅ Detects C++ compiler
4. ✅ Compiles C++ code
5. ✅ Links into binary
6. ✅ Caches compiled code
7. ✅ Handles incremental builds
8. ✅ Works on all platforms

**You never touch C++ directly!**

---

## No Manual Steps Required

### ❌ You DON'T need to:
- Manually compile C++
- Run cmake
- Configure build systems
- Manage C++ dependencies
- Write makefiles
- Handle linking

### ✅ You ONLY need to:
- Add one line to Cargo.toml
- Write Rust code
- Run `cargo build`

---

## Comparison with Current Dependencies

### onig_sys (Already Using)
```toml
[dependencies]
onig = "6.5"  # ← Cargo handles C compilation

[build-dependencies]
cc = "1.2"  # ← Automatically used
```

### hnswlib (Adding)
```toml
[dependencies]
hnswlib = { git = "..." }  # ← Cargo handles C++ compilation

[build-dependencies]
cc = "1.2"  # ← Already in hnswlib's Cargo.toml
```

**Identical process!**

---

## Next Steps

### 1. Create cortex-memory Crate
```bash
cd crates
cargo new cortex-memory --lib
```

### 2. Add hnswlib Dependency
```toml
# crates/cortex-memory/Cargo.toml
[dependencies]
hnswlib = { version = "0.8.2", git = "https://github.com/chroma-core/hnswlib.git", branch = "master" }
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### 3. Test Build
```bash
cd crates/cortex-memory
cargo build
```

**Expected output**:
```
   Compiling hnswlib v0.8.2
   Compiling cortex-memory v0.1.0
    Finished dev [unoptimized + debuginfo] target(s) in 45.2s
```

✅ If this works, you're ready to implement!

---

## Questions?

**Q: Do I need to install anything special?**
A: No! You already have everything (C++ compiler for onig_sys)

**Q: Will it work on CI/CD?**
A: Yes! Same as current build

**Q: What if build fails?**
A: Same troubleshooting as onig_sys (check compiler)

**Q: Can I start now?**
A: Yes! Just add the dependency and test build

Ready to start implementing?
