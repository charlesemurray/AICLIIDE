use clap::{
    Args,
    Subcommand,
};

#[derive(Debug, Clone, PartialEq, Subcommand)]
pub enum MemorySubcommand {
    /// Show memory configuration
    Config,
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
}

impl MemorySubcommand {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Config => "config",
            Self::List(_) => "list",
            Self::Search(_) => "search",
            Self::Stats => "stats",
            Self::Cleanup(_) => "cleanup",
            Self::Toggle(_) => "toggle",
        }
    }
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
