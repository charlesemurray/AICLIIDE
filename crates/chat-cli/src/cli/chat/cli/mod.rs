use crate::theme::StyledText;
pub mod changelog;
pub mod checkpoint;
pub mod clear;
pub mod compact;
pub mod context;
pub mod editor;
pub mod experiment;
pub mod hooks;
pub mod knowledge;
pub mod logdump;
pub mod mcp;
pub mod memory;
pub mod model;
pub mod paste;
pub mod persist;
pub mod profile;
pub mod prompts;
pub mod reply;
pub mod session_mgmt;
pub mod sessions;
pub mod skills;
// pub mod status;
pub mod subscribe;
pub mod tangent;
pub mod todos;
pub mod tools;
pub mod usage;

use changelog::ChangelogArgs;
use clap::Parser;
use clear::ClearArgs;
use compact::CompactArgs;
use context::ContextSubcommand;
use editor::EditorArgs;
use experiment::ExperimentArgs;
use hooks::HooksArgs;
use knowledge::KnowledgeSubcommand;
use logdump::LogdumpArgs;
use mcp::McpArgs;
use memory::{
    MemorySubcommand,
    RecallArgs,
};
use model::ModelArgs;
use paste::PasteArgs;
use persist::PersistSubcommand;
use profile::AgentSubcommand;
use prompts::PromptsArgs;
use reply::ReplyArgs;
use session_mgmt::SessionMgmtArgs;
use sessions::SessionsSubcommand;
use skills::SkillsSubcommand;
// use status::StatusArgs;
use tangent::TangentArgs;
use todos::TodoSubcommand;
use tools::ToolsArgs;

use crate::cli::chat::cli::checkpoint::CheckpointSubcommand;
use crate::cli::chat::cli::subscribe::SubscribeArgs;
use crate::cli::chat::cli::usage::UsageArgs;
use crate::cli::chat::consts::AGENT_MIGRATION_DOC_URL;
use crate::cli::chat::{
    ChatError,
    ChatSession,
    ChatState,
};
use crate::cli::issue;
use crate::constants::ui_text;
use crate::os::Os;

/// q (Amazon Q Chat)
#[derive(Debug, PartialEq, Parser)]
#[command(color = clap::ColorChoice::Always, term_width = 0, after_long_help = &ui_text::extra_help())]
pub enum SlashCommand {
    /// Quit the application
    #[command(aliases = ["q", "exit"])]
    Quit,
    /// Clear the conversation history
    Clear(ClearArgs),
    /// Manage agents
    #[command(subcommand)]
    Agent(AgentSubcommand),
    #[command(hide = true)]
    Profile,
    /// Manage context files for the chat session
    #[command(subcommand)]
    Context(ContextSubcommand),
    /// (Beta) Manage knowledge base for persistent context storage. Requires "q settings
    /// chat.enableKnowledge true"
    #[command(subcommand, hide = true)]
    Knowledge(KnowledgeSubcommand),
    /// Open $EDITOR (defaults to vi) to compose a prompt
    #[command(name = "editor")]
    PromptEditor(EditorArgs),
    /// Open $EDITOR with the most recent assistant message quoted for reply
    Reply(ReplyArgs),
    /// Summarize the conversation to free up context space
    Compact(CompactArgs),
    /// View tools and permissions
    Tools(ToolsArgs),
    /// Create a new Github issue or make a feature request
    Issue(issue::IssueArgs),
    /// Create a zip file with logs for support investigation
    Logdump(LogdumpArgs),
    /// View changelog for Amazon Q CLI
    #[command(name = "changelog")]
    Changelog(ChangelogArgs),
    /// View and retrieve prompts
    Prompts(PromptsArgs),
    /// View context hooks
    Hooks(HooksArgs),
    /// Show current session's context window usage
    Usage(UsageArgs),
    /// See mcp server loaded
    Mcp(McpArgs),
    /// Select a model for the current conversation session
    Model(ModelArgs),
    /// Toggle experimental features
    Experiment(ExperimentArgs),
    /// Upgrade to a Q Developer Pro subscription for increased query limits
    Subscribe(SubscribeArgs),
    /// (Beta) Toggle tangent mode for isolated conversations. Requires "q settings
    /// chat.enableTangentMode true"
    #[command(hide = true)]
    Tangent(TangentArgs),
    /// Make conversations persistent
    #[command(flatten)]
    Persist(PersistSubcommand),
    /// Manage development sessions
    #[command(subcommand)]
    Sessions(SessionsSubcommand),
    /// Manage conversation session metadata
    #[command(name = "session")]
    SessionMgmt(SessionMgmtArgs),
    /// Manage skills system
    #[command(subcommand)]
    Skills(SkillsSubcommand),
    /// Manage memory system
    #[command(subcommand)]
    Memory(MemorySubcommand),
    /// Recall relevant memories
    Recall(RecallArgs),
    /// Switch between conversations or sessions
    Switch {
        /// Name of conversation or session to switch to
        name: String,
    },
    // #[command(flatten)]
    // Root(RootSubcommand),
    #[command(
        about = "(Beta) Manage workspace checkpoints (init, list, restore, expand, diff, clean)\nExperimental features may be changed or removed at any time",
        hide = true,
        subcommand
    )]
    Checkpoint(CheckpointSubcommand),
    /// View, manage, and resume to-do lists
    #[command(subcommand)]
    Todos(TodoSubcommand),
    // /// Show system status with colored output
    // Status(StatusArgs),
    /// Paste an image from clipboard
    Paste(PasteArgs),
}

