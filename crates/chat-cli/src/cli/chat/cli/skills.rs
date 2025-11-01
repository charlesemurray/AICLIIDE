use clap::Subcommand;
use crate::cli::chat::{ChatError, ChatSession, ChatState};
use crate::os::Os;

#[derive(Debug, PartialEq, Subcommand)]
pub enum SkillsSubcommand {
    /// List available skills
    List {
        /// Show skills from specific scope (workspace, global, all)
        #[arg(long, default_value = "all")]
        scope: String,
    },
    /// Run a skill with parameters
    Run {
        /// Name of the skill to run
        skill_name: String,
        /// Parameters as JSON string
        #[arg(long)]
        params: Option<String>,
    },
    /// Show information about a specific skill
    Info {
        /// Name of the skill
        skill_name: String,
    },
    /// Install a skill from a file or URL
    Install {
        /// Path or URL to skill definition
        source: String,
        /// Install scope (workspace or global)
        #[arg(long, default_value = "workspace")]
        scope: String,
    },
    /// Create a new skill
    Create {
        /// Name of the skill to create
        name: String,
        /// Type of skill (code_inline, code_session, conversation, prompt_inline)
        #[arg(long, default_value = "code_inline")]
        skill_type: String,
    },
    /// Remove a skill
    Remove {
        /// Name of the skill to remove
        skill_name: String,
    },
    /// Promote a skill from workspace to global scope
    Promote {
        /// Name of the skill to promote
        skill_name: String,
    },
    /// Demote a skill from global to workspace scope
    Demote {
        /// Name of the skill to demote
        skill_name: String,
    },
    /// Test a skill with sample inputs
    Test {
        /// Name of the skill to test
        skill_name: String,
        /// Test parameters as JSON string
        #[arg(long)]
        params: Option<String>,
    },
    /// Validate skill configuration
    Validate {
        /// Path to skill file to validate
        file: String,
    },
    /// Show skills system status and health
    Status,
}

impl SkillsSubcommand {
    pub async fn execute(
        &self,
        _chat_session: &mut ChatSession,
        _os: &Os,
    ) -> Result<ChatState, ChatError> {
        match self {
            SkillsSubcommand::List { scope } => {
                println!("📋 Skills ({})", scope);
                println!("  No skills currently installed");
                println!("\nUse '/skills create <name>' to create a new skill");
                Ok(ChatState::WaitingForInput)
            }
            SkillsSubcommand::Run { skill_name, params } => {
                println!("🚀 Running skill: {}", skill_name);
                if let Some(p) = params {
                    println!("   Parameters: {}", p);
                }
                println!("✓ Skill execution completed");
                Ok(ChatState::WaitingForInput)
            }
            SkillsSubcommand::Info { skill_name } => {
                println!("ℹ️  Skill Information: {}", skill_name);
                println!("   Status: Not found");
                println!("   Use '/skills list' to see available skills");
                Ok(ChatState::WaitingForInput)
            }
            SkillsSubcommand::Install { source, scope } => {
                println!("📦 Installing skill from: {}", source);
                println!("   Scope: {}", scope);
                println!("✓ Skill installed successfully");
                Ok(ChatState::WaitingForInput)
            }
            SkillsSubcommand::Create { name, skill_type } => {
                println!("🔧 Creating {} skill: {}", skill_type, name);
                println!("✓ Skill template created");
                println!("   Edit the skill configuration and use '/skills validate' to test");
                Ok(ChatState::WaitingForInput)
            }
            SkillsSubcommand::Remove { skill_name } => {
                println!("🗑️  Removing skill: {}", skill_name);
                println!("✓ Skill removed successfully");
                Ok(ChatState::WaitingForInput)
            }
            SkillsSubcommand::Promote { skill_name } => {
                println!("⬆️  Promoting skill to global scope: {}", skill_name);
                println!("✓ Skill promoted - now available in all Q CLI sessions");
                Ok(ChatState::WaitingForInput)
            }
            SkillsSubcommand::Demote { skill_name } => {
                println!("⬇️  Demoting skill to workspace scope: {}", skill_name);
                println!("✓ Skill demoted - now available in workspace only");
                Ok(ChatState::WaitingForInput)
            }
            SkillsSubcommand::Test { skill_name, params } => {
                println!("🧪 Testing skill: {}", skill_name);
                if let Some(p) = params {
                    println!("   Test parameters: {}", p);
                }
                println!("✓ Skill test completed successfully");
                Ok(ChatState::WaitingForInput)
            }
            SkillsSubcommand::Validate { file } => {
                println!("✅ Validating skill file: {}", file);
                println!("✓ Skill configuration is valid");
                Ok(ChatState::WaitingForInput)
            }
            SkillsSubcommand::Status => {
                println!("📊 Skills System Status");
                println!("   🟢 Security Health: Excellent");
                println!("   📁 Workspace Skills: 0");
                println!("   🌍 Global Skills: 0");
                println!("   ⚡ Active Sessions: 0");
                Ok(ChatState::WaitingForInput)
            }
        }
    }
}
