# Code Search Implementation Checklist

## Pre-Implementation Setup

### Environment Preparation
- [ ] Ensure ripgrep is installed: `rg --version`
- [ ] Verify Rust toolchain: `cargo --version`
- [ ] Create feature branch: `git checkout -b feat/code-search-tool`
- [ ] Baseline tests pass: `cargo test`

## Day 1: Foundation

### Step 1.1: Basic Structure (2 hours)
**Goal:** Create compilable tool structure

#### Tasks:
- [ ] Create file: `crates/chat-cli/src/cli/chat/tools/code_search.rs`
- [ ] Add basic struct definition:
```rust
#[derive(Debug, Clone, Deserialize)]
pub struct CodeSearch {
    pub query: String,
    pub path: Option<String>,
    pub file_types: Option<Vec<String>>,
    pub limit: Option<usize>,
}
```
- [ ] Add to `crates/chat-cli/src/cli/chat/tools/mod.rs`:
```rust
pub mod code_search;
use code_search::CodeSearch;
```
- [ ] Write basic tests (see test specs)
- [ ] Validate: `cargo test code_search::structure_tests`
- [ ] Validate: `cargo check`
- [ ] **Git commit:** `feat: add basic CodeSearch struct with tests`

### Step 1.2: Validation Logic (2 hours)
**Goal:** Implement parameter validation

#### Tasks:
- [ ] Add validation method:
```rust
impl CodeSearch {
    pub async fn validate(&mut self, os: &Os) -> Result<()> {
        if self.query.trim().is_empty() {
            bail!("Search query cannot be empty");
        }
        // ... rest of validation
        Ok(())
    }
}
```
- [ ] Add validation tests (see test specs)
- [ ] Validate: `cargo test code_search::validation`
- [ ] Validate: `cargo clippy -- -D warnings`
- [ ] **Git commit:** `feat: implement CodeSearch parameter validation`

### Step 1.3: Permission System (3 hours)
**Goal:** Implement permission evaluation

#### Tasks:
- [ ] Study `fs_read.rs` permission pattern
- [ ] Implement `eval_perm` method following exact pattern
- [ ] Add permission tests (see test specs)
- [ ] Test with various agent configurations
- [ ] Validate: `cargo test code_search::permission_tests`
- [ ] **Git commit:** `feat: implement CodeSearch permission system`

### Step 1.4: Tool Registration (2 hours)
**Goal:** Register tool in Q CLI system

#### Tasks:
- [ ] Add to `Tool` enum in `tools/mod.rs`:
```rust
CodeSearch(CodeSearch),
```
- [ ] Add to `requires_acceptance` method:
```rust
Tool::CodeSearch(code_search) => code_search.eval_perm(os, agent),
```
- [ ] Add to `invoke` method (stub implementation):
```rust
Tool::CodeSearch(code_search) => {
    Ok(InvokeOutput {
        output: OutputKind::Text("CodeSearch not yet implemented".to_string())
    })
}
```
- [ ] Add to `tool_manager.rs` match statement:
```rust
"code_search" => Tool::CodeSearch(serde_json::from_value::<CodeSearch>(value.args)?),
```
- [ ] Add to `NATIVE_TOOLS` array
- [ ] Add schema to `tool_index.json`
- [ ] Test tool registration: Start Q CLI and verify tool appears in `/tools`
- [ ] **Git commit:** `feat: register CodeSearch tool in Q CLI system`

**End of Day 1 Validation:**
- [ ] All tests pass: `cargo test`
- [ ] No compilation warnings: `cargo clippy`
- [ ] Tool appears in Q CLI: `/tools` command shows code_search
- [ ] Tool can be invoked (returns stub message)

## Day 2: Search Implementation

### Step 2.1: Search Result Structure (1 hour)
**Goal:** Define search result data structures

#### Tasks:
- [ ] Add SearchResult struct:
```rust
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub file_path: PathBuf,
    pub line_number: usize,
    pub line_content: String,
    pub match_context: Option<String>,
}
```
- [ ] Add result parsing utilities
- [ ] Write structure tests
- [ ] **Git commit:** `feat: add SearchResult data structures`

### Step 2.2: Ripgrep Integration (4 hours)
**Goal:** Implement reliable text search

#### Tasks:
- [ ] Implement `ripgrep_search` method:
```rust
async fn ripgrep_search(&self, search_path: &Path, os: &Os) -> Result<Vec<SearchResult>> {
    let mut cmd = std::process::Command::new("rg");
    // ... command construction
}
```
- [ ] Add command construction logic
- [ ] Add output parsing logic
- [ ] Handle ripgrep not installed gracefully
- [ ] Add comprehensive search tests (see test specs)
- [ ] Test with various file types and patterns
- [ ] Validate: `cargo test code_search::search_tests`
- [ ] **Git commit:** `feat: implement ripgrep-based text search`

### Step 2.3: Output Formatting (2 hours)
**Goal:** Format results for LLM consumption

#### Tasks:
- [ ] Implement `format_results` method
- [ ] Handle empty results gracefully
- [ ] Format multiple results clearly
- [ ] Implement result limiting
- [ ] Add formatting tests (see test specs)
- [ ] Validate: `cargo test code_search::formatting_tests`
- [ ] **Git commit:** `feat: implement search result formatting`

**End of Day 2 Validation:**
- [ ] Search functionality works on test files
- [ ] Output is well-formatted and readable
- [ ] All edge cases handled (no results, many results, errors)

## Day 3: Integration

### Step 3.1: Complete Tool Implementation (3 hours)
**Goal:** Implement full invoke() method

