use crossterm::style::Color;

use crate::theme::{StatusColors, UiColors};

/// Formatter for command output with semantic colors
#[derive(Debug)]
pub struct CommandOutputFormatter<'a> {
    status_colors: &'a StatusColors,
    ui_colors: &'a UiColors,
}

impl<'a> CommandOutputFormatter<'a> {
    pub fn new(status_colors: &'a StatusColors, ui_colors: &'a UiColors) -> Self {
        Self {
            status_colors,
            ui_colors,
        }
    }

    /// Format a success message
    pub fn success(&self, message: impl Into<String>) -> String {
        use crossterm::style::Stylize;
        message.into().with(self.status_colors.success).to_string()
    }

    /// Format an error message
    pub fn error(&self, message: impl Into<String>) -> String {
        use crossterm::style::Stylize;
        message.into().with(self.status_colors.error).to_string()
    }

    /// Format a warning message
    pub fn warning(&self, message: impl Into<String>) -> String {
        use crossterm::style::Stylize;
        message.into().with(self.status_colors.warning).to_string()
    }

    /// Format an info message
    pub fn info(&self, message: impl Into<String>) -> String {
        use crossterm::style::Stylize;
        message.into().with(self.status_colors.info).to_string()
    }

    /// Format primary text
    pub fn primary(&self, message: impl Into<String>) -> String {
        use crossterm::style::Stylize;
        message.into().with(self.ui_colors.primary_text).to_string()
    }

    /// Format secondary/dim text
    pub fn secondary(&self, message: impl Into<String>) -> String {
        use crossterm::style::Stylize;
        message.into().with(self.ui_colors.secondary_text).to_string()
    }

    /// Format emphasized text
    pub fn emphasis(&self, message: impl Into<String>) -> String {
        use crossterm::style::Stylize;
        message.into().with(self.ui_colors.emphasis).to_string()
    }

    /// Format command/code text
    pub fn command(&self, message: impl Into<String>) -> String {
        use crossterm::style::Stylize;
        message.into().with(self.ui_colors.command_highlight).to_string()
    }

    /// Format a list item with bullet
    pub fn list_item(&self, message: impl Into<String>) -> String {
        format!("  {} {}", self.secondary("•"), self.primary(message.into()))
    }

    /// Format a header with emphasis
    pub fn header(&self, message: impl Into<String>) -> String {
        use crossterm::style::Stylize;
        message.into().with(self.ui_colors.emphasis).bold().to_string()
    }

    /// Format a file path
    pub fn file_path(&self, path: impl Into<String>) -> String {
        use crossterm::style::Stylize;
        path.into().with(Color::Cyan).to_string()
    }

    /// Format a status indicator with checkmark
    pub fn status_ok(&self, message: impl Into<String>) -> String {
        format!("{} {}", self.success("✓"), self.primary(message.into()))
    }

    /// Format a status indicator with X
    pub fn status_error(&self, message: impl Into<String>) -> String {
        format!("{} {}", self.error("✗"), self.primary(message.into()))
    }

    /// Format a status indicator with warning
    pub fn status_warning(&self, message: impl Into<String>) -> String {
        format!("{} {}", self.warning("⚠"), self.primary(message.into()))
    }
}

/// Create a formatter using the global theme
pub fn formatter() -> CommandOutputFormatter<'static> {
    use crate::theme;
    CommandOutputFormatter::new(&theme::theme().status, &theme::theme().ui)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{StatusColors, UiColors};

    fn create_test_formatter() -> CommandOutputFormatter<'static> {
        // Use default colors for testing
        static STATUS: StatusColors = StatusColors {
            error: Color::Red,
            warning: Color::Yellow,
            success: Color::Green,
            info: Color::Blue,
        };
        static UI: UiColors = UiColors {
            primary_brand: Color::Cyan,
            primary_text: Color::White,
            secondary_text: Color::DarkGrey,
            emphasis: Color::Magenta,
            command_highlight: Color::Green,
        };

        CommandOutputFormatter::new(&STATUS, &UI)
    }

    #[test]
    fn test_success_formatting() {
        let formatter = create_test_formatter();
        let result = formatter.success("Operation completed");
        assert!(result.contains("Operation completed"));
        // Note: We can't easily test ANSI codes in unit tests, but structure is correct
    }

    #[test]
    fn test_error_formatting() {
        let formatter = create_test_formatter();
        let result = formatter.error("Something went wrong");
        assert!(result.contains("Something went wrong"));
    }

    #[test]
    fn test_warning_formatting() {
        let formatter = create_test_formatter();
        let result = formatter.warning("This is a warning");
        assert!(result.contains("This is a warning"));
    }

    #[test]
    fn test_info_formatting() {
        let formatter = create_test_formatter();
        let result = formatter.info("Information message");
        assert!(result.contains("Information message"));
    }

    #[test]
    fn test_primary_formatting() {
        let formatter = create_test_formatter();
        let result = formatter.primary("Primary text");
        assert!(result.contains("Primary text"));
    }

    #[test]
    fn test_secondary_formatting() {
        let formatter = create_test_formatter();
        let result = formatter.secondary("Secondary text");
        assert!(result.contains("Secondary text"));
    }

    #[test]
    fn test_emphasis_formatting() {
        let formatter = create_test_formatter();
        let result = formatter.emphasis("Important text");
        assert!(result.contains("Important text"));
    }

    #[test]
    fn test_command_formatting() {
        let formatter = create_test_formatter();
        let result = formatter.command("git status");
        assert!(result.contains("git status"));
    }

    #[test]
    fn test_list_item_formatting() {
        let formatter = create_test_formatter();
        let result = formatter.list_item("First item");
        assert!(result.contains("•"));
        assert!(result.contains("First item"));
        assert!(result.starts_with("  "));
    }

    #[test]
    fn test_header_formatting() {
        let formatter = create_test_formatter();
        let result = formatter.header("Section Header");
        assert!(result.contains("Section Header"));
    }

    #[test]
    fn test_file_path_formatting() {
        let formatter = create_test_formatter();
        let result = formatter.file_path("/path/to/file.txt");
        assert!(result.contains("/path/to/file.txt"));
    }

    #[test]
    fn test_status_indicators() {
        let formatter = create_test_formatter();

        let ok = formatter.status_ok("Task completed");
        assert!(ok.contains("✓"));
        assert!(ok.contains("Task completed"));

        let error = formatter.status_error("Task failed");
        assert!(error.contains("✗"));
        assert!(error.contains("Task failed"));

        let warning = formatter.status_warning("Task has issues");
        assert!(warning.contains("⚠"));
        assert!(warning.contains("Task has issues"));
    }

    #[test]
    fn test_global_formatter() {
        let formatter = formatter();
        let result = formatter.success("Global theme test");
        assert!(result.contains("Global theme test"));
    }

    #[test]
    fn test_multiple_formatting_calls() {
        let formatter = create_test_formatter();

        let header = formatter.header("Test Results");
        let success = formatter.status_ok("All tests passed");
        let info = formatter.info("Run completed in 2.3s");

        assert!(header.contains("Test Results"));
        assert!(success.contains("✓"));
        assert!(success.contains("All tests passed"));
        assert!(info.contains("Run completed in 2.3s"));
    }
}
