# Cortex Memory System - Single Binary Solution

## Hard Constraint

**Must package into Q CLI binary** - No external services, no Python, no ChromaDB server.

---

## Options Analysis

### ❌ Option 1: ChromaDB (Original Plan)
**Why it fails**: Requires Python runtime + ChromaDB server
- Not packageable in Rust binary
- External process management
- **REJECTED**

### ❌ Option 2: Chroma's `hnswlib` 
**Why it's problematic**: C++ library with system dependencies
- Requires C++ compiler at build time
- Platform-specific binaries
- Harder cross-compilation
- **RISKY** but possible

### ✅ Option 3: Pure Rust with `hnsw_rs`
**Why it works**: Already in Q CLI, pure Rust
- No external dependencies
- Single binary
- Cross-platform
- **PREFERRED** if we can make it work

### ✅ Option 4: Simplified In-Memory Solution
**Why it works**: Fallback if HNSW limitations are blockers
- Pure Rust
- Single binary
- Simpler but less scalable
- **BACKUP PLAN**

---

## Recommended Approach

### Phase 1: Investigate `hnsw_rs` (1 day)

**Test script**:
```rust
// Test if hnsw_rs supports what we need
use hnsw_rs::hnsw::Hnsw;

#[test]
fn test_hnsw_rs_capabilities() {
    let mut index = Hnsw::new(16, 100, 16, 100, hnsw_rs::dist::DistCosine);
    
    // Test 1: Basic operations
    let vec1 = vec![1.0, 2.0, 3.0];
    index.insert((&vec1, 0));
    
    // Test 2: Can we search?
    let results = index.search(&vec1, 5, 100);
    assert!(!results.is_empty());
    
    // Test 3: Does delete exist? (THIS IS THE KEY QUESTION)
    // Check API documentation or try:
    // index.delete(0)?;
    
    // Test 4: Can we get by ID?
    // Check if this exists:
    // let vec = index.get(0)?;
}
```

**Outcomes**:

**A. If `hnsw_rs` supports deletion**:
→ Use Option 3 (Pure Rust with workarounds)

**B. If `hnsw_rs` doesn't support deletion**:
→ Evaluate Option 2 (hnswlib) vs Option 4 (Simplified)

---

## Option 3 Details: Pure Rust Solution

### Architecture
```
cortex-memory (pure Rust crate)
├── Uses existing semantic-search-client
├── Adds ID mapping layer (String → usize)
├── Adds metadata filtering (post-search)
└── Soft deletes if hard deletes unavailable
```

### ID Mapping Strategy
```rust
// Store in semantic-search-client's DataPoint payload
pub struct DataPoint {
    pub id: usize,  // HNSW internal ID
    pub payload: HashMap<String, Value>,  // Contains "cortex_id": "uuid-string"
    pub vector: Vec<f32>,
}

// Mapping layer
pub struct CortexAdapter {
    client: AsyncSemanticSearchClient,
    id_counter: AtomicUsize,
    // Persist mapping in a special context
    id_map_context: String,
}

impl CortexAdapter {
    pub async fn add_document(&self, id: String, content: String, metadata: HashMap<String, Value>) -> Result<()> {
        let internal_id = self.id_counter.fetch_add(1, Ordering::SeqCst);
        
        let mut full_metadata = metadata;
        full_metadata.insert("cortex_id".to_string(), json!(id));
        full_metadata.insert("cortex_internal_id".to_string(), json!(internal_id));
        
        // Use existing semantic-search-client
        self.client.add_document_to_context(
            &self.context_name,
            internal_id,
            content,
            full_metadata,
        ).await
    }
    
    pub async fn get_document(&self, id: &str) -> Result<Option<Document>> {
        // Search by cortex_id in metadata
        let results = self.client.search_context(&self.context_name, id, 1).await?;
        
        results.into_iter()
            .find(|r| r.point.payload.get("cortex_id")
                .and_then(|v| v.as_str()) == Some(id))
            .map(|r| Document::from_search_result(r))
            .ok_or(Error::NotFound)
    }
    
    pub async fn delete_document(&self, id: &str) -> Result<bool> {
        // If hnsw_rs supports delete:
        // 1. Find internal_id from metadata
        // 2. Call index.delete(internal_id)
        
        // If NOT supported (soft delete):
        let doc = self.get_document(id).await?;
        let mut metadata = doc.metadata;
        metadata.insert("deleted".to_string(), json!(true));
        
        // Update metadata (or re-add with deleted flag)
        self.update_metadata(id, metadata).await
    }
    
    pub async fn search(&self, query: &str, limit: usize, filter: Option<Filter>) -> Result<Vec<Document>> {
        // Search with larger limit to account for filtering
        let results = self.client.search_context(
            &self.context_name,
            query,
            limit * 3,  // Over-fetch
        ).await?;
        
        // Post-filter
        let filtered: Vec<_> = results.into_iter()
            .filter(|r| {
                // Skip soft-deleted
                if r.point.payload.get("deleted").and_then(|v| v.as_bool()).unwrap_or(false) {
                    return false;
                }
                // Apply user filters
                if let Some(ref f) = filter {
                    matches_filter(&r.point.payload, f)
                } else {
                    true
                }
            })
            .take(limit)
            .map(Document::from_search_result)
            .collect();
        
        Ok(filtered)
    }
}
```

