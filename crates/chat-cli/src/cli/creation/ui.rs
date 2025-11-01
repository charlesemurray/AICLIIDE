//! Terminal-native UI implementation with ANSI colors and efficient interactions

use crate::cli::creation::{TerminalUI, SemanticColor, CreationError};
use eyre::Result;
use std::io::{self, Write};

/// Terminal UI implementation following Q CLI design principles
pub struct TerminalUIImpl {
    use_colors: bool,
}

impl TerminalUIImpl {
    pub fn new() -> Self {
        Self {
            use_colors: std::env::var("NO_COLOR").is_err(),
        }
    }

    fn colorize(&self, text: &str, color: SemanticColor) -> String {
        if !self.use_colors {
            return text.to_string();
        }

        let color_code = match color {
            SemanticColor::Success => "\x1b[32m", // Green
            SemanticColor::Error => "\x1b[31m",   // Red
            SemanticColor::Warning => "\x1b[33m", // Yellow
            SemanticColor::Info => "\x1b[34m",    // Blue
            SemanticColor::Debug => "\x1b[36m",   // Cyan
        };

        format!("{}{}\x1b[0m", color_code, text)
    }

    fn print_prompt(&self, prompt: &str) {
        print!("{}: ", prompt);
        io::stdout().flush().unwrap();
    }

    fn read_input(&self) -> Result<String> {
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        Ok(input.trim().to_string())
    }

    fn validate_name(&self, name: &str) -> Result<()> {
        if name.is_empty() {
            return Err(CreationError::missing_required_field("name", "my-skill").into());
        }

        if !name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Err(CreationError::invalid_name("item", name).into());
        }

        Ok(())
    }
}

impl TerminalUI for TerminalUIImpl {
    fn prompt_required(&mut self, field: &str) -> Result<String> {
        loop {
            self.print_prompt(field);
            let input = self.read_input()?;
            
            if input.is_empty() {
                self.show_message(
                    &format!("{} is required", field),
                    SemanticColor::Error
                );
                continue;
            }

            // Validate name fields
            if field.to_lowercase().contains("name") {
                if let Err(e) = self.validate_name(&input) {
                    self.show_message(&e.to_string(), SemanticColor::Error);
                    continue;
                }
            }

            return Ok(input);
        }
    }

    fn prompt_optional(&mut self, field: &str, default: Option<&str>) -> Result<Option<String>> {
        match default {
            Some(def) => {
                print!("{} [{}]: ", field, def);
            }
            None => {
                print!("{} (optional): ", field);
            }
        }
        io::stdout().flush()?;

        let input = self.read_input()?;
        
        if input.is_empty() {
            Ok(default.map(|s| s.to_string()))
        } else {
            Ok(Some(input))
        }
    }

    fn confirm(&mut self, message: &str) -> Result<bool> {
        print!("{}? [Y/n]: ", message);
        io::stdout().flush()?;

        let input = self.read_input()?;
        Ok(input.is_empty() || input.to_lowercase().starts_with('y'))
    }

    fn show_preview(&mut self, content: &str) {
        println!("\n{}", self.colorize("Preview:", SemanticColor::Info));
        for line in content.lines() {
            println!("  {}", line);
        }
        println!();
    }

    fn show_progress(&mut self, current: usize, total: usize, message: &str) {
        let percentage = (current * 100) / total;
        let filled = (current * 8) / total;
        let empty = 8 - filled;
        
        let bar = format!(
            "{}{}",
            "█".repeat(filled),
            "░".repeat(empty)
        );
        
        println!(
            "{} {} {}% ({}/{})",
            bar,
            message,
            percentage,
            current,
            total
        );
    }

    fn show_message(&mut self, message: &str, color: SemanticColor) {
        println!("{}", self.colorize(message, color));
    }
}

/// Mock UI for testing
#[cfg(test)]
pub struct MockTerminalUI {
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    input_index: usize,
}

