# Gap Closure Plan

**Goal:** Address all CRITICAL and HIGH severity findings from adversarial review

---

## Priority 1: CRITICAL (Blocks Production)

### Gap #1: No Parameter Validation
**Finding:** Skills accept any args without validation  
**Impact:** Runtime failures with poor error messages  
**Fix:** Add validation before constructing SkillTool

**Implementation:**
```rust
// In tool_manager.rs:928, before constructing SkillTool
if let Some(definition) = self.skill_registry.get(name) {
    // NEW: Validate args against definition.parameters
    validate_skill_args(&value.args, &definition.parameters)
        .map_err(|e| ToolResult {
            tool_use_id: value.id.clone(),
            content: vec![ToolResultContentBlock::Text(format!(
                "Invalid parameters for skill '{}': {}", name, e
            ))],
            status: ToolResultStatus::Error,
        })?;
    
    let skill_tool = SkillTool::from_definition(definition);
    return Ok(Tool::SkillNew(skill_tool));
}

// NEW: Add validation function
fn validate_skill_args(args: &Value, parameters: &[Parameter]) -> Result<(), String> {
    // Check required parameters present
    // Check parameter types match
    // Check no extra parameters
}
```

**Test:**
```rust
#[test]
fn test_skill_parameter_validation_missing_required() {
    // Skill requires "expression"
    // LLM sends empty args
    // Should return error
}

#[test]
fn test_skill_parameter_validation_wrong_type() {
    // Skill requires string
    // LLM sends number
    // Should return error
}
```

**Effort:** 2-3 hours

---

## Priority 2: HIGH (Causes Incorrect Behavior)

### Gap #2: No Collision Detection
**Finding:** Skills can shadow built-ins, schema/routing inconsistent  
**Impact:** Wrong tool executes, user confusion  
**Fix:** Add collision detection and namespace or error

**Option A: Namespace (Recommended)**
```rust
// In load_tools(), prefix skill names
for skill_def in self.skill_registry.list_skills() {
    let tool_spec = SkillTool::definition_to_toolspec(&skill_def);
    self.schema.insert(
        format!("skill:{}", skill_def.name),  // ← Namespace
        tool_spec
    );
}

// In get_tool_from_tool_use(), check namespaced name
name if name.starts_with("skill:") => {
    let skill_name = name.strip_prefix("skill:").unwrap();
    if let Some(definition) = self.skill_registry.get(skill_name) {
        // ...
    }
}
```

**Option B: Error on Collision**
```rust
// In load_tools(), check before insert
for skill_def in self.skill_registry.list_skills() {
    if self.schema.contains_key(&skill_def.name) {
        return Err(format!(
            "Skill '{}' collides with existing tool", 
            skill_def.name
        ));
    }
    // ...
}
```

**Test:**
```rust
#[test]
fn test_skill_collision_with_builtin() {
    // Create skill named "fs_read"
    // Should error or use namespace
}
```

**Effort:** 3-4 hours

---

## Priority 3: MEDIUM (Quality/Reliability)

### Gap #3: Schema/Registry Mismatch
**Finding:** Silent fallback when skill in schema but not registry  
**Impact:** Confusing errors, wrong tool execution  
**Fix:** Track tool source in schema, error on mismatch

**Implementation:**
```rust
// Add source tracking to schema
struct ToolEntry {
    spec: ToolSpec,
    source: ToolSource,  // NEW
}

enum ToolSource {
    Builtin,
    Skill,
    Workflow,
    Mcp(String),
}

// In get_tool_from_tool_use()
if let Some(entry) = self.schema.get(name) {
    match entry.source {
        ToolSource::Skill => {
            // Must be in registry
            if let Some(def) = self.skill_registry.get(name) {
                return Ok(Tool::SkillNew(...));
            } else {
                return Err(ToolResult {
                    content: vec![ToolResultContentBlock::Text(
                        format!("Skill '{}' in schema but not in registry", name)
                    )],
                    status: ToolResultStatus::Error,
                });
            }
        }
        // ...
    }
}
```

**Test:**
```rust
#[test]
fn test_schema_registry_mismatch() {
    // Add to schema manually
    // Don't add to registry
    // Should error with specific message
}
```

**Effort:** 2-3 hours

---

### Gap #4: Incomplete End-to-End Test
**Finding:** Test stops at routing, doesn't execute skill  
**Impact:** No proof skill execution works  
**Fix:** Extend test to execute and verify result

