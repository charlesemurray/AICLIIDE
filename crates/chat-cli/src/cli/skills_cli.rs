use std::fs;
use std::io::{
    self,
    Write,
};
use std::process::ExitCode;

use clap::{
    Args,
    Subcommand,
};
use eyre::Result;
use serde_json::json;

use crate::cli::skills::validation::SkillValidator;
use crate::cli::skills::{
    SkillError,
    SkillRegistry,
};
use crate::os::Os;

/// Error types for skills CLI operations
mod error {
    use std::fmt;

    #[derive(Debug)]
    pub enum SkillsCliError {
        SkillNotFound(String),
        InvalidInput(String),
        InvalidTemplate(String),
        FileNotFound(String),
        ValidationFailed(String),
        ExecutionFailed(String),
        IoError(std::io::Error),
        SerializationError(serde_json::Error),
        HomeDirectoryNotFound,
    }

    impl fmt::Display for SkillsCliError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::SkillNotFound(name) => write!(f, "Skill '{}' not found", name),
                Self::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
                Self::InvalidTemplate(name) => write!(f, "Invalid template: {}", name),
                Self::FileNotFound(path) => write!(f, "File not found: {}", path),
                Self::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
                Self::ExecutionFailed(msg) => write!(f, "Execution failed: {}", msg),
                Self::IoError(e) => write!(f, "IO error: {}", e),
                Self::SerializationError(e) => write!(f, "Serialization error: {}", e),
                Self::HomeDirectoryNotFound => write!(f, "Could not find home directory"),
            }
        }
    }

    impl std::error::Error for SkillsCliError {}

    impl From<std::io::Error> for SkillsCliError {
        fn from(e: std::io::Error) -> Self {
            Self::IoError(e)
        }
    }

    impl From<serde_json::Error> for SkillsCliError {
        fn from(e: serde_json::Error) -> Self {
            Self::SerializationError(e)
        }
    }
}

/// Constants for the skills CLI
mod constants {
    /// Directory name for skills in workspace
    pub const SKILLS_DIR_NAME: &str = ".q-skills";

    /// Directory name for skills in home directory
    pub const HOME_SKILLS_DIR_NAME: &str = ".q-skills";

    /// File extension for skill files
    pub const SKILL_FILE_EXTENSION: &str = "json";

    /// Template names
    pub mod templates {
        pub const COMMAND: &str = "command";
        pub const SCRIPT: &str = "script";
        pub const HTTP_API: &str = "http-api";
        pub const FILE_PROCESSOR: &str = "file-processor";
    }

    /// User-facing messages
    pub mod messages {
        pub const NO_SKILLS_FOUND: &str = "No skills found.";
        pub const TIP_TRY_EXAMPLE: &str = "💡 Try: q skills example";
        pub const TIP_GET_DETAILS: &str = "💡 Get details: q skills info <name>";
        pub const TIP_USE_NATURAL_LANGUAGE: &str = "💡 Tip: Use natural language in chat";
        pub const TIP_SKILLS_LOCATION: &str = "💡 Tip: Skills are in ~/.q-skills/";
        pub const AVAILABLE_SKILLS_HEADER: &str = "Available Skills:";
        pub const SKILL_CREATED: &str = "✓ Created skill:";
        pub const SKILL_VALID: &str = "✓ Skill file is valid";
        pub const VALIDATION_FAILED: &str = "✗ Validation failed:";
        pub const UNKNOWN_TEMPLATE: &str = "Unknown template:";
        pub const AVAILABLE_TEMPLATES: &str = "Available templates:";
        pub const INVALID_TEMPLATE_ERROR: &str = "Invalid template";
        pub const COULD_NOT_FIND_HOME: &str = "Could not find home directory";
        pub const NO_SKILLS_DIR_FOUND: &str = "No skills directory found at";
        pub const SKILL_NOT_FOUND: &str = "Skill '{}' not found";
        pub const REMOVE_CONFIRM_PROMPT: &str = "Remove skill '{}'? (y/N): ";
        pub const CANCELLED: &str = "Cancelled";
        pub const SKILL_REMOVED: &str = "✓ Removed skill:";
        pub const SKILL_EXECUTION_FAILED: &str = "Skill execution failed";
        pub const VALIDATION_FAILED_ERROR: &str = "Validation failed";
        pub const FAILED_TO_READ_FILE: &str = "Failed to read file:";
        pub const INVALID_JSON_FOR_SKILL: &str = "Invalid JSON for skill '{}':";
    }

    /// Help text
    pub mod help {
        pub const HEADER: &str = "Q Skills Help";
        pub const COMMANDS_HEADER: &str = "Commands:";
        pub const CMD_LIST: &str = "  q skills list       List all skills";
        pub const CMD_INFO: &str = "  q skills info <name>  Show skill details";
        pub const CMD_RUN: &str = "  q skills run <name>   Run a skill";
        pub const CMD_EXAMPLE: &str = "  q skills example    Interactive creation";
        pub const CMD_HELP: &str = "  q skills help       Show this help";
    }

