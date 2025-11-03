# Iteration Process Requirements

## Overview

Each iteration in the gap closure plan must follow these process requirements to ensure quality, maintainability, and successful completion.

---

## Universal Requirements (Every Iteration)

### 1. Before Starting

**Planning** (15-30 min):
- [ ] Read and understand the iteration goal
- [ ] Review files to be modified/created
- [ ] Identify dependencies on previous iterations
- [ ] Estimate time accurately
- [ ] Set up development environment

**Checklist**:
```bash
# Ensure clean working state
git status
git pull origin main

# Create feature branch
git checkout -b feature/iteration-X-Y-Z

# Verify build works
cargo build --bin chat_cli
cargo test
```

---

### 2. During Implementation

**Development Process**:
- [ ] Write code incrementally (small commits)
- [ ] Follow existing code style and patterns
- [ ] Add inline comments for complex logic
- [ ] Keep functions small and focused
- [ ] No placeholders or TODOs

**Continuous Validation**:
```bash
# After each significant change
cargo build --bin chat_cli    # Must compile
cargo test                     # Existing tests pass
cargo clippy                   # No new warnings
cargo +nightly fmt             # Format code
```

**Time Tracking**:
- [ ] Log actual time spent
- [ ] Note any blockers immediately
- [ ] Update estimate if needed

---

### 3. Testing Requirements

**Unit Tests** (Required):
- [ ] Test happy path
- [ ] Test error cases
- [ ] Test edge cases
- [ ] Test with invalid inputs
- [ ] Minimum 80% code coverage for new code

**Integration Tests** (Required):
- [ ] Test feature end-to-end
- [ ] Test interaction with existing features
- [ ] Test in realistic scenarios

**Manual Testing** (Required):
- [ ] Run the actual CLI command
- [ ] Verify output is correct
- [ ] Test error scenarios manually
- [ ] Verify user experience

**Test Checklist**:
```bash
# Run all tests
cargo test

# Run specific test
cargo test <test_name>

# Run with output
cargo test -- --nocapture

# Manual testing
cargo run --bin chat_cli -- <command>
```

---

### 4. Documentation Requirements

**Code Documentation**:
- [ ] Add doc comments to public functions
- [ ] Document parameters and return values
- [ ] Add usage examples in doc comments
- [ ] Document error conditions

**User Documentation** (if user-facing):
- [ ] Update relevant .md files
- [ ] Add examples
- [ ] Update troubleshooting guide if needed
- [ ] Update quick start if needed

**Example**:
```rust
/// Loads skills from the specified directory
///
/// # Arguments
/// * `path` - Directory containing skill JSON files
///
/// # Returns
/// * `Ok(LoadingSummary)` - Summary of loaded skills
/// * `Err(...)` - If directory doesn't exist or is unreadable
///
/// # Example
/// ```
/// let summary = registry.load_from_directory(Path::new("~/.q-skills")).await?;
/// ```
pub async fn load_from_directory(&mut self, path: &Path) -> Result<LoadingSummary>
```

---

### 5. Code Review Requirements

**Self-Review Checklist**:
- [ ] Code follows Rust idioms
- [ ] No unwrap() in production code (use ? or proper error handling)
- [ ] Error messages are user-friendly
- [ ] No hardcoded paths or values
- [ ] No debug print statements left in
- [ ] All tests pass
- [ ] No clippy warnings
- [ ] Code is formatted

**Review Process**:
```bash
# Self-review diff
git diff main

# Check for common issues
grep -r "unwrap()" src/
grep -r "println!" src/
grep -r "TODO" src/
grep -r "FIXME" src/
```

---

### 6. Git Commit Requirements

**Commit Message Format**:
```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types**:
- `feat`: New feature
- `fix`: Bug fix
- `test`: Adding tests
- `docs`: Documentation
- `refactor`: Code refactoring
- `style`: Formatting
- `perf`: Performance improvement

**Example**:
```
feat(skills): add skill loading feedback

- Show progress while loading skills
- Display success/failure for each skill
- Print summary at end
- Add user-friendly error messages

Closes #123
```

**Commit Checklist**:
- [ ] Descriptive subject line (< 72 chars)
- [ ] Body explains what and why
- [ ] References issue/ticket if applicable
- [ ] One logical change per commit
- [ ] All tests pass before committing

---

### 7. Completion Criteria

**Definition of Done**:
- [ ] Code implemented and working
- [ ] All tests written and passing
- [ ] Code reviewed (self or peer)
- [ ] Documentation updated
- [ ] Manual testing completed
- [ ] No regressions introduced
- [ ] Committed with proper message
- [ ] Ready for next iteration

**Validation**:
```bash
# Final checks before marking complete
cargo build --bin chat_cli
cargo test
cargo clippy
cargo +nightly fmt --check

# Verify feature works
cargo run --bin chat_cli -- <test command>
```

---

## Phase-Specific Requirements

### Phase 1: Critical Gaps

**Additional Requirements**:
- [ ] User-facing changes must be intuitive
- [ ] Error messages must be actionable
- [ ] Performance impact < 5%
- [ ] Backward compatible

