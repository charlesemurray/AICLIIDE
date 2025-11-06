# Workflow Integration Test Checklist

## Status: ⚠️ NOT TESTED

The workflow system has been implemented but **not yet tested** due to pre-existing compilation errors in the codebase.

## What Was Implemented

### Core Features
- ✅ Workflow execution engine
- ✅ Skill invocation from workflows
- ✅ CLI commands (/workflows list, run, add, remove, info)
- ✅ LLM integration (workflows in tool schema)
- ✅ Cycle detection (petgraph)
- ✅ Execution timeout (5 minutes per step)
- ✅ Permission checking (workflows require approval)

### Test Files Created
- ✅ `.q-workflows/test-skill-execution.json` - Tests skill invocation
- ✅ `.q-workflows/test-cycle-detection.json` - Tests cycle detection
- ✅ `.q-workflows/test-timeout.json` - Tests timeout protection
- ✅ `.q-workflows/test-dangerous-ops.json` - Tests permission checking
- ✅ `tests/workflow_integration_test.rs` - Automated test suite

## Manual Test Plan

### Test 1: Basic Workflow Execution
**File:** `.q-workflows/test-skill-invocation.json`

**Command:**
```bash
/workflows run test-skill-invocation
```

**Expected Output:**
```
🔄 Workflow 'test-skill-invocation' completed

Executed 1 steps successfully in X.XXms

Step 'calculate': 8 (completed in X.XXms)
```

**Success Criteria:**
- [ ] Workflow executes without errors
- [ ] Shows actual result (8) not just "Executed tool"
- [ ] Completes in < 1 second

---

### Test 2: Multi-Step Skill Execution
**File:** `.q-workflows/test-skill-execution.json`

**Command:**
```bash
/workflows run test-skill-execution
```

**Expected Output:**
```
🔄 Workflow 'test-skill-execution' completed

Executed 2 steps successfully in X.XXms

Step 'add': 15 (completed in X.XXms)
Step 'multiply': 21 (completed in X.XXms)
```

**Success Criteria:**
- [ ] Both steps execute in order
- [ ] Correct results: 10+5=15, 3*7=21
- [ ] No errors

---

### Test 3: Cycle Detection
**File:** `.q-workflows/test-cycle-detection.json`

**Command:**
```bash
/workflows add .q-workflows/test-cycle-detection.json
```

**Expected Output:**
```
❌ Invalid workflow: Workflow contains cycles - steps cannot reference each other
```

**Success Criteria:**
- [ ] Workflow rejected at validation time
- [ ] Error message mentions "cycle"
- [ ] Workflow NOT added to registry

---

### Test 4: Execution Timeout
**File:** `.q-workflows/test-timeout.json`

**Command:**
```bash
/workflows run test-timeout
```

**Expected Behavior:**
- Workflow starts executing
- After 5 minutes, times out with error

**Expected Output:**
```
❌ Workflow 'test-timeout' failed: Step 'long-sleep' timed out after 300 seconds
```

**Success Criteria:**
- [ ] Workflow times out after 5 minutes (not 10)
- [ ] Error message mentions timeout
- [ ] Terminal not hung, can continue using Q

**Note:** This test takes 5 minutes. Skip if time-constrained.

---

### Test 5: Permission Checking (LLM)
**Scenario:** LLM tries to create and run a workflow

**Steps:**
1. Start Q chat
2. Ask: "Create a workflow that lists files in /tmp"
3. LLM should generate workflow with execute_bash
4. Q should ask for permission before running

**Expected Behavior:**
```
Q wants to run workflow 'list-tmp-files'
This workflow will execute bash commands.

Allow? [y/N]
```

**Success Criteria:**
- [ ] Permission prompt appears
- [ ] Workflow doesn't run without approval
- [ ] Approving with 'y' runs the workflow
- [ ] Denying with 'N' cancels execution

---

### Test 6: CLI Workflow Commands
**Commands to test:**

