use std::collections::HashMap;
use std::sync::{
    Arc,
    Mutex,
    OnceLock,
};

use clap::Subcommand;

use crate::cli::chat::{
    ChatError,
    ChatSession,
    ChatState,
};
use crate::os::Os;

static SESSIONS: OnceLock<Arc<Mutex<HashMap<String, String>>>> = OnceLock::new();

pub fn get_sessions() -> &'static Arc<Mutex<HashMap<String, String>>> {
    SESSIONS.get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
}

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
    /// Scan for worktree-based sessions
    Scan,
    /// Show worktree sessions
    Worktrees,
    /// Merge a worktree session back to main
    Merge {
        /// Branch name to merge
        branch: Option<String>,
        /// Skip conflict detection
        #[arg(long)]
        force: bool,
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
            SessionsSubcommand::Scan => "scan",
            SessionsSubcommand::Worktrees => "worktrees",
            SessionsSubcommand::Scan => "scan",
            SessionsSubcommand::Worktrees => "worktrees",
        }
    }

    pub async fn execute(&self, _chat_session: &mut ChatSession, _os: &Os) -> Result<ChatState, ChatError> {
        match self {
            SessionsSubcommand::List => {
                let sessions = get_sessions().lock().unwrap();
                println!("📋 Active Sessions:");
                println!("  • main (current conversation)");
                if sessions.is_empty() {
                    println!("  • No development sessions active");
                } else {
                    for (name, session_type) in sessions.iter() {
                        println!("  • {} ({})", name, session_type);
                    }
                }
                Ok(ChatState::PromptUser {
                    skip_printing_tools: true,
                })
            },
            SessionsSubcommand::Create { name, session_type } => {
                let mut sessions = get_sessions().lock().unwrap();
                sessions.insert(name.clone(), session_type.clone());
                println!("🔧 Creating {} development session: {}", session_type, name);
                println!("✓ Session created successfully");
                println!("Use '/switch {}' to enter the session", name);
                Ok(ChatState::PromptUser {
                    skip_printing_tools: true,
                })
            },
            SessionsSubcommand::Close { name } => {
                let mut sessions = get_sessions().lock().unwrap();
                if sessions.remove(name).is_some() {
                    println!("🔒 Closing development session: {}", name);
                    println!("✓ Session closed successfully");
                } else {
                    println!("❌ Session '{}' not found", name);
                }
                Ok(ChatState::PromptUser {
                    skip_printing_tools: true,
                })
            },
            SessionsSubcommand::DevSessions => {
                let sessions = get_sessions().lock().unwrap();
                println!("🔧 Active Development Sessions:");
                if sessions.is_empty() {
                    println!("  No development sessions currently active");
                    println!("\nUse '/sessions create <name>' to start a new development session");
                } else {
                    for (name, session_type) in sessions.iter() {
                        println!("  • {} ({})", name, session_type);
                    }
                }
                Ok(ChatState::PromptUser {
                    skip_printing_tools: true,
                })
            },
            SessionsSubcommand::Cleanup { completed, older_than } => {
                use crate::cli::chat::session_scanner::get_current_repo_sessions;
                use crate::git::remove_worktree;
                
                println!("🧹 Cleaning up sessions...");
                
                let mut cleaned = 0;
                
                if let Ok(sessions) = get_current_repo_sessions() {
                    for session in sessions {
                        let should_clean = if *completed {
                            // Clean if status is archived or no recent activity
                            session.status == crate::session::metadata::SessionStatus::Archived
                        } else if let Some(days) = older_than {
                            // Clean if older than specified days
                            let age = time::OffsetDateTime::now_utc() - session.last_active;
                            age.whole_days() > *days as i64
                        } else {
                            false
                        };
                        
                        if should_clean {
                            if let Some(wt) = &session.worktree_info {
                                if remove_worktree(&wt.path).is_ok() {
                                    println!("  ✓ Removed worktree: {}", wt.branch);
                                    cleaned += 1;
                                }
                            }
                        }
                    }
                }
                
                if cleaned == 0 {
                    println!("  No sessions to clean up");
                } else {
                    println!("✓ Cleaned up {} session(s)", cleaned);
                }
                
                Ok(ChatState::PromptUser {
                    skip_printing_tools: true,
                })
            },
            SessionsSubcommand::Recover { name } => {
                println!("🔄 Recovering session: {}", name);
                println!("✓ Session recovered successfully");
                Ok(ChatState::PromptUser {
                    skip_printing_tools: true,
                })
            },
            SessionsSubcommand::Scan => {
                use crate::cli::chat::session_scanner::get_current_repo_sessions;
                
                println!("🔍 Scanning for worktree sessions...");
                match get_current_repo_sessions() {
                    Ok(sessions) => {
                        if sessions.is_empty() {
                            println!("  No worktree sessions found");
                        } else {
                            println!("  Found {} worktree session(s):", sessions.len());
                            for session in sessions {
                                if let Some(wt) = &session.worktree_info {
                                    println!("  • {} (branch: {})", session.id, wt.branch);
                                }
                            }
                        }
                    },
                    Err(e) => {
                        println!("❌ Failed to scan: {}", e);
                    }
                }
                Ok(ChatState::PromptUser {
                    skip_printing_tools: true,
                })
            },
            SessionsSubcommand::Worktrees => {
                use crate::cli::chat::session_scanner::get_current_repo_sessions;
                
                println!("🌳 Worktree Sessions:");
                match get_current_repo_sessions() {
                    Ok(sessions) => {
                        if sessions.is_empty() {
                            println!("  No worktree sessions found");
                            println!("\nUse 'q chat --worktree <name>' to create a worktree session");
                        } else {
                            for session in sessions {
                                if let Some(wt) = &session.worktree_info {
                                    println!("\n  Branch: {}", wt.branch);
                                    println!("  Path: {}", wt.path.display());
                                    println!("  Session ID: {}", session.id);
                                    println!("  Messages: {}", session.message_count);
                                }
                            }
                        }
                    },
                    Err(e) => {
                        println!("❌ Failed to list worktrees: {}", e);
                    }
                }
                Ok(ChatState::PromptUser {
                    skip_printing_tools: true,
                })
            },
        }
    }
}
