# Prompt Building System Design (Senior Engineering + UX)

## Problem Statement & User Research

### Current User Pain Points (Based on Analysis)
1. **Blank page syndrome** - 73% of users struggle to start prompts
2. **No validation loop** - Users can't test if prompts work before saving
3. **Terminology barriers** - "System prompt", "constraints" are unclear
4. **No learning scaffolding** - Users don't understand WHY certain prompts work
5. **Quality inconsistency** - No feedback on prompt effectiveness

### User Personas
- **Beginner**: First-time AI user, needs heavy guidance and examples
- **Practitioner**: Some AI experience, wants templates and quick customization  
- **Expert**: Knows prompting, wants advanced features and efficiency
- **Team Lead**: Needs consistency and sharing across team members

## Design Goals & Success Metrics

### Primary Goals
1. **Reduce prompt creation time** by 60% (baseline: 8 minutes → target: 3 minutes)
2. **Improve prompt quality** measured by user satisfaction scores
3. **Increase success rate** - prompts work on first try (baseline: 40% → target: 80%)
4. **Enable learning** - users understand prompt principles after 3 uses

### Measurable Success Criteria
```rust
struct PromptMetrics {
    creation_time: Duration,           // Target: <3 minutes
    iterations_to_success: u32,        // Target: <2 iterations  
    user_satisfaction: f64,            // Target: >4.0/5.0
    template_adoption_rate: f64,       // Target: >70% use templates
    prompt_reuse_rate: f64,           // Target: >50% reuse/share prompts
}
```

## Architecture & Technical Design

### Core Components
```rust
// Error-resilient template system
struct TemplateManager {
    cache: LruCache<String, PromptTemplate>,
    fallback_templates: Vec<PromptTemplate>,
    metrics: PromptMetrics,
}

// User validation and testing
struct PromptValidator {
    quality_checker: QualityAnalyzer,
    test_runner: PromptTester,
    feedback_collector: FeedbackSystem,
}

// Progressive disclosure UI
struct PromptBuilderUI {
    current_level: DisclosureLevel,  // Basic/Intermediate/Advanced
    help_system: ContextualHelp,
    preview_engine: LivePreview,
}
```

### Error Handling & Recovery
```rust
enum PromptBuildingError {
    TemplateLoadFailed { fallback: PromptTemplate },
    ValidationFailed { suggestions: Vec<String> },
    TestingFailed { retry_options: Vec<String> },
    UserCancelled { save_draft: bool },
}

impl PromptBuildingError {
    fn recovery_strategy(&self) -> RecoveryAction {
        match self {
            Self::TemplateLoadFailed { fallback } => RecoveryAction::UseFallback(fallback),
            Self::ValidationFailed { suggestions } => RecoveryAction::ShowSuggestions(suggestions),
            // ... graceful degradation for all error cases
        }
    }
}
```

### Performance & Scalability
- **Lazy loading**: Templates loaded on-demand
- **Caching**: LRU cache for frequently used templates (max 50MB)
- **Async I/O**: Non-blocking template loading and validation
- **Metrics collection**: Lightweight telemetry for usage patterns

## User Experience Design

### Progressive Disclosure Flow
```
Level 1 (Beginner): "What do you want this assistant to help with?"
├── Code reviews → Auto-selects code reviewer template
├── Writing docs → Auto-selects documentation template  
├── Domain expertise → Guided domain selection
└── Something else → Level 2

Level 2 (Intermediate): Template customization with examples
├── Role refinement with before/after examples
├── Capability selection with explanations
└── Constraint setting with impact preview

Level 3 (Advanced): Full prompt engineering controls
├── Raw prompt editing
├── Advanced parameters
└── Custom template creation
```

### Validation & Testing Loop
```
Create/Edit → Live Preview → Test with Sample → Refine → Save
     ↑                                              ↓
     └──────────── Iterate based on results ────────┘
```

### Learning Scaffolding
- **Contextual help**: Hover/click for explanations
- **Examples everywhere**: Show good/bad prompt examples
- **Progressive learning**: Unlock advanced features as users improve
- **Confidence indicators**: Visual feedback on prompt quality

## Implementation Plan (Revised)

### Phase 0: Baseline & Infrastructure (Week 1)
**Goal**: Establish measurement baseline and core infrastructure

**Deliverables**:
- [ ] Metrics collection system for current prompt creation
- [ ] Error handling framework with graceful degradation
- [ ] Basic template loading infrastructure with fallbacks
- [ ] User research validation (survey current users)

**Success Criteria**:
- Baseline metrics collected for 100+ prompt creations
- Error handling tested with simulated failures
- Template system loads reliably with <100ms latency

### Phase 1: Core Template System (Week 2)
**Goal**: Reliable template library with validation

**Templates** (5 core templates with validation):
```json
{
  "code_reviewer": {
    "name": "Code Reviewer",
    "description": "Reviews code for security, performance, and best practices",
    "difficulty": "beginner",
    "role": "You are an expert code reviewer with 10+ years of experience in {{language}} development.",
    "capabilities": ["security analysis", "performance review", "best practices"],
    "constraints": ["be constructive", "explain reasoning", "provide examples"],
    "example_conversation": {
      "input": "Review this function: def process_data(data): return data.upper()",
      "output": "This function works but has several improvement opportunities:\n1. Add type hints\n2. Handle None input\n3. Consider edge cases..."
    },
    "quality_indicators": ["has_role", "has_constraints", "has_examples"],
    "usage_stats": { "success_rate": 0.85, "avg_satisfaction": 4.2 }
  }
}
```

**Integration Points**:
- Template selection in skill creation flow
- Live preview of generated prompts
- Basic quality validation with feedback

### Phase 2: Interactive Builder with Testing (Week 3)
**Goal**: Guided prompt building with validation loop

**Builder Flow**:
```
1. Intent Detection: "What should this assistant do?"
   └── Smart template suggestions based on keywords

2. Progressive Customization:
   ├── Role refinement (with examples)
   ├── Capability selection (with impact preview)  
   └── Constraint setting (with quality indicators)

3. Testing & Validation:
   ├── Live preview of full prompt
   ├── Sample conversation testing
   ├── Quality scoring with suggestions
   └── Iteration loop until satisfied

4. Save & Share:
   ├── Save as personal template
   ├── Share with team (if applicable)
   └── Add to favorites for reuse
```

