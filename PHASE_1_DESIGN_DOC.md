# Phase 1: Core Template System - Design Document

## Problem Statement

### Current System Analysis
The existing skill creation flow forces users to write system prompts from scratch, resulting in:
- **High failure rate**: 60% of manually created prompts fail on first use
- **Cognitive overload**: Users report "blank page syndrome" - don't know where to start
- **Quality inconsistency**: No validation or quality assurance for prompts
- **Time inefficiency**: 8+ minutes average creation time

### Technical Constraints
- **CLI responsiveness**: Operations must complete <100ms to maintain perceived performance
- **Memory footprint**: Cannot exceed 10MB additional memory usage
- **Local-first**: No external dependencies or network calls
- **Backward compatibility**: Cannot break existing `SkillCreationFlow` API

### Success Criteria
- Template-based prompts achieve >85% first-try success rate
- Template selection and customization completes in <3 minutes
- >70% user adoption of templates over manual entry

## System Architecture

### High-Level Design

```
┌─────────────────────────────────────────────────────────────┐
│                    SkillCreationFlow                        │
│  ┌─────────────────┐    ┌─────────────────────────────────┐ │
│  │ Manual Entry    │    │      Template System           │ │
│  │ (existing)      │    │                                 │ │
│  └─────────────────┘    │  ┌─────────────────────────────┐ │ │
│                         │  │     TemplateManager         │ │ │
│                         │  │                             │ │ │
│                         │  │ ┌─────────┐ ┌─────────────┐ │ │ │
│                         │  │ │Template │ │   Quality   │ │ │ │
│                         │  │ │Storage  │ │ Validator   │ │ │ │
│                         │  │ └─────────┘ └─────────────┘ │ │ │
│                         │  │                             │ │ │
│                         │  │ ┌─────────┐ ┌─────────────┐ │ │ │
│                         │  │ │Template │ │    Cache    │ │ │ │
│                         │  │ │Renderer │ │  Manager    │ │ │ │
│                         │  │ └─────────┘ └─────────────┘ │ │ │
│                         │  └─────────────────────────────┘ │ │
│                         └─────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### Core Components

#### TemplateManager
**Responsibility**: Orchestrate template operations and maintain system state
**Key Interfaces**:
```rust
pub trait TemplateManager {
    async fn list_available_templates(&self) -> Result<Vec<TemplateInfo>>;
    async fn load_template(&self, id: &str) -> Result<PromptTemplate>;
    async fn render_template(&self, template: &PromptTemplate, params: &HashMap<String, String>) -> Result<String>;
    fn validate_quality(&self, prompt: &str) -> QualityScore;
}
```

#### TemplateStorage
**Responsibility**: Abstract template persistence with fallback strategy
**Design Decision**: Hybrid storage model
- **Primary**: Embedded templates (compile-time, always available)
- **Secondary**: File-based user templates (runtime, optional)
- **Fallback**: Minimal hardcoded template (emergency)

**Rationale**: Embedded templates ensure system always works even with filesystem issues, while file-based storage enables user customization.

#### QualityValidator  
**Responsibility**: Assess prompt quality using multi-dimensional scoring
**Algorithm**: Weighted component scoring
```rust
QualityScore = Σ(component_score * weight) where:
- role_clarity: 0.30 (most critical for prompt effectiveness)
- constraint_balance: 0.25 (prevents over/under-specification)  
- example_quality: 0.20 (essential for user understanding)
- capability_coverage: 0.15 (scope definition)
- coherence: 0.10 (readability)
```

#### TemplateRenderer
**Responsibility**: Safe parameter substitution with injection prevention
**Security Model**: 
- Whitelist-based parameter validation
- HTML/template injection prevention
- Bounded execution (timeout, recursion limits)

#### CacheManager
**Responsibility**: Performance optimization through intelligent caching
**Strategy**: Two-tier caching
- **L1**: In-memory LRU cache (hot templates, <1ms access)
- **L2**: Disk-based cache (warm templates, <10ms access)

## Technical Design Decisions

### ADR-001: Template Storage Architecture

**Problem**: How to store templates reliably while supporting customization?

**Alternatives Considered**:
1. **Pure file-based**: Simple but fragile (corruption, missing files)
2. **Pure embedded**: Reliable but inflexible (no customization)  
3. **Database**: Overkill for CLI tool, adds complexity
4. **Hybrid embedded + file** (CHOSEN)

**Decision Rationale**:
- Embedded templates guarantee system functionality
- File-based templates enable user customization
- Graceful degradation when files unavailable
- Minimal complexity increase

**Trade-offs**:
- ✅ Reliability: Always have working templates
- ✅ Flexibility: Support user customization
- ❌ Binary size: +2MB for embedded templates
- ❌ Complexity: Multiple code paths

### ADR-002: Quality Validation Approach

**Problem**: How to automatically assess prompt quality?

**Alternatives Considered**:
1. **Simple heuristics**: Fast but inaccurate
2. **ML-based scoring**: Accurate but complex, requires training data
3. **Rule-based validation**: Predictable but rigid
4. **Multi-dimensional weighted scoring** (CHOSEN)

**Decision Rationale**:
- Balances accuracy with maintainability
- Configurable weights allow tuning without code changes
- Deterministic results (same input = same score)
- Fast execution (<10ms)

**Implementation**:
```rust
pub struct QualityValidator {
    weights: QualityWeights,
    rules: Vec<Box<dyn ValidationRule>>,
}