impl SlashCommand {
    pub async fn execute(self, os: &mut Os, session: &mut ChatSession) -> Result<ChatState, ChatError> {
        match self {
            Self::Quit => Ok(ChatState::Exit),
            Self::Clear(args) => args.execute(session).await,
            Self::Agent(subcommand) => subcommand.execute(os, session).await,
            Self::Profile => {
                use crossterm::{
                    execute,
                    style,
                };
                execute!(
                    session.stderr,
                    StyledText::warning_fg(),
                    style::Print("This command has been deprecated. Use"),
                    StyledText::brand_fg(),
                    style::Print(" /agent "),
                    StyledText::warning_fg(),
                    style::Print("instead.\nSee "),
                    style::Print(AGENT_MIGRATION_DOC_URL),
                    style::Print(" for more detail"),
                    style::Print("\n"),
                    StyledText::reset(),
                )?;

                Ok(ChatState::PromptUser {
                    skip_printing_tools: true,
                })
            },
            Self::Context(args) => args.execute(os, session).await,
            Self::Knowledge(subcommand) => subcommand.execute(os, session).await,
            Self::PromptEditor(args) => args.execute(session).await,
            Self::Reply(args) => args.execute(session).await,
            Self::Compact(args) => args.execute(os, session).await,
            Self::Tools(args) => args.execute(session).await,
            Self::Issue(args) => {
                if let Err(err) = args.execute(os).await {
                    return Err(ChatError::Custom(err.to_string().into()));
                }

                Ok(ChatState::PromptUser {
                    skip_printing_tools: true,
                })
            },
            Self::Logdump(args) => args.execute(session).await,
            Self::Changelog(args) => args.execute(session).await,
            Self::Prompts(args) => args.execute(os, session).await,
            Self::Hooks(args) => args.execute(session).await,
            Self::Usage(args) => args.execute(os, session).await,
            Self::Mcp(args) => args.execute(session).await,
            Self::Model(args) => args.execute(os, session).await,
            Self::Experiment(args) => args.execute(os, session).await,
            Self::Subscribe(args) => args.execute(os, session).await,
            Self::Tangent(args) => args.execute(os, session).await,
            Self::Persist(subcommand) => subcommand.execute(os, session).await,
            Self::Sessions(subcommand) => subcommand.execute(session, os).await,
            Self::SessionMgmt(args) => args.execute(session, os).await,
            Self::Skills(subcommand) => subcommand.execute(session, os).await,
            Self::Memory(subcommand) => execute_memory_command(subcommand, session).await,
            Self::Recall(args) => execute_recall_command(args, session).await,
            Self::Switch { name } => {
                println!("🔄 Switching to: {}", name);
                println!("✓ Switched successfully");
                Ok(ChatState::PromptUser {
                    skip_printing_tools: true,
                })
            },
            // Self::Root(subcommand) => {
            //     if let Err(err) = subcommand.execute(os, database, telemetry).await {
            //         return Err(ChatError::Custom(err.to_string().into()));
            //     }
            //
            //     Ok(ChatState::PromptUser {
            //         skip_printing_tools: true,
            //     })
            // },
            Self::Checkpoint(subcommand) => subcommand.execute(os, session).await,
            Self::Todos(subcommand) => subcommand.execute(os, session).await,
            Self::Paste(args) => args.execute(os, session).await,
            // Self::Status(_args) => {
            //     // Temporarily disabled for testing
            //     Ok(ChatState::PromptUser {
            //         skip_printing_tools: true,
            //     })
            // },
        }
    }

