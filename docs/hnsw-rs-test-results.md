# hnsw_rs v0.3.1 - Test Results & Gap Analysis

## Test Results Summary

### ✅ What Works

| Feature | Status | Notes |
|---------|--------|-------|
| Insert vectors | ✅ Works | `hnsw.insert((&vec, id))` |
| Search (top-K) | ✅ Works | `hnsw.search(&query, k, ef_search)` |
| Cosine similarity | ✅ Works | `DistCosine` distance function |
| Thread-safe reads | ✅ Works | Can wrap in `Arc` for sharing |
| Auto-resize | ✅ Works | Can insert beyond initial capacity |
| usize IDs | ✅ Works | Native ID type |

### ❌ What's Missing (Critical)

| Feature | Status | Impact | Workaround |
|---------|--------|--------|------------|
| Delete by ID | ❌ Not found | 🔴 **BLOCKER** | Soft delete or rebuild |
| Get by ID | ❌ Not found | 🔴 **BLOCKER** | Linear search data_points |
| Filtered search | ❌ Not found | 🟡 High | Post-filter results |
| String IDs | ❌ Not supported | 🟡 High | ID mapping layer |
| Update vector | ❌ Not found | 🟢 Low | Delete + Add |

### ⚠️ What's Unclear

| Feature | Status | Need to Check |
|---------|--------|---------------|
| Persistence | ⚠️ Unknown | How does Q CLI persist? |
| Concurrent writes | ⚠️ Unknown | Requires `&mut` or interior mutability? |
| Memory usage | ⚠️ Unknown | How much overhead per vector? |

---

## Detailed Analysis

### 1. Delete Operation - CRITICAL GAP

**What Cortex needs**:
```python
# From ChromaRetriever
def delete_document(self, doc_id: str):
    self.collection.delete(ids=[doc_id])
```

**What hnsw_rs provides**:
```rust
// NO DELETE METHOD FOUND
// Checked: insert(), search(), len(), is_empty()
// Missing: delete(), remove(), mark_deleted()
```

**Impact**: 
- Cannot remove memories from LTM
- Memory leaks over time
- No way to implement memory evolution (merge requires delete)

**Workarounds**:

**A. Soft Delete** (Easiest)
```rust
// Add "deleted" flag to metadata
pub struct DataPoint {
    pub id: usize,
    pub payload: HashMap<String, Value>,  // Add "deleted": true
    pub vector: Vec<f32>,
}

// Filter out deleted in search
fn search(&self, query: &[f32], limit: usize) -> Vec<SearchResult> {
    let results = self.index.search(query, limit * 2, 100);
    results.into_iter()
        .filter(|(id, _)| !self.data_points[*id].is_deleted())
        .take(limit)
        .collect()
}
```
- ✅ Simple to implement
- ❌ Wastes memory (deleted items stay in index)
- ❌ Slows down search (need to over-fetch)
- ❌ Requires periodic compaction

**B. Rebuild Index** (Nuclear option)
```rust
fn delete(&mut self, id: usize) -> Result<()> {
    // Remove from data_points
    self.data_points.retain(|p| p.id != id);
    
    // Rebuild entire HNSW index
    self.rebuild_index()?;
    
    Ok(())
}
```
- ✅ Actually removes data
- ❌ O(n) operation
- ❌ Blocks all operations during rebuild
- ❌ Unacceptable for frequent deletes

**C. Switch to hnswlib** (Like Chroma)
```rust
// Use Chroma's approach with hnswlib C++ library
// Has native delete support
index.delete(id)?;
```
- ✅ Native delete support
- ✅ Proven in Chroma
- ❌ C++ dependency (harder to package)
- ❌ More complex build

---

### 2. Get by ID - CRITICAL GAP

**What Cortex needs**:
```python
# From ChromaRetriever
def get_document(self, doc_id: str) -> Optional[Dict]:
    results = self.collection.get(ids=[doc_id])
```

**What hnsw_rs provides**:
```rust
// NO GET METHOD FOUND
// Can only search, not direct lookup
```

**Impact**:
- Cannot retrieve specific memory by ID
- Must search or linear scan
- Inefficient for direct access

**Workarounds**:

**A. Maintain Separate HashMap** (Current Q CLI approach)
```rust
pub struct SemanticContext {
    data_points: Vec<DataPoint>,  // All documents
    index: Option<VectorIndex>,   // HNSW index
}

// Get by ID requires linear search through Vec
fn get(&self, id: usize) -> Option<&DataPoint> {
    self.data_points.iter().find(|p| p.id == id)
}
```
- ✅ Already implemented in Q CLI
- ❌ O(n) lookup
- ❌ Slow for large datasets

