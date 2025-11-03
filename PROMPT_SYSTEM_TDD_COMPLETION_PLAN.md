# Prompt System TDD Completion Plan

## Overview
Complete the stubbed/placeholder implementations in prompt_system using strict TDD methodology.
Each step must: compile, pass tests, and implement real functionality (no placeholders).

## Current Gaps Identified

1. **QualityValidator** - Returns hardcoded fake scores
2. **CacheManager** - No-op implementation, doesn't cache
3. **TemplateRenderer** - Ignores parameters completely
4. **Similarity Matching** - Just returns first template
5. **Metrics Collection** - Placeholder module only
6. **Test Case Management** - Placeholder module only

---

## Phase 1: Quality Validator (Real Implementation)
**Goal**: Replace fake validator with actual quality scoring
**Estimated Time**: 4 hours

### Step 1.1: Define Quality Metrics (30 min)
**Test First**:
```rust
#[test]
fn test_quality_validator_checks_role_clarity() {
    let validator = MultiDimensionalValidator::new();
    let clear_role = "You are an expert code reviewer specializing in Rust.";
    let vague_role = "You help.";
    
    let clear_score = validator.validate(clear_role);
    let vague_score = validator.validate(vague_role);
    
    assert!(clear_score.overall_score > vague_score.overall_score);
    assert!(clear_score.component_scores.contains_key("role_clarity"));
}
```

**Implementation**:
- Add `role_clarity` metric: checks length, specificity, domain keywords
- Score 0.0-1.0 based on word count, technical terms, specificity markers
- Must return actual calculated score, not hardcoded

**Validation**:
- [ ] Test passes
- [ ] `cargo test quality_validator` passes
- [ ] No hardcoded scores in implementation
- [ ] Score varies with different inputs

### Step 1.2: Add Capability Completeness Check (30 min)
**Test First**:
```rust
#[test]
fn test_quality_validator_checks_capabilities() {
    let validator = MultiDimensionalValidator::new();
    let detailed = "Capabilities:\n- Analyze code\n- Find bugs\n- Suggest fixes";
    let minimal = "Capabilities:\n- Help";
    
    let detailed_score = validator.validate(detailed);
    let minimal_score = validator.validate(minimal);
    
    assert!(detailed_score.component_scores["capability_completeness"] > 
            minimal_score.component_scores["capability_completeness"]);
}
```

**Implementation**:
- Parse capabilities section
- Count specific, actionable capabilities
- Score based on quantity and specificity
- Add to component_scores map

**Validation**:
- [ ] Test passes
- [ ] Scores reflect actual capability count
- [ ] Component score is in 0.0-1.0 range

### Step 1.3: Add Constraint Validation (30 min)
**Test First**:
```rust
#[test]
fn test_quality_validator_checks_constraints() {
    let validator = MultiDimensionalValidator::new();
    let with_constraints = "Constraints:\n- Be concise\n- Cite sources\n- Avoid speculation";
    let without = "Do your best.";
    
    let with_score = validator.validate(with_constraints);
    let without_score = validator.validate(without);
    
    assert!(with_score.component_scores["constraint_clarity"] > 
            without_score.component_scores["constraint_clarity"]);
}
```

**Implementation**:
- Parse constraints section
- Validate constraints are specific and measurable
- Score based on clarity and enforceability

**Validation**:
- [ ] Test passes
- [ ] Constraint scoring is accurate
- [ ] Edge cases handled (no constraints, malformed)

### Step 1.4: Add Example Quality Check (30 min)
**Test First**:
```rust
#[test]
fn test_quality_validator_checks_examples() {
    let validator = MultiDimensionalValidator::new();
    let with_examples = "Examples:\nInput: Review this code\nOutput: Here's my analysis...";
    let without = "No examples provided.";
    
    let with_score = validator.validate(with_examples);
    let without_score = validator.validate(without);
    
    assert!(with_score.component_scores["example_quality"] > 
            without_score.component_scores["example_quality"]);
}
```

**Implementation**:
- Detect example sections
- Validate input/output pairs are present
- Score based on example completeness and relevance

**Validation**:
- [ ] Test passes
- [ ] Example detection works
- [ ] Scoring reflects example quality

