# Cortex Integration Analysis for Q CLI

## Executive Summary

Cortex is a Python-based advanced memory system for AI agents. This document analyzes what would be required to integrate Cortex's capabilities into Q CLI (Rust-based).

**Key Finding**: Q CLI already has 80% of the infrastructure needed. We can either:
1. **Wrap Cortex** as a Python service (faster, less work)
2. **Port Cortex** to Rust using existing Q CLI components (more integrated, better performance)

---

## Cortex Architecture Overview

### Core Components

```
cortex/
├── memory_system.py      # Main orchestrator (AgenticMemorySystem)
├── stm.py               # Short-term memory (in-memory LRU cache)
├── ltm.py               # Long-term memory (ChromaDB wrapper)
├── processors.py        # Light/Deep/Retrieval processors
├── collection_manager.py # Smart collections (hierarchical organization)
├── embedding_manager.py  # Embedding generation (OpenAI or local)
├── memory_note.py       # Memory data structure
└── retrieval/
    └── retrievers.py    # ChromaDB interface
```

### Key Features

1. **Two-Tier Memory System**
   - STM: In-memory OrderedDict with LRU eviction (default: 20 items)
   - LTM: ChromaDB persistent storage with vector search

2. **Three Processing Pipelines**
   - **Light Processor**: Fast keyword extraction for STM
   - **Deep Processor**: LLM-based metadata extraction for LTM
   - **Retrieval Processor**: Context-aware reranking

3. **Smart Collections**
   - Hierarchical categorization (e.g., `work.programming.python`)
   - Auto-creates collections at threshold (10 memories)
   - Query enhancement per collection

4. **Memory Evolution**
   - Automatic relationship detection
   - Memory merging for complementary info
   - Bidirectional linking with typed relationships

5. **Temporal Awareness**
   - Recency scoring with configurable weights
   - Date range filtering at DB level
   - Auto-detection of temporal queries

---

## Q CLI Existing Infrastructure

### What Q CLI Already Has

| Feature | Q CLI Component | Status |
|---------|----------------|--------|
| Vector embeddings | `semantic_search_client` | ✅ Complete |
| Vector index | HNSW via `hnsw_rs` | ✅ Complete |
| BM25 search | `bm25` crate | ✅ Complete |
| Local embeddings | Candle (Rust ML) | ✅ Complete |
| Persistent storage | File-based serialization | ✅ Complete |
| Background processing | Tokio async | ✅ Complete |
| File chunking | Text chunker | ✅ Complete |
| Pattern filtering | Glob patterns | ✅ Complete |

### What Q CLI Lacks (Cortex Features)

| Feature | Cortex Implementation | Effort to Add |
|---------|----------------------|---------------|
| STM/LTM separation | Python OrderedDict + ChromaDB | Medium |
| Smart collections | LLM-based categorization | High |
| Memory evolution | LLM-based relationship detection | High |
| Temporal scoring | Recency weighting | Low |
| Light/Deep processing | Dual pipeline | Medium |
| LLM integration | OpenAI/Ollama via litellm | Medium |

---

## Integration Options

### Option 1: Python Service Wrapper (Recommended for MVP)

**Architecture:**
```
Q CLI (Rust)
    ↓ HTTP/gRPC
Cortex Service (Python)
    ↓
ChromaDB Server
```

**Pros:**
- Minimal changes to Q CLI
- Use Cortex as-is (battle-tested)
- Fast to implement (1-2 weeks)
- Easy to update Cortex independently

**Cons:**
- Python runtime dependency
- Network overhead for local calls
- More complex deployment

**Implementation Steps:**
1. Package Cortex as FastAPI service (already has API in `api/`)
2. Add Rust HTTP client to Q CLI
3. Create Q CLI wrapper functions
4. Handle lifecycle (start/stop service)

**Code Example:**
```rust
// In Q CLI
pub struct CortexClient {
    base_url: String,
    client: reqwest::Client,
}

impl CortexClient {
    pub async fn add_note(&self, content: &str, metadata: HashMap<String, Value>) -> Result<String> {
        let response = self.client
            .post(&format!("{}/memories", self.base_url))
            .json(&json!({
                "content": content,
                "metadata": metadata
            }))
            .send()
            .await?;
        
        Ok(response.json::<MemoryResponse>().await?.id)
    }
    
    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let response = self.client
            .get(&format!("{}/memories/search", self.base_url))
            .query(&[("query", query), ("limit", &limit.to_string())])
            .send()
            .await?;
        
        Ok(response.json().await?)
    }
}
```

---

### Option 2: Native Rust Port (Better Long-term)

**Architecture:**
```
Q CLI (Rust)
    ↓
Cortex Module (Rust)
    ↓
semantic_search_client (existing)
    ↓
HNSW + BM25 (existing)
```

**Pros:**
- No external dependencies
- Better performance (no network/serialization)
- Single binary deployment
- Type safety across the stack

**Cons:**
- Significant development effort (4-6 weeks)
- Need to port Python logic to Rust
- Maintain parity with Cortex updates

**Implementation Steps:**

