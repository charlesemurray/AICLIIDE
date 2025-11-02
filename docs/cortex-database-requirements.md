# Cortex Database Requirements Analysis

## Overview

This document analyzes Cortex's Python implementation to extract all database requirements needed for a Rust port.

---

## Core Database Operations

### 1. Document Management

#### 1.1 Add Document
```python
# From ChromaRetriever.add_document()
def add_document(self, document: str, metadata: Dict, doc_id: str):
    # Requirements:
    # - Accept custom string ID (not auto-generated)
    # - Store document text
    # - Store arbitrary metadata (dict)
    # - Generate and store embedding vector
    # - Handle updates (delete + re-add if ID exists)
```

**Requirements**:
- ✅ Custom string IDs (user-provided, typically UUID)
- ✅ Document text storage
- ✅ Metadata storage (arbitrary key-value pairs)
- ✅ Embedding vector storage (generated from text)
- ✅ Upsert behavior (update if exists)

#### 1.2 Get Document by ID
```python
# From ChromaRetriever.get_document()
def get_document(self, doc_id: str) -> Optional[Dict]:
    # Requirements:
    # - Direct lookup by ID (not search)
    # - Return document text
    # - Return all metadata
    # - Return None if not found
```

**Requirements**:
- ✅ Direct ID-based retrieval (O(1) or O(log n))
- ✅ Return full document with metadata
- ✅ Handle missing documents gracefully

#### 1.3 Delete Document
```python
# From ChromaRetriever.delete_document()
def delete_document(self, doc_id: str):
    # Requirements:
    # - Remove document by ID
    # - Remove from vector index
    # - Remove from metadata store
    # - Clean up internal tracking
```

**Requirements**:
- ✅ Delete by ID
- ✅ Remove from all indices (vector + metadata)
- ✅ No orphaned data

#### 1.4 Update Document
```python
# Implemented as delete + add
if doc_id in self.content_hashes:
    self.collection.delete(ids=[doc_id])
self.collection.add(...)
```

**Requirements**:
- ✅ Update existing document
- ✅ Preserve ID
- ✅ Update embedding if content changed
- ⚠️ Can be implemented as delete + add

---

## Search Operations

### 2. Vector Search

#### 2.1 Basic Semantic Search
```python
# From ChromaRetriever.search()
def search(self, query: str, k: int = 5, where_filter: Optional[Dict] = None):
    # Requirements:
    # - Generate query embedding
    # - Find k nearest neighbors by cosine similarity
    # - Return results with distances
    # - Include document text and metadata
```

**Requirements**:
- ✅ Cosine similarity search
- ✅ Top-K results
- ✅ Return distance/score
- ✅ Return full documents with metadata

#### 2.2 Filtered Search
```python
# From ChromaRetriever.search() with where_filter
results = self.collection.query(
    query_embeddings=[query_embedding],
    n_results=k,
    where=where_filter,  # <-- Metadata filtering
    include=['metadatas', 'documents', 'distances']
)
```

**Requirements**:
- ✅ Filter by metadata fields DURING search
- ✅ Support multiple filter conditions
- ✅ Efficient (not post-filtering all results)

---

## Metadata Filtering

### 3. Filter Operations

#### 3.1 Equality Filter
```python
# From LTM.search()
where_filter = {"user_id": {"$eq": user_id}}
```

**Requirements**:
- ✅ Exact match on field value
- ✅ Support for strings, numbers, booleans

#### 3.2 Compound Filters
```python
# From LTM.search()
where_conditions = []
if user_id is not None:
    where_conditions.append({"user_id": {"$eq": user_id}})
if session_id is not None:
    where_conditions.append({"session_id": {"$eq": session_id}})

chroma_where = {"$and": where_conditions}
```

**Requirements**:
- ✅ AND logic (all conditions must match)
- ✅ Multiple field filters
- ⚠️ OR logic not used in Cortex but nice to have

