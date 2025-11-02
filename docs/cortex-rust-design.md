# Cortex Memory System - Rust Implementation Design

## Executive Summary

This document provides a complete design for porting Cortex from Python to Rust, ensuring feature parity while leveraging Q CLI's infrastructure and hnswlib.

**Key Principle**: Incremental implementation with continuous testing against Python Cortex behavior.

---

## Architecture Overview

### Python Cortex (Current)
```
AgenticMemorySystem
├── STM (OrderedDict + in-memory embeddings)
├── LTM (ChromaDB via HTTP)
├── Processors (Light/Deep/Retrieval)
├── CollectionManager (LLM-based categorization)
├── EmbeddingManager (OpenAI/SentenceTransformers)
└── LLMController (litellm)
```

### Rust Cortex (Target)
```
cortex-memory crate
├── memory_system.rs      # AgenticMemorySystem
│   ├── STM (LRU cache + embeddings)
│   └── LTM (hnswlib + document store)
├── processors/
│   ├── light.rs          # Fast keyword extraction
│   ├── deep.rs           # LLM-based metadata
│   └── retrieval.rs      # Context-aware reranking
├── collections.rs        # Smart collections (optional)
├── evolution.rs          # Memory linking (optional)
└── llm/
    └── client.rs         # LLM integration
```

---

## Phase-by-Phase Implementation Strategy

### Phase 0: Foundation (Week 1)
**Goal**: Core data structures and ID mapping

**Deliverables**:
1. MemoryNote struct
2. ID mapping layer (String ↔ usize)
3. Basic error types
4. Configuration structs

**Testing**: Unit tests for each component

---

### Phase 1: STM Implementation (Week 1-2)
**Goal**: Working short-term memory with LRU eviction

**Python Reference**:
```python
# cortex/stm.py
class ShortTermMemory:
    def __init__(self, capacity=20):
        self.cache = OrderedDict()  # LRU
        self.embeddings = {}
    
    def add(self, memory_id, content, metadata, embedding):
        if len(self.cache) >= self.capacity:
            oldest_id = next(iter(self.cache))
            del self.cache[oldest_id]
            del self.embeddings[oldest_id]
        self.cache[memory_id] = memory_data
        self.embeddings[memory_id] = embedding
    
    def search(self, query_embedding, limit):
        # Brute force cosine similarity
        results = []
        for id, emb in self.embeddings.items():
            similarity = cosine_similarity(query_embedding, emb)
            results.append((id, similarity))
        results.sort(reverse=True)
        return results[:limit]
```

**Rust Implementation**:
```rust
use lru::LruCache;
use std::collections::HashMap;

pub struct ShortTermMemory {
    cache: LruCache<String, MemoryNote>,
    embeddings: HashMap<String, Vec<f32>>,
    capacity: usize,
}

impl ShortTermMemory {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: LruCache::new(NonZeroUsize::new(capacity).unwrap()),
            embeddings: HashMap::new(),
            capacity,
        }
    }
    
    pub fn add(&mut self, id: String, note: MemoryNote, embedding: Vec<f32>) {
        // LRU automatically evicts oldest
        if let Some((evicted_id, _)) = self.cache.push(id.clone(), note) {
            self.embeddings.remove(&evicted_id);
        }
        self.embeddings.insert(id, embedding);
    }
    
    pub fn search(&self, query_embedding: &[f32], limit: usize) -> Vec<SearchResult> {
        let mut results: Vec<_> = self.embeddings
            .iter()
            .map(|(id, emb)| {
                let similarity = cosine_similarity(query_embedding, emb);
                (id.clone(), similarity)
            })
            .collect();
        
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        results.truncate(limit);
        
        results.into_iter()
            .filter_map(|(id, score)| {
                self.cache.peek(&id).map(|note| SearchResult {
                    id,
                    content: note.content.clone(),
                    score,
                    metadata: note.metadata.clone(),
                })
            })
            .collect()
    }
}
```

**Compatibility Testing**:
```rust
#[test]
fn test_stm_matches_python_behavior() {
    // Add 25 items to capacity-20 STM
    // Verify first 5 are evicted
    // Verify LRU order matches Python OrderedDict
}
```

---

### Phase 2: LTM Implementation (Week 2-3)
**Goal**: Working long-term memory with hnswlib

**Python Reference**:
```python
# cortex/ltm.py
class LongTermMemory:
    def __init__(self):
        self.collections = {}  # user/session -> ChromaRetriever
    
    def add(self, memory_id, content, metadata, user_id, session_id):
        collection = self._get_collection(user_id, session_id)
        collection.add_document(content, metadata, memory_id)
    
    def search(self, query, limit, where_filter):
        collection = self._get_collection(user_id, session_id)
        return collection.search(query, limit, where_filter)
```