**Implementation:**
```rust
#[tokio::test]
async fn test_end_to_end_skill_invocation_via_llm() {
    // ... existing setup ...
    
    // Get tool from tool use
    let tool = manager.get_tool_from_tool_use(tool_use).await.unwrap();
    
    // NEW: Actually execute the skill
    let result = match tool {
        Tool::SkillNew(skill) => {
            skill.execute(&manager, &args).await
        }
        _ => panic!("Expected SkillNew"),
    };
    
    // NEW: Verify result
    assert!(result.is_ok());
    let tool_result = result.unwrap();
    assert_eq!(tool_result.status, ToolResultStatus::Success);
    assert!(tool_result.content[0].contains("Hello World"));
}
```

**Effort:** 1-2 hours

---

### Gap #5: Inefficient Allocation
**Finding:** Clones strings on every invocation  
**Impact:** Performance degradation at scale  
**Fix:** Cache SkillTool instances

**Implementation:**
```rust
// In ToolManager
struct ToolManager {
    skill_cache: HashMap<String, Arc<SkillTool>>,  // NEW
    // ...
}

// In get_tool_from_tool_use()
if let Some(definition) = self.skill_registry.get(name) {
    // Check cache first
    let skill_tool = self.skill_cache
        .entry(name.to_string())
        .or_insert_with(|| {
            Arc::new(SkillTool::from_definition(definition))
        });
    
    return Ok(Tool::SkillNew(Arc::clone(skill_tool)));
}
```

**Test:**
```rust
#[test]
fn test_skill_caching() {
    // Invoke same skill twice
    // Verify only one allocation
}
```

**Effort:** 1-2 hours

---

## Priority 4: LOW (Code Quality)

### Gap #6: Unused `self` Parameter
**Finding:** definition_to_toolspec doesn't use self  
**Impact:** API confusion  
**Fix:** Make it a static method

**Implementation:**
```rust
// Change from:
pub fn definition_to_toolspec(&self, definition: &SkillDefinition) -> ToolSpec

// To:
pub fn definition_to_toolspec(definition: &SkillDefinition) -> ToolSpec
```

**Effort:** 15 minutes

---

## Execution Plan

### Phase 1: Critical Fixes (Day 1)
1. ✅ Fix compilation errors (DONE)
2. ⏳ Add parameter validation (#1) - 2-3 hours
3. ⏳ Add collision detection (#2) - 3-4 hours

**Checkpoint:** Run tests, verify critical gaps closed

### Phase 2: Quality Fixes (Day 2)
4. ⏳ Fix schema/registry mismatch (#3) - 2-3 hours
5. ⏳ Complete end-to-end test (#4) - 1-2 hours
6. ⏳ Add caching (#5) - 1-2 hours

**Checkpoint:** Run full test suite, verify all tests pass

### Phase 3: Polish (Day 2)
7. ⏳ Fix unused self (#6) - 15 minutes
8. ⏳ Run adversarial review again
9. ⏳ Document remaining gaps (if any)

**Checkpoint:** Re-run adversarial checklist, verify all items pass

---

## Success Criteria

### Before Claiming "Production Ready"
- [ ] All CRITICAL gaps closed
- [ ] All HIGH gaps closed
- [ ] All tests run and pass (GREEN output)
- [ ] End-to-end test executes skill
- [ ] Parameter validation works
- [ ] Collision detection works
- [ ] Adversarial review passes

### Evidence Required
- [ ] `cargo test --lib skill` output (GREEN)
- [ ] `cargo test --lib workflow` output (GREEN)
- [ ] Test output showing skill execution
- [ ] Test output showing parameter validation
- [ ] Test output showing collision detection

---

## Timeline

**Total effort:** 10-15 hours  
**Timeline:** 2 days  
**Blockers:** None (compilation fixed)

**Day 1:**
- Morning: Parameter validation (3 hours)
- Afternoon: Collision detection (4 hours)
- Evening: Test and verify (1 hour)

**Day 2:**
- Morning: Schema/registry fix (3 hours)
- Afternoon: Complete e2e test + caching (3 hours)
- Evening: Polish + re-review (2 hours)

---

## Next Steps

**Immediate:**
1. Start with Gap #1 (parameter validation) - highest impact
2. Write test first (TDD)
3. Implement validation
4. Verify test passes
5. Move to Gap #2

**After each gap:**
1. Write test
2. Implement fix
3. Run test (show GREEN)
4. Commit with evidence
5. Update this document

**Final step:**
- Re-run adversarial review
- Show all gaps closed
- Provide evidence for all claims