pub trait ValidationRule: Send + Sync {
    fn evaluate(&self, template: &PromptTemplate) -> f64; // 0.0-1.0
    fn feedback(&self, template: &PromptTemplate) -> Option<String>;
}
```

### ADR-003: Template Parameter System

**Problem**: How to make templates customizable while maintaining safety?

**Alternatives Considered**:
1. **String interpolation**: Simple but unsafe (injection attacks)
2. **Jinja2-style templating**: Powerful but heavy dependency
3. **Custom DSL**: Maximum flexibility but high maintenance
4. **Mustache + type validation** (CHOSEN)

**Decision Rationale**:
- Familiar syntax (Mustache widely known)
- Type safety prevents runtime errors
- Injection-safe by design
- Minimal dependencies

**Security Model**:
```rust
pub enum ParameterType {
    String { max_length: usize, allowed_chars: CharSet },
    Enum { options: Vec<String> },
    Number { min: f64, max: f64 },
    Boolean,
}

// All parameters validated before substitution
fn validate_parameter(value: &str, param_type: &ParameterType) -> Result<String>;
```

## Interface Design

### Public API Contract

```rust
// Main entry point - maintains backward compatibility
pub struct PromptSystem {
    manager: TemplateManager,
    validator: QualityValidator,
}

impl PromptSystem {
    // Core operations
    pub async fn new() -> Result<Self>;
    pub async fn list_templates(&self) -> Result<Vec<TemplateInfo>>;
    pub async fn get_template(&self, id: &str) -> Result<PromptTemplate>;
    pub async fn render_template(&self, template: &PromptTemplate, params: &HashMap<String, String>) -> Result<String>;
    pub fn validate_prompt(&self, prompt: &str) -> QualityScore;
    
    // Integration points
    pub async fn suggest_templates_for_use_case(&self, use_case: &str) -> Result<Vec<TemplateInfo>>;
    pub fn get_template_parameters(&self, template: &PromptTemplate) -> Vec<ParameterInfo>;
}

// Template metadata for UI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: TemplateCategory,
    pub difficulty: DifficultyLevel,
    pub estimated_quality: f64,
    pub usage_stats: UsageStats,
}

// Quality assessment result
#[derive(Debug, Clone)]
pub struct QualityScore {
    pub overall_score: f64,        // 0.0-5.0
    pub component_scores: HashMap<String, f64>,
    pub feedback: Vec<QualityFeedback>,
    pub confidence: f64,           // How confident we are in this score
}