#### 3.3 Range Filters (Temporal)
```python
# From ChromaRetriever - stores timestamp_epoch for range queries
processed_metadata['timestamp_epoch'] = datetime.fromisoformat(ts).timestamp()

# Implied usage (not shown but mentioned in comments):
# where_filter = {"timestamp_epoch": {"$gt": start_time}}
```

**Requirements**:
- ✅ Greater than (>)
- ✅ Less than (<)
- ✅ Numeric comparisons
- ⚠️ Used for temporal filtering

---

## Metadata Storage

### 4. Metadata Types

#### 4.1 Supported Types
```python
# From ChromaRetriever.add_document()
for key, value in metadata.items():
    if isinstance(value, (int, float, bool)):
        processed_metadata[key] = value  # Store directly
    elif isinstance(value, list):
        processed_metadata[key] = json.dumps(value)  # Serialize
    elif isinstance(value, dict):
        processed_metadata[key] = json.dumps(value)  # Serialize
    else:
        processed_metadata[key] = str(value)  # Convert to string
```

**Requirements**:
- ✅ Primitives: int, float, bool, string
- ✅ Complex: lists, dicts (serialized as JSON strings)
- ✅ Null/None handling
- ✅ Type preservation on retrieval

#### 4.2 Special Metadata Fields
```python
# From memory_system.py and ChromaRetriever
metadata = {
    "id": memory.id,                    # String (UUID)
    "keywords": memory.keywords,         # List[str]
    "links": memory.links,              # Dict (serialized)
    "retrieval_count": memory.retrieval_count,  # int
    "timestamp": memory.timestamp,       # str (ISO format)
    "timestamp_epoch": epoch_time,       # float (for range queries)
    "last_accessed": memory.last_accessed,  # str
    "context": memory.context,           # str
    "category": memory.category,         # str (hierarchical: "work.programming.python")
    "tags": memory.tags,                # List[str]
    "user_id": user_id,                 # str (for multi-user)
    "session_id": session_id,           # str (for sessions)
    "evolution_history": history,        # List (serialized)
}
```

**Requirements**:
- ✅ Store all these field types
- ✅ Preserve structure (lists, dicts)
- ✅ Support hierarchical categories
- ✅ Support user/session isolation

---

## User & Session Isolation

### 5. Multi-Tenancy

#### 5.1 User Isolation
```python
# From STM and LTM
def add(self, memory_id: str, content: str, metadata: Dict, 
        user_id: Optional[str] = None, session_id: Optional[str] = None):
    # Store user_id in metadata
    if user_id is not None:
        metadata["user_id"] = user_id
```

**Requirements**:
- ✅ Store user_id with each document
- ✅ Filter by user_id in searches
- ✅ Prevent cross-user data leakage

#### 5.2 Session Isolation
```python
# From LTM._get_collection_name()
def _get_collection_name(self, user_id: Optional[str], session_id: Optional[str]) -> str:
    if user_id and session_id:
        return f"memories_{user_id}_{session_id}"
    elif user_id:
        return f"memories_{user_id}"
    elif session_id:
        return f"memories_{session_id}"
    return "memories"
```

**Requirements**:
- ✅ Separate collections per user/session
- ✅ OR filter by session_id in metadata
- ⚠️ Collection-based isolation is ChromaDB-specific
- ⚠️ Can use metadata filtering instead

---

## Collection Management

### 6. Collections (Optional but Used)

#### 6.1 Multiple Collections
```python
# From LTM
self.collections: Dict[str, ChromaRetriever] = {}

def _get_collection(self, user_id, session_id):
    collection_name = self._get_collection_name(user_id, session_id)
    if collection_name not in self.collections:
        retriever = ChromaRetriever(collection_name=collection_name, ...)
        self.collections[collection_name] = retriever
    return self.collections[collection_name]
```

**Requirements**:
- ⚠️ Multiple named collections (like separate databases)
- ⚠️ Create collection on-demand
- ⚠️ List all collections
- ⚠️ Delete collection
- **Alternative**: Use single collection with metadata filtering

---

## Performance Requirements

### 7. Scale & Performance

