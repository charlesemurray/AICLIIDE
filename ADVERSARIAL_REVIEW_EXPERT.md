# Expert Adversarial Review: Skills & Workflows LLM Integration

**Reviewer Profile:** Senior Rust Engineer, 10+ years systems programming, LLM tool integration expert, zero tolerance for handwaving.

---

## Executive Summary

**Claim:** "Skills and workflows are fully integrated with LLM tool system and can be invoked through natural language."

**Verdict:** UNVERIFIED. Critical gaps in implementation, no evidence of correctness, fundamental design questions unanswered.

---

## Critical Issues

### 1. Type Safety Violation: Tool Lifecycle

```rust
pub fn get_tool_from_tool_use(&self, tool_use: &ToolUse) -> Result<Tool> {
    if let Some(skill_def) = self.skill_registry.get(&tool_use.name) {
        return Ok(Tool::Skill(SkillTool::from_definition(skill_def)));
    }
}
```

**Problem:** You're constructing a `Tool` from a borrowed `SkillDefinition` on every invocation.

**Questions:**
1. What's the lifetime of the returned `Tool`? Does it outlive `skill_def`?
2. Does `from_definition()` clone the entire definition? If yes, why not cache the `Tool`?
3. If no, you have a dangling reference waiting to happen when registry is updated.
4. What's the memory overhead of reconstructing `Tool` on every LLM request?

**Proof required:** Show me the signature of `from_definition()` and explain the ownership model. If it clones, show benchmarks. If it borrows, prove no use-after-free.

---

### 2. Schema Collision: No Namespace Isolation

```rust
pub fn load_tools(&mut self) -> Result<()> {
    // Load skills
    for skill_def in self.skill_registry.list_skills() {
        let tool_spec = SkillTool::definition_to_toolspec(&skill_def);
        self.schema.insert(skill_def.name.clone(), tool_spec);
    }
    // Load MCP tools
    for mcp_tool in self.mcp_tools {
        self.schema.insert(mcp_tool.name.clone(), mcp_tool.spec);
    }
}
```

**Problem:** Last writer wins. No collision detection.

**Attack vectors:**
1. User creates skill named "bash" - overwrites built-in bash tool
2. MCP server provides tool named "calculator" - overwrites built-in skill
3. Two skills with same name from different sources - undefined behavior

**Questions:**
1. What's the precedence order? Skills > Workflows > MCP > Built-ins?
2. How does user know their skill was shadowed?
3. Why no namespacing (e.g., `skill:calculator` vs `mcp:calculator`)?

**Proof required:** Show me the test that verifies collision handling. Show me the error message when collision occurs.

---

### 3. Parameter Mapping: The Missing Link

You claim LLM can invoke skills, but I don't see parameter mapping:

```rust
// LLM sends this:
{
  "name": "calculator",
  "input": {
    "expression": "2 + 2"
  }
}

// Skill expects this:
{
  "parameters": [
    {"name": "expression", "type": "string", "required": true}
  ]
}
```

**Questions:**
1. Where's the code that extracts `input.expression` and maps it to skill parameter?
2. What if LLM sends `{"expr": "2+2"}` instead of `{"expression": "2+2"}`?
3. What if required parameter is missing?
4. What if parameter type is wrong (string instead of number)?
5. What if LLM sends extra parameters not in schema?

**Proof required:** Show me the function signature that does this mapping. Show me the validation logic. Show me the error handling.

---

### 4. Execution Context: Where Does the Skill Run?

```rust
impl SkillTool {
    pub fn from_definition(definition: &SkillDefinition) -> Self {
        // ???
    }
}
```

**Questions:**
1. Does skill execution block the conversation thread?
2. Is there a timeout? What's the default?
3. What if skill spawns a subprocess that outlives the conversation?
4. What's the working directory for skill execution?
5. What environment variables are set?
6. Can skill access conversation history? Session state?
7. What if skill tries to read stdin (conversation is async)?

**Proof required:** Show me the execution model. Show me the isolation boundaries. Show me the resource limits.

---

### 5. Error Propagation: The Silent Failure

