# Cortex Database Requirements - Gap Analysis

## Executive Summary

Q CLI's `semantic-search-client` has **most** of what Cortex needs, but is missing some critical document-level operations. This document identifies the gaps and proposes solutions.

**Status**: 🟡 **70% Ready** - Core infrastructure exists, needs document-level API additions

---

## What Q CLI Has ✅

### 1. Vector Search (HNSW)
- ✅ High-performance HNSW index
- ✅ Cosine similarity search
- ✅ Persistent storage
- ✅ Async operations

### 2. BM25 Keyword Search
- ✅ Full BM25 implementation
- ✅ Hybrid search capability
- ✅ Fast keyword matching

### 3. Embeddings
- ✅ Local embedding generation (Candle)
- ✅ Multiple model support
- ✅ Caching

### 4. Context Management
- ✅ Multiple contexts (like collections)
- ✅ Persistent/volatile contexts
- ✅ Context isolation

### 5. Background Processing
- ✅ Async indexing
- ✅ Cancellation support
- ✅ Progress tracking

### 6. File Processing
- ✅ Directory indexing
- ✅ Pattern filtering
- ✅ Multiple file types

---

## What's Missing ❌

### 1. Document-Level Operations (CRITICAL)

**Cortex Needs:**
```python
# Python Cortex
ltm.add(memory_id, content, metadata)  # Add single document
ltm.get(memory_id)                      # Get by ID
ltm.delete(memory_id)                   # Delete by ID
ltm.update(memory_id, metadata)         # Update metadata
```

**Q CLI Currently Has:**
```rust
// Only context-level operations
client.add_context(request)  // Adds entire directory
client.search_context(id, query, limit)
client.remove_context_by_id(id)
```

**Gap**: No way to add/get/delete/update individual documents within a context.

**Impact**: 🔴 **BLOCKER** - Cannot implement Cortex memory operations without this.

---

### 2. Metadata Filtering (HIGH PRIORITY)

**Cortex Needs:**
```python
# Filter by metadata fields
ltm.search(query, where_filter={
    "user_id": {"$eq": "user123"},
    "category": {"$eq": "work"},
    "created_at": {"$gt": "2024-01-01"}
})
```

**Q CLI Currently Has:**
```rust
// No metadata filtering in search
client.search_context(context_id, query, limit)
// Returns all results, no filtering
```

**Gap**: Cannot filter search results by metadata fields.

**Impact**: 🟡 **HIGH** - Needed for user isolation and temporal filtering.

---

### 3. Direct Document Retrieval (MEDIUM PRIORITY)

**Cortex Needs:**
```python
# Get document by ID without search
memory = ltm.get(memory_id)
```

**Q CLI Currently Has:**
```rust
// Must search to find documents
// No direct ID lookup
```

**Gap**: No way to retrieve a specific document by ID.

**Impact**: 🟡 **MEDIUM** - Can work around with in-memory cache, but inefficient.

---

### 4. Metadata Updates (LOW PRIORITY)

**Cortex Needs:**
```python
# Update document metadata (for memory evolution)
ltm.update(memory_id, {"links": new_links})
```

**Q CLI Currently Has:**
```rust
// No update operation
// Must delete and re-add
```

**Gap**: Cannot update document metadata without re-indexing.

**Impact**: 🟢 **LOW** - Can delete and re-add, but less efficient.

---

## Detailed Gap Analysis

### Gap 1: Document-Level Add/Get/Delete

#### Current Architecture
```rust
// semantic-search-client/src/client/context/semantic_context.rs
pub struct SemanticContext {
    vector_index: VectorIndex,  // HNSW index
    data_points: Vec<DataPoint>, // All documents
}

// Only bulk operations exist
impl SemanticContext {
    pub fn add_items(&mut self, items: Vec<DataPoint>) { ... }
    pub fn search(&self, query_embedding: &[f32], k: usize) -> Vec<SearchResult> { ... }
}
```

#### What Needs to Be Added
```rust
// NEW: Document-level operations
impl SemanticContext {
    /// Add a single document with custom ID
    pub fn add_document(
        &mut self,
        id: String,
        content: String,
        metadata: HashMap<String, Value>,
    ) -> Result<()> {
        // Generate embedding
        // Create DataPoint with custom ID
        // Add to vector_index
        // Add to data_points
    }
    
    /// Get document by ID
    pub fn get_document(&self, id: &str) -> Option<&DataPoint> {
        // Lookup in data_points by ID
    }
    
    /// Delete document by ID
    pub fn delete_document(&mut self, id: &str) -> Result<bool> {
        // Remove from vector_index
        // Remove from data_points
    }
    
    /// Update document metadata
    pub fn update_document_metadata(
        &mut self,
        id: &str,
        metadata: HashMap<String, Value>,
    ) -> Result<()> {
        // Find document
        // Update metadata
        // Persist changes
    }
}
```