**B. Add ID Index** (Better)
```rust
pub struct SemanticContext {
    data_points: Vec<DataPoint>,
    id_index: HashMap<usize, usize>,  // id -> vec index
    index: Option<VectorIndex>,
}

fn get(&self, id: usize) -> Option<&DataPoint> {
    self.id_index.get(&id)
        .and_then(|&idx| self.data_points.get(idx))
}
```
- ✅ O(1) lookup
- ✅ Small memory overhead
- ✅ Easy to add

---

### 3. Filtered Search - HIGH PRIORITY GAP

**What Cortex needs**:
```python
# From ChromaRetriever
results = self.collection.query(
    query_embeddings=[embedding],
    n_results=k,
    where={"user_id": {"$eq": "user123"}},  # Filter during search
)
```

**What hnsw_rs provides**:
```rust
// Only basic search
fn search(&self, query: &[f32], k: usize, ef_search: usize) -> Vec<Neighbour>
// No allowed_ids, no disallowed_ids, no filter parameter
```

**Comparison with Chroma's hnswlib**:
```rust
// Chroma's hnswlib has:
fn query(
    &self,
    vector: &[f32],
    k: usize,
    allowed_ids: &[usize],      // Pre-filter
    disallowed_ids: &[usize],   // Exclude
) -> Result<(Vec<usize>, Vec<f32>)>
```

**Impact**:
- Must post-filter all results
- Inefficient for selective queries
- Need to over-fetch to get enough results

**Workarounds**:

**A. Post-Filter** (Only option)
```rust
fn search_with_filter(
    &self,
    query: &[f32],
    limit: usize,
    filter: Option<Filter>,
) -> Vec<SearchResult> {
    // Over-fetch to account for filtering
    let fetch_limit = if filter.is_some() { limit * 3 } else { limit };
    
    let results = self.index.search(query, fetch_limit, 100);
    
    // Filter results
    results.into_iter()
        .filter(|(id, _)| {
            let point = &self.data_points[*id];
            filter.as_ref().map_or(true, |f| matches_filter(point, f))
        })
        .take(limit)
        .collect()
}
```
- ✅ Works
- ❌ Inefficient (fetches unnecessary results)
- ❌ May not get enough results if many filtered out
- ⚠️ Acceptable for Cortex's scale (hundreds to thousands)

---

### 4. String IDs - HIGH PRIORITY GAP

**What Cortex needs**:
```python
# String UUIDs
doc_id = "550e8400-e29b-41d4-a716-446655440000"
self.collection.add(ids=[doc_id], ...)
```

**What hnsw_rs provides**:
```rust
// Only usize IDs
hnsw.insert((&vector, 0_usize));
```

**Impact**:
- Cannot use UUIDs directly
- Need bidirectional mapping
- Mapping must persist

**Workarounds**:

**A. ID Mapping Layer** (Required)
```rust
pub struct IdMapper {
    string_to_usize: HashMap<String, usize>,
    usize_to_string: HashMap<usize, String>,
    next_id: AtomicUsize,
}

impl IdMapper {
    pub fn get_or_create(&mut self, string_id: String) -> usize {
        if let Some(&id) = self.string_to_usize.get(&string_id) {
            return id;
        }
        
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        self.string_to_usize.insert(string_id.clone(), id);
        self.usize_to_string.insert(id, string_id);
        id
    }
    
    pub fn get_string(&self, id: usize) -> Option<&String> {
        self.usize_to_string.get(&id)
    }
}
```
- ✅ Bidirectional mapping
- ✅ Persistent
- ❌ Extra memory overhead
- ❌ Must persist mapping separately

**B. Store in Metadata** (Alternative)
```rust
// Store string ID in DataPoint payload
pub struct DataPoint {
    pub id: usize,  // Internal HNSW ID
    pub payload: HashMap<String, Value>,  // Contains "cortex_id": "uuid-string"
    pub vector: Vec<f32>,
}
```
- ✅ No separate mapping needed
- ✅ ID travels with document
- ❌ Still need reverse lookup (usize -> string)
- ⚠️ Hybrid approach: store in payload + maintain reverse map

---

### 5. Persistence - UNCLEAR

**What Q CLI does**:
```rust
// From SemanticContext
pub fn save(&self) -> Result<()> {
    // Save data_points as JSON
    let file = File::create(&self.data_path)?;
    serde_json::to_writer(writer, &self.data_points)?;
    
    // Rebuild index on load
    if !context.data_points.is_empty() {
        context.rebuild_index()?;
    }
    
    Ok(())
}
```

**Findings**:
- ✅ Q CLI saves data_points (documents + metadata)
- ✅ Rebuilds HNSW index from data on load
- ❌ Does NOT persist HNSW index itself
- ⚠️ Rebuild on every restart (acceptable for moderate sizes)

