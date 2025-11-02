# No Placeholders - Quick Reference

## Before Every Commit

Run these commands:
```bash
# Search for placeholders
grep -r "TODO" crates/chat-cli/src/session/
grep -r "FIXME" crates/chat-cli/src/session/
grep -r "unimplemented!" crates/chat-cli/src/session/
grep -r "todo!" crates/chat-cli/src/session/

# Should return ZERO results in implementation code
# (TODOs in tests or comments are OK)
```

## Validation Commands

```bash
# Must all pass
cargo test
cargo check
cargo clippy
cargo fmt --check
cargo test --doc
```

## Red Flags

### ❌ Forbidden Patterns
- `TODO: implement this`
- `unimplemented!()`
- `todo!()`
- `panic!("not implemented")`
- Functions that just return `Ok(())`
- Empty match arms
- `// Will add later`
- `// Placeholder`

### ✅ Required Patterns
- All error cases handled
- All match arms implemented
- All trait methods complete
- Rustdoc on public items
- Tests for happy and error paths
- User-friendly error messages

## Step Completion Criteria

A step is done when:
1. ✅ All tests pass
2. ✅ Zero compiler warnings
3. ✅ Zero clippy warnings
4. ✅ Zero placeholder searches
5. ✅ Manual testing works
6. ✅ Code review passes
7. ✅ User approves

**If ANY fails → Step is NOT done**

## Quick Self-Review

Ask yourself:
- [ ] Would I deploy this to production right now?
- [ ] Is every function fully implemented?
- [ ] Are all error cases handled?
- [ ] Can I delete any "TODO" comments?
- [ ] Are there any shortcuts I took?

If you answer "no" to any → **Not ready to commit**

## Example: Complete vs Incomplete

### ❌ Incomplete
```rust
pub fn validate_name(name: &str) -> Result<()> {
    // TODO: add validation
    Ok(())
}
```

### ✅ Complete
```rust
/// Validates a session name
/// 
/// # Rules
/// - 1-100 characters
/// - Alphanumeric, dash, underscore only
/// 
/// # Errors
/// Returns `SessionError::InvalidName` if validation fails
pub fn validate_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(SessionError::InvalidName("Name cannot be empty".into()));
    }
    if name.len() > 100 {
        return Err(SessionError::InvalidName("Name too long (max 100)".into()));
    }
    if !name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err(SessionError::InvalidName(
            "Only alphanumeric, dash, and underscore allowed".into()
        ));
    }
    Ok(())
}
```

## Enforcement

Before user signoff, reviewer must verify:
1. Run all validation commands
2. Search for placeholders
3. Read implementation code
4. Verify tests cover all paths
5. Check error handling
6. Confirm documentation exists

**No exceptions. No shortcuts.**