### Step 1.5: Implement Overall Score Calculation (30 min)
**Test First**:
```rust
#[test]
fn test_quality_validator_overall_score() {
    let validator = MultiDimensionalValidator::new();
    let high_quality = "You are an expert Rust developer.\nCapabilities:\n- Review code\n- Find bugs\nConstraints:\n- Be specific\nExamples:\nInput: x\nOutput: y";
    let low_quality = "Help with stuff.";
    
    let high_score = validator.validate(high_quality);
    let low_score = validator.validate(low_quality);
    
    assert!(high_score.overall_score > 0.7);
    assert!(low_score.overall_score < 0.4);
    assert_eq!(high_score.component_scores.len(), 4); // All components scored
}
```

**Implementation**:
- Weighted average of component scores
- Weights: role_clarity (30%), capabilities (25%), constraints (25%), examples (20%)
- Overall score = sum(component * weight)

**Validation**:
- [ ] Test passes
- [ ] Overall score is weighted average
- [ ] All component scores present
- [ ] Score range is 0.0-1.0

### Step 1.6: Add Quality Feedback Generation (1 hour)
**Test First**:
```rust
#[test]
fn test_quality_validator_provides_feedback() {
    let validator = MultiDimensionalValidator::new();
    let vague_prompt = "You help with things.";
    
    let score = validator.validate(vague_prompt);
    
    assert!(!score.feedback.is_empty());
    assert!(score.feedback.iter().any(|f| 
        f.message.contains("role") && f.severity == FeedbackSeverity::Warning
    ));
    assert!(score.feedback.iter().any(|f| f.suggestion.is_some()));
}
```

**Implementation**:
- Generate feedback for each low-scoring component
- Provide specific suggestions for improvement
- Categorize by severity (Error, Warning, Info)
- Include actionable recommendations

**Validation**:
- [ ] Test passes
- [ ] Feedback is specific and actionable
- [ ] Suggestions are relevant
- [ ] Severity levels are appropriate

---

## Phase 2: Template Renderer (Parameter Substitution)
**Goal**: Implement actual parameter rendering with validation
**Estimated Time**: 3 hours

### Step 2.1: Basic Parameter Substitution (45 min)
**Test First**:
```rust
#[test]
async fn test_renderer_substitutes_simple_params() {
    let renderer = SafeTemplateRenderer::new();
    let template = create_test_template_with_params();
    let mut params = HashMap::new();
    params.insert("language".to_string(), "Rust".to_string());
    
    let result = renderer.render(&template, &params).await.unwrap();
    
    assert!(result.contains("Rust"));
    assert!(!result.contains("{{language}}"));
}
```

**Implementation**:
- Parse template for `{{param_name}}` placeholders
- Replace with values from params HashMap
- Handle missing parameters gracefully

**Validation**:
- [ ] Test passes
- [ ] All parameters substituted
- [ ] Template structure preserved
- [ ] No placeholder syntax remains

### Step 2.2: Parameter Type Validation (45 min)
**Test First**:
```rust
#[test]
async fn test_renderer_validates_enum_params() {
    let renderer = SafeTemplateRenderer::new();
    let template = create_template_with_enum_param(); // language: [rust, python, java]
    let mut params = HashMap::new();
    params.insert("language".to_string(), "cobol".to_string()); // Invalid
    
    let result = renderer.render(&template, &params).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid value for language"));
}
```

**Implementation**:
- Validate enum parameters against allowed options
- Validate number parameters against min/max
- Validate string parameters against max_length
- Return descriptive errors for invalid values

**Validation**:
- [ ] Test passes
- [ ] All parameter types validated
- [ ] Error messages are clear
- [ ] Valid values pass through

### Step 2.3: Required Parameter Checking (30 min)
**Test First**:
```rust
#[test]
async fn test_renderer_requires_mandatory_params() {
    let renderer = SafeTemplateRenderer::new();
    let template = create_template_with_required_param();
    let params = HashMap::new(); // Missing required param
    
    let result = renderer.render(&template, &params).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Required parameter"));
}
```

**Implementation**:
- Check all required parameters are present
- Return error listing missing parameters
- Apply defaults for optional parameters

