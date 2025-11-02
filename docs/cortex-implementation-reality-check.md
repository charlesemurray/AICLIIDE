# Cortex Implementation - Reality Check

## What I Now Know from Chroma

After examining Chroma's Rust implementation, here's what I learned:

### 1. HNSW Deletion Support

**Chroma uses `hnswlib` (C++ library with Rust bindings)**:
```rust
// From chroma/rust/index/src/hnsw.rs
fn delete(&self, id: usize) -> Result<(), Box<dyn ChromaError>> {
    self.index.delete(id).map_err(|e| WrappedHnswError(e).boxed())
}
```

**Q CLI uses `hnsw_rs` (pure Rust)**:
- Need to verify if `hnsw_rs` v0.3.1 supports deletion
- If not, this is a **critical blocker**

### 2. Document IDs

**Chroma**: Uses `usize` for IDs (same as Q CLI)
```rust
fn add(&self, id: usize, vector: &[f32])
fn delete(&self, id: usize)
fn get(&self, id: usize) -> Result<Option<Vec<f32>>>
```

**Q CLI**: Also uses `usize` in DataPoint
```rust
pub struct DataPoint {
    pub id: usize,  // Not String!
    pub payload: HashMap<String, serde_json::Value>,
    pub vector: Vec<f32>,
}
```

**Implication**: We need a mapping layer from String IDs (for Cortex) to usize (for HNSW)

### 3. Metadata Filtering

**Chroma approach**:
```rust
fn query(
    &self,
    vector: &[f32],
    k: usize,
    allowed_ids: &[usize],      // Pre-filtered IDs
    disallowed_ids: &[usize],   // Excluded IDs
) -> Result<(Vec<usize>, Vec<f32>)>
```

They filter **before** querying HNSW, not after. This is more efficient.

---

## Critical Questions I Still Can't Answer

### Question 1: Does `hnsw_rs` support deletion?

**What I need to check**:
```rust
// Does this exist in hnsw_rs?
impl Hnsw {
    pub fn delete(&self, id: usize) -> Result<()> { ... }
}
```

**If NO**: 
- We'd need to rebuild the entire index on every deletion
- OR switch to `hnswlib` (like Chroma)
- OR implement soft deletes (mark as deleted, filter in results)

**If YES**:
- How efficient is it?
- Does it require rebuilding?
- What happens to the index structure?

### Question 2: How to map String IDs to usize?

**Options**:

**A. Sequential Counter** (Simple but fragile)
```rust
struct IdMapper {
    next_id: AtomicUsize,
    string_to_usize: HashMap<String, usize>,
    usize_to_string: HashMap<usize, String>,
}
```
- Problem: Deletions leave gaps
- Problem: Not persistent across restarts

**B. Hash-based** (Collision risk)
```rust
fn string_to_usize(s: &str) -> usize {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish() as usize
}
```
- Problem: Collisions possible
- Problem: Can't reverse (usize -> String)

