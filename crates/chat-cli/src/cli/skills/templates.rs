//! Skill templates for quick creation

use serde_json::json;

/// Available skill templates
#[derive(Debug, Clone)]
pub enum SkillTemplate {
    Command,
    Script,
    HttpApi,
    FileProcessor,
}

impl SkillTemplate {
    /// Get all available templates
    pub fn all() -> Vec<SkillTemplate> {
        vec![
            SkillTemplate::Command,
            SkillTemplate::Script,
            SkillTemplate::HttpApi,
            SkillTemplate::FileProcessor,
        ]
    }

    /// Get template name
    pub fn name(&self) -> &str {
        match self {
            SkillTemplate::Command => "command",
            SkillTemplate::Script => "script",
            SkillTemplate::HttpApi => "http-api",
            SkillTemplate::FileProcessor => "file-processor",
        }
    }

    /// Get template description
    pub fn description(&self) -> &str {
        match self {
            SkillTemplate::Command => "Run a simple command",
            SkillTemplate::Script => "Execute a shell script",
            SkillTemplate::HttpApi => "Call an HTTP API",
            SkillTemplate::FileProcessor => "Process files",
        }
    }

    /// Generate skill JSON from template
    pub fn generate(&self, name: &str, description: &str) -> serde_json::Value {
        match self {
            SkillTemplate::Command => json!({
                "name": name,
                "description": description,
                "skill_type": "code_inline",
                "parameters": [
                    {
                        "name": "input",
                        "type": "string",
                        "required": true,
                        "description": "Input to process"
                    }
                ],
                "implementation": {
                    "type": "command",
                    "command": "echo {{input}}"
                }
            }),
            SkillTemplate::Script => json!({
                "name": name,
                "description": description,
                "skill_type": "code_inline",
                "parameters": [
                    {
                        "name": "args",
                        "type": "string",
                        "required": false,
                        "description": "Script arguments"
                    }
                ],
                "implementation": {
                    "type": "command",
                    "command": "./scripts/{{name}}.sh {{args}}"
                }
            }),
            SkillTemplate::HttpApi => json!({
                "name": name,
                "description": description,
                "skill_type": "code_inline",
                "parameters": [
                    {
                        "name": "endpoint",
                        "type": "string",
                        "required": true,
                        "description": "API endpoint"
                    },
                    {
                        "name": "method",
                        "type": "string",
                        "required": false,
                        "description": "HTTP method (GET, POST, etc.)"
                    }
                ],
                "implementation": {
                    "type": "command",
                    "command": "curl -X {{method}} {{endpoint}}"
                }
            }),
            SkillTemplate::FileProcessor => json!({
                "name": name,
                "description": description,
                "skill_type": "code_inline",
                "parameters": [
                    {
                        "name": "file",
                        "type": "string",
                        "required": true,
                        "description": "File to process"
                    }
                ],
                "implementation": {
                    "type": "command",
                    "command": "cat {{file}} | process"
                }
            }),
        }
    }

    /// Get usage example
    pub fn example(&self, name: &str) -> String {
        match self {
            SkillTemplate::Command => {
                format!("q chat \"use {} to process 'hello world'\"", name)
            },
            SkillTemplate::Script => {
                format!("q chat \"use {} with arguments '--verbose'\"", name)
            },
            SkillTemplate::HttpApi => {
                format!("q chat \"use {} to call https://api.example.com/data\"", name)
            },
            SkillTemplate::FileProcessor => {
                format!("q chat \"use {} to process data.txt\"", name)
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_templates() {
        let templates = SkillTemplate::all();
        assert_eq!(templates.len(), 4);
    }

    #[test]
    fn test_template_names() {
        assert_eq!(SkillTemplate::Command.name(), "command");
        assert_eq!(SkillTemplate::Script.name(), "script");
        assert_eq!(SkillTemplate::HttpApi.name(), "http-api");
        assert_eq!(SkillTemplate::FileProcessor.name(), "file-processor");
    }

    #[test]
    fn test_template_generation() {
        let template = SkillTemplate::Command;
        let json = template.generate("test", "Test skill");

        assert_eq!(json["name"], "test");
        assert_eq!(json["description"], "Test skill");
        assert!(json["parameters"].is_array());
        assert!(json["implementation"].is_object());
    }

    #[test]
    fn test_all_templates_generate() {
        for template in SkillTemplate::all() {
            let json = template.generate("test", "Test");
            assert!(json["name"].is_string());
            assert!(json["implementation"].is_object());
        }
    }

    #[test]
    fn test_template_examples() {
        let template = SkillTemplate::Command;
        let example = template.example("my-skill");
        assert!(example.contains("my-skill"));
        assert!(example.contains("q chat"));
    }
}
