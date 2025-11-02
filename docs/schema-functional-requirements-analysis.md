# Schema Functional Requirements Analysis

**Date**: November 1, 2025  
**Purpose**: Determine the correct JSON schemas based on actual functional requirements

## Executive Summary

This analysis examines what each schema system is **actually trying to achieve** functionally, then determines the correct schema structure based on those requirements rather than just documenting existing inconsistencies.

## Skills Schema Functional Analysis

### ✅ **What Skills Need to Do**
1. **Template Parameter Substitution**: Replace `{param_name}` in prompt templates
2. **Type Validation**: Validate parameter values match expected types
3. **Pattern Validation**: Validate strings match regex patterns (security)
4. **Enum Validation**: Restrict values to predefined lists
5. **Required Field Validation**: Ensure required parameters are provided
6. **Security Validation**: Prevent injection attacks through parameter validation

### ✅ **Current Implementation Analysis**
**Location**: `crates/chat-cli/src/cli/skills/types.rs:68-75`

```rust
pub struct Parameter {
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: String,           // ✅ NEEDED: "string", "number", "boolean", "enum"
    pub values: Option<Vec<String>>,  // ✅ NEEDED: For enum validation
    pub required: Option<bool>,       // ✅ NEEDED: For required field validation
    pub pattern: Option<String>,      // ✅ NEEDED: For regex/security validation
}
```

**Validation Logic**: `crates/chat-cli/src/cli/skills/validation.rs:108-150`
- ✅ Uses `param_type` for type checking
- ✅ Uses `values` for enum validation  
- ✅ Uses `pattern` for regex validation
- ✅ Uses `required` for mandatory field checking

### ✅ **Skills Schema Verdict: CORRECT**
The current Skills parameter schema is **functionally correct** and supports all required use cases:

```json
{
    "name": "parameter_name",
    "type": "string|number|boolean|enum",
    "required": true|false,
    "pattern": "regex_pattern",      // for security validation
    "values": ["opt1", "opt2"]       // for enum validation
}
```

## Custom Commands Schema Functional Analysis

### ⚠️ **What Custom Commands Need to Do**
1. **Parameter Substitution**: Replace `{{param_name}}` in command templates
2. **Required Field Validation**: Ensure required parameters are provided
3. **Default Value Handling**: Use defaults when parameters not provided
4. **Simple Type Validation**: Basic string parameter validation
5. **Command Execution**: Pass parameters to shell commands/scripts

### ⚠️ **Current Implementation Analysis**
**Location**: `crates/chat-cli/src/cli/custom_commands/types.rs:24-29`

```rust
pub struct CommandParameter {
    pub name: String,
    pub description: String,         // ❓ NEEDED?: Only for help/documentation
    pub required: bool,              // ✅ NEEDED: For required validation
    pub default_value: Option<String>, // ✅ NEEDED: For default handling
}
```

**Validation Logic**: `crates/chat-cli/src/cli/custom_commands/types.rs:95-104`
- ✅ Uses `required` for mandatory field checking
- ❌ **NO TYPE VALIDATION** - treats everything as strings
- ❌ **NO PATTERN VALIDATION** - no security validation
- ❌ **NO ENUM VALIDATION** - no value restrictions

**Execution Logic**: `crates/chat-cli/src/cli/custom_commands/executor.rs:25-30`
- ✅ Uses `{{param_name}}` template substitution
- ✅ Uses `default_value` for missing parameters

### ⚠️ **Custom Commands Schema Issues**

#### **Problem 1: Missing Type System**
**Current**: Everything treated as string
**Need**: Support for different parameter types (boolean flags, numbers, enums)

**Example Use Case**: 
```bash
/git commit --all=true --message="fix bug"
```
- `--all` should be boolean
- `--message` should be string

#### **Problem 2: No Security Validation**
**Current**: No pattern validation
**Need**: Prevent command injection through parameters

**Security Risk**:
```bash
/deploy --env="prod; rm -rf /"
```

#### **Problem 3: Schema Mismatch**
**Examples Use**: Object-based with rich types
**Code Uses**: Array-based with simple strings

## Functional Requirements Comparison

| Feature | Skills Schema | Custom Commands Schema | Custom Commands Need |
|---------|---------------|------------------------|---------------------|
| Type Validation | ✅ Full support | ❌ String only | ✅ **NEEDED** |
| Pattern Validation | ✅ Regex support | ❌ None | ✅ **NEEDED** (security) |
| Enum Validation | ✅ Values array | ❌ None | ✅ **NEEDED** |
| Required Fields | ✅ Boolean | ✅ Boolean | ✅ **CURRENT OK** |
| Default Values | ❌ Not needed | ✅ String | ✅ **CURRENT OK** |
| Description | ❌ Not needed | ✅ String | ❓ **NICE TO HAVE** |

## Recommended Schema Corrections

### ✅ **Skills Schema: Keep Current**
The Skills schema is functionally correct and well-implemented.

### ⚠️ **Custom Commands Schema: Needs Enhancement**

#### **Option A: Enhance Current Structure (RECOMMENDED)**
```rust
pub struct CommandParameter {
    pub name: String,
    pub param_type: String,              // ADD: "string", "boolean", "number", "enum"
    pub required: bool,                  // KEEP: Current functionality
    pub default_value: Option<String>,   // KEEP: Current functionality  
    pub description: Option<String>,     // MAKE OPTIONAL: For help text
    pub values: Option<Vec<String>>,     // ADD: For enum validation
    pub pattern: Option<String>,         // ADD: For security validation
}
```

**JSON Schema**:
```json
"parameters": [
    {
        "name": "env",
        "type": "enum",
        "values": ["dev", "staging", "prod"],
        "required": true,
        "description": "Deployment environment"
    },
    {
        "name": "dry_run", 
        "type": "boolean",
        "required": false,
        "default_value": "false",
        "description": "Preview changes without executing"
    }
]
```

#### **Option B: Align with Skills Schema**
Use identical parameter structure as Skills, add `default_value` field.

## Implementation Priority

### **HIGH PRIORITY: Security & Type Safety**
1. **Add type validation** to prevent parameter type errors
2. **Add pattern validation** to prevent command injection
3. **Add enum validation** for restricted parameter values

### **MEDIUM PRIORITY: Schema Consistency**  
4. **Standardize JSON format** between examples and implementation
5. **Add comprehensive validation tests**
6. **Create schema reference documentation**

### **LOW PRIORITY: User Experience**
7. **Maintain description field** for help text
8. **Add autocomplete support** for enum parameters

## Conclusion

### ✅ **Skills Schema**: Functionally Complete
The Skills parameter schema correctly supports all required functionality with proper type validation, security checks, and enum support.

### ⚠️ **Custom Commands Schema**: Functionally Incomplete
The Custom Commands schema lacks essential features:
- **No type validation** (security risk)
- **No pattern validation** (injection vulnerability) 
- **No enum validation** (user experience issue)
- **Schema inconsistency** (maintenance problem)

**Recommendation**: Enhance Custom Commands schema to include type system and security validation similar to Skills schema, while maintaining backward compatibility with `default_value` and `description` fields.

**Next Steps**: 
1. Implement enhanced `CommandParameter` struct
2. Add validation logic similar to Skills
3. Update examples to match implementation
4. Add comprehensive tests
