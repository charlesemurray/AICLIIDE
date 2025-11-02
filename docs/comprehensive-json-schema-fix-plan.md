# Comprehensive JSON Schema Fix Implementation Plan

**Created**: November 2, 2025  
**Status**: READY FOR IMPLEMENTATION  
**Priority**: CRITICAL (Security vulnerabilities must be fixed immediately)

## Complete Issue Inventory

### 🚨 **CRITICAL SECURITY ISSUES**
1. **Command Injection Vulnerability** - Custom Commands have no parameter validation
2. **No Type Validation** - All parameters treated as strings, no type safety
3. **No Pattern Validation** - No regex validation to prevent malicious input

### ⚠️ **HIGH PRIORITY CONSISTENCY ISSUES**
4. **Schema Mismatch** - Examples use object-based, code uses array-based parameters
5. **Field Name Conflicts** - `default` vs `default_value`, inconsistent naming
6. **Missing Schema Documentation** - No Custom Commands schema reference

### 📝 **MEDIUM PRIORITY ISSUES**
7. **No Comprehensive Tests** - Custom Commands lack validation test coverage
8. **Skills Example Inconsistency** - One example uses deprecated schema
9. **Documentation Gaps** - Missing validation error documentation

## Detailed Implementation Plan

### **PHASE 1: CRITICAL SECURITY FIXES**

#### **Task 1.1: Enhance CommandParameter Struct**
**File**: `crates/chat-cli/src/cli/custom_commands/types.rs`
**Priority**: CRITICAL
**Estimated Time**: 2 hours

**Current Code**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandParameter {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub default_value: Option<String>,
}
```

**New Code**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandParameter {
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: String,              // NEW: "string", "boolean", "number", "enum"
    pub required: bool,                  // KEEP: Existing functionality
    pub default_value: Option<String>,   // KEEP: Existing functionality
    pub description: Option<String>,     // CHANGE: Make optional
    pub values: Option<Vec<String>>,     // NEW: For enum validation
    pub pattern: Option<String>,         // NEW: For security validation (regex)
}

impl CommandParameter {
    pub fn required(name: String, param_type: String) -> Self {
        Self {
            name,
            param_type,
            required: true,
            default_value: None,
            description: None,
            values: None,
            pattern: None,
        }
    }

    pub fn optional(name: String, param_type: String, default: Option<String>) -> Self {
        Self {
            name,
            param_type,
            required: false,
            default_value: default,
            description: None,
            values: None,
            pattern: None,
        }
    }

    pub fn enum_param(name: String, values: Vec<String>, required: bool) -> Self {
        Self {
            name,
            param_type: "enum".to_string(),
            required,
            default_value: None,
            description: None,
            values: Some(values),
            pattern: None,
        }
    }

    pub fn with_pattern(mut self, pattern: String) -> Self {
        self.pattern = Some(pattern);
        self
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}
```

#### **Task 1.2: Create Parameter Validation Module**
**File**: `crates/chat-cli/src/cli/custom_commands/validation.rs` (NEW FILE)
**Priority**: CRITICAL
**Estimated Time**: 4 hours