**Validation**:
- [ ] Feature solves the identified gap
- [ ] User can complete the workflow
- [ ] No new blockers introduced

---

### Phase 2: User Testing

**Additional Requirements**:
- [ ] Test protocol documented
- [ ] Test users recruited
- [ ] Feedback collected systematically
- [ ] Issues prioritized
- [ ] Iteration plan updated based on feedback

**User Testing Checklist**:
- [ ] 5 users minimum
- [ ] Diverse skill levels
- [ ] Observe, don't guide
- [ ] Record pain points
- [ ] Collect quantitative data

---

### Phase 3: Polish

**Additional Requirements**:
- [ ] Visual consistency
- [ ] Performance optimized
- [ ] Edge cases handled
- [ ] Documentation comprehensive

---

## Iteration-Specific Requirements

### 1.1.1: Create Agent Mock

**Prerequisites**:
- [ ] Understand ToolSpec structure
- [ ] Review existing agent code
- [ ] Understand test requirements

**Implementation Requirements**:
- [ ] Mock must be simple and focused
- [ ] No complex AI logic needed
- [ ] Pattern matching for test cases
- [ ] Easy to extend for new tests

**Testing Requirements**:
- [ ] Test mock can discover tools
- [ ] Test mock can select tools
- [ ] Test mock handles invalid input
- [ ] Test mock is deterministic

**Completion Criteria**:
- [ ] Mock agent works in tests
- [ ] At least 3 test cases pass
- [ ] Code is reusable for other tests

---

### 1.1.2: Natural Language to Skill Test

**Prerequisites**:
- [ ] Step 1.1.1 complete (mock agent exists)
- [ ] Understand skill invocation flow

**Implementation Requirements**:
- [ ] Test realistic user inputs
- [ ] Cover multiple skill types
- [ ] Test parameter extraction
- [ ] Test error scenarios

**Testing Requirements**:
- [ ] At least 5 test scenarios
- [ ] Happy path works
- [ ] Error cases handled
- [ ] Edge cases covered

**Completion Criteria**:
- [ ] All tests pass
- [ ] Proves natural language invocation works
- [ ] No flaky tests

---

### 1.1.3: ChatSession Integration Test

**Prerequisites**:
- [ ] Steps 1.1.1 and 1.1.2 complete
- [ ] Understand ChatSession lifecycle

**Implementation Requirements**:
- [ ] Use real ChatSession
- [ ] Test in realistic context
- [ ] Verify end-to-end flow
- [ ] Test error handling

**Testing Requirements**:
- [ ] Test skill invocation in chat
- [ ] Test result returned to user
- [ ] Test error handling
- [ ] Test multiple invocations

**Completion Criteria**:
- [ ] Integration test passes
- [ ] Proves feature works in production context
- [ ] No regressions in existing chat tests

---

### 1.2.1: Skill Loading Feedback

**Prerequisites**:
- [ ] Understand SkillRegistry code
- [ ] Review user feedback requirements

**Implementation Requirements**:
- [ ] Print progress during loading
- [ ] Show success/failure per skill
- [ ] Print summary at end
- [ ] User-friendly messages

**Testing Requirements**:
- [ ] Test with valid skills
- [ ] Test with invalid skills
- [ ] Test with empty directory
- [ ] Verify output format

**Completion Criteria**:
- [ ] User sees clear feedback
- [ ] Manual testing confirms good UX
- [ ] Tests verify output

---

### 1.2.2: Skill Execution Feedback

**Prerequisites**:
- [ ] Understand SkillTool code
- [ ] Review execution flow

**Implementation Requirements**:
- [ ] Show skill name being executed
- [ ] Show execution time
- [ ] Show success/failure
- [ ] Show result preview

**Testing Requirements**:
- [ ] Test with fast skills
- [ ] Test with slow skills
- [ ] Test with failing skills
- [ ] Verify timing accuracy

**Completion Criteria**:
- [ ] User knows what's happening
- [ ] Feedback is helpful
- [ ] Performance impact minimal

---

### 1.3.1: Error Message Redesign

**Prerequisites**:
- [ ] Review all error types
- [ ] Understand user pain points

**Implementation Requirements**:
- [ ] Plain English messages
- [ ] Actionable tips
- [ ] Recovery suggestions
- [ ] Relevant commands

**Testing Requirements**:
- [ ] Test each error type
- [ ] Verify message clarity
- [ ] Test suggestions work
- [ ] User testing feedback

**Completion Criteria**:
- [ ] All errors have good messages
- [ ] Users can recover from errors
- [ ] No technical jargon

---

### 1.3.2: Error Recovery Paths

**Prerequisites**:
- [ ] Step 1.3.1 complete
- [ ] Understand common failures

**Implementation Requirements**:
- [ ] Specific suggestions per error
- [ ] Commands user can run
- [ ] Links to documentation
- [ ] Examples

**Testing Requirements**:
- [ ] Test each recovery path
- [ ] Verify suggestions work
- [ ] Test with real users

**Completion Criteria**:
- [ ] Users can recover from errors
- [ ] Suggestions are helpful
- [ ] Documentation links work

---

### 1.4.1: Enhanced Skills List Command

