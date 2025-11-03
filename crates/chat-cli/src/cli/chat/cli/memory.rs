use clap::{
    Args,
    Subcommand,
};

#[derive(Debug, Clone, PartialEq, Subcommand)]
pub enum MemorySubcommand {
    /// Show memory configuration
    Config,
    /// Update memory settings
    Set(SetArgs),
    /// List stored memories
    List(ListArgs),
    /// Search memories
    Search(SearchArgs),
    /// Show memory statistics
    Stats,
    /// Clean up old memories
    Cleanup(CleanupArgs),
    /// Toggle memory on/off
    Toggle(ToggleArgs),
    /// Provide feedback on a memory
    Feedback(FeedbackArgs),
}

impl MemorySubcommand {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Config => "config",
            Self::Set(_) => "set",
            Self::List(_) => "list",
            Self::Search(_) => "search",
            Self::Stats => "stats",
            Self::Cleanup(_) => "cleanup",
            Self::Toggle(_) => "toggle",
            Self::Feedback(_) => "feedback",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Args)]
pub struct SetArgs {
    /// Setting to update
    #[arg(value_enum)]
    pub setting: MemorySetting,

    /// Value to set (for boolean settings, omit for true)
    pub value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, clap::ValueEnum)]
pub enum MemorySetting {
    /// Enable/disable verbose mode
    Verbose,
}

#[derive(Debug, Clone, PartialEq, Args)]
pub struct ListArgs {
    /// Maximum number of memories to show
    #[arg(long, default_value = "10")]
    pub limit: usize,

    /// Filter by session ID
    #[arg(long)]
    pub session: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Args)]
pub struct SearchArgs {
    /// Search query
    pub query: String,

    /// Maximum number of results
    #[arg(long, default_value = "5")]
    pub limit: usize,
}

#[derive(Debug, Clone, PartialEq, Args)]
pub struct CleanupArgs {
    /// Skip confirmation prompt
    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, Clone, PartialEq, Args)]
pub struct ToggleArgs {
    /// Disable memory
    #[arg(long)]
    pub disable: bool,
}

#[derive(Debug, Clone, PartialEq, Args)]
pub struct FeedbackArgs {
    /// Memory ID to provide feedback on
    pub memory_id: String,

    /// Mark as helpful
    #[arg(long)]
    pub helpful: bool,

    /// Mark as not helpful
    #[arg(long)]
    pub not_helpful: bool,
}

#[derive(Debug, Clone, PartialEq, Args)]
pub struct RecallArgs {
    /// Query to recall memories for
    pub query: String,

    /// Search across all sessions
    #[arg(long)]
    pub global: bool,

    /// Specific session to search
    #[arg(long)]
    pub session: Option<String>,

    /// Maximum number of results
    #[arg(long, default_value = "5")]
    pub limit: usize,
}
