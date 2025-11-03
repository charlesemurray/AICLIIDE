# Adversarial Prompt: Implementation Discipline

**Purpose**: Enforce engineering discipline through TDD, incremental progress, and continuous verification  
**Principle**: If you can't prove each step works, you didn't do the step

---

## The Core Rules

### Rule 1: Red-Green-Commit
**Every implementation follows this cycle. No exceptions.**

```
1. Write test (must compile, must fail) → RED
2. Write minimal code (must compile, must pass) → GREEN
3. Commit with message → COMMIT
4. Repeat
```

**If you skip any step, start over.**

---

## The Adversarial Questions

### Before You Start

**"Show me the test that will prove this works."**

Not allowed:
- "I'll write tests after"
- "I'll test it manually"
- "The existing tests cover it"

Required:
- Actual test code
- Test compiles
- Test fails with expected error

**If you don't have a failing test, you can't start coding.**

---

### After Each Step

**"Prove this step is complete."**

Required evidence:
```bash
# 1. Tests compile
$ cargo test --no-run
   Compiling package v1.0.0
   Finished test [unoptimized + debuginfo] target(s)

# 2. Tests fail (RED)
$ cargo test test_name
test test_name ... FAILED

# 3. Code compiles
$ cargo check
   Finished dev [unoptimized + debuginfo] target(s)

# 4. Tests pass (GREEN)
$ cargo test test_name
test test_name ... ok

# 5. All tests still pass
$ cargo test
test result: ok. N passed; 0 failed

# 6. Git commit
$ git add -A
$ git commit -m "feat: add feature X with test"
[branch abc123] feat: add feature X with test
```

**If you can't show all 6, the step isn't done.**

---

### Before Claiming Done

**"Show me there are no placeholders."**

Search for:
```bash
$ grep -r "TODO" src/
$ grep -r "FIXME" src/
$ grep -r "stub" src/
$ grep -r "placeholder" src/
$ grep -r "simplified" src/
$ grep -r "unimplemented!" src/
```

**If any match, you're not done.**

---

### Before Moving On

**"Show me the git log."**

Required:
```bash
$ git log --oneline -5
abc123 feat: add feature X with test
def456 test: add test for feature X
ghi789 refactor: extract helper function
...
```

**Each commit must:**
- Be atomic (one logical change)
- Have meaningful message
- Leave code in working state
- Pass all tests

**If commits are missing or messy, you didn't follow discipline.**

---

## The Implementation Checklist

### For Each Feature

```
STEP 1: Write Test
- [ ] Test file created/modified
- [ ] Test compiles: `cargo test --no-run`
- [ ] Test fails: `cargo test test_name` → FAILED
- [ ] Commit: "test: add test for X"

STEP 2: Minimal Implementation
- [ ] Code compiles: `cargo check`
- [ ] Test passes: `cargo test test_name` → ok
- [ ] All tests pass: `cargo test` → all ok
- [ ] Commit: "feat: implement X"

STEP 3: Verify No Placeholders
- [ ] No TODO comments
- [ ] No stub implementations
- [ ] No unimplemented!() calls
- [ ] No "simplified" versions

STEP 4: Integration Check
- [ ] Feature works end-to-end
- [ ] No compilation warnings
- [ ] No test warnings
- [ ] Commit: "test: add integration test for X"

STEP 5: Documentation
- [ ] Code comments added
- [ ] Public API documented
- [ ] Commit: "docs: document X"
```

---

## Red Flags

### You're Doing It Wrong If:

❌ **"I'll write tests later"**
- Tests come first, always

❌ **"The code compiles, that's good enough"**
- Tests must pass too

❌ **"I'll commit when it's all done"**
- Commit after each working step

❌ **"This is just a placeholder for now"**
- No placeholders. Ever.

❌ **"I'll clean up the TODOs later"**
- No TODOs in committed code

❌ **"The test passes, but I haven't run all tests"**
- All tests must pass, always

❌ **"I made a big commit with everything"**
- Commits must be atomic

---

## The Discipline Test

### Question 1: Can you show me the failing test?
If no → You didn't do TDD

### Question 2: Can you show me each commit?
If no → You didn't work incrementally

### Question 3: Can you show me all tests passing?
If no → You broke something

### Question 4: Can you show me zero TODOs?
If no → You left placeholders

### Question 5: Can you show me clean compilation?
If no → You have warnings

### Question 6: Can you roll back one commit?
If no → Your commits aren't atomic

---

## Example: Good Discipline

```bash
# Step 1: Write test
$ cat src/feedback.rs
#[test]
fn test_record_feedback() {
    let mgr = FeedbackManager::new("test.db").unwrap();
    mgr.record_feedback("id1", true).unwrap();
    let feedback = mgr.get_feedback("id1").unwrap();
    assert_eq!(feedback.helpful, true);
}

$ cargo test test_record_feedback
   Compiling package v1.0.0
test test_record_feedback ... FAILED
error: cannot find function `record_feedback`

$ git add src/feedback.rs
$ git commit -m "test: add test for record_feedback"

# Step 2: Minimal implementation
$ cat src/feedback.rs
impl FeedbackManager {
    pub fn record_feedback(&self, id: &str, helpful: bool) -> Result<()> {
        self.conn.execute(
            "INSERT INTO feedback (id, helpful) VALUES (?1, ?2)",
            [id, &helpful.to_string()],
        )?;
        Ok(())
    }
}

$ cargo test test_record_feedback
test test_record_feedback ... ok

$ cargo test
test result: ok. 46 passed; 0 failed

$ git add src/feedback.rs
$ git commit -m "feat: implement record_feedback"

# Step 3: Verify no placeholders
$ grep -r "TODO" src/
(no output)

$ grep -r "stub" src/
(no output)

# Done with this step
```