**Prerequisites**:
- [ ] Review current list command
- [ ] Understand user needs

**Implementation Requirements**:
- [ ] Clear formatting
- [ ] Show descriptions
- [ ] Show parameter counts
- [ ] Usage hints

**Testing Requirements**:
- [ ] Test with 0 skills
- [ ] Test with 1 skill
- [ ] Test with many skills
- [ ] Verify formatting

**Completion Criteria**:
- [ ] Output is clear and helpful
- [ ] Users can find skills
- [ ] Manual testing confirms good UX

---

### 1.4.2: Skill Info Command

**Prerequisites**:
- [ ] Step 1.4.1 complete
- [ ] Understand skill metadata

**Implementation Requirements**:
- [ ] Show all skill details
- [ ] Show parameters
- [ ] Show usage examples
- [ ] Clear formatting

**Testing Requirements**:
- [ ] Test with various skills
- [ ] Test with missing data
- [ ] Verify formatting
- [ ] Test error cases

**Completion Criteria**:
- [ ] Users get detailed info
- [ ] Output is helpful
- [ ] Examples are clear

---

## Quality Gates

### Gate 1: Code Quality
**Must Pass**:
```bash
cargo build --bin chat_cli     # ✅ Compiles
cargo test                      # ✅ All tests pass
cargo clippy                    # ✅ No warnings
cargo +nightly fmt --check      # ✅ Formatted
```

### Gate 2: Functionality
**Must Pass**:
- [ ] Feature works as designed
- [ ] No regressions
- [ ] Error handling works
- [ ] Edge cases handled

### Gate 3: User Experience
**Must Pass**:
- [ ] Manual testing successful
- [ ] Output is clear
- [ ] Errors are helpful
- [ ] Performance acceptable

### Gate 4: Documentation
**Must Pass**:
- [ ] Code documented
- [ ] User docs updated
- [ ] Examples provided
- [ ] Troubleshooting updated

---

## Time Management

### Time Tracking Template
```
Iteration: X.Y.Z
Estimated: X hours
Actual: Y hours
Variance: +/- Z hours

Breakdown:
- Planning: X min
- Implementation: Y hours
- Testing: Z hours
- Documentation: W hours
- Review: V hours

Blockers:
- [List any blockers]

Notes:
- [Any learnings or issues]
```

### When Behind Schedule
1. **Assess**: Why are we behind?
2. **Communicate**: Update stakeholders
3. **Adjust**: Reduce scope or extend timeline
4. **Focus**: Prioritize critical features
5. **Help**: Ask for assistance if needed

### When Ahead of Schedule
1. **Validate**: Double-check quality
2. **Polish**: Improve UX
3. **Document**: Enhance documentation
4. **Test**: Add more test cases
5. **Review**: Help others

---

## Communication Requirements

### Daily Updates
**Format**:
```
Date: YYYY-MM-DD
Iteration: X.Y.Z
Status: On Track / At Risk / Blocked

Completed Today:
- [List accomplishments]

Plan for Tomorrow:
- [List next steps]

Blockers:
- [List any blockers]
```

### Weekly Summary
**Format**:
```
Week: X
Phase: Y
Progress: Z%

Completed Iterations:
- [List completed]

In Progress:
- [List in progress]

Upcoming:
- [List next]

Risks:
- [List risks]

Metrics:
- Tests: X passing
- Coverage: Y%
- Performance: Z ms
```

---

## Risk Management

### Common Risks

**Risk**: Tests are flaky
**Mitigation**: Use deterministic mocks, avoid timing dependencies

**Risk**: Performance regression
**Mitigation**: Benchmark before/after, optimize if needed

**Risk**: Breaking changes
**Mitigation**: Run full test suite, manual testing

**Risk**: Scope creep
**Mitigation**: Stick to iteration goals, defer extras

**Risk**: Technical blockers
**Mitigation**: Identify early, escalate quickly, have backup plan

---

## Success Metrics

### Per Iteration
- [ ] Completed on time (±20%)
- [ ] All tests passing
- [ ] No regressions
- [ ] Documentation updated
- [ ] Quality gates passed

### Per Phase
- [ ] All iterations complete
- [ ] User validation successful
- [ ] Performance targets met
- [ ] No critical bugs

### Overall Project
- [ ] All phases complete
- [ ] User testing successful
- [ ] Production ready
- [ ] Documentation comprehensive

---

## Checklist Summary

### Before Starting Iteration
- [ ] Understand requirements
- [ ] Review dependencies
- [ ] Set up environment
- [ ] Create feature branch

### During Iteration
- [ ] Write code incrementally
- [ ] Test continuously
- [ ] Document as you go
- [ ] Track time

### Before Completing Iteration
- [ ] All tests pass
- [ ] Code reviewed
- [ ] Documentation updated
- [ ] Manual testing done
- [ ] Quality gates passed
- [ ] Committed properly

### After Completing Iteration
- [ ] Update progress tracker
- [ ] Communicate completion
- [ ] Prepare for next iteration
- [ ] Document learnings

---

**Document Version**: 1.0  
**Last Updated**: 2025-11-03  
**Status**: Ready for use
