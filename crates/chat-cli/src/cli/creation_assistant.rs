use crate::cli::skills::types::SkillType;
use crate::cli::custom_commands::*;
use serde_json::{json, Value};

#[derive(Debug, Clone, PartialEq)]
pub enum CreationType {
    Skill(SkillType),
    CustomCommand(CommandType),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommandType {
    Script,
    Alias,
    Builtin,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CreationState {
    Discovery,      // Understanding what user wants to build
    Configuration,  // Setting up parameters
    Testing,        // Testing functionality
    Completion,     // Finalizing and saving
}

pub struct UnifiedCreationAssistant {
    creation_type: CreationType,
    state: CreationState,
    name: String,
    description: String,
    command: Option<String>,
    parameters: Vec<CommandParameter>,
}

impl UnifiedCreationAssistant {
    pub fn new_skill(name: &str, skill_type: SkillType) -> Self {
        Self {
            creation_type: CreationType::Skill(skill_type),
            state: CreationState::Discovery,
            name: name.to_string(),
            description: String::new(),
            command: None,
            parameters: Vec::new(),
        }
    }

    pub fn new_custom_command(name: &str) -> Self {
        Self {
            creation_type: CreationType::CustomCommand(CommandType::Script),
            state: CreationState::Discovery,
            name: name.to_string(),
            description: String::new(),
            command: None,
            parameters: Vec::new(),
        }
    }

    pub fn start_discovery(&self) -> String {
        match &self.creation_type {
            CreationType::Skill(skill_type) => {
                let skill_type_name = match skill_type {
                    SkillType::CodeInline => "command",
                    SkillType::PromptInline => "template",
                    SkillType::Conversation => "assistant",
                    SkillType::CodeSession => "REPL",
                };
                format!(
                    "🛠️ Skill Creation Assistant\nCreating {} skill: {}\n\nWhat are you trying to accomplish with this skill?",
                    skill_type_name, self.name
                )
            }
            CreationType::CustomCommand(_) => {
                format!(
                    "🛠️ Custom Command Creation Assistant\nCreating command: /{}\n\nWhat should this command do? (e.g., 'run git status', 'deploy to staging', 'create alias for ls -la')",
                    self.name
                )
            }
        }
    }

    pub fn handle_discovery_response(&mut self, user_input: &str) -> String {
        self.description = user_input.to_string();
        self.state = CreationState::Configuration;

        match &self.creation_type {
            CreationType::Skill(_) => {
                // Delegate to existing skills assistant
                "Please provide the command or prompt for your skill:".to_string()
            }
            CreationType::CustomCommand(_) => {
                // Determine command type from user input
                if user_input.to_lowercase().contains("alias") {
                    self.creation_type = CreationType::CustomCommand(CommandType::Alias);
                    "What command should this alias point to? (e.g., 'ls -la', 'git status --short')"
                } else if user_input.to_lowercase().contains("builtin") || user_input.to_lowercase().contains("save") || user_input.to_lowercase().contains("context") {
                    self.creation_type = CreationType::CustomCommand(CommandType::Builtin);
                    "Which builtin function? (save_context, clear_context, show_stats)"
                } else {
                    "What shell command should this execute? (use {{param}} for parameters, e.g., 'git checkout {{branch}}')"
                }.to_string()
            }
        }
    }

    pub fn handle_configuration_response(&mut self, user_input: &str) -> String {
        self.command = Some(user_input.to_string());

        match &self.creation_type {
            CreationType::CustomCommand(_) => {
                // Check if command has parameters
                if user_input.contains("{{") && user_input.contains("}}") {
                    "I see your command uses parameters. Let's configure them.\nFor each {{param}} in your command, tell me:\n- Is it required? (yes/no)\n- Description?\n\nExample: 'branch: required, Git branch to checkout'"
                } else {
                    self.state = CreationState::Completion;
                    self.generate_completion_message()
                }.to_string()
            }
            CreationType::Skill(_) => {
                // Delegate to skills assistant
                self.state = CreationState::Completion;
                "Skill configuration complete. Ready to save?".to_string()
            }
        }
    }

    pub fn handle_parameter_configuration(&mut self, user_input: &str) -> String {
        // Parse parameter configuration
        for line in user_input.lines() {
            if let Some((name, config)) = line.split_once(':') {
                let name = name.trim();
                let config = config.trim().to_lowercase();
                let required = config.contains("required");
                let description = config.split(',').nth(1).unwrap_or("Parameter").trim();

                self.parameters.push(CommandParameter {
                    name: name.to_string(),
                    description: description.to_string(),
                    required,
                    default_value: if required { None } else { Some("".to_string()) },
                });
            }
        }

        self.state = CreationState::Completion;
        self.generate_completion_message()
    }

    fn generate_completion_message(&self) -> String {
        match &self.creation_type {
            CreationType::CustomCommand(cmd_type) => {
                let type_name = match cmd_type {
                    CommandType::Script => "Script",
                    CommandType::Alias => "Alias", 
                    CommandType::Builtin => "Builtin",
                };

                let mut message = format!(
                    "✅ Custom Command Ready!\n\nName: /{}\nType: {}\nDescription: {}\n",
                    self.name, type_name, self.description
                );

                if let Some(cmd) = &self.command {
                    message.push_str(&format!("Command: {}\n", cmd));
                }

                if !self.parameters.is_empty() {
                    message.push_str("Parameters:\n");
                    for param in &self.parameters {
                        let req = if param.required { "required" } else { "optional" };
                        message.push_str(&format!("  - {}: {} ({})\n", param.name, param.description, req));
                    }
                }

                message.push_str("\nSave this command? (yes/no)");
                message
            }
            CreationType::Skill(_) => {
                "Skill ready to save!".to_string()
            }
        }
    }

    pub fn create_custom_command(&self) -> Result<CustomCommand, CommandError> {
        let command = self.command.as_ref().ok_or_else(|| 
            CommandError::InvalidParameter("No command specified".to_string()))?;

        let handler = match &self.creation_type {
            CreationType::CustomCommand(CommandType::Script) => {
                CommandHandler::Script { 
                    command: command.clone(), 
                    args: vec![] 
                }
            }
            CreationType::CustomCommand(CommandType::Alias) => {
                CommandHandler::Alias { 
                    target: command.clone() 
                }
            }
            CreationType::CustomCommand(CommandType::Builtin) => {
                CommandHandler::Builtin { 
                    function_name: command.clone() 
                }
            }
            _ => return Err(CommandError::InvalidParameter("Not a custom command".to_string())),
        };

        let mut cmd = CustomCommand {
            name: self.name.clone(),
            description: self.description.clone(),
            handler,
            parameters: self.parameters.clone(),
            created_at: chrono::Utc::now().to_rfc3339(),
            usage_count: 0,
        };

        Ok(cmd)
    }

    pub fn state(&self) -> &CreationState {
        &self.state
    }

    pub fn is_complete(&self) -> bool {
        matches!(self.state, CreationState::Completion)
    }
}

pub struct UnifiedCreationCLI {
    assistant: UnifiedCreationAssistant,
}

impl UnifiedCreationCLI {
    pub fn new_skill(name: &str, skill_type: SkillType) -> Self {
        Self {
            assistant: UnifiedCreationAssistant::new_skill(name, skill_type),
        }
    }

    pub fn new_custom_command(name: &str) -> Self {
        Self {
            assistant: UnifiedCreationAssistant::new_custom_command(name),
        }
    }

    pub async fn run_interactive(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("{}", self.assistant.start_discovery());
        
        // Discovery phase
        let user_input = self.get_user_input()?;
        println!("{}", self.assistant.handle_discovery_response(&user_input));
        
        // Configuration phase
        let user_input = self.get_user_input()?;
        let response = self.assistant.handle_configuration_response(&user_input);
        println!("{}", response);
        
        // Parameter configuration if needed
        if !self.assistant.is_complete() {
            let user_input = self.get_user_input()?;
            println!("{}", self.assistant.handle_parameter_configuration(&user_input));
        }
        
        // Final confirmation
        if self.assistant.is_complete() {
            let user_input = self.get_user_input()?;
            if user_input.trim().to_lowercase().starts_with('y') {
                self.save_creation().await?;
                println!("✅ Created successfully!");
            } else {
                println!("❌ Creation cancelled.");
            }
        }
        
        Ok(())
    }

    async fn save_creation(&self) -> Result<(), Box<dyn std::error::Error>> {
        match &self.assistant.creation_type {
            CreationType::CustomCommand(_) => {
                let cmd = self.assistant.create_custom_command()?;
                let commands_dir = std::env::current_dir()?.join(".q-commands");
                let mut registry = CustomCommandRegistry::new(commands_dir)?;
                registry.add_command(cmd)?;
            }
            CreationType::Skill(_) => {
                // Delegate to skills creation logic
                println!("Skill creation not implemented in unified assistant yet");
            }
        }
        Ok(())
    }

    fn get_user_input(&self) -> Result<String, Box<dyn std::error::Error>> {
        use std::io::{self, Write};
        print!("> ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        Ok(input.trim().to_string())
    }
}