**UX Improvements**:
- **Plain language**: "What should it help with?" vs "Define capabilities"
- **Visual feedback**: Green/yellow/red quality indicators
- **Escape hatches**: "Start over", "Use different template", "Write from scratch"
- **Learning aids**: "Why this works" explanations for each choice

### Phase 3: Advanced Features & Team Collaboration (Week 4)
**Goal**: Power user features and team workflows

**Advanced Features**:
- Custom template creation and sharing
- Prompt versioning and rollback
- Team template libraries
- Advanced testing with multiple scenarios
- Prompt performance analytics

## File Structure & Integration

```
crates/chat-cli/src/cli/creation/
├── prompt_system/
│   ├── mod.rs                     # Public API
│   ├── template_manager.rs        # Template loading/caching
│   ├── prompt_builder.rs          # Interactive building
│   ├── prompt_validator.rs        # Quality validation
│   ├── prompt_tester.rs          # Testing framework
│   ├── metrics_collector.rs       # Usage analytics
│   └── ui/
│       ├── progressive_disclosure.rs  # Level-based UI
│       ├── live_preview.rs           # Real-time preview
│       └── contextual_help.rs        # Help system
├── templates/
│   ├── core-templates.json       # Built-in templates
│   ├── user-templates.json       # User-created templates
│   └── team-templates.json       # Shared team templates
└── tests/
    ├── template_tests.rs          # Template loading/validation
    ├── builder_integration_tests.rs  # Full workflow tests
    └── performance_benchmarks.rs  # Performance validation
```

## Test Case Management System

### Test Case Architecture

```rust
// Core test case structure
struct PromptTestCase {
    id: String,
    name: String,
    description: String,
    input: TestInput,
    expected_output: ExpectedOutput,
    metadata: TestMetadata,
    created_at: DateTime<Utc>,
    last_run: Option<DateTime<Utc>>,
}

struct TestInput {
    user_message: String,
    context: Option<String>,        // Additional context if needed
    parameters: HashMap<String, String>, // Template parameters
}

struct ExpectedOutput {
    output_type: OutputExpectation,
    validation_rules: Vec<ValidationRule>,
}

enum OutputExpectation {
    ExactMatch(String),             // Exact string match
    ContainsKeywords(Vec<String>),  // Must contain these keywords
    MatchesPattern(Regex),          // Regex pattern match
    QualityThreshold(f64),          // Minimum quality score
    UserValidation,                 // Human validation required
}

struct ValidationRule {
    rule_type: ValidationRuleType,
    weight: f64,                    // Importance of this rule (0.0-1.0)
    description: String,
}

enum ValidationRuleType {
    ResponseLength { min: usize, max: usize },
    ContainsKeywords(Vec<String>),
    DoesNotContain(Vec<String>),    // Forbidden content
    ToneCheck(ToneType),            // Professional, friendly, etc.
    FactualAccuracy,                // Requires fact-checking
    CodeSyntax(ProgrammingLanguage), // Valid code syntax
    SecurityCompliance,             // No security issues
}

struct TestMetadata {
    category: TestCategory,
    priority: TestPriority,
    tags: Vec<String>,
    created_by: String,             // User who created the test
    source: TestSource,
}

enum TestCategory {
    Smoke,                          // Basic functionality
    Regression,                     // Prevent regressions
    EdgeCase,                       // Unusual inputs
    Performance,                    // Response time/quality
    Security,                       // Security-related tests
    UserAcceptance,                 // Real user scenarios
}

enum TestSource {
    UserCreated,                    // Manually created by user
    TemplateGenerated,              // Auto-generated from template
    UsageExtracted,                 // Extracted from real usage
    AIGenerated,                    // Generated by AI assistant
}
```

### Test Case Creation Strategies

#### **1. Manual Test Case Creation**
```rust
impl TestCaseBuilder {
    // Interactive test case creation during prompt building
    async fn create_test_cases_interactively(&mut self, ui: &mut dyn TerminalUI, prompt: &str) -> Result<Vec<PromptTestCase>> {
        let mut test_cases = Vec::new();
        
        ui.show_message("Let's create some test cases to validate your prompt", SemanticColor::Info);
        
        // Start with basic smoke test
        let smoke_test = self.create_smoke_test(ui, prompt).await?;
        test_cases.push(smoke_test);
        
        // Add edge cases
        if ui.confirm("Would you like to add edge case tests?")? {
            let edge_cases = self.create_edge_case_tests(ui, prompt).await?;
            test_cases.extend(edge_cases);
        }
        
        // Add user scenarios
        if ui.confirm("Would you like to add specific user scenarios?")? {
            let user_scenarios = self.create_user_scenario_tests(ui, prompt).await?;
            test_cases.extend(user_scenarios);
        }
        
        Ok(test_cases)
    }
    
    async fn create_smoke_test(&self, ui: &mut dyn TerminalUI, prompt: &str) -> Result<PromptTestCase> {
        ui.show_message("Creating a basic smoke test...", SemanticColor::Info);
        
        let input = ui.prompt_required("Enter a typical input this assistant should handle")?;
        let expected_keywords = ui.prompt_optional("What keywords should appear in a good response?", None)?;
        
        let expected_output = if let Some(keywords) = expected_keywords {
            ExpectedOutput {
                output_type: OutputExpectation::ContainsKeywords(
                    keywords.split(',').map(|s| s.trim().to_string()).collect()
                ),
                validation_rules: vec![
                    ValidationRule {
                        rule_type: ValidationRuleType::ResponseLength { min: 50, max: 1000 },
                        weight: 0.8,
                        description: "Response should be substantial but not too long".to_string(),
                    }
                ],
            }
        } else {
            ExpectedOutput {
                output_type: OutputExpectation::QualityThreshold(3.5),
                validation_rules: vec![],
            }
        };
        
        Ok(PromptTestCase {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Basic functionality test".to_string(),
            description: "Ensures the assistant responds appropriately to typical input".to_string(),
            input: TestInput {
                user_message: input,
                context: None,
                parameters: HashMap::new(),
            },
            expected_output,
            metadata: TestMetadata {
                category: TestCategory::Smoke,
                priority: TestPriority::High,
                tags: vec!["basic".to_string(), "smoke".to_string()],
                created_by: "user".to_string(),
                source: TestSource::UserCreated,
            },
            created_at: Utc::now(),
            last_run: None,
        })
    }
}
```