    /// Example text
    pub mod example {
        pub const HEADER: &str = "Interactive skill creation example:";
        pub const STEP_1: &str = "1. Create a simple command skill:";
        pub const STEP_1_CMD: &str = "   q skills create hello --from-template command";
        pub const STEP_2: &str = "2. View available templates:";
        pub const STEP_2_CMD: &str = "   q skills create --help";
        pub const STEP_3: &str = "3. See examples in: examples/skills/";
    }
}

// Map user-friendly names to internal types
fn map_user_type_to_internal(user_type: &str) -> &str {
    match user_type {
        "command" => "code_inline",
        "repl" => "code_session",
        "assistant" => "conversation",
        "template" => "prompt_inline",
        // Pass through internal types for backward compatibility
        internal => internal,
    }
}

#[derive(Debug, Args, PartialEq)]
pub struct SkillsArgs {
    #[command(subcommand)]
    pub command: SkillsCommand,
}

#[derive(Debug, Subcommand, PartialEq)]
pub enum SkillsCommand {
    /// List available skills
    List,
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
    /// Show help and examples
    Help,
    /// Run interactive skill creation example
    Example,
    /// Validate a skill file
    Validate {
        /// Path to skill file
        file: std::path::PathBuf,
    },
    /// Install a skill from a file or URL
    Install {
        /// Path or URL to skill definition
        source: String,
    },
    /// Create a new skill
    Create {
        /// Name of the skill to create
        name: String,
        /// Template to use (command, script, http-api, file-processor)
        #[arg(long)]
        from_template: Option<String>,
        /// Description of the skill
        #[arg(long, short)]
        description: Option<String>,
        /// Type of skill to create (code_inline, code_session, conversation, prompt_inline, rust)
        #[arg(long, short)]
        skill_type: Option<String>,
        /// Interactive mode for guided creation
        #[arg(long, short)]
        interactive: bool,
        /// Wizard mode for step-by-step creation
        #[arg(long, short)]
        wizard: bool,
        /// Quick creation with minimal prompts
        #[arg(long, short)]
        quick: bool,
        /// Command to execute (for quick code_inline creation)
        #[arg(long)]
        command: Option<String>,
        /// Template text (for quick prompt_inline creation)
        #[arg(long)]
        template: Option<String>,
    },
    /// Remove a skill
    Remove {
        /// Name of the skill to remove
        skill_name: String,
    },
}

// Separate enum for slash commands
#[derive(Debug, Subcommand, PartialEq)]
pub enum SkillsSlashCommand {
    /// List available skills
    List,
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
}

