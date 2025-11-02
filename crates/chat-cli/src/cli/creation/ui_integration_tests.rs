//! Integration tests for enhanced creation UI using existing Q CLI patterns

use std::io::{self, Write};

use crossterm::style::Color;
use eyre::Result;

use super::types::*;
use crate::util::input;

/// Enhanced TerminalUI implementation that builds on existing Q CLI input utilities
pub struct EnhancedTerminalUI;

impl EnhancedTerminalUI {
    pub fn new() -> Self {
        Self
    }
}

impl TerminalUI for EnhancedTerminalUI {
    fn show_message(&mut self, message: &str, color: SemanticColor) {
        let crossterm_color = match color {
            SemanticColor::Info => Color::Cyan,
            SemanticColor::Success => Color::Green,
            SemanticColor::Warning => Color::Yellow,
            SemanticColor::Error => Color::Red,
            SemanticColor::Debug => Color::DarkGrey,
        };
        let _ = input::show_message(message, crossterm_color);
    }

    fn show_preview(&mut self, content: &str) {
        println!("Preview:\n{}", content);
    }

    fn show_progress(&mut self, current: usize, total: usize, message: &str) {
        println!("Progress: {}/{} - {}", current, total, message);
    }

    fn prompt_required(&mut self, field: &str) -> Result<String> {
        input::prompt_required(field)
    }

    fn prompt_optional(&mut self, field: &str, default: Option<&str>) -> Result<Option<String>> {
        input::prompt_optional(field, default)
    }

    fn confirm(&mut self, message: &str) -> Result<bool> {
        input::confirm(message)
    }

    fn select_option(&mut self, prompt: &str, options: &[(&str, &str)]) -> Result<String> {
        input::select_option(prompt, options)
    }

    fn select_multiple(&mut self, prompt: &str, options: &[(&str, &str)], allow_other: bool) -> Result<Vec<String>> {
        input::select_multiple(prompt, options, allow_other)
    }
}

/// Mock UI for testing that simulates user input (reusing existing test patterns)
pub struct MockUI {
    inputs: Vec<String>,
    current_input: usize,
    outputs: Vec<String>,
}

impl MockUI {
    pub fn new(inputs: Vec<&str>) -> Self {
        Self {
            inputs: inputs.into_iter().map(|s| s.to_string()).collect(),
            current_input: 0,
            outputs: Vec::new(),
        }
    }

    pub fn get_outputs(&self) -> &[String] {
        &self.outputs
    }

    fn next_input(&mut self) -> Result<String> {
        if self.current_input >= self.inputs.len() {
            return Err(eyre::eyre!("No more mock inputs available"));
        }
        let input = self.inputs[self.current_input].clone();
        self.current_input += 1;
        Ok(input)
    }
}

impl TerminalUI for MockUI {
    fn show_message(&mut self, message: &str, _color: SemanticColor) {
        self.outputs.push(format!("MSG: {}", message));
    }

    fn show_preview(&mut self, content: &str) {
        self.outputs.push(format!("PREVIEW: {}", content));
    }

    fn show_progress(&mut self, current: usize, total: usize, message: &str) {
        self.outputs
            .push(format!("PROGRESS: {}/{} - {}", current, total, message));
    }

    fn prompt_required(&mut self, field: &str) -> Result<String> {
        self.outputs.push(format!("PROMPT_REQ: {}", field));
        let input = self.next_input()?;
        if input.is_empty() {
            return Err(eyre::eyre!("Input required for: {}", field));
        }
        Ok(input)
    }

    fn prompt_optional(&mut self, field: &str, default: Option<&str>) -> Result<Option<String>> {
        let prompt = if let Some(def) = default {
            format!("PROMPT_OPT: {} [{}]", field, def)
        } else {
            format!("PROMPT_OPT: {}", field)
        };
        self.outputs.push(prompt);

        let input = self.next_input()?;
        if input.is_empty() {
            Ok(default.map(|s| s.to_string()))
        } else {
            Ok(Some(input))
        }
    }

    fn confirm(&mut self, message: &str) -> Result<bool> {
        self.outputs.push(format!("CONFIRM: {}", message));
        let input = self.next_input()?;
        Ok(!matches!(input.to_lowercase().as_str(), "n" | "no" | "false" | "0"))
    }

    fn select_option(&mut self, prompt: &str, options: &[(&str, &str)]) -> Result<String> {
        self.outputs.push(format!("SELECT: {}", prompt));
        for (i, (key, desc)) in options.iter().enumerate() {
            self.outputs.push(format!("  {}. {} - {}", i + 1, key, desc));
        }

        let input = self.next_input()?;

        // Handle numeric selection (1-based)
        if let Ok(num) = input.parse::<usize>() {
            if num > 0 && num <= options.len() {
                return Ok(options[num - 1].0.to_string());
            }
        }

        // Handle key selection
        for (key, _) in options {
            if input == *key {
                return Ok(key.to_string());
            }
        }

        Err(eyre::eyre!("Invalid selection: {}", input))
    }

