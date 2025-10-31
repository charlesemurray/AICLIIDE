use std::fmt;

/// Semantic colors for Q CLI interface
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SemanticColor {
    Debug,      // Blue - debug sessions, analysis, system info
    Success,    // Green - success states, planning, healthy status
    Warning,    // Yellow - warnings, build ops, degraded status
    Error,      // Red - errors, failures, critical issues
    Development,// Purple - development sessions, skill creation
    Data,       // Cyan - file operations, data, secondary info
    Network,    // Magenta - network operations, external services
    Primary,    // White - primary content, results
    Secondary,  // Gray - secondary text, hints, metadata
}

/// Text styling options
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextStyle {
    Normal,
    Bold,
    Dim,
    Italic,
}

/// A styled text element with color and formatting
#[derive(Debug, Clone, PartialEq)]
pub struct StyledText {
    pub text: String,
    pub color: Option<SemanticColor>,
    pub style: TextStyle,
}

impl StyledText {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            color: None,
            style: TextStyle::Normal,
        }
    }

    pub fn with_color(mut self, color: SemanticColor) -> Self {
        self.color = Some(color);
        self
    }

    pub fn with_style(mut self, style: TextStyle) -> Self {
        self.style = style;
        self
    }

    pub fn debug(text: impl Into<String>) -> Self {
        Self::new(text).with_color(SemanticColor::Debug)
    }

    pub fn success(text: impl Into<String>) -> Self {
        Self::new(text).with_color(SemanticColor::Success)
    }

    pub fn warning(text: impl Into<String>) -> Self {
        Self::new(text).with_color(SemanticColor::Warning)
    }

    pub fn error(text: impl Into<String>) -> Self {
        Self::new(text).with_color(SemanticColor::Error)
    }

    pub fn development(text: impl Into<String>) -> Self {
        Self::new(text).with_color(SemanticColor::Development)
    }

    pub fn data(text: impl Into<String>) -> Self {
        Self::new(text).with_color(SemanticColor::Data)
    }

    pub fn network(text: impl Into<String>) -> Self {
        Self::new(text).with_color(SemanticColor::Network)
    }

    pub fn primary(text: impl Into<String>) -> Self {
        Self::new(text).with_color(SemanticColor::Primary)
    }

    pub fn secondary(text: impl Into<String>) -> Self {
        Self::new(text).with_color(SemanticColor::Secondary)
    }

    pub fn bold(mut self) -> Self {
        self.style = TextStyle::Bold;
        self
    }

    pub fn dim(mut self) -> Self {
        self.style = TextStyle::Dim;
        self
    }
}

impl fmt::Display for StyledText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // For now, just return the text without ANSI codes
        // ANSI formatting will be added in the next step
        write!(f, "{}", self.text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_styled_text_creation() {
        let text = StyledText::new("Hello");
        assert_eq!(text.text, "Hello");
        assert_eq!(text.color, None);
        assert_eq!(text.style, TextStyle::Normal);
    }

    #[test]
    fn test_semantic_color_constructors() {
        let debug_text = StyledText::debug("Debug message");
        assert_eq!(debug_text.color, Some(SemanticColor::Debug));
        assert_eq!(debug_text.text, "Debug message");

        let success_text = StyledText::success("Success!");
        assert_eq!(success_text.color, Some(SemanticColor::Success));

        let error_text = StyledText::error("Error occurred");
        assert_eq!(error_text.color, Some(SemanticColor::Error));
    }

    #[test]
    fn test_text_styling() {
        let bold_text = StyledText::new("Bold text").bold();
        assert_eq!(bold_text.style, TextStyle::Bold);

        let dim_text = StyledText::new("Dim text").dim();
        assert_eq!(dim_text.style, TextStyle::Dim);
    }

    #[test]
    fn test_chaining() {
        let styled = StyledText::debug("Debug message").bold();
        assert_eq!(styled.color, Some(SemanticColor::Debug));
        assert_eq!(styled.style, TextStyle::Bold);
        assert_eq!(styled.text, "Debug message");
    }

    #[test]
    fn test_display_basic() {
        let text = StyledText::new("Hello World");
        assert_eq!(format!("{}", text), "Hello World");
    }
}