#[derive(Debug, Clone)]
pub struct QualityFeedback {
    pub severity: FeedbackSeverity,
    pub message: String,
    pub suggestion: Option<String>,
}
```

### Integration Contract

```rust
// How SkillCreationFlow will integrate
impl SkillCreationFlow {
    async fn create_prompt_with_template_system(&mut self) -> Result<String> {
        let prompt_system = PromptSystem::new().await?;
        
        // 1. Show template options
        let templates = prompt_system.list_templates().await?;
        let selected = self.ui.select_template(&templates)?;
        
        // 2. Customize template parameters
        let template = prompt_system.get_template(&selected.id).await?;
        let params = self.ui.collect_template_parameters(&template)?;
        
        // 3. Render and validate
        let rendered = prompt_system.render_template(&template, &params).await?;
        let quality = prompt_system.validate_prompt(&rendered);
        
        // 4. Show preview and get confirmation
        self.ui.show_prompt_preview(&rendered, &quality)?;
        if self.ui.confirm("Use this prompt?")? {
            Ok(rendered)
        } else {
            // Allow iteration or fallback to manual
            self.create_prompt_with_template_system().await
        }
    }
}
```

## Performance Design

### Latency Requirements
- Template listing: <10ms (cached), <50ms (cold)
- Template loading: <20ms (cached), <100ms (cold)  
- Template rendering: <5ms
- Quality validation: <10ms

### Memory Management
```rust
// Memory-efficient caching strategy
pub struct CacheManager {
    // L1: Hot templates in memory (max 5MB)
    memory_cache: LruCache<String, Arc<PromptTemplate>>,
    
    // L2: Warm templates on disk (max 50MB)  
    disk_cache: DiskCache,
    
    // Memory pressure handling
    max_memory_usage: usize,
    current_memory_usage: AtomicUsize,
}

impl CacheManager {
    // Evict under memory pressure
    fn ensure_memory_limit(&mut self) {
        while self.current_memory_usage.load(Ordering::Relaxed) > self.max_memory_usage {
            if let Some((_, template)) = self.memory_cache.pop_lru() {
                self.current_memory_usage.fetch_sub(
                    estimate_template_size(&template),
                    Ordering::Relaxed
                );
            } else {
                break; // Cache empty
            }
        }
    }
}
```

### Async Design
All I/O operations are non-blocking:
```rust
// Template loading pipeline
async fn load_template_pipeline(id: &str) -> Result<PromptTemplate> {
    // Check memory cache (synchronous, <1ms)
    if let Some(template) = memory_cache.get(id) {
        return Ok(template.clone());
    }
    
    // Check disk cache (async, <10ms)
    if let Some(template) = disk_cache.load(id).await? {
        memory_cache.insert(id, template.clone());
        return Ok(template);
    }
    
    // Load from source (async, <100ms)
    let template = load_from_source(id).await?;
    
    // Cache for future use
    tokio::spawn(async move {
        disk_cache.store(id, &template).await;
    });
    
    memory_cache.insert(id, template.clone());
    Ok(template)
}
```

## Security Design

### Threat Model
1. **Template injection attacks**: Malicious parameters in template rendering
2. **Path traversal**: Accessing files outside template directory
3. **DoS attacks**: Resource exhaustion through large inputs
4. **Data exfiltration**: Templates accessing sensitive information

### Security Controls

#### Input Validation
```rust
// All user input sanitized before use
fn sanitize_parameter(value: &str, param_type: &ParameterType) -> Result<String> {
    match param_type {
        ParameterType::String { max_length, allowed_chars } => {
            let sanitized: String = value
                .chars()
                .filter(|c| allowed_chars.contains(*c))
                .take(*max_length)
                .collect();
            Ok(sanitized)
        }
        ParameterType::Enum { options } => {
            if options.contains(&value.to_string()) {
                Ok(value.to_string())
            } else {
                Err(SecurityError::InvalidEnumValue)
            }
        }
        // ... other types
    }
}
```

#### Resource Limits
```rust
// Prevent resource exhaustion
const MAX_TEMPLATE_SIZE: usize = 100_000;      // 100KB per template
const MAX_RENDER_TIME: Duration = Duration::from_millis(100);
const MAX_CACHE_SIZE: usize = 10_000_000;      // 10MB total cache
const MAX_PARAMETER_LENGTH: usize = 1000;      // 1KB per parameter
```

#### File System Security
```rust
// Prevent path traversal
fn validate_template_path(path: &Path) -> Result<PathBuf> {
    let canonical = path.canonicalize()
        .map_err(|_| SecurityError::InvalidPath)?;
    
    let template_dir = get_template_directory().canonicalize()
        .map_err(|_| SecurityError::ConfigurationError)?;
    
    if !canonical.starts_with(&template_dir) {
        return Err(SecurityError::PathTraversal);
    }
    
    Ok(canonical)
}
```

## Error Handling & Resilience

### Error Categories
```rust
#[derive(Debug, thiserror::Error)]
pub enum TemplateError {
    #[error("Template not found: {id}")]
    NotFound { id: String },
    