```bash
# List workflows
/workflows list

# Show workflow info
/workflows info test-skill-execution

# Add workflow
/workflows add examples/workflows/hello-workflow.json

# Remove workflow
/workflows remove hello-workflow

# Run workflow
/workflows run test-skill-execution
```

**Success Criteria:**
- [ ] All commands execute without errors
- [ ] List shows all workflows
- [ ] Info shows workflow details (steps, description)
- [ ] Add successfully loads workflow
- [ ] Remove deletes workflow
- [ ] Run executes workflow and shows results

---

### Test 7: Workflow from LLM
**Scenario:** LLM generates and executes workflow

**Steps:**
1. Start Q chat
2. Ask: "Calculate 5 + 3 using a workflow"
3. LLM should create workflow with calculator skill
4. Approve execution
5. Check result

**Expected Behavior:**
- LLM creates workflow definition
- Q asks permission
- Workflow executes
- Result is 8

**Success Criteria:**
- [ ] LLM can generate valid workflow JSON
- [ ] Workflow passes validation
- [ ] Execution returns correct result
- [ ] No errors or crashes

---

## Automated Tests

**File:** `tests/workflow_integration_test.rs`

**Status:** ❌ Cannot run due to pre-existing compilation errors

**Tests Included:**
- `test_cycle_detection_self_reference` - Validates cycle detection
- `test_valid_workflow_passes_validation` - Validates valid workflows pass
- `test_workflow_execution_basic` - Tests basic execution
- `test_workflow_timeout` - Tests timeout protection
- `test_workflow_permission_checking` - Tests permission system
- `test_empty_workflow_fails_validation` - Tests validation rules
- `test_invalid_name_fails_validation` - Tests name validation

**To Run (once compilation fixed):**
```bash
cargo test --test workflow_integration_test
```

---

## Integration Completeness Checklist

### Core Functionality
- [ ] Workflows execute successfully
- [ ] Skills invoked from workflows return results
- [ ] Multi-step workflows execute in order
- [ ] Context variables work ({{steps.step1.output}})

### Security
- [ ] Cycle detection prevents infinite loops
- [ ] Timeout prevents hung workflows
- [ ] Permission system asks before dangerous operations
- [ ] LLM-generated workflows require approval

### CLI Integration
- [ ] /workflows list shows all workflows
- [ ] /workflows run executes workflows
- [ ] /workflows add loads new workflows
- [ ] /workflows remove deletes workflows
- [ ] /workflows info shows details

### LLM Integration
- [ ] Workflows appear in tool schema
- [ ] LLM can invoke workflows
- [ ] LLM can generate workflow definitions
- [ ] Permission prompts work correctly

### Error Handling
- [ ] Invalid workflows rejected at validation
- [ ] Missing tools fail gracefully
- [ ] Timeout errors are clear
- [ ] Permission denials don't crash

---

## Known Issues

1. **Automated tests blocked** - Pre-existing compilation errors in `WorktreeInfo` and `SessionMgmtSubcommand`
2. **No actual test execution** - All tests are theoretical, none have been run
3. **Skill execution untested** - The fix for skill invocation (line 129) has not been verified
4. **Timeout untested** - 5-minute timeout has not been validated
5. **Permission flow untested** - LLM permission prompts not verified

---

## Definition of Done

The workflow integration is **COMPLETE** when:

- [ ] All manual tests pass
- [ ] Automated tests compile and pass
- [ ] LLM can successfully use workflows
- [ ] No crashes or hangs
- [ ] Security features (cycle, timeout, permissions) verified
- [ ] Documentation updated with test results

**Current Status:** 🔴 **NOT DONE** - Implementation complete, testing incomplete

---

## Next Steps

1. Fix pre-existing compilation errors in codebase
2. Run automated test suite
3. Execute manual test checklist
4. Fix any bugs discovered
5. Update this document with results
6. Mark integration as complete
