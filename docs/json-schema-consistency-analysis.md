# JSON Schema Consistency Analysis

**Date**: November 1, 2025  
**Status**: ⚠️ MIXED - Skills schema resolved, Custom Commands schema inconsistent  
**Reference**: [Skills JSON Schema Reference](./skills-json-schema-reference.md)

## Executive Summary

This analysis examines JSON schema consistency across the Q CLI codebase for **both Skills and Custom Commands**. While Skills schema inconsistencies have been resolved, Custom Commands use a different schema pattern with their own inconsistencies.

## Schema Types Overview

### 1. Skills JSON Schema
**Purpose**: Define reusable skills for AI interactions  
**File Pattern**: `*.json` in skills directories  
**Status**: ✅ CONSISTENT

### 2. Custom Commands JSON Schema  
**Purpose**: Define slash commands for CLI operations  
**File Pattern**: `*-command.json` in examples  
**Status**: ⚠️ INCONSISTENT

## Skills Schema Analysis

### ✅ **RESOLVED ISSUES**

#### 1. Parameter Schema Standardization
**Issue**: Multiple competing parameter schemas existed across the codebase
- **Documentation Schema**: Used `description`, `default` fields (user-friendly)
- **Technical Schema**: Used `type`, `pattern`, `values` fields (implementation)
- **Hybrid Schema**: Mixed both approaches inconsistently

**Resolution**: Standardized on technical schema with `type` field as documented in [Skills JSON Schema Reference](./skills-json-schema-reference.md)

#### 2. Manual Verification Test Fixed
**Issue**: Test used old documentation schema causing skill loading failures
**Resolution**: Updated to use correct technical parameter schema with `type` field

### ✅ **CURRENT CONSISTENT AREAS**

#### 1. Code Generation Templates (`skills_cli.rs`)
**Location**: `crates/chat-cli/src/cli/skills_cli.rs:404-480`
**Status**: ✅ CONSISTENT - Uses correct technical schema

```json
"parameters": [
    {
        "name": "type",
        "type": "enum",
        "values": ["test", "documentation", "example"],
        "required": true
    },
    {
        "name": "language", 
        "type": "string",
        "pattern": "^[a-zA-Z]+$",
        "required": true
    }
]
```

#### 2. Test Skills (`.q-skills/` directory)
**Status**: ✅ CONSISTENT - All use technical schema

- `greeter.json`: Uses `type: "string"`
- `test-prompt.json`: Uses `type: "enum"` with `values` and `type: "string"` with `pattern`
- `prompt-inline-test.json`: Uses technical schema

#### 3. JSON Schema Tests
**Location**: `crates/chat-cli/src/cli/skills/tests/json_schema_tests.rs`
**Status**: ✅ CONSISTENT - 23 comprehensive tests validate technical schema

#### 4. Documentation Examples
**Location**: `docs/skills-system-design-v2.md`, `docs/skill-creation-assistant.md`
**Status**: ✅ CONSISTENT - All examples use technical schema

### ⚠️ **SKILLS INCONSISTENT AREAS**

#### 1. Example Skills Directory
**Location**: `examples/commit-message-skill.json`
**Issue**: Uses old documentation schema with `description` fields
**Impact**: LOW - Example file, not used in runtime

```json
// INCONSISTENT - Uses old schema
"parameters": [
    {
        "name": "type",
        "description": "Commit type (feat, fix, docs, etc.)",
        "required": true
    }
]
```

## Custom Commands Schema Analysis

### ⚠️ **MAJOR INCONSISTENCIES IDENTIFIED**

#### 1. Multiple Parameter Schema Patterns
**Issue**: Custom Commands use different parameter schemas across different contexts

**Pattern A - Object-based (Examples)**:
```json
"parameters": {
    "path": {
        "type": "string",
        "required": false,
        "description": "Directory path to list"
    }
}
```

**Pattern B - Array-based (Code)**:
```json
"parameters": [
    {
        "name": "path",
        "description": "Directory path to list", 
        "required": false,
        "default_value": "."
    }
]
```

#### 2. Field Name Inconsistencies
**Examples vs Code Implementation**:
- Examples use: `"required": true/false`
- Code uses: `"required": bool` + `"default_value": Option<String>`
- Examples use: `"default": "value"`
- Code uses: `"default_value": Option<String>`

#### 3. Type System Differences
**Examples**: Rich type system with `"enum"`, `"array"`, `"boolean"`
**Code**: Simple string-based with `CommandParameter` struct

