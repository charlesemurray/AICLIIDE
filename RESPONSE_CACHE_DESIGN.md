# Response Cache Design

## Overview
Add transparent response caching to Q CLI to reduce API calls and provide instant responses for repeated queries.

## Architecture

### Cache Key Strategy
Hash of:
- User input message content
- Conversation history (last N messages)
- Current model
- Relevant context (files, tools available)

```rust
struct CacheKey {
    user_input_hash: String,      // SHA256 of user message
    history_hash: String,          // SHA256 of last 3 messages
    model: String,                 // e.g., "anthropic.claude-3-sonnet"
    context_hash: String,          // SHA256 of file paths + tool list
}
```

### Database Schema

```sql
-- Migration: 009_response_cache.sql
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
CREATE INDEX idx_response_cache_model ON response_cache(model);
CREATE INDEX idx_response_cache_created ON response_cache(created_at);

-- Cleanup old entries
CREATE TRIGGER cleanup_expired_cache
AFTER INSERT ON response_cache
BEGIN
    DELETE FROM response_cache WHERE expires_at < strftime('%s', 'now');
END;
```

### Settings

Add to `Setting` enum in `database/settings.rs`:

```rust
#[strum(message = "Enable response caching (boolean)")]
CacheEnabled,

#[strum(message = "Cache TTL in seconds (number)")]
CacheTtlSeconds,

#[strum(message = "Maximum cache entries (number)")]
CacheMaxEntries,

#[strum(message = "Number of history messages to include in cache key (number)")]
CacheHistoryDepth,
```

Default values:
- `cache.enabled` = `true`
- `cache.ttlSeconds` = `3600` (1 hour)
- `cache.maxEntries` = `1000`
- `cache.historyDepth` = `3`

## Implementation

### File Structure

```
crates/chat-cli/src/
├── api_client/
│   ├── cache.rs          # NEW: Cache implementation
│   └── mod.rs            # Updated: integrate cache
└── database/
    └── sqlite_migrations/
        └── 009_response_cache.sql  # NEW: Cache schema
```

### Core Cache Module

`crates/chat-cli/src/api_client/cache.rs`:

```rust
use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};
use crate::database::Database;
use crate::api_client::model::ConversationState;
use crate::api_client::send_message_output::SendMessageOutput;

pub struct ResponseCache {
    db: Database,
    enabled: bool,
    ttl_seconds: i64,
    max_entries: usize,
    history_depth: usize,
}

impl ResponseCache {
    pub fn new(db: Database) -> Result<Self, CacheError> {
        let enabled = db.get_setting("cache.enabled")?.unwrap_or(true);
        let ttl_seconds = db.get_setting("cache.ttlSeconds")?.unwrap_or(3600);
        let max_entries = db.get_setting("cache.maxEntries")?.unwrap_or(1000);
        let history_depth = db.get_setting("cache.historyDepth")?.unwrap_or(3);
        
        Ok(Self { db, enabled, ttl_seconds, max_entries, history_depth })
    }

    pub async fn get(
        &self,
        conversation: &ConversationState,
        model: &str,
    ) -> Result<Option<SendMessageOutput>, CacheError> {
        if !self.enabled {
            return Ok(None);
        }

        let key = self.compute_key(conversation, model)?;
        self.db.get_cached_response(&key)
    }

    pub async fn put(
        &self,
        conversation: &ConversationState,
        model: &str,
        response: &SendMessageOutput,
    ) -> Result<(), CacheError> {
        if !self.enabled {
            return Ok(());
        }

        let key = self.compute_key(conversation, model)?;
        let expires_at = chrono::Utc::now().timestamp() + self.ttl_seconds;
        
        self.db.put_cached_response(&key, conversation, model, response, expires_at)?;
        self.enforce_max_entries()?;
        
        Ok(())
    }

    fn compute_key(
        &self,
        conversation: &ConversationState,
        model: &str,
    ) -> Result<String, CacheError> {
        let mut hasher = Sha256::new();
        
        // Hash user input
        hasher.update(conversation.user_input_message.content.as_bytes());
        
        // Hash recent history
        if let Some(history) = &conversation.history {
            let recent = history.iter()
                .rev()
                .take(self.history_depth)
                .collect::<Vec<_>>();
            
            for msg in recent {
                hasher.update(format!("{:?}", msg).as_bytes());
            }
        }
        
        // Hash model
        hasher.update(model.as_bytes());
        
        Ok(format!("{:x}", hasher.finalize()))
    }

    fn enforce_max_entries(&self) -> Result<(), CacheError> {
        self.db.cleanup_old_cache_entries(self.max_entries)
    }
}
```

### Database Integration

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
                params![now, key]
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
            params![key, conversation_id, user_input, response_json, model, now, expires_at]
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
            params![max_entries]
        )?;
        
        Ok(())
    }
}
```

### API Client Integration

Update `crates/chat-cli/src/api_client/mod.rs`:

```rust
use crate::api_client::cache::ResponseCache;

pub struct ApiClient {
    // ... existing fields ...
    cache: Option<ResponseCache>,
}

impl ApiClient {
    pub async fn send_message(
        &self,
        conversation: ConversationState
    ) -> Result<SendMessageOutput, ApiClientError> {
        let model = self.get_current_model().await?;
        
        // Try cache first
        if let Some(cache) = &self.cache {
            if let Some(cached) = cache.get(&conversation, &model).await? {
                debug!("Cache hit for conversation");
                return Ok(cached);
            }
        }
        
        // Cache miss - make API call
        debug!("Cache miss - calling API");
        let response = self.send_message_impl(conversation.clone()).await?;
        
        // Store in cache
        if let Some(cache) = &self.cache {
            cache.put(&conversation, &model, &response).await?;
        }
        
        Ok(response)
    }
    
    fn send_message_impl(&self, conversation: ConversationState) 
        -> Result<SendMessageOutput, ApiClientError> {
        // Existing implementation
    }
}
```

## Cache Invalidation

### Automatic
- TTL expiration (default 1 hour)
- LRU eviction when max_entries exceeded
- Trigger on INSERT cleans expired entries

### Manual
Add CLI command:

```bash
q cache clear              # Clear all cache
q cache clear --expired    # Clear only expired
q cache stats              # Show cache statistics
```

## Metrics

Track in telemetry:
- Cache hit rate
- Cache miss rate
- Average response time (cached vs uncached)
- Cache size
- Eviction count

## Testing Strategy

### Unit Tests
- Cache key generation
- TTL expiration
- LRU eviction
- Serialization/deserialization

### Integration Tests
- End-to-end cache hit/miss
- Multiple conversations
- Cache invalidation
- Settings changes

### Performance Tests
- Cache lookup speed (< 10ms)
- Storage overhead
- Memory usage

## Migration Path

1. Add migration `009_response_cache.sql`
2. Add cache settings to `Setting` enum
3. Implement `ResponseCache` module
4. Add database methods
5. Integrate into `ApiClient`
6. Add CLI commands
7. Add telemetry
8. Test and deploy

## Success Metrics

- 40-60% cache hit rate in typical usage
- < 10ms cache lookup time
- Zero impact on cache miss path
- Transparent to user (except speed improvement)
