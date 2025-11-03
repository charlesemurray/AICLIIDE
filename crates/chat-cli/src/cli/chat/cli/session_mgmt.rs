use clap::{Args, Subcommand};

use crate::cli::chat::{ChatError, ChatSession, ChatState};
use crate::os::Os;
use crate::session::{SessionManager, SessionStatus};

#[derive(Debug, PartialEq, Args)]
pub struct SessionMgmtArgs {
    #[command(subcommand)]
    pub command: SessionMgmtSubcommand,
}

#[derive(Debug, PartialEq, Subcommand)]
pub enum SessionMgmtSubcommand {
    /// List active sessions
    List,
    /// Show archived sessions
    History {
        /// Limit number of results
        #[arg(long, default_value = "10")]
        limit: usize,
        /// Search term to filter sessions
        #[arg(long)]
        search: Option<String>,
    },
    /// Show background sessions
    Background {
        /// Limit number of results
        #[arg(long, default_value = "10")]
        limit: usize,
        /// Search term to filter sessions
        #[arg(long)]
        search: Option<String>,
    },
    /// Archive a session
    Archive {
        /// Session ID to archive
        session_id: String,
    },
    /// Name a session
    Name {
        /// Session ID to name
        session_id: String,
        /// New name for the session
        name: String,
    },
}

impl SessionMgmtArgs {
    pub async fn execute(self, _session: &mut ChatSession, os: &Os) -> Result<ChatState, ChatError> {
        match self.command {
            SessionMgmtSubcommand::List => {
                let manager = SessionManager::new(os);
                let sessions = manager
                    .list_by_status(SessionStatus::Active)
                    .await
                    .map_err(|e| ChatError::Std(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

                println!("💬 Active Sessions:");
                if sessions.is_empty() {
                    println!("  No active sessions found");
                } else {
                    for (idx, session) in sessions.iter().enumerate() {
                        let name = session.name.as_deref().unwrap_or(&session.id[..8]);
                        let age = format_duration(session.last_active);
                        println!(
                            "  {}. {} - \"{}\" ({} ago, {} messages, {} files)",
                            idx + 1,
                            name,
                            session.first_message,
                            age,
                            session.message_count,
                            session.file_count
                        );
                    }
                }

                Ok(ChatState::PromptUser {
                    skip_printing_tools: true,
                })
            },
            SessionMgmtSubcommand::History { limit, search } => {
                let manager = SessionManager::new(os);
                let mut sessions = manager
                    .list_by_status(SessionStatus::Archived)
                    .await
                    .map_err(|e| ChatError::Std(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

                if let Some(term) = search {
                    sessions.retain(|s| {
                        s.first_message.to_lowercase().contains(&term.to_lowercase())
                            || s.name.as_ref().map_or(false, |n| n.to_lowercase().contains(&term.to_lowercase()))
                    });
                }

                sessions.truncate(limit);

                println!("📚 Session History:");
                if sessions.is_empty() {
                    println!("  No archived sessions found");
                } else {
                    for (idx, session) in sessions.iter().enumerate() {
                        let name = session.name.as_deref().unwrap_or(&session.id[..8]);
                        let age = format_duration(session.last_active);
                        println!(
                            "  {}. {} - \"{}\" ({} ago, {} files)",
                            idx + 1,
                            name,
                            session.first_message,
                            age,
                            session.file_count
                        );
                    }
                }

                Ok(ChatState::PromptUser {
                    skip_printing_tools: true,
                })
            },
            SessionMgmtSubcommand::Background { limit, search } => {
                let manager = SessionManager::new(os);
                let mut sessions = manager
                    .list_by_status(SessionStatus::Background)
                    .await
                    .map_err(|e| ChatError::Std(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

                if let Some(term) = search {
                    sessions.retain(|s| {
                        s.first_message.to_lowercase().contains(&term.to_lowercase())
                            || s.name.as_ref().map_or(false, |n| n.to_lowercase().contains(&term.to_lowercase()))
                    });
                }

                sessions.truncate(limit);

                println!("🔄 Background Sessions:");
                if sessions.is_empty() {
                    println!("  No background sessions found");
                } else {
                    for (idx, session) in sessions.iter().enumerate() {
                        let name = session.name.as_deref().unwrap_or(&session.id[..8]);
                        let age = format_duration(session.last_active);
                        println!(
                            "  {}. {} - \"{}\" ({} ago, {} files)",
                            idx + 1,
                            name,
                            session.first_message,
                            age,
                            session.file_count
                        );
                    }
                }

                Ok(ChatState::PromptUser {
                    skip_printing_tools: true,
                })
            },
            SessionMgmtSubcommand::Archive { session_id } => {
                let manager = SessionManager::new(os);
                manager
                    .archive_session(&session_id)
                    .await
                    .map_err(|e| ChatError::Std(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

                println!("✓ Session '{}' archived successfully", session_id);
                Ok(ChatState::PromptUser {
                    skip_printing_tools: true,
                })
            },
            SessionMgmtSubcommand::Name { session_id, name } => {
                let manager = SessionManager::new(os);
                manager
                    .name_session(&session_id, name.clone())
                    .await
                    .map_err(|e| ChatError::Std(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

                println!("✓ Session '{}' named: {}", session_id, name);
                Ok(ChatState::PromptUser {
                    skip_printing_tools: true,
                })
            },
        }
    }
}

fn format_duration(timestamp: time::OffsetDateTime) -> String {
    let now = time::OffsetDateTime::now_utc();
    let duration = now - timestamp;

    if duration.whole_days() > 0 {
        format!("{} days", duration.whole_days())
    } else if duration.whole_hours() > 0 {
        format!("{} hours", duration.whole_hours())
    } else if duration.whole_minutes() > 0 {
        format!("{} minutes", duration.whole_minutes())
    } else {
        "just now".to_string()
    }
}