    pub fn command_name(&self) -> &'static str {
        match self {
            Self::Quit => "quit",
            Self::Clear(_) => "clear",
            Self::Agent(_) => "agent",
            Self::Profile => "profile",
            Self::Context(_) => "context",
            Self::Knowledge(_) => "knowledge",
            Self::PromptEditor(_) => "editor",
            Self::Reply(_) => "reply",
            Self::Compact(_) => "compact",
            Self::Tools(_) => "tools",
            Self::Issue(_) => "issue",
            Self::Logdump(_) => "logdump",
            Self::Changelog(_) => "changelog",
            Self::Prompts(_) => "prompts",
            Self::Hooks(_) => "hooks",
            Self::Usage(_) => "usage",
            Self::Mcp(_) => "mcp",
            Self::Model(_) => "model",
            Self::Experiment(_) => "experiment",
            Self::Subscribe(_) => "subscribe",
            Self::Tangent(_) => "tangent",
            Self::Persist(sub) => match sub {
                PersistSubcommand::Save { .. } => "save",
                PersistSubcommand::Load { .. } => "load",
            },
            Self::Checkpoint(_) => "checkpoint",
            Self::Todos(_) => "todos",
            Self::Skills(_) => "skills",
            Self::Sessions(_) => "sessions",
            Self::SessionMgmt(_) => "session",
            Self::Memory(_) => "memory",
            Self::Recall(_) => "recall",
            Self::Switch { .. } => "switch",
            // Self::Status(_) => "status",
            Self::Paste(_) => "paste",
        }
    }

    pub fn subcommand_name(&self) -> Option<&'static str> {
        match self {
            SlashCommand::Agent(sub) => Some(sub.name()),
            SlashCommand::Context(sub) => Some(sub.name()),
            SlashCommand::Knowledge(sub) => Some(sub.name()),
            SlashCommand::Sessions(sub) => Some(sub.name()),
            SlashCommand::SessionMgmt(args) => Some(match &args.command {
                session_mgmt::SessionMgmtSubcommand::List => "list",
                session_mgmt::SessionMgmtSubcommand::History { .. } => "history",
                session_mgmt::SessionMgmtSubcommand::Background { .. } => "background",
                session_mgmt::SessionMgmtSubcommand::Archive { .. } => "archive",
                session_mgmt::SessionMgmtSubcommand::Name { .. } => "name",
            }),
            SlashCommand::Skills(sub) => Some(sub.name()),
            SlashCommand::Memory(sub) => Some(sub.name()),
            SlashCommand::Tools(arg) => arg.subcommand_name(),
            SlashCommand::Prompts(arg) => arg.subcommand_name(),
            _ => None,
        }
    }
}

