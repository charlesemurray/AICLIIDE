use crate::cli::skills::types::SkillType;
use crate::cli::custom_commands::*;
use serde_json::Value;
use std::io::{self, Write};

#[derive(Debug, Clone, PartialEq)]
pub enum CreationType {
    Skill(SkillType),
    CustomCommand,
    Agent,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CreationState {
    Discovery,
    Configuration,
    Completion,
}

pub struct UnifiedCreationAssistant {
    creation_type: CreationType,
    state: CreationState,
    name: String,
    description: String,
    parameters: Vec<Parameter>,
    command: Option<String>,
    command_type: Option<CommandType>,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub default_value: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommandType {
    Script,
    Alias,
    Builtin,
}

impl UnifiedCreationAssistant {
    pub fn new_custom_command(name: &str) -> Self {
        Self {
            creation_type: CreationType::CustomCommand,
            state: CreationState::Discovery,
            name: name.to_string(),
            description: String::new(),
            parameters: Vec::new(),
            command: None,
            command_type: Some(CommandType::Script),
        }
    }

    pub fn start_discovery(&self) -> String {
        match &self.creation_type {
            CreationType::CustomCommand => {
                format!(
                    "🛠️ Creation Assistant - Custom Command\nCreating command: /{}\n\nWhat should this command do?",
                    self.name
                )
            }
            _ => "Creation type not implemented".to_string(),
        }
    }

    pub fn handle_discovery_response(&mut self, user_input: &str) -> String {
        self.description = user_input.to_string();
        self.state = CreationState::Configuration;
        
        match &self.creation_type {
            CreationType::CustomCommand => {
                let input_lower = user_input.to_lowercase();
                if input_lower.contains("alias") {
                    self.command_type = Some(CommandType::Alias);
                    "What command should this alias point to?"
                } else if input_lower.contains("builtin") {
                    self.command_type = Some(CommandType::Builtin);
                    "Which builtin function?"
                } else {
                    "What shell command should this execute?"
                }.to_string()
            }
            _ => "Not implemented".to_string(),
        }
    }

    pub fn handle_configuration_response(&mut self, user_input: &str) -> String {
        self.command = Some(user_input.to_string());
        self.state = CreationState::Completion;
        self.generate_completion_message()
    }

    fn generate_completion_message(&self) -> String {
        match &self.creation_type {
            CreationType::CustomCommand => {
                format!(
                    "✅ Custom Command Ready!\n\nName: /{}\nDescription: {}\n\nSave this command? (yes/no)",
                    self.name, self.description
                )
            }
            _ => "Not implemented".to_string(),
        }
    }

    pub fn create_artifact(&self) -> Result<CreationArtifact, String> {
        match &self.creation_type {
            CreationType::CustomCommand => {
                let command = self.command.as_ref().ok_or("No command specified")?;
                let command_type = self.command_type.as_ref().ok_or("No command type")?;

                let handler = match command_type {
                    CommandType::Script => CommandHandler::Script { 
                        command: command.clone(), 
                        args: vec![] 
                    },
                    CommandType::Alias => CommandHandler::Alias { 
                        target: command.clone() 
                    },
                    CommandType::Builtin => CommandHandler::Builtin { 
                        function_name: command.clone() 
                    },
                };

                let cmd = CustomCommand {
                    name: self.name.clone(),
                    description: self.description.clone(),
                    handler,
                    parameters: vec![],
                    created_at: chrono::Utc::now().to_rfc3339(),
                    usage_count: 0,
                };

                Ok(CreationArtifact::CustomCommand(cmd))
            }
            _ => Err("Not implemented".to_string()),
        }
    }

    pub fn is_complete(&self) -> bool {
        matches!(self.state, CreationState::Completion)
    }
}

#[derive(Debug)]
pub enum CreationArtifact {
    CustomCommand(CustomCommand),
    Skill(Value),
    Agent(Value),
}

pub struct UnifiedCreationCLI {
    assistant: UnifiedCreationAssistant,
}

impl UnifiedCreationCLI {
    pub fn new_custom_command(name: &str) -> Self {
        Self {
            assistant: UnifiedCreationAssistant::new_custom_command(name),
        }
    }

    pub async fn run_interactive(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("{}", self.assistant.start_discovery());
        
        let user_input = self.get_user_input()?;
        println!("{}", self.assistant.handle_discovery_response(&user_input));
        
        let user_input = self.get_user_input()?;
        println!("{}", self.assistant.handle_configuration_response(&user_input));
        
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
        match self.assistant.create_artifact()? {
            CreationArtifact::CustomCommand(cmd) => {
                let commands_dir = std::env::current_dir()?.join(".q-commands");
                let mut registry = CustomCommandRegistry::new(commands_dir)?;
                registry.add_command(cmd)?;
            }
            _ => println!("Not implemented"),
        }
        Ok(())
    }

    fn get_user_input(&self) -> Result<String, Box<dyn std::error::Error>> {
        print!("> ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        Ok(input.trim().to_string())
    }
}