#### 7.1 Expected Scale
```python
# From constants.py
DEFAULT_STM_CAPACITY = 20  # Short-term memory size
# LTM: Hundreds to thousands of memories per user
```

**Requirements**:
- ✅ Handle 20 items in STM (in-memory, fast)
- ✅ Handle 1,000-10,000 items in LTM per user
- ✅ Sub-second search latency
- ⚠️ Not designed for millions of documents

#### 7.2 Search Performance
```python
# From evaluation results (cortex-integration-analysis.md)
# Top-K 20: ~2-8s latency (with smart collections)
# Top-K 35: similar latency
```

**Requirements**:
- ✅ Search latency < 2s for typical queries
- ✅ Acceptable to be slower than pure HNSW
- ⚠️ Background processing acceptable for adds

---

## Persistence

### 8. Data Persistence

#### 8.1 Save/Load
```python
# From ChromaRetriever
def save(self, file_path, embeddings_path=None):
    data = {'content_hashes': self.content_hashes}
    with open(file_path, 'wb') as f:
        pickle.dump(data, f)

def load(self, file_path, embeddings_path=None):
    with open(file_path, 'rb') as f:
        data = pickle.load(f)
```

**Requirements**:
- ✅ Persist to disk
- ✅ Load from disk on restart
- ✅ Preserve all data (documents, metadata, embeddings)
- ⚠️ ChromaDB handles this automatically

---

## Summary: Must-Have vs Nice-to-Have

### Must-Have (Critical for Cortex)

| Requirement | Priority | Notes |
|-------------|----------|-------|
| Add document with custom ID | 🔴 Critical | Core operation |
| Get document by ID | 🔴 Critical | Direct retrieval |
| Delete document by ID | 🔴 Critical | Memory management |
| Vector search (cosine) | 🔴 Critical | Core search |
| Top-K results | 🔴 Critical | Limit results |
| Metadata storage | 🔴 Critical | All fields |
| Metadata filtering (equality) | 🔴 Critical | User isolation |
| AND filters | 🔴 Critical | Multi-condition |
| Persistence | 🔴 Critical | Survive restarts |

### Nice-to-Have (Can Work Around)

| Requirement | Priority | Workaround |
|-------------|----------|------------|
| Update document | 🟡 Medium | Delete + Add |
| Range filters (>, <) | 🟡 Medium | Post-filter |
| OR filters | 🟢 Low | Not used |
| Multiple collections | 🟢 Low | Use metadata |
| Efficient filtered search | 🟡 Medium | Post-filter acceptable |

---

## Database API Contract

### Minimal Interface Needed

```rust
pub trait CortexDatabase {
    // Document operations
    async fn add(&self, id: String, content: String, metadata: HashMap<String, Value>) -> Result<()>;
    async fn get(&self, id: &str) -> Result<Option<Document>>;
    async fn delete(&self, id: &str) -> Result<bool>;
    
    // Search operations
    async fn search(&self, query: &str, limit: usize, filter: Option<Filter>) -> Result<Vec<SearchResult>>;
    
    // Persistence
    async fn save(&self) -> Result<()>;
    async fn load(&self) -> Result<()>;
}

pub struct Filter {
    pub conditions: Vec<FilterCondition>,
    pub logic: FilterLogic,  // AND or OR
}

pub struct FilterCondition {
    pub field: String,
    pub operator: FilterOp,  // Eq, Gt, Lt, Contains
    pub value: Value,
}

pub struct Document {
    pub id: String,
    pub content: String,
    pub metadata: HashMap<String, Value>,
    pub embedding: Vec<f32>,
}

pub struct SearchResult {
    pub document: Document,
    pub score: f32,
    pub distance: f32,
}
```

---

## Next Steps

1. **Test `hnsw_rs` against this contract**:
   - Does it support all Must-Have operations?
   - What workarounds needed for Nice-to-Have?

2. **Identify gaps**:
   - What's missing?
   - What's inefficient?
   - What requires library changes?

3. **Design solution**:
   - Pure `hnsw_rs` with workarounds?
   - Switch to `hnswlib`?
   - Build simplified solution?
