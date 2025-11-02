use std::collections::HashMap;

use eyre::Result;
use serde_json::Value;

pub struct SimpleTemplateLoader {
    templates: HashMap<String, String>,
}

impl SimpleTemplateLoader {
    pub fn new() -> Self {
        let mut loader = Self {
            templates: HashMap::new(),
        };
        loader.load_default_templates();
        loader
    }

    fn load_default_templates(&mut self) {
        // Basic skill template
        self.templates.insert(
            "skill_basic".to_string(),
            r#"{
  "name": "{{name}}",
  "description": "{{description}}",
  "type": "command",
  "command": "{{command}}",
  "args": [],
  "timeout": 30,
  "working_directory": ".",
  "environment": {}
}"#
            .to_string(),
        );

        // Basic command template
        self.templates.insert(
            "command_basic".to_string(),
            r#"{
  "name": "{{name}}",
  "description": "{{description}}",
  "command": "{{command}}",
  "args": {{args}},
  "aliases": []
}"#
            .to_string(),
        );

        // Basic agent template
        self.templates.insert(
            "agent_basic".to_string(),
            r#"{
  "name": "{{name}}",
  "description": "{{description}}",
  "role": "{{role}}",
  "capabilities": [{{capabilities}}],
  "constraints": []
}"#
            .to_string(),
        );
    }

    pub fn get_template(&self, id: &str) -> Option<&String> {
        self.templates.get(id)
    }

    pub fn list_templates(&self) -> Vec<String> {
        self.templates.keys().cloned().collect()
    }

    pub fn render_template(&self, template_id: &str, params: &HashMap<String, String>) -> Result<String> {
        let template = self
            .templates
            .get(template_id)
            .ok_or_else(|| eyre::eyre!("Template not found: {}", template_id))?;

        let mut result = template.clone();
        for (key, value) in params {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_loading() {
        let loader = SimpleTemplateLoader::new();
        assert!(loader.get_template("skill_basic").is_some());
        assert!(loader.get_template("command_basic").is_some());
        assert!(loader.get_template("agent_basic").is_some());
    }

    #[test]
    fn test_template_rendering() {
        let loader = SimpleTemplateLoader::new();
        let mut params = HashMap::new();
        params.insert("name".to_string(), "test_skill".to_string());
        params.insert("description".to_string(), "A test skill".to_string());
        params.insert("command".to_string(), "echo hello".to_string());

        let result = loader.render_template("skill_basic", &params).unwrap();
        assert!(result.contains("test_skill"));
        assert!(result.contains("A test skill"));
        assert!(result.contains("echo hello"));
    }
}
