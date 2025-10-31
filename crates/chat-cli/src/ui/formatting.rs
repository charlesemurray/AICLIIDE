use crate::ui::colors::{SemanticColor, StyledText, TextStyle};

/// ANSI color codes for terminal output
pub struct AnsiCodes;

impl AnsiCodes {
    // Color codes
    pub const RESET: &'static str = "\x1b[0m";
    pub const BLUE: &'static str = "\x1b[34m";
    pub const BRIGHT_BLUE: &'static str = "\x1b[94m";
    pub const GREEN: &'static str = "\x1b[32m";
    pub const BRIGHT_GREEN: &'static str = "\x1b[92m";
    pub const YELLOW: &'static str = "\x1b[33m";
    pub const BRIGHT_YELLOW: &'static str = "\x1b[93m";
    pub const RED: &'static str = "\x1b[31m";
    pub const BRIGHT_RED: &'static str = "\x1b[91m";
    pub const MAGENTA: &'static str = "\x1b[35m";
    pub const BRIGHT_MAGENTA: &'static str = "\x1b[95m";
    pub const CYAN: &'static str = "\x1b[36m";
    pub const BRIGHT_CYAN: &'static str = "\x1b[96m";
    pub const WHITE: &'static str = "\x1b[37m";
    pub const BRIGHT_WHITE: &'static str = "\x1b[97m";
    pub const GRAY: &'static str = "\x1b[90m";

    // Style codes
    pub const BOLD: &'static str = "\x1b[1m";
    pub const DIM: &'static str = "\x1b[2m";
    pub const ITALIC: &'static str = "\x1b[3m";
}

/// Formats styled text with ANSI color codes
pub fn format_styled_text(styled: &StyledText, use_colors: bool) -> String {
    if !use_colors {
        return styled.text.clone();
    }

    let mut result = String::new();

    // Add style codes
    match styled.style {
        TextStyle::Bold => result.push_str(AnsiCodes::BOLD),
        TextStyle::Dim => result.push_str(AnsiCodes::DIM),
        TextStyle::Italic => result.push_str(AnsiCodes::ITALIC),
        TextStyle::Normal => {}
    }

    // Add color codes
    if let Some(color) = styled.color {
        let color_code = match color {
            SemanticColor::Debug => AnsiCodes::BRIGHT_BLUE,
            SemanticColor::Success => AnsiCodes::BRIGHT_GREEN,
            SemanticColor::Warning => AnsiCodes::BRIGHT_YELLOW,
            SemanticColor::Error => AnsiCodes::BRIGHT_RED,
            SemanticColor::Development => AnsiCodes::BRIGHT_MAGENTA,
            SemanticColor::Data => AnsiCodes::BRIGHT_CYAN,
            SemanticColor::Network => AnsiCodes::MAGENTA,
            SemanticColor::Primary => AnsiCodes::BRIGHT_WHITE,
            SemanticColor::Secondary => AnsiCodes::GRAY,
        };
        result.push_str(color_code);
    }

    // Add the text
    result.push_str(&styled.text);

    // Add reset code if we added any formatting
    if styled.color.is_some() || styled.style != TextStyle::Normal {
        result.push_str(AnsiCodes::RESET);
    }

    result
}

/// Check if the current terminal supports colors
pub fn supports_color() -> bool {
    // Check common environment variables that indicate color support
    std::env::var("NO_COLOR").is_err() && 
    (std::env::var("FORCE_COLOR").is_ok() || 
     std::env::var("TERM").map_or(false, |term| {
         !term.is_empty() && term != "dumb"
     }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_without_colors() {
        let styled = StyledText::debug("Hello").bold();
        let result = format_styled_text(&styled, false);
        assert_eq!(result, "Hello");
    }

    #[test]
    fn test_format_with_colors() {
        let styled = StyledText::debug("Hello");
        let result = format_styled_text(&styled, true);
        assert!(result.contains("Hello"));
        assert!(result.contains(AnsiCodes::BRIGHT_BLUE));
        assert!(result.contains(AnsiCodes::RESET));
    }

    #[test]
    fn test_format_with_bold_style() {
        let styled = StyledText::new("Hello").bold();
        let result = format_styled_text(&styled, true);
        assert!(result.contains(AnsiCodes::BOLD));
        assert!(result.contains(AnsiCodes::RESET));
    }

    #[test]
    fn test_format_with_color_and_style() {
        let styled = StyledText::success("Done").bold();
        let result = format_styled_text(&styled, true);
        assert!(result.contains(AnsiCodes::BOLD));
        assert!(result.contains(AnsiCodes::BRIGHT_GREEN));
        assert!(result.contains(AnsiCodes::RESET));
    }

    #[test]
    fn test_all_semantic_colors() {
        let colors = [
            (SemanticColor::Debug, AnsiCodes::BRIGHT_BLUE),
            (SemanticColor::Success, AnsiCodes::BRIGHT_GREEN),
            (SemanticColor::Warning, AnsiCodes::BRIGHT_YELLOW),
            (SemanticColor::Error, AnsiCodes::BRIGHT_RED),
            (SemanticColor::Development, AnsiCodes::BRIGHT_MAGENTA),
            (SemanticColor::Data, AnsiCodes::BRIGHT_CYAN),
            (SemanticColor::Network, AnsiCodes::MAGENTA),
            (SemanticColor::Primary, AnsiCodes::BRIGHT_WHITE),
            (SemanticColor::Secondary, AnsiCodes::GRAY),
        ];

        for (color, expected_code) in colors {
            let styled = StyledText::new("test").with_color(color);
            let result = format_styled_text(&styled, true);
            assert!(result.contains(expected_code), 
                "Color {:?} should produce code {}", color, expected_code);
        }
    }

    #[test]
    fn test_no_formatting_for_plain_text() {
        let styled = StyledText::new("Plain text");
        let result = format_styled_text(&styled, true);
        assert_eq!(result, "Plain text");
        assert!(!result.contains(AnsiCodes::RESET));
    }
}
