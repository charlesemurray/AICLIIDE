# Cortex Memory System - Rust Implementation Plan

## Executive Summary

This document outlines the design and implementation plan for porting Cortex's advanced memory system to Rust, leveraging Q CLI's existing semantic search infrastructure. The implementation will provide a two-tier memory system (STM/LTM) with smart collections, memory evolution, and temporal awareness.

**Key Decision**: Use Q CLI's existing HNSW + BM25 hybrid search instead of ChromaDB, eliminating external dependencies while maintaining performance.

**Timeline**: 8-10 weeks for full implementation
**Effort**: Medium complexity - leverages 80% of existing Q CLI infrastructure

---

## Architecture Overview

### Current Cortex (Python)
```
AgenticMemorySystem
├── STM (OrderedDict + in-memory embeddings)
├── LTM (ChromaDB via HTTP)
├── Processors (Light/Deep/Retrieval)
├── CollectionManager (LLM-based categorization)
├── EmbeddingManager (OpenAI/SentenceTransformers)
└── LLMController (litellm)
```

### Target Rust Implementation
```
cortex-memory (new crate)
├── memory_system.rs      # AgenticMemorySystem orchestrator
├── stm.rs               # ShortTermMemory (LRU cache)
├── ltm.rs               # LongTermMemory (wraps semantic_search_client)
├── processors.rs        # Light/Deep/Retrieval processors
├── collections.rs       # CollectionManager
├── evolution.rs         # Memory evolution and linking
└── temporal.rs          # Temporal scoring and filtering

Integrates with existing Q CLI:
├── semantic-search-client (HNSW + BM25)
├── embedding (Candle-based local embeddings)
└── chat-cli (LLM integration)
```

---

## Core Components Design

### 1. Memory Tiers

#### 1.1 Short-Term Memory (STM)
**Purpose**: Fast, in-memory cache for recent memories with LRU eviction

**Rust Implementation**:
```rust
// crates/cortex-memory/src/stm.rs
use lru::LruCache;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ShortTermMemory {
    // Per-user/session memory stores
    stores: Arc<RwLock<HashMap<UserKey, MemoryStore>>>,
    capacity: usize,
}

struct MemoryStore {
    cache: LruCache<String, MemoryNote>,
    embeddings: HashMap<String, Vec<f32>>,
}

#[derive(Hash, Eq, PartialEq, Clone)]
struct UserKey {
    user_id: Option<String>,
    session_id: Option<String>,
}

impl ShortTermMemory {
    pub fn new(capacity: usize) -> Self {
        Self {
            stores: Arc::new(RwLock::new(HashMap::new())),
            capacity,
        }
    }
    
    pub async fn add(&self, key: UserKey, id: String, note: MemoryNote, embedding: Vec<f32>) {
        let mut stores = self.stores.write().await;
        let store = stores.entry(key).or_insert_with(|| MemoryStore {
            cache: LruCache::new(self.capacity.try_into().unwrap()),
            embeddings: HashMap::new(),
        });
        
        store.cache.put(id.clone(), note);
        store.embeddings.insert(id, embedding);
    }
    
    pub async fn search(&self, key: &UserKey, query_embedding: &[f32], limit: usize) -> Vec<SearchResult> {
        let stores = self.stores.read().await;
        let Some(store) = stores.get(key) else {
            return Vec::new();
        };
        
        let mut results: Vec<_> = store.embeddings
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
                store.cache.peek(&id).map(|note| SearchResult {
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

**Key Differences from Python**:
- Uses `lru` crate instead of OrderedDict
- Async RwLock for thread-safe concurrent access
- Per-user/session isolation maintained
- Zero-copy where possible

#### 1.2 Long-Term Memory (LTM)
**Purpose**: Persistent storage with vector search, wraps Q CLI's semantic_search_client

**Rust Implementation**:
```rust
// crates/cortex-memory/src/ltm.rs
use semantic_search_client::{AsyncSemanticSearchClient, SearchResult as SSResult};
use std::sync::Arc;

