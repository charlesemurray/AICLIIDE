use crate::cli::skills::{SkillRegistry, SkillError};
use crate::cli::skills::validation::SkillValidator;
use crate::os::Os;
use clap::{Args, Subcommand};
use eyre::Result;
use serde_json::json;
use std::fs;
use std::process::ExitCode;
use std::io::{self, Write};

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
    /// Install a skill from a file or URL
    Install {
        /// Path or URL to skill definition
        source: String,
    },
    /// Create a new skill
    Create {
        /// Name of the skill to create
        name: String,
        /// Type of skill to create (code_inline, code_session, conversation, prompt_inline, rust)
        #[arg(long, short)]
        skill_type: Option<String>,
        /// Interactive mode for guided creation
        #[arg(long, short)]
        interactive: bool,
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
        let mut registry = SkillRegistry::with_builtins();
        
        // Load skills from current directory
        let current_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        if let Err(e) = registry.reload_workspace_skills(&current_dir).await {
            eprintln!("Warning: Failed to load workspace skills: {}", e);
        }
        
        match self.command {
            SkillsCommand::List => {
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
            SkillsCommand::Run { skill_name, params } => {
                let params = match params {
                    Some(p) => serde_json::from_str(&p)
                        .map_err(|e| SkillError::InvalidInput(format!("Invalid JSON: {}", e)))?,
                    None => json!({}),
                };
                
                let result = registry.execute_skill(&skill_name, params).await
                    .map_err(|e| eyre::eyre!("Skill execution failed: {}", e))?;
                println!("{}", result.output);
                
                Ok(ExitCode::SUCCESS)
            },
            SkillsCommand::Info { skill_name } => {
                match registry.get(&skill_name) {
                    Some(skill) => {
                        println!("Name: {}", skill.name());
                        println!("Description: {}", skill.description());
                        println!("Interactive: {}", skill.supports_interactive());
                        
                        let ui = skill.render_ui().await
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
            SkillsCommand::Install { source: _source } => {
                // TODO: Implement skill installation
                println!("Skill installation not yet implemented");
                Ok(ExitCode::SUCCESS)
            },
            SkillsCommand::Create { name, skill_type, interactive } => {
                if interactive || skill_type.is_none() {
                    create_skill_interactive(&name).await?;
                } else {
                    let skill_type = skill_type.as_ref().unwrap();
                    create_skill_template(&name, skill_type)?;
                }
                Ok(ExitCode::SUCCESS)
            },
        }
    }
}

impl SkillsSlashCommand {
    pub async fn execute(self, _os: &mut Os) -> Result<ExitCode> {
        let registry = SkillRegistry::with_builtins();
        
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
                
                let result = registry.execute_skill(&skill_name, params).await
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
                        
                        let ui = skill.render_ui().await
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
        "code_inline" => create_json_skill_template(name, "code_inline"),
        "code_session" => create_json_skill_template(name, "code_session"),
        "conversation" => create_json_skill_template(name, "conversation"),
        "prompt_inline" => create_json_skill_template(name, "prompt_inline"),
        _ => Err(eyre::eyre!("Unknown skill type: {}. Valid types: rust, code_inline, code_session, conversation, prompt_inline", skill_type)),
    }
}

fn create_rust_skill_template(name: &str) -> Result<()> {
    let template = format!(r#"use async_trait::async_trait;
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

async fn create_skill_interactive(name: &str) -> Result<()> {
    println!("Creating skill: {}", name);
    println!("Select skill type:");
    println!("1. code_inline - Execute commands and return output");
    println!("2. code_session - Maintain persistent command sessions");
    println!("3. conversation - AI conversation prompts with context");
    println!("4. prompt_inline - Parameterized prompt templates");
    println!("5. rust - Rust skill (advanced)");
    
    print!("Enter choice (1-5): ");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    let skill_type = match input.trim() {
        "1" => "code_inline",
        "2" => "code_session", 
        "3" => "conversation",
        "4" => "prompt_inline",
        "5" => "rust",
        _ => return Err(eyre::eyre!("Invalid choice")),
    };
    
    if skill_type == "rust" {
        create_rust_skill_template(name)?;
        return Ok(());
    }
    
    // For JSON skills, create with customization
    match skill_type {
        "code_inline" => create_code_inline_interactive(name).await,
        "code_session" => create_code_session_interactive(name).await,
        "conversation" => create_conversation_interactive(name).await,
        "prompt_inline" => create_prompt_inline_interactive(name).await,
        _ => unreachable!(),
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
            "max_files": 10
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
