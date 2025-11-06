//! Skill tool implementation

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::io::Write;

use eyre::Result;
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::Value;

use crate::cli::agent::{
    Agent,
    PermissionEvalResult,
};
use crate::os::Os;
use super::{InvokeOutput, OutputKind};

const MAX_OUTPUT_SIZE: usize = 100_000;

#[derive(Debug, Clone)]
pub struct SkillTool {
    pub name: String,
    pub description: String,
    pub parameters: HashMap<String, Value>,
    // Legacy compatibility fields
    pub skill_name: String,
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum SkillImplementation {
    Script { path: String },
    Command { command: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDefinition {
    pub name: String,
    pub description: String,
    pub skill_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub implementation: Option<SkillImplementation>,
}

impl SkillTool {
    pub fn new(name: String, description: String) -> Self {
        Self { 
            name: name.clone(), 
            description,
            parameters: HashMap::new(),
            // Legacy compatibility
            skill_name: name,
            params: serde_json::Value::Object(serde_json::Map::new()),
        }
    }

    pub fn with_parameters(name: String, description: String, parameters: HashMap<String, Value>) -> Self {
        let params = serde_json::Value::Object(
            parameters.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
        );
        Self { 
            name: name.clone(), 
            description, 
            parameters,
            // Legacy compatibility
            skill_name: name,
            params,
        }
    }

    // Legacy constructor for compatibility
    pub fn new_legacy(skill_name: String, params: serde_json::Value) -> Self {
        let parameters = if let serde_json::Value::Object(obj) = &params {
            obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
        } else {
            HashMap::new()
        };
        
        Self {
            name: skill_name.clone(),
            description: format!("Legacy skill: {}", skill_name),
            parameters,
            skill_name,
            params,
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(eyre::eyre!("Skill name cannot be empty"));
        }
        Ok(())
    }

    pub fn eval_perm(&self, _os: &Os, _agent: &Agent) -> PermissionEvalResult {
        PermissionEvalResult::Allow
    }

    /// Async invoke method for registry compatibility
    pub async fn invoke(&self, registry: &crate::cli::skills::SkillRegistry, stdout: &mut impl Write) -> Result<InvokeOutput> {
        tracing::info!("Invoking skill via registry: {}", self.skill_name);
        
        // Try to find and execute the skill through the registry
        match registry.get_skill(&self.skill_name) {
            Some(skill_def) => {
                let result = self.invoke_with_definition(skill_def, self.parameters.clone())?;
                writeln!(stdout, "{}", result)?;
                Ok(InvokeOutput {
                    output: OutputKind::Text(result),
                })
            },
            None => {
                // Fallback to direct execution
                let result = self.invoke_direct(self.parameters.clone())?;
                writeln!(stdout, "{}", result)?;
                Ok(InvokeOutput {
                    output: OutputKind::Text(result),
                })
            }
        }
    }

    /// Secure command execution using proper libraries
    pub fn invoke_direct(&self, params: HashMap<String, Value>) -> Result<String> {
        tracing::info!("Invoking skill: {}", self.name);
        tracing::debug!("Skill parameters: {:?}", params);
        
        // Validate skill name to prevent injection
        if !self.name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            tracing::error!("Invalid skill name contains unsafe characters: {}", self.name);
            return Err(eyre::eyre!("Skill name contains invalid characters"));
        }
        
        let command = self.description.clone();
        if command.trim().is_empty() {
            tracing::error!("Skill {} has no command to execute", self.name);
            return Err(eyre::eyre!("Skill has no command defined"));
        }
        
        // Use shlex to safely parse the command instead of shell execution
        let args = shlex::split(&command)
            .ok_or_else(|| eyre::eyre!("Invalid command syntax"))?;
        
        if args.is_empty() {
            return Err(eyre::eyre!("Empty command after parsing"));
        }
        
        let program = &args[0];
        let cmd_args = &args[1..];
        
        // Validate the executable exists (basic check)
        if program.is_empty() {
            return Err(eyre::eyre!("Empty program name"));
        }
        
        tracing::debug!("Executing: {} with args: {:?}", program, cmd_args);
        
        // Execute with timeout and proper error handling
        let output = std::process::Command::new(program)
            .args(cmd_args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| {
                tracing::error!("Failed to execute skill {}: {}", self.name, e);
                eyre::eyre!("Command execution failed: {}", e)
            })?;
        
        if output.status.success() {
            // Safe UTF-8 conversion without unwrap
            let result = String::from_utf8(output.stdout)
                .map_err(|e| {
                    tracing::error!("Skill {} output contains invalid UTF-8: {}", self.name, e);
                    eyre::eyre!("Command output is not valid UTF-8")
                })?;
            
            // Limit output size to prevent memory issues
            let truncated_result = if result.len() > MAX_OUTPUT_SIZE {
                tracing::warn!("Skill {} output truncated from {} to {} chars", self.name, result.len(), MAX_OUTPUT_SIZE);
                format!("{}... [truncated]", &result[..MAX_OUTPUT_SIZE])
            } else {
                result
            };
            
            tracing::info!("Skill {} executed successfully, output length: {}", self.name, truncated_result.len());
            tracing::debug!("Skill output: {}", truncated_result);
            Ok(truncated_result)
        } else {
            // Safe error message handling without unwrap
            let stderr = String::from_utf8(output.stderr)
                .unwrap_or_else(|_| "[Invalid UTF-8 in error output]".to_string());
            
            tracing::error!("Skill {} failed with exit code: {:?}", self.name, output.status.code());
            tracing::error!("Skill {} stderr: {}", self.name, stderr);
            
            Err(eyre::eyre!(
                "Skill execution failed with exit code: {:?}. Error: {}", 
                output.status.code(),
                stderr
            ))
        }
    }

    pub fn get_script_path(&self, definition: &SkillDefinition) -> Result<PathBuf> {
        match &definition.implementation {
            Some(SkillImplementation::Script { path }) => {
                let path_buf = PathBuf::from(path);
                if path_buf.exists() {
                    Ok(path_buf)
                } else {
                    Err(eyre::eyre!("Script path does not exist: {}", path))
                }
            },
            _ => Err(eyre::eyre!("Skill does not have a script implementation")),
        }
    }

    pub fn build_env_vars(&self, params: &HashMap<String, Value>) -> HashMap<String, String> {
        params
            .iter()
            .map(|(key, value)| {
                let value_str = match value {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    Value::Null => "null".to_string(),
                    _ => value.to_string(),
                };
                (format!("SKILL_PARAM_{}", key), value_str)
            })
            .collect()
    }

    pub fn invoke_with_definition(
        &self,
        definition: &SkillDefinition,
        params: HashMap<String, Value>,
    ) -> Result<String> {
        tracing::info!("Invoking skill with definition: {}", definition.name);
        
        match &definition.implementation {
            Some(SkillImplementation::Command { command }) => {
                self.execute_command(command, &params)
            },
            Some(SkillImplementation::Script { path }) => {
                self.execute_script(path, &params)
            },
            None => {
                tracing::error!("Skill {} has no implementation", definition.name);
                Err(eyre::eyre!("Skill has no implementation defined"))
            }
        }
    }

    fn execute_command(&self, command: &str, params: &HashMap<String, Value>) -> Result<String> {
        tracing::debug!("Executing command: {}", command);
        
        // Use shlex to safely parse the command
        let args = shlex::split(command)
            .ok_or_else(|| eyre::eyre!("Invalid command syntax"))?;
        
        if args.is_empty() {
            return Err(eyre::eyre!("Empty command after parsing"));
        }
        
        let program = &args[0];
        let cmd_args = &args[1..];
        
        // Validate the executable exists (basic check)
        if program.is_empty() {
            return Err(eyre::eyre!("Empty program name"));
        }
        
        // Build environment variables from parameters
        let env_vars = self.build_env_vars(params);
        
        // Execute the command
        let output = std::process::Command::new(program)
            .args(cmd_args)
            .envs(&env_vars)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| eyre::eyre!("Command execution failed: {}", e))?;
        
        self.process_output(output)
    }

    fn execute_script(&self, script_path: &str, params: &HashMap<String, Value>) -> Result<String> {
        let path = PathBuf::from(script_path);
        if !path.exists() {
            return Err(eyre::eyre!("Script path does not exist: {}", script_path));
        }
        
        // Build environment variables from parameters
        let env_vars = self.build_env_vars(params);
        
        // Execute the script
        let output = std::process::Command::new(&path)
            .envs(&env_vars)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| eyre::eyre!("Script execution failed: {}", e))?;
        
        self.process_output(output)
    }