pub struct LongTermMemory {
    client: Arc<AsyncSemanticSearchClient>,
    context_prefix: String,
}

impl LongTermMemory {
    pub async fn new(base_dir: PathBuf) -> Result<Self> {
        let client = AsyncSemanticSearchClient::new(base_dir).await?;
        Ok(Self {
            client: Arc::new(client),
            context_prefix: "cortex_ltm".to_string(),
        })
    }
    
    fn get_context_name(&self, user_id: Option<&str>, session_id: Option<&str>) -> String {
        match (user_id, session_id) {
            (Some(u), Some(s)) => format!("{}_{}_{}", self.context_prefix, u, s),
            (Some(u), None) => format!("{}_{}", self.context_prefix, u),
            (None, Some(s)) => format!("{}_{}", self.context_prefix, s),
            (None, None) => self.context_prefix.clone(),
        }
    }
    
    pub async fn add(&self, key: &UserKey, id: String, content: String, metadata: HashMap<String, Value>) -> Result<()> {
        let context_name = self.get_context_name(key.user_id.as_deref(), key.session_id.as_deref());
        
        // Use semantic_search_client's add_document
        self.client.add_document(
            &context_name,
            id,
            content,
            metadata,
        ).await
    }
    
    pub async fn search(&self, key: &UserKey, query: &str, limit: usize, filter: Option<Filter>) -> Result<Vec<SearchResult>> {
        let context_name = self.get_context_name(key.user_id.as_deref(), key.session_id.as_deref());
        
        // Use semantic_search_client's hybrid search (HNSW + BM25)
        let results = self.client.search(&context_name, query, limit).await?;
        
        // Convert to our SearchResult format
        Ok(results.into_iter().map(|r| SearchResult {
            id: r.id,
            content: r.text().unwrap_or_default().to_string(),
            score: 1.0 - r.distance, // Convert distance to similarity
            metadata: r.metadata,
        }).collect())
    }
}
```

**Key Advantages**:
- No ChromaDB dependency - uses Q CLI's in-process HNSW
- Hybrid search (vector + BM25) out of the box
- Persistent storage via Q CLI's existing mechanisms
- Better performance (no HTTP overhead)

---

## Implementation Phases

### Phase 1: Core Memory System (Weeks 1-3)

#### Week 1: Foundation
- [ ] Create `cortex-memory` crate
- [ ] Implement `MemoryNote` data structure
- [ ] Implement `ShortTermMemory` with LRU cache
- [ ] Add unit tests for STM

#### Week 2: LTM Integration
- [ ] Implement `LongTermMemory` wrapper around semantic_search_client
- [ ] Add user/session isolation
- [ ] Implement metadata filtering
- [ ] Add unit tests for LTM

#### Week 3: Memory System Orchestrator
- [ ] Implement `AgenticMemorySystem` main struct
- [ ] Add `add_note()` with STM/LTM routing
- [ ] Implement hybrid search across STM + LTM
- [ ] Add background processing with tokio
- [ ] Integration tests

**Deliverable**: Basic two-tier memory system working

---

### Phase 2: Processors (Weeks 4-5)

#### Week 4: Light & Deep Processors
- [ ] Implement `LightProcessor` for STM (keyword extraction)
- [ ] Implement `DeepProcessor` for LTM (LLM-based metadata)
- [ ] Add LLM client integration (OpenAI/Anthropic)
- [ ] Add processor tests

#### Week 5: Retrieval Processor
- [ ] Implement `RetrievalProcessor` for context-aware reranking
- [ ] Add temporal scoring
- [ ] Implement result merging logic
- [ ] Add processor benchmarks

**Deliverable**: Full processing pipeline operational

---

### Phase 3: Advanced Features (Weeks 6-8)

#### Week 6: Smart Collections
- [ ] Implement `CollectionManager`
- [ ] Add LLM-based categorization
- [ ] Implement hierarchical collections
- [ ] Add collection threshold logic
- [ ] Collection tests

#### Week 7: Memory Evolution
- [ ] Implement relationship detection
- [ ] Add memory linking system
- [ ] Implement memory merging
- [ ] Add evolution tests

#### Week 8: Temporal Features
- [ ] Implement recency scoring
- [ ] Add date range filtering
- [ ] Implement temporal query detection
- [ ] Add temporal tests

**Deliverable**: Feature parity with Python Cortex

---

### Phase 4: Integration & Optimization (Weeks 9-10)

#### Week 9: Q CLI Integration
- [ ] Add cortex-memory to chat-cli
- [ ] Implement CLI commands (`q memory add`, `q memory search`)
- [ ] Add configuration options
- [ ] Integration tests with chat flow

#### Week 10: Performance & Polish
- [ ] Benchmark against Python Cortex
- [ ] Optimize hot paths
- [ ] Add comprehensive documentation
- [ ] Create migration guide

**Deliverable**: Production-ready Cortex in Rust

---

## Detailed Component Specifications

### Memory Note Structure
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryNote {
    pub id: String,
    pub content: String,
    pub metadata: HashMap<String, Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl MemoryNote {
    pub fn keywords(&self) -> Vec<String> {
        self.metadata.get("keywords")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default()
    }
    
    pub fn context(&self) -> String {
        self.metadata.get("context")
            .and_then(|v| v.as_str())
            .unwrap_or("General")
            .to_string()
    }
    
    pub fn tags(&self) -> Vec<String> {
        self.metadata.get("tags")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default()
    }
}
```

