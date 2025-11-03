//! Mock agent for testing natural language skill invocation
//!
//! This mock agent simulates how a real agent would discover and invoke skills
//! based on natural language input. It uses simple pattern matching for testing.

use chat_cli::cli::chat::tools::ToolSpec;
use chat_cli::cli::skills::SkillRegistry;
use serde_json::{json, Value};

/// Response from the mock agent
#[derive(Debug, Clone)]
pub struct AgentResponse {
    /// The tool that was selected
    pub selected_tool: Option<String>,
    /// The parameters extracted from natural language
    pub parameters: Value,
    /// The result of execution (simulated)
    pub result: String,
    /// Whether execution was successful
    pub success: bool,
}

impl AgentResponse {
    /// Check if a specific tool was used
    pub fn used_tool(&self, tool_name: &str) -> bool {
        self.selected_tool.as_deref() == Some(tool_name)
    }

    /// Get the result string
    pub fn result(&self) -> &str {
        &self.result
    }
}

/// Mock agent that can discover and invoke skills
pub struct MockAgent {
    available_tools: Vec<ToolSpec>,
}

impl MockAgent {
    /// Create a mock agent with skills from a registry
    pub fn with_skills(registry: &SkillRegistry) -> Self {
        let tools = registry.get_all_toolspecs();
        Self {
            available_tools: tools,
        }
    }

    /// Process natural language input and return a response
    ///
    /// This uses simple pattern matching to simulate agent behavior:
    /// - "calculate X + Y" -> calculator skill
    /// - "add X and Y" -> calculator skill
    /// - etc.
    pub async fn process_input(&self, input: &str) -> AgentResponse {
        let input_lower = input.to_lowercase();

        // Pattern matching for calculator skill
        if input_lower.contains("calculate") || input_lower.contains("add") || input_lower.contains("multiply") {
            return self.handle_calculator(&input_lower).await;
        }

        // No matching tool found
        AgentResponse {
            selected_tool: None,
            parameters: json!({}),
            result: "I don't know how to help with that.".to_string(),
            success: false,
        }
    }

    /// Handle calculator skill invocation
    async fn handle_calculator(&self, input: &str) -> AgentResponse {
        // Check if calculator skill is available
        if !self.has_tool("calculator") {
            return AgentResponse {
                selected_tool: Some("calculator".to_string()),
                parameters: json!({}),
                result: "Calculator skill not available".to_string(),
                success: false,
            };
        }

        // Extract numbers and operation
        let (a, b, op) = self.extract_calculation(input);

        // Simulate execution
        let result = match op.as_str() {
            "add" => a + b,
            "multiply" => a * b,
            "subtract" => a - b,
            "divide" if b != 0.0 => a / b,
            _ => 0.0,
        };

        AgentResponse {
            selected_tool: Some("calculator".to_string()),
            parameters: json!({
                "a": a,
                "b": b,
                "op": op
            }),
            result: result.to_string(),
            success: true,
        }
    }

    /// Extract calculation from natural language
    fn extract_calculation(&self, input: &str) -> (f64, f64, String) {
        // Simple extraction: look for numbers
        let numbers: Vec<f64> = input
            .split_whitespace()
            .filter_map(|s| s.parse::<f64>().ok())
            .collect();

        let a = numbers.get(0).copied().unwrap_or(0.0);
        let b = numbers.get(1).copied().unwrap_or(0.0);

        // Determine operation
        let op = if input.contains("add") || input.contains('+') {
            "add"
        } else if input.contains("multiply") || input.contains('*') {
            "multiply"
        } else if input.contains("subtract") || input.contains('-') {
            "subtract"
        } else if input.contains("divide") || input.contains('/') {
            "divide"
        } else {
            "add"
        };

        (a, b, op.to_string())
    }

    /// Check if a tool is available
    fn has_tool(&self, tool_name: &str) -> bool {
        self.available_tools.iter().any(|t| t.name == tool_name)
    }

    /// Get all available tool names
    pub fn available_tool_names(&self) -> Vec<String> {
        self.available_tools.iter().map(|t| t.name.clone()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_agent_discovers_skills() {
        let registry = SkillRegistry::with_builtins();
        let agent = MockAgent::with_skills(&registry);

        let tools = agent.available_tool_names();
        assert!(!tools.is_empty(), "Agent should discover skills");
        assert!(tools.contains(&"calculator".to_string()), "Should find calculator");
    }

    #[tokio::test]
    async fn test_mock_agent_selects_calculator() {
        let registry = SkillRegistry::with_builtins();
        let agent = MockAgent::with_skills(&registry);

        let response = agent.process_input("calculate 5 plus 3").await;

        assert!(response.used_tool("calculator"), "Should select calculator");
        assert!(response.success, "Should succeed");
    }

    #[tokio::test]
    async fn test_mock_agent_extracts_parameters() {
        let registry = SkillRegistry::with_builtins();
        let agent = MockAgent::with_skills(&registry);

        let response = agent.process_input("add 10 and 5").await;

        assert_eq!(response.parameters["a"], 10.0);
        assert_eq!(response.parameters["b"], 5.0);
        assert_eq!(response.parameters["op"], "add");
    }

    #[tokio::test]
    async fn test_mock_agent_handles_invalid_input() {
        let registry = SkillRegistry::with_builtins();
        let agent = MockAgent::with_skills(&registry);

        let response = agent.process_input("do something random").await;

        assert!(!response.success, "Should fail for unknown input");
        assert!(response.selected_tool.is_none(), "Should not select a tool");
    }
}
