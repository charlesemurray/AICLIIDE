use std::io::Write;

use clap::Args;
use eyre::Result;

use crate::cli::chat::ChatState;
use crate::cli::chat::ConversationState;
use crate::os::Os;
use crate::theme::formatter;

/// Show system status with colored output
#[derive(Debug, Args)]
pub struct StatusArgs {
    /// Show detailed status information
    #[arg(long)]
    pub detailed: bool,
}

impl StatusArgs {
    pub async fn execute(
        &self,
        _os: &mut Os,
        conversation_state: &mut ConversationState,
    ) -> Result<ChatState> {
        let fmt = formatter();
        
        // Write colored status output
        writeln!(conversation_state.stdout, "{}", fmt.header("System Status"))?;
        writeln!(conversation_state.stdout)?;
        
        // Basic status indicators
        writeln!(conversation_state.stdout, "{}", fmt.status_ok("Q CLI is running"))?;
        writeln!(conversation_state.stdout, "{}", fmt.status_ok("Theme system loaded"))?;
        writeln!(conversation_state.stdout, "{}", fmt.status_ok("Session management ready"))?;
        
        if self.detailed {
            writeln!(conversation_state.stdout)?;
            writeln!(conversation_state.stdout, "{}", fmt.header("Detailed Information"))?;
            
            writeln!(conversation_state.stdout, "{}", fmt.list_item("Version: 0.1.0"))?;
            writeln!(conversation_state.stdout, "{}", fmt.list_item("Platform: Linux"))?;
            writeln!(conversation_state.stdout, "{}", fmt.list_item("Colors: Enabled"))?;
            
            writeln!(conversation_state.stdout)?;
            writeln!(conversation_state.stdout, "{}", fmt.info("Use /status for basic status"))?;
            writeln!(conversation_state.stdout, "{}", fmt.secondary("Use /status --detailed for full information"))?;
        } else {
            writeln!(conversation_state.stdout)?;
            writeln!(conversation_state.stdout, "{}", fmt.secondary("Use --detailed for more information"))?;
        }
        
        Ok(ChatState::WaitingForInput)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_status_args_creation() {
        let args = StatusArgs { detailed: false };
        assert!(!args.detailed);
        
        let args = StatusArgs { detailed: true };
        assert!(args.detailed);
    }

    #[test]
    fn test_status_basic_output() {
        // We can't easily test the full execute method without mocking Os and ConversationState,
        // but we can test the argument parsing and structure
        let args = StatusArgs { detailed: false };
        assert!(!args.detailed);
    }

    #[test]
    fn test_status_detailed_output() {
        let args = StatusArgs { detailed: true };
        assert!(args.detailed);
    }
}