**C. Persistent Mapping** (Chroma's approach?)
```rust
// Store mapping in metadata
struct DocumentStore {
    id_map: HashMap<String, usize>,
    reverse_map: HashMap<usize, String>,
    next_id: usize,
}
```
- Requires persistence
- Adds complexity
- But most robust

### Question 3: How does metadata filtering actually work?

**Current Q CLI**:
```rust
// SemanticContext::search returns indices
let results = index.search(query_vector, limit, 100);

// Then we map to DataPoints
results.into_iter()
    .map(|(id, distance)| {
        let point = self.data_points[id].clone();
        SearchResult::new(point, distance)
    })
    .collect()
```

**To add filtering**:
```rust
// Option A: Filter after search (inefficient)
let results = index.search(query_vector, limit * 3, 100);
let filtered = results.into_iter()
    .filter(|(id, _)| matches_filter(&self.data_points[*id], filter))
    .take(limit)
    .collect();

// Option B: Pre-filter IDs (Chroma's way, more efficient)
let allowed_ids: Vec<usize> = self.data_points
    .iter()
    .enumerate()
    .filter(|(_, point)| matches_filter(point, filter))
    .map(|(idx, _)| idx)
    .collect();

// But hnsw_rs doesn't support allowed_ids parameter!
let results = index.search(query_vector, limit, 100);
```

**Problem**: `hnsw_rs` doesn't have `allowed_ids` parameter like `hnswlib`

---

## What I Need to Investigate

### Priority 1: HNSW Library Capabilities

1. **Check `hnsw_rs` v0.3.1 API**:
   - Does it support `delete()`?
   - Does it support `allowed_ids` filtering?
   - What's the performance of deletions?

2. **Compare with `hnswlib`**:
   - Would switching be worth it?
   - What's the migration cost?
   - Licensing implications?

### Priority 2: ID Mapping Strategy

1. **Test ID mapping approaches**:
   - Build prototype with each approach
   - Measure memory overhead
   - Test persistence

2. **Understand Q CLI's current ID scheme**:
   - How are IDs currently assigned?
   - Is there already a mapping somewhere?
   - Can we extend it?

### Priority 3: Filtering Implementation

1. **Benchmark filtering approaches**:
   - Post-search filtering cost
   - Pre-filtering cost
   - Memory usage

2. **Determine if we need to modify `hnsw_rs`**:
   - Can we contribute `allowed_ids` support?
   - Or fork and maintain?
   - Or work around it?

---

## Honest Assessment

### What I Can Implement Now ✅

1. **Basic document operations** (with caveats):
   - `add_document()` - Yes, with ID mapping
   - `get_document()` - Yes, with ID lookup
   - `delete_document()` - **Maybe**, depends on `hnsw_rs`

2. **Metadata storage**:
   - Store in DataPoint.payload - Yes
   - Retrieve by ID - Yes

### What I Cannot Implement Without More Info ❌

1. **Efficient deletion**:
   - Don't know if `hnsw_rs` supports it
   - Don't know performance characteristics
   - May need index rebuilds

2. **Efficient metadata filtering**:
   - `hnsw_rs` may not support pre-filtering
   - Post-filtering is inefficient
   - May need library modifications

3. **Robust ID mapping**:
   - Need to understand Q CLI's persistence model
   - Need to design mapping that survives restarts
   - Need to handle ID reuse after deletions

---

## Recommended Next Steps

### Step 1: Investigate `hnsw_rs` (1 day)

```rust
// Create test to check capabilities
#[test]
fn test_hnsw_rs_capabilities() {
    let index = Hnsw::new(/* ... */);
    
    // Test 1: Can we delete?
    index.insert(&vec![1.0, 2.0], 0);
    // Does this method exist?
    // index.delete(0)?;
    
    // Test 2: Can we filter?
    // Does search accept allowed_ids?
    // index.search(&query, k, allowed_ids)?;
}
```

### Step 2: Prototype ID Mapping (2 days)

```rust
// Test different approaches
mod id_mapping_tests {
    #[test]
    fn test_sequential_mapping() { /* ... */ }
    
    #[test]
    fn test_hash_mapping() { /* ... */ }
    
    #[test]
    fn test_persistent_mapping() { /* ... */ }
}
```

### Step 3: Benchmark Filtering (1 day)

```rust
// Measure post-filter performance
#[bench]
fn bench_post_filter(b: &mut Bencher) {
    // Search for k*3, then filter to k
}

// Compare with pre-filter
#[bench]
fn bench_pre_filter(b: &mut Bencher) {
    // Filter IDs first, then search
}
```

### Step 4: Make Go/No-Go Decision (1 day)

Based on findings:

**GO if**:
- `hnsw_rs` supports deletion (even if slow)
- We can implement ID mapping
- Post-filtering is acceptable for MVP

**NO-GO if**:
- `hnsw_rs` doesn't support deletion at all
- Would require major `hnsw_rs` modifications
- Performance is unacceptable

**ALTERNATIVE if NO-GO**:
- Switch to `hnswlib` (like Chroma)
- Use ChromaDB directly (original plan)
- Build simpler in-memory solution for MVP

---

## What I Should Have Said Earlier

**Honest answer to "Do you know enough to implement this?"**

**No, not yet.** I need to:

1. Verify `hnsw_rs` capabilities (deletion, filtering)
2. Understand Q CLI's ID and persistence model
3. Prototype and benchmark approaches
4. Make informed architectural decisions

**Timeline**: 3-5 days of investigation before confident implementation

**Risk**: Medium-High - Core dependencies may not support required features

Would you like me to start with Step 1 (investigating `hnsw_rs`) to get concrete answers?
