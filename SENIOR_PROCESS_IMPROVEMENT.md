# Senior-Level Process Improvement Plan

**Date**: 2025-11-03  
**Goal**: Elevate process from Mid (5/10) to Senior (9/10)

---

## The Core Problem

**Current**: Write code → Commit → Test later (maybe)  
**Senior**: Write test → Run (RED) → Implement → Run (GREEN) → Commit

**Gap**: Skipping verification steps between implementation and commit

---

## Concrete Process Changes

### 1. Strict TDD Cycle (Non-Negotiable)

#### Before Every Implementation:

```bash
# Step 1: Write the test FIRST
vim tool_manager.rs
# Add: test_skills_in_tool_schema()

# Step 2: Run test - MUST FAIL
cargo test test_skills_in_tool_schema
# Expected output: "FAILED" or "method not found"
# If it passes, test is wrong!

# Step 3: Implement minimal code
vim tool_manager.rs
# Add: skills to schema loop

# Step 4: Run test - MUST PASS
cargo test test_skills_in_tool_schema
# Expected output: "test ... ok"
# If it fails, fix implementation

# Step 5: Only NOW commit
git add -A
git commit -m "Add skills to tool schema"
```

**Rule**: If you can't show me RED → GREEN, you didn't do TDD.

---

### 2. Verification Checklist (Every Commit)

Create a script: `verify.sh`

```bash
#!/bin/bash
set -e

echo "1. Formatting..."
cargo +nightly fmt --check || (cargo +nightly fmt && echo "Fixed formatting")

echo "2. Compiling..."
cargo build --lib

echo "3. Linting..."
cargo clippy -- -D warnings

echo "4. Testing..."
cargo test --lib

echo "✅ All checks passed - safe to commit"
```

**Rule**: Run `./verify.sh` before EVERY commit. No exceptions.

---

### 3. Commit Message Template

Create `.gitmessage`:

```
<type>: <subject>

Verified:
- [ ] Tests pass (cargo test <test_name>)
- [ ] Code compiles (cargo build --lib)
- [ ] Linting clean (cargo clippy)
- [ ] Formatted (cargo +nightly fmt)

Changes:
- 

Testing:
$ cargo test <specific_test>
<paste output showing PASS>
```

**Usage**:
```bash
git config commit.template .gitmessage
```

**Rule**: Every commit must show verification evidence.

---

### 4. Feature Branch Workflow

```bash
# Start feature
git checkout -b feature/add-skills-to-schema

# Make change
vim tool_manager.rs

# Verify
./verify.sh

# Commit with evidence
git commit -m "Add skills to schema

Verified:
- [x] Tests pass (cargo test test_skills_in_tool_schema)
- [x] Code compiles
- [x] Linting clean

Testing:
$ cargo test test_skills_in_tool_schema
test test_skills_in_tool_schema ... ok
"

# Merge only after ALL tests pass
cargo test --lib
git checkout main
git merge feature/add-skills-to-schema
```

**Rule**: Never merge without full test suite passing.

---

### 5. Communication Standards

#### ❌ Don't Say:
- "Production ready" (without deployment)
- "Tests pass" (without showing output)
- "Complete" (without verification)
- "No errors" (without running checks)

#### ✅ Do Say:
- "Implementation complete, tests passing" (with proof)
- "Code compiles, running full test suite now"
- "Feature implemented, verified locally"
- "Tests written and verified: [paste output]"

**Rule**: Every claim needs evidence.

---

### 6. Pre-Commit Hook

Create `.git/hooks/pre-commit`:

```bash
#!/bin/bash

echo "Running pre-commit checks..."

# Check formatting
if ! cargo +nightly fmt --check; then
    echo "❌ Code not formatted. Run: cargo +nightly fmt"
    exit 1
fi

# Check compilation
if ! cargo build --lib 2>&1 | grep -q "Finished"; then
    echo "❌ Code doesn't compile"
    exit 1
fi

# Run tests for changed files
CHANGED_FILES=$(git diff --cached --name-only | grep "\.rs$")
if [ -n "$CHANGED_FILES" ]; then
    echo "Running tests for changed files..."
    if ! cargo test --lib; then
        echo "❌ Tests failed"
        exit 1
    fi
fi

echo "✅ Pre-commit checks passed"
```

```bash
chmod +x .git/hooks/pre-commit
```

**Rule**: Commit hook prevents bad commits automatically.

---

### 7. Code Review Checklist

Before requesting review:

```markdown
## Self-Review Checklist

- [ ] All tests pass locally
- [ ] Code compiles without warnings
- [ ] Ran `cargo clippy` - no issues
- [ ] Formatted with `cargo +nightly fmt`
- [ ] Added tests for new functionality
- [ ] Tests demonstrate RED → GREEN cycle
- [ ] No `unwrap()` or `panic!()` in production code
- [ ] Error handling is comprehensive
- [ ] Documentation added for public APIs
- [ ] Commit messages include verification evidence

## Test Evidence

```bash
$ cargo test <relevant_tests>
<paste output>
```

## Manual Testing

Tested:
1. <scenario 1>
2. <scenario 2>

Results: <describe>
```

**Rule**: Complete checklist before asking for review.

---

### 8. Iteration Template

For each feature iteration:

```markdown
## Iteration N: <Feature Name>

### 1. Write Test (RED)
```rust
#[test]
fn test_feature() {
    // Test code
}
```

### 2. Run Test - Verify FAIL
```bash
$ cargo test test_feature
test test_feature ... FAILED
```

### 3. Implement
```rust
pub fn feature() {
    // Implementation
}
```

### 4. Run Test - Verify PASS
```bash
$ cargo test test_feature
test test_feature ... ok
```

### 5. Commit
```bash
git commit -m "Add feature

