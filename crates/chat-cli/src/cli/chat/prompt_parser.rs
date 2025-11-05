use crate::cli::agent::DEFAULT_AGENT_NAME;

/// Components extracted from a prompt string
#[derive(Debug, PartialEq)]
pub struct PromptComponents {
    pub delegate_notifier: Option<String>,
    pub profile: Option<String>,
    pub warning: bool,
    pub tangent_mode: bool,
    pub usage_percentage: Option<f32>,
}

/// Parse prompt components from a plain text prompt
pub fn parse_prompt_components(prompt: &str) -> Option<PromptComponents> {
    // Expected format: "[agent] 6% !> " or "> " or "!> " or "[agent] ↯ > " or "6% ↯ > " etc.
    let mut delegate_notifier = None::<String>;
    let mut profile = None;
    let mut warning = false;
    let mut tangent_mode = false;
    let mut usage_percentage = None;
    let mut remaining = prompt.trim();

    // Check for delegate notifier first
    if let Some(start) = remaining.find('[') {
        if let Some(end) = remaining.find(']') {
            if start < end {
                let content = &remaining[start + 1..end];
                // Only set profile if it's not "BACKGROUND TASK READY" or if it doesn't end with newline
                if content == "BACKGROUND TASK READY" && remaining[end + 1..].starts_with('\n') {
                    delegate_notifier = Some(content.to_string());
                    remaining = remaining[end + 1..].trim_start();
                }
            }
        }
    }

    // Check for agent pattern [agent] first
    if let Some(start) = remaining.find('[') {
        if let Some(end) = remaining.find(']') {
            if start < end {
                let content = &remaining[start + 1..end];
                profile = Some(content.to_string());
                remaining = remaining[end + 1..].trim_start();
            }
        }
    }

    // Check for percentage pattern (e.g., "6% ")
    if let Some(percent_pos) = remaining.find('%') {
        let before_percent = &remaining[..percent_pos];
        if let Ok(percentage) = before_percent.trim().parse::<f32>() {
            usage_percentage = Some(percentage);
            if let Some(space_after_percent) = remaining[percent_pos..].find(' ') {
                remaining = remaining[percent_pos + space_after_percent + 1..].trim_start();
            }
        }
    }

    // Check for tangent mode ↯ first
    if let Some(after_tangent) = remaining.strip_prefix('↯') {
        tangent_mode = true;
        remaining = after_tangent.trim_start();
    }

    // Check for warning symbol ! (comes after tangent mode)
    if remaining.starts_with('!') {
        warning = true;
        remaining = remaining[1..].trim_start();
    }

    // Should end with "> " for both normal and tangent mode
    if remaining.trim_end() == ">" {
        Some(PromptComponents {
            delegate_notifier,
            profile,
            warning,
            tangent_mode,
            usage_percentage,
        })
    } else {
        None
    }
}

