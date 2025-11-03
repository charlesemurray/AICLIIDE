# Adversarial Verification Protocol

## Purpose
This document defines a ruthless verification standard to prevent claiming "production ready" without empirical proof.

## The Adversarial Prompt

**Assumption: Your implementation has critical flaws until proven otherwise.**

### 1. Runtime Evidence (Non-Negotiable)

Show actual execution, not code:

```bash
# Must show GREEN output
cargo test --lib workflow -- --nocapture
cargo test --lib skill -- --nocapture
cargo test test_end_to_end_skill_invocation_via_llm -- --nocapture
cargo test test_end_to_end_workflow_invocation_via_llm -- --nocapture
```

**Required:** All tests pass with visible output proving execution path.

### 2. LLM Integration Proof

Prove the LLM can discover and invoke skills:

- [ ] Show exact JSON in tool schema sent to LLM
- [ ] Show real LLM tool_use request attempting to invoke skill
- [ ] Show tool_use struct being parsed correctly
- [ ] Show routing logic selecting skill/workflow path (not MCP fallback)
- [ ] Show skill/workflow executing and returning result

**Test:** Add `println!` statements in routing logic and prove they execute.

### 3. Break It Intentionally

Demonstrate robustness by breaking it:

- [ ] Malformed skill JSON - does it fail gracefully?
- [ ] Missing required parameter - does validation catch it?
- [ ] Skill file locked/permission denied - does error handling work?
- [ ] Concurrent skill loading - race conditions?
- [ ] 1000 skills loaded - performance degradation?

**Test:** Each failure mode must have a test that proves error handling works.

### 4. Design Scrutiny

Question every design decision:

- [ ] Why is `from_definition(&self, definition)` an instance method when it doesn't use `self`?
- [ ] Explain lifetime semantics of SkillDefinition → ToolSpec conversion
- [ ] Prove no use-after-free or dangling references
- [ ] Why duplicate code between skills/workflows instead of generic implementation?
- [ ] What's the memory overhead of loading N skills?

**Test:** Must articulate ownership model and prove memory safety.

### 5. Filesystem Integration

Test the actual user journey:

- [ ] Skill JSON file doesn't exist
- [ ] Skill JSON is malformed
- [ ] Skill directory has wrong permissions
- [ ] Skill file is being written while loading
- [ ] Symlink to skill file
- [ ] Skill file is empty

**Test:** Integration tests that touch real filesystem, not mocks.

### 6. Concurrency & Scale

Prove it works under load:

- [ ] Two sessions load skills simultaneously
- [ ] Skill registry accessed from multiple threads
- [ ] 100 skills loaded - measure time
- [ ] 1000 skills loaded - measure memory
- [ ] Skill invoked while registry is being updated

**Test:** Concurrent test with thread sanitizer enabled.

### 7. Coverage Measurement

Quantify what's actually tested:

```bash
cargo tarpaulin --lib --out Html
```

- [ ] Line coverage > 80% for new code
- [ ] Branch coverage > 70% for new code
- [ ] All error paths have tests

**Test:** Generate coverage report and identify untested paths.

## Verification Checklist

Before claiming "production ready":

- [ ] All runtime tests pass with visible output
- [ ] LLM integration proven with real tool_use flow
- [ ] All 6 failure modes tested and handled
- [ ] Design decisions justified with ownership model explained
- [ ] Filesystem integration tests pass
- [ ] Concurrency tests pass with no race conditions
- [ ] Coverage > 80% for new code
- [ ] Performance acceptable at scale (1000 skills < 100ms load time)

## The Standard

**"If you can't show me the test passing, you can't claim the feature works."**

Compilation is table stakes. Runtime proof is the standard.

## Usage

When reviewing implementation:

1. Apply this checklist ruthlessly
2. Demand evidence for every claim
3. Assume adversarial mindset: "How can I break this?"
4. Accept only empirical proof, not explanations
5. If any item fails, implementation is NOT production ready

## Red Flags

Signs of insufficient verification:

- "Tests compile but can't run them because..."
- "It should work because the code looks right"
- "I tested it manually and it worked"
- "The design is correct so tests will pass"
- Claims without evidence
- Explanations instead of test output

## Success Criteria

Implementation is production ready when:

1. Every checklist item has ✅ with evidence
2. All tests run and pass (GREEN output shown)
3. All failure modes tested and handled
4. Coverage report shows >80% for new code
5. Performance benchmarks meet requirements
6. No "should work" or "looks correct" - only "proven to work"
