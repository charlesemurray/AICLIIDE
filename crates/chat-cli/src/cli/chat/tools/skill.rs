//! Skill tool implementation

use std::collections::HashMap;
use std::path::PathBuf;

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

const MAX_OUTPUT_SIZE: usize = 100_000;

#[derive(Debug, Clone)]
pub struct SkillTool {
    pub name: String,
    pub description: String,
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
        Self { name, description }
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

    pub fn invoke(&self, _params: HashMap<String, Value>) -> Result<String> {
        Ok("not implemented".to_string())
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

    pub fn execute_script(&self, definition: &SkillDefinition, params: &HashMap<String, Value>) -> Result<String> {
        let script_path = self.get_script_path(definition)?;
        let env_vars = self.build_env_vars(params);

        let output = std::process::Command::new(&script_path).envs(&env_vars).output()?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(eyre::eyre!("Script execution failed: {}", stderr))
        }
    }

    pub async fn execute_script_with_timeout(
        &self,
        definition: &SkillDefinition,
        params: &HashMap<String, Value>,
        timeout_secs: u64,
    ) -> Result<String> {
        let script_path = self.get_script_path(definition)?;
        let env_vars = self.build_env_vars(params);

        let timeout_duration = std::time::Duration::from_secs(timeout_secs);

        let output = tokio::time::timeout(
            timeout_duration,
            tokio::task::spawn_blocking(move || std::process::Command::new(&script_path).envs(&env_vars).output()),
        )
        .await
        .map_err(|_| eyre::eyre!("Script execution timeout after {} seconds", timeout_secs))?
        .map_err(|e| eyre::eyre!("Failed to spawn script: {}", e))??;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(eyre::eyre!("Script execution failed: {}", stderr))
        }
    }

    pub fn parse_command_template(&self, template: &str, params: &HashMap<String, Value>) -> Result<String> {
        let mut result = template.to_string();

        for (key, value) in params {
            let placeholder = format!("{{{{{}}}}}", key);
            let value_str = match value {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                Value::Null => "null".to_string(),
                _ => value.to_string(),
            };
            result = result.replace(&placeholder, &value_str);
        }

        Ok(result)
    }

    pub fn execute_command(&self, definition: &SkillDefinition, params: &HashMap<String, Value>) -> Result<String> {
        match &definition.implementation {
            Some(SkillImplementation::Command { command }) => {
                let parsed_command = self.parse_command_template(command, params)?;

                #[cfg(unix)]
                let output = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(&parsed_command)
                    .output()?;

                #[cfg(windows)]
                let output = std::process::Command::new("cmd")
                    .arg("/C")
                    .arg(&parsed_command)
                    .output()?;

                if output.status.success() {
                    Ok(String::from_utf8_lossy(&output.stdout).to_string())
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    Err(eyre::eyre!("Command execution failed: {}", stderr))
                }
            },
            _ => Err(eyre::eyre!("Skill does not have a command implementation")),
        }
    }

    pub async fn execute_command_with_timeout(
        &self,
        definition: &SkillDefinition,
        params: &HashMap<String, Value>,
        timeout_secs: u64,
    ) -> Result<String> {
        match &definition.implementation {
            Some(SkillImplementation::Command { command }) => {
                let parsed_command = self.parse_command_template(command, params)?;
                let timeout_duration = std::time::Duration::from_secs(timeout_secs);

                let output = tokio::time::timeout(
                    timeout_duration,
                    tokio::task::spawn_blocking(move || {
                        #[cfg(unix)]
                        let result = std::process::Command::new("sh").arg("-c").arg(&parsed_command).output();

                        #[cfg(windows)]
                        let result = std::process::Command::new("cmd")
                            .arg("/C")
                            .arg(&parsed_command)
                            .output();

                        result
                    }),
                )
                .await
                .map_err(|_| eyre::eyre!("Command execution timeout after {} seconds", timeout_secs))?
                .map_err(|e| eyre::eyre!("Failed to spawn command: {}", e))??;

                if output.status.success() {
                    Ok(String::from_utf8_lossy(&output.stdout).to_string())
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    Err(eyre::eyre!("Command execution failed: {}", stderr))
                }
            },
            _ => Err(eyre::eyre!("Skill does not have a command implementation")),
        }
    }

    pub fn format_output(&self, stdout: String, stderr: String) -> String {
        let mut output = String::new();

        if !stdout.is_empty() {
            output.push_str(&stdout);
        }

        if !stderr.is_empty() {
            if !output.is_empty() {
                output.push_str("\n\n");
            }
            output.push_str("Warnings/Info:\n");
            output.push_str(&stderr);
        }

        output
    }

    pub fn truncate_output(&self, output: String) -> String {
        if output.len() <= MAX_OUTPUT_SIZE {
            output
        } else {
            let truncated = &output[..MAX_OUTPUT_SIZE];
            format!(
                "{}\n\n... (output truncated, {} bytes total, showing first {} bytes)",
                truncated,
                output.len(),
                MAX_OUTPUT_SIZE
            )
        }
    }

    pub fn format_error(&self, error: &eyre::Error) -> String {
        format!("Error executing skill: {}", error)
    }

    pub fn invoke_with_definition(
        &self,
        definition: &SkillDefinition,
        params: HashMap<String, Value>,
    ) -> Result<String> {
        match &definition.implementation {
            Some(SkillImplementation::Script { .. }) => {
                let output = tokio::runtime::Runtime::new()?
                    .block_on(self.execute_script_with_timeout(definition, &params, 30))?;
                Ok(self.truncate_output(output))
            },
            Some(SkillImplementation::Command { .. }) => {
                let output = tokio::runtime::Runtime::new()?
                    .block_on(self.execute_command_with_timeout(definition, &params, 30))?;
                Ok(self.truncate_output(output))
            },
            None => Err(eyre::eyre!("Skill has no implementation defined")),
        }
    }

    pub fn definition_to_toolspec(&self, definition: &SkillDefinition) -> super::ToolSpec {
        use super::{
            InputSchema,
            ToolOrigin,
        };

        let input_schema = definition.parameters.clone().unwrap_or(serde_json::json!({
            "type": "object",
            "properties": {},
            "required": []
        }));

        super::ToolSpec {
            name: definition.name.clone(),
            description: definition.description.clone(),
            input_schema: InputSchema(input_schema),
            tool_origin: ToolOrigin::Skill(definition.name.clone()),
        }
    }

    pub fn from_definition(definition: &SkillDefinition) -> Self {
        Self {
            name: definition.name.clone(),
            description: definition.description.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_tool_creation() {
        let skill = SkillTool::new("test-skill".to_string(), "A test skill".to_string());
        assert_eq!(skill.name, "test-skill");
        assert_eq!(skill.description, "A test skill");
    }

    #[test]
    fn test_skill_tool_clone() {
        let skill = SkillTool::new("original".to_string(), "Original skill".to_string());
        let cloned = skill.clone();
        assert_eq!(cloned.name, skill.name);
        assert_eq!(cloned.description, skill.description);
    }

    #[test]
    fn test_skill_tool_validate_success() {
        let skill = SkillTool::new("valid-skill".to_string(), "Description".to_string());
        assert!(skill.validate().is_ok());
    }

    #[test]
    fn test_skill_tool_validate_empty_name() {
        let skill = SkillTool::new("".to_string(), "Description".to_string());
        let result = skill.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }

    #[tokio::test]
    async fn test_skill_tool_eval_perm() {
        use crate::cli::agent::{
            Agent,
            PermissionEvalResult,
        };
        use crate::os::Os;

        let skill = SkillTool::new("test-skill".to_string(), "Test".to_string());
        let os = Os::new().await.unwrap();
        let agent = Agent::default();

        let result = skill.eval_perm(&os, &agent);
        assert_eq!(result, PermissionEvalResult::Allow);
    }

    #[test]
    fn test_skill_definition_deserialize() {
        let json = r#"{
            "name": "test-skill",
            "description": "A test skill",
            "skill_type": "code_inline"
        }"#;

        let definition: SkillDefinition = serde_json::from_str(json).unwrap();
        assert_eq!(definition.name, "test-skill");
        assert_eq!(definition.description, "A test skill");
        assert_eq!(definition.skill_type, "code_inline");
    }

    #[test]
    fn test_skill_definition_with_parameters() {
        let json = r#"{
            "name": "calculator",
            "description": "A calculator skill",
            "skill_type": "code_inline",
            "parameters": {
                "type": "object",
                "properties": {
                    "a": {"type": "number"},
                    "b": {"type": "number"}
                }
            }
        }"#;

        let definition: SkillDefinition = serde_json::from_str(json).unwrap();
        assert_eq!(definition.name, "calculator");
        assert!(definition.parameters.is_some());
        let params = definition.parameters.unwrap();
        assert!(params.get("type").is_some());
    }

    #[test]
    fn test_skill_definition_script_implementation() {
        let json = r#"{
            "name": "test-skill",
            "description": "A test skill",
            "skill_type": "code_inline",
            "implementation": {
                "type": "script",
                "path": "./scripts/test.sh"
            }
        }"#;

        let definition: SkillDefinition = serde_json::from_str(json).unwrap();
        assert!(definition.implementation.is_some());
        match definition.implementation.unwrap() {
            SkillImplementation::Script { path } => {
                assert_eq!(path, "./scripts/test.sh");
            },
            _ => panic!("Expected Script implementation"),
        }
    }

    #[test]
    fn test_skill_definition_command_implementation() {
        let json = r#"{
            "name": "test-skill",
            "description": "A test skill",
            "skill_type": "code_inline",
            "implementation": {
                "type": "command",
                "command": "echo 'Hello'"
            }
        }"#;

        let definition: SkillDefinition = serde_json::from_str(json).unwrap();
        assert!(definition.implementation.is_some());
        match definition.implementation.unwrap() {
            SkillImplementation::Command { command } => {
                assert_eq!(command, "echo 'Hello'");
            },
            _ => panic!("Expected Command implementation"),
        }
    }

    #[test]
    fn test_build_env_vars() {
        use std::collections::HashMap;

        use serde_json::json;

        let skill = SkillTool::new("test".to_string(), "Test".to_string());
        let mut params = HashMap::new();
        params.insert("name".to_string(), json!("Alice"));
        params.insert("age".to_string(), json!(30));
        params.insert("active".to_string(), json!(true));

        let env_vars = skill.build_env_vars(&params);

        assert_eq!(env_vars.get("SKILL_PARAM_name"), Some(&"Alice".to_string()));
        assert_eq!(env_vars.get("SKILL_PARAM_age"), Some(&"30".to_string()));
        assert_eq!(env_vars.get("SKILL_PARAM_active"), Some(&"true".to_string()));
    }

    #[test]
    fn test_execute_simple_script() {
        use std::collections::HashMap;
        use std::fs;

        use serde_json::json;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let script_path = dir.path().join("test.sh");

        #[cfg(unix)]
        let script_content = "#!/bin/bash\necho \"Hello $SKILL_PARAM_name\"";
        #[cfg(windows)]
        let script_content = "@echo off\necho Hello %SKILL_PARAM_name%";

        fs::write(&script_path, script_content).unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&script_path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&script_path, perms).unwrap();
        }

        let definition = SkillDefinition {
            name: "test".to_string(),
            description: "Test".to_string(),
            skill_type: "code_inline".to_string(),
            parameters: None,
            implementation: Some(SkillImplementation::Script {
                path: script_path.to_string_lossy().to_string(),
            }),
        };

        let skill = SkillTool::new("test".to_string(), "Test".to_string());
        let mut params = HashMap::new();
        params.insert("name".to_string(), json!("World"));

        let result = skill.execute_script(&definition, &params);

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Hello World"));
    }

    #[tokio::test]
    async fn test_script_timeout() {
        use std::collections::HashMap;
        use std::fs;

        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let script_path = dir.path().join("slow.sh");

        #[cfg(unix)]
        let script_content = "#!/bin/bash\nsleep 10";
        #[cfg(windows)]
        let script_content = "@echo off\ntimeout /t 10";

        fs::write(&script_path, script_content).unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&script_path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&script_path, perms).unwrap();
        }

        let definition = SkillDefinition {
            name: "slow".to_string(),
            description: "Slow script".to_string(),
            skill_type: "code_inline".to_string(),
            parameters: None,
            implementation: Some(SkillImplementation::Script {
                path: script_path.to_string_lossy().to_string(),
            }),
        };

        let skill = SkillTool::new("slow".to_string(), "Slow".to_string());
        let params = HashMap::new();

        let result = skill.execute_script_with_timeout(&definition, &params, 1).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("timeout"));
    }

    #[test]
    fn test_capture_stderr() {
        use std::collections::HashMap;
        use std::fs;

        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let script_path = dir.path().join("error.sh");

        #[cfg(unix)]
        let script_content = "#!/bin/bash\necho 'Error message' >&2\nexit 1";
        #[cfg(windows)]
        let script_content = "@echo off\necho Error message 1>&2\nexit /b 1";

        fs::write(&script_path, script_content).unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&script_path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&script_path, perms).unwrap();
        }

        let definition = SkillDefinition {
            name: "error".to_string(),
            description: "Error script".to_string(),
            skill_type: "code_inline".to_string(),
            parameters: None,
            implementation: Some(SkillImplementation::Script {
                path: script_path.to_string_lossy().to_string(),
            }),
        };

        let skill = SkillTool::new("error".to_string(), "Error".to_string());
        let params = HashMap::new();

        let result = skill.execute_script(&definition, &params);

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Error message"));
    }

    #[test]
    fn test_nonzero_exit_code() {
        use std::collections::HashMap;
        use std::fs;

        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let script_path = dir.path().join("exit_code.sh");

        #[cfg(unix)]
        let script_content = "#!/bin/bash\nexit 42";
        #[cfg(windows)]
        let script_content = "@echo off\nexit /b 42";

        fs::write(&script_path, script_content).unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&script_path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&script_path, perms).unwrap();
        }

        let definition = SkillDefinition {
            name: "exit_code".to_string(),
            description: "Exit code test".to_string(),
            skill_type: "code_inline".to_string(),
            parameters: None,
            implementation: Some(SkillImplementation::Script {
                path: script_path.to_string_lossy().to_string(),
            }),
        };

        let skill = SkillTool::new("exit_code".to_string(), "Exit".to_string());
        let params = HashMap::new();

        let result = skill.execute_script(&definition, &params);

        assert!(result.is_err());
        // Verify the error indicates script failure
        assert!(result.unwrap_err().to_string().contains("failed"));
    }

    #[test]
    fn test_parse_command_template() {
        use std::collections::HashMap;

        use serde_json::json;

        let skill = SkillTool::new("test".to_string(), "Test".to_string());
        let mut params = HashMap::new();
        params.insert("name".to_string(), json!("Alice"));
        params.insert("age".to_string(), json!(30));

        let template = "echo 'Hello {{name}}, you are {{age}} years old'";
        let result = skill.parse_command_template(template, &params);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "echo 'Hello Alice, you are 30 years old'");
    }

    #[test]
    fn test_execute_command() {
        use std::collections::HashMap;

        use serde_json::json;

        let definition = SkillDefinition {
            name: "echo".to_string(),
            description: "Echo command".to_string(),
            skill_type: "code_inline".to_string(),
            parameters: None,
            implementation: Some(SkillImplementation::Command {
                command: "echo {{message}}".to_string(),
            }),
        };

        let skill = SkillTool::new("echo".to_string(), "Echo".to_string());
        let mut params = HashMap::new();
        params.insert("message".to_string(), json!("Hello World"));

        let result = skill.execute_command(&definition, &params);

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Hello World"));
    }

    #[tokio::test]
    async fn test_command_timeout() {
        use std::collections::HashMap;

        #[cfg(unix)]
        let definition = SkillDefinition {
            name: "slow".to_string(),
            description: "Slow command".to_string(),
            skill_type: "code_inline".to_string(),
            parameters: None,
            implementation: Some(SkillImplementation::Command {
                command: "sleep 10".to_string(),
            }),
        };

        #[cfg(windows)]
        let definition = SkillDefinition {
            name: "slow".to_string(),
            description: "Slow command".to_string(),
            skill_type: "code_inline".to_string(),
            parameters: None,
            implementation: Some(SkillImplementation::Command {
                command: "timeout /t 10".to_string(),
            }),
        };

        let skill = SkillTool::new("slow".to_string(), "Slow".to_string());
        let params = HashMap::new();

        let result = skill.execute_command_with_timeout(&definition, &params, 1).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("timeout"));
    }

    #[test]
    fn test_format_output() {
        let skill = SkillTool::new("test".to_string(), "Test".to_string());

        let stdout = "Command output\nLine 2".to_string();
        let stderr = "Warning message".to_string();

        let result = skill.format_output(stdout, stderr);

        assert!(result.contains("Command output"));
        assert!(result.contains("Line 2"));
        // stderr should be included if present
        assert!(result.contains("Warning message"));
    }

    #[test]
    fn test_truncate_output() {
        let skill = SkillTool::new("test".to_string(), "Test".to_string());

        // Create output larger than max size
        let large_output = "x".repeat(150_000);

        let result = skill.truncate_output(large_output);

        // Should be truncated
        assert!(result.len() < 150_000);
        assert!(result.contains("truncated"));
    }

    #[test]
    fn test_format_error() {
        let skill = SkillTool::new("test".to_string(), "Test".to_string());

        let error = eyre::eyre!("Script execution failed: command not found");

        let result = skill.format_error(&error);

        assert!(result.contains("Error"));
        assert!(result.contains("command not found"));
    }

    #[test]
    fn test_definition_to_toolspec() {
        let skill = SkillTool::new("test-skill".to_string(), "Test skill".to_string());
        let definition = SkillDefinition {
            name: "test-skill".to_string(),
            description: "Test skill".to_string(),
            skill_type: "code_inline".to_string(),
            parameters: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {"type": "string"}
                }
            })),
            implementation: Some(SkillImplementation::Command {
                command: "echo {{name}}".to_string(),
            }),
        };

        let toolspec = skill.definition_to_toolspec(&definition);

        assert_eq!(toolspec.name, "test-skill");
        assert_eq!(toolspec.description, "Test skill");
    }

    #[test]
    fn test_from_definition() {
        let definition = SkillDefinition {
            name: "my-skill".to_string(),
            description: "My skill description".to_string(),
            skill_type: "code_inline".to_string(),
            parameters: None,
            implementation: None,
        };

        let skill = SkillTool::from_definition(&definition);

        assert_eq!(skill.name, "my-skill");
        assert_eq!(skill.description, "My skill description");
    }
}