**Rust Implementation**:
```rust
use crate::hnsw_wrapper::CortexHnswIndex;

pub struct LongTermMemory {
    // Per-user/session indices
    indices: HashMap<UserKey, CortexHnswIndex>,
    // Document storage
    documents: HashMap<String, Document>,
    // Embedding generator
    embedder: Arc<dyn Embedder>,
}

impl LongTermMemory {
    pub async fn add(
        &mut self,
        id: String,
        content: String,
        metadata: HashMap<String, Value>,
        user_key: UserKey,
    ) -> Result<()> {
        // Generate embedding
        let embedding = self.embedder.embed(&content).await?;
        
        // Get or create index for user/session
        let index = self.indices
            .entry(user_key)
            .or_insert_with(|| CortexHnswIndex::new(384, 10000).unwrap());
        
        // Add to HNSW
        index.add(id.clone(), &embedding)?;
        
        // Store document
        self.documents.insert(id, Document {
            content,
            metadata,
            embedding,
        });
        
        Ok(())
    }
    
    pub async fn search(
        &self,
        query: &str,
        limit: usize,
        filter: Option<&Filter>,
        user_key: &UserKey,
    ) -> Result<Vec<SearchResult>> {
        // Get index
        let index = self.indices.get(user_key)
            .ok_or(Error::NoMemories)?;
        
        // Generate query embedding
        let query_embedding = self.embedder.embed(query).await?;
        
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
        let results = index.search(&query_embedding, limit, allowed_ids.as_deref())?;
        
        // Build search results
        Ok(results.into_iter()
            .filter_map(|(id, distance)| {
                self.documents.get(&id).map(|doc| SearchResult {
                    id: id.clone(),
                    content: doc.content.clone(),
                    score: 1.0 - distance,
                    distance,
                    metadata: doc.metadata.clone(),
                })
            })
            .collect())
    }
}
```

**Compatibility Testing**:
```rust
#[test]
fn test_ltm_matches_python_behavior() {
    // Add same documents as Python test
    // Search with same query
    // Verify results match (within floating point tolerance)
    // Verify metadata filtering works identically
}
```

---

### Phase 3: Memory System Orchestrator (Week 3-4)
**Goal**: Integrate STM + LTM with background processing

**Python Reference**:
```python
# cortex/memory_system.py
class AgenticMemorySystem:
    def add_note(self, content, metadata, user_id, session_id):
        note = MemoryNote(content, **metadata)
        
        # Light processing for STM (immediate)
        stm_metadata = self.light_processor.process(content, metadata)
        embedding = self.embedding_manager.get_embedding(content)
        self.stm.add(note.id, content, stm_metadata, embedding)
        
        # Deep processing for LTM (background)
        if self.enable_background_processing:
            self._ltm_executor.submit(self._process_ltm, note, user_id, session_id)
        else:
            self._process_ltm(note, user_id, session_id)
        
        return note.id
    
    def search(self, query, limit, user_id, session_id):
        # Search STM
        stm_results = self.stm.search(query_embedding, limit)
        
        # Search LTM
        ltm_results = self.ltm.search(query, limit, where_filter)
        
        # Merge and rerank
        all_results = stm_results + ltm_results
        return self.retrieval_processor.process(all_results, context)[:limit]
```

**Rust Implementation**:
```rust
pub struct AgenticMemorySystem {
    stm: Arc<RwLock<ShortTermMemory>>,
    ltm: Arc<RwLock<LongTermMemory>>,
    light_processor: LightProcessor,
    deep_processor: Arc<DeepProcessor>,
    retrieval_processor: RetrievalProcessor,
    config: CortexConfig,
}

impl AgenticMemorySystem {
    pub async fn add_note(
        &self,
        content: String,
        metadata: HashMap<String, Value>,
        user_id: Option<String>,
        session_id: Option<String>,
    ) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let user_key = UserKey { user_id, session_id };
        
        // Light processing (fast)
        let stm_metadata = self.light_processor.process(&content, &metadata)?;
        let embedding = self.generate_embedding(&content).await?;
        
        // Add to STM immediately
        let note = MemoryNote::new(id.clone(), content.clone(), stm_metadata.clone());
        {
            let mut stm = self.stm.write().await;
            stm.add(id.clone(), note, embedding.clone());
        }
        
        // Deep processing for LTM (background if enabled)
        if self.config.enable_background_processing {
            let ltm = self.ltm.clone();
            let deep_processor = self.deep_processor.clone();
            let id_clone = id.clone();
            let content_clone = content.clone();
            let user_key_clone = user_key.clone();
            
            tokio::spawn(async move {
                if let Ok(ltm_metadata) = deep_processor.process(&content_clone, &stm_metadata).await {
                    let mut ltm = ltm.write().await;
                    let _ = ltm.add(id_clone, content_clone, ltm_metadata, user_key_clone).await;
                }
            });
        } else {
            // Synchronous
            let ltm_metadata = self.deep_processor.process(&content, &stm_metadata).await?;
            let mut ltm = self.ltm.write().await;
            ltm.add(id.clone(), content, ltm_metadata, user_key).await?;
        }
        
        Ok(id)
    }
    
    pub async fn search(
        &self,
        query: &str,
        limit: usize,
        user_id: Option<String>,
        session_id: Option<String>,
        context: Option<String>,
    ) -> Result<Vec<SearchResult>> {
        let user_key = UserKey { user_id, session_id };
        let query_embedding = self.generate_embedding(query).await?;
        
        // Search STM
        let stm_results = {
            let stm = self.stm.read().await;
            stm.search(&query_embedding, limit)
        };
        
        // Search LTM
        let ltm_results = {
            let ltm = self.ltm.read().await;
            ltm.search(query, limit, None, &user_key).await?
        };
        
        // Merge results
        let mut all_results = stm_results;
        all_results.extend(ltm_results);
        
        // Apply retrieval processing (reranking, temporal scoring)
        let processed = self.retrieval_processor.process(all_results, context.as_deref())?;
        
        Ok(processed.into_iter().take(limit).collect())
    }
}
```

