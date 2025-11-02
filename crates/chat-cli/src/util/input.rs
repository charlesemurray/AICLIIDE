//! Input utilities for interactive prompts, building on existing Q CLI patterns

use std::io::{
    self,
    Write,
};

use crossterm::cursor::{
    MoveToColumn,
    MoveUp,
};
use crossterm::execute;
use crossterm::style::{
    Color,
    Print,
    ResetColor,
    SetForegroundColor,
};
use crossterm::terminal::{
    Clear,
    ClearType,
};
use eyre::Result;

/// Get user input with a prompt, following existing Q CLI patterns
pub fn prompt_required(prompt: &str) -> Result<String> {
    print!("{}: ", prompt);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(eyre::eyre!("Input required for: {}", prompt));
    }

    Ok(trimmed.to_string())
}

/// Get optional user input with default value
pub fn prompt_optional(prompt: &str, default: Option<&str>) -> Result<Option<String>> {
    if let Some(def) = default {
        print!("{} [{}]: ", prompt, def);
    } else {
        print!("{}: ", prompt);
    }
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let trimmed = input.trim();
    if trimmed.is_empty() {
        Ok(default.map(|s| s.to_string()))
    } else {
        Ok(Some(trimmed.to_string()))
    }
}

/// Confirm with yes/no, following existing Q CLI patterns
pub fn confirm(message: &str) -> Result<bool> {
    print!("{} (Y/n): ", message);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let trimmed = input.trim().to_lowercase();
    Ok(!matches!(trimmed.as_str(), "n" | "no" | "false" | "0"))
}

/// Select from multiple options with numbered menu
pub fn select_option(prompt: &str, options: &[(&str, &str)]) -> Result<String> {
    println!("{}", prompt);

    // Display options with colors
    for (i, (key, description)) in options.iter().enumerate() {
        execute!(
            io::stdout(),
            SetForegroundColor(Color::Cyan),
            Print(format!("  {}. ", i + 1)),
            SetForegroundColor(Color::White),
            Print(key),
            SetForegroundColor(Color::DarkGrey),
            Print(format!(" - {}", description)),
            ResetColor,
            Print("\n")
        )?;
    }

    print!("\nChoose (1-{}): ", options.len());
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let trimmed = input.trim();

    // Handle numeric selection (1-based)
    if let Ok(num) = trimmed.parse::<usize>() {
        if num > 0 && num <= options.len() {
            return Ok(options[num - 1].0.to_string());
        }
    }

    // Handle key selection
    for (key, _) in options {
        if trimmed == *key {
            return Ok(key.to_string());
        }
    }

    Err(eyre::eyre!(
        "Invalid selection: {}. Please choose 1-{} or enter the key name.",
        trimmed,
        options.len()
    ))
}

/// Select multiple options (comma-separated)
pub fn select_multiple(prompt: &str, options: &[(&str, &str)], allow_other: bool) -> Result<Vec<String>> {
    println!("{}", prompt);

    // Display options with colors
    for (i, (key, description)) in options.iter().enumerate() {
        execute!(
            io::stdout(),
            SetForegroundColor(Color::Cyan),
            Print(format!("  {}. ", i + 1)),
            SetForegroundColor(Color::White),
            Print(key),
            SetForegroundColor(Color::DarkGrey),
            Print(format!(" - {}", description)),
            ResetColor,
            Print("\n")
        )?;
    }

    if allow_other {
        execute!(
            io::stdout(),
            SetForegroundColor(Color::DarkGrey),
            Print("  (You can also type custom values)\n"),
            ResetColor
        )?;
    }

    print!("\nChoose multiple (comma-separated, e.g., 1,3,5): ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    let mut selections = Vec::new();
    for part in trimmed.split(',') {
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

/// Show a colored message (reusing existing Q CLI color patterns)
pub fn show_message(message: &str, color: Color) -> Result<()> {
    execute!(
        io::stdout(),
        SetForegroundColor(color),
        Print(message),
        ResetColor,
        Print("\n")
    )?;
    Ok(())
}

/// Show an info message in cyan (following Q CLI patterns)
pub fn show_info(message: &str) -> Result<()> {
    show_message(message, Color::Cyan)
}

/// Show a success message in green
pub fn show_success(message: &str) -> Result<()> {
    show_message(message, Color::Green)
}

/// Show an error message in red
pub fn show_error(message: &str) -> Result<()> {
    show_message(message, Color::Red)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests can't easily test actual stdin/stdout interaction
    // The main tests will be in the integration layer with MockUI

    #[test]
    fn test_option_parsing_logic() {
        let options = &[
            ("command", "Execute shell commands"),
            ("assistant", "AI conversational helper"),
            ("template", "Text generation"),
        ];

        // Test that we can find options by key
        let found = options.iter().find(|(key, _)| *key == "assistant");
        assert!(found.is_some());
        assert_eq!(found.unwrap().0, "assistant");
    }

    #[test]
    fn test_numeric_selection_logic() {
        let options = &[
            ("command", "Execute shell commands"),
            ("assistant", "AI conversational helper"),
        ];

        // Test 1-based indexing
        let input = "2";
        if let Ok(num) = input.parse::<usize>() {
            if num > 0 && num <= options.len() {
                assert_eq!(options[num - 1].0, "assistant");
            }
        }
    }

    #[test]
    fn test_multiple_selection_parsing() {
        let input = "1,3,custom";
        let parts: Vec<&str> = input.split(',').map(|s| s.trim()).collect();
        assert_eq!(parts, vec!["1", "3", "custom"]);
    }
}