```rust
use super::types::{CommandParameter, CommandError};
use regex::Regex;

impl CommandParameter {
    pub fn validate_value(&self, value: &str) -> Result<String, CommandError> {
        match self.param_type.as_str() {
            "string" => self.validate_string(value),
            "boolean" => self.validate_boolean(value),
            "number" => self.validate_number(value),
            "enum" => self.validate_enum(value),
            _ => Err(CommandError::InvalidParameter(
                format!("Unknown parameter type '{}' for parameter '{}'", self.param_type, self.name)
            ))
        }
    }

    fn validate_string(&self, value: &str) -> Result<String, CommandError> {
        // Security: Check for command injection patterns
        self.validate_security(value)?;
        
        // Validate against pattern if provided
        if let Some(pattern) = &self.pattern {
            let regex = Regex::new(pattern)
                .map_err(|_| CommandError::InvalidParameter(
                    format!("Invalid regex pattern for parameter '{}'", self.name)
                ))?;
            
            if !regex.is_match(value) {
                return Err(CommandError::InvalidParameter(
                    format!("Parameter '{}' does not match pattern '{}'", self.name, pattern)
                ));
            }
        }
        
        Ok(value.to_string())
    }

    fn validate_boolean(&self, value: &str) -> Result<String, CommandError> {
        match value.to_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => Ok("true".to_string()),
            "false" | "0" | "no" | "off" => Ok("false".to_string()),
            _ => Err(CommandError::InvalidParameter(
                format!("Parameter '{}' must be a boolean (true/false)", self.name)
            ))
        }
    }

    fn validate_number(&self, value: &str) -> Result<String, CommandError> {
        value.parse::<f64>()
            .map_err(|_| CommandError::InvalidParameter(
                format!("Parameter '{}' must be a number", self.name)
            ))?;
        Ok(value.to_string())
    }

    fn validate_enum(&self, value: &str) -> Result<String, CommandError> {
        if let Some(allowed_values) = &self.values {
            if allowed_values.contains(&value.to_string()) {
                Ok(value.to_string())
            } else {
                Err(CommandError::InvalidParameter(
                    format!("Parameter '{}' must be one of: {:?}", self.name, allowed_values)
                ))
            }
        } else {
            Err(CommandError::InvalidParameter(
                format!("Enum parameter '{}' missing allowed values", self.name)
            ))
        }
    }

    fn validate_security(&self, value: &str) -> Result<(), CommandError> {
        // Check for dangerous command injection patterns
        let dangerous_patterns = [
            ";", "|", "&", "$", "`", "$(", 
            "rm -rf", "sudo", "chmod", "chown",
            "../", "..\\", "/etc/", "C:\\",
            "powershell", "cmd.exe", "bash -c", "sh -c",
            "eval", "exec", "system", "popen"
        ];

        for pattern in &dangerous_patterns {
            if value.contains(pattern) {
                return Err(CommandError::InvalidParameter(
                    format!("Parameter '{}' contains potentially dangerous pattern: '{}'", self.name, pattern)
                ));
            }
        }

        Ok(())
    }
}

pub struct ParameterValidator;

impl ParameterValidator {
    pub fn validate_all(
        parameters: &[CommandParameter], 
        args: &std::collections::HashMap<String, String>
    ) -> Result<std::collections::HashMap<String, String>, CommandError> {
        let mut validated_args = std::collections::HashMap::new();

        // Check required parameters
        for param in parameters {
            if param.required && !args.contains_key(&param.name) {
                return Err(CommandError::InvalidParameter(
                    format!("Required parameter '{}' is missing", param.name)
                ));
            }
        }

        // Validate provided parameters
        for (key, value) in args {
            if let Some(param) = parameters.iter().find(|p| p.name == *key) {
                let validated_value = param.validate_value(value)?;
                validated_args.insert(key.clone(), validated_value);
            } else {
                return Err(CommandError::InvalidParameter(
                    format!("Unknown parameter: '{}'", key)
                ));
            }
        }

        // Add default values for missing optional parameters
        for param in parameters {
            if !param.required && !validated_args.contains_key(&param.name) {
                if let Some(default) = &param.default_value {
                    let validated_default = param.validate_value(default)?;
                    validated_args.insert(param.name.clone(), validated_default);
                }
            }
        }

        Ok(validated_args)
    }
}
```

#### **Task 1.3: Update CommandError Enum**
**File**: `crates/chat-cli/src/cli/custom_commands/types.rs`
**Priority**: CRITICAL
**Estimated Time**: 30 minutes

```rust
#[derive(Debug)]
pub enum CommandError {
    NotFound(String),
    InvalidParameter(String),
    SecurityViolation(String),      // NEW: For security-related errors
    TypeValidationFailed(String),   // NEW: For type validation errors
    ExecutionFailed(String),
    RegistryError(String),
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandError::NotFound(msg) => write!(f, "Command not found: {}", msg),
            CommandError::InvalidParameter(msg) => write!(f, "Invalid parameter: {}", msg),
            CommandError::SecurityViolation(msg) => write!(f, "Security violation: {}", msg),
            CommandError::TypeValidationFailed(msg) => write!(f, "Type validation failed: {}", msg),
            CommandError::ExecutionFailed(msg) => write!(f, "Execution failed: {}", msg),
            CommandError::RegistryError(msg) => write!(f, "Registry error: {}", msg),
        }
    }
}