impl SkillsArgs {
    pub async fn execute(self, _os: &mut Os) -> Result<ExitCode> {
        let current_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let registry = SkillRegistry::with_workspace_skills(&current_dir)
            .await
            .unwrap_or_else(|_| SkillRegistry::with_builtins());

        match self.command {
            SkillsCommand::List => {
                // Show tutorial on first run
                use crate::cli::skills::onboarding;
                let _ = onboarding::show_tutorial_if_needed(&mut std::io::stdout());

                let skills = registry.list();

                if skills.is_empty() {
                    println!("{}\n", constants::messages::NO_SKILLS_FOUND);
                    println!("{}", constants::messages::TIP_TRY_EXAMPLE);
                    return Ok(ExitCode::SUCCESS);
                }

                println!("{}\n", constants::messages::AVAILABLE_SKILLS_HEADER);
                for skill in skills {
                    println!("  📦 {}", skill.name());
                    println!("     {}", skill.description());
                    let aliases = skill.aliases();
                    if !aliases.is_empty() {
                        println!("     Aliases: {}", aliases.join(", "));
                    }
                    println!();
                }

                println!("{}", constants::messages::TIP_GET_DETAILS);
                println!("{}", constants::messages::TIP_TRY_EXAMPLE);

                Ok(ExitCode::SUCCESS)
            },
            SkillsCommand::Run { skill_name, params } => {
                let params = match params {
                    Some(p) => serde_json::from_str(&p).map_err(|e| {
                        SkillError::InvalidInput(format!(
                            "{} {}",
                            constants::messages::INVALID_JSON_FOR_SKILL.replace("{}", &skill_name),
                            e
                        ))
                    })?,
                    None => json!({}),
                };

                match registry.execute_skill(&skill_name, params).await {
                    Ok(result) => {
                        println!("{}", result.output);
                        Ok(ExitCode::SUCCESS)
                    },
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        Err(eyre::eyre!(constants::messages::SKILL_EXECUTION_FAILED))
                    },
                }
            },
            SkillsCommand::Info { skill_name } => {
                match registry.get(&skill_name) {
                    Some(skill) => {
                        println!("Name: {}", skill.name());
                        println!("Description: {}", skill.description());
                        println!("Interactive: {}", skill.supports_interactive());

                        let ui = skill
                            .render_ui()
                            .await
                            .map_err(|e| eyre::eyre!("Failed to render UI: {}", e))?;
                        if !ui.elements.is_empty() {
                            println!("UI Elements: {}", ui.elements.len());
                        }
                    },
                    None => {
                        return Err(eyre::eyre!("Skill '{}' not found", skill_name));
                    },
                }

                Ok(ExitCode::SUCCESS)
            },
            SkillsCommand::Help => {
                println!("{}\n", constants::help::HEADER);
                println!("{}", constants::help::COMMANDS_HEADER);
                println!("{}", constants::help::CMD_LIST);
                println!("{}", constants::help::CMD_INFO);
                println!("{}", constants::help::CMD_RUN);
                println!("{}", constants::help::CMD_EXAMPLE);
                println!("{}\n", constants::help::CMD_HELP);
                println!("{}", constants::messages::TIP_USE_NATURAL_LANGUAGE);
                println!("{}", constants::messages::TIP_SKILLS_LOCATION);
                Ok(ExitCode::SUCCESS)
            },
            SkillsCommand::Example => {
                println!("{}\n", constants::example::HEADER);
                println!("{}", constants::example::STEP_1);
                println!("{}\n", constants::example::STEP_1_CMD);
                println!("{}", constants::example::STEP_2);
                println!("{}\n", constants::example::STEP_2_CMD);
                println!("{}", constants::example::STEP_3);
                Ok(ExitCode::SUCCESS)
            },
            SkillsCommand::Validate { file } => {
                use crate::cli::skills::validation::SkillValidator;

                let content = fs::read_to_string(&file)
                    .map_err(|e| eyre::eyre!("{} {}", constants::messages::FAILED_TO_READ_FILE, e))?;

                match SkillValidator::validate_skill_json(&content) {
                    Ok(_) => {
                        println!("{}", constants::messages::SKILL_VALID);
                        Ok(ExitCode::SUCCESS)
                    },
                    Err(e) => {
                        eprintln!("{} {}", constants::messages::VALIDATION_FAILED, e);
                        Err(eyre::eyre!(constants::messages::VALIDATION_FAILED_ERROR))
                    },
                }
            },
            SkillsCommand::Install { source: _source } => {
                // TODO: Implement skill installation
                println!("Skill installation not yet implemented");
                Ok(ExitCode::SUCCESS)
            },
            SkillsCommand::Create {
                name,
                from_template,
                description,
                ..
            } => {
                use crate::cli::skills::templates::SkillTemplate;

                if let Some(template_name) = from_template {
                    // Create from template
                    let template = match template_name.as_str() {
                        constants::templates::COMMAND => SkillTemplate::Command,
                        constants::templates::SCRIPT => SkillTemplate::Script,
                        constants::templates::HTTP_API => SkillTemplate::HttpApi,
                        constants::templates::FILE_PROCESSOR => SkillTemplate::FileProcessor,
                        _ => {
                            eprintln!("{} {}\n", constants::messages::UNKNOWN_TEMPLATE, template_name);
                            eprintln!("{}", constants::messages::AVAILABLE_TEMPLATES);
                            for t in SkillTemplate::all() {
                                eprintln!("  {} - {}", t.name(), t.description());
                            }
                            return Err(eyre::eyre!(constants::messages::INVALID_TEMPLATE_ERROR));
                        },
                    };

                    let desc = description.unwrap_or_else(|| format!("{} skill", name));
                    let skill_json = template.generate(&name, &desc);

                    // Save to ~/.q-skills/
                    let skills_dir = dirs::home_dir()
                        .ok_or_else(|| eyre::eyre!(constants::messages::COULD_NOT_FIND_HOME))?
                        .join(constants::HOME_SKILLS_DIR_NAME);

                    std::fs::create_dir_all(&skills_dir)?;
                    let skill_file = skills_dir.join(format!("{}.{}", name, constants::SKILL_FILE_EXTENSION));
                    std::fs::write(&skill_file, serde_json::to_string_pretty(&skill_json)?)?;

                    println!("{} {}", constants::messages::SKILL_CREATED, skill_file.display());
                    println!("\nUsage:");
                    println!("  {}", template.example(&name));

                    return Ok(ExitCode::SUCCESS);
                }

                // Fallback message
                println!("⚠️  Use templates for quick creation:");
                println!(
                    "  q skills create {} --from-template command --description 'My skill'\n",
                    name
                );
                println!("{}", constants::messages::AVAILABLE_TEMPLATES);
                for t in SkillTemplate::all() {
                    println!("  {} - {}", t.name(), t.description());
                }

                Ok(ExitCode::SUCCESS)
            },
            SkillsCommand::Remove { skill_name } => {
                // Find skill file in workspace
                let skills_dir = current_dir.join(constants::SKILLS_DIR_NAME);
                if !skills_dir.exists() {
                    return Err(eyre::eyre!(
                        "{} {}",
                        constants::messages::NO_SKILLS_DIR_FOUND,
                        skills_dir.display()
                    ));
                }

                // Look for skill file
                let skill_file = skills_dir.join(format!("{}.{}", skill_name, constants::SKILL_FILE_EXTENSION));
                if !skill_file.exists() {
                    return Err(eyre::eyre!("Skill '{}' not found", skill_name));
                }

                // Confirm removal
                print!(
                    "{}",
                    constants::messages::REMOVE_CONFIRM_PROMPT.replace("{}", &skill_name)
                );
                io::stdout().flush()?;

                let mut input = String::new();
                io::stdin().read_line(&mut input)?;

                if input.trim().to_lowercase() != "y" {
                    println!("{}", constants::messages::CANCELLED);
                    return Ok(ExitCode::SUCCESS);
                }

                // Remove file
                fs::remove_file(&skill_file)?;
                println!("{} '{}'", constants::messages::SKILL_REMOVED, skill_name);

                Ok(ExitCode::SUCCESS)
            },
        }
    }
}