**Validation**:
- [ ] Test passes
- [ ] Missing required params detected
- [ ] Defaults applied correctly
- [ ] Error lists all missing params

### Step 2.4: Safe Rendering (Injection Prevention) (1 hour)
**Test First**:
```rust
#[test]
async fn test_renderer_prevents_injection() {
    let renderer = SafeTemplateRenderer::new();
    let template = create_test_template();
    let mut params = HashMap::new();
    params.insert("input".to_string(), "{{role}}".to_string()); // Nested template
    
    let result = renderer.render(&template, &params).await.unwrap();
    
    assert!(result.contains("{{role}}")); // Should be escaped, not evaluated
    assert!(!result.contains(template.role)); // Should not substitute nested
}
```

**Implementation**:
- Escape parameter values to prevent nested substitution
- Sanitize special characters
- Prevent recursive template expansion
- Add max recursion depth check

**Validation**:
- [ ] Test passes
- [ ] Nested templates not evaluated
- [ ] Special chars escaped
- [ ] No infinite recursion possible

---

## Phase 3: Cache Manager (Real Caching)
**Goal**: Implement actual in-memory caching with TTL
**Estimated Time**: 2.5 hours

### Step 3.1: In-Memory Cache Storage (45 min)
**Test First**:
```rust
#[tokio::test]
async fn test_cache_stores_and_retrieves() {
    let cache = TwoTierCacheManager::new();
    let template = create_test_template();
    
    cache.put("test_id", &template).await.unwrap();
    let retrieved = cache.get("test_id").await.unwrap();
    
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().id, template.id);
}
```

**Implementation**:
- Use `Arc<Mutex<HashMap<String, CacheEntry>>>` for thread-safe storage
- Store template with timestamp
- Implement get/put operations

**Validation**:
- [ ] Test passes
- [ ] Thread-safe access works
- [ ] Multiple gets return same data
- [ ] No data races

### Step 3.2: TTL (Time-To-Live) Implementation (45 min)
**Test First**:
```rust
#[tokio::test]
async fn test_cache_expires_old_entries() {
    let cache = TwoTierCacheManager::with_ttl(Duration::from_millis(100));
    let template = create_test_template();
    
    cache.put("test_id", &template).await.unwrap();
    tokio::time::sleep(Duration::from_millis(150)).await;
    
    let retrieved = cache.get("test_id").await.unwrap();
    assert!(retrieved.is_none()); // Should be expired
}
```

**Implementation**:
- Add timestamp to CacheEntry
- Check age on get()
- Return None if expired
- Configurable TTL duration

**Validation**:
- [ ] Test passes
- [ ] Expired entries return None
- [ ] Fresh entries return Some
- [ ] TTL is configurable

### Step 3.3: Cache Eviction (LRU) (1 hour)
**Test First**:
```rust
#[tokio::test]
async fn test_cache_evicts_lru_when_full() {
    let cache = TwoTierCacheManager::with_capacity(2);
    
    cache.put("id1", &create_test_template()).await.unwrap();
    cache.put("id2", &create_test_template()).await.unwrap();
    cache.put("id3", &create_test_template()).await.unwrap(); // Should evict id1
    
    assert!(cache.get("id1").await.unwrap().is_none());
    assert!(cache.get("id2").await.unwrap().is_some());
    assert!(cache.get("id3").await.unwrap().is_some());
}
```

**Implementation**:
- Track access order with linked list or VecDeque
- Evict least recently used when at capacity
- Update access time on get()
- Configurable max capacity

**Validation**:
- [ ] Test passes
- [ ] LRU eviction works correctly
- [ ] Access updates order
- [ ] Capacity limit enforced

---

## Phase 4: Similarity Matching (Real Algorithm)
**Goal**: Implement actual similarity scoring
**Estimated Time**: 2 hours

### Step 4.1: Tag-Based Similarity (45 min)
**Test First**:
```rust
#[tokio::test]
async fn test_find_similar_by_tags() {
    let manager = DefaultTemplateManager::new().await.unwrap();
    let target = create_template_with_tags(vec!["code", "rust", "review"]);
    
    let similar = manager.find_similar_template(&target.id).await.unwrap();
    
    assert_ne!(similar.id, target.id);
    assert!(similar.tags.iter().any(|t| target.tags.contains(t))); // Shares tags
}
```