impl std::error::Error for CommandError {}
```

#### **Task 1.4: Update CommandExecutor with Enhanced Validation**
**File**: `crates/chat-cli/src/cli/custom_commands/executor.rs`
**Priority**: CRITICAL
**Estimated Time**: 1 hour

```rust
use super::types::{CustomCommand, CommandHandler, CommandExecution, CommandError};
use super::validation::ParameterValidator;
use std::collections::HashMap;
use std::process::Command;

impl CommandExecutor {
    pub fn execute(command: &CustomCommand, execution: &CommandExecution) -> Result<String, CommandError> {
        // Enhanced parameter validation with type checking and security
        let validated_args = ParameterValidator::validate_all(&command.parameters, &execution.arguments)?;

        // Execute based on handler type with validated parameters
        match &command.handler {
            CommandHandler::Script { command: cmd, args } => {
                Self::execute_script(cmd, args, &validated_args)
            }
            CommandHandler::Alias { target } => {
                Self::execute_alias(target, &validated_args)
            }
            CommandHandler::Builtin { function_name } => {
                Self::execute_builtin(function_name, &validated_args)
            }
        }
    }

    fn execute_script(script: &str, args: &[String], params: &HashMap<String, String>) -> Result<String, CommandError> {
        // Replace template variables in script with VALIDATED parameters
        let mut processed_script = script.to_string();
        for (key, value) in params {
            let placeholder = format!("{{{{{}}}}}", key);
            processed_script = processed_script.replace(&placeholder, value);
        }

        // Execute the processed script
        let output = Command::new("sh")
            .arg("-c")
            .arg(&processed_script)
            .args(args)
            .output()
            .map_err(|e| CommandError::ExecutionFailed(format!("Failed to execute script: {}", e)))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(CommandError::ExecutionFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ))
        }
    }

    // ... rest of implementation
}
```

#### **Task 1.5: Update Module Declarations**
**File**: `crates/chat-cli/src/cli/custom_commands/mod.rs`
**Priority**: CRITICAL
**Estimated Time**: 5 minutes

```rust
pub mod executor;
pub mod types;
pub mod validation;  // NEW: Add validation module

pub use executor::*;
pub use types::*;
pub use validation::*;  // NEW: Export validation types
```

### **PHASE 2: COMPREHENSIVE SECURITY TESTING**

#### **Task 2.1: Add Security-Focused Tests**
**File**: `crates/chat-cli/src/cli/custom_commands/tests.rs`
**Priority**: CRITICAL
**Estimated Time**: 3 hours

```rust
use super::*;
use std::collections::HashMap;

#[cfg(test)]
mod security_tests {
    use super::*;

    #[test]
    fn test_command_injection_prevention() {
        let param = CommandParameter::required("env".to_string(), "string".to_string())
            .with_pattern("^[a-zA-Z0-9_-]+$".to_string());

        // Should FAIL - injection attempts
        assert!(param.validate_value("prod; rm -rf /").is_err());
        assert!(param.validate_value("test | cat /etc/passwd").is_err());
        assert!(param.validate_value("dev && sudo rm -rf /").is_err());
        assert!(param.validate_value("staging; powershell -c 'evil'").is_err());
        assert!(param.validate_value("prod$(rm -rf /)").is_err());
        assert!(param.validate_value("test`rm -rf /`").is_err());

        // Should PASS - safe values
        assert!(param.validate_value("production").is_ok());
        assert!(param.validate_value("dev-environment").is_ok());
        assert!(param.validate_value("test_env").is_ok());
    }

