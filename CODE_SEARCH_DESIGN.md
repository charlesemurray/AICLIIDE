# Code Search Tool Design Document

## Overview

A built-in code search tool for Q CLI that provides intelligent, fast search across codebases using hybrid text and semantic search capabilities. The tool leverages existing Q CLI infrastructure (cortex-memory, delegate agents) while providing graceful fallbacks and continuous learning.

## Problem Statement

Currently, Q CLI relies heavily on `execute_bash` commands (`grep`, `find`, `rg`) for code exploration, which:
- Requires LLM to construct complex bash commands
- Often produces incomplete or missed results
- Has no semantic understanding of code structure
- Repeats slow searches without caching

## Goals

### Primary Goals
- Replace most bash-based code searches with a dedicated tool
- Provide both text and semantic search capabilities
- Maintain fast response times through intelligent indexing
- Learn and improve from user interactions

### Secondary Goals
- Integrate seamlessly with existing Q CLI workflows
- Respect system resources and user privacy
- Support incremental workspace analysis
- Enable cross-file relationship understanding

## Architecture

### Core Components

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Code Search   │    │  Cortex Memory  │    │ Delegate Agents │
│      Tool       │◄──►│   (Embeddings)  │    │   (Background)  │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  Text Search    │    │  Vector Index   │    │ File Watcher    │
│   (Fallback)    │    │   (HNSW)        │    │  & Updater      │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### Tool Interface

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct CodeSearch {
    pub query: String,
    pub search_type: SearchType,
    pub path: Option<String>,
    pub file_types: Option<Vec<String>>,
    pub limit: Option<usize>,
    pub enable_indexing: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SearchType {
    Auto,     // Intelligent selection
    Text,     // Fast text search
    Semantic, // Meaning-based search
    Symbol,   // Functions/classes (future)
    File,     // Filename search
}
```

## Implementation Strategy

### Phase 1: MVP (Weeks 1-2)

**Core Functionality:**
- Basic text search using ripgrep as backend
- Simple file filtering (extensions, gitignore)
- Workspace-scoped storage in `.amazonq/code-index/`
- Fallback-first approach (no indexing by default)

**Key Features:**
```rust
// Immediate text search with optional indexing
async fn search_with_fallback(&self, query: &str) -> Result<InvokeOutput> {
    if self.index_exists() && self.index_is_fresh() {
        self.search_indexed(query).await
    } else {
        let results = self.text_search_fallback(query).await?;
        self.offer_indexing_if_beneficial(results)
    }
}
```

### Phase 2: Intelligent Indexing (Weeks 3-4)

**Enhanced Capabilities:**
- Cortex-memory integration for semantic search
- Background indexing agent using delegate system
- Smart file prioritization with LLM assistance
- Incremental index updates

**Multi-Agent Architecture:**
```rust
// Indexing Agent - Analyzes and builds initial index
// Maintenance Agent - Watches for changes and updates
// Search Coordinator - Manages search strategies
```

### Phase 3: Learning & Optimization (Weeks 5-6)

**Adaptive Features:**
- Search result quality tracking
- Fallback pattern analysis
- User feedback integration
- Performance optimization based on usage

## Storage Strategy

### Index Location
```
project-root/
├── .amazonq/
│   ├── code-index/
│   │   ├── embeddings.db      # Cortex memory database
│   │   ├── metadata.json      # Index metadata
│   │   ├── file-hashes.json   # Change detection
│   │   └── search-metrics.json # Learning data
│   └── ...
```

### Metadata Structure
```rust
#[derive(Serialize, Deserialize)]
struct IndexMetadata {
    created_at: SystemTime,
    last_updated: SystemTime,
    file_count: usize,
    index_version: String,
    workspace_hash: String,
    search_metrics: SearchMetrics,
}
```

## Resource Management

### Safety Limits
```rust
struct CodeSearchConfig {
    max_index_size_mb: u64,        // Default: 100MB
    max_memory_usage_mb: u64,      // Default: 200MB
    cpu_throttle_threshold: f32,   // Default: 0.8
    enable_sensitive_filtering: bool, // Default: true
    auto_index_threshold: usize,   // Files count for auto-indexing
}
```

### Performance Considerations
- **Memory**: Monitor embedding memory usage, implement LRU cache
- **CPU**: Background indexing with throttling during high system load
- **Disk**: Index size limits with automatic cleanup of old data
- **Network**: All processing local, no external API calls

## Security & Privacy

### Data Protection
- **Secret Filtering**: Detect and exclude API keys, tokens, passwords
- **Local Processing**: All embeddings generated locally using cortex-memory
- **Permission Respect**: Only index files user has read access to
- **Gitignore Integration**: Respect existing ignore patterns

### Implementation
```rust
fn filter_sensitive_content(content: &str) -> String {
    // Remove common secret patterns
    let patterns = [
        r"api[_-]?key\s*[:=]\s*['\"]?[\w-]+",
        r"password\s*[:=]\s*['\"]?[\w-]+",
        r"token\s*[:=]\s*['\"]?[\w-]+",
    ];
    // Apply filtering...
}
```

## Learning & Adaptation

### Metrics Collection
```rust
struct SearchMetrics {
    query_patterns: HashMap<String, u32>,
    index_hit_rate: f32,
    fallback_frequency: u32,
    user_satisfaction: Vec<bool>,
    performance_data: Vec<SearchPerformance>,
}
```

### Feedback Loops
- **Implicit**: Track which results users interact with
- **Explicit**: Occasional satisfaction surveys
- **Behavioral**: Monitor follow-up searches and bash fallbacks
- **Performance**: Response times and resource usage

### Adaptation Strategies
```rust
impl LearningSystem {
    async fn analyze_search_patterns(&self) -> Vec<Improvement> {
        // Identify frequently missed patterns
        // Suggest indexing priority changes
        // Recommend configuration adjustments
    }
    
    async fn update_indexing_strategy(&mut self, improvements: Vec<Improvement>) {
        // Apply learned optimizations
        // Update file filtering rules
        // Adjust search ranking algorithms
    }
}
```

## Integration Points

### Q CLI Tool Registration
```rust
// In tool_manager.rs
"code_search" => Tool::CodeSearch(serde_json::from_value::<CodeSearch>(value.args)?),

// In tools/mod.rs
pub enum Tool {
    // ... existing tools
    CodeSearch(CodeSearch),
}
```

### Tool Schema
```json
{
  "code_search": {
    "name": "code_search",
    "description": "Search through code and documentation files with intelligent indexing. Supports text, semantic, and symbol search with automatic fallback to reliable text search.",
    "input_schema": {
      "type": "object",
      "properties": {
        "query": {
          "type": "string", 
          "description": "Search query or pattern"
        },
        "search_type": {
          "type": "string",
          "enum": ["auto", "text", "semantic", "symbol", "file"],
          "default": "auto",
          "description": "Type of search to perform"
        }
      },
      "required": ["query"]
    }
  }
}
```

### Agent Configuration
```json
{
  "toolsSettings": {
    "code_search": {
      "maxIndexSizeMB": 100,
      "autoIndexThreshold": 1000,
      "enableSensitiveFiltering": true,
      "excludePatterns": ["target/", "node_modules/", "*.min.js"]
    }
  }
}
```

## Risk Mitigation

### Technical Risks
- **Index Corruption**: Validation and rebuild mechanisms
- **Resource Exhaustion**: Hard limits and graceful degradation
- **Performance Regression**: Always-available text search fallback
- **Cross-Platform Issues**: Extensive testing on Windows/Mac/Linux

### User Experience Risks
- **Slow Initial Experience**: Start with fast text search, enhance over time
- **Poor Search Quality**: Continuous learning and fallback strategies
- **Resource Conflicts**: Background processing with user activity detection
- **Privacy Concerns**: Local-only processing with clear data handling

### Operational Risks
- **Maintenance Burden**: Self-managing agents with automatic cleanup
- **Configuration Complexity**: Smart defaults with minimal required config
- **Debugging Difficulty**: Comprehensive logging and metrics collection

## Success Metrics

### Technical Metrics
- **Search Response Time**: <500ms for indexed searches, <2s for fallback
- **Index Build Time**: <30s for typical repositories (<1000 files)
- **Memory Usage**: <200MB peak during indexing
- **Accuracy**: >80% relevant results in top 10

### User Experience Metrics
- **Adoption Rate**: % of bash searches replaced by code_search
- **User Satisfaction**: Feedback scores and continued usage
- **Performance Impact**: System responsiveness during indexing
- **Error Rate**: Failed searches requiring manual intervention

### Business Metrics
- **Developer Productivity**: Reduced time spent on code exploration
- **Tool Reliability**: Decreased support requests related to search
- **Feature Usage**: Adoption of semantic vs text search modes

## Future Enhancements

### Short Term (3-6 months)
- **Symbol Search**: AST-based function/class search
- **Cross-Reference**: Find usages and definitions
- **Smart Suggestions**: Query completion and refinement
- **Performance Optimization**: Faster indexing and search

### Long Term (6-12 months)
- **Code-Specific Embeddings**: Fine-tuned models for code understanding
- **Architectural Search**: Find patterns and design concepts
- **Team Learning**: Shared indexing strategies across team members
- **IDE Integration**: Export search capabilities to external tools

## Conclusion

This design provides a robust, scalable code search solution that enhances Q CLI's capabilities while maintaining reliability through fallback mechanisms and continuous learning. The phased approach ensures immediate value while building toward advanced semantic search capabilities.

The key innovation is the hybrid approach: start with proven text search, enhance with semantic capabilities, and continuously learn from user behavior to optimize the experience over time.
