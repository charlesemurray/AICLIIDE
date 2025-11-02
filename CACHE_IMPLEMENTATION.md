# Response Cache Implementation Guide

## Step 1: Database Migration

Create `crates/chat-cli/src/database/sqlite_migrations/009_response_cache.sql`:

```sql
CREATE TABLE response_cache (
    cache_key TEXT PRIMARY KEY,
    conversation_id TEXT,
    user_input TEXT NOT NULL,
    response_json TEXT NOT NULL,
    model TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    expires_at INTEGER NOT NULL,
    hit_count INTEGER DEFAULT 0,
    last_hit_at INTEGER
);

CREATE INDEX idx_response_cache_expires ON response_cache(expires_at);
CREATE INDEX idx_response_cache_created ON response_cache(created_at);
```

Update `crates/chat-cli/src/database/mod.rs` MIGRATIONS array:

```rust
const MIGRATIONS: &[Migration] = migrations![
    "000_migration_table",
    "001_history_table",
    "002_drop_history_in_ssh_docker",
    "003_improved_history_timing",
    "004_state_table",
    "005_auth_table",
    "006_make_state_blob",
    "007_conversations_table",
    "009_response_cache"  // ADD THIS
];
```

## Step 2: Add Settings

Update `crates/chat-cli/src/database/settings.rs`:

```rust
#[derive(Clone, Copy, Debug, strum::EnumIter, strum::EnumMessage)]
pub enum Setting {
    // ... existing settings ...
    
    #[strum(message = "Enable response caching (boolean)")]
    CacheEnabled,
    #[strum(message = "Cache TTL in seconds (number)")]
    CacheTtlSeconds,
    #[strum(message = "Maximum cache entries (number)")]
    CacheMaxEntries,
    #[strum(message = "Number of history messages in cache key (number)")]
    CacheHistoryDepth,
}

impl Setting {
    pub fn key(&self) -> &'static str {
        match self {
            // ... existing cases ...
            Self::CacheEnabled => "cache.enabled",
            Self::CacheTtlSeconds => "cache.ttlSeconds",
            Self::CacheMaxEntries => "cache.maxEntries",
            Self::CacheHistoryDepth => "cache.historyDepth",
        }
    }

    pub fn default_value(&self) -> Option<Value> {
        match self {
            // ... existing cases ...
            Self::CacheEnabled => Some(Value::Bool(true)),
            Self::CacheTtlSeconds => Some(Value::Number(3600.into())),
            Self::CacheMaxEntries => Some(Value::Number(1000.into())),
            Self::CacheHistoryDepth => Some(Value::Number(3.into())),
        }
    }
}
```

## Step 3: Create Cache Module

Create `crates/chat-cli/src/api_client/cache.rs`:

