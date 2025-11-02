use async_trait::async_trait;
use serde_json::{
    Value,
    json,
};

use crate::cli::chat::tools::{
    InputSchema,
    ToolOrigin,
    ToolSpec,
};
use crate::cli::skills::toolspec_conversion::ConversionError;
use crate::cli::skills::{
    Result,
    Skill,
    SkillError,
    SkillResult,
    SkillUI,
    UIElement,
};

pub struct Calculator;

impl Calculator {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }
}

#[async_trait]
impl Skill for Calculator {
    fn name(&self) -> &str {
        "calculator"
    }

    fn description(&self) -> &str {
        "Basic calculator operations (add, subtract, multiply, divide)"
    }

    fn aliases(&self) -> Vec<String> {
        vec!["calc".to_string(), "math".to_string()]
    }

    async fn execute(&self, params: Value) -> Result<SkillResult> {
        let a = params["a"]
            .as_f64()
            .ok_or_else(|| SkillError::InvalidInput("Missing or invalid parameter 'a'".to_string()))?;

        let b = params["b"]
            .as_f64()
            .ok_or_else(|| SkillError::InvalidInput("Missing or invalid parameter 'b'".to_string()))?;

        let operation = params["op"].as_str().unwrap_or("add");

        let result = match operation {
            "add" => a + b,
            "subtract" => a - b,
            "multiply" => a * b,
            "divide" => {
                if b == 0.0 {
                    return Err(SkillError::InvalidInput("Division by zero".to_string()));
                }
                a / b
            },
            _ => return Err(SkillError::InvalidInput(format!("Unknown operation: {}", operation))),
        };

        Ok(SkillResult {
            output: result.to_string(),
            ui_updates: None,
            state_changes: None,
        })
    }

    async fn render_ui(&self) -> Result<SkillUI> {
        Ok(SkillUI {
            elements: vec![
                UIElement::Text("Calculator".to_string()),
                UIElement::Input {
                    id: "a".to_string(),
                    placeholder: "First number".to_string(),
                },
                UIElement::Input {
                    id: "b".to_string(),
                    placeholder: "Second number".to_string(),
                },
                UIElement::Button {
                    id: "add".to_string(),
                    label: "Add".to_string(),
                },
                UIElement::Button {
                    id: "subtract".to_string(),
                    label: "Subtract".to_string(),
                },
            ],
            interactive: true,
        })
    }

    fn supports_interactive(&self) -> bool {
        true
    }

    fn to_toolspec(&self) -> std::result::Result<ToolSpec, ConversionError> {
        Ok(ToolSpec {
            name: self.name().to_string(),
            description: self.description().to_string(),
            input_schema: InputSchema(json!({
                "type": "object",
                "properties": {
                    "a": {
                        "type": "number",
                        "description": "First operand"
                    },
                    "b": {
                        "type": "number",
                        "description": "Second operand"
                    },
                    "op": {
                        "type": "string",
                        "enum": ["add", "subtract", "multiply", "divide"],
                        "description": "Operation to perform"
                    }
                },
                "required": ["a", "b"]
            })),
            tool_origin: ToolOrigin::Skill(self.name().to_string()),
        })
    }
}
