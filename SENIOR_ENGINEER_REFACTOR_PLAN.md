# Senior Engineer Standards Refactor Plan

## Current State: Code Quality Assessment ❌

**Current Level**: Junior/Mid-level implementation  
**Target Level**: Senior Software Engineer standards  
**Status**: Major refactoring required

---

## Critical Issues to Address

### 🚨 **Issue 1: Fake/Stub Implementations**
**Current Problem**:
```rust
// TransitionManager - just counts, no real data
pub fn add_transition_record(&mut self, _from: ConversationMode, _to: ConversationMode, _confirmed: bool) -> bool {
    self.transition_count += 1;  // Fake implementation
    true
}
```

**Senior Standard Required**:
- Real data structures with proper storage
- Actual business logic implementation
- Meaningful state management

### 🚨 **Issue 2: Poor Error Handling**
**Current Problem**:
```rust
// Inconsistent error types
Result<bool, String>  // String errors
bool                  // No error handling
```

**Senior Standard Required**:
- Custom error types with proper hierarchy
- Consistent error handling strategy
- Meaningful error messages with context

### 🚨 **Issue 3: No Design Patterns**
**Current Problem**:
- Monolithic structs doing everything
- No abstractions or interfaces
- Tight coupling between components

**Senior Standard Required**:
- Strategy Pattern for different behaviors
- Repository Pattern for data access
- Builder Pattern for complex objects
- Dependency Injection for loose coupling

---

## Refactoring Plan

### Phase 1: Error Handling & Types (2-3 hours)

#### 1.1 Create Proper Error Types
```rust
#[derive(Debug, thiserror::Error)]
pub enum ConversationModeError {
    #[error("Invalid mode transition from {from:?} to {to:?}: {reason}")]
    InvalidTransition { from: ConversationMode, to: ConversationMode, reason: String },
    
    #[error("Configuration error: {0}")]
    Configuration(#[from] ConfigError),
    
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),
    
    #[error("Template not found: {name}")]
    TemplateNotFound { name: String },
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Invalid configuration format: {0}")]
    InvalidFormat(String),
    
    #[error("Missing required field: {field}")]
    MissingField { field: String },
}

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("Failed to read from storage: {0}")]
    ReadError(String),
    
    #[error("Failed to write to storage: {0}")]
    WriteError(String),
}
```

#### 1.2 Update All Function Signatures
```rust
// Before: inconsistent error handling
pub fn add_transition_record(...) -> bool
pub fn transition_with_confirmation(...) -> Result<bool, String>

// After: consistent error handling
pub fn add_transition_record(...) -> Result<(), ConversationModeError>
pub fn transition_with_confirmation(...) -> Result<TransitionResult, ConversationModeError>
```

### Phase 2: Proper Data Structures (3-4 hours)

#### 2.1 Real TransitionManager Implementation
```rust
#[derive(Debug)]
pub struct TransitionManager {
    transitions: VecDeque<ModeTransition>,
    max_history: usize,
    storage: Box<dyn TransitionStorage>,
}

#[derive(Debug, Clone)]
pub struct ModeTransition {
    pub id: TransitionId,
    pub from: ConversationMode,
    pub to: ConversationMode,
    pub trigger: ModeTransitionTrigger,
    pub timestamp: SystemTime,
    pub user_confirmed: bool,
    pub context: Option<String>,
}

impl TransitionManager {
    pub fn new(storage: Box<dyn TransitionStorage>) -> Self {
        Self {
            transitions: VecDeque::with_capacity(DEFAULT_MAX_HISTORY),
            max_history: DEFAULT_MAX_HISTORY,
            storage,
        }
    }
    
    pub fn add_transition(&mut self, transition: ModeTransition) -> Result<(), ConversationModeError> {
        // Real implementation with validation
        self.validate_transition(&transition)?;
        
        if self.transitions.len() >= self.max_history {
            self.transitions.pop_front();
        }
        
        self.transitions.push_back(transition.clone());
        self.storage.store_transition(&transition)?;
        
        Ok(())
    }
    
    pub fn get_recent_transitions(&self, limit: usize) -> Vec<&ModeTransition> {
        self.transitions.iter().rev().take(limit).collect()
    }
    
    pub fn can_undo_last_transition(&self) -> bool {
        self.transitions.back()
            .map(|t| !t.user_confirmed && t.trigger == ModeTransitionTrigger::Auto)
            .unwrap_or(false)
    }
}
```