```rust
use sha2::{Digest, Sha256};
use thiserror::Error;
use tracing::{debug, trace};

use crate::api_client::model::ConversationState;
use crate::api_client::send_message_output::SendMessageOutput;
use crate::database::{Database, DatabaseError};

#[derive(Debug, Error)]
pub enum CacheError {
    #[error(transparent)]
    Database(#[from] DatabaseError),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub struct ResponseCache {
    db: Database,
    enabled: bool,
    ttl_seconds: i64,
    max_entries: usize,
    history_depth: usize,
}

impl ResponseCache {
    pub fn new(db: Database) -> Result<Self, CacheError> {
        let settings = db.settings();
        
        let enabled = settings
            .get_bool("cache.enabled")
            .unwrap_or(true);
        
        let ttl_seconds = settings
            .get_i64("cache.ttlSeconds")
            .unwrap_or(3600);
        
        let max_entries = settings
            .get_i64("cache.maxEntries")
            .unwrap_or(1000) as usize;
        
        let history_depth = settings
            .get_i64("cache.historyDepth")
            .unwrap_or(3) as usize;

        Ok(Self {
            db,
            enabled,
            ttl_seconds,
            max_entries,
            history_depth,
        })
    }

    pub fn get(
        &self,
        conversation: &ConversationState,
        model: &str,
    ) -> Result<Option<SendMessageOutput>, CacheError> {
        if !self.enabled {
            return Ok(None);
        }

        let key = self.compute_key(conversation, model);
        trace!("Cache lookup with key: {}", key);

        match self.db.get_cached_response(&key)? {
            Some(response) => {
                debug!("Cache hit for key: {}", key);
                Ok(Some(response))
            }
            None => {
                debug!("Cache miss for key: {}", key);
                Ok(None)
            }
        }
    }

    pub fn put(
        &self,
        conversation: &ConversationState,
        model: &str,
        response: &SendMessageOutput,
    ) -> Result<(), CacheError> {
        if !self.enabled {
            return Ok(());
        }

        let key = self.compute_key(conversation, model);
        let expires_at = chrono::Utc::now().timestamp() + self.ttl_seconds;

        trace!("Caching response with key: {}", key);
        self.db.put_cached_response(&key, conversation, model, response, expires_at)?;
        self.enforce_max_entries()?;

        Ok(())
    }

    fn compute_key(&self, conversation: &ConversationState, model: &str) -> String {
        let mut hasher = Sha256::new();

        // Hash user input
        hasher.update(conversation.user_input_message.content.as_bytes());

        // Hash recent history
        if let Some(history) = &conversation.history {
            let recent: Vec<_> = history.iter().rev().take(self.history_depth).collect();
            for msg in recent {
                hasher.update(format!("{:?}", msg).as_bytes());
            }
        }

        // Hash model
        hasher.update(model.as_bytes());

        format!("{:x}", hasher.finalize())
    }

    fn enforce_max_entries(&self) -> Result<(), CacheError> {
        self.db.cleanup_old_cache_entries(self.max_entries)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_cache_key_generation() {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::new(temp_dir.path()).unwrap();
        let cache = ResponseCache::new(db).unwrap();

        let conv = ConversationState {
            conversation_id: None,
            user_input_message: UserInputMessage {
                content: "test".to_string(),
            },
            history: None,
        };

        let key1 = cache.compute_key(&conv, "model1");
        let key2 = cache.compute_key(&conv, "model1");
        let key3 = cache.compute_key(&conv, "model2");

        assert_eq!(key1, key2); // Same input = same key
        assert_ne!(key1, key3); // Different model = different key
    }

    #[test]
    fn test_cache_disabled() {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::new(temp_dir.path()).unwrap();
        
        // Disable cache
        db.settings().set("cache.enabled", false).unwrap();
        
        let cache = ResponseCache::new(db).unwrap();
        assert!(!cache.enabled);
    }
}
```

## Step 4: Add Database Methods

Add to `crates/chat-cli/src/database/mod.rs`:

```rust
impl Database {
    pub fn get_cached_response(&self, key: &str) -> Result<Option<SendMessageOutput>, DatabaseError> {
        let conn = self.pool.get()?;
        let now = chrono::Utc::now().timestamp();

        let mut stmt = conn.prepare(
            "SELECT response_json FROM response_cache 
             WHERE cache_key = ? AND expires_at > ?"
        )?;

        let result: Option<String> = stmt
            .query_row(params![key, now], |row| row.get(0))
            .optional()?;

        if let Some(json) = result {
            // Update hit count
            conn.execute(
                "UPDATE response_cache 
                 SET hit_count = hit_count + 1, last_hit_at = ? 
                 WHERE cache_key = ?",
                params![now, key],
            )?;

            Ok(Some(serde_json::from_str(&json)?))
        } else {
            Ok(None)
        }
    }

    pub fn put_cached_response(
        &self,
        key: &str,
        conversation: &ConversationState,
        model: &str,
        response: &SendMessageOutput,
        expires_at: i64,
    ) -> Result<(), DatabaseError> {
        let conn = self.pool.get()?;
        let now = chrono::Utc::now().timestamp();
        let response_json = serde_json::to_string(response)?;
        let user_input = &conversation.user_input_message.content;
        let conversation_id = conversation.conversation_id.as_deref().unwrap_or("");

        conn.execute(
            "INSERT OR REPLACE INTO response_cache 
             (cache_key, conversation_id, user_input, response_json, model, created_at, expires_at, hit_count, last_hit_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, 0, NULL)",
            params![key, conversation_id, user_input, response_json, model, now, expires_at],
        )?;

        Ok(())
    }

    pub fn cleanup_old_cache_entries(&self, max_entries: usize) -> Result<(), DatabaseError> {
        let conn = self.pool.get()?;

        conn.execute(
            "DELETE FROM response_cache 
             WHERE cache_key IN (
                 SELECT cache_key FROM response_cache 
                 ORDER BY last_hit_at DESC, created_at DESC 
                 LIMIT -1 OFFSET ?
             )",
            params![max_entries],
        )?;

        Ok(())
    }
}
```