impl SkillsSlashCommand {
    pub async fn execute(self, _os: &mut Os) -> Result<ExitCode> {
        let current_dir = std::env::current_dir()?;
        let registry = SkillRegistry::with_workspace_skills(&current_dir)
            .await
            .unwrap_or_else(|_| SkillRegistry::with_builtins());

        match self {
            Self::List => {
                let skills = registry.list();

                for skill in skills {
                    let aliases = skill.aliases();
                    if aliases.is_empty() {
                        println!("{}: {}", skill.name(), skill.description());
                    } else {
                        println!("{} ({}): {}", skill.name(), aliases.join(", "), skill.description());
                    }
                }

                Ok(ExitCode::SUCCESS)
            },
            Self::Run { skill_name, params } => {
                let params = match params {
                    Some(p) => serde_json::from_str(&p)
                        .map_err(|e| SkillError::InvalidInput(format!("Invalid JSON: {}", e)))?,
                    None => json!({}),
                };

                let result = registry
                    .execute_skill(&skill_name, params)
                    .await
                    .map_err(|e| eyre::eyre!("Skill execution failed: {}", e))?;
                println!("{}", result.output);

                Ok(ExitCode::SUCCESS)
            },
            Self::Info { skill_name } => {
                match registry.get(&skill_name) {
                    Some(skill) => {
                        println!("Name: {}", skill.name());
                        println!("Description: {}", skill.description());
                        println!("Interactive: {}", skill.supports_interactive());

                        let aliases = skill.aliases();
                        if !aliases.is_empty() {
                            println!("Aliases: {}", aliases.join(", "));
                        }

                        let ui = skill
                            .render_ui()
                            .await
                            .map_err(|e| eyre::eyre!("Failed to render UI: {}", e))?;
                        if !ui.elements.is_empty() {
                            println!("UI Elements: {}", ui.elements.len());
                        }
                    },
                    None => {
                        return Err(eyre::eyre!("Skill '{}' not found", skill_name));
                    },
                }

                Ok(ExitCode::SUCCESS)
            },
        }
    }
}
fn create_skill_template(name: &str, skill_type: &str) -> Result<()> {
    match skill_type {
        "rust" => create_rust_skill_template(name),
        "code_inline" => create_simple_command_skill(name),
        "code_session" => create_simple_repl_skill(name),
        "conversation" => create_simple_assistant_skill(name),
        "prompt_inline" => create_simple_template_skill(name),
        _ => Err(eyre::eyre!(
            "Unknown skill type: {}. Valid types: rust, code_inline, code_session, conversation, prompt_inline",
            skill_type
        )),
    }
}

fn create_simple_command_skill(name: &str) -> Result<()> {
    println!("Creating command skill: {}", name);
    println!("Setting up command execution skill...");
    print!("What command should this skill execute?");
    println!("\n(Press Enter for default: echo 'Hello from skill!')");

    let mut command = String::new();
    io::stdin().read_line(&mut command)?;
    let _command = command.trim();
    let _command = if _command.is_empty() {
        "echo 'Hello from skill!'"
    } else {
        _command
    };

    let skill = json!({
        "name": name,
        "description": format!("Command execution skill: {}", name),
        "version": "1.0.0",
        "type": "code_inline",
        "command": "echo",
        "args": ["Hello from skill!"],
        "timeout": 30
    });

    save_and_validate_json_skill(name, &skill)?;
    println!("Skill created successfully: .q-skills/{}.json", name);
    println!("Use '/skills run {}' to test your skill", name);
    Ok(())
}

fn create_simple_repl_skill(name: &str) -> Result<()> {
    println!("Creating repl skill: {}", name);
    println!("Setting up interactive coding environment...");
    print!("Which interpreter should this use? (python3, node, etc.)");
    println!("\n(Press Enter for default: python3)");

    let mut interpreter = String::new();
    io::stdin().read_line(&mut interpreter)?;
    let interpreter = interpreter.trim();
    let interpreter = if interpreter.is_empty() { "python3" } else { interpreter };

    let skill = json!({
        "name": name,
        "description": format!("Interactive {} environment", interpreter),
        "version": "1.0.0",
        "type": "code_session",
        "command": interpreter,
        "session_config": {
            "session_timeout": 3600,
            "max_sessions": 5,
            "cleanup_on_exit": true
        }
    });

    save_and_validate_json_skill(name, &skill)?;
    println!("Skill created successfully: .q-skills/{}.json", name);
    println!("Use '/skills run {}' to test your skill", name);
    Ok(())
}

