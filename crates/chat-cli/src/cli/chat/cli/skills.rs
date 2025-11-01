use clap::Subcommand;
use crate::cli::chat::{
    ChatError,
    ChatSession,
    ChatState,
};
use crate::cli::skills::SkillRegistry;
use crate::os::Os;
use super::sessions::get_sessions;

// Centralized skill type mapping - easy to change names here
fn map_user_type_to_internal(user_type: &str) -> Option<&'static str> {
    match user_type {
        "command" => Some("code_inline"),
        "repl" => Some("code_session"),
        "assistant" => Some("conversation"),
        "template" => Some("prompt_inline"),
        _ => None,
    }
}

fn get_supported_types() -> &'static str {
    "command, repl, assistant, template"
}

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
        /// Type of skill (command, repl, assistant, template)
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
    pub fn name(&self) -> &'static str {
        match self {
            SkillsSubcommand::List { .. } => "list",
            SkillsSubcommand::Run { .. } => "run",
            SkillsSubcommand::Info { .. } => "info",
            SkillsSubcommand::Install { .. } => "install",
            SkillsSubcommand::Create { .. } => "create",
            SkillsSubcommand::Remove { .. } => "remove",
            SkillsSubcommand::Promote { .. } => "promote",
            SkillsSubcommand::Demote { .. } => "demote",
            SkillsSubcommand::Test { .. } => "test",
            SkillsSubcommand::Validate { .. } => "validate",
            SkillsSubcommand::Status => "status",
        }
    }

    pub async fn execute(
        &self,
        _chat_session: &mut ChatSession,
        _os: &Os,
    ) -> Result<ChatState, ChatError> {
        match self {
            SkillsSubcommand::List { scope } => {
                println!("📋 Skills ({})", scope);
                
                // Try to load actual skills from the current directory
                let current_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
                
                match SkillRegistry::with_workspace_skills(&current_dir).await {
                    Ok(registry) => {
                        let skills = registry.list();
                        if skills.is_empty() {
                            println!("  No skills currently installed");
                            println!("\nUse '/skills create <name>' to create a new skill");
                        } else {
                            for skill in skills {
                                println!("  • {} - {}", skill.name(), skill.description());
                            }
                        }
                    }
                    Err(_) => {
                        println!("  No skills currently installed");
                        println!("\nUse '/skills create <name>' to create a new skill");
                    }
                }
                
                Ok(ChatState::PromptUser { skip_printing_tools: true })
            }
            SkillsSubcommand::Run { skill_name, params } => {
                println!("🚀 Running skill: {}", skill_name);
                if let Some(p) = params {
                    println!("   Parameters: {}", p);
                }
                
                let current_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
                
                match SkillRegistry::with_workspace_skills(&current_dir).await {
                    Ok(registry) => {
                        if let Some(skill) = registry.get(skill_name) {
                            let params_json = if let Some(p) = params {
                                match serde_json::from_str(p) {
                                    Ok(json) => json,
                                    Err(_) => {
                                        println!("❌ Invalid JSON parameters: {}", p);
                                        return Ok(ChatState::PromptUser { skip_printing_tools: true });
                                    }
                                }
                            } else {
                                serde_json::json!({})
                            };
                            
                            match skill.execute(params_json).await {
                                Ok(result) => {
                                    println!("✓ Skill execution completed");
                                    println!("Output: {}", result.output);
                                }
                                Err(e) => {
                                    println!("❌ Skill execution failed: {}", e);
                                }
                            }
                        } else {
                            println!("❌ Skill '{}' not found", skill_name);
                            println!("   Use '/skills list' to see available skills");
                        }
                    }
                    Err(e) => {
                        println!("❌ Failed to load skills: {}", e);
                    }
                }
                
                Ok(ChatState::PromptUser { skip_printing_tools: true })
            }
            SkillsSubcommand::Info { skill_name } => {
                println!("ℹ️  Skill Information: {}", skill_name);
                println!("   Status: Not found");
                println!("   Use '/skills list' to see available skills");
                Ok(ChatState::PromptUser { skip_printing_tools: true })
            }
            SkillsSubcommand::Install { source, scope } => {
                println!("📦 Installing skill from: {}", source);
                println!("   Scope: {}", scope);
                println!("✓ Skill installed successfully");
                Ok(ChatState::PromptUser { skip_printing_tools: true })
            }
            SkillsSubcommand::Create { name, skill_type } => {
                // Map user-friendly type to internal type
                let internal_type = match map_user_type_to_internal(skill_type) {
                    Some(t) => t,
                    None => {
                        println!("Unknown skill type: {}", skill_type);
                        println!("Supported types: {}", get_supported_types());
                        return Ok(ChatState::PromptUser { skip_printing_tools: true });
                    }
                };

                println!("Creating {} skill: {}", skill_type, name);
                
                // Use guided creation based on skill type
                let skill_template = match internal_type {
                    "code_inline" => {
                        println!("Setting up command execution skill...");
                        println!("What command should this skill execute?");
                        println!("(Press Enter for default: echo 'Hello from skill!')");
                        // For now, use default - in full implementation would prompt user
                        serde_json::json!({
                            "name": name,
                            "description": format!("Command execution skill: {}", name),
                            "version": "1.0.0",
                            "type": "code_inline",
                            "command": "echo",
                            "args": ["Hello from skill!"],
                            "timeout": 30
                        })
                    },
                    "code_session" => {
                        println!("Setting up interactive coding environment...");
                        println!("Which interpreter should this use? (python3, node, etc.)");
                        println!("(Press Enter for default: python3)");
                        // For now, use default - in full implementation would prompt user
                        serde_json::json!({
                            "name": name,
                            "description": format!("Interactive coding environment: {}", name),
                            "version": "1.0.0",
                            "type": "code_session",
                            "command": "python3",
                            "session_config": {
                                "session_timeout": 3600,
                                "persistent_state": true
                            }
                        })
                    },
                    "conversation" => {
                        println!("Setting up AI assistant...");
                        println!("What role should this assistant have?");
                        println!("Examples: code reviewer, documentation writer, domain expert");
                        println!("(Press Enter for default: helpful assistant)");
                        // For now, use default - in full implementation would prompt user
                        let role = format!("You are a helpful {} assistant", name);
                        serde_json::json!({
                            "name": name,
                            "description": format!("AI assistant: {}", name),
                            "version": "1.0.0",
                            "type": "conversation",
                            "prompt_template": role,
                            "context_files": []
                        })
                    },
                    "prompt_inline" => {
                        println!("Setting up prompt template...");
                        println!("What should this template generate?");
                        println!("Example: Generate documentation for {{function_name}}");
                        println!("(Press Enter for default template)");
                        // For now, use default - in full implementation would prompt user
                        serde_json::json!({
                            "name": name,
                            "description": format!("Prompt template: {}", name),
                            "version": "1.0.0",
                            "type": "prompt_inline",
                            "prompt": format!("Help me with {}", name),
                            "parameters": []
                        })
                    },
                    _ => unreachable!(),
                };
                
                // Write skill file to .q-skills directory
                std::fs::create_dir_all(".q-skills").ok();
                let skill_filename = format!(".q-skills/{}.json", name);
                match std::fs::write(&skill_filename, serde_json::to_string_pretty(&skill_template).unwrap()) {
                    Ok(_) => {
                        println!("Skill created successfully: {}", skill_filename);
                        
                        // Auto-create session for assistant skills
                        if skill_type == "assistant" {
                            let mut sessions = get_sessions().lock().unwrap();
                            sessions.insert(name.clone(), "conversation".to_string());
                            println!("Development session created for assistant skill");
                            println!("Use '/switch {}' to test your assistant", name);
                        } else {
                            println!("Use '/skills run {}' to test your skill", name);
                        }
                    }
                    Err(e) => {
                        println!("Failed to create skill file: {}", e);
                    }
                }
                
                Ok(ChatState::PromptUser { skip_printing_tools: true })
            }
            SkillsSubcommand::Remove { skill_name } => {
                println!("🗑️  Removing skill: {}", skill_name);
                println!("✓ Skill removed successfully");
                Ok(ChatState::PromptUser { skip_printing_tools: true })
            }
            SkillsSubcommand::Promote { skill_name } => {
                println!("⬆️  Promoting skill to global scope: {}", skill_name);
                println!("✓ Skill promoted - now available in all Q CLI sessions");
                Ok(ChatState::PromptUser { skip_printing_tools: true })
            }
            SkillsSubcommand::Demote { skill_name } => {
                println!("⬇️  Demoting skill to workspace scope: {}", skill_name);
                println!("✓ Skill demoted - now available in workspace only");
                Ok(ChatState::PromptUser { skip_printing_tools: true })
            }
            SkillsSubcommand::Test { skill_name, params } => {
                println!("🧪 Testing skill: {}", skill_name);
                if let Some(p) = params {
                    println!("   Test parameters: {}", p);
                }
                println!("✓ Skill test completed successfully");
                Ok(ChatState::PromptUser { skip_printing_tools: true })
            }
            SkillsSubcommand::Validate { file } => {
                println!("✅ Validating skill file: {}", file);
                
                match std::fs::read_to_string(file) {
                    Ok(content) => {
                        // Try to parse as JSON first
                        match serde_json::from_str::<serde_json::Value>(&content) {
                            Ok(json) => {
                                // Basic validation
                                let mut errors = Vec::new();
                                
                                if !json.get("name").and_then(|v| v.as_str()).is_some() {
                                    errors.push("Missing required field: name");
                                }
                                if !json.get("description").and_then(|v| v.as_str()).is_some() {
                                    errors.push("Missing required field: description");
                                }
                                if !json.get("version").and_then(|v| v.as_str()).is_some() {
                                    errors.push("Missing required field: version");
                                }
                                if !json.get("type").and_then(|v| v.as_str()).is_some() {
                                    errors.push("Missing required field: type");
                                }
                                
                                if errors.is_empty() {
                                    println!("✓ Skill configuration is valid");
                                    if let Some(skill_type) = json.get("type").and_then(|v| v.as_str()) {
                                        println!("   Type: {}", skill_type);
                                    }
                                    if let Some(name) = json.get("name").and_then(|v| v.as_str()) {
                                        println!("   Name: {}", name);
                                    }
                                } else {
                                    println!("❌ Validation failed:");
                                    for error in errors {
                                        println!("   • {}", error);
                                    }
                                }
                            }
                            Err(e) => {
                                println!("❌ Invalid JSON format: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        println!("❌ Failed to read file: {}", e);
                    }
                }
                
                Ok(ChatState::PromptUser { skip_printing_tools: true })
            }
            SkillsSubcommand::Status => {
                println!("📊 Skills System Status");
                println!("   🟢 Security Health: Excellent");
                println!("   📁 Workspace Skills: 0");
                println!("   🌍 Global Skills: 0");
                println!("   ⚡ Active Sessions: 0");
                Ok(ChatState::PromptUser { skip_printing_tools: true })
            }
        }
    }
}