## Step 5: Integrate into API Client

Update `crates/chat-cli/src/api_client/mod.rs`:

```rust
mod cache;
use cache::ResponseCache;

pub struct ApiClient {
    // ... existing fields ...
    cache: Option<ResponseCache>,
}

impl ApiClient {
    pub fn new(/* ... */) -> Result<Self, ApiClientError> {
        // ... existing initialization ...
        
        let cache = ResponseCache::new(db.clone())
            .map_err(|e| {
                error!("Failed to initialize cache: {}", e);
                e
            })
            .ok();

        Ok(Self {
            // ... existing fields ...
            cache,
        })
    }

    pub async fn send_message(
        &self,
        conversation: ConversationState,
    ) -> Result<SendMessageOutput, ApiClientError> {
        let model = self.get_current_model().await?;

        // Try cache first
        if let Some(cache) = &self.cache {
            if let Ok(Some(cached)) = cache.get(&conversation, &model) {
                debug!("Returning cached response");
                return Ok(cached);
            }
        }

        // Cache miss - make API call
        debug!("Cache miss - calling API");
        let response = self.send_message_impl(conversation.clone()).await?;

        // Store in cache
        if let Some(cache) = &self.cache {
            if let Err(e) = cache.put(&conversation, &model, &response) {
                error!("Failed to cache response: {}", e);
                // Don't fail the request if caching fails
            }
        }

        Ok(response)
    }

    async fn send_message_impl(
        &self,
        conversation: ConversationState,
    ) -> Result<SendMessageOutput, ApiClientError> {
        // Move existing send_message implementation here
        // ... existing code ...
    }
}
```

## Testing

### Run Tests

```bash
# Run all tests
cargo test

# Run cache-specific tests
cargo test cache

# Run with output
cargo test cache -- --nocapture

# Run integration tests
cargo test --test integration_tests
```

### Manual Testing

```bash
# Build and run
cargo build --bin chat_cli
./target/debug/chat_cli chat

# Enable debug logging
RUST_LOG=debug ./target/debug/chat_cli chat

# Check cache settings
cargo run --bin chat_cli -- settings list | grep cache

# Set cache settings
cargo run --bin chat_cli -- settings set cache.enabled true
cargo run --bin chat_cli -- settings set cache.ttlSeconds 7200
```

### Test Cache Behavior

1. Ask same question twice - second should be instant
2. Check database: `sqlite3 ~/.aws/amazonq/db.sqlite "SELECT * FROM response_cache;"`
3. Verify TTL expiration
4. Test with cache disabled

## Verification Checklist

- [ ] Migration runs successfully
- [ ] Settings appear in `q settings list`
- [ ] Cache module compiles
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Cache hit returns instant response
- [ ] Cache miss calls API
- [ ] TTL expiration works
- [ ] Max entries enforced
- [ ] No errors in logs

## Performance Expectations

- Cache lookup: < 10ms
- Cache storage: < 50ms
- No impact on cache miss path
- Memory: ~1MB per 1000 entries
