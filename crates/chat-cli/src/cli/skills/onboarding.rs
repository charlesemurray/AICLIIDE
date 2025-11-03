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

/// Run an interactive example to create a skill
pub fn run_interactive_example() -> Result<()> {
    use std::io::{self, Write, IsTerminal};

    // Check if running in interactive terminal
    if !io::stdin().is_terminal() {
        println!("❌ This command requires an interactive terminal");
        println!("💡 Use: q skills create <name> --from-template <template>");
        return Ok(());
    }

    println!("🎓 Interactive Skill Creation Example\n");
    println!("Let's create a simple skill together!\n");

    // Get skill name with validation
    let name = loop {
        print!("Enter skill name (alphanumeric, hyphens, underscores only): ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let name = input.trim();

        if name.is_empty() {
            println!("❌ Skill name cannot be empty");
            continue;
        }

        // Validate name: alphanumeric, hyphens, underscores only
        if !name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            println!("❌ Invalid characters. Use only letters, numbers, hyphens, underscores");
            continue;
        }

        // Check for path traversal
        if name.contains("..") || name.contains('/') || name.contains('\\') {
            println!("❌ Invalid skill name");
            continue;
        }

        break name.to_string();
    };

    // Get description
    print!("Enter description: ");
    io::stdout().flush()?;
    let mut description = String::new();
    io::stdin().read_line(&mut description)?;
    let description = description.trim();

    if description.is_empty() {
        println!("❌ Description cannot be empty");
        return Ok(());
    }

    // Choose template
    println!("\nAvailable templates:");
    println!("  1. command  - Run a shell command");
    println!("  2. script   - Execute a script file");
    println!("  3. http-api - Call an HTTP API");
    println!("  4. file-processor - Process files");

    let template = loop {
        print!("\nChoose template (1-4): ");
        io::stdout().flush()?;
        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;

        match choice.trim() {
            "1" => break "command",
            "2" => break "script",
            "3" => break "http-api",
            "4" => break "file-processor",
            _ => {
                println!("❌ Invalid choice. Enter 1, 2, 3, or 4");
                continue;
            }
        }
    };

    // Show what will be created
    println!("\n📝 Creating skill with:");
    println!("  Name: {}", name);
    println!("  Description: {}", description);
    println!("  Template: {}", template);

    print!("\nCreate this skill? (y/N): ");
    io::stdout().flush()?;
    let mut confirm = String::new();
    io::stdin().read_line(&mut confirm)?;

    if confirm.trim().to_lowercase() != "y" {
        println!("❌ Cancelled");
        return Ok(());
    }

    // Create the skill
    use crate::cli::skills::templates::SkillTemplate;

    let skill_template = match template {
        "command" => SkillTemplate::Command,
        "script" => SkillTemplate::Script,
        "http-api" => SkillTemplate::HttpApi,
        "file-processor" => SkillTemplate::FileProcessor,
        _ => unreachable!(),
    };

    let skill_json = skill_template.generate(&name, description);

    // Save to ~/.q-skills/
    let skills_dir = dirs::home_dir()
        .ok_or_else(|| eyre::eyre!("Could not find home directory"))?
        .join(".q-skills");

    std::fs::create_dir_all(&skills_dir)?;
    let skill_file = skills_dir.join(format!("{}.json", name));
    
    // Check if file already exists
    if skill_file.exists() {
        println!("❌ Skill '{}' already exists at {}", name, skill_file.display());
        return Ok(());
    }
    
    std::fs::write(&skill_file, serde_json::to_string_pretty(&skill_json)?)?;

    println!("\n✅ Created skill: {}", skill_file.display());
    println!("\n📚 Next steps:");
    println!("  • View: q skills info {}", name);
    println!("  • Edit: Open {}", skill_file.display());
    println!("  • Use: q chat \"use {} to...\"", name);

    Ok(())
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
