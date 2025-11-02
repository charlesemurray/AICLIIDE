# Q CLI Embedding Research - Findings

## Summary

**Q CLI already has a complete embedding system we can use!** ✅

---

## What Q CLI Has

### 1. Embedding Infrastructure (`semantic-search-client` crate)

**Location**: `crates/semantic-search-client/src/embedding/`

**Components**:
- `TextEmbedderTrait` - Interface for embedding generation
- `CandleTextEmbedder` - Local ML model using Candle framework
- `MockTextEmbedder` - Placeholder for testing/BM25
- `EmbeddingType` enum - Fast (BM25) vs Best (ML model)

### 2. Model: all-MiniLM-L6-v2

**Specs**:
- **Dimensions**: 384 (matches our Cortex implementation!)
- **Model**: sentence-transformers/all-MiniLM-L6-v2
- **Framework**: Candle (Rust ML framework)
- **Size**: ~80MB (model weights downloaded on first use)
- **Speed**: Fast enough for real-time use
- **Quality**: Good semantic search quality

**Platform Support**:
- ✅ macOS: Full support
- ✅ Windows: Full support  
- ✅ Linux x86_64: Full support
- ⚠️ Linux ARM64: Falls back to BM25 (no ML model)

### 3. Usage Pattern

```rust
use semantic_search_client::embedding::{EmbeddingType, TextEmbedderTrait};
use semantic_search_client::client::embedder_factory::create_embedder;

// Create embedder
let embedder = create_embedder(EmbeddingType::Best)?;

// Generate embedding
let embedding: Vec<f32> = embedder.embed("some text")?;
// Returns 384-dimensional vector

// Batch embeddings
let embeddings: Vec<Vec<f32>> = embedder.embed_batch(&texts)?;
```

### 4. Current Usage in Q CLI

**Knowledge Store** (`crates/chat-cli/src/util/knowledge_store.rs`):
- Uses `AsyncSemanticSearchClient` 
- Embeds code files for semantic search
- Already integrated with Q CLI's agent system

---

## Decision for Cortex

### ✅ Recommendation: Use Q CLI's Existing Embedder

**Reasons**:
1. **Already integrated** - No new dependencies
2. **Same dimensions** - 384 matches our Cortex LTM/HNSW setup
3. **Proven** - Already used in production for knowledge search
4. **Maintained** - Part of Q CLI codebase
5. **Zero additional binary size** - Already included

**Implementation**:
```rust
// In cortex-memory/src/qcli_api.rs
use semantic_search_client::embedding::{EmbeddingType, TextEmbedderTrait};
use semantic_search_client::client::embedder_factory::create_embedder;

pub struct CortexMemory {
    manager: MemoryManager,
    embedder: Box<dyn TextEmbedderTrait>,
}

impl CortexMemory {
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let embedder = create_embedder(EmbeddingType::Best)?;
        let manager = MemoryManager::new(db_path, 384, 100)?; // 384 dimensions!
        Ok(Self { manager, embedder })
    }
    
    pub fn store_interaction(&mut self, user_msg: &str, assistant_msg: &str) -> Result<String> {
        let content = format!("User: {}\nAssistant: {}", user_msg, assistant_msg);
        let embedding = self.embedder.embed(&content)?;
        
        let note = MemoryNote::new(
            uuid::Uuid::new_v4().to_string(),
            content,
            HashMap::new(),
        );
        
        self.manager.add(note, embedding)?;
        Ok(note.id)
    }
    
    pub fn recall_context(&self, query: &str, limit: usize) -> Result<Vec<ContextItem>> {
        let query_embedding = self.embedder.embed(query)?;
        let results = self.manager.search(&query_embedding, limit);
        
        // Convert to ContextItem...
        Ok(results)
    }
}
```

---

## Answer to Question 1: Embedding Generation

**Selected Option**: **Use Q CLI's existing `CandleTextEmbedder`**

**Why**:
- ✅ Already in codebase (no new dependencies)
- ✅ 384 dimensions (perfect match for our HNSW setup)
- ✅ Good quality (all-MiniLM-L6-v2 is industry standard)
- ✅ Fast enough for real-time use
- ✅ Works offline (local model)
- ✅ Zero additional binary size

**Fallback for Linux ARM64**:
- Use `EmbeddingType::Fast` (BM25 keyword search)
- Still functional, just not semantic
- Can add ARM64 support later if needed

**No need for**:
- ❌ External LLM API calls
- ❌ New ML model integration
- ❌ Placeholder implementations
- ❌ Additional dependencies

---

## Implementation Steps

### 1. Add Dependency
```toml
# crates/cortex-memory/Cargo.toml
[dependencies]
semantic-search-client = { path = "../semantic-search-client" }
```

### 2. Create Wrapper
```rust
// crates/cortex-memory/src/embedder.rs
use semantic_search_client::embedding::{EmbeddingType, TextEmbedderTrait};
use semantic_search_client::client::embedder_factory::create_embedder;

pub struct CortexEmbedder {
    inner: Box<dyn TextEmbedderTrait>,
}

impl CortexEmbedder {
    pub fn new() -> Result<Self> {
        let inner = create_embedder(EmbeddingType::Best)?;
        Ok(Self { inner })
    }
    
    pub fn embed(&self, text: &str) -> Result<Vec<f32>> {
        self.inner.embed(text)
    }
}
```

### 3. Integrate with MemoryManager
```rust
// Update CortexMemory API to use embedder
pub struct CortexMemory {
    manager: MemoryManager,
    embedder: CortexEmbedder,
}
```

---

## Performance Expectations

**Embedding Generation**:
- Single text: ~10-50ms
- Batch (10 texts): ~50-200ms
- Acceptable for chat use case

**Memory Recall**:
- Embedding generation: ~10-50ms
- HNSW search: ~1-10ms
- Total: ~20-60ms (well under 100ms target)

---

## Next Steps

1. ✅ **Decision made**: Use Q CLI's existing embedder
2. Add `semantic-search-client` dependency to `cortex-memory`
3. Create `CortexEmbedder` wrapper
4. Update `CortexMemory` API to use embedder
5. Test end-to-end: store → embed → search → recall

---

## Conclusion

**We don't need to build or integrate a new embedding system!** Q CLI already has exactly what we need:
- ✅ Local ML model (all-MiniLM-L6-v2)
- ✅ 384 dimensions (matches our setup)
- ✅ Good quality semantic search
- ✅ Already integrated and tested
- ✅ Zero additional cost

This significantly simplifies Phase 1 implementation. We can move directly to integration without worrying about embeddings.