pub fn generate_prompt(
    current_profile: Option<&str>,
    warning: bool,
    tangent_mode: bool,
    usage_percentage: Option<f32>,
    session_name: Option<&str>,
) -> String {
    // Generate plain text prompt that will be colored by highlight_prompt
    let warning_symbol = if warning { "!" } else { "" };
    let profile_part = current_profile
        .filter(|&p| p != DEFAULT_AGENT_NAME)
        .map(|p| format!("[{p}] "))
        .unwrap_or_default();

    let percentage_part = usage_percentage.map(|p| format!("{:.0}% ", p)).unwrap_or_default();
    
    let session_part = session_name.map(|s| format!("({}) ", s)).unwrap_or_default();

    if tangent_mode {
        format!("{session_part}{profile_part}{percentage_part}↯ {warning_symbol}> ")
    } else {
        format!("{session_part}{profile_part}{percentage_part}{warning_symbol}> ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_prompt() {
        // Test default prompt (no profile)
        assert_eq!(generate_prompt(None, false, false, None, None), "> ");
        // Test default prompt with warning
        assert_eq!(generate_prompt(None, true, false, None, None), "!> ");
        // Test tangent mode
        assert_eq!(generate_prompt(None, false, true, None, None), "↯ > ");
        // Test tangent mode with warning
        assert_eq!(generate_prompt(None, true, true, None, None), "↯ !> ");
        // Test default profile (should be same as no profile)
        assert_eq!(generate_prompt(Some(DEFAULT_AGENT_NAME), false, false, None, None), "> ");
        // Test custom profile
        assert_eq!(
            generate_prompt(Some("test-profile"), false, false, None, None),
            "[test-profile] > "
        );
        // Test custom profile with tangent mode
        assert_eq!(
            generate_prompt(Some("test-profile"), false, true, None, None),
            "[test-profile] ↯ > "
        );
        // Test another custom profile with warning
        assert_eq!(generate_prompt(Some("dev"), true, false, None, None), "[dev] !> ");
        // Test custom profile with warning and tangent mode
        assert_eq!(generate_prompt(Some("dev"), true, true, None, None), "[dev] ↯ !> ");
        // Test custom profile with usage percentage
        assert_eq!(
            generate_prompt(Some("rust-agent"), false, false, Some(6.2), None),
            "[rust-agent] 6% > "
        );
        // Test custom profile with usage percentage and warning
        assert_eq!(
            generate_prompt(Some("rust-agent"), true, false, Some(15.7), None),
            "[rust-agent] 16% !> "
        );
        // Test usage percentage without profile
        assert_eq!(generate_prompt(None, false, false, Some(25.3), None), "25% > ");
        // Test usage percentage with tangent mode
        assert_eq!(generate_prompt(None, false, true, Some(8.9), None), "9% ↯ > ");
        // Test session name
        assert_eq!(generate_prompt(None, false, false, None, Some("my-feature")), "(my-feature) > ");
        // Test session name with profile
        assert_eq!(
            generate_prompt(Some("dev"), false, false, None, Some("bugfix")),
            "(bugfix) [dev] > "
        );
    }

    #[test]
    fn test_parse_prompt_components() {
        // Test basic prompt
        let components = parse_prompt_components("> ").unwrap();
        assert!(components.profile.is_none());
        assert!(!components.warning);
        assert!(!components.tangent_mode);
        assert!(components.usage_percentage.is_none());

        // Test warning prompt
        let components = parse_prompt_components("!> ").unwrap();
        assert!(components.profile.is_none());
        assert!(components.warning);
        assert!(!components.tangent_mode);
        assert!(components.usage_percentage.is_none());

        // Test tangent mode
        let components = parse_prompt_components("↯ > ").unwrap();
        assert!(components.profile.is_none());
        assert!(!components.warning);
        assert!(components.tangent_mode);
        assert!(components.usage_percentage.is_none());

        // Test tangent mode with warning
        let components = parse_prompt_components("↯ !> ").unwrap();
        assert!(components.profile.is_none());
        assert!(components.warning);
        assert!(components.tangent_mode);
        assert!(components.usage_percentage.is_none());

        // Test profile prompt
        let components = parse_prompt_components("[test] > ").unwrap();
        assert_eq!(components.profile.as_deref(), Some("test"));
        assert!(!components.warning);
        assert!(!components.tangent_mode);
        assert!(components.usage_percentage.is_none());

        // Test profile with warning
        let components = parse_prompt_components("[dev] !> ").unwrap();
        assert_eq!(components.profile.as_deref(), Some("dev"));
        assert!(components.warning);
        assert!(!components.tangent_mode);
        assert!(components.usage_percentage.is_none());

        // Test profile with tangent mode
        let components = parse_prompt_components("[dev] ↯ > ").unwrap();
        assert_eq!(components.profile.as_deref(), Some("dev"));
        assert!(!components.warning);
        assert!(components.tangent_mode);
        assert!(components.usage_percentage.is_none());

        // Test profile with warning and tangent mode
        let components = parse_prompt_components("[dev] ↯ !> ").unwrap();
        assert_eq!(components.profile.as_deref(), Some("dev"));
        assert!(components.warning);
        assert!(components.tangent_mode);
        assert!(components.usage_percentage.is_none());

        // Test prompts with percentages
        let components = parse_prompt_components("[rust-agent] 6% > ").unwrap();
        assert_eq!(components.profile.as_deref(), Some("rust-agent"));
        assert!(!components.warning);
        assert!(!components.tangent_mode);
        assert_eq!(components.usage_percentage, Some(6.0));

        let components = parse_prompt_components("25% > ").unwrap();
        assert!(components.profile.is_none());
        assert!(!components.warning);
        assert!(!components.tangent_mode);
        assert_eq!(components.usage_percentage, Some(25.0));

        let components = parse_prompt_components("8% ↯ > ").unwrap();
        assert!(components.profile.is_none());
        assert!(!components.warning);
        assert!(components.tangent_mode);
        assert_eq!(components.usage_percentage, Some(8.0));

        // Test invalid prompt
        assert!(parse_prompt_components("invalid").is_none());
    }
}