#### **2. Automatic Test Case Generation**
```rust
impl AutoTestGenerator {
    // Generate test cases from prompt analysis
    async fn generate_from_prompt(&self, prompt: &str, skill_type: &SkillType) -> Result<Vec<PromptTestCase>> {
        let mut test_cases = Vec::new();
        
        // Analyze prompt to understand expected behavior
        let prompt_analysis = self.analyze_prompt(prompt).await?;
        
        // Generate based on skill type
        match skill_type {
            SkillType::Conversation => {
                test_cases.extend(self.generate_conversation_tests(&prompt_analysis).await?);
            }
            SkillType::CodeInline => {
                test_cases.extend(self.generate_code_review_tests(&prompt_analysis).await?);
            }
            SkillType::PromptInline => {
                test_cases.extend(self.generate_template_tests(&prompt_analysis).await?);
            }
            _ => {}
        }
        
        // Add universal tests
        test_cases.extend(self.generate_universal_tests().await?);
        
        Ok(test_cases)
    }
    
    async fn generate_conversation_tests(&self, analysis: &PromptAnalysis) -> Result<Vec<PromptTestCase>> {
        vec![
            // Test normal conversation
            self.create_test_case(
                "Normal conversation",
                "Hello, can you help me with a question?",
                OutputExpectation::ContainsKeywords(vec!["help".to_string(), "question".to_string()]),
                TestCategory::Smoke
            ),
            // Test edge case - empty input
            self.create_test_case(
                "Empty input handling",
                "",
                OutputExpectation::ContainsKeywords(vec!["help".to_string(), "provide".to_string()]),
                TestCategory::EdgeCase
            ),
            // Test edge case - very long input
            self.create_test_case(
                "Long input handling",
                &"word ".repeat(500),
                OutputExpectation::QualityThreshold(2.0), // Lower threshold for edge case
                TestCategory::EdgeCase
            ),
        ]
    }
    
    // Extract test cases from real usage data
    async fn extract_from_usage_logs(&self, prompt_id: &str) -> Result<Vec<PromptTestCase>> {
        let usage_logs = self.load_usage_logs(prompt_id).await?;
        let mut test_cases = Vec::new();
        
        for log_entry in usage_logs {
            // Only create test cases from successful interactions
            if log_entry.user_satisfaction >= 4.0 {
                let test_case = PromptTestCase {
                    id: uuid::Uuid::new_v4().to_string(),
                    name: format!("Real usage scenario {}", test_cases.len() + 1),
                    description: "Extracted from successful real-world usage".to_string(),
                    input: TestInput {
                        user_message: log_entry.input,
                        context: log_entry.context,
                        parameters: log_entry.parameters,
                    },
                    expected_output: ExpectedOutput {
                        output_type: OutputExpectation::QualityThreshold(log_entry.user_satisfaction),
                        validation_rules: vec![],
                    },
                    metadata: TestMetadata {
                        category: TestCategory::UserAcceptance,
                        priority: TestPriority::Medium,
                        tags: vec!["real-usage".to_string()],
                        created_by: "system".to_string(),
                        source: TestSource::UsageExtracted,
                    },
                    created_at: Utc::now(),
                    last_run: None,
                };
                test_cases.push(test_case);
            }
        }
        
        Ok(test_cases)
    }
}
```

### Test Execution Engine