#### 2.2 Real TemplateManager Implementation
```rust
#[derive(Debug)]
pub struct TemplateManager {
    templates: HashMap<String, ModeTemplate>,
    storage: Box<dyn TemplateStorage>,
}

impl TemplateManager {
    pub fn new(storage: Box<dyn TemplateStorage>) -> Result<Self, ConversationModeError> {
        let templates = storage.load_all_templates()?;
        Ok(Self { templates, storage })
    }
    
    pub fn add_template(&mut self, template: ModeTemplate) -> Result<(), ConversationModeError> {
        template.validate()?;
        
        if self.templates.contains_key(&template.name) {
            return Err(ConversationModeError::InvalidOperation {
                operation: "add_template".to_string(),
                reason: format!("Template '{}' already exists", template.name),
            });
        }
        
        self.storage.store_template(&template)?;
        self.templates.insert(template.name.clone(), template);
        
        Ok(())
    }
    
    pub fn get_template(&self, name: &str) -> Option<&ModeTemplate> {
        self.templates.get(name)
    }
    
    pub fn list_templates(&self) -> Vec<&ModeTemplate> {
        self.templates.values().collect()
    }
}
```

### Phase 3: Design Patterns Implementation (4-5 hours)

#### 3.1 Repository Pattern for Data Access
```rust
pub trait TransitionStorage: Send + Sync {
    fn store_transition(&mut self, transition: &ModeTransition) -> Result<(), StorageError>;
    fn load_transitions(&self, limit: usize) -> Result<Vec<ModeTransition>, StorageError>;
    fn clear_old_transitions(&mut self, before: SystemTime) -> Result<usize, StorageError>;
}

pub trait TemplateStorage: Send + Sync {
    fn store_template(&mut self, template: &ModeTemplate) -> Result<(), StorageError>;
    fn load_template(&self, name: &str) -> Result<Option<ModeTemplate>, StorageError>;
    fn load_all_templates(&self) -> Result<HashMap<String, ModeTemplate>, StorageError>;
    fn delete_template(&mut self, name: &str) -> Result<bool, StorageError>;
}

pub trait PreferenceStorage: Send + Sync {
    fn save_preferences(&mut self, prefs: &UserPreferences) -> Result<(), StorageError>;
    fn load_preferences(&self) -> Result<UserPreferences, StorageError>;
}

// Implementations
pub struct FileTransitionStorage {
    file_path: PathBuf,
}

pub struct InMemoryTransitionStorage {
    transitions: Vec<ModeTransition>,
}
```

#### 3.2 Strategy Pattern for Mode Detection
```rust
pub trait ModeDetectionStrategy: Send + Sync {
    fn detect_mode(&self, context: &str) -> Option<(ConversationMode, f32)>;
    fn get_confidence_threshold(&self) -> f32;
}

pub struct KeywordBasedDetection {
    patterns: HashMap<ConversationMode, Vec<String>>,
}

pub struct MLBasedDetection {
    model: Box<dyn MLModel>,
}

pub struct HybridDetection {
    strategies: Vec<Box<dyn ModeDetectionStrategy>>,
    weights: Vec<f32>,
}

pub struct ModeSuggestionEngine {
    strategy: Box<dyn ModeDetectionStrategy>,
    learning_enabled: bool,
}
```

#### 3.3 Builder Pattern for Complex Objects
```rust
pub struct ModeTransitionBuilder {
    from: Option<ConversationMode>,
    to: Option<ConversationMode>,
    trigger: Option<ModeTransitionTrigger>,
    context: Option<String>,
    user_confirmed: bool,
}

impl ModeTransitionBuilder {
    pub fn new() -> Self {
        Self {
            from: None,
            to: None,
            trigger: None,
            context: None,
            user_confirmed: false,
        }
    }
    
    pub fn from_mode(mut self, mode: ConversationMode) -> Self {
        self.from = Some(mode);
        self
    }
    
    pub fn to_mode(mut self, mode: ConversationMode) -> Self {
        self.to = Some(mode);
        self
    }
    
    pub fn with_trigger(mut self, trigger: ModeTransitionTrigger) -> Self {
        self.trigger = Some(trigger);
        self
    }
    
    pub fn with_context(mut self, context: String) -> Self {
        self.context = Some(context);
        self
    }
    
    pub fn user_confirmed(mut self) -> Self {
        self.user_confirmed = true;
        self
    }
    
    pub fn build(self) -> Result<ModeTransition, ConversationModeError> {
        let from = self.from.ok_or(ConversationModeError::MissingField { field: "from".to_string() })?;
        let to = self.to.ok_or(ConversationModeError::MissingField { field: "to".to_string() })?;
        let trigger = self.trigger.ok_or(ConversationModeError::MissingField { field: "trigger".to_string() })?;
        
        Ok(ModeTransition {
            id: TransitionId::new(),
            from,
            to,
            trigger,
            timestamp: SystemTime::now(),
            user_confirmed: self.user_confirmed,
            context: self.context,
        })
    }
}
```

