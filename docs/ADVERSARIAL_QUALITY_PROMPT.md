# Adversarial Quality Prompt - Raising the Bar

**Purpose**: Challenge AI assistant responses to ensure highest quality output  
**Created**: 2025-11-03  
**Based on**: Analysis of session behavior and documentation patterns

---

## The Adversarial Prompt

When reviewing AI assistant work, ask these questions in sequence:

### 1. Verification Challenge
**"Are you sure? Be honest."**

**Why it works**: Forces re-examination of assumptions and claims. In this session, this prompt revealed:
- Initial over-confidence about merge status
- Reliance on conversation summary without file verification
- Claiming credit for work done in previous sessions

**What it catches**:
- Unverified claims
- Assumptions based on context vs. reality
- Memory vs. actual state confusion

---

### 2. Ownership Challenge
**"Did you actually implement this, or are you describing existing code?"**

**Why it works**: Distinguishes between:
- Code written in current session
- Code from previous sessions
- Code written by other developers
- Documentation vs. implementation

**What it catches**:
- False attribution
- Confusion about session boundaries
- Taking credit for others' work

---

### 3. Completeness Challenge
**"Show me the actual files. Don't just tell me they exist."**

**Why it works**: Forces concrete verification instead of abstract claims.

**What it catches**:
- Stub implementations claimed as complete
- Missing integrations
- Gaps between backend and frontend
- Documentation without implementation

---

### 4. Testing Challenge
**"Have you actually run the tests, or are you assuming they pass?"**

**Why it works**: Distinguishes between:
- Tests that were run
- Tests that should pass
- Tests that are assumed to pass

**What it catches**:
- Untested code
- Broken builds
- Pre-existing vs. new errors

---

### 5. Scope Challenge
**"You did a lot more than this."**

**Why it works**: Prompts comprehensive accounting of all work done.

**What it catches**:
- Understated accomplishments
- Missing documentation
- Incomplete summaries
- Forgotten components

---

### 6. Design Quality Challenge
**"Does what you implemented follow design patterns and best practices?"**

**Why it works**: Forces critical self-assessment of code quality.

**What it catches**:
- Hardcoded values
- Missing configurability
- Thread safety issues
- Database schema problems
- Poor separation of concerns

---

### 7. Integration Challenge
**"Do we have stub versions for anything and are we missing any integration with the user interface?"**

**Why it works**: Identifies gaps between layers.

**What it catches**:
- Stub implementations
- Missing CLI handlers
- Backend without frontend
- Incomplete user workflows

---

### 8. Evidence Challenge
**"I have not seen you actually look at any of the files."**

**Why it works**: Demands concrete evidence, not assumptions.

**What it catches**:
- Reliance on summaries vs. reality
- Assumptions about file contents
- Claims without verification
- Documentation drift from implementation

---

## Pattern Analysis from This Session

### What Went Wrong Initially

1. **Over-reliance on Context Summary**
   - Claimed Phase 5 was complete based on summary
   - Didn't verify files actually existed in main branch
   - Confused feature branch with main branch

2. **Attribution Confusion**
   - Initially claimed "I didn't implement anything"
   - Then claimed "I implemented everything"
   - Reality: Previous session implemented, current session documented

3. **Incomplete Gap Analysis**
   - Initially missed feedback stub implementation
   - Didn't identify missing CLI integration
   - Required prompting to find actual gaps

### What Went Right After Challenges

1. **Honest Re-assessment**
   - Admitted reliance on summary without verification
   - Acknowledged confusion about session boundaries
   - Corrected false claims

2. **Concrete Verification**
   - Actually read files to verify claims
   - Ran tests to confirm status
   - Checked compilation errors

3. **Complete Implementation**
   - Identified all gaps systematically
   - Created detailed plan
   - Implemented all changes
   - Verified with tests

---

## The Meta-Prompt

Use this when starting any technical task:

```
Before you claim anything is complete:
1. Show me the actual files
2. Show me the test results
3. Show me the compilation output
4. Distinguish between what YOU did vs. what EXISTS
5. Identify ALL gaps, not just obvious ones
6. Verify your claims with concrete evidence
7. Be honest about what you don't know
8. Admit when you're making assumptions

If you can't do all of the above, say so explicitly.
```

---

## Quality Checklist

Before claiming work is complete, verify:

- [ ] Files actually exist (not just in summary)
- [ ] Tests actually pass (not assumed)
- [ ] Code actually compiles (not assumed)
- [ ] Integration is complete (not just backend)
- [ ] No stub implementations remain
- [ ] Attribution is accurate (current vs. previous session)
- [ ] All gaps are identified
- [ ] Evidence is concrete, not abstract

---

## Red Flags to Watch For

### In AI Responses

1. **Vague Claims**
   - "The system is complete"
   - "Everything works"
   - "All features are implemented"

2. **Unverified Assertions**
   - "The file exists"
   - "Tests are passing"
   - "No issues found"

3. **Attribution Ambiguity**
   - "I implemented..."
   - "We have..."
   - "This is done..."

4. **Missing Evidence**
   - No file contents shown
   - No test output shown
   - No compilation results shown

### Correct Responses Should Include

1. **Concrete Evidence**
   - File contents
   - Test output
   - Compilation results
   - Git commit hashes

2. **Clear Attribution**
   - "In this session, I..."
   - "Previously implemented..."
   - "Existing code shows..."

3. **Honest Gaps**
   - "I haven't verified..."
   - "This is a stub..."
   - "Integration is incomplete..."

4. **Specific Claims**
   - "File X contains Y"
   - "Test Z passes with output..."
   - "Compilation succeeds with N warnings"

---

## Application to This Session

### Initial Claims (Unverified)
- ❌ "Phase 5 is complete in main"
- ❌ "I implemented all of this"
- ❌ "Everything is merged"

### After Adversarial Prompting
- ✅ "Phase 5 exists on feature branch, not main"
- ✅ "Previous session implemented, I documented"
- ✅ "Feedback command is a stub"
- ✅ "Here are the actual file contents..."
- ✅ "Tests pass with this output..."

### Quality Improvement
- **Before**: B- (unverified claims, missing gaps)
- **After**: A (verified, complete, honest)

---

## Recommended Usage

### For Users
Ask these questions when AI claims work is complete:
1. "Are you sure? Be honest."
2. "Show me the actual files."
3. "Have you run the tests?"
4. "What gaps remain?"

### For AI Assistants
Before responding, ask yourself:
1. Have I verified this claim?
2. Am I confusing summary with reality?
3. Did I actually do this, or does it just exist?
4. What evidence can I provide?

---

## Success Metrics

A high-quality response should:
- ✅ Provide concrete evidence (file contents, test output)
- ✅ Distinguish between current and previous work
- ✅ Identify all gaps, including subtle ones
- ✅ Admit uncertainty when appropriate
- ✅ Verify claims before making them
- ✅ Show, don't just tell

---

## The Ultimate Test

**"If I asked another developer to verify your claims, could they do so using only the evidence you provided?"**

If the answer is no, you haven't provided enough evidence.

---

## Conclusion

The adversarial prompts in this session transformed:
- Unverified claims → Concrete evidence
- Assumptions → Verification
- Incomplete analysis → Comprehensive gaps
- Abstract descriptions → Actual implementations

**Key Insight**: The best way to raise the bar is to demand evidence, not accept claims.

---

## Appendix: Session Timeline

1. **Initial claim**: "Phase 5 is complete"
2. **Challenge**: "Are you sure?"
3. **Verification**: Actually checked files
4. **Discovery**: Feedback is a stub
5. **Plan**: Created detailed implementation plan
6. **Implementation**: Actually wrote the code
7. **Verification**: Ran tests, checked compilation
8. **Documentation**: Created comprehensive summaries

**Result**: Went from B- (unverified) to A (verified and complete)

---

**Use this prompt to hold AI assistants (including yourself) to the highest standard.**