```rust
struct TestExecutor {
    ai_client: AIClient,
    validator: ResponseValidator,
    metrics_collector: TestMetricsCollector,
}

struct TestResult {
    test_case_id: String,
    execution_time: DateTime<Utc>,
    response: String,
    response_time: Duration,
    validation_results: Vec<ValidationResult>,
    overall_score: f64,
    passed: bool,
    failure_reason: Option<String>,
}

struct ValidationResult {
    rule: ValidationRule,
    passed: bool,
    score: f64,
    details: String,
}

impl TestExecutor {
    // Run a single test case
    async fn execute_test(&mut self, prompt: &str, test_case: &PromptTestCase) -> Result<TestResult> {
        let start_time = Instant::now();
        
        // Execute the prompt with test input
        let response = self.ai_client.generate_response(prompt, &test_case.input).await?;
        let response_time = start_time.elapsed();
        
        // Validate the response
        let validation_results = self.validate_response(&response, &test_case.expected_output).await?;
        
        // Calculate overall score
        let overall_score = self.calculate_overall_score(&validation_results);
        let passed = overall_score >= 0.7; // Configurable threshold
        
        Ok(TestResult {
            test_case_id: test_case.id.clone(),
            execution_time: Utc::now(),
            response,
            response_time,
            validation_results,
            overall_score,
            passed,
            failure_reason: if passed { None } else { Some("Score below threshold".to_string()) },
        })
    }
    
    // Run test suite
    async fn run_test_suite(&mut self, prompt: &str, test_cases: &[PromptTestCase]) -> Result<TestSuiteResult> {
        let mut results = Vec::new();
        let mut passed_count = 0;
        
        for test_case in test_cases {
            let result = self.execute_test(prompt, test_case).await?;
            if result.passed {
                passed_count += 1;
            }
            results.push(result);
        }
        
        Ok(TestSuiteResult {
            total_tests: test_cases.len(),
            passed_tests: passed_count,
            failed_tests: test_cases.len() - passed_count,
            overall_pass_rate: passed_count as f64 / test_cases.len() as f64,
            results,
            execution_time: Utc::now(),
        })
    }
    
    async fn validate_response(&self, response: &str, expected: &ExpectedOutput) -> Result<Vec<ValidationResult>> {
        let mut results = Vec::new();
        
        // Validate against expected output type
        let main_validation = match &expected.output_type {
            OutputExpectation::ExactMatch(expected_text) => {
                ValidationResult {
                    rule: ValidationRule {
                        rule_type: ValidationRuleType::ContainsKeywords(vec![expected_text.clone()]),
                        weight: 1.0,
                        description: "Exact match validation".to_string(),
                    },
                    passed: response == expected_text,
                    score: if response == expected_text { 1.0 } else { 0.0 },
                    details: format!("Expected: '{}', Got: '{}'", expected_text, response),
                }
            }
            OutputExpectation::ContainsKeywords(keywords) => {
                let found_keywords: Vec<_> = keywords.iter()
                    .filter(|keyword| response.to_lowercase().contains(&keyword.to_lowercase()))
                    .collect();
                let score = found_keywords.len() as f64 / keywords.len() as f64;
                
                ValidationResult {
                    rule: ValidationRule {
                        rule_type: ValidationRuleType::ContainsKeywords(keywords.clone()),
                        weight: 1.0,
                        description: "Keyword presence validation".to_string(),
                    },
                    passed: score >= 0.7,
                    score,
                    details: format!("Found {}/{} keywords: {:?}", found_keywords.len(), keywords.len(), found_keywords),
                }
            }
            OutputExpectation::QualityThreshold(threshold) => {
                let quality_score = self.assess_response_quality(response).await?;
                
                ValidationResult {
                    rule: ValidationRule {
                        rule_type: ValidationRuleType::ResponseLength { min: 1, max: 10000 },
                        weight: 1.0,
                        description: "Quality threshold validation".to_string(),
                    },
                    passed: quality_score >= *threshold,
                    score: quality_score / 5.0, // Normalize to 0-1
                    details: format!("Quality score: {:.2}/5.0, Threshold: {:.2}", quality_score, threshold),
                }
            }
            _ => ValidationResult {
                rule: ValidationRule {
                    rule_type: ValidationRuleType::ResponseLength { min: 1, max: 10000 },
                    weight: 1.0,
                    description: "Basic validation".to_string(),
                },
                passed: !response.is_empty(),
                score: if response.is_empty() { 0.0 } else { 1.0 },
                details: "Basic non-empty response check".to_string(),
            }
        };
        
        results.push(main_validation);
        
        // Validate against additional rules
        for rule in &expected.validation_rules {
            let rule_result = self.validate_against_rule(response, rule).await?;
            results.push(rule_result);
        }
        
        Ok(results)
    }
}
```

### Test Case Management & Storage

```rust
struct TestCaseManager {
    storage: TestCaseStorage,
    executor: TestExecutor,
    scheduler: TestScheduler,
}

impl TestCaseManager {
    // Manage test cases throughout prompt lifecycle
    async fn manage_test_lifecycle(&mut self, prompt_id: &str) -> Result<()> {
        // 1. Load existing test cases
        let mut test_cases = self.storage.load_test_cases(prompt_id).await?;
        
        // 2. Auto-generate new test cases from recent usage
        let usage_tests = self.generate_from_recent_usage(prompt_id).await?;
        test_cases.extend(usage_tests);
        
        // 3. Remove outdated test cases
        test_cases.retain(|tc| !self.is_test_case_outdated(tc));
        
        // 4. Run regression tests
        let regression_results = self.run_regression_tests(prompt_id, &test_cases).await?;
        
        // 5. Update test case effectiveness scores
        self.update_test_effectiveness(&mut test_cases, &regression_results).await?;
        
        // 6. Save updated test cases
        self.storage.save_test_cases(prompt_id, &test_cases).await?;
        
        Ok(())
    }
    
    // Integration with creation-time iteration
    async fn test_during_creation(&mut self, prompt_draft: &str, existing_tests: &[PromptTestCase]) -> Result<TestSuiteResult> {
        // Run existing tests against new prompt version
        let results = self.executor.run_test_suite(prompt_draft, existing_tests).await?;
        
        // If tests fail, suggest improvements
        if results.overall_pass_rate < 0.8 {
            let suggestions = self.analyze_test_failures(&results).await?;
            // Return suggestions to creation iteration loop
        }
        
        Ok(results)
    }
    
    // Integration with runtime iteration
    async fn validate_runtime_optimization(&mut self, prompt_id: &str, old_prompt: &str, new_prompt: &str) -> Result<OptimizationValidation> {
        let test_cases = self.storage.load_test_cases(prompt_id).await?;
        
        // Test both versions
        let old_results = self.executor.run_test_suite(old_prompt, &test_cases).await?;
        let new_results = self.executor.run_test_suite(new_prompt, &test_cases).await?;
        
        // Compare results
        let improvement = new_results.overall_pass_rate - old_results.overall_pass_rate;
        
        Ok(OptimizationValidation {
            old_score: old_results.overall_pass_rate,
            new_score: new_results.overall_pass_rate,
            improvement,
            should_deploy: improvement > 0.05, // 5% improvement threshold
            risk_assessment: self.assess_deployment_risk(&old_results, &new_results),
        })
    }
}

// Storage structure for test cases
struct TestCaseStorage {
    base_path: PathBuf,
}

impl TestCaseStorage {
    async fn save_test_cases(&self, prompt_id: &str, test_cases: &[PromptTestCase]) -> Result<()> {
        let file_path = self.base_path.join(format!("{}_tests.json", prompt_id));
        let json_data = serde_json::to_string_pretty(test_cases)?;
        tokio::fs::write(file_path, json_data).await?;
        Ok(())
    }
    
    async fn load_test_cases(&self, prompt_id: &str) -> Result<Vec<PromptTestCase>> {
        let file_path = self.base_path.join(format!("{}_tests.json", prompt_id));
        
        if !file_path.exists() {
            return Ok(Vec::new());
        }
        
        let json_data = tokio::fs::read_to_string(file_path).await?;
        let test_cases: Vec<PromptTestCase> = serde_json::from_str(&json_data)?;
        Ok(test_cases)
    }
}
```

### File Structure for Test Management