### Phase 4: Configuration & Serialization Separation (2-3 hours)

#### 4.1 Separate Configuration Concerns
```rust
pub struct UserPreferences {
    pub default_mode: ConversationMode,
    pub auto_detection_enabled: bool,
    pub visual_indicators_enabled: bool,
    pub transition_confirmations: bool,
}

// Separate serialization logic
pub struct PreferenceSerializer;

impl PreferenceSerializer {
    pub fn to_toml(prefs: &UserPreferences) -> Result<String, ConfigError> {
        toml::to_string(prefs).map_err(|e| ConfigError::SerializationError(e.to_string()))
    }
    
    pub fn from_toml(content: &str) -> Result<UserPreferences, ConfigError> {
        toml::from_str(content).map_err(|e| ConfigError::DeserializationError(e.to_string()))
    }
    
    pub fn to_json(prefs: &UserPreferences) -> Result<String, ConfigError> {
        serde_json::to_string_pretty(prefs).map_err(|e| ConfigError::SerializationError(e.to_string()))
    }
    
    pub fn from_json(content: &str) -> Result<UserPreferences, ConfigError> {
        serde_json::from_str(content).map_err(|e| ConfigError::DeserializationError(e.to_string()))
    }
}

// Separate persistence logic
pub struct PreferenceManager {
    storage: Box<dyn PreferenceStorage>,
    serializer: PreferenceSerializer,
}
```

### Phase 5: Dependency Injection & Testing (2-3 hours)

#### 5.1 Dependency Injection Container
```rust
pub struct ConversationModeContainer {
    transition_storage: Box<dyn TransitionStorage>,
    template_storage: Box<dyn TemplateStorage>,
    preference_storage: Box<dyn PreferenceStorage>,
    detection_strategy: Box<dyn ModeDetectionStrategy>,
}

impl ConversationModeContainer {
    pub fn new() -> Self {
        Self {
            transition_storage: Box::new(FileTransitionStorage::new("~/.q/transitions.json")),
            template_storage: Box::new(FileTemplateStorage::new("~/.q/templates/")),
            preference_storage: Box::new(FilePreferenceStorage::new("~/.q/preferences.toml")),
            detection_strategy: Box::new(HybridDetection::default()),
        }
    }
    
    pub fn for_testing() -> Self {
        Self {
            transition_storage: Box::new(InMemoryTransitionStorage::new()),
            template_storage: Box::new(InMemoryTemplateStorage::new()),
            preference_storage: Box::new(InMemoryPreferenceStorage::new()),
            detection_strategy: Box::new(KeywordBasedDetection::default()),
        }
    }
    
    pub fn create_transition_manager(&self) -> TransitionManager {
        TransitionManager::new(self.transition_storage.clone())
    }
    
    pub fn create_template_manager(&self) -> Result<TemplateManager, ConversationModeError> {
        TemplateManager::new(self.template_storage.clone())
    }
}
```

#### 5.2 Proper Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_container() -> ConversationModeContainer {
        ConversationModeContainer::for_testing()
    }
    
    #[test]
    fn test_transition_manager_real_implementation() {
        let container = create_test_container();
        let mut manager = container.create_transition_manager();
        
        let transition = ModeTransitionBuilder::new()
            .from_mode(ConversationMode::Interactive)
            .to_mode(ConversationMode::ExecutePlan)
            .with_trigger(ModeTransitionTrigger::UserCommand)
            .build()
            .unwrap();
            
        assert!(manager.add_transition(transition).is_ok());
        assert_eq!(manager.get_recent_transitions(1).len(), 1);
    }
    
    #[test]
    fn test_template_manager_validation() {
        let container = create_test_container();
        let mut manager = container.create_template_manager().unwrap();
        
        let invalid_template = ModeTemplate {
            name: "".to_string(), // Invalid empty name
            description: "Test".to_string(),
            initial_mode: ConversationMode::Interactive,
        };
        
        assert!(manager.add_template(invalid_template).is_err());
    }
}
```

### Phase 6: Performance & Production Readiness (2-3 hours)

#### 6.1 Performance Optimizations
```rust
// Lazy loading
pub struct LazyTemplateManager {
    storage: Box<dyn TemplateStorage>,
    cache: RwLock<HashMap<String, ModeTemplate>>,
    loaded: AtomicBool,
}