#### Implementation Complexity
- **Effort**: 2-3 days
- **Risk**: Low - straightforward additions
- **Dependencies**: None

---

### Gap 2: Metadata Filtering

#### Current Search Implementation
```rust
// semantic-search-client/src/client/context/semantic_context.rs
pub fn search(&self, query_embedding: &[f32], k: usize) -> Vec<SearchResult> {
    // HNSW search returns top-k by similarity
    // No filtering applied
    self.vector_index.search(query_embedding, k)
}
```

#### What Needs to Be Added
```rust
#[derive(Debug, Clone)]
pub struct SearchFilter {
    pub conditions: Vec<FilterCondition>,
}

#[derive(Debug, Clone)]
pub struct FilterCondition {
    pub field: String,
    pub operator: FilterOperator,
    pub value: Value,
}

#[derive(Debug, Clone)]
pub enum FilterOperator {
    Equals,
    NotEquals,
    GreaterThan,
    LessThan,
    Contains,
    In,
}

impl SemanticContext {
    pub fn search_with_filter(
        &self,
        query_embedding: &[f32],
        k: usize,
        filter: Option<SearchFilter>,
    ) -> Vec<SearchResult> {
        // Get candidates from HNSW (fetch more than k)
        let candidates = self.vector_index.search(query_embedding, k * 3);
        
        // Apply metadata filters
        let filtered = if let Some(filter) = filter {
            candidates.into_iter()
                .filter(|result| self.matches_filter(&result.point, &filter))
                .collect()
        } else {
            candidates
        };
        
        // Return top k after filtering
        filtered.into_iter().take(k).collect()
    }
    
    fn matches_filter(&self, point: &DataPoint, filter: &SearchFilter) -> bool {
        filter.conditions.iter().all(|condition| {
            if let Some(value) = point.payload.get(&condition.field) {
                self.evaluate_condition(value, condition)
            } else {
                false
            }
        })
    }
}
```

#### Implementation Complexity
- **Effort**: 3-4 days
- **Risk**: Medium - need to handle various data types
- **Dependencies**: None

---

### Gap 3: Direct Document Retrieval

#### Current Limitation
```rust
// No ID-based lookup
// DataPoints stored in Vec, no HashMap
pub struct SemanticContext {
    data_points: Vec<DataPoint>,  // Linear search required
}
```

#### What Needs to Be Added
```rust
use std::collections::HashMap;

pub struct SemanticContext {
    data_points: Vec<DataPoint>,
    id_index: HashMap<String, usize>,  // NEW: ID -> index mapping
}

impl SemanticContext {
    pub fn get_document_by_id(&self, id: &str) -> Option<&DataPoint> {
        self.id_index.get(id)
            .and_then(|&idx| self.data_points.get(idx))
    }
}
```

#### Implementation Complexity
- **Effort**: 1 day
- **Risk**: Low - simple HashMap addition
- **Dependencies**: None

---

### Gap 4: Metadata Updates

#### Current Limitation
```rust
// No update operation
// Must delete and re-add entire document
```

#### What Needs to Be Added
```rust
impl SemanticContext {
    pub fn update_document_metadata(
        &mut self,
        id: &str,
        metadata: HashMap<String, Value>,
    ) -> Result<()> {
        if let Some(&idx) = self.id_index.get(id) {
            if let Some(point) = self.data_points.get_mut(idx) {
                // Merge or replace metadata
                point.payload.extend(metadata);
                return Ok(());
            }
        }
        Err(Error::NotFound)
    }
}
```

#### Implementation Complexity
- **Effort**: 1 day
- **Risk**: Low
- **Dependencies**: Gap 3 (ID index)

---

## Proposed Solution: Extend semantic-search-client

### Phase 1: Core Document Operations (Week 1)
1. Add `id_index: HashMap<String, usize>` to `SemanticContext`
2. Implement `add_document()` for single document adds
3. Implement `get_document_by_id()` for direct retrieval
4. Implement `delete_document()` for single document deletion
5. Update `AsyncSemanticSearchClient` to expose these operations

### Phase 2: Metadata Filtering (Week 1-2)
1. Create `SearchFilter` and related types
2. Implement `search_with_filter()` in `SemanticContext`
3. Add filter support to `AsyncSemanticSearchClient`
4. Add tests for various filter conditions

### Phase 3: Metadata Updates (Week 2)
1. Implement `update_document_metadata()`
2. Add persistence for metadata changes
3. Expose via `AsyncSemanticSearchClient`

