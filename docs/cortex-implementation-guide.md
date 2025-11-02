# Cortex Memory System - Implementation Guide

This guide provides detailed code examples and implementation patterns for building the Cortex memory system in Rust.

## Table of Contents
1. [Project Structure](#project-structure)
2. [Core Implementation](#core-implementation)
3. [Processor Implementation](#processor-implementation)
4. [Advanced Features](#advanced-features)
5. [CLI Integration](#cli-integration)

---

## Project Structure

```
crates/cortex-memory/
├── Cargo.toml
├── src/
│   ├── lib.rs                 # Public API
│   ├── memory_system.rs       # AgenticMemorySystem
│   ├── stm.rs                 # ShortTermMemory
│   ├── ltm.rs                 # LongTermMemory
│   ├── memory_note.rs         # MemoryNote data structure
│   ├── processors/
│   │   ├── mod.rs
│   │   ├── light.rs           # LightProcessor
│   │   ├── deep.rs            # DeepProcessor
│   │   └── retrieval.rs       # RetrievalProcessor
│   ├── collections.rs         # CollectionManager
│   ├── evolution.rs           # Memory evolution
│   ├── temporal.rs            # Temporal scoring
│   ├── llm/
│   │   ├── mod.rs
│   │   ├── client.rs          # LLM client abstraction
│   │   └── providers.rs       # OpenAI, Anthropic, etc.
│   ├── config.rs              # Configuration
│   ├── error.rs               # Error types
│   └── types.rs               # Common types
├── tests/
│   ├── integration_tests.rs
│   ├── stm_tests.rs
│   └── ltm_tests.rs
└── examples/
    ├── basic_usage.rs
    └── advanced_features.rs
```

---

## Core Implementation

### 1. Memory Note (memory_note.rs)

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    
    pub fn category(&self) -> Option<String> {
        self.metadata
            .get("category")
            .and_then(|v| v.as_str())
            .map(String::from)
    }
    
    pub fn links(&self) -> HashMap<String, Vec<MemoryLink>> {
        self.metadata
            .get("links")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryLink {
    pub target_id: String,
    pub relationship_type: String,
    pub strength: f32,
    pub reason: Option<String>,
}
```

### 2. Short-Term Memory (stm.rs)

```rust
use crate::memory_note::MemoryNote;
use crate::types::{SearchResult, UserKey};
use crate::error::Result;
use lru::LruCache;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ShortTermMemory {
    stores: Arc<RwLock<HashMap<UserKey, MemoryStore>>>,
    capacity: usize,
}

struct MemoryStore {
    cache: LruCache<String, MemoryNote>,
    embeddings: HashMap<String, Vec<f32>>,
}

impl ShortTermMemory {
    pub fn new(capacity: usize) -> Self {
        Self {
            stores: Arc::new(RwLock::new(HashMap::new())),
            capacity,
        }
    }
    
    pub async fn add(
        &self,
        key: UserKey,
        id: String,
        note: MemoryNote,
        embedding: Vec<f32>,
    ) -> Result<()> {
        let mut stores = self.stores.write().await;
        let store = stores.entry(key).or_insert_with(|| MemoryStore {
            cache: LruCache::new(NonZeroUsize::new(self.capacity).unwrap()),
            embeddings: HashMap::new(),
        });
        
        // LRU will automatically evict oldest if at capacity
        if let Some((evicted_id, _)) = store.cache.push(id.clone(), note) {
            store.embeddings.remove(&evicted_id);
        }
        store.embeddings.insert(id, embedding);
        
        Ok(())
    }
    
    pub async fn get(&self, key: &UserKey, id: &str) -> Option<MemoryNote> {
        let mut stores = self.stores.write().await;
        stores.get_mut(key)?.cache.get(id).cloned()
    }
    
    pub async fn search(
        &self,
        key: &UserKey,
        query_embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        let stores = self.stores.read().await;
        let Some(store) = stores.get(key) else {
            return Ok(Vec::new());
        };
        
        let mut results: Vec<_> = store
            .embeddings
            .iter()
            .map(|(id, emb)| {
                let similarity = cosine_similarity(query_embedding, emb);
                (id.clone(), similarity)
            })
            .collect();
        
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);
        
        let search_results = results
            .into_iter()
            .filter_map(|(id, score)| {
                store.cache.peek(&id).map(|note| SearchResult {
                    id: id.clone(),
                    content: note.content.clone(),
                    score,
                    distance: 1.0 - score,
                    metadata: note.metadata.clone(),
                    source: crate::types::MemorySource::ShortTerm,
                })
            })
            .collect();
        
        Ok(search_results)
    }
    
    pub async fn delete(&self, key: &UserKey, id: &str) -> Result<bool> {
        let mut stores = self.stores.write().await;
        if let Some(store) = stores.get_mut(key) {
            store.embeddings.remove(id);
            Ok(store.cache.pop(id).is_some())
        } else {
            Ok(false)
        }
    }
    
    pub async fn clear(&self, key: Option<&UserKey>) -> Result<()> {
        let mut stores = self.stores.write().await;
        if let Some(key) = key {
            stores.remove(key);
        } else {
            stores.clear();
        }
        Ok(())
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}
```

### 3. Long-Term Memory (ltm.rs)

```rust
use crate::error::{Result, CortexError};
use crate::types::{SearchResult, UserKey, MemorySource};
use semantic_search_client::AsyncSemanticSearchClient;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use serde_json::Value;

pub struct LongTermMemory {
    client: Arc<AsyncSemanticSearchClient>,
    context_prefix: String,
}

impl LongTermMemory {
    pub async fn new(base_dir: PathBuf, context_prefix: String) -> Result<Self> {
        let client = AsyncSemanticSearchClient::new(base_dir)
            .await
            .map_err(|e| CortexError::StorageError(e.to_string()))?;
        
        Ok(Self {
            client: Arc::new(client),
            context_prefix,
        })
    }
    
    fn get_context_name(&self, key: &UserKey) -> String {
        match (&key.user_id, &key.session_id) {
            (Some(u), Some(s)) => format!("{}_{}_{}", self.context_prefix, u, s),
            (Some(u), None) => format!("{}_{}", self.context_prefix, u),
            (None, Some(s)) => format!("{}_{}", self.context_prefix, s),
            (None, None) => self.context_prefix.clone(),
        }
    }
    
    pub async fn add(
        &self,
        key: &UserKey,
        id: String,
        content: String,
        metadata: HashMap<String, Value>,
    ) -> Result<()> {
        let context_name = self.get_context_name(key);
        
        // Ensure context exists
        self.ensure_context(&context_name).await?;
        
        // Add document to semantic search
        self.client
            .add_document(&context_name, id, content, metadata)
            .await
            .map_err(|e| CortexError::StorageError(e.to_string()))?;
        
        Ok(())
    }
    
    pub async fn get(&self, key: &UserKey, id: &str) -> Result<Option<SearchResult>> {
        let context_name = self.get_context_name(key);
        
        // Get document by ID
        let result = self.client
            .get_document(&context_name, id)
            .await
            .map_err(|e| CortexError::StorageError(e.to_string()))?;
        
        Ok(result.map(|doc| SearchResult {
            id: id.to_string(),
            content: doc.text().unwrap_or_default().to_string(),
            score: 1.0,
            distance: 0.0,
            metadata: doc.metadata,
            source: MemorySource::LongTerm,
        }))
    }
    
    pub async fn search(
        &self,
        key: &UserKey,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        let context_name = self.get_context_name(key);
        
        // Use semantic_search_client's hybrid search
        let results = self.client
            .search(&context_name, query, limit)
            .await
            .map_err(|e| CortexError::StorageError(e.to_string()))?;
        
        // Convert to our SearchResult format
        Ok(results
            .into_iter()
            .map(|r| SearchResult {
                id: r.id,
                content: r.text().unwrap_or_default().to_string(),
                score: 1.0 - r.distance, // Convert distance to similarity
                distance: r.distance,
                metadata: r.metadata,
                source: MemorySource::LongTerm,
            })
            .collect())
    }
    
    pub async fn delete(&self, key: &UserKey, id: &str) -> Result<bool> {
        let context_name = self.get_context_name(key);
        
        self.client
            .delete_document(&context_name, id)
            .await
            .map_err(|e| CortexError::StorageError(e.to_string()))?;
        
        Ok(true)
    }
    
    pub async fn clear(&self, key: Option<&UserKey>) -> Result<()> {
        if let Some(key) = key {
            let context_name = self.get_context_name(key);
            self.client
                .delete_context(&context_name)
                .await
                .map_err(|e| CortexError::StorageError(e.to_string()))?;
        } else {
            // Clear all cortex contexts
            let contexts = self.client
                .list_contexts()
                .await
                .map_err(|e| CortexError::StorageError(e.to_string()))?;
            
            for context in contexts {
                if context.starts_with(&self.context_prefix) {
                    self.client
                        .delete_context(&context)
                        .await
                        .map_err(|e| CortexError::StorageError(e.to_string()))?;
                }
            }
        }
        
        Ok(())
    }
    
    async fn ensure_context(&self, context_name: &str) -> Result<()> {
        // Check if context exists, create if not
        let contexts = self.client
            .list_contexts()
            .await
            .map_err(|e| CortexError::StorageError(e.to_string()))?;
        
        if !contexts.contains(&context_name.to_string()) {
            self.client
                .create_context(context_name, "Cortex memory context")
                .await
                .map_err(|e| CortexError::StorageError(e.to_string()))?;
        }
        
        Ok(())
    }
}
```

### 4. Agentic Memory System (memory_system.rs)

```rust
use crate::config::CortexConfig;
use crate::error::Result;
use crate::ltm::LongTermMemory;
use crate::memory_note::MemoryNote;
use crate::processors::{DeepProcessor, LightProcessor, RetrievalProcessor};
use crate::stm::ShortTermMemory;
use crate::types::{SearchResult, UserKey};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use serde_json::Value;

pub struct AgenticMemorySystem {
    stm: Arc<ShortTermMemory>,
    ltm: Arc<LongTermMemory>,
    light_processor: Arc<LightProcessor>,
    deep_processor: Arc<DeepProcessor>,
    retrieval_processor: Arc<RetrievalProcessor>,
    config: Arc<RwLock<CortexConfig>>,
}

impl AgenticMemorySystem {
    pub async fn new(base_dir: PathBuf, config: CortexConfig) -> Result<Self> {
        let stm = Arc::new(ShortTermMemory::new(config.stm_capacity));
        let ltm = Arc::new(LongTermMemory::new(base_dir, "cortex_ltm".to_string()).await?);
        
        let light_processor = Arc::new(LightProcessor::new());
        let deep_processor = Arc::new(DeepProcessor::new(config.clone()));
        let retrieval_processor = Arc::new(RetrievalProcessor::new(config.temporal_weight));
        
        Ok(Self {
            stm,
            ltm,
            light_processor,
            deep_processor,
            retrieval_processor,
            config: Arc::new(RwLock::new(config)),
        })
    }
    
    pub async fn add_note(
        &self,
        content: String,
        metadata: HashMap<String, Value>,
        user_id: Option<String>,
        session_id: Option<String>,
    ) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let key = UserKey { user_id, session_id };
        
        // Light processing for STM (fast)
        let stm_metadata = self.light_processor.process(&content, metadata.clone()).await?;
        let embedding = self.generate_embedding(&content).await?;
        
        // Create memory note
        let note = MemoryNote::new(id.clone(), content.clone(), stm_metadata.clone());
        
        // Add to STM immediately
        self.stm.add(key.clone(), id.clone(), note, embedding).await?;
        
        // Deep processing for LTM (background if enabled)
        let config = self.config.read().await;
        if config.enable_background_processing {
            let ltm = self.ltm.clone();
            let deep_processor = self.deep_processor.clone();
            let id_clone = id.clone();
            let content_clone = content.clone();
            let key_clone = key.clone();
            
            tokio::spawn(async move {
                if let Ok(ltm_metadata) = deep_processor.process(&content_clone, stm_metadata).await {
                    let _ = ltm.add(&key_clone, id_clone, content_clone, ltm_metadata).await;
                }
            });
        } else {
            let ltm_metadata = self.deep_processor.process(&content, stm_metadata).await?;
            self.ltm.add(&key, id.clone(), content, ltm_metadata).await?;
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
        let key = UserKey { user_id, session_id };
        let query_embedding = self.generate_embedding(query).await?;
        
        // Search STM
        let stm_results = self.stm.search(&key, &query_embedding, limit).await?;
        
        // Search LTM
        let ltm_results = self.ltm.search(&key, query, limit).await?;
        
        // Merge results
        let mut all_results = stm_results;
        all_results.extend(ltm_results);
        
        // Apply retrieval processing (reranking, temporal scoring)
        let processed_results = self.retrieval_processor
            .process(all_results, context.as_deref())
            .await?;
        
        // Return top results
        Ok(processed_results.into_iter().take(limit).collect())
    }
    
    pub async fn get(
        &self,
        id: &str,
        user_id: Option<String>,
        session_id: Option<String>,
    ) -> Result<Option<MemoryNote>> {
        let key = UserKey { user_id, session_id };
        
        // Try STM first
        if let Some(note) = self.stm.get(&key, id).await {
            return Ok(Some(note));
        }
        
        // Try LTM
        if let Some(result) = self.ltm.get(&key, id).await? {
            let note = MemoryNote::new(result.id, result.content, result.metadata);
            return Ok(Some(note));
        }
        
        Ok(None)
    }
    
    pub async fn delete(
        &self,
        id: &str,
        user_id: Option<String>,
        session_id: Option<String>,
    ) -> Result<bool> {
        let key = UserKey { user_id, session_id };
        
        // Delete from both STM and LTM
        let stm_deleted = self.stm.delete(&key, id).await?;
        let ltm_deleted = self.ltm.delete(&key, id).await?;
        
        Ok(stm_deleted || ltm_deleted)
    }
    
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        // Use Q CLI's existing embedding system
        // This would integrate with semantic_search_client's embedding
        todo!("Integrate with Q CLI embedding system")
    }
}
```

---

## Common Types (types.rs)

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use serde_json::Value;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct UserKey {
    pub user_id: Option<String>,
    pub session_id: Option<String>,
}

impl Default for UserKey {
    fn default() -> Self {
        Self {
            user_id: None,
            session_id: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub id: String,
    pub content: String,
    pub score: f32,
    pub distance: f32,
    pub metadata: HashMap<String, Value>,
    pub source: MemorySource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemorySource {
    ShortTerm,
    LongTerm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Filter {
    pub field: String,
    pub operator: FilterOperator,
    pub value: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterOperator {
    Equals,
    Contains,
    GreaterThan,
    LessThan,
}
```

---

## Error Handling (error.rs)

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CortexError {
    #[error("Storage error: {0}")]
    StorageError(String),
    
    #[error("Embedding error: {0}")]
    EmbeddingError(String),
    
    #[error("LLM error: {0}")]
    LlmError(String),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

pub type Result<T> = std::result::Result<T, CortexError>;
```

---

## Configuration (config.rs)

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CortexConfig {
    pub stm_capacity: usize,
    pub enable_smart_collections: bool,
    pub enable_background_processing: bool,
    pub enable_memory_evolution: bool,
    pub temporal_weight: f32,
    pub collection_threshold: usize,
    pub llm_config: LlmConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub provider: LlmProvider,
    pub model: String,
    pub api_key_env: String,
    pub max_tokens: usize,
    pub temperature: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LlmProvider {
    OpenAI,
    Anthropic,
    Local,
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
            llm_config: LlmConfig::default(),
        }
    }
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: LlmProvider::OpenAI,
            model: "gpt-4".to_string(),
            api_key_env: "OPENAI_API_KEY".to_string(),
            max_tokens: 2000,
            temperature: 0.7,
        }
    }
}
```

---

## Example Usage

```rust
use cortex_memory::{AgenticMemorySystem, CortexConfig};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create memory system
    let config = CortexConfig::default();
    let base_dir = PathBuf::from("~/.q/cortex");
    let memory = AgenticMemorySystem::new(base_dir, config).await?;
    
    // Add a memory
    let id = memory.add_note(
        "Rust is a systems programming language".to_string(),
        HashMap::new(),
        Some("user123".to_string()),
        None,
    ).await?;
    
    println!("Added memory: {}", id);
    
    // Search memories
    let results = memory.search(
        "programming language",
        10,
        Some("user123".to_string()),
        None,
        None,
    ).await?;
    
    for result in results {
        println!("Score: {:.3} - {}", result.score, result.content);
    }
    
    Ok(())
}
```

This implementation guide provides the foundation for building the Cortex memory system in Rust. The next document will cover processors, collections, and advanced features.
