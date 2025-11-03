//! Minimal integration of multi-session feature into chat

use std::io::Write;
use eyre::Result;

use crate::cli::chat::coordinator::MultiSessionCoordinator;
use crate::cli::chat::input_router::{InputRouter, SessionCommand};
use crate::cli::chat::session_switcher::SessionSwitcher;
use crate::cli::chat::visual_feedback::VisualFeedback;

/// Handle session command with coordinator
pub async fn handle_session_command<W: Write>(
    input: &str,
    coordinator: &mut MultiSessionCoordinator,
    writer: &mut W,
) -> Result<bool> {
    // Parse command
    let cmd = match InputRouter::parse(input)? {
        Some(cmd) => cmd,
        None => return Ok(false), // Not a session command
    };

    // Execute command
    let mut switcher = SessionSwitcher::new();
    
    match cmd {
        SessionCommand::List { .. } => {
            switcher.list_sessions(coordinator, writer).await?;
        }
        SessionCommand::Switch(name) => {
            switcher.switch_to(coordinator, &name, writer).await?;
            // Save active session state
            if let Some(id) = coordinator.active_session_id().await {
                let _ = coordinator.save_session(&id).await; // Ignore errors
            }
        }
        SessionCommand::New { session_type, name } => {
            VisualFeedback::info(writer, &format!(
                "Creating new {:?} session{}",
                session_type,
                name.as_ref().map(|n| format!(" '{}'", n)).unwrap_or_default()
            ))?;
            // Note: Actual creation requires more parameters from chat context
            VisualFeedback::warning(writer, "Session creation not yet fully integrated")?;
        }
        SessionCommand::Close(name_opt) => {
            let name = name_opt.as_ref().ok_or_else(|| eyre::eyre!("Session name required"))?;
            coordinator.close_session(name).await?;
            VisualFeedback::success(writer, &format!("Closed session '{}'", name))?;
        }
        SessionCommand::Rename(new_name) => {
            VisualFeedback::warning(writer, &format!("Rename to '{}' not yet implemented", new_name))?;
        }
        SessionCommand::SessionName(name_opt) => {
            if let Some(name) = name_opt {
                VisualFeedback::warning(writer, &format!("Set name to '{}' not yet implemented", name))?;
            } else {
                VisualFeedback::info(writer, "View session name not yet implemented")?;
            }
        }
    }

    Ok(true) // Command was handled
}