### Search Result Structure
```rust
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub id: String,
    pub content: String,
    pub score: f32,
    pub distance: f32,
    pub metadata: HashMap<String, Value>,
    pub source: MemorySource,
}

#[derive(Debug, Clone, Copy)]
pub enum MemorySource {
    ShortTerm,
    LongTerm,
}
```

---

## Dependencies

### New Dependencies for cortex-memory
```toml
[dependencies]
# Core
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.0", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }

# Memory management
lru = "0.12"

# Existing Q CLI crates
semantic-search-client = { path = "../semantic-search-client" }

# LLM integration
reqwest = { version = "0.11", features = ["json"] }
async-trait = "0.1"

# Logging
tracing = "0.1"
```

---

## Configuration

### Config Structure
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CortexConfig {
    pub stm_capacity: usize,
    pub enable_smart_collections: bool,
    pub enable_background_processing: bool,
    pub enable_memory_evolution: bool,
    pub temporal_weight: f32,
    pub collection_threshold: usize,
    pub llm_provider: LlmProvider,
    pub llm_model: String,
}

impl Default for CortexConfig {
    fn default() -> Self {
        Self {
            stm_capacity: 20,
            enable_smart_collections: true,
            enable_background_processing: true,
            enable_memory_evolution: true,
            temporal_weight: 0.3,
            collection_threshold: 10,
            llm_provider: LlmProvider::OpenAI,
            llm_model: "gpt-4".to_string(),
        }
    }
}
```

### User Configuration File
```toml
# ~/.q/cortex.toml
[memory]
stm_capacity = 20
enable_smart_collections = true
enable_background_processing = true
enable_memory_evolution = true
temporal_weight = 0.3

[llm]
provider = "openai"
model = "gpt-4"
api_key_env = "OPENAI_API_KEY"
```

---

## Testing Strategy

### Unit Tests
- STM: LRU eviction, search, user isolation
- LTM: Add, search, delete, filtering
- Processors: Keyword extraction, LLM metadata, reranking
- Collections: Categorization, threshold, hierarchy

### Integration Tests
- End-to-end memory flow (add → STM → LTM)
- Hybrid search (STM + LTM merging)
- Background processing
- Multi-user scenarios

### Performance Tests
- Benchmark against Python Cortex
- Memory usage profiling
- Search latency measurements
- Concurrent access stress tests

### Target Metrics
- Add note: < 50ms (STM), < 200ms (LTM background)
- Search: < 100ms (STM), < 500ms (hybrid)
- Memory usage: < 100MB for 10k memories
- Throughput: > 1000 ops/sec

---

## Migration from Python Cortex

### Data Migration
```rust
pub async fn migrate_from_chroma(chroma_uri: &str, ltm: &LongTermMemory) -> Result<()> {
    // Connect to ChromaDB
    let client = ChromaClient::new(chroma_uri)?;
    
    // For each collection
    for collection in client.list_collections().await? {
        // Get all documents
        let docs = client.get_all(&collection).await?;
        
        // Add to Q CLI's semantic search
        for doc in docs {
            ltm.add(
                &UserKey::default(),
                doc.id,
                doc.content,
                doc.metadata,
            ).await?;
        }
    }
    
    Ok(())
}
```

### API Compatibility Layer
```rust
// Provide Python-like API for easy migration
impl AgenticMemorySystem {
    pub async fn add_memory(&self, content: String, metadata: Option<HashMap<String, Value>>) -> Result<String> {
        self.add_note(content, metadata.unwrap_or_default(), None, None).await
    }
    
