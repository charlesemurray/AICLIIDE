//! Terminal-native UI implementation with ANSI colors and efficient interactions

use std::io::{self, Write};

use eyre::Result;

use crate::cli::creation::{CreationError, SemanticColor, TerminalUI};

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
                self.show_message(&format!("{} is required", field), SemanticColor::Error);
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
            },
            None => {
                print!("{} (optional): ", field);
            },
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

        let bar = format!("{}{}", "█".repeat(filled), "░".repeat(empty));

        println!("{} {} {}% ({}/{})", bar, message, percentage, current, total);
    }

    fn show_message(&mut self, message: &str, color: SemanticColor) {
        println!("{}", self.colorize(message, color));
    }

    fn select_option(&mut self, prompt: &str, options: &[(&str, &str)]) -> Result<String> {
        loop {
            println!("{}", self.colorize(prompt, SemanticColor::Info));

            // Display options with colors
            for (i, (key, description)) in options.iter().enumerate() {
                println!(
                    "  {}. {} - {}",
                    self.colorize(&(i + 1).to_string(), SemanticColor::Info),
                    self.colorize(key, SemanticColor::Success),
                    self.colorize(description, SemanticColor::Debug)
                );
            }

            print!("\nChoose (1-{}): ", options.len());
            io::stdout().flush()?;

            let input = self.read_input()?;

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

            self.show_message(
                &format!(
                    "Invalid selection: {}. Please choose 1-{} or enter the key name.",
                    input,
                    options.len()
                ),
                SemanticColor::Error,
            );
        }
    }

    fn select_multiple(&mut self, prompt: &str, options: &[(&str, &str)], allow_other: bool) -> Result<Vec<String>> {
        println!("{}", self.colorize(prompt, SemanticColor::Info));

        // Display options with colors
        for (i, (key, description)) in options.iter().enumerate() {
            println!(
                "  {}. {} - {}",
                self.colorize(&(i + 1).to_string(), SemanticColor::Info),
                self.colorize(key, SemanticColor::Success),
                self.colorize(description, SemanticColor::Debug)
            );
        }

        if allow_other {
            println!(
                "{}",
                self.colorize("  (You can also type custom values)", SemanticColor::Debug)
            );
        }

        print!("\nChoose multiple (comma-separated, e.g., 1,3,5): ");
        io::stdout().flush()?;

        let input = self.read_input()?;
        if input.is_empty() {
            return Ok(Vec::new());
        }

        let mut selections = Vec::new();
        for part in input.split(',') {
            let part = part.trim();

            // Handle numeric selection
            if let Ok(num) = part.parse::<usize>() {
                if num > 0 && num <= options.len() {
                    selections.push(options[num - 1].0.to_string());
                    continue;
                }
            }

            // Handle key selection
            let mut found = false;
            for (key, _) in options {
                if part == *key {
                    selections.push(key.to_string());
                    found = true;
                    break;
                }
            }

            // Handle custom values if allowed
            if !found && allow_other && !part.is_empty() {
                selections.push(part.to_string());
            }
        }

        Ok(selections)
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

    fn select_option(&mut self, prompt: &str, options: &[(&str, &str)]) -> Result<String> {
        self.record_output(format!("SELECT: {}", prompt));
        for (i, (key, desc)) in options.iter().enumerate() {
            self.record_output(format!("  {}. {} - {}", i + 1, key, desc));
        }

        let input = self.next_input();

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
        self.record_output(format!("SELECT_MULTI: {}", prompt));
        for (i, (key, desc)) in options.iter().enumerate() {
            self.record_output(format!("  {}. {} - {}", i + 1, key, desc));
        }

        let input = self.next_input();
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
    fn test_colorize_with_colors() {
        let ui = TerminalUIImpl { use_colors: true };
        let result = ui.colorize("test", SemanticColor::Success);
        assert!(result.contains("\x1b[32m")); // Green
        assert!(result.contains("\x1b[0m")); // Reset
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