### 📊 **Custom Commands Inconsistency Details**

#### Examples Schema (git-helper-command.json)
```json
{
  "$schema": "https://raw.githubusercontent.com/aws/amazon-q-developer-cli/main/schemas/slash-command-v1.json",
  "parameters": {
    "subcommand": {
      "type": "string",
      "required": true,
      "description": "Git subcommand to execute"
    }
  }
}
```

#### Code Implementation (types.rs)
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandParameter {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub default_value: Option<String>,
}
```

#### Documentation Schema (custom-slash-commands-design.md)
```json
"parameters": {
    "path": {"type": "string", "default": "."},
    "all": {"type": "boolean", "default": false}
}
```

## Schema Validation Coverage

### ✅ **Skills Schema Coverage**
- **23 JSON schema tests** covering all field combinations
- **Parameter validation tests** for all supported types
- **Edge case testing** for missing/invalid fields
- **Complete skill examples** for all skill types

### ❌ **Custom Commands Schema Coverage**
- **No comprehensive schema tests** found
- **No validation between examples and implementation**
- **No schema reference documentation**
- **Inconsistent field usage** across codebase

## Recommendations

### 1. Fix Skills Schema Inconsistencies
**Priority**: LOW
**Action**: Update `examples/commit-message-skill.json` to use technical schema

### 2. Standardize Custom Commands Schema
**Priority**: HIGH
**Actions**:
- Choose single parameter schema pattern (object vs array)
- Align example files with code implementation
- Create Custom Commands schema reference document
- Add validation tests for Custom Commands schema

### 3. Create Schema Documentation
**Priority**: HIGH
**Actions**:
- Document Custom Commands JSON schema specification
- Create examples that match implementation
- Add schema validation to Custom Commands loading

### 4. Schema Validation Enforcement
**Priority**: MEDIUM
**Current**: Manual validation in Skills tests only
**Recommendation**: Add runtime schema validation for both Skills and Custom Commands

## Implementation Status

### ✅ **Skills Schema - Completed**
- [x] Identified all schema inconsistencies
- [x] Created comprehensive schema reference documentation
- [x] Implemented 23-test validation suite
- [x] Fixed manual verification test
- [x] Updated documentation examples
- [x] Standardized code generation templates

### ❌ **Custom Commands Schema - Needs Work**
- [ ] Analyze parameter schema inconsistencies
- [ ] Choose standard schema pattern (object vs array)
- [ ] Update examples to match implementation
- [ ] Create Custom Commands schema reference
- [ ] Add validation tests
- [ ] Align documentation with implementation

### 📋 **Remaining Tasks**
- [ ] Update `examples/commit-message-skill.json` to technical schema
- [ ] **HIGH PRIORITY**: Standardize Custom Commands parameter schema
- [ ] **HIGH PRIORITY**: Create Custom Commands schema documentation
- [ ] Add schema validation to CI/CD pipeline for both schemas

## Technical Details

### Skills Parameter Schema (STANDARD)
```json
{
    "name": "parameter_name",
    "type": "string|number|boolean|enum",
    "required": true|false,
    "pattern": "regex_pattern",      // for string type
    "values": ["opt1", "opt2"],      // for enum type
    "min": 0,                        // for number type
    "max": 100                       // for number type
}
```

### Custom Commands Parameter Schema (INCONSISTENT)
**Needs Standardization** - Multiple patterns in use:

**Option 1 - Object Pattern (Examples)**:
```json
"parameters": {
    "param_name": {
        "type": "string",
        "required": true,
        "description": "Parameter description"
    }
}
```

**Option 2 - Array Pattern (Code)**:
```json
"parameters": [
    {
        "name": "param_name",
        "description": "Parameter description",
        "required": true,
        "default_value": null
    }
]
```

## Conclusion

The JSON schema consistency analysis reveals a **mixed status**:

### ✅ **Skills Schema**: CONSISTENT
Major inconsistencies have been successfully resolved. The codebase now uses a standardized technical parameter schema with comprehensive validation coverage. Only minor inconsistencies remain in example files.

### ⚠️ **Custom Commands Schema**: INCONSISTENT  
Significant inconsistencies exist between examples, documentation, and code implementation. The parameter schema patterns are incompatible and need standardization.

**Overall Status**: ⚠️ **MIXED** - Skills resolved, Custom Commands need major schema standardization work.

**Immediate Action Required**: Standardize Custom Commands parameter schema to prevent similar issues that occurred with Skills schema.