#### 1. Add STM Layer (1 week)
```rust
// crates/cortex-memory/src/stm.rs
use std::collections::HashMap;
use lru::LruCache;

pub struct ShortTermMemory {
    cache: LruCache<String, MemoryNote>,
    embeddings: HashMap<String, Vec<f32>>,
}

impl ShortTermMemory {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: LruCache::new(capacity),
            embeddings: HashMap::new(),
        }
    }
    
    pub fn add(&mut self, id: String, note: MemoryNote, embedding: Vec<f32>) {
        self.cache.put(id.clone(), note);
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

#### 2. Add LTM Layer (1 week)
```rust
// crates/cortex-memory/src/ltm.rs
use semantic_search_client::AsyncSemanticSearchClient;

pub struct LongTermMemory {
    client: AsyncSemanticSearchClient,
    collection_name: String,
}

impl LongTermMemory {
    pub async fn new(collection_name: String) -> Result<Self> {
        let client = AsyncSemanticSearchClient::new_with_default_dir().await?;
        Ok(Self { client, collection_name })
    }
    
    pub async fn add(&self, id: String, content: String, metadata: HashMap<String, Value>) -> Result<()> {
        // Use existing semantic_search_client
        self.client.add_document(&self.collection_name, id, content, metadata).await
    }
    
    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        self.client.search(&self.collection_name, query, limit).await
    }
}
```

#### 3. Add Memory System Orchestrator (1 week)
```rust
// crates/cortex-memory/src/lib.rs
pub struct AgenticMemorySystem {
    stm: Arc<Mutex<ShortTermMemory>>,
    ltm: Arc<LongTermMemory>,
    light_processor: LightProcessor,
    deep_processor: DeepProcessor,
    collection_manager: Option<CollectionManager>,
    background_executor: Option<ThreadPool>,
}

impl AgenticMemorySystem {
    pub async fn add_note(&self, content: String, metadata: HashMap<String, Value>) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        
        // Light processing for STM (fast)
        let stm_metadata = self.light_processor.process(&content, &metadata)?;
        let embedding = self.generate_embedding(&content).await?;
        
        // Add to STM immediately
        {
            let mut stm = self.stm.lock().await;
            stm.add(id.clone(), MemoryNote::new(content.clone(), stm_metadata.clone()), embedding.clone());
        }
        
        // Deep processing for LTM (background)
        if let Some(executor) = &self.background_executor {
            let ltm = self.ltm.clone();
            let content = content.clone();
            let id = id.clone();
            
            executor.spawn(async move {
                let ltm_metadata = self.deep_processor.process(&content, &metadata).await?;
                ltm.add(id, content, ltm_metadata).await?;
                Ok::<_, Error>(())
            });
        }
        
        Ok(id)
    }
    
    pub async fn search(&self, query: &str, limit: usize, temporal_weight: f32) -> Result<Vec<SearchResult>> {
        // Hybrid search: STM + LTM
        let stm_results = {
            let stm = self.stm.lock().await;
            stm.search(&self.generate_embedding(query).await?, limit)
        };
        
        let ltm_results = self.ltm.search(query, limit).await?;
        
        // Merge with temporal weighting
        self.merge_results(stm_results, ltm_results, temporal_weight)
    }
}
```

#### 4. Add Smart Collections (2 weeks)
```rust
// crates/cortex-memory/src/collections.rs
pub struct CollectionManager {
    collections: HashMap<String, Collection>,
    category_counts: HashMap<String, usize>,
    llm_client: LlmClient,
}

impl CollectionManager {
    pub async fn update_category(&mut self, category: &str) -> Result<()> {
        let count = self.category_counts.entry(category.to_string()).or_insert(0);
        *count += 1;
        
        if *count >= COLLECTION_THRESHOLD && !self.collections.contains_key(category) {
            self.create_collection(category).await?;
        }
        
        Ok(())
    }
    
    async fn create_collection(&mut self, category: &str) -> Result<()> {
        // Use LLM to generate collection metadata
        let metadata = self.llm_client.generate_collection_metadata(category).await?;
        
        let collection = Collection {
            name: category.to_string(),
            metadata,
            query_helpers: HashMap::new(),
        };
        
        self.collections.insert(category.to_string(), collection);
        Ok(())
    }
}
```

#### 5. Add LLM Integration (1 week)
```rust
// crates/cortex-memory/src/llm.rs
use reqwest::Client;

pub struct LlmClient {
    client: Client,
    api_key: String,
    model: String,
}

