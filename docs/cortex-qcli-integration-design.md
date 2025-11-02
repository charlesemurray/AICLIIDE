# Cortex Memory - Q CLI Integration & UX Design

## Executive Summary

This document provides a comprehensive design for integrating Cortex memory system into Q CLI, including technical architecture, API design, and user experience.

**Goal**: Enable Q CLI to remember context across conversations, improving relevance and reducing repetition.

---

## Table of Contents

1. [Architecture Design](#architecture-design)
2. [API Design](#api-design)
3. [Storage Design](#storage-design)
4. [UX Design](#ux-design)
5. [Implementation Plan](#implementation-plan)
6. [Migration Strategy](#migration-strategy)

---

## Architecture Design

### Current Q CLI Architecture

```
┌─────────────────────────────────────────────────────┐
│                    Q CLI                            │
├─────────────────────────────────────────────────────┤
│  Chat Loop                                          │
│  ├── User Input                                     │
│  ├── LLM Request (with conversation history)       │
│  ├── Tool Execution                                 │
│  └── Response Display                               │
├─────────────────────────────────────────────────────┤
│  Session Management                                 │
│  └── In-memory conversation state                   │
└─────────────────────────────────────────────────────┘
```

### Proposed Architecture with Cortex

```
┌─────────────────────────────────────────────────────┐
│                    Q CLI                            │
├─────────────────────────────────────────────────────┤
│  Chat Loop                                          │
│  ├── User Input                                     │
│  ├── Memory Recall (retrieve relevant context)     │
│  ├── LLM Request (with history + recalled context) │
│  ├── Memory Store (save interaction)               │
│  ├── Tool Execution                                 │
│  └── Response Display                               │
├─────────────────────────────────────────────────────┤
│  Session Management                                 │
│  ├── In-memory conversation state                   │
│  └── Cortex Memory Manager                          │
│      ├── STM (recent interactions)                  │
│      └── LTM (persistent knowledge)                 │
└─────────────────────────────────────────────────────┘
```

### Component Integration Points

```rust
// In chat loop
pub struct ChatSession {
    conversation: ConversationState,
    memory: Option<MemoryManager>,  // New
    tool_manager: ToolManager,
    // ... existing fields
}

impl ChatSession {
    pub async fn process_message(&mut self, input: &str) -> Result<()> {
        // 1. Recall relevant context from memory
        let context = self.recall_context(input).await?;
        
        // 2. Build prompt with context
        let prompt = self.build_prompt_with_context(input, &context);
        
        // 3. Send to LLM
        let response = self.send_to_llm(&prompt).await?;
        
        // 4. Store interaction in memory
        self.store_interaction(input, &response).await?;
        
        // 5. Display response
        self.display_response(&response)?;
        
        Ok(())
    }
}
```

---

## API Design

### Public API for Q CLI

```rust
// crates/cortex-memory/src/qcli_api.rs

use crate::{MemoryManager, MemoryNote, Result};
use std::path::Path;

/// High-level API for Q CLI integration
pub struct CortexMemory {
    manager: MemoryManager,
    embedding_service: Box<dyn EmbeddingService>,
}

impl CortexMemory {
    /// Initialize Cortex memory for a user
    pub fn new<P: AsRef<Path>>(
        db_path: P,
        embedding_service: Box<dyn EmbeddingService>,
    ) -> Result<Self> {
        let manager = MemoryManager::new(db_path, 384, 100)?;
        Ok(Self {
            manager,
            embedding_service,
        })
    }
    
    /// Store a user message and assistant response
    pub async fn store_interaction(
        &mut self,
        user_message: &str,
        assistant_response: &str,
        metadata: InteractionMetadata,
    ) -> Result<String> {
        let content = format!("User: {}\nAssistant: {}", user_message, assistant_response);
        let embedding = self.embedding_service.embed(&content).await?;
        
        let note = MemoryNote::new(
            uuid::Uuid::new_v4().to_string(),
            content,
            metadata.into_map(),
        );
        
        self.manager.add(note, embedding)?;
        Ok(note.id)
    }
    
    /// Recall relevant context for a query
    pub async fn recall_context(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<ContextItem>> {
        let query_embedding = self.embedding_service.embed(query).await?;
        let results = self.manager.search(&query_embedding, limit);
        
        Ok(results
            .into_iter()
            .map(|(id, score)| {
                let note = self.manager.get(&id).unwrap().unwrap();
                ContextItem {
                    content: note.content,
                    relevance: score,
                    timestamp: note.created_at,
                }
            })
            .collect())
    }
    
    /// Promote important memories to long-term storage
    pub async fn promote_to_ltm(&mut self, memory_id: &str) -> Result<bool> {
        let embedding = self.embedding_service.embed(&memory_id).await?;
        self.manager.promote_to_ltm(memory_id, embedding)
    }
}

/// Metadata for storing interactions
#[derive(Debug, Clone)]
pub struct InteractionMetadata {
    pub session_id: String,
    pub user_id: Option<String>,
    pub timestamp: String,
    pub tags: Vec<String>,
    pub context: String,
}

/// Context item returned from recall
#[derive(Debug, Clone)]
pub struct ContextItem {
    pub content: String,
    pub relevance: f32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Trait for embedding generation
#[async_trait::async_trait]
pub trait EmbeddingService: Send + Sync {
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;
}
```

### Embedding Service Implementation

**✅ DECISION: Use Q CLI's existing `CandleTextEmbedder`**

Q CLI already has a complete embedding system in `semantic-search-client` crate:
- Model: all-MiniLM-L6-v2 (384 dimensions)
- Framework: Candle (Rust ML)
- Quality: Production-proven for knowledge search
- Performance: ~10-50ms per embedding
- Works offline, no external API needed

```rust
use semantic_search_client::embedding::{EmbeddingType, TextEmbedderTrait};
use semantic_search_client::client::embedder_factory::create_embedder;

pub struct CortexMemory {
    manager: MemoryManager,
    embedder: Box<dyn TextEmbedderTrait>,
}

impl CortexMemory {
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        // Use Q CLI's existing embedder - 384 dimensions matches our HNSW setup!
        let embedder = create_embedder(EmbeddingType::Best)?;
        let manager = MemoryManager::new(db_path, 384, 100)?;
        Ok(Self { manager, embedder })
    }
    
    pub async fn store_interaction(
        &mut self,
        user_message: &str,
        assistant_response: &str,
        metadata: InteractionMetadata,
    ) -> Result<String> {
        let content = format!("User: {}\nAssistant: {}", user_message, assistant_response);
        let embedding = self.embedder.embed(&content)?;
        
        let note = MemoryNote::new(
            uuid::Uuid::new_v4().to_string(),
            content,
            metadata.into_map(),
        );
        
        self.manager.add(note, embedding)?;
        Ok(note.id)
    }
    
    pub async fn recall_context(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<ContextItem>> {
        let query_embedding = self.embedder.embed(query)?;
        let results = self.manager.search(&query_embedding, limit);
        
        Ok(results
            .into_iter()
            .map(|(id, score)| {
                let note = self.manager.get(&id).unwrap().unwrap();
                ContextItem {
                    content: note.content,
                    relevance: score,
                    timestamp: note.created_at,
                }
            })
            .collect())
    }
}
```

**See**: `docs/cortex-embedding-research.md` for detailed research findings.

---

## Storage Design

### Database Location

```
~/.q/
├── config/
├── logs/
└── memory/
    ├── global.db          # Cross-session memories
    └── sessions/
        ├── session-1.db   # Per-session memories
        └── session-2.db
```

### Schema Design

```sql
-- SQLite schema (already implemented in LTM)
CREATE TABLE memories (
    id TEXT PRIMARY KEY,
    content TEXT NOT NULL,
    metadata TEXT NOT NULL,  -- JSON: {session_id, user_id, tags, context}
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Index for fast session filtering
CREATE INDEX idx_session ON memories(json_extract(metadata, '$.session_id'));
CREATE INDEX idx_user ON memories(json_extract(metadata, '$.user_id'));
```

### Storage Strategy

**Option 1: Single Global Database** (Recommended for MVP)
- One `~/.q/memory/cortex.db` for all memories
- Filter by session_id/user_id in queries
- Simpler implementation
- Easier to search across sessions

**Option 2: Per-Session Databases**
- Separate DB per session
- Better isolation
- More complex management
- Harder to search across sessions

**Recommendation**: Start with Option 1, migrate to Option 2 if needed

---

## UX Design

### User-Facing Features

#### 1. Automatic Memory (Default Behavior)

**User Experience**: Transparent, no explicit commands needed

```bash
$ q chat

You: How do I create a Rust struct?
Q: Here's how to create a Rust struct...

# Later in same session
You: Can you show me an example with that?
Q: Sure! Based on our earlier discussion about Rust structs...
   # Q automatically recalls previous context
```

**Implementation**:
- Every interaction automatically stored in STM
- Relevant context automatically recalled for each query
- No user action required

#### 2. Memory Commands

**List Recent Memories**
```bash
$ q memory list
Recent memories (last 10):
1. [2 min ago] Discussion about Rust structs
2. [5 min ago] AWS Lambda deployment question
3. [1 hour ago] Python async/await explanation
...

$ q memory list --session <session-id>
$ q memory list --limit 20
```

**Search Memories**
```bash
$ q memory search "rust structs"
Found 3 relevant memories:
1. [2 min ago] How to create a Rust struct (relevance: 0.95)
2. [1 day ago] Rust struct vs enum discussion (relevance: 0.82)
3. [3 days ago] Rust ownership in structs (relevance: 0.75)

$ q memory search "rust" --limit 5
```

**Show Memory Details**
```bash
$ q memory show <memory-id>
Memory ID: abc123
Created: 2025-11-02 14:30:00
Session: session-xyz
Tags: rust, programming, structs

Content:
User: How do I create a Rust struct?
Assistant: Here's how to create a Rust struct...
```

**Delete Memories**
```bash
$ q memory delete <memory-id>
Deleted memory: abc123

$ q memory clear --session <session-id>
Cleared 15 memories from session

$ q memory clear --all
⚠️  This will delete ALL memories. Are you sure? (y/N)
```

**Memory Statistics**
```bash
$ q memory stats
Memory Statistics:
- Total memories: 1,247
- Short-term (active): 42
- Long-term (archived): 1,205
- Sessions: 23
- Oldest memory: 30 days ago
- Storage size: 12.4 MB
```

#### 3. Memory Control Flags

**Disable Memory for Sensitive Conversations**
```bash
$ q chat --no-memory
# Memory not stored or recalled for this session

$ q chat --ephemeral
# Alias for --no-memory
```

**Explicit Memory Promotion**
```bash
You: Remember this: my AWS account ID is 123456789
Q: I'll remember that. [Memory saved to long-term storage]

# In code:
# Detect "remember this" pattern and promote to LTM immediately
```

**Memory Scope**
```bash
$ q chat --memory-scope session  # Default: only this session
$ q chat --memory-scope global   # Search all sessions
$ q chat --memory-scope none     # No memory
```

#### 4. Visual Indicators

**In Chat Interface**
```bash
You: How do I deploy to AWS Lambda?

[🧠 Recalling context...]  # Brief indicator while searching

Q: Based on your previous work with Python Lambda functions...
   # Response includes recalled context

[💾 Memory saved]  # Optional: show when important memory stored
```

**Memory Relevance Indicator**
```bash
Q: I found some relevant context from our previous conversations:

   📌 [95% relevant] Discussion about Lambda deployment (2 days ago)
   📌 [82% relevant] AWS credentials setup (1 week ago)
   
   Based on this context...
```

#### 5. Privacy & Control

**Memory Settings**
```bash
$ q settings memory

Memory Settings:
✓ Automatic memory enabled
✓ Cross-session recall enabled
  Memory retention: 30 days
  Auto-promote threshold: 3 references
  
Change settings:
  q settings memory --retention 90
  q settings memory --no-cross-session
  q settings memory --disable
```

**Export Memories**
```bash
$ q memory export memories.json
Exported 1,247 memories to memories.json

$ q memory export --session <session-id> session-memories.json
```

**Import Memories**
```bash
$ q memory import memories.json
Imported 1,247 memories
```

---

## Implementation Plan

### Phase 1: Core Integration (Week 1)

**Goal**: Basic memory storage and recall working in Q CLI

**Tasks**:
1. Add `cortex-memory` dependency to `chat-cli`
2. Create `CortexMemory` wrapper API
3. Initialize memory manager in chat session
4. Store interactions after each exchange
5. Basic recall before LLM request

**Deliverables**:
- Memory automatically stored
- Memory automatically recalled
- No user-facing commands yet

**Testing**:
- Unit tests for API
- Integration test: store and recall in chat session
- Manual testing: verify context improves responses

### Phase 2: Memory Commands (Week 2)

**Goal**: User can view and manage memories

**Tasks**:
1. Add `q memory` subcommand
2. Implement `list`, `search`, `show`, `delete`
3. Add `--no-memory` flag to chat
4. Add visual indicators in chat

**Deliverables**:
- `q memory list`
- `q memory search <query>`
- `q memory show <id>`
- `q memory delete <id>`
- `q chat --no-memory`

**Testing**:
- CLI command tests
- UX testing with real users
- Performance testing with large memory sets

### Phase 3: Advanced Features (Week 3)

**Goal**: Smart memory management and user control

**Tasks**:
1. Implement memory statistics
2. Add memory export/import
3. Add memory settings
4. Implement auto-promotion logic
5. Add "remember this" pattern detection

**Deliverables**:
- `q memory stats`
- `q memory export/import`
- `q settings memory`
- Smart promotion to LTM
- Explicit memory commands

**Testing**:
- Long-term usage testing
- Memory retention testing
- Performance benchmarks

### Phase 4: Embedding Integration (Week 4)

**Goal**: Real embeddings instead of placeholders

**Tasks**:
1. Integrate with Q's LLM for embeddings
2. Or use lightweight embedding model (e.g., all-MiniLM)
3. Benchmark embedding generation speed
4. Optimize for performance

**Deliverables**:
- Real semantic search
- Fast embedding generation
- Improved recall relevance

**Testing**:
- Semantic search quality tests
- Performance benchmarks
- A/B testing vs placeholder embeddings

---

## Migration Strategy

### Rollout Plan

**Stage 1: Internal Testing** (Week 1-2)
- Feature flag: `QCLI_ENABLE_MEMORY=1`
- Internal team testing only
- Gather feedback on UX

**Stage 2: Beta Release** (Week 3-4)
- Opt-in beta flag: `q settings memory --enable-beta`
- Limited user group
- Monitor performance and bugs

**Stage 3: General Availability** (Week 5+)
- Default enabled for all users
- Opt-out available: `q settings memory --disable`
- Full documentation and tutorials

### Backward Compatibility

**Existing Sessions**:
- Continue to work without memory
- Memory is additive, not breaking
- No migration needed

**Storage**:
- New `~/.q/memory/` directory
- Doesn't affect existing Q CLI data
- Can be deleted without breaking Q CLI

---

## UX Principles

### 1. Invisible by Default
- Memory should "just work" without user thinking about it
- No explicit commands required for basic usage
- Transparent context enhancement

### 2. Controllable When Needed
- Users can inspect what's remembered
- Users can delete specific memories
- Users can disable memory entirely

### 3. Privacy-Conscious
- Clear about what's stored
- Easy to delete
- Local storage only (no cloud sync by default)
- Sensitive data warnings

### 4. Performance-Aware
- Fast recall (< 100ms)
- Non-blocking storage
- Minimal impact on chat latency

### 5. Helpful, Not Intrusive
- Subtle indicators, not noisy
- Relevant context only
- Graceful degradation if memory unavailable

---

## Open Questions - RESOLVED ✅

### 1. ✅ Embedding Model
**Decision**: Use Q CLI's existing `CandleTextEmbedder` (all-MiniLM-L6-v2, 384 dims)
- Already integrated in `semantic-search-client` crate
- Zero additional cost, production-proven
- See: `docs/cortex-embedding-research.md`

### 2. ✅ Memory Scope Default
**Decision**: Session-only by default, configurable cross-session
- Default: Current session only
- `/recall --session <id>` for specific sessions
- `/recall --global` for all sessions
- `/recall --list-sessions` for discovery
- See: `docs/cortex-session-integration.md`

### 3. ✅ Memory Retention
**Decision**: Hybrid (30 days OR 100 MB, whichever first)
- Configurable via `~/.q/config/settings.json`
- Settings: `memory.retentionDays`, `memory.maxSizeMb`
- Auto-cleanup on startup, after store, daily
- See: `docs/cortex-memory-config.md`

### 4. ✅ Privacy Default
**Decision**: Enabled by default with clear disclosure
- Local storage only (no cloud sync)
- Welcome message on first run
- Easy opt-out: `/memory toggle --disable`
- Ephemeral sessions: `q chat --no-memory`
- See: `docs/cortex-privacy-design.md`

### 5. ✅ Visual Indicators
**Decision**: Minimal (Q CLI's Spinner during recall)
- `▰▰▰▱▱▱▱ Recalling context...` during search
- Silent during store
- Verbose mode available via `memory.verbose` setting
- See: `docs/cortex-visual-indicators.md`
## Success Metrics

### Technical Metrics
- Memory recall latency < 100ms (p95)
- Storage overhead < 100MB for typical user
- Zero crashes related to memory
- 99.9% uptime for memory features

### UX Metrics
- 80%+ users find context helpful
- < 5% users disable memory
- 50%+ users use memory commands
- Positive feedback on relevance

### Business Metrics
- Reduced repetitive questions
- Improved conversation quality
- Higher user satisfaction scores
- Increased Q CLI engagement

---

## Next Steps

1. **Review this design** with team
2. **Prototype Phase 1** (core integration)
3. **User testing** with internal team
4. **Iterate** based on feedback
5. **Ship Phase 1** to beta users

---

## Appendix: Example User Flows

### Flow 1: First-Time User

```bash
$ q chat

Welcome to Amazon Q Developer CLI!

You: How do I create a React component?
Q: Here's how to create a React component...

You: Can you show me with TypeScript?
Q: Sure! Here's the TypeScript version...

# Memory automatically stored, no user action needed
```

### Flow 2: Returning User

```bash
$ q chat

You: What was that React component pattern we discussed?

[🧠 Recalling context...]

Q: I found our previous discussion about React components!
   
   📌 [95% relevant] React component creation (2 days ago)
   
   Based on that conversation, you were asking about...
```

### Flow 3: Power User

```bash
$ q memory search "react"
Found 5 relevant memories:
1. React component patterns (2 days ago)
2. React hooks explanation (1 week ago)
...

$ q memory show mem-123
Memory ID: mem-123
Created: 2025-10-31 14:30:00
Content: Discussion about React hooks...

$ q memory delete mem-123
Deleted memory: mem-123

$ q memory stats
Total memories: 1,246
```

### Flow 4: Privacy-Conscious User

```bash
$ q chat --no-memory

You: [sensitive question about internal systems]
Q: [response]

# Nothing stored, nothing recalled

$ q memory list
No memories found (memory was disabled for last session)
```
