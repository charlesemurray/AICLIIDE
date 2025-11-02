# How to Verify Phase 2 Works

## Quick Verification

### 1. Run the Tests (Fastest)

```bash
cd /local/workspace/q-cli/amazon-q-developer-cli

# Run all prompt system tests (75 tests)
cargo test --package chat_cli --lib prompt_system

# Run CLI parsing tests (3 tests)
cargo test --package chat_cli --lib assistant_cli

# Run integration tests (2 tests)
cargo test --package chat_cli --lib skill_prompt_integration
```

**Expected Output:**
```
test result: ok. 75 passed; 0 failed; 0 ignored
test result: ok. 3 passed; 0 failed; 0 ignored
test result: ok. 2 passed; 0 failed; 0 ignored
```

### 2. Check the CLI Command Exists

```bash
# Build the CLI
cargo build --package chat_cli

# Check help shows the new command
cargo run --bin chat_cli -- create --help
```

**Expected Output:**
```
Create a new skill, command, agent, or assistant

Usage: q create <COMMAND>

Commands:
  skill      Create a new skill
  command    Create a new custom command
  agent      Create a new agent
  assistant  Create an AI assistant (interactive prompt builder)  ← NEW!
  help       Print this message or the help of the given subcommand(s)
```

### 3. Test the Assistant Command Parsing

```bash
# Test basic command
cargo run --bin chat_cli -- create assistant --help

# Test template mode
cargo run --bin chat_cli -- create assistant template --help

# Test custom mode
cargo run --bin chat_cli -- create assistant custom --help
```

**Expected Output:**
```
Create an AI assistant (interactive prompt builder)

Usage: q create assistant [COMMAND]

Commands:
  template  Use a pre-built template
  custom    Build from scratch
  help      Print this message or the help of the given subcommand(s)
```

## Detailed Verification

### Run Individual Test Suites

```bash
# 1. Builder tests (7 tests)
cargo test --package chat_cli --lib prompt_system::builder_tests

# 2. Interactive tests (5 tests)
cargo test --package chat_cli --lib prompt_system::interactive_tests

# 3. Integration tests (8 tests)
cargo test --package chat_cli --lib prompt_system::integration_tests

# 4. Performance tests (12 tests)
cargo test --package chat_cli --lib prompt_system::performance_tests

# 5. CLI tests (3 tests)
cargo test --package chat_cli --lib assistant_cli
```

### Check Code Compiles

```bash
# Check the creation module compiles
cargo check --package chat_cli

# Look for our specific files
ls -la crates/chat-cli/src/cli/creation/flows/skill_prompt_integration.rs
ls -la crates/chat-cli/src/cli/creation/tests/assistant_cli.rs
ls -la crates/chat-cli/src/cli/creation/prompt_system/interactive.rs
```

## What Each Test Verifies

### Prompt System Tests (75 tests)
- ✅ PromptBuilder creates valid templates
- ✅ CommandBuilder creates valid commands
- ✅ InteractivePromptBuilder guides users
- ✅ Template selection works
- ✅ Custom creation works
- ✅ Validation provides feedback
- ✅ Quality scoring works
- ✅ Performance is fast (<20ms)
- ✅ Memory is stable

### CLI Tests (3 tests)
- ✅ `q create assistant` parses correctly
- ✅ `q create assistant template` parses correctly
- ✅ `q create assistant custom` parses correctly

### Integration Tests (2 tests)
- ✅ Template creation flow works end-to-end
- ✅ Custom creation flow works end-to-end

## Quick Smoke Test

Run this single command to verify everything:

```bash
cd /local/workspace/q-cli/amazon-q-developer-cli && \
cargo test --package chat_cli --lib prompt_system assistant_cli 2>&1 | \
grep "test result"
```

**Expected:**
```
test result: ok. 75 passed; 0 failed; 0 ignored
test result: ok. 3 passed; 0 failed; 0 ignored
```

## Visual Verification

### Check the Code Structure

```bash
# List all new files
find crates/chat-cli/src/cli/creation -name "*prompt*" -o -name "*assistant*" | grep -E "(interactive|integration|assistant_cli)"
```

**Expected:**
```
crates/chat-cli/src/cli/creation/prompt_system/interactive.rs
crates/chat-cli/src/cli/creation/prompt_system/interactive_tests.rs
crates/chat-cli/src/cli/creation/flows/skill_prompt_integration.rs
crates/chat-cli/src/cli/creation/tests/assistant_cli.rs
```

### Check Test Count

```bash
# Count total tests
cargo test --package chat_cli --lib prompt_system assistant_cli --no-run 2>&1 | \
grep -E "test.*::" | wc -l
```

**Expected:** ~78 tests

## Troubleshooting

### If tests fail:
```bash
# Get detailed output
cargo test --package chat_cli --lib prompt_system -- --nocapture

# Check specific test
cargo test --package chat_cli --lib prompt_system::interactive_tests::test_create_from_template_code_reviewer -- --nocapture
```

### If command doesn't show:
```bash
# Rebuild
cargo clean
cargo build --package chat_cli

# Check again
cargo run --bin chat_cli -- create --help
```

## Success Criteria

✅ All tests pass (75+ tests)
✅ CLI command parses correctly
✅ Help text shows assistant command
✅ Code compiles without errors
✅ Performance tests pass (<20ms)
✅ Memory tests pass (no leaks)

## What You Should See

When everything works:
- ✅ 75 prompt_system tests passing
- ✅ 3 assistant_cli tests passing
- ✅ 2 skill_prompt_integration tests passing
- ✅ `q create assistant` in help output
- ✅ Clean compilation (warnings OK, no errors)
- ✅ All files present

---

**Quick Verification Command:**
```bash
cd /local/workspace/q-cli/amazon-q-developer-cli && \
cargo test --package chat_cli --lib prompt_system 2>&1 | tail -3
```

**Expected:**
```
test result: ok. 75 passed; 0 failed; 0 ignored; 0 measured; XXX filtered out; finished in 0.0Xs
```

If you see this, **Phase 2 is working!** ✅