// Async operations
impl TransitionManager {
    pub async fn add_transition_async(&mut self, transition: ModeTransition) -> Result<(), ConversationModeError> {
        // Async validation and storage
        self.validate_transition_async(&transition).await?;
        self.storage.store_transition_async(&transition).await?;
        
        // Update in-memory state
        self.add_to_memory(transition);
        
        Ok(())
    }
}

// Memory management
impl TransitionManager {
    pub fn cleanup_old_transitions(&mut self, retention_days: u32) -> Result<usize, ConversationModeError> {
        let cutoff = SystemTime::now() - Duration::from_secs(retention_days as u64 * 24 * 60 * 60);
        
        let removed_count = self.transitions.retain(|t| t.timestamp > cutoff);
        self.storage.clear_old_transitions(cutoff)?;
        
        Ok(removed_count)
    }
}
```

#### 6.2 Configuration Management
```rust
#[derive(Debug, Clone, Deserialize)]
pub struct ConversationModeConfig {
    pub max_transition_history: usize,
    pub auto_cleanup_days: u32,
    pub detection_confidence_threshold: f32,
    pub storage_backend: StorageBackend,
    pub performance: PerformanceConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PerformanceConfig {
    pub enable_caching: bool,
    pub cache_size: usize,
    pub async_operations: bool,
}

impl Default for ConversationModeConfig {
    fn default() -> Self {
        Self {
            max_transition_history: 100,
            auto_cleanup_days: 30,
            detection_confidence_threshold: 0.7,
            storage_backend: StorageBackend::File,
            performance: PerformanceConfig::default(),
        }
    }
}
```

---

## Implementation Timeline

### Week 1: Foundation (12-15 hours)
- **Phase 1**: Error Handling & Types (2-3h)
- **Phase 2**: Proper Data Structures (3-4h)
- **Phase 3**: Design Patterns (4-5h)
- **Phase 4**: Configuration Separation (2-3h)

### Week 2: Quality & Production (8-10 hours)
- **Phase 5**: Dependency Injection & Testing (2-3h)
- **Phase 6**: Performance & Production Readiness (2-3h)
- **Integration**: Update CLI integration (2-3h)
- **Documentation**: Update docs and examples (2h)

**Total Estimated Time**: 20-25 hours

---

## Success Criteria

### ✅ **Senior Engineer Standards Met When**:

**Architecture & Design**:
- ✅ Proper design patterns implemented (Strategy, Repository, Builder)
- ✅ Clear separation of concerns
- ✅ Dependency injection for loose coupling
- ✅ Proper abstractions with traits

**Code Quality**:
- ✅ Real implementations with actual business logic
- ✅ Consistent error handling with custom error types
- ✅ No magic numbers or hardcoded values
- ✅ Meaningful naming and documentation

**Testing**:
- ✅ Comprehensive unit tests with real implementations
- ✅ Integration tests with dependency injection
- ✅ Mock implementations for testing only

**Production Readiness**:
- ✅ Real data persistence with multiple backends
- ✅ Performance optimizations (caching, async)
- ✅ Configuration management
- ✅ Memory management and cleanup

**Maintainability**:
- ✅ Loose coupling between components
- ✅ Easy to extend and modify
- ✅ Clear interfaces and contracts

---

## Risk Mitigation

### **Risk 1**: Breaking Existing Integration
**Mitigation**: Implement adapter pattern to maintain backward compatibility

### **Risk 2**: Performance Impact
**Mitigation**: Implement lazy loading and caching strategies

### **Risk 3**: Complexity Increase
**Mitigation**: Provide simple factory methods for common use cases

---

## Next Steps

1. **Review and Approve Plan**: Ensure all senior engineer standards are addressed
2. **Begin Phase 1**: Start with error handling and types
3. **Incremental Implementation**: Implement one phase at a time with testing
4. **Integration Testing**: Ensure CLI integration works with new architecture
5. **Performance Testing**: Validate performance meets requirements

**Goal**: Transform junior-level stub code into production-ready, senior engineer quality implementation.