fn create_simple_assistant_skill(name: &str) -> Result<()> {
    println!("Creating assistant skill: {}", name);
    println!("Setting up AI assistant...");
    print!("What role should this assistant have?");
    println!("\nExamples: code reviewer, documentation writer, domain expert");
    println!("(Press Enter for default: helpful assistant)");

    let mut role = String::new();
    io::stdin().read_line(&mut role)?;
    let role = role.trim();
    let role = if role.is_empty() { "helpful assistant" } else { role };

    let skill = json!({
        "name": name,
        "description": format!("AI assistant: {}", name),
        "version": "1.0.0",
        "type": "conversation",
        "prompt_template": format!("You are a helpful {} assistant", role),
        "context_files": {
            "patterns": ["*.rs", "*.py", "*.js", "*.md"],
            "max_files": 10,
            "max_file_size_kb": 100
        }
    });

    save_and_validate_json_skill(name, &skill)?;
    println!("Skill created successfully: .q-skills/{}.json", name);
    println!("Development session created for assistant skill");
    println!("Use '/switch {}' to test your assistant", name);
    Ok(())
}

fn create_simple_template_skill(name: &str) -> Result<()> {
    println!("Creating template skill: {}", name);
    println!("Setting up prompt template...");
    print!("What should this template generate?");
    println!("\nExample: Generate documentation for {{function_name}}");
    println!("(Press Enter for default template)");

    let mut template = String::new();
    io::stdin().read_line(&mut template)?;
    let template = template.trim();
    let template = if template.is_empty() {
        format!("Help me with {}", name)
    } else {
        template.to_string()
    };

    let skill = json!({
        "name": name,
        "description": format!("Prompt template: {}", name),
        "version": "1.0.0",
        "type": "prompt_inline",
        "prompt": template,
        "parameters": []
    });

    save_and_validate_json_skill(name, &skill)?;
    println!("Skill created successfully: .q-skills/{}.json", name);
    println!("Use '/skills run {}' to test your skill", name);
    Ok(())
}

fn create_rust_skill_template(name: &str) -> Result<()> {
    let template = format!(
        r#"use async_trait::async_trait;
use serde_json::{{json, Value}};
use crate::cli::skills::{{Skill, SkillResult, SkillUI, UIElement, Result}};

pub struct {}Skill;

#[async_trait]
impl Skill for {}Skill {{
    fn name(&self) -> &str {{
        "{}"
    }}

    fn description(&self) -> &str {{
        "A custom skill"
    }}

    async fn execute(&self, params: Value) -> Result<SkillResult> {{
        Ok(SkillResult {{
            output: "Hello from {}!".to_string(),
            ui_updates: None,
            state_changes: None,
        }})
    }}

    async fn render_ui(&self) -> Result<SkillUI> {{
        Ok(SkillUI {{
            elements: vec![UIElement::Text("Custom skill UI".to_string())],
            interactive: false,
        }})
    }}
}}
"#,
        name.chars().next().unwrap().to_uppercase().collect::<String>() + &name[1..],
        name.chars().next().unwrap().to_uppercase().collect::<String>() + &name[1..],
        name,
        name
    );

    fs::write(format!("{}.rs", name), template)?;
    println!("Created Rust skill template: {}.rs", name);
    Ok(())
}

fn create_json_skill_template(name: &str, skill_type: &str) -> Result<()> {
    let template = match skill_type {
        "code_inline" => json!({
            "name": name,
            "description": format!("A {} skill", name),
            "version": "1.0.0",
            "type": "code_inline",
            "command": "echo",
            "args": ["Hello from code inline skill!"],
            "timeout": 30,
            "security": {
                "resource_limits": {
                    "max_memory_mb": 100,
                    "max_execution_time": 30
                }
            }
        }),
        "code_session" => json!({
            "name": name,
            "description": format!("A {} skill", name),
            "version": "1.0.0",
            "type": "code_session",
            "command": "python3",
            "session_config": {
                "session_timeout": 3600,
                "max_sessions": 5,
                "cleanup_on_exit": true
            },
            "security": {
                "permissions": {
                    "file_read": ["./"],
                    "network_access": false
                }
            }
        }),
        "conversation" => json!({
            "name": name,
            "description": format!("A {} skill", name),
            "version": "1.0.0",
            "type": "conversation",
            "prompt_template": "Analyze this: {input}",
            "context_files": {
                "patterns": ["*.rs", "*.py"],
                "max_files": 10,
                "max_file_size_kb": 100
            },
            "security": {
                "permissions": {
                    "file_read": ["./src", "./tests"]
                }
            }
        }),
        "prompt_inline" => json!({
            "name": name,
            "description": format!("A {} skill", name),
            "version": "1.0.0",
            "type": "prompt_inline",
            "prompt": "Generate {type} for {language}",
            "parameters": [
                {
                    "name": "type",
                    "type": "enum",
                    "values": ["test", "documentation", "example"],
                    "required": true
                },
                {
                    "name": "language",
                    "type": "string",
                    "pattern": "^[a-zA-Z]+$",
                    "required": true
                }
            ]
        }),
        _ => return Err(eyre::eyre!("Unknown skill type: {}", skill_type)),
    };

    // Validate the generated JSON
    let json_str = serde_json::to_string_pretty(&template)?;
    if let Err(e) = SkillValidator::validate_skill_json(&json_str) {
        return Err(eyre::eyre!("Generated invalid JSON: {}", e));
    }

    // Ensure .q-skills directory exists
    fs::create_dir_all(".q-skills")?;

    let filename = format!(".q-skills/{}.json", name);
    fs::write(&filename, json_str)?;
    println!("Created {} skill: {}", skill_type, filename);
    Ok(())
}