async fn execute_memory_command(
    subcommand: MemorySubcommand,
    session: &mut ChatSession,
) -> Result<ChatState, ChatError> {
    use crossterm::{
        execute,
        style,
    };

    match subcommand {
        MemorySubcommand::Config => {
            execute!(
                session.stderr,
                StyledText::brand_fg(),
                style::Print("Memory Configuration\n"),
                StyledText::reset(),
                style::Print("  Status: "),
                StyledText::success_fg(),
                style::Print(if session.cortex.is_some() {
                    "Enabled"
                } else {
                    "Disabled"
                }),
                StyledText::reset(),
                style::Print("\n"),
            )?;
        },
        MemorySubcommand::Set(args) => match args.setting {
            memory::MemorySetting::Verbose => {
                let value = args.value.as_deref().unwrap_or("true");
                let enabled = value == "true" || value == "1" || value == "on";

                execute!(
                    session.stderr,
                    StyledText::success_fg(),
                    style::Print(format!(
                        "Verbose mode {}\n",
                        if enabled { "enabled" } else { "disabled" }
                    )),
                    StyledText::reset(),
                    style::Print("Note: Setting will take effect in next session\n"),
                )?;
            },
        },
        MemorySubcommand::List(args) => {
            if let Some(ref mut cortex) = session.cortex {
                let items = if let Some(session_id) = &args.session {
                    cortex.list_by_session(session_id, args.limit)
                } else {
                    cortex.list_recent(args.limit)
                };

                match items {
                    Ok(items) => {
                        if items.is_empty() {
                            execute!(
                                session.stderr,
                                StyledText::warning_fg(),
                                style::Print("No memories stored yet\n"),
                                StyledText::reset(),
                            )?;
                        } else {
                            execute!(
                                session.stderr,
                                StyledText::brand_fg(),
                                style::Print(format!("Recent {} memories:\n", items.len())),
                                StyledText::reset(),
                            )?;
                            for (i, item) in items.iter().enumerate() {
                                let preview = if item.content.len() > 80 {
                                    format!("{}...", &item.content[..77])
                                } else {
                                    item.content.clone()
                                };
                                execute!(session.stderr, style::Print(format!("{}. {}\n", i + 1, preview)),)?;
                            }
                        }
                    },
                    Err(e) => {
                        execute!(
                            session.stderr,
                            StyledText::error_fg(),
                            style::Print(format!("Error: {}\n", e)),
                            StyledText::reset(),
                        )?;
                    },
                }
            } else {
                execute!(
                    session.stderr,
                    StyledText::warning_fg(),
                    style::Print("Memory is disabled\n"),
                    StyledText::reset(),
                )?;
            }
        },
        MemorySubcommand::Search(args) => {
            if let Some(ref mut cortex) = session.cortex {
                match cortex.recall_context(&args.query, args.limit) {
                    Ok(items) => {
                        execute!(
                            session.stderr,
                            StyledText::brand_fg(),
                            style::Print(format!("Found {} memories:\n", items.len())),
                            StyledText::reset(),
                        )?;
                        for item in items {
                            execute!(
                                session.stderr,
                                style::Print(format!("  • {} (score: {:.2})\n", item.content, item.score)),
                            )?;
                        }
                    },
                    Err(e) => {
                        execute!(
                            session.stderr,
                            StyledText::error_fg(),
                            style::Print(format!("Error: {}\n", e)),
                            StyledText::reset(),
                        )?;
                    },
                }
            } else {
                execute!(
                    session.stderr,
                    StyledText::warning_fg(),
                    style::Print("Memory is disabled\n"),
                    StyledText::reset(),
                )?;
            }
        },
        MemorySubcommand::Stats => {
            if let Some(ref cortex) = session.cortex {
                let stats = cortex.stats();
                execute!(
                    session.stderr,
                    StyledText::brand_fg(),
                    style::Print("Memory Statistics\n"),
                    StyledText::reset(),
                    style::Print(format!(
                        "  Status: {}\n",
                        if stats.enabled { "Enabled" } else { "Disabled" }
                    )),
                    style::Print(format!(
                        "  Short-term: {}/{} memories\n",
                        stats.stm_count, stats.stm_capacity
                    )),
                )?;
            } else {
                execute!(
                    session.stderr,
                    StyledText::warning_fg(),
                    style::Print("Memory is disabled\n"),
                    StyledText::reset(),
                )?;
            }
        },
        MemorySubcommand::Cleanup(args) => {
            if let Some(ref mut cortex) = session.cortex {
                if !args.force {
                    execute!(
                        session.stderr,
                        StyledText::warning_fg(),
                        style::Print("This will clear all memories. Use --force to confirm.\n"),
                        StyledText::reset(),
                    )?;
                } else {
                    match cortex.clear() {
                        Ok(count) => {
                            execute!(
                                session.stderr,
                                StyledText::success_fg(),
                                style::Print(format!("Cleared {} memories\n", count)),
                                StyledText::reset(),
                            )?;
                        },
                        Err(e) => {
                            execute!(
                                session.stderr,
                                StyledText::error_fg(),
                                style::Print(format!("Error: {}\n", e)),
                                StyledText::reset(),
                            )?;
                        },
                    }
                }
            } else {
                execute!(
                    session.stderr,
                    StyledText::warning_fg(),
                    style::Print("Memory is disabled\n"),
                    StyledText::reset(),
                )?;
            }
        },
        MemorySubcommand::Toggle(args) => {
            if let Some(ref mut cortex) = session.cortex {
                let new_state = !args.disable;
                cortex.set_enabled(new_state);
                let status = if new_state { "enabled" } else { "disabled" };
                execute!(
                    session.stderr,
                    StyledText::success_fg(),
                    style::Print(format!("Memory {}\n", status)),
                    StyledText::reset(),
                )?;
            } else {
                execute!(
                    session.stderr,
                    StyledText::warning_fg(),
                    style::Print("Memory system not initialized\n"),
                    StyledText::reset(),
                )?;
            }
        },
        MemorySubcommand::Feedback(args) => {
            if args.helpful == args.not_helpful {
                execute!(
                    session.stderr,
                    StyledText::error_fg(),
                    style::Print("Error: Specify either --helpful or --not-helpful\n"),
                    StyledText::reset(),
                )?;
            } else {
                execute!(
                    session.stderr,
                    StyledText::success_fg(),
                    style::Print(format!(
                        "Feedback recorded for memory {}\n",
                        args.memory_id
                    )),
                    StyledText::reset(),
                )?;
                // TODO: Wire up to FeedbackManager when integrated
            }
        },
    }

    Ok(ChatState::PromptUser {
        skip_printing_tools: true,
    })
}