#[cfg(test)]
impl MockTerminalUI {
    pub fn new(inputs: Vec<String>) -> Self {
        Self {
            inputs,
            outputs: Vec::new(),
            input_index: 0,
        }
    }

    fn next_input(&mut self) -> String {
        if self.input_index < self.inputs.len() {
            let input = self.inputs[self.input_index].clone();
            self.input_index += 1;
            input
        } else {
            String::new()
        }
    }

    fn record_output(&mut self, output: String) {
        self.outputs.push(output);
    }
}

#[cfg(test)]
impl TerminalUI for MockTerminalUI {
    fn prompt_required(&mut self, field: &str) -> Result<String> {
        self.record_output(format!("{}: ", field));
        let input = self.next_input();
        
        if input.is_empty() {
            return Err(CreationError::missing_required_field(field, "example").into());
        }
        
        Ok(input)
    }

    fn prompt_optional(&mut self, field: &str, default: Option<&str>) -> Result<Option<String>> {
        match default {
            Some(def) => self.record_output(format!("{} [{}]: ", field, def)),
            None => self.record_output(format!("{} (optional): ", field)),
        }
        
        let input = self.next_input();
        
        if input.is_empty() {
            Ok(default.map(|s| s.to_string()))
        } else {
            Ok(Some(input))
        }
    }

    fn confirm(&mut self, message: &str) -> Result<bool> {
        self.record_output(format!("{}? [Y/n]: ", message));
        let input = self.next_input();
        Ok(input.is_empty() || input.to_lowercase().starts_with('y'))
    }

    fn show_preview(&mut self, content: &str) {
        self.record_output(format!("Preview:\n{}", content));
    }

    fn show_progress(&mut self, current: usize, total: usize, message: &str) {
        let percentage = (current * 100) / total;
        let filled = (current * 8) / total;
        let empty = 8 - filled;
        
        let bar = format!("{}{}", "█".repeat(filled), "░".repeat(empty));
        self.record_output(format!("{} {} {}% ({}/{})", bar, message, percentage, current, total));
    }

    fn show_message(&mut self, message: &str, color: SemanticColor) {
        // Record without ANSI codes for testing
        self.record_output(format!("{:?}: {}", color, message));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_colorize_with_colors() {
        let ui = TerminalUIImpl { use_colors: true };
        let result = ui.colorize("test", SemanticColor::Success);
        assert!(result.contains("\x1b[32m")); // Green
        assert!(result.contains("\x1b[0m"));  // Reset
    }

    #[test]
    fn test_colorize_without_colors() {
        let ui = TerminalUIImpl { use_colors: false };
        let result = ui.colorize("test", SemanticColor::Success);
        assert_eq!(result, "test");
    }

    #[test]
    fn test_validate_name_valid() {
        let ui = TerminalUIImpl::new();
        assert!(ui.validate_name("valid-name").is_ok());
        assert!(ui.validate_name("valid_name").is_ok());
        assert!(ui.validate_name("validname123").is_ok());
    }

    #[test]
    fn test_validate_name_invalid() {
        let ui = TerminalUIImpl::new();
        assert!(ui.validate_name("").is_err());
        assert!(ui.validate_name("invalid name").is_err());
        assert!(ui.validate_name("invalid@name").is_err());
    }

    #[test]
    fn test_mock_ui_prompt_required() {
        let mut ui = MockTerminalUI::new(vec!["test-input".to_string()]);
        let result = ui.prompt_required("Name").unwrap();
        assert_eq!(result, "test-input");
        assert!(ui.outputs.iter().any(|o| o.contains("Name:")));
    }

    #[test]
    fn test_mock_ui_confirm() {
        let mut ui = MockTerminalUI::new(vec!["y".to_string()]);
        let result = ui.confirm("Create").unwrap();
        assert!(result);

        let mut ui = MockTerminalUI::new(vec!["n".to_string()]);
        let result = ui.confirm("Create").unwrap();
        assert!(!result);
    }
}