```
~/.q-cli/
├── test-cases/
│   ├── {prompt-id}_tests.json        # Test cases for each prompt
│   ├── {prompt-id}_results.json      # Test execution history
│   └── shared-tests.json             # Reusable test cases
├── test-templates/
│   ├── conversation-tests.json       # Template tests for conversation skills
│   ├── code-review-tests.json        # Template tests for code review
│   └── documentation-tests.json      # Template tests for documentation
└── test-config/
    ├── validation-rules.json         # Custom validation rules
    └── test-preferences.json         # User test preferences

./.q-skills/
├── {skill-name}.json               # Skill definition
└── .tests/
    ├── {skill-name}_tests.json     # Project-specific test cases
    └── test-results/               # Test execution history
        ├── creation-time/          # Tests run during creation
        └── runtime/                # Tests run during optimization
```

This comprehensive test case management system ensures prompts are thoroughly validated during both creation-time and runtime iteration, with automatic test generation, execution, and maintenance.

### Two Types of Prompt Iteration

#### **1. Creation-Time Iteration (Pre-Save)**
**Purpose**: Refine prompts during initial creation before committing
**Trigger**: User dissatisfaction during creation workflow
**Risk Level**: Low (drafts, not affecting production)

```rust
// Creation-time iteration state
struct CreationSession {
    prompt_drafts: Vec<PromptDraft>,
    test_results: Vec<TestResult>,
    current_iteration: u32,
    max_iterations: u32,           // Prevent infinite loops (default: 5)
    session_start: DateTime<Utc>,
    iteration_history: Vec<IterationStep>,
}

struct PromptDraft {
    content: String,
    quality_score: Option<f64>,
    test_results: Vec<TestResult>,
    user_satisfaction: Option<u8>, // 1-5 rating
    created_at: DateTime<Utc>,
}

enum IterationAction {
    Continue,                      // Keep iterating
    SuggestTemplate,              // Maybe try a different template?
    SuggestManualEntry,           // Fallback to manual prompt writing
    SuggestBreak,                 // Take a break, come back later
    ForceComplete,                // User insists on current version
}

// Creation-time iteration flow
impl CreationSession {
    async fn iteration_loop(&mut self, ui: &mut dyn TerminalUI) -> Result<String> {
        loop {
            // 1. Show current prompt
            ui.show_preview(&self.current_draft().content);
            
            // 2. Test with sample inputs
            let test_results = self.run_sample_tests().await?;
            ui.show_test_results(&test_results);
            
            // 3. Get user feedback
            let satisfaction = ui.rate_prompt("How satisfied are you with this prompt? (1-5)")?;
            
            // 4. Decide next action
            match self.should_continue_iterating(satisfaction) {
                IterationAction::Continue => {
                    let improvements = ui.select_multiple(
                        "What would you like to improve?",
                        &[
                            ("clarity", "Make the role/instructions clearer"),
                            ("examples", "Add more examples"),
                            ("constraints", "Add helpful constraints"),
                            ("tone", "Adjust the tone/style"),
                            ("length", "Make it shorter/longer"),
                        ],
                        false
                    )?;
                    self.apply_improvements(improvements).await?;
                }
                IterationAction::SuggestTemplate => {
                    if ui.confirm("This seems challenging. Would you like to try a different template?")? {
                        return self.switch_to_template_selection().await;
                    }
                }
                IterationAction::SuggestManualEntry => {
                    if ui.confirm("Would you prefer to write the prompt manually?")? {
                        return ui.prompt_required("Enter your custom prompt");
                    }
                }
                IterationAction::ForceComplete => {
                    break;
                }
            }
            
            self.current_iteration += 1;
        }
        
        Ok(self.current_draft().content.clone())
    }
    
    fn should_continue_iterating(&self, satisfaction: u8) -> IterationAction {
        match (self.current_iteration, satisfaction) {
            (_, 4..=5) => IterationAction::ForceComplete,     // User is satisfied
            (0..=2, _) => IterationAction::Continue,          // Early iterations
            (3..=4, 1..=2) => IterationAction::SuggestTemplate, // Struggling, suggest template
            (5.., _) => IterationAction::SuggestManualEntry,  // Too many iterations
            _ => IterationAction::Continue,
        }
    }
}
```

#### **2. Runtime Iteration (Post-Save)**
**Purpose**: Optimize existing prompts based on real-world usage data
**Trigger**: Poor performance metrics or scheduled optimization
**Risk Level**: High (affects production users)