**Implementation**:
- Calculate Jaccard similarity on tags
- Score = |intersection| / |union|
- Return template with highest tag overlap

**Validation**:
- [ ] Test passes
- [ ] Returns different template
- [ ] Similarity based on tags
- [ ] Handles no matches gracefully

### Step 4.2: Category and Difficulty Matching (45 min)
**Test First**:
```rust
#[tokio::test]
async fn test_find_similar_by_category() {
    let manager = DefaultTemplateManager::new().await.unwrap();
    let target = create_template(TemplateCategory::CodeReviewer, DifficultyLevel::Advanced);
    
    let similar = manager.find_similar_template(&target.id).await.unwrap();
    
    assert_eq!(similar.category, target.category);
    // Difficulty should be close (within 1 level)
}
```

**Implementation**:
- Prioritize same category (weight: 0.5)
- Score difficulty proximity (weight: 0.2)
- Combine with tag similarity (weight: 0.3)
- Return highest combined score

**Validation**:
- [ ] Test passes
- [ ] Category matching works
- [ ] Difficulty proximity calculated
- [ ] Combined scoring accurate

### Step 4.3: Fallback to Emergency Template (30 min)
**Test First**:
```rust
#[tokio::test]
async fn test_fallback_when_no_similar() {
    let manager = DefaultTemplateManager::new().await.unwrap();
    // Clear all templates except one
    
    let similar = manager.find_similar_template("only_template").await.unwrap();
    
    assert_eq!(similar.id, "emergency");
    assert!(similar.name.contains("Basic"));
}
```

**Implementation**:
- Return emergency template if no templates available
- Return emergency if similarity score < 0.1
- Log fallback usage for monitoring

**Validation**:
- [ ] Test passes
- [ ] Emergency template returned
- [ ] Fallback logged
- [ ] Never panics

---

## Phase 5: Metrics Collection (Real Implementation)
**Goal**: Implement actual metrics tracking
**Estimated Time**: 3 hours

### Step 5.1: Usage Tracking (1 hour)
**Test First**:
```rust
#[tokio::test]
async fn test_metrics_tracks_usage() {
    let metrics = MetricsCollector::new();
    
    metrics.record_usage("template_id").await;
    metrics.record_usage("template_id").await;
    
    let stats = metrics.get_stats("template_id").await.unwrap();
    assert_eq!(stats.usage_count, 2);
}
```

**Implementation**:
- Store usage count per template
- Thread-safe increment
- Persist to disk periodically
- Load on startup

**Validation**:
- [ ] Test passes
- [ ] Count increments correctly
- [ ] Thread-safe
- [ ] Persists across restarts

### Step 5.2: Success Rate Tracking (1 hour)
**Test First**:
```rust
#[tokio::test]
async fn test_metrics_tracks_success_rate() {
    let metrics = MetricsCollector::new();
    
    metrics.record_outcome("template_id", true).await;
    metrics.record_outcome("template_id", false).await;
    metrics.record_outcome("template_id", true).await;
    
    let stats = metrics.get_stats("template_id").await.unwrap();
    assert_eq!(stats.success_rate, 0.666, epsilon = 0.01);
}
```

**Implementation**:
- Track success/failure outcomes
- Calculate rolling success rate
- Store last N outcomes (e.g., 100)
- Update on each outcome

**Validation**:
- [ ] Test passes
- [ ] Success rate calculated correctly
- [ ] Rolling window works
- [ ] Handles edge cases (0 outcomes)

### Step 5.3: Quality Satisfaction Tracking (1 hour)
**Test First**:
```rust
#[tokio::test]
async fn test_metrics_tracks_satisfaction() {
    let metrics = MetricsCollector::new();
    
    metrics.record_satisfaction("template_id", 4.5).await;
    metrics.record_satisfaction("template_id", 3.5).await;
    
    let stats = metrics.get_stats("template_id").await.unwrap();
    assert_eq!(stats.avg_satisfaction, 4.0);
}
```

**Implementation**:
- Accept satisfaction scores (1.0-5.0)
- Calculate running average
- Store last N scores
- Update average on each score