### Pros
- ✅ Single binary (pure Rust)
- ✅ Uses existing infrastructure
- ✅ No external dependencies
- ✅ Cross-platform

### Cons
- ⚠️ Post-filtering less efficient
- ⚠️ Soft deletes waste space (need compaction)
- ⚠️ ID mapping adds complexity

---

## Option 4 Details: Simplified In-Memory Solution

If `hnsw_rs` is too limited, build simpler solution:

```rust
// Simplified memory system without HNSW
pub struct SimpleMemorySystem {
    memories: Arc<RwLock<HashMap<String, MemoryNote>>>,
    embeddings: Arc<RwLock<HashMap<String, Vec<f32>>>>,
    embedding_client: EmbeddingClient,
}

impl SimpleMemorySystem {
    pub async fn add(&self, id: String, content: String, metadata: HashMap<String, Value>) -> Result<()> {
        let embedding = self.embedding_client.embed(&content).await?;
        
        let note = MemoryNote { id: id.clone(), content, metadata, created_at: Utc::now() };
        
        self.memories.write().await.insert(id.clone(), note);
        self.embeddings.write().await.insert(id, embedding);
        
        Ok(())
    }
    
    pub async fn search(&self, query: &str, limit: usize, filter: Option<Filter>) -> Result<Vec<SearchResult>> {
        let query_embedding = self.embedding_client.embed(query).await?;
        
        let embeddings = self.embeddings.read().await;
        let memories = self.memories.read().await;
        
        // Brute force cosine similarity
        let mut results: Vec<_> = embeddings.iter()
            .filter_map(|(id, emb)| {
                let memory = memories.get(id)?;
                
                // Apply filter
                if let Some(ref f) = filter {
                    if !matches_filter(&memory.metadata, f) {
                        return None;
                    }
                }
                
                let similarity = cosine_similarity(&query_embedding, emb);
                Some((id.clone(), similarity, memory.clone()))
            })
            .collect();
        
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        results.truncate(limit);
        
        Ok(results.into_iter().map(|(id, score, memory)| SearchResult {
            id,
            score,
            content: memory.content,
            metadata: memory.metadata,
        }).collect())
    }
    
    pub async fn delete(&self, id: &str) -> Result<bool> {
        let mut memories = self.memories.write().await;
        let mut embeddings = self.embeddings.write().await;
        
        embeddings.remove(id);
        Ok(memories.remove(id).is_some())
    }
}
```

### Pros
- ✅ Single binary (pure Rust)
- ✅ Simple, easy to understand
- ✅ Full control over operations
- ✅ True deletes, no soft delete issues
- ✅ Easy filtering

### Cons
- ⚠️ O(n) search (no HNSW optimization)
- ⚠️ Limited to memory size
- ⚠️ Slower for large datasets (>10k memories)

---

## Decision Matrix

| Criteria | Option 2 (hnswlib) | Option 3 (hnsw_rs) | Option 4 (Simple) |
|----------|-------------------|-------------------|-------------------|
| Single binary | ⚠️ Possible | ✅ Yes | ✅ Yes |
| Cross-platform | ⚠️ Complex | ✅ Easy | ✅ Easy |
| Performance | ✅ Excellent | ✅ Good | ⚠️ Limited |
| Deletions | ✅ Native | ❓ Unknown | ✅ Native |
| Filtering | ✅ Native | ⚠️ Post-filter | ✅ Native |
| Complexity | ⚠️ High | ⚠️ Medium | ✅ Low |
| Risk | ⚠️ Medium | ❓ Unknown | ✅ Low |

---

## Recommendation

### Immediate Action (Today)

**Test `hnsw_rs` capabilities**:
```bash
cd crates/semantic-search-client
cargo test --test test_hnsw_capabilities
```

Create test file to check:
1. Does delete exist?
2. What's the API?
3. Performance characteristics?

### Decision Tree

```
Does hnsw_rs support deletion?
├─ YES → Use Option 3 (Pure Rust with hnsw_rs)
│         Timeline: 2-3 weeks
│         Risk: Low
│
└─ NO → Choose between:
        ├─ Option 2 (hnswlib) - Better performance, harder build
        │   Timeline: 3-4 weeks
        │   Risk: Medium (build complexity)
        │
        └─ Option 4 (Simple) - Easier, limited scale
            Timeline: 1-2 weeks
            Risk: Low (but performance limited)
```

### My Recommendation

**Start with Option 4 (Simple) as MVP**, then:
- If performance is acceptable → Ship it
- If not → Investigate Option 2 (hnswlib) or enhance Option 3

**Rationale**:
- Guaranteed to work (no unknowns)
- Fast to implement (1-2 weeks)
- Single binary, pure Rust
- Can always optimize later
- Cortex use case (20 STM + moderate LTM) fits simple solution

---

## Next Steps

1. **Today**: Test `hnsw_rs` API (1 hour)
2. **Tomorrow**: Prototype Option 4 (1 day)
3. **This week**: Benchmark with realistic data (1 day)
4. **Decision**: Go with Option 4 or investigate alternatives

Would you like me to start with the `hnsw_rs` test or jump straight to prototyping Option 4?
