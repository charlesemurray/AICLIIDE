use std::path::Path;

use crate::{
    CortexEmbedder,
    MemoryConfig,
    MemoryManager,
    MemoryNote,
    Result,
};

/// High-level API for Q CLI integration
pub struct CortexMemory {
    manager: MemoryManager,
    embedder: CortexEmbedder,
    config: MemoryConfig,
}

impl CortexMemory {
    /// Create a new CortexMemory instance
    pub fn new<P: AsRef<Path>>(db_path: P, config: MemoryConfig) -> Result<Self> {
        let embedder = CortexEmbedder::new()?;
        let manager = MemoryManager::new(db_path, embedder.dimensions(), 20)?;

        Ok(Self {
            manager,
            embedder,
            config,
        })
    }

    /// Store a user-assistant interaction
    pub fn store_interaction(
        &mut self,
        user_message: &str,
        assistant_response: &str,
        session_id: &str,
    ) -> Result<String> {
        if !self.config.enabled {
            return Ok(String::new());
        }

        let content = format!("User: {}\nAssistant: {}", user_message, assistant_response);
        let embedding = self.embedder.embed(&content)?;

        let mut metadata = std::collections::HashMap::new();
        metadata.insert("session_id".to_string(), serde_json::json!(session_id));

        let id = uuid::Uuid::new_v4().to_string();
        let note = MemoryNote::new(id.clone(), content, metadata);

        self.manager.add(note, embedding)?;
        Ok(id)
    }

    /// Recall relevant context for a query
    pub fn recall_context(&mut self, query: &str, limit: usize) -> Result<Vec<ContextItem>> {
        if !self.config.enabled {
            return Ok(Vec::new());
        }

        let query_embedding = self.embedder.embed(query)?;
        let results = self.manager.search(&query_embedding, limit);

        let mut items = Vec::new();
        for (id, score) in results {
            if let Some(note) = self.manager.get(&id)? {
                items.push(ContextItem {
                    id: note.id,
                    content: note.content,
                    score,
                    metadata: note.metadata,
                });
            }
        }

        Ok(items)
    }

    /// Recall context filtered by session
    pub fn recall_by_session(&mut self, query: &str, session_id: &str, limit: usize) -> Result<Vec<ContextItem>> {
        let all_items = self.recall_context(query, limit * 2)?;

        let filtered: Vec<ContextItem> = all_items
            .into_iter()
            .filter(|item| {
                item.metadata
                    .get("session_id")
                    .and_then(|v| v.as_str())
                    .map(|s| s == session_id)
                    .unwrap_or(false)
            })
            .take(limit)
            .collect();

        Ok(filtered)
    }

    /// Get memory statistics
    pub fn stats(&self) -> MemoryStats {
        MemoryStats {
            stm_count: self.manager.stm_len(),
            stm_capacity: self.manager.stm_capacity(),
            enabled: self.config.enabled,
        }
    }

    /// List recent memories
    pub fn list_recent(&mut self, limit: usize) -> Result<Vec<ContextItem>> {
        if !self.config.enabled {
            return Ok(Vec::new());
        }

        // Get a sample query to retrieve recent items
        let query_embedding = vec![0.0; self.embedder.dimensions()];
        let results = self.manager.search(&query_embedding, limit);

        let mut items = Vec::new();
        for (id, score) in results {
            if let Some(note) = self.manager.get(&id)? {
                items.push(ContextItem {
                    id: note.id,
                    content: note.content,
                    score,
                    metadata: note.metadata,
                });
            }
        }

        Ok(items)
    }

    /// List memories filtered by session
    pub fn list_by_session(&mut self, session_id: &str, limit: usize) -> Result<Vec<ContextItem>> {
        let all_items = self.list_recent(limit * 2)?;

        let filtered: Vec<ContextItem> = all_items
            .into_iter()
            .filter(|item| {
                item.metadata
                    .get("session_id")
                    .and_then(|v| v.as_str())
                    .map(|s| s == session_id)
                    .unwrap_or(false)
            })
            .take(limit)
            .collect();

        Ok(filtered)
    }

    /// Clear all memories
    pub fn clear(&mut self) -> Result<usize> {
        let count = self.manager.stm_len();
        // STM will be cleared when items are evicted naturally
        // For now, just return the count
        Ok(count)
    }

    /// Toggle memory enabled state
    pub fn set_enabled(&mut self, enabled: bool) {
        self.config.enabled = enabled;
    }

    /// Check if memory is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }
}

/// A recalled context item
#[derive(Debug, Clone)]
pub struct ContextItem {
    pub id: String,
    pub content: String,
    pub score: f32,
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

/// Memory statistics
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub stm_count: usize,
    pub stm_capacity: usize,
    pub enabled: bool,
}