#### Tasks:
- [ ] Implement complete `invoke` method:
```rust
pub async fn invoke(&self, os: &Os) -> Result<InvokeOutput> {
    let search_path = self.get_search_path(os)?;
    let results = self.search_with_fallback(&search_path, os).await?;
    let formatted_output = self.format_results(results);
    
    Ok(InvokeOutput {
        output: OutputKind::Text(formatted_output)
    })
}
```
- [ ] Add path resolution logic
- [ ] Add error handling and user feedback
- [ ] Update tool registration to use real implementation
- [ ] Test end-to-end functionality
- [ ] **Git commit:** `feat: complete CodeSearch tool implementation`

### Step 3.2: Integration Testing (3 hours)
**Goal:** Test tool in real Q CLI environment

#### Tasks:
- [ ] Create integration test file: `tests/integration/code_search_tests.rs`
- [ ] Test tool in actual chat session
- [ ] Test with various agent configurations
- [ ] Test permission system integration
- [ ] Validate: `cargo test --test integration`
- [ ] **Git commit:** `test: add CodeSearch integration tests`

### Step 3.3: Performance Validation (2 hours)
**Goal:** Ensure acceptable performance

#### Tasks:
- [ ] Test on large repository (>1000 files)
- [ ] Measure response times
- [ ] Test memory usage
- [ ] Add performance regression tests
- [ ] Optimize if necessary
- [ ] **Git commit:** `perf: validate and optimize CodeSearch performance`

**End of Day 3 Validation:**
- [ ] Tool works end-to-end in Q CLI
- [ ] Performance meets requirements (<2s for typical search)
- [ ] Integration tests pass
- [ ] No regressions in existing functionality

## Day 4: Polish & Documentation

### Step 4.1: Edge Case Handling (2 hours)
**Goal:** Handle all edge cases gracefully

#### Tasks:
- [ ] Test with binary files
- [ ] Test with very large files
- [ ] Test with permission denied scenarios
- [ ] Test with ripgrep not installed
- [ ] Add graceful error messages
- [ ] **Git commit:** `fix: handle edge cases and improve error messages`

### Step 4.2: Documentation (2 hours)
**Goal:** Complete documentation

#### Tasks:
- [ ] Add comprehensive doc comments to all public methods
- [ ] Update `docs/built-in-tools.md` with code_search section
- [ ] Add usage examples
- [ ] Create troubleshooting guide
- [ ] **Git commit:** `docs: add comprehensive CodeSearch documentation`

### Step 4.3: Final Testing (2 hours)
**Goal:** Comprehensive validation

#### Tasks:
- [ ] Run full test suite: `cargo test`
- [ ] Run integration tests: `cargo test --test integration`
- [ ] Test in real usage scenarios
- [ ] Validate documentation accuracy
- [ ] Check code coverage
- [ ] **Git commit:** `test: final validation and cleanup`

### Step 4.4: Code Review Preparation (2 hours)
**Goal:** Prepare for code review

#### Tasks:
- [ ] Self-review all code changes
- [ ] Ensure consistent code style
- [ ] Verify no placeholder code remains
- [ ] Check all tests pass
- [ ] Verify documentation is complete
- [ ] Create pull request with detailed description

**End of Day 4 Validation:**
- [ ] All tests pass (unit + integration)
- [ ] Documentation is complete and accurate
- [ ] Code is ready for production use
- [ ] No placeholders or TODOs remain

## Final Validation Checklist

### Code Quality
- [ ] No `todo!()`, `unimplemented!()`, or `panic!()` in production code
- [ ] All public functions have documentation
- [ ] Error messages are user-friendly
- [ ] No `unwrap()` calls in production code
- [ ] Proper error propagation with `?` operator

### Testing
- [ ] Unit test coverage >90%
- [ ] All integration tests pass
- [ ] Performance tests pass
- [ ] Edge cases covered
- [ ] Error scenarios tested

### Integration
- [ ] Tool appears in `/tools` command
- [ ] Tool works in chat sessions
- [ ] Permission system works correctly
- [ ] Agent configurations respected
- [ ] No regressions in existing functionality

### Performance
- [ ] Response time <500ms for typical searches
- [ ] Memory usage reasonable (<100MB during search)
- [ ] Handles large repositories gracefully
- [ ] Graceful degradation when ripgrep unavailable

### Documentation
- [ ] All public APIs documented
- [ ] Usage examples work as documented
- [ ] Troubleshooting guide complete
- [ ] Built-in tools documentation updated

## Git Commit History Example

```
feat: add basic CodeSearch struct with tests
feat: implement CodeSearch parameter validation  
feat: implement CodeSearch permission system
feat: register CodeSearch tool in Q CLI system
feat: add SearchResult data structures
feat: implement ripgrep-based text search
feat: implement search result formatting
feat: complete CodeSearch tool implementation
test: add CodeSearch integration tests
perf: validate and optimize CodeSearch performance
fix: handle edge cases and improve error messages
docs: add comprehensive CodeSearch documentation
test: final validation and cleanup
```

## Success Criteria

### Functional Requirements
- [ ] Tool successfully replaces bash-based code searches
- [ ] Provides consistent, structured output
- [ ] Respects permission configurations
- [ ] Handles errors gracefully

### Non-Functional Requirements
- [ ] Response time <500ms for indexed searches, <2s for text search
- [ ] Memory usage <100MB during operation
- [ ] No impact on Q CLI startup time
- [ ] Graceful handling of missing dependencies

### Quality Requirements
- [ ] Test coverage >90%
- [ ] Zero compilation warnings
- [ ] All documentation complete
- [ ] Code review ready

This checklist ensures systematic, test-driven implementation with no placeholders and comprehensive validation at every step.
