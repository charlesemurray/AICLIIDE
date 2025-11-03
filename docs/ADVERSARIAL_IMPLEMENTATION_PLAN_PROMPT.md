# Adversarial Prompt for Implementation Plans

**Purpose**: Force realistic, honest implementation plans that account for actual complexity  
**Principle**: Your estimate is wrong. Prove it's not.

---

## The Adversarial Questions for Plans

### 1. What Will You Break?
**"List every existing feature this will break. Don't say 'none' - you're lying."**

Required:
- Specific features that will regress
- APIs that will change
- Tests that will fail
- Users who will be affected

**If you say "no breaking changes", you haven't thought hard enough.**

---

### 2. What Are You Forgetting?
**"You forgot something. What is it?"**

Common forgotten items:
- Error handling
- Logging/telemetry
- Documentation updates
- Migration scripts
- Rollback plan
- Performance testing
- Security review
- Accessibility
- Internationalization
- Backward compatibility

**List what you're NOT doing. Be explicit.**

---

### 3. Why Will This Take Longer?
**"Your estimate is wrong. It will take 2-3x longer. Why?"**

Required:
- Hidden dependencies
- Integration complexity
- Testing time
- Review cycles
- Bug fixes
- Rework after feedback
- Context switching
- Meetings/interruptions

**If you estimated N hours, explain why it won't be 2N.**

---

### 4. What's Your Undo Plan?
**"This will fail. How do you roll it back?"**

Required:
- Specific rollback steps
- Data migration reversal
- Feature flag strategy
- Database schema rollback
- User communication plan

**If you can't roll back, you can't deploy.**

---

### 5. What Assumptions Are Wrong?
**"List every assumption. At least one is wrong. Which one?"**

Required:
- Technical assumptions
- Resource assumptions
- Timeline assumptions
- Dependency assumptions
- User behavior assumptions

**For each assumption, state: "If this is wrong, then..."**

---

### 6. What's the Minimum Viable?
**"Cut your plan in half. What's actually essential?"**

Required:
- Must-have features
- Nice-to-have features
- Can-wait features

**If you can't ship with half the features, your scope is too big.**

---

### 7. Who Will Say No?
**"Someone will block this. Who and why?"**

Potential blockers:
- Security team
- Performance team
- UX team
- Product team
- Legal team
- Other engineers

**For each blocker, state mitigation plan.**

---

### 8. What's Your Proof of Concept?
**"Build the riskiest part first. What is it?"**

Required:
- Highest risk component
- Proof of concept plan
- Success criteria
- Failure criteria
- Time box (max 2 hours)

**If you can't prove it works in 2 hours, your plan is too risky.**

---

## Implementation Plan Template

### Executive Summary
```
WHAT: [One sentence]
WHY: [One sentence - business value]
RISK: [High/Medium/Low + why]
TIME: [Realistic estimate + confidence level]
```

### What Will Break
```
BREAKING CHANGES:
- Feature X will stop working because...
- API Y will change signature...
- Test Z will fail because...

AFFECTED USERS:
- User group A (N users)
- User group B (M users)

MITIGATION:
- How we'll handle each breaking change
```

### Scope
```
MUST HAVE (MVP):
- [ ] Feature A (2h)
- [ ] Feature B (3h)

NICE TO HAVE (V2):
- [ ] Feature C (4h)
- [ ] Feature D (2h)

NOT DOING:
- Feature E (out of scope)
- Feature F (too risky)
```

### Dependencies
```
REQUIRES:
- Library X (exists? version?)
- Service Y (available? SLA?)
- Team Z approval (timeline?)

BLOCKS:
- Feature A (who's waiting?)
- Team B (what do they need?)
```

### Risks
```
HIGH RISK:
- Risk 1: [description]
  - Probability: [%]
  - Impact: [severity]
  - Mitigation: [plan]

MEDIUM RISK:
- Risk 2: [description]
  - Mitigation: [plan]

ASSUMPTIONS (at least one is wrong):
- Assumption 1: [if wrong, then...]
- Assumption 2: [if wrong, then...]
```

### Proof of Concept
```
RISKIEST PART: [what]
POC PLAN: [how to prove in 2h]
SUCCESS: [what proves it works]
FAILURE: [what proves it won't work]
```

### Rollback Plan
```
IF THIS FAILS:
1. [Immediate action]
2. [Data rollback]
3. [Feature flag disable]
4. [User communication]

ROLLBACK TIME: [how long]
DATA LOSS: [yes/no + what]
```

### Timeline
```
OPTIMISTIC: [N hours]
REALISTIC: [2N hours]
PESSIMISTIC: [3N hours]

CONFIDENCE: [%]

WHY IT WILL TAKE LONGER:
- [Reason 1]
- [Reason 2]
- [Reason 3]
```

### What We're NOT Doing
```
EXPLICITLY OUT OF SCOPE:
- [ ] Error handling for edge case X
- [ ] Performance optimization
- [ ] Comprehensive testing
- [ ] Documentation
- [ ] Migration script

TECHNICAL DEBT CREATED:
- [Debt 1]
- [Debt 2]
```

### Review Checklist
```
BEFORE STARTING:
- [ ] Security review
- [ ] Performance review
- [ ] UX review
- [ ] Architecture review

BEFORE SHIPPING:
- [ ] Unit tests
- [ ] Integration tests
- [ ] Manual testing
- [ ] Documentation
- [ ] Rollback tested
```

