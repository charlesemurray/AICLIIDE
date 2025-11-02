# JSON Schema Fix Implementation Plan

**Created**: November 2, 2025  
**Priority**: CRITICAL (Security vulnerabilities identified)

## Issues to Fix

### 🚨 **CRITICAL: Custom Commands Security Vulnerability**
- **Issue**: Command injection vulnerability - no parameter validation
- **Risk**: `HIGH` - Can execute arbitrary commands
- **Example**: `/deploy --env="prod; rm -rf /"` would execute `rm -rf /`

### ⚠️ **HIGH: Custom Commands Schema Inconsistency**
- **Issue**: Examples use different schema than implementation
- **Impact**: Developer confusion, runtime failures

### 📝 **LOW: Skills Example Inconsistency**
- **Issue**: One example file uses deprecated schema
- **Impact**: Minimal - just documentation

## Implementation Plan

### **Phase 1: Security Fix (CRITICAL - Do First)**

#### **Task 1.1: Enhance CommandParameter Struct**
**File**: `crates/chat-cli/src/cli/custom_commands/types.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandParameter {
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: String,              // ADD: "string", "boolean", "number", "enum"
    pub required: bool,                  // KEEP: Current functionality
    pub default_value: Option<String>,   // KEEP: Current functionality  
    pub description: Option<String>,     // MAKE OPTIONAL: For help text
    pub values: Option<Vec<String>>,     // ADD: For enum validation
    pub pattern: Option<String>,         // ADD: For security validation
}
```

#### **Task 1.2: Add Parameter Validation Logic**
**File**: `crates/chat-cli/src/cli/custom_commands/validation.rs` (NEW)

```rust
impl CommandParameter {
    pub fn validate_value(&self, value: &str) -> Result<(), CommandError> {
        match self.param_type.as_str() {
            "string" => self.validate_string(value),
            "boolean" => self.validate_boolean(value),
            "number" => self.validate_number(value),
            "enum" => self.validate_enum(value),
            _ => Err(CommandError::InvalidParameter(format!("Unknown type: {}", self.param_type)))
        }
    }
    
    fn validate_string(&self, value: &str) -> Result<(), CommandError> {
        // Security: Check for injection patterns
        if let Some(pattern) = &self.pattern {
            // Validate against regex pattern
        }
        // Check for dangerous patterns: ;, |, &, $, `, etc.
        Ok(())
    }
}
```

#### **Task 1.3: Update Validation in CommandExecutor**
**File**: `crates/chat-cli/src/cli/custom_commands/executor.rs`

```rust
impl CommandExecutor {
    pub fn execute(command: &CustomCommand, execution: &CommandExecution) -> Result<String, CommandError> {
        // Enhanced validation with type checking
        command.validate_parameters_with_types(&execution.arguments)?;
        // ... rest of execution
    }
}
```

### **Phase 2: Schema Consistency (HIGH)**

#### **Task 2.1: Update Example Files**
**Files**: `examples/*-command.json`

Change from:
```json
"parameters": {
    "env": {
        "type": "string",
        "required": true
    }
}
```

To:
```json
"parameters": [
    {
        "name": "env",
        "type": "string", 
        "required": true
    }
]
```

#### **Task 2.2: Create Custom Commands Schema Reference**
**File**: `docs/custom-commands-json-schema-reference.md` (NEW)

### **Phase 3: Testing & Validation (HIGH)**

#### **Task 3.1: Add Comprehensive Tests**
**File**: `crates/chat-cli/src/cli/custom_commands/tests.rs`

```rust
#[test]
fn test_parameter_type_validation() {
    // Test string, boolean, number, enum validation
}

#[test] 
fn test_security_validation() {
    // Test injection prevention
}

#[test]
fn test_schema_consistency() {
    // Test JSON parsing matches struct
}
```

#### **Task 3.2: Add Security Tests**
```rust
#[test]
fn test_command_injection_prevention() {
    let param = CommandParameter {
        name: "env".to_string(),
        param_type: "string".to_string(),
        pattern: Some("^[a-zA-Z0-9_-]+$".to_string()),
        // ...
    };
    
    // Should fail
    assert!(param.validate_value("prod; rm -rf /").is_err());
    
    // Should pass  
    assert!(param.validate_value("production").is_ok());
}
```

### **Phase 4: Documentation & Cleanup (MEDIUM)**

#### **Task 4.1: Fix Skills Example**
**File**: `examples/commit-message-skill.json`

Update to use technical schema with `type` field.

#### **Task 4.2: Update Documentation**
Update any docs referencing the old Custom Commands schema.

## Implementation Order

### **Week 1: Security (CRITICAL)**
1. ✅ Task 1.1: Enhance CommandParameter struct
2. ✅ Task 1.2: Add validation logic  
3. ✅ Task 1.3: Update executor validation
4. ✅ Task 3.2: Add security tests

### **Week 2: Consistency (HIGH)**  
5. ✅ Task 2.1: Update example files
6. ✅ Task 2.2: Create schema reference
7. ✅ Task 3.1: Add comprehensive tests

### **Week 3: Cleanup (MEDIUM)**
8. ✅ Task 4.1: Fix skills example
9. ✅ Task 4.2: Update documentation

## Success Criteria

### **Security Fixed**
- [ ] No command injection vulnerabilities
- [ ] All parameters validated by type
- [ ] Pattern validation prevents malicious input
- [ ] Security tests pass

### **Schema Consistent**
- [ ] Examples match implementation
- [ ] Schema reference document created
- [ ] All tests pass
- [ ] No runtime schema errors

### **Quality Improved**
- [ ] Comprehensive test coverage
- [ ] Clear error messages
- [ ] Documentation updated

## Risk Mitigation

### **Backward Compatibility**
- Keep existing `description` and `default_value` fields
- Make new fields optional initially
- Gradual migration path for existing commands

### **Testing Strategy**
- Add security-focused tests first
- Test with real command injection attempts
- Validate all parameter types work correctly

## Files to Modify

### **Core Implementation**
- `crates/chat-cli/src/cli/custom_commands/types.rs` - Enhance struct
- `crates/chat-cli/src/cli/custom_commands/validation.rs` - NEW validation logic
- `crates/chat-cli/src/cli/custom_commands/executor.rs` - Update validation calls

### **Examples & Documentation**  
- `examples/git-helper-command.json` - Update schema
- `examples/error-demo-command.json` - Update schema
- `examples/commit-message-skill.json` - Fix deprecated schema
- `docs/custom-commands-json-schema-reference.md` - NEW reference

### **Tests**
- `crates/chat-cli/src/cli/custom_commands/tests.rs` - Add comprehensive tests

## Next Steps

1. **START IMMEDIATELY**: Task 1.1 - Enhance CommandParameter struct (security critical)
2. **Validate approach** with simple test case
3. **Implement security validation** before any other changes
4. **Test thoroughly** with injection attempts
5. **Update examples** to match new schema

**CRITICAL**: The security vulnerability should be fixed before any other work to prevent potential system compromise.