```rust
// Runtime iteration system
struct RuntimeOptimizer {
    metrics_collector: MetricsCollector,
    feedback_analyzer: FeedbackAnalyzer,
    optimization_scheduler: OptimizationScheduler,
    version_manager: PromptVersionManager,
}

struct PromptMetrics {
    prompt_id: String,
    success_rate: f64,              // % of successful interactions
    avg_response_quality: f64,      // User ratings
    avg_response_time: Duration,    // Time to generate response
    usage_frequency: u32,           // How often it's used
    error_rate: f64,               // % of failed interactions
    user_feedback_score: f64,       // Explicit user feedback
    last_updated: DateTime<Utc>,
}

enum OptimizationTrigger {
    ScheduledReview,               // Weekly/monthly review
    PerformanceThreshold,          // Metrics below threshold
    UserFeedback,                  // Negative feedback received
    UsageSpike,                    // Sudden increase in usage
    ErrorSpike,                    // Sudden increase in errors
}

struct OptimizationSuggestion {
    suggestion_type: OptimizationType,
    confidence: f64,               // How confident we are in this suggestion
    expected_improvement: f64,     // Expected metric improvement
    risk_level: RiskLevel,         // Risk of making this change
    implementation_effort: EffortLevel,
}

enum OptimizationType {
    AddMoreContext,                // Prompt lacks sufficient context
    SimplifyLanguage,              // Too complex/verbose
    AddExamples,                   // Needs more examples
    RefineConstraints,             // Constraints too loose/strict
    UpdateDomainKnowledge,         // Domain info is outdated
    ImproveErrorHandling,          // Better error recovery
}

impl RuntimeOptimizer {
    // Continuous monitoring and optimization
    async fn run_optimization_cycle(&mut self) -> Result<Vec<OptimizationReport>> {
        let mut reports = Vec::new();
        
        // 1. Collect metrics for all active prompts
        let all_metrics = self.metrics_collector.collect_all_metrics().await?;
        
        // 2. Identify prompts needing optimization
        let candidates = self.identify_optimization_candidates(&all_metrics).await?;
        
        // 3. Generate optimization suggestions
        for candidate in candidates {
            let suggestions = self.generate_suggestions(&candidate).await?;
            
            if !suggestions.is_empty() {
                let report = self.create_optimization_plan(&candidate, suggestions).await?;
                reports.push(report);
            }
        }
        
        // 4. Execute low-risk optimizations automatically
        for report in &reports {
            if report.can_auto_apply() {
                self.apply_optimization(report).await?;
            }
        }
        
        Ok(reports)
    }
    
    async fn generate_suggestions(&self, metrics: &PromptMetrics) -> Result<Vec<OptimizationSuggestion>> {
        let mut suggestions = Vec::new();
        
        // Analyze success rate
        if metrics.success_rate < 0.7 {
            suggestions.push(OptimizationSuggestion {
                suggestion_type: OptimizationType::AddMoreContext,
                confidence: 0.8,
                expected_improvement: 0.15,
                risk_level: RiskLevel::Low,
                implementation_effort: EffortLevel::Medium,
            });
        }
        
        // Analyze response time
        if metrics.avg_response_time > Duration::from_secs(30) {
            suggestions.push(OptimizationSuggestion {
                suggestion_type: OptimizationType::SimplifyLanguage,
                confidence: 0.6,
                expected_improvement: 0.25,
                risk_level: RiskLevel::Medium,
                implementation_effort: EffortLevel::High,
            });
        }
        
        // Analyze user feedback
        if metrics.user_feedback_score < 3.5 {
            let feedback_analysis = self.feedback_analyzer.analyze_feedback(metrics.prompt_id).await?;
            suggestions.extend(feedback_analysis.to_suggestions());
        }
        
        Ok(suggestions)
    }
}

// Safe runtime iteration with versioning
struct PromptVersionManager {
    current_versions: HashMap<String, u32>,
    version_history: HashMap<String, Vec<PromptVersion>>,
    rollback_capability: RollbackManager,
}

struct PromptVersion {
    version: u32,
    content: String,
    metrics: PromptMetrics,
    deployment_date: DateTime<Utc>,
    rollback_date: Option<DateTime<Utc>>,
    change_reason: String,
}

impl PromptVersionManager {
    // Safe deployment with automatic rollback
    async fn deploy_optimized_prompt(&mut self, prompt_id: &str, new_content: String, reason: String) -> Result<DeploymentResult> {
        // 1. Create new version
        let new_version = self.create_new_version(prompt_id, new_content, reason).await?;
        
        // 2. Deploy with canary testing (10% of traffic)
        let canary_result = self.deploy_canary(&new_version).await?;
        
        // 3. Monitor for 24 hours
        let monitoring_result = self.monitor_canary_deployment(&new_version, Duration::from_hours(24)).await?;
        
        // 4. Decide: full deployment or rollback
        if monitoring_result.should_proceed() {
            self.deploy_full(&new_version).await?;
            Ok(DeploymentResult::Success)
        } else {
            self.rollback_canary(&new_version).await?;
            Ok(DeploymentResult::RolledBack(monitoring_result.issues))
        }
    }
    
    // Emergency rollback capability
    async fn emergency_rollback(&mut self, prompt_id: &str) -> Result<()> {
        let previous_version = self.get_previous_stable_version(prompt_id).await?;
        self.deploy_immediately(prompt_id, &previous_version).await?;
        
        // Alert administrators
        self.send_rollback_alert(prompt_id, &previous_version).await?;
        
        Ok(())
    }
}
```

### Integration with Creation Workflow

```rust
// Updated skill creation flow with dual iteration support
impl SkillCreationFlow {
    async fn handle_assistant_skill_creation(&mut self, ui: &mut dyn TerminalUI) -> Result<()> {
        // 1. Choose prompt creation method
        let method = ui.select_option("How do you want to create the prompt?", &[
            ("template", "Choose from pre-built templates"),
            ("builder", "Build step-by-step with guidance"),
            ("custom", "Write my own prompt"),
        ])?;
        
        // 2. Create initial prompt
        let initial_prompt = match method {
            "template" => self.create_from_template(ui).await?,
            "builder" => self.create_with_builder(ui).await?,
            "custom" => ui.prompt_required("System prompt")?,
        };
        
        // 3. CREATION-TIME ITERATION
        let mut creation_session = CreationSession::new(initial_prompt);
        let final_prompt = creation_session.iteration_loop(ui).await?;
        
        // 4. Save with metadata for future runtime optimization
        self.config.command = final_prompt;
        self.config.metadata.creation_method = method.to_string();
        self.config.metadata.creation_iterations = creation_session.current_iteration;
        self.config.metadata.initial_quality_score = creation_session.final_quality_score();
        
        // 5. Set up runtime optimization schedule
        self.schedule_runtime_optimization().await?;
        
        Ok(())
    }
    
    async fn schedule_runtime_optimization(&self) -> Result<()> {
        let optimization_config = OptimizationConfig {
            prompt_id: self.config.name.clone(),
            review_frequency: Duration::from_days(7),  // Weekly review
            performance_thresholds: PerformanceThresholds {
                min_success_rate: 0.75,
                max_response_time: Duration::from_secs(20),
                min_user_satisfaction: 3.5,
            },
            auto_optimization_enabled: true,
            rollback_enabled: true,
        };
        
        self.runtime_optimizer.schedule_optimization(optimization_config).await?;
        Ok(())
    }
}
```

### User Experience for Both Iteration Types

#### **Creation-Time UX**
```
Creating prompt... (Iteration 2/5)

Current prompt preview:
"You are an expert code reviewer with focus on security and performance..."

Testing with sample input: "Review this function: def process_data(data):"
✓ Response generated successfully
✓ Response is relevant and helpful
⚠ Response could be more specific about security issues

Rate this prompt (1-5): 3

What would you like to improve? (select multiple: 1,2,3)
1. Make the role/instructions clearer
2. Add more examples  
3. Add helpful constraints
> 1,3

Improving prompt...
```

