//! Minimal integration of multi-session feature into chat

use std::io::Write;

use eyre::Result;

use crate::cli::chat::coordinator::MultiSessionCoordinator;
use crate::cli::chat::input_router::{
    InputRouter,
    SessionCommand,
};
use crate::cli::chat::session_switcher::SessionSwitcher;
use crate::cli::chat::visual_feedback::VisualFeedback;

/// Handle session command with coordinator
pub async fn handle_session_command<W: Write>(
    input: &str,
    coordinator: &mut MultiSessionCoordinator,
    writer: &mut W,
) -> Result<bool> {
    // Check for help
    if input.contains("help") || input.contains("--help") || input.contains("-h") {
        show_help(writer)?;
        return Ok(true);
    }

    // Parse command
    let cmd = match InputRouter::parse(input)? {
        Some(cmd) => cmd,
        None => return Ok(false), // Not a session command
    };

    // Execute command
    let mut switcher = SessionSwitcher::new();

    match cmd {
        SessionCommand::List { waiting, .. } => {
            if waiting {
                switcher.list_waiting_sessions(coordinator, writer).await?;
            } else {
                switcher.list_sessions(coordinator, writer).await?;
            }
        },
        SessionCommand::Switch(name) => {
            switcher.switch_to(coordinator, &name, writer).await?;
            // Save active session state
            if let Some(id) = coordinator.active_session_id().await {
                let _ = coordinator.save_session(&id).await; // Ignore errors
            }
        },
        SessionCommand::New { session_type, name } => {
            VisualFeedback::info(
                writer,
                &format!(
                    "Creating new {:?} session{}",
                    session_type,
                    name.as_ref().map(|n| format!(" '{}'", n)).unwrap_or_default()
                ),
            )?;
            // Note: Actual creation requires more parameters from chat context
            VisualFeedback::warning(writer, "Session creation not yet fully integrated")?;
        },
        SessionCommand::Close(name_opt) => {
            let name = name_opt.as_ref().ok_or_else(|| eyre::eyre!("Session name required"))?;
            coordinator.close_session(name).await?;
            VisualFeedback::success(writer, &format!("Closed session '{}'", name))?;
        },
        SessionCommand::Rename(new_name) => {
            VisualFeedback::warning(writer, &format!("Rename to '{}' not yet implemented", new_name))?;
        },
        SessionCommand::SessionName(name_opt) => {
            if let Some(name) = name_opt {
                VisualFeedback::warning(writer, &format!("Set name to '{}' not yet implemented", name))?;
            } else {
                VisualFeedback::info(writer, "View session name not yet implemented")?;
            }
        },
    }

    Ok(true) // Command was handled
}

fn show_help<W: Write>(writer: &mut W) -> Result<()> {
    writeln!(writer, "\n📋 Session Commands:")?;
    writeln!(writer, "\n  /sessions              List active sessions")?;
    writeln!(writer, "  /sessions --waiting    List sessions waiting for input")?;
    writeln!(writer, "  /sessions --all        List all sessions (including completed)")?;
    writeln!(writer, "\n  /switch <name>         Switch to a session by name")?;
    writeln!(writer, "  /s <name>              Short alias for /switch")?;
    writeln!(writer, "\n  /new [name]            Create a new session")?;
    writeln!(writer, "  /close [name]          Close a session (current if no name)")?;
    writeln!(writer, "  /rename <name>         Rename current session")?;
    writeln!(writer, "\nExamples:")?;
    writeln!(writer, "  /sessions              # List all active sessions")?;
    writeln!(writer, "  /sessions --waiting    # Show sessions needing input")?;
    writeln!(writer, "  /switch my-feature     # Switch to 'my-feature' session")?;
    writeln!(writer, "  /new auth-work         # Create new session named 'auth-work'")?;
    writeln!(writer)?;
    Ok(())
}
