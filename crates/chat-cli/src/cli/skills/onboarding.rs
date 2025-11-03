//! Onboarding experience for skills feature

use std::io::Write;
use std::path::PathBuf;

use eyre::Result;

const TUTORIAL_SHOWN_FILE: &str = ".q-skills-tutorial-shown";

/// Check if the first-run tutorial has been shown
pub fn has_shown_tutorial() -> bool {
    if let Some(home) = dirs::home_dir() {
        home.join(TUTORIAL_SHOWN_FILE).exists()
    } else {
        true // Assume shown if can't determine
    }
}

/// Mark the tutorial as shown
fn mark_tutorial_shown() -> Result<()> {
    if let Some(home) = dirs::home_dir() {
        let marker = home.join(TUTORIAL_SHOWN_FILE);
        std::fs::write(marker, "")?;
    }
    Ok(())
}

/// Show the first-run tutorial
pub fn show_tutorial(output: &mut impl Write) -> Result<()> {
    writeln!(output, "Welcome to Q Skills! 🎉\n")?;
    writeln!(output, "Skills let you extend Q with custom capabilities.\n")?;

    writeln!(output, "Quick Start:")?;
    writeln!(output, "  1. List skills: q skills list")?;
    writeln!(output, "  2. Use in chat: q chat \"use calculator to add 5 and 3\"")?;
    writeln!(output, "  3. Get details: q skills info calculator\n")?;

    writeln!(output, "Example skills are in: examples/skills/")?;
    writeln!(output, "Learn more: docs/SKILLS_QUICKSTART.md\n")?;

    mark_tutorial_shown()?;

    Ok(())
}

/// Show tutorial if it hasn't been shown yet
pub fn show_tutorial_if_needed(output: &mut impl Write) -> Result<bool> {
    if !has_shown_tutorial() {
        show_tutorial(output)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tutorial_output() {
        let mut output = Vec::new();
        show_tutorial(&mut output).unwrap();

        let text = String::from_utf8(output).unwrap();
        assert!(text.contains("Welcome to Q Skills!"));
        assert!(text.contains("Quick Start:"));
        assert!(text.contains("q skills list"));
        assert!(text.contains("examples/skills/"));
    }

    #[test]
    fn test_tutorial_content() {
        let mut output = Vec::new();
        show_tutorial(&mut output).unwrap();

        let text = String::from_utf8(output).unwrap();

        // Check all key elements
        assert!(text.contains("🎉"));
        assert!(text.contains("1."));
        assert!(text.contains("2."));
        assert!(text.contains("3."));
        assert!(text.contains("q chat"));
        assert!(text.contains("calculator"));
    }
}
