//! Tests for enhanced UI methods with multiple choice support

use super::ui::*;
use super::types::*;
use eyre::Result;

/// Mock UI for testing that simulates user input
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
        self.outputs.push(message.to_string());
    }

    fn prompt_required(&mut self, field: &str) -> Result<String> {
        self.outputs.push(format!("PROMPT: {}", field));
        self.next_input()
    }

    fn prompt_optional(&mut self, field: &str, default: Option<&str>) -> Result<Option<String>> {
        let prompt = if let Some(def) = default {
            format!("PROMPT: {} [{}]", field, def)
        } else {
            format!("PROMPT: {}", field)
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
        Ok(matches!(input.to_lowercase().as_str(), "y" | "yes" | "true" | "1"))
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
                // Handle key selection
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
    fn test_select_multiple_empty() {
        let mut ui = MockUI::new(vec![""]);
        
        let options = &[
            ("rust", "Systems programming language"),
            ("python", "General purpose language"),
        ];
        
        let result = ui.select_multiple("Choose languages:", options, false).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_confirm_yes_variations() {
        let test_cases = vec!["y", "yes", "Y", "YES", "true", "1"];
        
        for input in test_cases {
            let mut ui = MockUI::new(vec![input]);
            let result = ui.confirm("Continue?").unwrap();
            assert!(result, "Input '{}' should be true", input);
        }
    }

    #[test]
    fn test_confirm_no_variations() {
        let test_cases = vec!["n", "no", "N", "NO", "false", "0", ""];
        
        for input in test_cases {
            let mut ui = MockUI::new(vec![input]);
            let result = ui.confirm("Continue?").unwrap();
            assert!(!result, "Input '{}' should be false", input);
        }
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
    fn test_prompt_optional_override_default() {
        let mut ui = MockUI::new(vec!["Custom description"]);
        
        let result = ui.prompt_optional("Description", Some("Default description")).unwrap();
        assert_eq!(result, Some("Custom description".to_string()));
    }
}