**Validation**:
- [ ] Test passes
- [ ] Average calculated correctly
- [ ] Score validation (1.0-5.0)
- [ ] Handles no scores

---

## Phase 6: Test Case Management (Real Implementation)
**Goal**: Implement test case execution and validation
**Estimated Time**: 3 hours

### Step 6.1: Test Case Storage (45 min)
**Test First**:
```rust
#[tokio::test]
async fn test_case_manager_stores_cases() {
    let manager = TestCaseManager::new(temp_dir());
    let test_case = create_test_case();
    
    manager.save_test_case(&test_case).await.unwrap();
    let loaded = manager.load_test_case(&test_case.id).await.unwrap();
    
    assert_eq!(loaded.id, test_case.id);
}
```

**Implementation**:
- Save test cases as JSON files
- Load from disk
- List all test cases
- Delete test cases

**Validation**:
- [ ] Test passes
- [ ] Files created correctly
- [ ] JSON format valid
- [ ] Load/save roundtrip works

### Step 6.2: Test Case Execution (1 hour)
**Test First**:
```rust
#[tokio::test]
async fn test_case_executor_runs_tests() {
    let executor = TestCaseExecutor::new();
    let test_case = create_keyword_test_case();
    let template = create_test_template();
    
    let result = executor.execute(&test_case, &template).await.unwrap();
    
    assert!(result.passed);
    assert!(result.details.is_some());
}
```

**Implementation**:
- Execute test case against template
- Check ExpectedOutput conditions
- Return TestResult with pass/fail
- Include detailed feedback

**Validation**:
- [ ] Test passes
- [ ] All output types supported
- [ ] Results are accurate
- [ ] Feedback is detailed

### Step 6.3: Test Suite Execution (1 hour 15 min)
**Test First**:
```rust
#[tokio::test]
async fn test_suite_runs_all_cases() {
    let manager = TestCaseManager::new(temp_dir());
    manager.save_test_case(&create_test_case()).await.unwrap();
    manager.save_test_case(&create_test_case()).await.unwrap();
    
    let executor = TestCaseExecutor::new();
    let template = create_test_template();
    
    let results = executor.run_suite(&manager, &template).await.unwrap();
    
    assert_eq!(results.total, 2);
    assert!(results.passed > 0);
}
```

**Implementation**:
- Load all test cases for template
- Execute each test case
- Aggregate results
- Generate summary report

**Validation**:
- [ ] Test passes
- [ ] All tests executed
- [ ] Results aggregated correctly
- [ ] Report is comprehensive

---

## Validation Criteria (Every Step)

### Compilation Check
```bash
cargo build --package chat_cli --lib
# Must exit with code 0
```

### Test Execution
```bash
cargo test --package chat_cli --lib creation::prompt_system::<module>
# Must pass all tests for that module
```

### Integration Test
```bash
cargo test --package chat_cli --lib creation::prompt_system::integration_tests
# Must pass after each phase
```

### Design Validation Questions
After each phase, answer:
1. Does this solve a real problem? (Not just satisfy a test)
2. Can this be extended without rewriting? (Open/Closed principle)
3. Are there any hardcoded values? (Must be NO)
4. Does this handle edge cases? (Empty inputs, invalid data, etc.)
5. Is the error handling comprehensive? (All failure modes covered)

---

## Estimated Total Time: 17.5 hours

### Phase Breakdown:
- Phase 1 (Quality Validator): 4 hours
- Phase 2 (Template Renderer): 3 hours
- Phase 3 (Cache Manager): 2.5 hours
- Phase 4 (Similarity Matching): 2 hours
- Phase 5 (Metrics Collection): 3 hours
- Phase 6 (Test Case Management): 3 hours

### Success Criteria:
- [ ] All 61 infrastructure test errors fixed
- [ ] All prompt_system tests pass
- [ ] Zero placeholder implementations remain
- [ ] All components have real, working functionality
- [ ] Code compiles with zero warnings
- [ ] Integration tests pass end-to-end
- [ ] Manual testing confirms features work

---

## Notes

- Each step must be completed before moving to next
- If a test fails, fix implementation before proceeding
- If design feels wrong, stop and reassess
- No "TODO" or "FIXME" comments allowed in final code
- Every function must have a real implementation
- All edge cases must be tested
