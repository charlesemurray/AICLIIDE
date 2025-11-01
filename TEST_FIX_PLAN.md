# Test Fix Action Plan

## Phase 1: Critical Security & Core Fixes (Days 1-3)

### 1.1 Fix Input Sanitization (SECURITY CRITICAL)
**File**: `crates/chat-cli/src/cli/skills/tests/security_tests.rs`
**Issue**: Command injection not being blocked
**Action**: 
```bash
# Test the current sanitization
cargo test test_input_sanitization -- --nocapture

# Fix the sanitization logic in security module
# Ensure dangerous patterns are properly filtered
```

### 1.2 Fix File Creation Logic
**Files**: 
- `crates/chat-cli/src/cli/creation/tests/cli.rs`
- `crates/chat-cli/src/cli/creation/flows/`

**Issues**: 
- `test_create_skill_execution` - skill files not created
- `test_create_command_execution` - command files not created  
- `test_create_agent_execution` - agent files not created

**Action**:
```bash
# Identify why file creation is failing
cargo test test_create_skill_execution -- --nocapture

# Fix the underlying file creation logic in flows
```

### 1.3 Fix Template System
**File**: `crates/chat-cli/src/cli/creation/prompt_system/tests/template_tests.rs`
**Issue**: Missing `metadata` field in templates
**Action**:
```bash
# Check template structure
cargo test test_template_loading -- --nocapture

# Add missing metadata field to template definitions
```

## Phase 2: Remove Broken/Useless Tests (Days 4-5)

### 2.1 Delete Trivial Tests
**Target tests to remove**:
```rust
// These tests provide no value:
fn test_app_name() { println!("{}", app_name()); }
fn test_behavior_version() { assert!(behavior_version() == BehaviorVersion::latest()); }
fn test_error_handling() { let error = SkillError::NotFound; assert_eq!(error.to_string(), "Skill not found"); }
```

### 2.2 Fix Always-Skipping Tests
**Files**: `crates/semantic-search-client/tests/`
**Issue**: Tests that always skip unless env var set
**Action**: Either mock dependencies or remove tests

### 2.3 Fix Timeout Test
**File**: `crates/chat-cli/src/cli/skills/tests/resilience_tests.rs`
**Issue**: `test_graceful_error_handling` hangs for 30s
**Action**: Add proper timeout handling or mock the slow operation

## Phase 3: Fix Integration Tests (Days 6-8)

### 3.1 Fix Creation Flow Tests
**Files**: `crates/chat-cli/src/cli/creation/tests/`
**Issues**: 
- Directory traversal errors
- Missing file operations
- Path resolution failures

**Action**: Mock file system operations for tests

### 3.2 Fix UX Tests  
**Files**: `crates/chat-cli/src/cli/creation/tests/ux.rs`
**Issues**:
- ANSI color tests failing
- Progress indication tests failing
- UI output validation failing

**Action**: Fix output formatting and validation logic

### 3.3 Fix Skills Tests
**Files**: `crates/chat-cli/src/cli/skills/tests/`
**Issues**:
- Workspace skill loading
- Parameter validation  
- Skill deserialization

**Action**: Fix skill loading and validation logic

## Phase 4: Performance & Quality (Days 9-10)

### 4.1 Optimize Remaining Slow Tests
**Target**: Tests taking >1 second
**Action**: Replace I/O operations with mocks

### 4.2 Add Missing Assertions
**Target**: Tests that create objects but don't verify behavior
**Action**: Add meaningful assertions to validate functionality

## Implementation Commands

### Day 1: Security Fix
```bash
# 1. Run failing security test to understand issue
cargo test test_input_sanitization -- --nocapture

# 2. Examine sanitization logic
grep -r "sanitiz" crates/chat-cli/src/cli/skills/

# 3. Fix the sanitization function
# 4. Verify fix works
cargo test test_input_sanitization
```

### Day 2: File Creation Fix
```bash
# 1. Run failing file creation tests
cargo test test_create_skill_execution -- --nocapture
cargo test test_create_command_execution -- --nocapture
cargo test test_create_agent_execution -- --nocapture

# 2. Trace through creation flows
# 3. Fix file writing logic
# 4. Verify fixes
cargo test cli::creation::tests::cli::command_execution
```

### Day 3: Template Fix
```bash
# 1. Run template test
cargo test test_template_loading -- --nocapture

# 2. Check template structure
find . -name "*.json" -path "*/templates/*" | head -5 | xargs cat

# 3. Add missing metadata fields
# 4. Verify fix
cargo test template_tests
```

### Day 4-5: Cleanup
```bash
# 1. Find and remove trivial tests
grep -r "println!" crates/chat-cli/src --include="*.rs" | grep "#\[test\]" -A 5 -B 5

# 2. Remove always-skipping tests
grep -r "return;" crates/ --include="*.rs" | grep -B 10 "#\[test\]"

# 3. Fix timeout test
cargo test test_graceful_error_handling --timeout 5
```

### Day 6-8: Integration Fixes
```bash
# 1. Run all failing integration tests
cargo test cli::creation::tests::integration -- --nocapture

# 2. Fix file system mocking
# 3. Fix UI output validation
# 4. Fix skill loading

# Verify fixes
cargo test cli::creation::tests
cargo test cli::skills::tests
```

### Day 9-10: Final Cleanup
```bash
# 1. Run full test suite
cargo test --lib

# 2. Identify remaining slow tests
cargo test --lib -- --report-time

# 3. Add missing assertions
# 4. Final verification
cargo test --lib
```

## Success Metrics

### Phase 1 Success:
- [ ] Security test passes (no command injection)
- [ ] File creation tests pass (files actually created)
- [ ] Template loading test passes

### Phase 2 Success:
- [ ] Removed 10+ trivial tests
- [ ] Fixed timeout test (completes in <5s)
- [ ] No tests that always skip

### Phase 3 Success:
- [ ] Integration tests pass
- [ ] UX tests pass  
- [ ] Skills tests pass

### Phase 4 Success:
- [ ] All tests complete in <30s total
- [ ] Test pass rate >95%
- [ ] No tests without meaningful assertions

## Risk Mitigation

1. **Backup**: Create branch before each phase
2. **Incremental**: Fix one test category at a time
3. **Verification**: Run full suite after each fix
4. **Rollback**: Keep working tests working

## Estimated Timeline: 10 days
- **Days 1-3**: Critical fixes (security, file creation, templates)
- **Days 4-5**: Remove broken/useless tests  
- **Days 6-8**: Fix integration and skills tests
- **Days 9-10**: Performance optimization and quality improvements
