# Test Consistency Guide

## Problem Identified

Tests were experiencing inconsistent results with fluctuating failure counts (13 → 24 → 19 failed tests) due to race conditions and shared resource conflicts when running in parallel.

## Root Cause Analysis

The main inconsistency sources were:

1. **Mixed Resource Isolation**: Some tests used proper isolation (`TestFixtures` with `TempDir`) while others used shared resources (`std::env::current_dir()`, `std::env::set_current_dir()`)
2. **Global State Mutation**: Tests changing the global working directory with `set_current_dir()` 
3. **Incorrect Test Expectations**: Tests asserting against wrong fields or expecting non-existent error messages

## Solution Strategy

### 1. Test Isolation
- **Replace shared directory usage** with isolated `TestFixtures`
- **Remove `std::env::set_current_dir()` calls** that change global state
- **Use `TempDir` for all file operations** to ensure each test has its own workspace

### 2. Fix Test Expectations  
- **Match assertions to actual implementation behavior**
- **Check correct fields** (e.g., `validation.suggestion` vs `validation.error_message()`)
- **Use proper TestFixtures paths** instead of hardcoded directories

### 3. Deterministic Resource Management
- **Consistent use of TestFixtures** across all creation tests
- **Proper cleanup through RAII** (automatic with `TempDir`)

## Implementation

### Files Fixed

#### Integration Tests (`crates/chat-cli/src/cli/creation/tests/integration.rs`)
```rust
// Before (problematic)
let temp_dir = TempDir::new()?;
std::env::set_current_dir(&temp_dir)?;

// After (isolated)
let fixtures = TestFixtures::new();
fixtures.setup_directories();
```

#### CLI Tests (`crates/chat-cli/src/cli/creation/tests/cli.rs`)
```rust
// Before (problematic)
std::env::set_current_dir(&fixtures.temp_dir).unwrap();

// After (isolated)
// Removed set_current_dir calls entirely
```

#### UX Tests (`crates/chat-cli/src/cli/creation/tests/ux.rs`)
```rust
// Before (problematic)
let context = CreationContext::new(&std::env::current_dir().unwrap()).unwrap();

// After (isolated)
let fixtures = TestFixtures::new();
fixtures.setup_directories();
let context = CreationContext::new(fixtures.temp_dir.path()).unwrap();

// Before (incorrect assertion)
assert!(error_msg.contains("force"));

// After (correct field)
assert!(validation.suggestion.contains("force"));
```

## Results

- **Before**: 19-24 failed tests (inconsistent)
- **After**: 12 failed tests (consistent)
- **Improvement**: ~40% reduction in test failures
- **Consistency**: Stable test counts across runs

## Best Practices for Future Tests

### ✅ Do
- Use `TestFixtures::new()` for all creation tests
- Call `fixtures.setup_directories()` to create required directories
- Pass `fixtures.temp_dir.path()` to constructors that need a base path
- Use `fixtures.skills_dir`, `fixtures.commands_dir`, etc. for file operations
- Assert against the correct fields (`error_message()` vs `suggestion`)

### ❌ Don't
- Use `std::env::current_dir()` in tests
- Use `std::env::set_current_dir()` in tests
- Create files in shared directories
- Assume test execution order
- Mix isolated and shared resource patterns

### Example Template
```rust
#[test]
fn test_my_feature() {
    let fixtures = TestFixtures::new();
    fixtures.setup_directories();
    
    // Use fixtures.temp_dir.path() for base paths
    let context = CreationContext::new(fixtures.temp_dir.path()).unwrap();
    
    // Use fixtures directories for file operations
    std::fs::write(fixtures.skills_dir.join("test.json"), "{}").unwrap();
    
    // Test your feature...
    let result = context.validate_name("test", &CreationType::Skill);
    assert!(!result.is_valid());
    assert!(result.error_message().contains("already exists"));
}
```

## Key Insight

The core principle is **test isolation**: each test should have its own isolated environment that doesn't interfere with other tests. This is achieved through:

1. **Unique temporary directories** per test via `TestFixtures`
2. **No global state mutations** (avoid `set_current_dir`)
3. **Proper resource cleanup** through RAII patterns

This approach scales well and prevents the "works alone but fails in parallel" problem common in integration tests.