    #[test]
    fn test_path_traversal_prevention() {
        let param = CommandParameter::required("path".to_string(), "string".to_string())
            .with_pattern("^[a-zA-Z0-9/_.-]+$".to_string());

        // Should FAIL - path traversal attempts
        assert!(param.validate_value("../../../etc/passwd").is_err());
        assert!(param.validate_value("..\\..\\windows\\system32").is_err());
        assert!(param.validate_value("/etc/shadow").is_err());

        // Should PASS - safe paths
        assert!(param.validate_value("./config/app.json").is_ok());
        assert!(param.validate_value("data/file.txt").is_ok());
    }

    #[test]
    fn test_type_validation() {
        // String parameter
        let string_param = CommandParameter::required("message".to_string(), "string".to_string());
        assert!(string_param.validate_value("hello world").is_ok());

        // Boolean parameter
        let bool_param = CommandParameter::required("enabled".to_string(), "boolean".to_string());
        assert_eq!(bool_param.validate_value("true").unwrap(), "true");
        assert_eq!(bool_param.validate_value("false").unwrap(), "false");
        assert_eq!(bool_param.validate_value("1").unwrap(), "true");
        assert_eq!(bool_param.validate_value("0").unwrap(), "false");
        assert!(bool_param.validate_value("maybe").is_err());

        // Number parameter
        let num_param = CommandParameter::required("count".to_string(), "number".to_string());
        assert!(num_param.validate_value("42").is_ok());
        assert!(num_param.validate_value("3.14").is_ok());
        assert!(num_param.validate_value("not_a_number").is_err());

        // Enum parameter
        let enum_param = CommandParameter::enum_param(
            "priority".to_string(),
            vec!["low".to_string(), "medium".to_string(), "high".to_string()],
            true
        );
        assert!(enum_param.validate_value("high").is_ok());
        assert!(enum_param.validate_value("invalid").is_err());
    }

    #[test]
    fn test_parameter_validator() {
        let parameters = vec![
            CommandParameter::required("env".to_string(), "enum".to_string())
                .with_description("Environment".to_string()),
            CommandParameter::optional("dry_run".to_string(), "boolean".to_string(), Some("false".to_string())),
        ];

        // Set enum values manually for test
        let mut env_param = parameters[0].clone();
        env_param.values = Some(vec!["dev".to_string(), "staging".to_string(), "prod".to_string()]);
        let test_params = vec![env_param, parameters[1].clone()];

        let mut args = HashMap::new();
        args.insert("env".to_string(), "prod".to_string());
        args.insert("dry_run".to_string(), "true".to_string());

        let result = ParameterValidator::validate_all(&test_params, &args);
        assert!(result.is_ok());

        let validated = result.unwrap();
        assert_eq!(validated.get("env").unwrap(), "prod");
        assert_eq!(validated.get("dry_run").unwrap(), "true");
    }

    #[test]
    fn test_missing_required_parameter() {
        let parameters = vec![
            CommandParameter::required("required_param".to_string(), "string".to_string()),
        ];

        let args = HashMap::new(); // Empty - missing required parameter

        let result = ParameterValidator::validate_all(&parameters, &args);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Required parameter 'required_param' is missing"));
    }