    fn select_multiple(&mut self, prompt: &str, options: &[(&str, &str)], _allow_other: bool) -> Result<Vec<String>> {
        self.outputs.push(format!("SELECT_MULTI: {}", prompt));
        for (i, (key, desc)) in options.iter().enumerate() {
            self.outputs.push(format!("  {}. {} - {}", i + 1, key, desc));
        }

        let input = self.next_input()?;
        if input.is_empty() {
            return Ok(Vec::new());
        }

        let selections: Vec<String> = input
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| {
                // Handle numeric selection
                if let Ok(num) = s.parse::<usize>() {
                    if num > 0 && num <= options.len() {
                        return options[num - 1].0.to_string();
                    }
                }
                // Handle key selection or custom value
                s.to_string()
            })
            .collect();

        Ok(selections)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_option_numeric() {
        let mut ui = MockUI::new(vec!["2"]);

        let options = &[
            ("command", "Execute shell commands"),
            ("assistant", "AI conversational helper"),
            ("template", "Text generation with variables"),
        ];

        let result = ui.select_option("Choose skill type:", options).unwrap();
        assert_eq!(result, "assistant");

        let outputs = ui.get_outputs();
        assert!(outputs[0].contains("Choose skill type:"));
        assert!(outputs[1].contains("1. command - Execute shell commands"));
        assert!(outputs[2].contains("2. assistant - AI conversational helper"));
    }

    #[test]
    fn test_select_option_by_key() {
        let mut ui = MockUI::new(vec!["template"]);

        let options = &[
            ("command", "Execute shell commands"),
            ("assistant", "AI conversational helper"),
            ("template", "Text generation with variables"),
        ];

        let result = ui.select_option("Choose skill type:", options).unwrap();
        assert_eq!(result, "template");
    }

    #[test]
    fn test_select_option_invalid() {
        let mut ui = MockUI::new(vec!["invalid"]);

        let options = &[
            ("command", "Execute shell commands"),
            ("assistant", "AI conversational helper"),
        ];

        let result = ui.select_option("Choose skill type:", options);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid selection: invalid"));
    }

    #[test]
    fn test_select_multiple() {
        let mut ui = MockUI::new(vec!["1,3,python"]);

        let options = &[
            ("rust", "Systems programming language"),
            ("javascript", "Web development language"),
            ("python", "General purpose language"),
            ("go", "Cloud native language"),
        ];

        let result = ui.select_multiple("Choose languages:", options, true).unwrap();
        assert_eq!(result, vec!["rust", "python", "python"]);
    }

    #[test]
    fn test_confirm_variations() {
        let test_cases = vec![
            ("y", true),
            ("yes", true),
            ("Y", true),
            ("", true), // Default to yes following Q CLI pattern
            ("n", false),
            ("no", false),
            ("false", false),
        ];

        for (input, expected) in test_cases {
            let mut ui = MockUI::new(vec![input]);
            let result = ui.confirm("Continue?").unwrap();
            assert_eq!(result, expected, "Input '{}' should be {}", input, expected);
        }
    }

    #[test]
    fn test_prompt_required_empty_fails() {
        let mut ui = MockUI::new(vec![""]);

        let result = ui.prompt_required("Name");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Input required"));
    }

    #[test]
    fn test_prompt_optional_with_default() {
        let mut ui = MockUI::new(vec![""]);

        let result = ui.prompt_optional("Description", Some("Default description")).unwrap();
        assert_eq!(result, Some("Default description".to_string()));

        let outputs = ui.get_outputs();
        assert!(outputs[0].contains("Description [Default description]"));
    }

    #[test]
    fn test_skill_creation_workflow() {
        // Test a complete skill creation workflow
        let mut ui = MockUI::new(vec![
            "2",             // Select "assistant" skill type
            "Code reviewer", // Description
            "y",             // Confirm creation
        ]);

        // Simulate skill type selection
        let skill_options = &[
            ("command", "Execute shell commands"),
            ("assistant", "AI conversational helper"),
            ("template", "Text generation"),
        ];

        let skill_type = ui.select_option("What type of skill?", skill_options).unwrap();
        assert_eq!(skill_type, "assistant");

        // Simulate description prompt
        let description = ui.prompt_optional("Description", Some("AI assistant")).unwrap();
        assert_eq!(description, Some("Code reviewer".to_string()));

        // Simulate confirmation
        let confirmed = ui.confirm("Create this skill?").unwrap();
        assert!(confirmed);

        // Verify the interaction flow
        let outputs = ui.get_outputs();
        assert!(outputs.iter().any(|o| o.contains("What type of skill?")));
        assert!(
            outputs
                .iter()
                .any(|o| o.contains("assistant - AI conversational helper"))
        );
        assert!(outputs.iter().any(|o| o.contains("Description [AI assistant]")));
        assert!(outputs.iter().any(|o| o.contains("Create this skill?")));
    }
}