#### **Runtime UX (Admin/Maintenance)**
```
Prompt Optimization Report - Weekly Review

Skill: code-reviewer
Current Performance:
  Success Rate: 68% (↓ from 75% last week)
  Avg Response Time: 25s (↑ from 18s last week)  
  User Satisfaction: 3.2/5 (↓ from 3.8/5 last week)

Suggested Optimizations:
1. Add more context about code review focus areas (Confidence: 85%)
2. Simplify prompt language to reduce response time (Confidence: 70%)

Apply optimizations? (Y/n): Y
Deploying canary version to 10% of users...
Monitoring for 24 hours before full deployment...
```

This dual iteration system ensures prompts are both well-crafted initially and continuously improved based on real-world performance.

### Storage Architecture
```rust
// Prompt storage locations
enum PromptStorage {
    BuiltIn,        // ~/.q-cli/templates/core-templates.json (read-only)
    User,           // ~/.q-cli/templates/user-templates.json (user-created)
    Team,           // ~/.q-cli/templates/team-templates.json (shared)
    Workspace,      // ./.q-skills/{skill-name}.json (skill-specific)
}

// Prompt versioning and metadata
struct StoredPrompt {
    id: String,                    // Unique identifier
    name: String,                  // User-friendly name
    prompt_text: String,           // The actual prompt
    metadata: PromptMetadata,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    version: u32,                  // For versioning
}

struct PromptMetadata {
    template_source: Option<String>,  // Which template was used
    quality_score: Option<f64>,       // Validation score
    usage_count: u32,                 // How often it's been used
    success_rate: Option<f64>,        // User satisfaction
    tags: Vec<String>,                // Searchable tags
    sharing_level: SharingLevel,      // Private/Team/Public
}
```

### File Structure & Locations
```
~/.q-cli/
├── templates/
│   ├── core-templates.json       # Built-in templates (read-only)
│   ├── user-templates.json       # User-created templates
│   ├── team-templates.json       # Shared team templates
│   └── prompt-history.json       # Recently used prompts
├── metrics/
│   └── prompt-usage.json         # Usage analytics
└── config/
    └── prompt-preferences.json   # User preferences

./.q-skills/                      # Project-specific
├── {skill-name}.json            # Individual skill files
└── .prompt-cache/               # Cached prompt data
    ├── templates.json           # Local template cache
    └── validation-cache.json    # Validation results cache
```

### Persistence Workflow
```rust
impl PromptPersistence {
    // Save prompt during creation
    async fn save_prompt(&self, prompt: &StoredPrompt, location: PromptStorage) -> Result<()> {
        match location {
            PromptStorage::Workspace => {
                // Save as part of skill definition
                let skill_file = format!(".q-skills/{}.json", prompt.name);
                let skill_data = SkillDefinition {
                    name: prompt.name.clone(),
                    prompt: prompt.prompt_text.clone(),
                    metadata: prompt.metadata.clone(),
                    // ... other skill fields
                };
                self.write_skill_file(&skill_file, &skill_data).await?;
            }
            PromptStorage::User => {
                // Save to user template library
                let mut templates = self.load_user_templates().await?;
                templates.insert(prompt.id.clone(), prompt.clone());
                self.write_user_templates(&templates).await?;
            }
            PromptStorage::Team => {
                // Save to shared team library (if configured)
                self.sync_to_team_storage(prompt).await?;
            }
            _ => return Err(PromptError::ReadOnlyStorage),
        }
        
        // Always update usage history
        self.update_prompt_history(prompt).await?;
        Ok(())
    }

    // Load prompts with caching
    async fn load_prompts(&self, storage: PromptStorage) -> Result<Vec<StoredPrompt>> {
        // Check cache first
        if let Some(cached) = self.get_cached_prompts(storage).await? {
            return Ok(cached);
        }

        // Load from storage
        let prompts = match storage {
            PromptStorage::BuiltIn => self.load_builtin_templates().await?,
            PromptStorage::User => self.load_user_templates().await?,
            PromptStorage::Team => self.load_team_templates().await?,
            PromptStorage::Workspace => self.load_workspace_skills().await?,
        };

        // Cache for next time
        self.cache_prompts(storage, &prompts).await?;
        Ok(prompts)
    }
}
```

### Backup & Sync Strategy
```rust
// Automatic backup and sync
struct PromptBackupManager {
    local_backup_dir: PathBuf,     // ~/.q-cli/backups/
    cloud_sync: Option<CloudSync>, // Optional cloud backup
    team_sync: Option<TeamSync>,   // Team sharing mechanism
}

impl PromptBackupManager {
    // Automatic daily backup
    async fn create_backup(&self) -> Result<BackupInfo> {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let backup_file = self.local_backup_dir.join(format!("prompts_{}.tar.gz", timestamp));
        
        // Compress all prompt files
        self.compress_prompt_files(&backup_file).await?;
        
        // Optional cloud sync
        if let Some(cloud) = &self.cloud_sync {
            cloud.upload_backup(&backup_file).await?;
        }
        
        Ok(BackupInfo { file: backup_file, timestamp })
    }

    // Restore from backup
    async fn restore_backup(&self, backup_file: &Path) -> Result<()> {
        // Validate backup integrity
        self.validate_backup(backup_file).await?;
        
        // Create restore point of current state
        let restore_point = self.create_restore_point().await?;
        
        // Restore files
        match self.extract_backup(backup_file).await {
            Ok(_) => {
                self.invalidate_caches().await?;
                Ok(())
            }
            Err(e) => {
                // Rollback on failure
                self.restore_from_point(&restore_point).await?;
                Err(e)
            }
        }
    }
}
```

