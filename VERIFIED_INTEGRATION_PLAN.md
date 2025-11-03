# Verified Integration Plan

**Date**: 2025-11-03  
**Goal**: Actually integrate features with verification at each step

## Execution Process

For each step:
1. Make the change
2. Verify file was modified: `git diff <file>`
3. Test compilation: `cargo build --bin chat_cli`
4. Commit immediately
5. Verify commit: `git show HEAD`

## Steps

### Step 1: Fix SkillTool Error Format (15 min)
**File**: `crates/chat-cli/src/cli/chat/tools/skill_tool.rs`
**Change**: Use `SkillError::NotFound` instead of string
**Verify**: `grep "SkillError::NotFound" crates/chat-cli/src/cli/chat/tools/skill_tool.rs`

### Step 2: Add ErrorRecovery to skills_cli imports (5 min)
**File**: `crates/chat-cli/src/cli/skills_cli.rs`
**Change**: Add `ErrorRecovery` to imports
**Verify**: `grep "ErrorRecovery" crates/chat-cli/src/cli/skills_cli.rs`

### Step 3: Update Run command with ErrorRecovery (15 min)
**File**: `crates/chat-cli/src/cli/skills_cli.rs`
**Change**: Show recovery guide on errors
**Verify**: `grep "format_recovery_guide" crates/chat-cli/src/cli/skills_cli.rs`

### Step 4: Add Example command to enum (5 min)
**File**: `crates/chat-cli/src/cli/skills_cli.rs`
**Change**: Add `Example` variant
**Verify**: `grep "Example," crates/chat-cli/src/cli/skills_cli.rs`

### Step 5: Add Example command handler (10 min)
**File**: `crates/chat-cli/src/cli/skills_cli.rs`
**Change**: Implement Example handler
**Verify**: `grep "run_interactive_example" crates/chat-cli/src/cli/skills_cli.rs`

### Step 6: Update List command with tutorial (10 min)
**File**: `crates/chat-cli/src/cli/skills_cli.rs`
**Change**: Call `show_tutorial_if_needed`
**Verify**: `grep "show_tutorial_if_needed" crates/chat-cli/src/cli/skills_cli.rs`

### Step 7: Add Help command handler (10 min)
**File**: `crates/chat-cli/src/cli/skills_cli.rs`
**Change**: Implement Help handler
**Verify**: `grep "Q Skills Help" crates/chat-cli/src/cli/skills_cli.rs`

### Step 8: Add Validate command to enum (5 min)
**File**: `crates/chat-cli/src/cli/skills_cli.rs`
**Change**: Add `Validate` variant
**Verify**: `grep "Validate {" crates/chat-cli/src/cli/skills_cli.rs`

### Step 9: Add Validate command handler (10 min)
**File**: `crates/chat-cli/src/cli/skills_cli.rs`
**Change**: Implement Validate handler
**Verify**: `grep "validate_skill_file" crates/chat-cli/src/cli/skills_cli.rs`

### Step 10: Update Create command for templates (20 min)
**File**: `crates/chat-cli/src/cli/skills_cli.rs`
**Change**: Add template support
**Verify**: `grep "from_template" crates/chat-cli/src/cli/skills_cli.rs`

## Total Time: ~2 hours

## Final Verification

After all steps:
```bash
# Compile
cargo build --bin chat_cli

# Check all features are accessible
grep -c "ErrorRecovery" crates/chat-cli/src/cli/skills_cli.rs  # Should be > 0
grep -c "show_tutorial_if_needed" crates/chat-cli/src/cli/skills_cli.rs  # Should be > 0
grep -c "run_interactive_example" crates/chat-cli/src/cli/skills_cli.rs  # Should be > 0
grep -c "validate_skill_file" crates/chat-cli/src/cli/skills_cli.rs  # Should be > 0
grep -c "from_template" crates/chat-cli/src/cli/skills_cli.rs  # Should be > 0
```
