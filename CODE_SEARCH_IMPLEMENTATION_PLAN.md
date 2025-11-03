# Code Search Tool - Implementation Plan

## Overview

Test-Driven Development implementation plan for the code search tool with strict no-placeholder policy, comprehensive testing, and incremental validation.

## Implementation Principles

### No Placeholders Policy
- Every commit must compile and pass all tests
- No `todo!()`, `unimplemented!()`, or placeholder comments
- Each step produces working, testable code
- Validation gates at every step

### TDD Approach
1. Write failing test
2. Implement minimal code to pass test
3. Refactor while keeping tests green
4. Validate compilation and functionality
5. Commit working code

## Phase 1: Foundation (Days 1-2)

### Step 1.1: Basic Structure & Tests
**Duration:** 2 hours
**Goal:** Create compilable tool structure with basic tests

**Tasks:**
```bash
# 1. Create tool module structure
touch crates/chat-cli/src/cli/chat/tools/code_search.rs

# 2. Add to mod.rs
# 3. Create basic struct and tests
```

**Test Cases:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_code_search_creation() {
        let search = CodeSearch {
            query: "test".to_string(),
            path: None,
            file_types: None,
            limit: None,
        };
        assert_eq!(search.query, "test");
    }
    
    #[test]
    fn test_validation_empty_query() {
        let mut search = CodeSearch {
            query: "".to_string(),
            path: None,
            file_types: None,
            limit: None,
        };
        // Test validation fails for empty query
    }
}
```

**Validation:**
- [ ] Code compiles: `cargo check`
- [ ] Tests pass: `cargo test code_search`
- [ ] No warnings: `cargo clippy`

**Git Commit:** `feat: add basic CodeSearch struct with validation tests`

### Step 1.2: Permission System
**Duration:** 3 hours
**Goal:** Implement permission evaluation following fs_read pattern

**Test Cases:**
```rust
#[test]
fn test_permission_allowed_paths() {
    // Test allowed paths configuration
}

#[test]
fn test_permission_denied_paths() {
    // Test denied paths configuration  
}

#[test]
fn test_permission_default_allow() {
    // Test default permission behavior
}
```

**Implementation:**
- Copy permission pattern from `fs_read.rs`
- Adapt for code search specific needs
- Implement path checking logic

**Validation:**
- [ ] All permission tests pass
- [ ] Integration with agent system works
- [ ] Code compiles without warnings

**Git Commit:** `feat: implement CodeSearch permission system`

### Step 1.3: Tool Registration
**Duration:** 2 hours  
**Goal:** Register tool in Q CLI system

**Tasks:**
1. Add to `Tool` enum in `mod.rs`
2. Add to `tool_manager.rs` match statement
3. Add to `NATIVE_TOOLS` array
4. Create tool schema in `tool_index.json`

**Test Cases:**
```rust
#[test]
fn test_tool_registration() {
    // Test tool can be created from JSON
    let json = r#"{"query": "test"}"#;
    let tool: CodeSearch = serde_json::from_str(json).unwrap();
    assert_eq!(tool.query, "test");
}
```

**Validation:**
- [ ] Tool appears in `/tools` command
- [ ] JSON deserialization works
- [ ] Tool can be invoked (even with stub implementation)

**Git Commit:** `feat: register CodeSearch tool in Q CLI system`

## Phase 2: Core Search (Days 3-4)

### Step 2.1: Ripgrep Integration
**Duration:** 4 hours
**Goal:** Implement reliable text search with ripgrep

**Test Cases:**
```rust
#[tokio::test]
async fn test_ripgrep_search_basic() {
    // Create test files
    // Run search
    // Verify results
}

#[tokio::test]
async fn test_ripgrep_file_type_filter() {
    // Test file type filtering works
}

#[tokio::test]
async fn test_ripgrep_limit_results() {
    // Test result limiting works
}
```

**Implementation:**
- Command construction for ripgrep
- Output parsing
- Error handling for missing ripgrep
- Result structure definition

**Validation:**
- [ ] Search works on test repository
- [ ] File type filtering works
- [ ] Result limiting works
- [ ] Handles ripgrep not installed gracefully

**Git Commit:** `feat: implement ripgrep-based text search`

### Step 2.2: Output Formatting
**Duration:** 2 hours
**Goal:** Format search results for LLM consumption

**Test Cases:**
```rust
#[test]
fn test_format_results_empty() {
    let results = vec![];
    let formatted = format_results(results, "test");
    assert!(formatted.contains("No results found"));
}

#[test]
fn test_format_results_multiple() {
    // Test multiple results formatting
}
```

**Implementation:**
- Result formatting function
- Consistent output structure
- Handle edge cases (no results, many results)

**Validation:**
- [ ] Output is well-formatted and readable
- [ ] LLM can parse results effectively
- [ ] Edge cases handled properly

**Git Commit:** `feat: implement search result formatting`

### Step 2.3: Full Tool Integration
**Duration:** 3 hours
**Goal:** Complete invoke() method and integration

**Test Cases:**
```rust
#[tokio::test]
async fn test_full_search_integration() {
    // End-to-end test of complete search
}