---

## Implementation Plan

### Step 1: Extend DataPoint Structure
```rust
// crates/semantic-search-client/src/types.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    pub id: String,  // Change from usize to String for custom IDs
    pub payload: HashMap<String, serde_json::Value>,
    pub vector: Vec<f32>,
}
```

### Step 2: Add Document Operations to SemanticContext
```rust
// crates/semantic-search-client/src/client/context/semantic_context.rs

impl SemanticContext {
    // Add ID index
    id_index: HashMap<String, usize>,
    
    // New methods
    pub fn add_document(...) -> Result<()>
    pub fn get_document_by_id(...) -> Option<&DataPoint>
    pub fn delete_document(...) -> Result<bool>
    pub fn update_document_metadata(...) -> Result<()>
    pub fn search_with_filter(...) -> Vec<SearchResult>
}
```

### Step 3: Expose via AsyncSemanticSearchClient
```rust
// crates/semantic-search-client/src/client/async_implementation.rs

impl AsyncSemanticSearchClient {
    pub async fn add_document(
        &self,
        context_id: &str,
        id: String,
        content: String,
        metadata: HashMap<String, Value>,
    ) -> Result<()> {
        self.context_manager
            .add_document(context_id, id, content, metadata, &*self.embedder)
            .await
    }
    
    pub async fn get_document(
        &self,
        context_id: &str,
        id: &str,
    ) -> Result<Option<DataPoint>> {
        self.context_manager
            .get_document(context_id, id)
            .await
    }
    
    pub async fn delete_document(
        &self,
        context_id: &str,
        id: &str,
    ) -> Result<bool> {
        self.context_manager
            .delete_document(context_id, id)
            .await
    }
    
    pub async fn search_with_filter(
        &self,
        context_id: &str,
        query: &str,
        limit: usize,
        filter: Option<SearchFilter>,
    ) -> Result<SearchResults> {
        self.context_manager
            .search_with_filter(context_id, query, limit, filter, &*self.embedder)
            .await
    }
}
```

---

## Alternative: Wrapper Layer

If modifying `semantic-search-client` is not desired, create a wrapper:

```rust
// crates/cortex-memory/src/storage_adapter.rs

pub struct CortexStorageAdapter {
    client: Arc<AsyncSemanticSearchClient>,
    // In-memory index for document IDs
    doc_index: Arc<RwLock<HashMap<String, DocumentMetadata>>>,
}

impl CortexStorageAdapter {
    pub async fn add_document(...) -> Result<()> {
        // Store metadata in doc_index
        // Use client.add_context() with single file
    }
    
    pub async fn get_document(...) -> Result<Option<Document>> {
        // Lookup in doc_index
        // Search client for content
    }
}
```

**Pros**: No changes to semantic-search-client
**Cons**: Less efficient, more complex, duplicates functionality

---

## Recommendation

**Extend semantic-search-client** with document-level operations.

**Rationale**:
1. These operations are generally useful, not just for Cortex
2. More efficient than wrapper approach
3. Better integration with existing infrastructure
4. Relatively low effort (5-7 days total)

**Timeline**:
- Week 1: Document operations + ID indexing
- Week 2: Metadata filtering + updates
- Week 3: Testing + documentation

**Risk**: Low - additive changes, no breaking modifications

---

## Testing Requirements

### Unit Tests
```rust
#[tokio::test]
async fn test_add_document() {
    let client = AsyncSemanticSearchClient::new_with_default_dir().await.unwrap();
    
    let mut metadata = HashMap::new();
    metadata.insert("user_id".to_string(), json!("user123"));
    
    client.add_document("context1", "doc1".to_string(), "content".to_string(), metadata).await.unwrap();
    
    let doc = client.get_document("context1", "doc1").await.unwrap();
    assert!(doc.is_some());
}

#[tokio::test]
async fn test_search_with_filter() {
    // Add documents with different metadata
    // Search with filter
    // Verify only matching documents returned
}
```

### Integration Tests
- Multi-user isolation
- Concurrent document operations
- Filter combinations
- Large-scale operations (1000+ documents)

---

## Summary

| Feature | Status | Effort | Priority |
|---------|--------|--------|----------|
| Document add/get/delete | ❌ Missing | 2-3 days | 🔴 Critical |
| Metadata filtering | ❌ Missing | 3-4 days | 🟡 High |
| Direct ID retrieval | ❌ Missing | 1 day | 🟡 Medium |
| Metadata updates | ❌ Missing | 1 day | 🟢 Low |

**Total Effort**: 7-9 days to fill all gaps

**Recommendation**: Proceed with extending semantic-search-client. The infrastructure is solid, just needs document-level API additions.