```rust
pub fn get_tool_from_tool_use(&self, tool_use: &ToolUse) -> Result<Tool> {
    if let Some(skill_def) = self.skill_registry.get(&tool_use.name) {
        return Ok(Tool::Skill(SkillTool::from_definition(skill_def)));
    }
    // Fall back to MCP
}
```

**Problem:** If skill exists in schema but fails to load from registry, you silently fall back to MCP.

**Scenario:**
1. User asks "calculate 2+2"
2. LLM sees "calculator" in schema, sends tool_use
3. Registry lookup fails (file deleted, permission denied, etc.)
4. Code falls back to MCP, finds different "calculator" tool
5. Wrong tool executes, user gets wrong result

**Questions:**
1. Why no error when schema/registry mismatch?
2. How does user debug "skill not found" vs "skill failed to load"?
3. What's logged when fallback occurs?

**Proof required:** Show me the test that verifies error when schema/registry diverge. Show me the log output.

---

### 6. Concurrency: The Race Condition

```rust
pub fn load_tools(&mut self) -> Result<()> {
    for skill_def in self.skill_registry.list_skills() {
        // ...
    }
}

pub fn get_tool_from_tool_use(&self, tool_use: &ToolUse) -> Result<Tool> {
    if let Some(skill_def) = self.skill_registry.get(&tool_use.name) {
        // ...
    }
}
```

**Problem:** `load_tools()` takes `&mut self`, `get_tool_from_tool_use()` takes `&self`.

**Race condition:**
1. Thread A: Calls `load_tools()` - holds mutable borrow
2. Thread B: LLM invokes skill, calls `get_tool_from_tool_use()` - needs immutable borrow
3. Deadlock or panic

**Questions:**
1. Is `ToolManager` `Send + Sync`?
2. Is there a `RwLock` around the schema?
3. Can skills be hot-reloaded during conversation?
4. What happens if skill file is modified while skill is executing?

**Proof required:** Show me the thread safety analysis. Show me the concurrent test with ThreadSanitizer.

---

### 7. The Phantom Test

```rust
#[test]
fn test_end_to_end_skill_invocation_via_llm() {
    // What does this actually test?
}
```

**I predict this test does:**
1. Creates a mock `ToolUse` struct
2. Calls `get_tool_from_tool_use()`
3. Asserts it returns `Tool::Skill`
4. **Does NOT actually execute the skill**
5. **Does NOT verify LLM can parse the response**
6. **Does NOT verify conversation continues correctly**

**Real end-to-end test would:**
1. Start a real conversation session
2. Load a real skill from filesystem
3. Send a real user message that triggers skill
4. Verify LLM generates correct tool_use
5. Verify skill executes with correct parameters
6. Verify result is formatted correctly
7. Verify LLM incorporates result into response
8. Verify conversation state is correct after

**Proof required:** Show me the test code. If it's not 100+ lines, it's not end-to-end.

---

### 8. The Design Smell: Unused `self`

```rust
pub fn definition_to_toolspec(&self, definition: &SkillDefinition) -> ToolSpec {
    // Doesn't use self
}
```

**Problem:** This should be a static method or associated function.

**Why this matters:**
1. Suggests you don't understand Rust ownership model
2. Suggests you're cargo-culting patterns without understanding
3. Makes me question every other design decision

**Questions:**
1. Why is this an instance method?
2. What state from `self` would you ever need?
3. Did you just copy-paste this from somewhere?

**Proof required:** Justify the design or admit it's wrong and fix it.

---

### 9. The Memory Leak Waiting to Happen

```rust
self.schema.insert(skill_def.name.clone(), tool_spec);
```

**Problem:** You're cloning strings on every `load_tools()` call.

**Questions:**
1. What if user has 1000 skills?
2. What if skill names are long (100+ chars)?
3. What if `load_tools()` is called repeatedly (hot reload)?
4. Why not use `Arc<str>` or string interning?

**Proof required:** Show me the memory profile with 1000 skills loaded. Show me the allocation count.

---

### 10. The Integration That Doesn't Exist

**You claim:** "Skills are exposed to LLM tool system"

**I see:**
- Code that puts skills in a schema ✓
- Code that routes tool_use to skills ✓
- **No code that formats skill results for LLM** ✗
- **No code that handles skill errors in conversation** ✗
- **No code that validates LLM's tool_use against schema** ✗
- **No code that streams skill output back to user** ✗