---

## Example: Bad Discipline

```bash
# Wrong: No test first
$ cat src/feedback.rs
impl FeedbackManager {
    pub fn record_feedback(&self, id: &str, helpful: bool) -> Result<()> {
        // TODO: implement this
        Ok(())
    }
}

$ cargo check
   Finished dev [unoptimized + debuginfo] target(s)

$ git commit -m "WIP: feedback stuff"

# Problems:
# 1. No test written first
# 2. TODO placeholder
# 3. Didn't verify tests pass
# 4. Vague commit message
# 5. Can't prove it works
```

---

## The Enforcement Mechanism

### Before Accepting Any Work

Run this checklist:

```bash
# 1. All tests pass
$ cargo test
test result: ok. N passed; 0 failed; 0 ignored

# 2. Code compiles cleanly
$ cargo check
   Finished dev [unoptimized + debuginfo] target(s)
(no warnings)

# 3. No placeholders
$ grep -r "TODO\|FIXME\|stub\|unimplemented!" src/
(no output)

# 4. Git history is clean
$ git log --oneline -10
(atomic commits with clear messages)

# 5. Each commit passes tests
$ git rebase -i HEAD~10 --exec "cargo test"
(all commits pass)
```

**If any check fails, work is not complete.**

---

## The Meta-Rule

**Every step must be provable:**
1. Test exists and fails → Prove with `cargo test` output
2. Code compiles → Prove with `cargo check` output
3. Test passes → Prove with `cargo test` output
4. All tests pass → Prove with `cargo test` output
5. No placeholders → Prove with `grep` output
6. Committed → Prove with `git log` output

**If you can't prove it, you didn't do it.**

---

## Common Excuses (All Invalid)

### "I'll write tests after I get it working"
❌ Wrong order. Test first, always.

### "I'll clean up the TODOs before merging"
❌ No TODOs in any commit. Ever.

### "I'll make smaller commits next time"
❌ Make smaller commits this time. Rebase if needed.

### "The tests are flaky"
❌ Fix the tests. Flaky tests = broken tests.

### "I need to commit this WIP to switch branches"
❌ Use `git stash`. Never commit broken code.

### "This is just a prototype"
❌ Prototypes follow the same discipline or get thrown away.

---

## Success Criteria

### A step is complete when:
- ✅ Test written and fails
- ✅ Code written and test passes
- ✅ All tests pass
- ✅ No TODOs, FIXMEs, stubs
- ✅ No compilation warnings
- ✅ Committed with clear message
- ✅ Can be rolled back cleanly

### A feature is complete when:
- ✅ All steps complete
- ✅ Integration test passes
- ✅ Documentation written
- ✅ No placeholders anywhere
- ✅ Git history is clean
- ✅ Can demo working feature

---

## The Adversarial Prompt

Use this before claiming any work is done:

```
PROVE IT:
1. Show me the test that failed
2. Show me the test passing
3. Show me all tests passing
4. Show me zero TODOs
5. Show me the git commits
6. Show me clean compilation

If you can't show all 6, you're not done.
```

---

## Application to Feedback Integration

### What I Should Have Done:

```bash
# Step 1: Test for FeedbackManager field
$ cat tests/session_test.rs
#[test]
fn test_session_has_feedback_manager() {
    let session = ChatSession::new(...);
    assert!(session.feedback_manager.is_some());
}

$ cargo test test_session_has_feedback_manager
FAILED (field doesn't exist)

$ git commit -m "test: add test for feedback_manager field"

# Step 2: Add field
$ cat src/session.rs
pub struct ChatSession {
    feedback_manager: Option<FeedbackManager>,
}

$ cargo test test_session_has_feedback_manager
ok

$ cargo test
ok. 45 passed

$ git commit -m "feat: add feedback_manager field to ChatSession"

# Step 3: Test for initialization
# Step 4: Implement initialization
# ... etc
```

### What I Actually Did:

```bash
# Wrong: Added everything at once
$ git diff
(500 lines of changes)

$ cargo test
(didn't run - chat_cli doesn't compile)

$ git commit -m "feat: add feedback system"
(one big commit, can't verify each step)
```

**I violated every rule.**

---

## The Bottom Line

**Engineering discipline is not optional:**
- TDD is not optional
- Incremental commits are not optional
- Continuous verification is not optional
- No placeholders is not optional

**If you skip discipline, you create technical debt.**

**Use this prompt to enforce discipline at every step.**

---

## Quick Reference Card

```
BEFORE CODING:
□ Write test
□ Test compiles
□ Test fails

AFTER CODING:
□ Code compiles
□ Test passes
□ All tests pass
□ No TODOs
□ Commit

BEFORE CLAIMING DONE:
□ Show failing test
□ Show passing test
□ Show all tests passing
□ Show zero placeholders
□ Show git log
□ Show clean compilation
```

**Print this. Check every box. Every time.**