// QUICK MODE: For experts who know exactly what they want
fn create_skill_quick(name: &str, skill_type: &str, command: Option<&str>, template: Option<&str>) -> Result<()> {
    match skill_type {
        "code_inline" => {
            let cmd = command.ok_or_else(|| eyre::eyre!("--command required for quick code_inline creation"))?;
            let skill = json!({
                "name": name,
                "description": format!("Quick {} skill", name),
                "version": "1.0.0",
                "type": "code_inline",
                "command": cmd,
                "args": []
            });
            save_and_validate_json_skill(name, &skill)?;
        },
        "prompt_inline" => {
            let tmpl = template.ok_or_else(|| eyre::eyre!("--template required for quick prompt_inline creation"))?;
            let skill = json!({
                "name": name,
                "description": format!("Quick {} skill", name),
                "version": "1.0.0",
                "type": "prompt_inline",
                "prompt": tmpl,
                "parameters": []
            });
            save_and_validate_json_skill(name, &skill)?;
        },
        _ => {
            return Err(eyre::eyre!(
                "Quick mode only supports code_inline and prompt_inline. Use --wizard for other types."
            ));
        },
    }
    Ok(())
}

// WIZARD MODE: Step-by-step guided creation with explanations
async fn create_skill_wizard(name: &str) -> Result<()> {
    println!("🧙 Skills Creation Wizard");
    println!("Let's create a skill called '{}'", name);
    println!();

    // Step 1: Understand user intent
    println!("What do you want this skill to do?");
    println!("1. 🔧 Run a command or script");
    println!("2. 💬 Have a conversation with AI");
    println!("3. 📝 Generate text from a template");
    println!("4. 🔄 Start an interactive session");
    println!("5. ⚡ Write custom Rust code");
    println!();

    print!("Choose (1-5): ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    match input.trim() {
        "1" => create_command_skill_wizard(name).await,
        "2" => create_conversation_skill_wizard(name).await,
        "3" => create_template_skill_wizard(name).await,
        "4" => create_session_skill_wizard(name).await,
        "5" => create_rust_skill_wizard(name).await,
        _ => Err(eyre::eyre!("Invalid choice")),
    }
}

async fn create_command_skill_wizard(name: &str) -> Result<()> {
    println!();
    println!("🔧 Creating a Command Skill");
    println!("This skill will run a command and return the output.");
    println!();

    print!("What command should it run? (e.g., 'ls', 'echo hello', 'python script.py'): ");
    io::stdout().flush()?;
    let mut command_input = String::new();
    io::stdin().read_line(&mut command_input)?;
    let command_parts: Vec<&str> = command_input.trim().split_whitespace().collect();

    if command_parts.is_empty() {
        return Err(eyre::eyre!("Command cannot be empty"));
    }

    let command = command_parts[0];
    let args: Vec<String> = command_parts[1..].iter().map(|s| s.to_string()).collect();

    print!("Brief description of what this does: ");
    io::stdout().flush()?;
    let mut desc = String::new();
    io::stdin().read_line(&mut desc)?;
    let description = if desc.trim().is_empty() {
        format!("Runs {}", command)
    } else {
        desc.trim().to_string()
    };

    let skill = json!({
        "name": name,
        "description": description,
        "version": "1.0.0",
        "type": "code_inline",
        "command": command,
        "args": args,
        "timeout": 30,
        "security": {
            "resource_limits": {
                "max_memory_mb": 100,
                "max_execution_time": 30
            }
        }
    });

    println!();
    println!("📋 Preview:");
    println!("   Name: {}", name);
    println!("   Command: {} {}", command, args.join(" "));
    println!("   Description: {}", description);
    println!();

    print!("Create this skill? (Y/n): ");
    io::stdout().flush()?;
    let mut confirm = String::new();
    io::stdin().read_line(&mut confirm)?;

    if confirm.trim().to_lowercase() == "n" {
        println!("❌ Cancelled");
        return Ok(());
    }

    save_and_validate_json_skill(name, &skill)?;
    println!("🎉 Skill created! Try: skills run {}", name);
    Ok(())
}