    #[test]
    fn test_unknown_parameter() {
        let parameters = vec![
            CommandParameter::required("known_param".to_string(), "string".to_string()),
        ];

        let mut args = HashMap::new();
        args.insert("known_param".to_string(), "value".to_string());
        args.insert("unknown_param".to_string(), "value".to_string()); // Unknown parameter

        let result = ParameterValidator::validate_all(&parameters, &args);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown parameter: 'unknown_param'"));
    }
}
```

### **PHASE 3: SCHEMA CONSISTENCY FIXES**

#### **Task 3.1: Update Example Files to Match Implementation**
**Files**: `examples/git-helper-command.json`, `examples/error-demo-command.json`
**Priority**: HIGH
**Estimated Time**: 1 hour

**Current (INCONSISTENT)**:
```json
{
  "parameters": {
    "subcommand": {
      "type": "string",
      "required": true,
      "description": "Git subcommand to execute"
    }
  }
}
```

**New (CONSISTENT)**:
```json
{
  "parameters": [
    {
      "name": "subcommand",
      "type": "string",
      "required": true,
      "description": "Git subcommand to execute"
    }
  ]
}
```

#### **Task 3.2: Create Custom Commands Schema Reference**
**File**: `docs/custom-commands-json-schema-reference.md` (NEW)
**Priority**: HIGH
**Estimated Time**: 2 hours

```markdown
# Custom Commands JSON Schema Reference

## Core Schema Structure

```json
{
  "command": "string (required)",
  "description": "string (required)",
  "aliases": "array (optional)",
  "version": "string (optional)",
  "author": "string (optional)",
  "handler": "object (required)",
  "parameters": "array (optional)"
}
```

## Parameter Schema

```json
{
  "name": "string (required)",
  "type": "string (required)",
  "required": "boolean (optional, default: false)",
  "default_value": "string (optional)",
  "description": "string (optional)",
  "values": "array (optional, required for enum type)",
  "pattern": "string (optional, regex for validation)"
}
```

## Parameter Types

- `"string"` - Text values with optional pattern validation
- `"boolean"` - True/false values (accepts: true/false, 1/0, yes/no, on/off)
- `"number"` - Numeric values (integers or floats)
- `"enum"` - Restricted to predefined values (requires `values` array)

## Security Features

- **Pattern Validation**: Regex patterns prevent malicious input
- **Injection Prevention**: Automatic detection of dangerous patterns
- **Type Safety**: Strong parameter type validation

## Complete Example

```json
{
  "command": "deploy",
  "description": "Deploy application to environment",
  "aliases": ["dep"],
  "version": "1.0.0",
  "handler": {
    "type": "script",
    "script": "deploy.sh {{env}} {{dry_run}}"
  },
  "parameters": [
    {
      "name": "env",
      "type": "enum",
      "values": ["dev", "staging", "prod"],
      "required": true,
      "description": "Target environment"
    },
    {
      "name": "dry_run",
      "type": "boolean",
      "required": false,
      "default_value": "false",
      "description": "Preview changes without executing"
    }
  ]
}
```
```

### **PHASE 4: SKILLS SCHEMA CLEANUP**

#### **Task 4.1: Fix Skills Example File**
**File**: `examples/commit-message-skill.json`
**Priority**: MEDIUM
**Estimated Time**: 15 minutes

**Current (DEPRECATED)**:
```json
"parameters": [
  {
    "name": "type",
    "description": "Commit type (feat, fix, docs, etc.)",
    "required": true
  }
]
```

**New (CORRECT)**:
```json
"parameters": [
  {
    "name": "type",
    "type": "enum",
    "values": ["feat", "fix", "docs", "style", "refactor", "test", "chore"],
    "required": true
  }
]
```

### **PHASE 5: COMPREHENSIVE TESTING & VALIDATION**

#### **Task 5.1: Add Integration Tests**
**File**: `crates/chat-cli/src/cli/custom_commands/integration_tests.rs` (NEW)
**Priority**: HIGH
**Estimated Time**: 2 hours

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_end_to_end_command_execution() {
        // Create a test command with parameters
        let mut cmd = CustomCommand::new_script(
            "test-cmd".to_string(),
            "Test command".to_string(),
            "echo 'Environment: {{env}}, Debug: {{debug}}'".to_string(),
        );

        cmd.parameters = vec![
            CommandParameter::enum_param(
                "env".to_string(),
                vec!["dev".to_string(), "prod".to_string()],
                true
            ),
            CommandParameter::optional("debug".to_string(), "boolean".to_string(), Some("false".to_string())),
        ];

        let mut args = HashMap::new();
        args.insert("env".to_string(), "dev".to_string());
        args.insert("debug".to_string(), "true".to_string());

        let execution = CommandExecution {
            command_name: "test-cmd".to_string(),
            arguments: args,
        };

        let result = CommandExecutor::execute(&cmd, &execution);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Environment: dev"));
        assert!(output.contains("Debug: true"));
    }

    #[test]
    fn test_security_prevents_execution() {
        let mut cmd = CustomCommand::new_script(
            "unsafe-cmd".to_string(),
            "Unsafe command".to_string(),
            "echo {{input}}".to_string(),
        );

        cmd.parameters = vec![
            CommandParameter::required("input".to_string(), "string".to_string())
                .with_pattern("^[a-zA-Z0-9\\s]+$".to_string()),
        ];

        let mut args = HashMap::new();
        args.insert("input".to_string(), "safe input; rm -rf /".to_string()); // Injection attempt

        let execution = CommandExecution {
            command_name: "unsafe-cmd".to_string(),
            arguments: args,
        };

        let result = CommandExecutor::execute(&cmd, &execution);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("dangerous pattern"));
    }
}
```