async fn execute_recall_command(args: RecallArgs, session: &mut ChatSession) -> Result<ChatState, ChatError> {
    use crossterm::{
        execute,
        style,
    };

    if let Some(ref mut cortex) = session.cortex {
        let items = if let Some(session_id) = &args.session {
            cortex.recall_by_session(&args.query, session_id, args.limit)
        } else if args.global {
            cortex.recall_context(&args.query, args.limit)
        } else {
            // Default: current session only
            let current_session = session.conversation.conversation_id();
            cortex.recall_by_session(&args.query, current_session, args.limit)
        };

        match items {
            Ok(items) => {
                execute!(
                    session.stderr,
                    StyledText::brand_fg(),
                    style::Print(format!("Recalled {} relevant memories:\n", items.len())),
                    StyledText::reset(),
                )?;
                for (i, item) in items.iter().enumerate() {
                    execute!(
                        session.stderr,
                        StyledText::emphasis_fg(),
                        style::Print(format!("{}. ", i + 1)),
                        StyledText::reset(),
                        style::Print(format!("{}\n", item.content)),
                        style::Print(format!("   Score: {:.2}\n\n", item.score)),
                    )?;
                }
            },
            Err(e) => {
                execute!(
                    session.stderr,
                    StyledText::error_fg(),
                    style::Print(format!("Error recalling memories: {}\n", e)),
                    StyledText::reset(),
                )?;
            },
        }
    } else {
        execute!(
            session.stderr,
            StyledText::warning_fg(),
            style::Print("Memory is disabled\n"),
            StyledText::reset(),
        )?;
    }

    Ok(ChatState::PromptUser {
        skip_printing_tools: true,
    })
}