#[tokio::test]
async fn test_search_with_permissions() {
    // Test search respects permission settings
}
```

**Implementation:**
- Complete `invoke()` method
- Path resolution and validation
- Error handling and user feedback
- Integration with permission system

**Validation:**
- [ ] Full search workflow works end-to-end
- [ ] Permissions are respected
- [ ] Error messages are helpful
- [ ] Performance is acceptable (<2s for typical search)

**Git Commit:** `feat: complete CodeSearch tool implementation`

## Phase 3: Testing & Polish (Day 5)

### Step 3.1: Integration Tests
**Duration:** 3 hours
**Goal:** Comprehensive integration testing

**Test Cases:**
```rust
// In tests/integration/code_search_tests.rs
#[tokio::test]
async fn test_code_search_in_chat_session() {
    // Test tool works in actual chat session
}

#[tokio::test]
async fn test_code_search_with_agent_config() {
    // Test with various agent configurations
}
```

**Validation:**
- [ ] Tool works in real chat sessions
- [ ] Agent configurations are respected
- [ ] No regressions in existing functionality

**Git Commit:** `test: add comprehensive integration tests for CodeSearch`

### Step 3.2: Performance & Edge Cases
**Duration:** 2 hours
**Goal:** Handle edge cases and optimize performance

**Test Cases:**
```rust
#[tokio::test]
async fn test_large_repository_search() {
    // Test performance on large codebases
}

#[tokio::test]
async fn test_binary_file_handling() {
    // Test graceful handling of binary files
}

#[tokio::test]
async fn test_permission_denied_graceful() {
    // Test graceful permission denial
}
```

**Validation:**
- [ ] Performance acceptable on large repositories
- [ ] Binary files handled gracefully
- [ ] Permission errors are user-friendly
- [ ] Memory usage is reasonable

**Git Commit:** `fix: handle edge cases and optimize performance`

### Step 3.3: Documentation & Examples
**Duration:** 2 hours
**Goal:** Complete documentation and usage examples

**Tasks:**
1. Update tool documentation
2. Add usage examples
3. Update built-in tools documentation
4. Create troubleshooting guide

**Validation:**
- [ ] Documentation is complete and accurate
- [ ] Examples work as documented
- [ ] Troubleshooting guide covers common issues

**Git Commit:** `docs: add comprehensive CodeSearch documentation`

## Validation Gates

### After Each Step
```bash
# Compilation check
cargo check --all-targets

# Test execution  
cargo test code_search

# Linting
cargo clippy -- -D warnings

# Format check
cargo fmt --check
```

### After Each Phase
```bash
# Full test suite
cargo test

# Integration tests
cargo test --test integration

# Performance benchmarks (if applicable)
cargo bench

# Documentation build
cargo doc --no-deps
```

## Quality Assurance

### Code Quality Checklist
- [ ] No `todo!()` or `unimplemented!()` macros
- [ ] All public functions have documentation
- [ ] Error messages are user-friendly
- [ ] No unwrap() calls in production code
- [ ] Proper error propagation with `?` operator

### Test Coverage Requirements
- [ ] Unit tests for all public functions
- [ ] Integration tests for main workflows
- [ ] Edge case testing (empty inputs, large inputs, errors)
- [ ] Permission system testing
- [ ] Performance regression tests

### Git Commit Standards
```
feat: add new feature
fix: bug fix
test: add or modify tests
docs: documentation changes
refactor: code refactoring
perf: performance improvements
```

## Risk Mitigation

### Compilation Failures
- Validate compilation after every change
- Use incremental development approach
- Keep changes small and focused

### Test Failures
- Write tests before implementation (TDD)
- Run tests frequently during development
- Fix failing tests immediately

### Integration Issues
- Test integration points early
- Use existing Q CLI patterns exactly
- Validate with real usage scenarios

## Success Criteria

### Phase 1 Complete
- [ ] Tool compiles and registers successfully
- [ ] Permission system works correctly
- [ ] Basic structure tests pass

### Phase 2 Complete  
- [ ] Search functionality works end-to-end
- [ ] Results are properly formatted
- [ ] Performance meets requirements (<2s)

### Phase 3 Complete
- [ ] All tests pass (unit + integration)
- [ ] Documentation is complete
- [ ] Tool is ready for production use

## Timeline Summary

- **Day 1:** Foundation setup and tool registration
- **Day 2:** Permission system and basic structure
- **Day 3:** Core search implementation
- **Day 4:** Output formatting and integration
- **Day 5:** Testing, polish, and documentation

**Total Effort:** 5 days with strict TDD and no-placeholder approach