Verified: Tests pass
Evidence: [see above]
"
```
```

**Rule**: Document RED → GREEN for every iteration.

---

## Specific Improvements for This Project

### What Should Have Happened:

#### Iteration 3: Add Skills to Schema

```bash
# 1. Write test FIRST
cat >> tool_manager.rs << 'EOF'
#[tokio::test]
async fn test_skills_in_tool_schema() {
    let mut os = Os::new().await.unwrap();
    let dir = tempdir().unwrap();
    
    let skill_json = r#"{"name": "test-skill", ...}"#;
    fs::write(dir.path().join("test-skill.json"), skill_json).unwrap();
    
    let mut manager = ToolManager::new_with_skills(&os).await.unwrap();
    manager.skill_registry.load_from_directory(dir.path()).await.unwrap();
    
    let schema = manager.load_tools(&mut os, &mut std::io::sink()).await.unwrap();
    
    assert!(schema.contains_key("test-skill"));
}
EOF

# 2. Run test - MUST FAIL
cargo test test_skills_in_tool_schema
# Output: FAILED (skill not in schema)

# 3. Implement
vim tool_manager.rs
# Add skills loop to load_tools()

# 4. Run test - MUST PASS
cargo test test_skills_in_tool_schema
# Output: test ... ok

# 5. Verify compilation
cargo build --lib
# Output: Finished

# 6. Commit with evidence
git commit -m "Add skills to tool schema in load_tools

Verified:
- [x] Test passes: test_skills_in_tool_schema
- [x] Code compiles
- [x] No clippy warnings

Test output:
$ cargo test test_skills_in_tool_schema
test test_skills_in_tool_schema ... ok

test result: ok. 1 passed
"
```

---

## Daily Workflow

### Morning:
```bash
# Pull latest
git pull origin main

# Verify everything works
cargo test --lib
cargo build --lib

# If anything fails, fix BEFORE starting work
```

### During Development:
```bash
# For EACH change:
1. Write test
2. Run test (RED)
3. Implement
4. Run test (GREEN)
5. Run ./verify.sh
6. Commit with evidence
```

### Before Pushing:
```bash
# Full verification
cargo test --lib
cargo build --lib
cargo clippy
cargo +nightly fmt --check

# Only push if ALL pass
git push origin <branch>
```

---

## Metrics to Track

### Process Health Indicators:

1. **Test-First Ratio**: % of commits with test before implementation
   - Target: 100%
   - Current: ~30%

2. **Verification Rate**: % of commits with passing tests
   - Target: 100%
   - Current: ~50%

3. **Bug Introduction Rate**: Bugs per 100 lines of code
   - Target: <1
   - Current: ~3

4. **Time to Detection**: Time between bug introduction and discovery
   - Target: <5 minutes (caught by tests)
   - Current: Hours/days

---

## Tools to Add

### 1. Watch Mode
```bash
# Auto-run tests on file change
cargo watch -x test
```

### 2. Test Coverage
```bash
# See what's tested
cargo tarpaulin --out Html
```

### 3. Continuous Integration
```yaml
# .github/workflows/ci.yml
name: CI
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - run: cargo test --lib
      - run: cargo clippy
      - run: cargo +nightly fmt --check
```

---

## Accountability Measures

### 1. Commit Review
Every Friday, review last week's commits:
- Did each have tests?
- Did each show verification?
- Were tests run before commit?

### 2. Bug Retrospective
For each bug:
- When was it introduced?
- Why wasn't it caught by tests?
- What test would have caught it?
- Add that test now

### 3. Process Audit
Monthly check:
- % commits with verification
- % tests written first
- Average time to bug detection
- Trend improving?

---

## The Non-Negotiables

### 1. Test Before Commit
```bash
# ALWAYS run before commit
cargo test <relevant_tests>
```
**No exceptions. Ever.**

### 2. Show Your Work
```bash
# Every claim needs proof
"Tests pass" → Show test output
"No errors" → Show compilation output
"Complete" → Show verification
```

### 3. RED → GREEN → REFACTOR
```bash
# TDD cycle is sacred
1. Write test (RED)
2. Make it pass (GREEN)
3. Clean up (REFACTOR)
4. Commit
```

### 4. Honest Communication
```bash
# Say what you know
"Implemented" ✅
"Tested locally" ✅
"Production ready" ❌ (unless deployed)
```

---

## 30-Day Improvement Plan

### Week 1: Build Habits
- [ ] Create verify.sh script
- [ ] Set up pre-commit hook
- [ ] Use commit template
- [ ] Run tests before EVERY commit

### Week 2: Enforce TDD
- [ ] Write test FIRST for every change
- [ ] Document RED → GREEN cycle
- [ ] No commits without test evidence
- [ ] Track test-first ratio

### Week 3: Improve Communication
- [ ] Include verification in all updates
- [ ] Show test output in commits
- [ ] Be honest about what's verified
- [ ] No claims without evidence

### Week 4: Measure & Adjust
- [ ] Review metrics
- [ ] Identify gaps
- [ ] Adjust process
- [ ] Celebrate improvements

---

## Success Criteria

### After 30 days:

✅ **100% of commits** have passing tests  
✅ **100% of features** have tests written first  
✅ **0 bugs** introduced that tests would catch  
✅ **<5 minutes** from bug introduction to detection  
✅ **All claims** backed by evidence  

---

## The Bottom Line

**Current process**: Hope it works → Commit → Fix later  
**Senior process**: Prove it works → Commit → Done

**Key insight**: The test suite is your proof. If you haven't run it, you haven't proven anything.

**One rule to rule them all**:
```
If you can't show me the test passing,
you can't claim the feature works.
```

That's it. That's the difference between mid-level and senior.