---

## Red Flags in Plans

### Optimistic Estimates
❌ "This will take 2 hours"  
✅ "Optimistic: 2h, Realistic: 4h, Pessimistic: 8h"

### No Breaking Changes
❌ "No breaking changes"  
✅ "Breaks feature X for users Y, mitigated by Z"

### Perfect Assumptions
❌ "Assuming library X works"  
✅ "Assuming library X works. If wrong, we need plan B (4h)"

### No Risks
❌ "Low risk"  
✅ "High risk: thread safety. Medium risk: performance. Mitigations: ..."

### Everything In Scope
❌ "Will implement A, B, C, D, E"  
✅ "MVP: A, B. V2: C, D. Not doing: E"

### No Rollback
❌ "Will deploy to production"  
✅ "Rollback: disable feature flag, revert schema, notify users"

---

## The Test

### Question 1: Can you ship half of this?
If no → Scope too big

### Question 2: Can you prove the risky part in 2 hours?
If no → Too risky

### Question 3: Can you roll back in 5 minutes?
If no → Too dangerous

### Question 4: What's your confidence level?
If >80% → You're lying to yourself

### Question 5: What will break?
If "nothing" → You haven't thought it through

---

## Example: Bad Plan

```
# Feedback System Implementation

## Plan
1. Add FeedbackManager
2. Implement CLI commands
3. Test and deploy

## Timeline
2 hours

## Risk
Low
```

**Problems:**
- No breaking changes listed
- No assumptions stated
- No rollback plan
- Optimistic timeline
- No scope definition
- No proof of concept

---

## Example: Good Plan

```
# Feedback System Implementation

## Executive Summary
WHAT: Add user feedback for memory quality
WHY: Improve recall quality through user signals
RISK: Medium (thread safety, validation gaps)
TIME: Optimistic 2h, Realistic 4h, Pessimistic 8h (50% confidence)

## What Will Break
BREAKING CHANGES:
- None (additive feature)

AFFECTED USERS:
- 0 (new feature)

BUT WILL FAIL IF:
- Concurrent feedback writes (no locking)
- Invalid memory IDs (no validation)
- Database corruption (no transactions)

## Scope
MUST HAVE (MVP):
- [ ] FeedbackManager with SQLite (1h)
- [ ] CLI command handler (30m)
- [ ] Basic error handling (30m)

NOT DOING (V1):
- Thread safety (needs connection pool - 2h)
- Validation (needs cortex integration - 1h)
- Transactions (needs refactor - 1h)
- Comprehensive tests (needs 2h)

TECHNICAL DEBT:
- No thread safety
- No validation
- No transactions

## Risks
HIGH RISK:
- Thread safety: Connection not Send+Sync
  - Probability: 80%
  - Impact: Database corruption
  - Mitigation: Document limitation, add TODO

MEDIUM RISK:
- Invalid memory IDs
  - Probability: 50%
  - Impact: Garbage data
  - Mitigation: Basic error handling

ASSUMPTIONS (at least one is wrong):
- Assume single-threaded access (WRONG if async)
- Assume valid memory IDs (WRONG if user error)
- Assume writable database path (WRONG if permissions)

## Proof of Concept
RISKIEST PART: SQLite integration
POC PLAN: Create FeedbackManager, write/read one entry (30m)
SUCCESS: Can record and retrieve feedback
FAILURE: SQLite errors, can't write

## Rollback Plan
IF THIS FAILS:
1. Remove feedback command from CLI
2. No data loss (new feature)
3. No schema rollback needed
4. Document known issues

ROLLBACK TIME: 5 minutes
DATA LOSS: No

## Timeline
OPTIMISTIC: 2h (everything works first try)
REALISTIC: 4h (bugs, testing, review)
PESSIMISTIC: 8h (major issues, rework)

CONFIDENCE: 50%

WHY IT WILL TAKE LONGER:
- Thread safety issues discovered
- Validation edge cases
- Integration with existing code
- Review feedback
- Documentation

## What We're NOT Doing
- Thread safety (V2)
- Validation (V2)
- Transactions (V2)
- Comprehensive tests (V2)
- Performance optimization (V2)
```

**Why This Is Better:**
- Honest about risks
- Explicit scope cuts
- Realistic timeline
- Clear rollback
- States assumptions
- Identifies technical debt

---

## The Meta-Rule

**Your plan is wrong. Make it less wrong by being honest about:**
1. What will break
2. What you're forgetting
3. Why it will take longer
4. How you'll roll back
5. Which assumptions are wrong
6. What's actually essential
7. Who will block you
8. What's the riskiest part

---

## Success Criteria

A plan passes if:
- ✅ Lists breaking changes (or explains why none)
- ✅ States assumptions and "if wrong" scenarios
- ✅ Has 3 timeline estimates (optimistic/realistic/pessimistic)
- ✅ Has rollback plan with time estimate
- ✅ Explicitly lists what's NOT being done
- ✅ Identifies riskiest part with POC plan
- ✅ Confidence level <80%

A plan fails if:
- ❌ Says "no breaking changes" without proof
- ❌ Single timeline estimate
- ❌ No rollback plan
- ❌ Everything in scope
- ❌ No risks identified
- ❌ Confidence >80%

---

**Use this to make plans that survive contact with reality.**