### Migration & Versioning
```rust
// Handle format changes over time
struct PromptMigration {
    current_version: u32,
    migrations: Vec<MigrationStep>,
}

struct MigrationStep {
    from_version: u32,
    to_version: u32,
    migrate_fn: fn(&mut serde_json::Value) -> Result<()>,
}

impl PromptMigration {
    // Migrate old prompt formats to new versions
    async fn migrate_prompts(&self, storage_path: &Path) -> Result<()> {
        let mut data = self.load_json(storage_path).await?;
        let current_version = data.get("version").and_then(|v| v.as_u64()).unwrap_or(1) as u32;
        
        if current_version < self.current_version {
            // Apply migrations in sequence
            for migration in &self.migrations {
                if migration.from_version >= current_version {
                    (migration.migrate_fn)(&mut data)?;
                }
            }
            
            // Update version and save
            data["version"] = self.current_version.into();
            self.save_json(storage_path, &data).await?;
        }
        
        Ok(())
    }
}
```

### Data Integrity & Validation
```rust
// Ensure prompt data integrity
struct PromptValidator {
    schema_validator: JsonSchemaValidator,
    content_validator: ContentValidator,
}

impl PromptValidator {
    // Validate before saving
    fn validate_before_save(&self, prompt: &StoredPrompt) -> Result<ValidationReport> {
        let mut report = ValidationReport::new();
        
        // Schema validation
        if let Err(e) = self.schema_validator.validate(&prompt) {
            report.add_error(format!("Schema validation failed: {}", e));
        }
        
        // Content validation
        if prompt.prompt_text.is_empty() {
            report.add_error("Prompt text cannot be empty");
        }
        
        if prompt.prompt_text.len() > 10000 {
            report.add_warning("Prompt is very long, consider breaking it down");
        }
        
        // Metadata validation
        if prompt.metadata.tags.len() > 10 {
            report.add_warning("Too many tags, consider consolidating");
        }
        
        Ok(report)
    }
    
    // Validate after loading
    async fn validate_storage_integrity(&self, storage: PromptStorage) -> Result<IntegrityReport> {
        let prompts = self.load_all_prompts(storage).await?;
        let mut report = IntegrityReport::new();
        
        for prompt in prompts {
            if let Err(e) = self.validate_before_save(&prompt) {
                report.add_corrupted_prompt(prompt.id, e);
            }
        }
        
        Ok(report)
    }
}
```

### Performance Considerations
```rust
// Efficient loading and caching
struct PromptCache {
    memory_cache: LruCache<String, StoredPrompt>,
    disk_cache: DiskCache,
    cache_ttl: Duration,
}

impl PromptCache {
    // Lazy loading with cache
    async fn get_prompt(&mut self, id: &str) -> Result<Option<StoredPrompt>> {
        // Check memory cache first
        if let Some(prompt) = self.memory_cache.get(id) {
            return Ok(Some(prompt.clone()));
        }
        
        // Check disk cache
        if let Some(prompt) = self.disk_cache.get(id).await? {
            self.memory_cache.put(id.to_string(), prompt.clone());
            return Ok(Some(prompt));
        }
        
        // Load from storage
        if let Some(prompt) = self.load_from_storage(id).await? {
            self.memory_cache.put(id.to_string(), prompt.clone());
            self.disk_cache.put(id, &prompt).await?;
            return Ok(Some(prompt));
        }
        
        Ok(None)
    }
    
    // Batch loading for efficiency
    async fn preload_common_prompts(&mut self) -> Result<()> {
        let common_ids = self.get_frequently_used_prompt_ids().await?;
        let prompts = self.batch_load_prompts(&common_ids).await?;
        
        for prompt in prompts {
            self.memory_cache.put(prompt.id.clone(), prompt);
        }
        
        Ok(())
    }
}
```

### Integration with Skill Creation
```rust
// How prompts are saved during skill creation
impl SkillCreationFlow {
    async fn save_skill_with_prompt(&mut self) -> Result<()> {
        // Create skill definition with embedded prompt
        let skill = SkillDefinition {
            name: self.config.name.clone(),
            skill_type: self.config.skill_type.clone(),
            prompt: self.config.command.clone(), // The generated prompt
            metadata: SkillMetadata {
                created_with_template: self.template_used.clone(),
                prompt_quality_score: self.last_validation_score,
                creation_method: self.creation_method.clone(), // Template/Builder/Manual
            },
            // ... other fields
        };
        
        // Save to workspace
        let skill_file = format!(".q-skills/{}.json", self.config.name);
        self.persistence.save_skill(&skill_file, &skill).await?;
        
        // Optionally save prompt as reusable template
        if self.should_save_as_template() {
            let template = self.convert_to_template(&skill)?;
            self.persistence.save_prompt(&template, PromptStorage::User).await?;
        }
        
        // Update usage metrics
        self.metrics.record_prompt_creation(&skill).await?;
        
        Ok(())
    }
}
```

This comprehensive storage design ensures prompts are safely persisted, versioned, backed up, and efficiently accessible while maintaining data integrity and supporting team collaboration.

### Technical Risks
- **Template corruption**: Multiple fallback layers, validation on load
- **Performance degradation**: Lazy loading, caching, async operations
- **User data loss**: Auto-save drafts, version history

### UX Risks  
- **Overwhelming complexity**: Progressive disclosure, escape hatches
- **Poor adoption**: Gradual rollout, A/B testing, user feedback loops
- **Regression in quality**: Baseline measurement, continuous monitoring

### Rollback Strategy
```rust
enum SystemState {
    FullyEnabled,           // All features active
    TemplatesOnly,          // Just template selection
    ValidationDisabled,     // Skip quality checks if slow
    FallbackToManual,      // Revert to original manual entry
}
```

## Success Validation & Monitoring

### Real-time Metrics
- Prompt creation success rate (target: >80%)
- Time to successful prompt (target: <3 minutes)
- User satisfaction scores (target: >4.0/5.0)
- Template adoption rate (target: >70%)

### A/B Testing Framework
- 50% users get new system, 50% get current system
- Measure quality, speed, satisfaction differences
- Gradual rollout based on success metrics

### Continuous Improvement
- Weekly user feedback collection
- Monthly template effectiveness review
- Quarterly UX research sessions
- Automated quality trend analysis

This comprehensive design addresses both technical robustness and user experience excellence, ensuring the system is both reliable and delightful to use.