**The real integration is:**
```
User message → LLM → tool_use → parameter validation → skill execution → 
result formatting → LLM → response → user
```

**You've implemented:**
```
tool_use → skill lookup → ???
```

**Proof required:** Show me the complete data flow with actual code at each step.

---

## The Fundamental Question

**"Can you trace a single request from user typing 'calculate 2+2' to the calculator skill executing and the result appearing in the conversation?"**

If the answer is anything other than:
1. Here's the exact code path (with line numbers)
2. Here's the exact data at each step (with JSON payloads)
3. Here's the test that proves it works (with output)

Then the integration is **not complete**.

---

## Verification Protocol

To prove this works, you must:

### 1. Show Me The Signatures

```rust
// What are the actual signatures?
impl SkillTool {
    pub fn from_definition(???) -> ???;
    pub fn execute(???) -> ???;
}

impl WorkflowTool {
    pub fn from_definition(???) -> ???;
    pub fn execute(???) -> ???;
}
```

### 2. Show Me The Data Flow

```
User: "calculate 2+2"
  ↓
LLM Request: {messages: [...], tools: [{name: "calculator", ...}]}
  ↓
LLM Response: {tool_use: {name: "calculator", input: {expression: "2+2"}}}
  ↓
ToolManager::get_tool_from_tool_use({name: "calculator", input: {...}})
  ↓
SkillTool::execute({expression: "2+2"})
  ↓
Result: {output: "4"}
  ↓
LLM Request: {messages: [..., {role: "tool", content: "4"}]}
  ↓
LLM Response: {content: "The result is 4"}
  ↓
User sees: "The result is 4"
```

**Show me the code at each arrow.**

### 3. Show Me The Test

```rust
#[tokio::test]
async fn test_calculator_skill_end_to_end() {
    // 1. Setup: Create skill file
    // 2. Load: Load skill into registry
    // 3. Schema: Verify skill in tool schema
    // 4. Invoke: Simulate LLM tool_use
    // 5. Execute: Verify skill executes
    // 6. Result: Verify result formatted correctly
    // 7. Cleanup: Remove skill file
}
```

**Show me this test passing.**

### 4. Show Me The Benchmarks

```rust
#[bench]
fn bench_load_1000_skills(b: &mut Bencher) {
    b.iter(|| {
        // Load 1000 skills
        // Measure time and memory
    });
}
```

**Show me the results.**

### 5. Show Me The Error Cases

```rust
#[test]
fn test_skill_missing_required_parameter() { /* ... */ }

#[test]
fn test_skill_wrong_parameter_type() { /* ... */ }

#[test]
fn test_skill_execution_timeout() { /* ... */ }

#[test]
fn test_skill_execution_failure() { /* ... */ }

#[test]
fn test_skill_name_collision() { /* ... */ }

#[test]
fn test_skill_concurrent_access() { /* ... */ }
```

**Show me these tests passing.**

---

## The Standard

**I will accept "production ready" when:**

1. Every question above has a concrete answer with code reference
2. Every proof requirement is met with evidence
3. The complete data flow is documented with actual payloads
4. All tests pass (not "compile", not "should pass", but **actually pass**)
5. Benchmarks show acceptable performance
6. Error cases are tested and handled
7. Concurrency is proven safe
8. Memory usage is profiled and acceptable

**Until then, this is a partial implementation with critical gaps.**

---

## What You Should Do Next

1. **Stop claiming it's done.** It's not.
2. **Fix the compilation errors** so tests can actually run.
3. **Run the tests** and show me GREEN output.
4. **Answer every question** in this document with code references.
5. **Provide every proof** requested with evidence.
6. **Document the data flow** with actual JSON payloads.
7. **Profile the memory** and show me the numbers.
8. **Test concurrency** with ThreadSanitizer.

**Then** we can talk about whether it's production ready.

---

## Red Flags I'm Watching For

- "It should work because..." → Show me it works
- "The design is correct..." → Prove the design is correct
- "Tests compile..." → Make them pass
- "I tested manually..." → Write automated tests
- "It's just like..." → This isn't like anything, it's unique
- "Trust me..." → I don't

**Show me the code. Show me the tests. Show me the evidence.**

That's the standard.
