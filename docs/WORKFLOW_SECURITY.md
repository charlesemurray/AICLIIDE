# Workflow Security Hardening

## Implemented Protections

### 1. Cycle Detection (Critical)
**Problem:** Workflows could invoke themselves recursively, causing infinite loops and crashes.

**Solution:** Added `petgraph` library for graph cycle detection.
- Validates workflow dependencies at load time
- Detects self-referencing steps
- Prevents stack overflow and OOM crashes

**Code:** `validation.rs:detect_cycles()`

**Example Blocked:**
```json
{
  "steps": [
    {"name": "step1", "tool": "step1"}  // Self-reference blocked
  ]
}
```

### 2. Execution Timeout (Critical)
**Problem:** Hung workflows could block the terminal indefinitely.

**Solution:** Added 5-minute timeout per workflow step.
- Uses `tokio::time::timeout()`
- Returns clear error message on timeout
- Prevents terminal lockup

**Code:** `workflow.rs:STEP_TIMEOUT`, line 355-377

**Example:**
```
Step 'long-running' timed out after 300 seconds
```

### 3. Permission Checking (Critical)
**Problem:** LLM could generate workflows that execute dangerous operations without user consent.

**Solution:** Changed workflow permission from `Allow` to `Ask`.
- User must approve workflow execution
- Protects against accidental `rm -rf` commands
- Applies to all LLM-generated workflows

**Code:** `workflow.rs:eval_perm()` returns `PermissionEvalResult::Ask`

**User Experience:**
```
Q wants to run workflow 'cleanup-files'
This workflow will:
  - Execute bash commands
  - Write to filesystem

Allow? [y/N]
```

## Threat Model

**Context:** Local CLI tool, single user, developer machine

**Protected Against:**
1. ✅ LLM generating infinite recursion workflows
2. ✅ LLM generating hung workflows that block terminal
3. ✅ LLM generating dangerous operations without approval
4. ✅ User accidentally running malicious workflows

**Not Protected Against:**
- User explicitly approving malicious workflow (user has shell access anyway)
- Command injection in user-created workflows (user controls the workflow)
- Concurrent workflow modifications (single-threaded CLI)

## Remaining Considerations

### Low Priority (Not Implemented)
These were considered but deemed unnecessary for local CLI context:

1. **Atomic Transactions** - Overkill for local automation scripts
2. **Thread Safety** - Single CLI session, no concurrent access
3. **Command Injection Sanitization** - User is injecting into their own shell
4. **Write-Ahead Log** - Not a database, no need for durability

### Future Enhancements
If workflows become more complex:

1. **Workflow-level timeout** - Currently per-step, could add total workflow timeout
2. **Resource limits** - Memory/CPU limits for workflow execution
3. **Audit logging** - Track all workflow executions for debugging
4. **Dry-run mode** - Preview workflow actions before execution

## Testing

### Manual Tests
1. Create workflow with self-reference → Should fail validation
2. Create workflow with 10-minute sleep → Should timeout after 5 minutes
3. LLM generates workflow with `execute_bash` → Should ask permission

### Automated Tests
See `validation.rs` tests for cycle detection validation.

## Dependencies

- `petgraph = "0.6"` - Graph algorithms for cycle detection
- `tokio::time` - Async timeout support (already in project)

## References

- Adversary Review: Found 8 critical/high issues
- This implementation addresses: Critical #1, #2, #3
- Remaining issues deemed low priority for local CLI context