async fn create_template_skill_wizard(name: &str) -> Result<()> {
    println!();
    println!("📝 Creating a Text Generator Skill");
    println!("This skill generates text by filling in a template with your input.");
    println!();

    print!("Enter your template (use {{variable}} for things that change): ");
    io::stdout().flush()?;
    let mut template = String::new();
    io::stdin().read_line(&mut template)?;
    let template = template.trim();

    if template.is_empty() {
        return Err(eyre::eyre!("Template cannot be empty"));
    }

    // Extract variables from template
    let variables: Vec<String> = template
        .split('{')
        .filter_map(|part| part.split('}').next())
        .filter(|var| !var.is_empty())
        .map(|var| var.to_string())
        .collect();

    let mut parameters = Vec::new();
    for var in &variables {
        parameters.push(json!({
            "name": var,
            "type": "string",
            "required": true
        }));
    }

    let skill = json!({
        "name": name,
        "description": format!("Generates text using template: {}", template),
        "version": "1.0.0",
        "type": "prompt_inline",
        "prompt": template,
        "parameters": parameters
    });

    println!();
    println!("📋 Preview:");
    println!("   Template: {}", template);
    if !variables.is_empty() {
        println!("   Variables: {}", variables.join(", "));
        println!(
            "   Usage: skills run {} --params '{{\"{}\":\"value\"}}'",
            name, variables[0]
        );
    }
    println!();

    print!("Create this skill? (Y/n): ");
    io::stdout().flush()?;
    let mut confirm = String::new();
    io::stdin().read_line(&mut confirm)?;

    if confirm.trim().to_lowercase() != "n" {
        save_and_validate_json_skill(name, &skill)?;
        println!("🎉 Skill created! Try: skills run {}", name);
    } else {
        println!("❌ Cancelled");
    }

    Ok(())
}

async fn create_conversation_skill_wizard(name: &str) -> Result<()> {
    println!();
    println!("💬 Creating an AI Conversation Skill");
    println!("This skill will have a conversation with AI about your input.");
    println!();

    print!("What should the AI help with? (e.g., 'Review this code', 'Explain this concept'): ");
    io::stdout().flush()?;
    let mut prompt = String::new();
    io::stdin().read_line(&mut prompt)?;
    let prompt = prompt.trim();

    let skill = json!({
        "name": name,
        "description": format!("AI conversation: {}", prompt),
        "version": "1.0.0",
        "type": "conversation",
        "prompt_template": format!("{}: {{input}}", prompt),
        "context_files": {
            "patterns": ["*.rs", "*.py", "*.js", "*.md"],
            "max_files": 10,
            "max_file_size_kb": 100
        }
    });

    save_and_validate_json_skill(name, &skill)?;
    println!("🎉 AI conversation skill created!");
    Ok(())
}

async fn create_session_skill_wizard(name: &str) -> Result<()> {
    println!();
    println!("🔄 Creating an Interactive Session Skill");
    println!("This skill starts a persistent session (like Python REPL, Node.js, etc.)");
    println!();

    print!("What program should it run? (e.g., 'python3', 'node', 'bash'): ");
    io::stdout().flush()?;
    let mut command = String::new();
    io::stdin().read_line(&mut command)?;
    let command = command.trim();

    let skill = json!({
        "name": name,
        "description": format!("Interactive {} session", command),
        "version": "1.0.0",
        "type": "code_session",
        "command": command,
        "session_config": {
            "session_timeout": 3600,
            "max_sessions": 5,
            "cleanup_on_exit": true
        }
    });

    save_and_validate_json_skill(name, &skill)?;
    println!("🎉 Session skill created!");
    Ok(())
}

async fn create_rust_skill_wizard(name: &str) -> Result<()> {
    println!();
    println!("⚡ Creating a Custom Rust Skill");
    println!("This creates a Rust template for advanced customization.");
    println!();

    create_rust_skill_template(name)?;
    println!("🎉 Rust skill template created!");
    println!("💡 Edit {}.rs to customize the behavior", name);
    Ok(())
}

// INTERACTIVE MODE: Simple prompts for skill type selection
async fn create_skill_interactive(name: &str) -> Result<()> {
    println!("Creating skill: {}", name);
    println!();
    println!("What type of skill do you want to create?");
    println!("1. Command (run a shell command)");
    println!("2. AI Chat (conversation with AI)");
    println!("3. Template (generate text from template)");
    println!("4. Session (interactive shell session)");
    println!("5. Custom (Rust code)");
    println!();

    print!("Choose (1-5): ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    match input.trim() {
        "1" => create_code_inline_interactive(name).await,
        "2" => create_conversation_interactive(name).await,
        "3" => create_prompt_inline_interactive(name).await,
        "4" => create_code_session_interactive(name).await,
        "5" => {
            create_rust_skill_template(name)?;
            Ok(())
        },
        _ => Err(eyre::eyre!("Invalid choice")),
    }
}