**Compatibility Testing**:
```rust
#[tokio::test]
async fn test_memory_system_matches_python() {
    // Add notes in same order as Python
    // Search with same queries
    // Verify STM/LTM split matches
    // Verify result ordering matches
}
```

---

## Critical Compatibility Points

### 1. ID Generation
**Python**: Uses `uuid.uuid4().hex`
**Rust**: Use `Uuid::new_v4().to_string()`

**Test**: Verify format compatibility

### 2. Embedding Similarity
**Python**: `np.dot(a, b) / (np.linalg.norm(a) * np.linalg.norm(b))`
**Rust**: Same formula, verify floating point precision

**Test**: Compare results within 1e-6 tolerance

### 3. LRU Eviction Order
**Python**: OrderedDict evicts oldest first
**Rust**: LruCache evicts oldest first

**Test**: Add items in sequence, verify eviction order matches

### 4. Metadata Filtering
**Python**: ChromaDB's `where` clause
**Rust**: Pre-filter documents, then search

**Test**: Same filter expressions produce same results

### 5. Result Ordering
**Python**: Sorted by score descending
**Rust**: Same sorting

**Test**: Verify tie-breaking behavior matches

---

## Data Structure Mapping

### MemoryNote
```python
# Python
class MemoryNote:
    def __init__(self, content, **kwargs):
        self.id = kwargs.get('id', uuid.uuid4().hex)
        self.content = content
        self.keywords = kwargs.get('keywords', [])
        self.context = kwargs.get('context', 'General')
        self.tags = kwargs.get('tags', [])
        self.timestamp = kwargs.get('timestamp', datetime.now().isoformat())
        self.links = kwargs.get('links', {})
```

```rust
// Rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryNote {
    pub id: String,
    pub content: String,
    pub keywords: Vec<String>,
    pub context: String,
    pub tags: Vec<String>,
    pub timestamp: DateTime<Utc>,
    pub links: HashMap<String, Vec<MemoryLink>>,
    pub metadata: HashMap<String, Value>,
}

impl MemoryNote {
    pub fn new(id: String, content: String, metadata: HashMap<String, Value>) -> Self {
        Self {
            id,
            content,
            keywords: extract_keywords(&metadata),
            context: extract_context(&metadata),
            tags: extract_tags(&metadata),
            timestamp: Utc::now(),
            links: extract_links(&metadata),
            metadata,
        }
    }
}
```

### SearchResult
```python
# Python
{
    'id': str,
    'content': str,
    'score': float,
    'distance': float,
    'metadata': dict
}
```

```rust
// Rust
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub id: String,
    pub content: String,
    pub score: f32,
    pub distance: f32,
    pub metadata: HashMap<String, Value>,
    pub source: MemorySource,  // STM or LTM
}
```

---

## Migration Strategy

### Option A: Side-by-Side (Recommended)
```
Q CLI
├── Python Cortex (existing, via subprocess)
└── Rust Cortex (new, native)
    └── Feature flag to switch
```

**Pros**:
- Can compare outputs directly
- Gradual migration
- Fallback if issues

**Cons**:
- Temporary complexity
- Two implementations to maintain

### Option B: Direct Replacement
```
Q CLI
└── Rust Cortex (only)
```

**Pros**:
- Clean, single implementation
- No temporary complexity

**Cons**:
- Higher risk
- No fallback
- Harder to debug differences