**For Cortex**:
- ✅ Can use same approach
- ✅ Persist documents + metadata + ID mapping
- ✅ Rebuild HNSW on startup
- ⚠️ Startup time proportional to dataset size

---

## Comparison: hnsw_rs vs hnswlib (Chroma)

| Feature | hnsw_rs | hnswlib (Chroma) | Winner |
|---------|---------|------------------|--------|
| Language | Pure Rust | C++ with bindings | hnsw_rs (easier) |
| Delete | ❌ No | ✅ Yes | hnswlib |
| Get by ID | ❌ No | ✅ Yes | hnswlib |
| Filtered search | ❌ No | ✅ Yes (allowed_ids) | hnswlib |
| Persistence | ⚠️ Rebuild | ✅ Native | hnswlib |
| Single binary | ✅ Yes | ⚠️ Harder | hnsw_rs |
| Cross-platform | ✅ Easy | ⚠️ Complex | hnsw_rs |
| Performance | ✅ Good | ✅ Excellent | Tie |
| Maturity | ⚠️ Moderate | ✅ Battle-tested | hnswlib |

---

## Recommendations

### Option 1: Use hnsw_rs with Workarounds (RECOMMENDED)

**Approach**:
- ✅ Soft deletes with periodic compaction
- ✅ ID mapping layer (String ↔ usize)
- ✅ Post-filtering for metadata
- ✅ ID index HashMap for O(1) lookup
- ✅ Rebuild HNSW on startup

**Pros**:
- ✅ Pure Rust, single binary
- ✅ No external dependencies
- ✅ Cross-platform easy
- ✅ Acceptable for Cortex's scale

**Cons**:
- ⚠️ Soft deletes waste memory
- ⚠️ Post-filtering less efficient
- ⚠️ Need compaction strategy

**Timeline**: 2-3 weeks

---

### Option 2: Switch to hnswlib (Like Chroma)

**Approach**:
- Use Chroma's Rust bindings to hnswlib
- Get native delete, get, filtered search
- More complex build but better features

**Pros**:
- ✅ All features work natively
- ✅ Battle-tested (Chroma uses it)
- ✅ Better performance

**Cons**:
- ❌ C++ dependency
- ❌ Harder cross-compilation
- ❌ More complex build
- ⚠️ Still single binary (static linking)

**Timeline**: 3-4 weeks

---

### Option 3: Simplified In-Memory (Fallback)

**Approach**:
- Skip HNSW entirely
- Brute-force cosine similarity
- O(n) search but simple

**Pros**:
- ✅ All operations trivial
- ✅ Pure Rust, simple
- ✅ True deletes, no soft delete
- ✅ Fast to implement

**Cons**:
- ❌ O(n) search performance
- ❌ Limited scale (< 10k items)

**Timeline**: 1-2 weeks

---

## Decision Matrix

| Criteria | Option 1 (hnsw_rs) | Option 2 (hnswlib) | Option 3 (Simple) |
|----------|-------------------|-------------------|-------------------|
| Single binary | ✅ Easy | ⚠️ Possible | ✅ Easy |
| Cross-platform | ✅ Easy | ⚠️ Hard | ✅ Easy |
| Delete support | ⚠️ Soft | ✅ Native | ✅ Native |
| Get by ID | ⚠️ O(1) with HashMap | ✅ Native | ✅ O(1) |
| Filtered search | ⚠️ Post-filter | ✅ Pre-filter | ✅ Native |
| Performance | ✅ Good | ✅ Excellent | ⚠️ Limited |
| Complexity | ⚠️ Medium | ⚠️ High | ✅ Low |
| Risk | 🟡 Medium | 🟡 Medium | 🟢 Low |
| Timeline | 2-3 weeks | 3-4 weeks | 1-2 weeks |

---

## Final Recommendation

**Start with Option 1 (hnsw_rs with workarounds)**

**Rationale**:
1. Leverages existing Q CLI infrastructure
2. Pure Rust, single binary guaranteed
3. Workarounds acceptable for Cortex's scale
4. Can optimize later if needed
5. Lowest risk for single-binary requirement

**Implementation Plan**:
1. Add ID mapping layer (String ↔ usize)
2. Add ID index HashMap for O(1) get
3. Implement soft deletes with "deleted" flag
4. Implement post-filtering for metadata
5. Add compaction strategy (rebuild without deleted)
6. Test with realistic Cortex workload

**If performance is insufficient**:
- Consider Option 2 (hnswlib) as upgrade path
- Or Option 3 (Simple) if scale is small enough

---

## Next Steps

1. ✅ **DONE**: Test hnsw_rs capabilities
2. ✅ **DONE**: Document gaps and workarounds
3. **TODO**: Prototype ID mapping layer
4. **TODO**: Prototype soft delete + compaction
5. **TODO**: Benchmark with realistic data
6. **TODO**: Make final decision