    #[error("Template validation failed: {reason}")]
    ValidationFailed { reason: String },
    
    #[error("Template rendering failed: {reason}")]
    RenderingFailed { reason: String },
    
    #[error("Security violation: {violation}")]
    SecurityViolation { violation: String },
    
    #[error("System error: {source}")]
    SystemError { source: Box<dyn std::error::Error + Send + Sync> },
}
```

### Failure Recovery Strategy
```rust
// Graceful degradation with multiple fallback levels
async fn load_template_with_fallbacks(id: &str) -> Result<PromptTemplate> {
    // Level 1: Try primary source
    match load_template_primary(id).await {
        Ok(template) => return Ok(template),
        Err(TemplateError::NotFound { .. }) => {
            // Level 2: Try similar template
            if let Ok(similar) = find_similar_template(id).await {
                return Ok(similar);
            }
        }
        Err(TemplateError::ValidationFailed { .. }) => {
            // Level 3: Load without validation (degraded mode)
            if let Ok(template) = load_template_unsafe(id).await {
                return Ok(template);
            }
        }
        Err(_) => {} // Continue to final fallback
    }
    
    // Level 4: Emergency fallback - basic template
    Ok(create_emergency_template())
}
```

## Data Models

### Core Template Structure
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    // Identity
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: u32,
    
    // Classification
    pub category: TemplateCategory,
    pub difficulty: DifficultyLevel,
    pub tags: Vec<String>,
    
    // Content
    pub role: String,
    pub capabilities: Vec<String>,
    pub constraints: Vec<String>,
    pub context: Option<String>,
    
    // Customization
    pub parameters: Vec<TemplateParameter>,
    
    // Learning aids
    pub examples: Vec<ExampleConversation>,
    pub quality_indicators: Vec<String>,
    
    // Metadata
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub usage_stats: UsageStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateParameter {
    pub name: String,
    pub param_type: ParameterType,
    pub description: String,
    pub default_value: Option<String>,
    pub required: bool,
    pub validation_rules: Vec<ValidationRule>,
}
```

## Migration & Compatibility

### Backward Compatibility Strategy
- Existing `SkillCreationFlow` API unchanged
- Template system is additive - manual entry still available
- Graceful fallback if template system fails

### Data Migration
```rust
// Handle template format evolution
#[derive(Debug, Serialize, Deserialize)]
struct TemplateFile {
    version: u32,
    templates: Vec<PromptTemplate>,
}

impl TemplateFile {
    fn migrate_to_current_version(mut self) -> Result<Self> {
        while self.version < CURRENT_VERSION {
            self = match self.version {
                1 => self.migrate_v1_to_v2()?,
                2 => self.migrate_v2_to_v3()?,
                _ => return Err(MigrationError::UnsupportedVersion(self.version)),
            };
        }
        Ok(self)
    }
}
```

## Monitoring & Observability

### Key Metrics
- **Performance**: Template load times, cache hit rates, rendering times
- **Quality**: Template quality scores, user satisfaction ratings
- **Usage**: Template adoption rates, most popular templates
- **Errors**: Error rates by category, failure recovery success

### Instrumentation Points
```rust
// Metrics collection with minimal overhead
pub struct TemplateMetrics {
    load_times: Histogram,
    cache_hits: Counter,
    quality_scores: Histogram,
    error_rates: Counter,
}

// Async metrics collection to avoid blocking operations
impl TemplateManager {
    async fn load_template_instrumented(&self, id: &str) -> Result<PromptTemplate> {
        let start = Instant::now();
        let result = self.load_template(id).await;
        
        // Record metrics asynchronously
        let duration = start.elapsed();
        tokio::spawn(async move {
            METRICS.load_times.record(duration.as_millis() as f64);
            match result {
                Ok(_) => METRICS.cache_hits.increment(1),
                Err(ref e) => METRICS.error_rates.increment(1),
            }
        });
        
        result
    }
}
```

This design provides a robust, secure, and performant foundation for the template system while maintaining backward compatibility and enabling future extensibility.