**Recommendation**: Start with Option A, migrate to Option B after validation

---

## Testing Strategy

### Unit Tests (Per Component)
```rust
#[cfg(test)]
mod tests {
    // Test each component in isolation
    #[test]
    fn test_stm_add_and_search() { }
    
    #[test]
    fn test_ltm_add_and_search() { }
    
    #[test]
    fn test_id_mapping() { }
}
```

### Integration Tests (Cross-Component)
```rust
#[tokio::test]
async fn test_stm_to_ltm_flow() {
    // Add to STM
    // Verify background processing moves to LTM
    // Search both tiers
}
```

### Compatibility Tests (Python Parity)
```rust
#[tokio::test]
async fn test_matches_python_cortex() {
    // Load same test data as Python tests
    // Run same operations
    // Compare outputs
    // Assert within tolerance
}
```

### Performance Tests
```rust
#[tokio::test]
async fn test_performance_targets() {
    // Add 1000 memories
    // Search 100 times
    // Assert < 100ms per search
}
```

---

## Error Handling Strategy

### Python Cortex Error Handling
```python
try:
    result = operation()
except Exception as e:
    logger.error(f"Error: {e}")
    return default_value
```

### Rust Error Handling
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CortexError {
    #[error("Memory not found: {0}")]
    NotFound(String),
    
    #[error("Embedding error: {0}")]
    EmbeddingError(String),
    
    #[error("LLM error: {0}")]
    LlmError(String),
    
    #[error("Storage error: {0}")]
    StorageError(#[from] hnswlib::HnswError),
}

pub type Result<T> = std::result::Result<T, CortexError>;
```

**Principle**: Fail fast in Rust, but provide clear error messages matching Python behavior

---

## Configuration Compatibility

### Python Config
```python
# cortex/constants.py
DEFAULT_STM_CAPACITY = 20
DEFAULT_EMBEDDING_MODEL = "text-embedding-ada-002"
DEFAULT_LLM_MODEL = "gpt-4"
```

### Rust Config
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CortexConfig {
    pub stm_capacity: usize,
    pub embedding_model: String,
    pub llm_model: String,
    pub enable_smart_collections: bool,
    pub enable_background_processing: bool,
    pub temporal_weight: f32,
}

impl Default for CortexConfig {
    fn default() -> Self {
        Self {
            stm_capacity: 20,  // Match Python
            embedding_model: "text-embedding-ada-002".to_string(),
            llm_model: "gpt-4".to_string(),
            enable_smart_collections: true,
            enable_background_processing: true,
            temporal_weight: 0.3,
        }
    }
}
```

---

## Phased Feature Implementation

### MVP (Weeks 1-4)
- ✅ STM with LRU
- ✅ LTM with hnswlib
- ✅ Basic search (no filtering)
- ✅ ID mapping
- ✅ Light processor (keyword extraction)

### Phase 2 (Weeks 5-6)
- ✅ Metadata filtering
- ✅ Deep processor (LLM integration)
- ✅ Retrieval processor (reranking)
- ✅ Background processing

### Phase 3 (Weeks 7-8) - Optional
- ⚠️ Smart collections
- ⚠️ Memory evolution
- ⚠️ Temporal scoring

---

## Risk Mitigation

### Risk 1: Floating Point Differences
**Mitigation**: Use tolerance-based comparisons (1e-6)

### Risk 2: Async Behavior Differences
**Mitigation**: Extensive testing of background processing

### Risk 3: LLM API Differences
**Mitigation**: Abstract LLM client, test with same prompts

### Risk 4: Performance Regression
**Mitigation**: Benchmark against Python, target 2x faster

---

## Success Criteria

### Functional Parity
- ✅ All Python Cortex tests pass in Rust
- ✅ Same results for same inputs (within tolerance)
- ✅ Same behavior for edge cases

### Performance
- ✅ 2x faster than Python Cortex
- ✅ < 100ms search latency
- ✅ < 50ms STM operations

### Integration
- ✅ Works with Q CLI chat flow
- ✅ Single binary deployment
- ✅ No Python dependency

---

## Timeline

| Week | Phase | Deliverable |
|------|-------|-------------|
| 1 | Foundation + STM | Working STM with tests |
| 2 | LTM | Working LTM with hnswlib |
| 3 | Integration | STM + LTM working together |
| 4 | Processors | Light/Deep/Retrieval processors |
| 5-6 | Polish | Background processing, filtering |
| 7-8 | Optional | Collections, evolution |

**Total**: 4-8 weeks depending on scope

---

## Next Steps

1. **Review this design** - Validate approach
2. **Create test fixtures** - Port Python test data
3. **Implement Phase 0** - Foundation
4. **Continuous testing** - Compare with Python at each step

Ready to start implementation?