    pub async fn search_memories(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        self.search(query, limit, None, None, None).await
    }
}
```

---

## Performance Optimizations

### 1. Embedding Caching
```rust
use moka::future::Cache;

pub struct EmbeddingCache {
    cache: Cache<String, Vec<f32>>,
}

impl EmbeddingCache {
    pub fn new(max_capacity: u64) -> Self {
        Self {
            cache: Cache::builder()
                .max_capacity(max_capacity)
                .time_to_live(Duration::from_secs(3600))
                .build(),
        }
    }
}
```

### 2. Batch Processing
```rust
impl AgenticMemorySystem {
    pub async fn add_notes_batch(&self, notes: Vec<(String, HashMap<String, Value>)>) -> Result<Vec<String>> {
        // Process in parallel
        let futures: Vec<_> = notes.into_iter()
            .map(|(content, metadata)| self.add_note(content, metadata, None, None))
            .collect();
        
        futures::future::try_join_all(futures).await
    }
}
```

### 3. Lazy Loading
```rust
// Only load LTM when needed
pub struct LazyLtm {
    inner: OnceCell<Arc<LongTermMemory>>,
    base_dir: PathBuf,
}
```

---

## Security Considerations

### 1. User Isolation
- Strict separation of user/session data
- No cross-user memory leakage
- Validate user_id/session_id on all operations

### 2. Input Validation
```rust
fn validate_content(content: &str) -> Result<()> {
    if content.len() > MAX_CONTENT_SIZE {
        return Err(Error::ContentTooLarge);
    }
    if content.trim().is_empty() {
        return Err(Error::EmptyContent);
    }
    Ok(())
}
```

### 3. API Key Management
- Never log API keys
- Use environment variables
- Support key rotation

---

## Monitoring & Observability

### Metrics to Track
```rust
use prometheus::{Counter, Histogram, Registry};

pub struct CortexMetrics {
    pub notes_added: Counter,
    pub searches_performed: Counter,
    pub stm_hits: Counter,
    pub ltm_hits: Counter,
    pub search_latency: Histogram,
    pub evolution_operations: Counter,
}
```

### Logging
```rust
use tracing::{info, warn, error, debug};

#[instrument(skip(self))]
pub async fn add_note(&self, content: String) -> Result<String> {
    debug!("Adding note with {} chars", content.len());
    // ...
    info!("Note added successfully: {}", id);
    Ok(id)
}
```

---

## Next Steps

1. **Review this plan** with the team
2. **Set up cortex-memory crate** structure
3. **Start Phase 1** implementation
4. **Weekly progress reviews** to adjust timeline

---

## Appendix: Key Differences from Python

| Aspect | Python Cortex | Rust Implementation |
|--------|--------------|---------------------|
| Storage | ChromaDB (HTTP) | HNSW (in-process) |
| Concurrency | ThreadPoolExecutor | Tokio async |
| Memory | OrderedDict | LRU crate |
| Embeddings | OpenAI/SentenceTransformers | Candle (local) |
| Type Safety | Runtime | Compile-time |
| Performance | ~2-8s | <2s (estimated) |
| Dependencies | Python + ChromaDB server | Single Rust binary |
