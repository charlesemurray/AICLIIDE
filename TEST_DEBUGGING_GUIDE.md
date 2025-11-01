# Test Debugging Guide

## Problem: Tests Not Running (0 tests discovered)

When `cargo test` shows "running 0 tests" despite having test functions in the code, use this systematic approach to diagnose and fix the issue.

## Diagnostic Strategies

### 1. Test Discovery Verification
```bash
# Check if tests are being discovered at all
cargo test --lib -- --list
```
**Expected**: Should list all available tests
**If 0 tests**: Tests aren't being compiled - proceed to step 2

### 2. Compilation Mode Analysis
```bash
# Check if tests compile in test mode
cargo test --lib --no-run
```
**Look for**: Compilation errors that prevent test inclusion
**Check**: If binary is created in `target/debug/deps/`

### 3. Conditional Compilation Audit
```bash
# Search for test-blocking cfg attributes
grep -r "cfg.*not.*test" src/
```
**Critical Issue**: `#![cfg(not(test))]` at crate root excludes entire library from test compilation
**Fix**: Remove or modify the attribute

### 4. Test Module Structure Verification
```bash
# Verify test modules are properly structured
grep -r "#\[cfg(test)\]" src/
grep -r "#\[test\]" src/
```
**Check for**:
- Missing `#[cfg(test)]` module declarations
- Missing imports in test modules (`use super::*;`)
- Compilation errors in test code

### 5. Import Dependencies Analysis
```bash
# Check for missing imports in test modules
cargo check --tests 2>&1 | grep -E "(error|unresolved)"
```
**Common Issues**:
- Missing `use crossterm::style::Color;` in theme tests
- Missing trait imports for test functionality
- Incorrect module paths

### 6. Workspace vs Crate Context
```bash
# Try running from different locations
cd crates/chat-cli && cargo test --lib
cd /workspace/root && cargo test --lib -p chat_cli
```
**Check**: If workspace configuration affects test discovery

## Step-by-Step Resolution Process

### Step 1: Identify Root Cause
1. Run `cargo test --lib -- --list`
2. If 0 tests, check for `#![cfg(not(test))]` in `lib.rs`
3. Remove or modify blocking cfg attributes

### Step 2: Fix Compilation Issues
1. Run `cargo check --tests`
2. Fix any compilation errors in test modules
3. Add missing imports (especially in theme modules)

### Step 3: Verify Test Structure
1. Ensure all test modules have `#[cfg(test)]`
2. Verify `use super::*;` imports
3. Check for proper `#[test]` function annotations

### Step 4: Validate Results
```bash
cargo test --lib
```
Should now show discovered and running tests.

## Common Issues and Solutions

### Issue: `#![cfg(not(test))]` in lib.rs
**Problem**: Excludes entire library from test compilation
**Solution**: Remove the attribute or change to conditional inclusion

### Issue: Missing imports in test modules
**Problem**: `Color::Blue` not found in theme tests
**Solution**: Add `use crossterm::style::Color;` to test modules

### Issue: Compilation errors in non-test code
**Problem**: Status command integration errors prevent test compilation
**Solution**: Temporarily comment out problematic code to isolate test issues

### Issue: Test modules not included
**Problem**: Missing `mod tests;` declarations
**Solution**: Ensure proper module structure with `#[cfg(test)]`

## Verification Commands

```bash
# Count total test functions
grep -r "#\[test\]" src/ | wc -l

# List all test modules
find src/ -name "*.rs" -exec grep -l "#\[cfg(test)\]" {} \;

# Check test compilation without running
cargo test --lib --no-run

# Run specific test module
cargo test theme::session_manager::tests
```

## Success Indicators

- `cargo test --lib -- --list` shows expected test count
- All test modules compile without errors
- Tests execute and show pass/fail results
- Test coverage matches implemented features

## Example Fix Applied

**Before**: `#![cfg(not(test))]` in lib.rs prevented all test compilation
**After**: Removed attribute, added missing imports, fixed 440 tests running successfully

This systematic approach resolved a complete test failure (0 tests) to full test suite execution (440 tests) by identifying and fixing the root cause blocking test compilation.