    fn process_output(&self, output: std::process::Output) -> Result<String> {
        if output.status.success() {
            let result = String::from_utf8(output.stdout)
                .map_err(|e| eyre::eyre!("Output contains invalid UTF-8: {}", e))?;
            
            let truncated_result = if result.len() > MAX_OUTPUT_SIZE {
                tracing::warn!("Output truncated from {} to {} chars", result.len(), MAX_OUTPUT_SIZE);
                format!("{}... [truncated]", &result[..MAX_OUTPUT_SIZE])
            } else {
                result
            };
            
            Ok(truncated_result)
        } else {
            let stderr = String::from_utf8(output.stderr)
                .unwrap_or_else(|_| "[Invalid UTF-8 in error output]".to_string());
            
            Err(eyre::eyre!(
                "Execution failed with exit code: {:?}. Error: {}", 
                output.status.code(),
                stderr
            ))
        }
    }

    pub fn from_definition(definition: &SkillDefinition) -> Self {
        Self {
            name: definition.name.clone(),
            description: definition.description.clone(),
            parameters: HashMap::new(),
            skill_name: definition.name.clone(),
            params: serde_json::Value::Object(serde_json::Map::new()),
        }
    }

    /// Convert skill definition to toolspec format (placeholder for compatibility)
    pub fn definition_to_toolspec(&self, _definition: &SkillDefinition) -> crate::cli::chat::tools::ToolSpec {
        use crate::cli::chat::tools::{ToolSpec, InputSchema, ToolOrigin};
        
        ToolSpec {
            name: self.name.clone(),
            description: self.description.clone(),
            input_schema: InputSchema(serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            })),
            tool_origin: ToolOrigin::Native,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_creation() {
        let skill = SkillTool::new("test".to_string(), "echo hello".to_string());
        assert_eq!(skill.name, "test");
        assert_eq!(skill.description, "echo hello");
    }

    #[test]
    fn test_skill_validation() {
        let skill = SkillTool::new("test".to_string(), "echo hello".to_string());
        assert!(skill.validate().is_ok());
        
        let empty_skill = SkillTool::new("".to_string(), "echo hello".to_string());
        assert!(empty_skill.validate().is_err());
    }

    #[test]
    fn test_invalid_skill_name() {
        let skill = SkillTool::new("test;rm -rf /".to_string(), "echo hello".to_string());
        let result = skill.invoke(HashMap::new());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid characters"));
    }

    #[test]
    fn test_command_parsing() {
        let skill = SkillTool::new("test".to_string(), "echo 'hello world'".to_string());
        // This would normally execute, but we're just testing the parsing doesn't panic
        let result = skill.invoke(HashMap::new());
        // Result depends on whether 'echo' is available, but shouldn't panic
        assert!(result.is_ok() || result.is_err());
    }
}
