# Development Workflow Rules - Quick Reference

## The Golden Rules

### 1. Iteration Size: MAX 2 HOURS
- Prefer 30-45 minute iterations
- One focused change per iteration
- If it takes longer, break it down more

### 2. TEST-DRIVEN DEVELOPMENT (STRICT)
**Write tests FIRST, then implement:**
```rust
// 1. Write the test (it will FAIL)
#[test]
fn test_new_feature() {
    let result = new_feature();
    assert_eq!(result, expected);
}

// 2. Run test - it FAILS ✅
// 3. Implement minimal code to make it PASS
// 4. Run test - it PASSES ✅
```

**Order:**
1. Write test(s) first
2. Run tests (they should FAIL)
3. Implement code
4. Run tests (they should PASS)
5. Format, lint, commit

### 3. NO PLACEHOLDERS (STRICT)
```rust
// ❌ NEVER DO THIS
fn my_function() {
    todo!()
}

fn another_function() {
    unimplemented!()
}

// ✅ DO THIS INSTEAD
fn my_function() -> Result<String> {
    Ok("minimal working implementation".to_string())
}
```

### 3. MUST COMPILE (STRICT)
Every iteration must result in compiling code:
```bash
cargo build --all-targets
# Must succeed before commit
```

### 4. MUST TEST (STRICT)
Every iteration includes at least 1 test:
```bash
cargo test
# Must pass before commit
```

### 5. COMMIT AFTER EVERY ITERATION
```bash
# The workflow:
cargo +nightly fmt
cargo clippy
cargo test
git add -A
git commit -m "Add Skill variant to ToolOrigin"
```

## Commit Message Format

```
<action> <what>
```

Examples:
- ✅ "Add Skill variant to ToolOrigin"
- ✅ "Implement script execution with timeout"
- ✅ "Fix clippy warnings in skill module"
- ❌ "WIP" (too vague)
- ❌ "Update code" (not specific)
- ❌ "Fix stuff" (not clear)

## Before Every Commit Checklist

```bash
# 1. Write tests FIRST
# Add test(s) to appropriate test module

# 2. Run tests (should FAIL initially)
cargo test <test_name>

# 3. Implement code to make tests pass

# 4. Run tests again (should PASS)
cargo test <test_name>

# 5. Format
cargo +nightly fmt

# 6. Lint
cargo clippy

# 7. Run all tests
cargo test

# 8. Stage
git add -A

# 9. Commit
git commit -m "Clear, specific message"

# 10. Push (every 3-4 commits)
git push
```

## Checkpoint Schedule

### Quick Checkpoint (10 min)
- After every 4 iterations
- Quick code review
- Check for issues

### Phase Checkpoint (1 hour)
- After every phase
- Run full test suite
- Run benchmarks
- Check test coverage
- Review technical debt
- Update documentation

## What to Do When...

### "This will take more than 2 hours"
→ Break it into smaller iterations

### "I need a placeholder to make it compile"
→ Use a minimal working implementation instead

### "Tests are failing"
→ Fix them before committing (strict rule)

### "Clippy has warnings"
→ Fix them before committing

### "I want to add a feature not in the plan"
→ Defer it, stick to the plan

### "The iteration is taking too long"
→ Commit what you have, break the rest into next iteration

## Minimal Working Implementation Pattern

Instead of placeholders, use minimal implementations:

```rust
// Instead of todo!(), do this:

// For functions that will process data:
fn process_data(&self, input: &str) -> Result<String> {
    Ok(input.to_string()) // Echo for now
}

// For functions that will validate:
fn validate(&self) -> Result<()> {
    Ok(()) // Accept all for now
}

// For functions that will transform:
fn transform(&self, value: Value) -> Result<Value> {
    Ok(value) // Pass through for now
}

// For functions that will load:
fn load(&self) -> Result<Vec<Item>> {
    Ok(Vec::new()) // Empty for now
}
```

## Progress Tracking

Track your progress in the implementation plan:
- [ ] Iteration not started
- [x] Iteration complete

Mark the commit hash next to completed iterations.

## Example Iteration (TDD)

**Iteration 1.1.1: Add Skill variant (30 min)**

1. Open `crates/chat-cli/src/cli/chat/tools/mod.rs`
2. **Write test first**: `test_tool_origin_skill_display()`
3. **Write test first**: `test_tool_origin_skill_serialization()`
4. Run: `cargo test test_tool_origin_skill` - **FAILS** ✅
5. Add `Skill(String)` to `ToolOrigin` enum
6. Update `Display` impl
7. Update `Hash`, `Borrow`, `Serialize`, `Deserialize` impls
8. Run: `cargo test test_tool_origin_skill` - **PASSES** ✅
9. Run: `cargo +nightly fmt`
10. Run: `cargo clippy`
11. Run: `cargo test` (all tests)
12. Commit: "Add Skill variant to ToolOrigin"
13. ✅ Done in 30 minutes

## Red Flags

🚩 "I'll write tests after implementing" → NO, write tests FIRST
🚩 "I'll fix the tests later" → NO, fix them now
🚩 "I'll add the implementation later" → NO, add minimal version now
🚩 "This is just temporary" → NO, make it work properly
🚩 "I'll commit when the feature is done" → NO, commit after each iteration
🚩 "The plan is too detailed" → NO, follow the plan

## Success Metrics

- ✅ 102 iterations complete
- ✅ 102+ commits
- ✅ 102+ tests
- ✅ 0 compilation failures
- ✅ 0 placeholders in code
- ✅ >85% test coverage
- ✅ All clippy warnings fixed

## Remember

**Small iterations + Working code + Regular commits = Success**

The plan is designed to prevent:
- Long debugging sessions
- Broken code sitting around
- Unclear progress
- Integration nightmares
- Technical debt accumulation

Follow the rules, trust the process.
