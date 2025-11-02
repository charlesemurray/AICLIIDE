# TDD Quick Reference

## The TDD Cycle (Red-Green-Refactor)

```
1. 🔴 RED:    Write test → Run → FAILS
2. 🟢 GREEN:  Write code → Run → PASSES  
3. 🔵 REFACTOR: Clean up → Run → PASSES
4. ✅ COMMIT
```

## Why TDD?

- **Ensures tests actually test something** (they fail first)
- **Prevents false positives** (tests that always pass)
- **Drives minimal implementation** (only write code to pass tests)
- **Documents requirements** (tests show what code should do)
- **Catches regressions** (tests protect against future breaks)

## Example: Adding a New Feature

### ❌ Wrong Way (Implementation First)
```rust
// 1. Write the code
pub fn calculate_total(items: &[Item]) -> f64 {
    items.iter().map(|i| i.price).sum()
}

// 2. Write test (might pass even if broken)
#[test]
fn test_calculate_total() {
    let items = vec![Item { price: 10.0 }];
    assert_eq!(calculate_total(&items), 10.0);
}
```
**Problem**: Test might pass even if implementation is wrong!

### ✅ Right Way (TDD)
```rust
// 1. Write test FIRST (will fail - function doesn't exist)
#[test]
fn test_calculate_total() {
    let items = vec![Item { price: 10.0 }, Item { price: 20.0 }];
    assert_eq!(calculate_total(&items), 30.0);
}

// 2. Run test → FAILS ✅ (good! proves test works)
// cargo test test_calculate_total
// Error: cannot find function `calculate_total`

// 3. Write MINIMAL code to pass
pub fn calculate_total(items: &[Item]) -> f64 {
    items.iter().map(|i| i.price).sum()
}

// 4. Run test → PASSES ✅
// cargo test test_calculate_total
// test test_calculate_total ... ok

// 5. Commit
```

## TDD Workflow for Each Iteration

```bash
# Step 1: Write test(s) FIRST
vim src/module.rs  # Add test

# Step 2: Run test - should FAIL
cargo test test_new_feature
# Expected: FAIL ✅

# Step 3: Implement minimal code
vim src/module.rs  # Add implementation

# Step 4: Run test - should PASS
cargo test test_new_feature
# Expected: PASS ✅

# Step 5: Format & lint
cargo +nightly fmt
cargo clippy

# Step 6: Run all tests
cargo test

# Step 7: Commit
git add -A
git commit -m "Add new feature"
```

## Common TDD Patterns

### Pattern 1: Test Edge Cases First
```rust
#[test]
fn test_empty_input() {
    assert_eq!(process(&[]), vec![]);
}

#[test]
fn test_single_item() {
    assert_eq!(process(&[1]), vec![1]);
}

#[test]
fn test_multiple_items() {
    assert_eq!(process(&[1, 2, 3]), vec![1, 2, 3]);
}
```

### Pattern 2: Test Error Cases
```rust
#[test]
fn test_invalid_input_returns_error() {
    let result = validate("");
    assert!(result.is_err());
}

#[test]
fn test_valid_input_returns_ok() {
    let result = validate("valid");
    assert!(result.is_ok());
}
```

### Pattern 3: Test Serialization
```rust
#[test]
fn test_serialization_roundtrip() {
    let original = MyStruct::new("test");
    let json = serde_json::to_string(&original).unwrap();
    let deserialized: MyStruct = serde_json::from_str(&json).unwrap();
    assert_eq!(original, deserialized);
}
```

## TDD Anti-Patterns (Avoid These)

### ❌ Writing Implementation First
```rust
// DON'T: Write code then test
fn my_function() { /* implementation */ }
#[test] fn test_my_function() { /* test */ }
```

### ❌ Tests That Always Pass
```rust
// DON'T: Test that doesn't verify anything
#[test]
fn test_something() {
    assert!(true);  // Always passes!
}
```

### ❌ Not Running Tests Before Implementation
```rust
// DON'T: Skip the "red" phase
// Write test → Skip running → Write code → Run test
// You never verified the test can fail!
```

### ❌ Testing Implementation Details
```rust
// DON'T: Test internal implementation
#[test]
fn test_internal_helper() {
    // Testing private function
}

// DO: Test public API
#[test]
fn test_public_behavior() {
    // Testing what users see
}
```

## TDD Checklist for Each Iteration

- [ ] 1. Write test(s) first
- [ ] 2. Run test - verify it FAILS
- [ ] 3. Write minimal implementation
- [ ] 4. Run test - verify it PASSES
- [ ] 5. Refactor if needed
- [ ] 6. Run test - still PASSES
- [ ] 7. Format & lint
- [ ] 8. Run all tests
- [ ] 9. Commit

## Benefits of TDD

✅ **Confidence**: Tests prove code works
✅ **Documentation**: Tests show how to use code
✅ **Design**: Writing tests first improves API design
✅ **Regression**: Tests catch future breaks
✅ **Debugging**: Failing test pinpoints problem
✅ **Refactoring**: Tests enable safe changes

## Remember

> "If you didn't see the test fail, you don't know if it's testing anything."

**Always see RED before GREEN!** 🔴 → 🟢