impl LlmClient {
    pub async fn analyze_content(&self, content: &str) -> Result<ContentMetadata> {
        let response = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(&self.api_key)
            .json(&json!({
                "model": self.model,
                "messages": [{
                    "role": "system",
                    "content": "Extract keywords, context, and category from the following content."
                }, {
                    "role": "user",
                    "content": content
                }],
                "response_format": { "type": "json_object" }
            }))
            .send()
            .await?;
        
        let result: LlmResponse = response.json().await?;
        Ok(serde_json::from_str(&result.choices[0].message.content)?)
    }
}
```

---

## Dependencies Required

### For Python Service Wrapper
```toml
# Add to Cargo.toml
[dependencies]
reqwest = { version = "0.11", features = ["json"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["process"] }
```

### For Native Rust Port
```toml
# New crate: crates/cortex-memory/Cargo.toml
[dependencies]
semantic_search_client = { path = "../semantic-search-client" }
lru = "0.12"
uuid = { version = "1.0", features = ["v4"] }
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
reqwest = { version = "0.11", features = ["json"] }
chrono = "0.4"
```

---

## Comparison: Cortex vs Q CLI Semantic Search

| Feature | Cortex | Q CLI semantic_search_client |
|---------|--------|------------------------------|
| **Storage** | ChromaDB (HTTP) | HNSW (in-process) |
| **Embeddings** | OpenAI or SentenceTransformers | Candle (local) |
| **Search** | Vector only | Vector + BM25 hybrid |
| **Memory Tiers** | STM + LTM | Single tier |
| **Collections** | Smart (LLM-based) | Manual contexts |
| **Evolution** | Automatic linking | None |
| **Temporal** | Recency scoring | Timestamp only |
| **Language** | Python | Rust |
| **Performance** | ~2-8s with collections | <2s |

---

## Recommended Implementation Plan

### Phase 1: MVP with Python Service (2 weeks)
1. Package Cortex as standalone service
2. Add HTTP client to Q CLI
3. Integrate with existing chat flow
4. Test with small codebase

### Phase 2: Enhanced Integration (4 weeks)
1. Port STM/LTM to Rust
2. Integrate with existing semantic_search_client
3. Add temporal scoring
4. Benchmark performance

### Phase 3: Advanced Features (6 weeks)
1. Port smart collections
2. Add memory evolution
3. Implement LLM-based metadata extraction
4. Full feature parity with Cortex

---

## Key Differences to Handle

### 1. Storage Backend
- **Cortex**: Requires ChromaDB server (separate process)
- **Q CLI**: Uses in-process HNSW (no external dependencies)
- **Solution**: Either run ChromaDB or port to use Q CLI's vector index

### 2. Embedding Generation
- **Cortex**: OpenAI API or SentenceTransformers (Python)
- **Q CLI**: Candle (Rust, local)
- **Solution**: Use Q CLI's existing embedding system

### 3. LLM Integration
- **Cortex**: Uses litellm for multi-provider support
- **Q CLI**: Direct API calls to Amazon Q
- **Solution**: Add OpenAI client for metadata extraction

### 4. Async Model
- **Cortex**: ThreadPoolExecutor (Python threads)
- **Q CLI**: Tokio (Rust async)
- **Solution**: Use tokio::spawn for background tasks

---

## Performance Considerations

### Cortex Benchmarks (from evaluation)
- Top-K 20: ~4,000 tokens, 0.706 LLM score, ~2-8s latency
- Top-K 35: ~7,000 tokens, 0.731 LLM score
- With smart collections: +2-6s overhead

### Expected Q CLI Performance
- Native Rust: 2-5x faster than Python
- In-process HNSW: No network overhead
- Candle embeddings: ~100ms vs ~200ms for API calls
- **Estimated**: <2s for most queries, <5s with collections

---

## Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_stm_lru_eviction() {
        let mut stm = ShortTermMemory::new(2);
        stm.add("1".into(), note1, emb1);
        stm.add("2".into(), note2, emb2);
        stm.add("3".into(), note3, emb3); // Should evict "1"
        
        assert!(stm.get("1").is_none());
        assert!(stm.get("2").is_some());
        assert!(stm.get("3").is_some());
    }
    
    #[tokio::test]
    async fn test_hybrid_search() {
        let system = AgenticMemorySystem::new().await.unwrap();
        system.add_note("Recent memory".into(), HashMap::new()).await.unwrap();
        
        let results = system.search("memory", 10, 0.5).await.unwrap();
        assert!(!results.is_empty());
    }
}
```

### Integration Tests
- Test with LoCoMo10 dataset (same as Cortex evaluation)
- Compare accuracy vs Cortex baseline
- Measure latency improvements

---

## Migration Path for Existing Q CLI Users

### Backward Compatibility
```rust
// Existing code continues to work
let client = AsyncSemanticSearchClient::new_with_default_dir().await?;
client.add_context(request).await?;

// New Cortex-style API available
let memory = AgenticMemorySystem::new().await?;
memory.add_note(content, metadata).await?;
```

### Configuration
```toml
# ~/.q/config.toml
[memory]
enable_cortex = true
stm_capacity = 20
enable_smart_collections = true
enable_background_processing = true
temporal_weight = 0.3
```

---

## Conclusion

**Recommendation**: Start with **Option 1 (Python Service Wrapper)** for quick validation, then migrate to **Option 2 (Native Rust Port)** for production.

**Rationale**:
1. Q CLI already has 80% of needed infrastructure
2. Python wrapper allows fast experimentation
3. Native port provides better performance and integration
4. Phased approach reduces risk

**Timeline**:
- MVP (Python wrapper): 2 weeks
- Native port: 6 weeks
- Full feature parity: 12 weeks

**Next Steps**:
1. Set up Cortex service locally
2. Create proof-of-concept integration
3. Benchmark against existing semantic search
4. Decide on final architecture based on results
