use std::io::Write;

use eyre::Result;

use super::{InvokeOutput, OutputKind};
use crate::cli::skills::SkillRegistry;

#[derive(Debug, Clone)]
pub struct SkillTool {
    pub skill_name: String,
    pub params: serde_json::Value,
}

impl SkillTool {
    pub fn new(skill_name: String, params: serde_json::Value) -> Self {
        Self { skill_name, params }
    }

    pub async fn invoke(&self, registry: &SkillRegistry, stdout: &mut impl Write) -> Result<InvokeOutput> {
        let skill = registry
            .get(&self.skill_name)
            .ok_or_else(|| eyre::eyre!("Skill not found: {}", self.skill_name))?;

        let result = skill
            .execute(self.params.clone())
            .await
            .map_err(|e| eyre::eyre!("Skill execution failed: {}", e))?;

        writeln!(stdout, "{}", result.output)?;

        Ok(InvokeOutput {
            output: OutputKind::Text(result.output),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::skills::SkillRegistry;

    #[tokio::test]
    async fn test_skill_tool_execution() {
        let registry = SkillRegistry::with_builtins();
        let tool = SkillTool::new(
            "calculator".to_string(),
            serde_json::json!({
                "a": 5.0,
                "b": 3.0,
                "op": "add"
            }),
        );
        let mut output = Vec::new();

        let result = tool.invoke(&registry, &mut output).await;
        assert!(result.is_ok());
        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("8"));
    }

    #[tokio::test]
    async fn test_skill_not_found() {
        let registry = SkillRegistry::new();
        let tool = SkillTool::new("nonexistent".to_string(), serde_json::json!({}));
        let mut output = Vec::new();

        let result = tool.invoke(&registry, &mut output).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Skill not found"));
    }

    #[tokio::test]
    async fn test_skill_with_parameters() {
        let registry = SkillRegistry::with_builtins();
        let tool = SkillTool::new(
            "calculator".to_string(),
            serde_json::json!({
                "a": 10.0,
                "b": 2.0,
                "op": "multiply"
            }),
        );
        let mut output = Vec::new();

        let result = tool.invoke(&registry, &mut output).await;
        assert!(result.is_ok());
        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("20"));
    }

    #[tokio::test]
    async fn test_skill_execution_error() {
        let registry = SkillRegistry::with_builtins();
        let tool = SkillTool::new(
            "calculator".to_string(),
            serde_json::json!({
                "a": 10.0,
                "b": 0.0,
                "op": "divide"
            }),
        );
        let mut output = Vec::new();

        let result = tool.invoke(&registry, &mut output).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Skill execution failed"));
    }
}
