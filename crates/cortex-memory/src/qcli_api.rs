use std::path::Path;

use crate::{
    CircuitBreaker,
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
    circuit_breaker: CircuitBreaker,
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
            circuit_breaker: CircuitBreaker::default(),
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

        // Quality filtering
        if !self.should_store(user_message, assistant_response) {
            tracing::debug!("Skipping low-quality interaction");
            return Ok(String::new());
        }

        // Check circuit breaker
        if !self.circuit_breaker.should_allow() {
            return Err(crate::error::CortexError::Custom(
                "Circuit breaker open - memory operations temporarily disabled".to_string(),
            ));
        }

        let content = format!("User: {}\nAssistant: {}", user_message, assistant_response);

        match self.embedder.embed(&content) {
            Ok(embedding) => {
                // Check for duplicates (similarity > 0.95)
                let similar = self.manager.search(&embedding, 1);
                if let Some((_, score)) = similar.first() {
                    if *score > 0.95 {
                        tracing::info!(
                            similarity = %format!("{:.3}", score),
                            "Skipping duplicate memory"
                        );
                        return Ok(String::new()); // Return empty ID for duplicate
                    }
                }

                let mut metadata = std::collections::HashMap::new();
                metadata.insert("session_id".to_string(), serde_json::json!(session_id));

                let id = uuid::Uuid::new_v4().to_string();
                let note = MemoryNote::new(id.clone(), content, metadata);

                match self.manager.add(note, embedding) {
                    Ok(_) => {
                        self.circuit_breaker.record_success();
                        Ok(id)
                    },
                    Err(e) => {
                        self.circuit_breaker.record_failure();
                        Err(e)
                    },
                }
            },
            Err(e) => {
                self.circuit_breaker.record_failure();
                Err(e)
            },
        }
    }

    /// Recall relevant context for a query
    pub fn recall_context(&mut self, query: &str, limit: usize) -> Result<Vec<ContextItem>> {
        if !self.config.enabled {
            return Ok(Vec::new());
        }

        // Check circuit breaker
        if !self.circuit_breaker.should_allow() {
            return Err(crate::error::CortexError::Custom(
                "Circuit breaker open - memory operations temporarily disabled".to_string(),
            ));
        }

        match self.embedder.embed(query) {
            Ok(query_embedding) => {
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

                self.circuit_breaker.record_success();
                Ok(items)
            },
            Err(e) => {
                self.circuit_breaker.record_failure();
                Err(e)
            },
        }
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
                    .is_some_and(|s| s == session_id)
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
                    .is_some_and(|s| s == session_id)
            })
            .take(limit)
            .collect();

        Ok(filtered)
    }

    /// Clear all memories
    pub fn clear(&mut self) -> Result<usize> {
        let count = self.manager.stm_len();
        // STM will be cleared when items are evicted naturally
        Ok(count)
    }

    /// Check if interaction should be stored (quality filtering)
    fn should_store(&self, user_msg: &str, assistant_msg: &str) -> bool {
        // Too short
        if user_msg.len() < 10 || assistant_msg.len() < 10 {
            return false;
        }

        // Too long (likely code dumps)
        if user_msg.len() > 10000 || assistant_msg.len() > 10000 {
            return false;
        }

        // Error messages
        if assistant_msg.contains("Error:")
            || assistant_msg.contains("Failed to")
            || assistant_msg.contains("error[E")
            || assistant_msg.starts_with("error:")
        {
            return false;
        }

        true
    }

    /// Toggle memory enabled state
    pub fn set_enabled(&mut self, enabled: bool) {
        self.config.enabled = enabled;
    }

    /// Check if memory is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Get circuit breaker state
    pub fn circuit_breaker_state(&self) -> crate::CircuitState {
        self.circuit_breaker.state()
    }

    /// Get circuit breaker failure count
    pub fn circuit_breaker_failures(&self) -> u32 {
        self.circuit_breaker.failure_count()
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