## Implementation Timeline

### **Week 1: Critical Security (MUST COMPLETE)**
- **Day 1-2**: Tasks 1.1-1.3 (Enhance struct, add validation)
- **Day 3**: Task 1.4 (Update executor)
- **Day 4**: Task 1.5 + Task 2.1 (Module updates + security tests)
- **Day 5**: Testing and bug fixes

### **Week 2: Schema Consistency**
- **Day 1**: Task 3.1 (Update example files)
- **Day 2**: Task 3.2 (Create schema reference)
- **Day 3**: Task 5.1 (Integration tests)
- **Day 4-5**: Testing and documentation

### **Week 3: Cleanup & Polish**
- **Day 1**: Task 4.1 (Fix skills example)
- **Day 2-3**: Additional testing and edge cases
- **Day 4-5**: Documentation updates and final validation

## Success Criteria

### **Security (CRITICAL)**
- [ ] All command injection attempts blocked
- [ ] Type validation prevents runtime errors
- [ ] Pattern validation enforces security rules
- [ ] Security tests pass with 100% coverage

### **Consistency (HIGH)**
- [ ] Examples match implementation exactly
- [ ] Schema reference document complete
- [ ] All JSON files validate against schema
- [ ] No runtime schema parsing errors

### **Quality (MEDIUM)**
- [ ] Comprehensive test coverage (>90%)
- [ ] Clear error messages for validation failures
- [ ] Documentation complete and accurate
- [ ] Backward compatibility maintained

## Risk Mitigation

### **Breaking Changes**
- Make new fields optional initially
- Provide migration guide for existing commands
- Support both old and new schemas during transition

### **Security Testing**
- Test with real injection payloads
- Validate against OWASP injection patterns
- Security review of validation logic

### **Performance**
- Benchmark parameter validation overhead
- Optimize regex compilation and caching
- Profile memory usage with large parameter sets

## Files Modified Summary

### **New Files**
- `crates/chat-cli/src/cli/custom_commands/validation.rs`
- `crates/chat-cli/src/cli/custom_commands/integration_tests.rs`
- `docs/custom-commands-json-schema-reference.md`

### **Modified Files**
- `crates/chat-cli/src/cli/custom_commands/types.rs`
- `crates/chat-cli/src/cli/custom_commands/executor.rs`
- `crates/chat-cli/src/cli/custom_commands/tests.rs`
- `crates/chat-cli/src/cli/custom_commands/mod.rs`
- `examples/git-helper-command.json`
- `examples/error-demo-command.json`
- `examples/commit-message-skill.json`

## Dependencies to Add

Add to `Cargo.toml`:
```toml
[dependencies]
regex = "1.0"  # For pattern validation (if not already present)
```

This comprehensive plan addresses every identified issue with detailed implementation steps, security considerations, and validation criteria.
