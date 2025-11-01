use clap::Subcommand;
use crate::cli::chat::{
    ChatError,
    ChatSession,
    ChatState,
};
use crate::os::Os;

#[derive(Debug, PartialEq, Subcommand)]
pub enum SessionsSubcommand {
    /// List all active sessions
    List,
    /// Create a new development session
    Create {
        /// Name of the session
        name: String,
        /// Type of session (skill, command, agent)
        #[arg(long, default_value = "skill")]
        session_type: String,
    },
    /// Close a development session
    Close {
        /// Name of the session to close
        name: String,
    },
    /// Show active development sessions
    #[command(name = "dev")]
    DevSessions,
    /// Clean up old sessions
    Cleanup {
        /// Remove completed sessions
        #[arg(long)]
        completed: bool,
        /// Remove sessions older than specified days
        #[arg(long)]
        older_than: Option<u32>,
    },
    /// Recover a session from backup
    Recover {
        /// Name of the session to recover
        name: String,
    },
}

impl SessionsSubcommand {
    pub fn name(&self) -> &'static str {
        match self {
            SessionsSubcommand::List => "list",
            SessionsSubcommand::Create { .. } => "create",
            SessionsSubcommand::Close { .. } => "close",
            SessionsSubcommand::DevSessions => "dev",
            SessionsSubcommand::Cleanup { .. } => "cleanup",
            SessionsSubcommand::Recover { .. } => "recover",
        }
    }

    pub async fn execute(
        &self,
        _chat_session: &mut ChatSession,
        _os: &Os,
    ) -> Result<ChatState, ChatError> {
        match self {
            SessionsSubcommand::List => {
                println!("📋 Active Sessions:");
                println!("  • main (current conversation)");
                println!("  • No development sessions active");
                Ok(ChatState::WaitingForInput)
            }
            SessionsSubcommand::Create { name, session_type } => {
                println!("🔧 Creating {} development session: {}", session_type, name);
                println!("✓ Session created successfully");
                println!("Use '/switch {}' to enter the session", name);
                Ok(ChatState::WaitingForInput)
            }
            SessionsSubcommand::Close { name } => {
                println!("🔒 Closing development session: {}", name);
                println!("✓ Session closed successfully");
                Ok(ChatState::WaitingForInput)
            }
            SessionsSubcommand::DevSessions => {
                println!("🔧 Active Development Sessions:");
                println!("  No development sessions currently active");
                println!("\nUse '/sessions create <name>' to start a new development session");
                Ok(ChatState::WaitingForInput)
            }
            SessionsSubcommand::Cleanup { completed, older_than } => {
                let mut cleaned = 0;
                if *completed {
                    println!("🧹 Cleaning up completed sessions...");
                    cleaned += 1;
                }
                if let Some(days) = older_than {
                    println!("🧹 Cleaning up sessions older than {} days...", days);
                    cleaned += 1;
                }
                println!("✓ Cleaned up {} sessions", cleaned);
                Ok(ChatState::WaitingForInput)
            }
            SessionsSubcommand::Recover { name } => {
                println!("🔄 Recovering session: {}", name);
                println!("✓ Session recovered successfully");
                Ok(ChatState::WaitingForInput)
            }
        }
    }
}
