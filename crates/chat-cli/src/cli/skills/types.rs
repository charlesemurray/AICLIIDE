use std::collections::HashMap;
use std::str::FromStr;

use serde::{
    Deserialize,
    Serialize,
};
use serde_json::json;

use crate::cli::chat::tools::{InputSchema, ToolOrigin, ToolSpec};
use crate::cli::skills::toolspec_conversion::{ConversionError, ToToolSpec};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SkillType {
    #[serde(rename = "command")]
    Command,
    #[serde(rename = "code_inline")]
    CodeInline,
    #[serde(rename = "code_session")]
    CodeSession,
    #[serde(rename = "conversation")]
    Conversation,
    #[serde(rename = "prompt_inline")]
    PromptInline,
}

impl FromStr for SkillType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "code_inline" => Ok(SkillType::CodeInline),
            "code_session" => Ok(SkillType::CodeSession),
            "conversation" => Ok(SkillType::Conversation),
            "prompt_inline" => Ok(SkillType::PromptInline),
            _ => Err(format!("Unknown skill type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub permissions: Option<Permissions>,
    pub resource_limits: Option<ResourceLimits>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permissions {
    pub file_read: Option<Vec<String>>,
    pub file_write: Option<Vec<String>>,
    pub network_access: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceLimits {
    pub max_memory_mb: Option<u32>,
    pub max_execution_time: Option<u32>,
    pub max_cpu_percent: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    pub session_timeout: Option<u32>,
    pub max_sessions: Option<u32>,
    pub cleanup_on_exit: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextFiles {
    pub patterns: Vec<String>,
    pub max_files: Option<u32>,
    pub max_file_size_kb: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: String,
    pub values: Option<Vec<String>>,
    pub required: Option<bool>,
    pub pattern: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonSkill {
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub skill_type: SkillType,
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub timeout: Option<u32>,
    pub security: Option<SecurityConfig>,
    pub session_config: Option<SessionConfig>,
    #[serde(alias = "prompt")]
    pub prompt_template: Option<String>,
    pub context_files: Option<ContextFiles>,
    pub parameters: Option<Vec<Parameter>>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl JsonSkill {
    pub fn from_json(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value(value)
    }

    pub async fn execute(&self, params: HashMap<String, String>) -> Result<String, String> {
        match self.skill_type {
            SkillType::Command => self.execute_command(params).await,
            SkillType::CodeInline => self.execute_code_inline(params).await,
            SkillType::CodeSession => self.execute_code_session(params).await,
            SkillType::Conversation => self.execute_conversation(params).await,
            SkillType::PromptInline => self.execute_prompt_inline(params).await,
        }
    }

    async fn execute_command(&self, _params: HashMap<String, String>) -> Result<String, String> {
        let command = self.command.as_ref().ok_or("No command specified")?;
        let empty_args = vec![];
        let args = self.args.as_ref().unwrap_or(&empty_args);

        let output = tokio::process::Command::new(command)
            .args(args)
            .output()
            .await
            .map_err(|e| format!("Failed to execute command: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    async fn execute_code_inline(&self, _params: HashMap<String, String>) -> Result<String, String> {
        let command = self.command.as_ref().ok_or("No command specified")?;
        let empty_args = vec![];
        let args = self.args.as_ref().unwrap_or(&empty_args);

        let output = tokio::process::Command::new(command)
            .args(args)
            .output()
            .await
            .map_err(|e| format!("Failed to execute command: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    async fn execute_code_session(&self, params: HashMap<String, String>) -> Result<String, String> {
        let command = self.command.as_ref().ok_or("No command specified")?;
        let input = params.get("input").unwrap_or(&String::new()).clone();

        let mut child = tokio::process::Command::new(command)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start session: {}", e))?;

        if let Some(stdin) = child.stdin.as_mut() {
            use tokio::io::AsyncWriteExt;
            stdin
                .write_all(input.as_bytes())
                .await
                .map_err(|e| format!("Failed to write to session: {}", e))?;
            stdin
                .write_all(b"\n")
                .await
                .map_err(|e| format!("Failed to write newline: {}", e))?;
        }

        let output = child
            .wait_with_output()
            .await
            .map_err(|e| format!("Session execution failed: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    async fn execute_conversation(&self, params: HashMap<String, String>) -> Result<String, String> {
        let template = self.prompt_template.as_ref().ok_or("No prompt template specified")?;
        let input = params.get("input").unwrap_or(&String::new()).clone();

        // Simple template substitution
        let prompt = template.replace("{input}", &input);

        // For now, return the formatted prompt (in real implementation, this would call AI)
        Ok(format!("AI Response to: {}", prompt))
    }

    async fn execute_prompt_inline(&self, params: HashMap<String, String>) -> Result<String, String> {
        let prompt = self.prompt_template.as_ref().ok_or("No prompt specified")?;

        let mut result = prompt.to_string();

        // Replace parameters in the prompt
        for (key, value) in params {
            let placeholder = format!("{{{}}}", key);
            result = result.replace(&placeholder, &value);
        }

        Ok(result)
    }
}

// Add security_config accessor for backward compatibility
impl JsonSkill {
    pub fn security_config(&self) -> Option<&SecurityConfig> {
        self.security.as_ref()
    }
}

impl ToToolSpec for JsonSkill {
    fn to_toolspec(&self) -> Result<ToolSpec, ConversionError> {
        let input_schema = self.build_input_schema()?;
        
        Ok(ToolSpec {
            name: self.name.clone(),
            description: self.description.clone()
                .unwrap_or_else(|| format!("Execute {} skill", self.name)),
            input_schema: InputSchema(input_schema),
            tool_origin: ToolOrigin::Skill(self.name.clone()),
        })
    }
}

impl JsonSkill {
    fn build_input_schema(&self) -> Result<serde_json::Value, ConversionError> {
        let mut properties = serde_json::Map::new();
        let mut required = Vec::new();
        
        if let Some(params) = &self.parameters {
            for param in params {
                properties.insert(
                    param.name.clone(),
                    self.param_to_schema(param)
                );
                if param.required.unwrap_or(false) {
                    required.push(param.name.clone());
                }
            }
        }
        
        Ok(json!({
            "type": "object",
            "properties": properties,
            "required": required
        }))
    }
    
    fn param_to_schema(&self, param: &Parameter) -> serde_json::Value {
        let mut schema = json!({
            "type": param.param_type.clone()
        });
        
        if let Some(values) = &param.values {
            schema["enum"] = json!(values);
        }
        
        if let Some(pattern) = &param.pattern {
            schema["pattern"] = json!(pattern);
        }
        
        schema
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_skill_to_toolspec() {
        let skill = JsonSkill {
            name: "test-skill".to_string(),
            description: Some("Test description".to_string()),
            skill_type: SkillType::Command,
            command: Some("echo".to_string()),
            args: None,
            timeout: None,
            security: None,
            session_config: None,
            prompt_template: None,
            context_files: None,
            parameters: None,
            extra: HashMap::new(),
        };
        
        let toolspec = skill.to_toolspec().unwrap();
        assert_eq!(toolspec.name, "test-skill");
        assert_eq!(toolspec.description, "Test description");
        assert!(matches!(toolspec.tool_origin, ToolOrigin::Skill(_)));
    }
    
    #[test]
    fn test_skill_with_parameters() {
        let skill = JsonSkill {
            name: "param-skill".to_string(),
            description: Some("Skill with params".to_string()),
            skill_type: SkillType::Command,
            command: Some("echo".to_string()),
            args: None,
            timeout: None,
            security: None,
            session_config: None,
            prompt_template: None,
            context_files: None,
            parameters: Some(vec![
                Parameter {
                    name: "input".to_string(),
                    param_type: "string".to_string(),
                    required: Some(true),
                    values: None,
                    pattern: None,
                }
            ]),
            extra: HashMap::new(),
        };
        
        let toolspec = skill.to_toolspec().unwrap();
        let schema = toolspec.input_schema.0;
        assert!(schema["required"].as_array().unwrap().contains(&json!("input")));
        assert_eq!(schema["properties"]["input"]["type"], "string");
    }
    
    #[test]
    fn test_skill_with_enum_values() {
        let skill = JsonSkill {
            name: "enum-skill".to_string(),
            description: Some("Skill with enum".to_string()),
            skill_type: SkillType::Command,
            command: Some("echo".to_string()),
            args: None,
            timeout: None,
            security: None,
            session_config: None,
            prompt_template: None,
            context_files: None,
            parameters: Some(vec![
                Parameter {
                    name: "option".to_string(),
                    param_type: "string".to_string(),
                    required: Some(false),
                    values: Some(vec!["a".to_string(), "b".to_string()]),
                    pattern: None,
                }
            ]),
            extra: HashMap::new(),
        };
        
        let toolspec = skill.to_toolspec().unwrap();
        let schema = toolspec.input_schema.0;
        let enum_values = schema["properties"]["option"]["enum"].as_array().unwrap();
        assert_eq!(enum_values.len(), 2);
        assert!(enum_values.contains(&json!("a")));
    }
    
    #[test]
    fn test_skill_with_pattern() {
        let skill = JsonSkill {
            name: "pattern-skill".to_string(),
            description: Some("Skill with pattern".to_string()),
            skill_type: SkillType::Command,
            command: Some("echo".to_string()),
            args: None,
            timeout: None,
            security: None,
            session_config: None,
            prompt_template: None,
            context_files: None,
            parameters: Some(vec![
                Parameter {
                    name: "email".to_string(),
                    param_type: "string".to_string(),
                    required: Some(true),
                    values: None,
                    pattern: Some("^[a-z]+@[a-z]+\\.[a-z]+$".to_string()),
                }
            ]),
            extra: HashMap::new(),
        };
        
        let toolspec = skill.to_toolspec().unwrap();
        let schema = toolspec.input_schema.0;
        assert_eq!(schema["properties"]["email"]["pattern"], "^[a-z]+@[a-z]+\\.[a-z]+$");
    }
    
    #[test]
    fn test_skill_without_description() {
        let skill = JsonSkill {
            name: "no-desc".to_string(),
            description: None,
            skill_type: SkillType::Command,
            command: Some("echo".to_string()),
            args: None,
            timeout: None,
            security: None,
            session_config: None,
            prompt_template: None,
            context_files: None,
            parameters: None,
            extra: HashMap::new(),
        };
        
        let toolspec = skill.to_toolspec().unwrap();
        assert_eq!(toolspec.description, "Execute no-desc skill");
    }
}
