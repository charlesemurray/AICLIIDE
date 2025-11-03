use clap::{Args, Subcommand};

#[derive(Debug, Clone, Subcommand)]
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

#[derive(Debug, Clone, Args)]
pub struct ListArgs {
    /// Maximum number of memories to show
    #[arg(long, default_value = "10")]
    pub limit: usize,
    
    /// Filter by session ID
    #[arg(long)]
    pub session: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub struct SearchArgs {
    /// Search query
    pub query: String,
    
    /// Maximum number of results
    #[arg(long, default_value = "5")]
    pub limit: usize,
}

#[derive(Debug, Clone, Args)]
pub struct CleanupArgs {
    /// Skip confirmation prompt
    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, Clone, Args)]
pub struct ToggleArgs {
    /// Disable memory
    #[arg(long)]
    pub disable: bool,
}

#[derive(Debug, Clone, Args)]
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