async fn create_code_inline_interactive(name: &str) -> Result<()> {
    print!("Enter command to execute: ");
    io::stdout().flush()?;
    let mut command = String::new();
    io::stdin().read_line(&mut command)?;
    let command = command.trim();

    print!("Enter description: ");
    io::stdout().flush()?;
    let mut description = String::new();
    io::stdin().read_line(&mut description)?;
    let description = description.trim();

    let template = json!({
        "name": name,
        "description": if description.is_empty() { format!("A {} skill", name) } else { description.to_string() },
        "version": "1.0.0",
        "type": "code_inline",
        "command": command,
        "args": [],
        "timeout": 30,
        "security": {
            "resource_limits": {
                "max_memory_mb": 100,
                "max_execution_time": 30
            }
        }
    });

    save_and_validate_json_skill(name, &template)
}

async fn create_code_session_interactive(name: &str) -> Result<()> {
    print!("Enter session command (e.g., python3, node): ");
    io::stdout().flush()?;
    let mut command = String::new();
    io::stdin().read_line(&mut command)?;
    let command = command.trim();

    let template = json!({
        "name": name,
        "description": format!("A {} session skill", name),
        "version": "1.0.0",
        "type": "code_session",
        "command": command,
        "session_config": {
            "session_timeout": 3600,
            "max_sessions": 5,
            "cleanup_on_exit": true
        }
    });

    save_and_validate_json_skill(name, &template)
}

async fn create_conversation_interactive(name: &str) -> Result<()> {
    print!("Enter prompt template (use {{input}} for user input): ");
    io::stdout().flush()?;
    let mut prompt = String::new();
    io::stdin().read_line(&mut prompt)?;
    let prompt = prompt.trim();

    let template = json!({
        "name": name,
        "description": format!("A {} conversation skill", name),
        "version": "1.0.0",
        "type": "conversation",
        "prompt_template": prompt,
        "context_files": {
            "patterns": ["*.rs", "*.py", "*.js"],
            "max_files": 10,
            "max_file_size_kb": 100
        }
    });

    save_and_validate_json_skill(name, &template)
}

async fn create_prompt_inline_interactive(name: &str) -> Result<()> {
    print!("Enter prompt template (use {{param_name}} for parameters): ");
    io::stdout().flush()?;
    let mut prompt = String::new();
    io::stdin().read_line(&mut prompt)?;
    let prompt = prompt.trim();

    let template = json!({
        "name": name,
        "description": format!("A {} prompt skill", name),
        "version": "1.0.0",
        "type": "prompt_inline",
        "prompt": prompt,
        "parameters": [
            {
                "name": "input",
                "type": "string",
                "required": true
            }
        ]
    });

    save_and_validate_json_skill(name, &template)
}

fn save_and_validate_json_skill(name: &str, template: &serde_json::Value) -> Result<()> {
    let json_str = serde_json::to_string_pretty(template)?;

    // Validate the generated JSON
    if let Err(e) = SkillValidator::validate_skill_json(&json_str) {
        return Err(eyre::eyre!("Generated invalid JSON: {}", e));
    }

    // Ensure .q-skills directory exists
    fs::create_dir_all(".q-skills")?;

    let filename = format!(".q-skills/{}.json", name);
    fs::write(&filename, json_str)?;
    println!("Created skill: {}", filename);
    println!("✅ JSON validation passed");
    Ok(())
}


/// Command handlers - extracted for testability
mod handlers {
    use super::*;
    use std::io::Write;

    /// Handle the list command
    pub async fn list_command(
        registry: &SkillRegistry,
        output: &mut dyn Write,
    ) -> Result<(), error::SkillsCliError> {
        let skills = registry.list();

        if skills.is_empty() {
            writeln!(output, "{}\n", constants::messages::NO_SKILLS_FOUND)?;
            writeln!(output, "{}", constants::messages::TIP_TRY_EXAMPLE)?;
            return Ok(());
        }

        writeln!(output, "{}\n", constants::messages::AVAILABLE_SKILLS_HEADER)?;
        for skill in skills {
            writeln!(output, "  📦 {}", skill.name())?;
            writeln!(output, "     {}", skill.description())?;
            let aliases = skill.aliases();
            if !aliases.is_empty() {
                writeln!(output, "     Aliases: {}", aliases.join(", "))?;
            }
            writeln!(output)?;
        }

        writeln!(output, "{}", constants::messages::TIP_GET_DETAILS)?;
        writeln!(output, "{}", constants::messages::TIP_TRY_EXAMPLE)?;

        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[tokio::test]
        async fn test_list_empty_registry() {
            let registry = SkillRegistry::new();
            let mut output = Vec::new();
            
            let result = list_command(&registry, &mut output).await;
            
            assert!(result.is_ok());
            let output_str = String::from_utf8(output).unwrap();
            assert!(output_str.contains("No skills found"));
            assert!(output_str.contains("💡 Try: q skills example"));
        }

        #[tokio::test]
        async fn test_list_with_skills() {
            let registry = SkillRegistry::with_builtins();
            let mut output = Vec::new();
            
            let result = list_command(&registry, &mut output).await;
            
            assert!(result.is_ok());
            let output_str = String::from_utf8(output).unwrap();
            assert!(output_str.contains("Available Skills:"));
            assert!(output_str.contains("📦"));
        }
    }
}
