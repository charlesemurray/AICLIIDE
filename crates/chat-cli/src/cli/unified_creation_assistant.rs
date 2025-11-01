use crate::cli::skills::types::SkillType;
use crate::cli::custom_commands::*;
use serde_json::Value;
use std::io::{self, Write};

#[derive(Debug, Clone, PartialEq)]
pub enum CreationType {
    Skill(SkillType),
    CustomCommand,
    Agent, // Future: Agent creation
}

#[derive(Debug, Clone, PartialEq)]
pub enum CreationState {
    Discovery,      // Understanding what user wants to build
    Configuration,  // Setting up parameters and details
    Testing,        // Testing functionality (skills only)
    Completion,     // Finalizing and saving
}

pub struct UnifiedCreationAssistant {
    creation_type: CreationType,
    state: CreationState,
    name: String,
    description: String,
    
    // Common fields
    parameters: Vec<Parameter>,
    
    // Type-specific fields
    command: Option<String>,           // For custom commands and some skills
    skill_type: Option<SkillType>,     // For skills
    command_type: Option<CommandType>, // For custom commands
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
    pub fn new_skill(name: &str, skill_type: SkillType) -> Self {
        Self {
            creation_type: CreationType::Skill(skill_type.clone()),
            state: CreationState::Discovery,
            name: name.to_string(),
            description: String::new(),
            parameters: Vec::new(),
            command: None,
            skill_type: Some(skill_type),
            command_type: None,
        }
    }

    pub fn new_custom_command(name: &str) -> Self {
        Self {
            creation_type: CreationType::CustomCommand,
            state: CreationState::Discovery,
            name: name.to_string(),
            description: String::new(),
            parameters: Vec::new(),
            command: None,
            skill_type: None,
            command_type: Some(CommandType::Script),
        }
    }

    pub fn new_agent(name: &str) -> Self {
        Self {
            creation_type: CreationType::Agent,
            state: CreationState::Discovery,
            name: name.to_string(),
            description: String::new(),
            parameters: Vec::new(),
            command: None,
            skill_type: None,
            command_type: None,
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
                    "🛠️ Creation Assistant - Skill\nCreating {} skill: {}\n\nWhat are you trying to accomplish with this skill?",
                    skill_type_name, self.name
                )
            }
            CreationType::CustomCommand => {
                format!(
                    "🛠️ Creation Assistant - Custom Command\nCreating command: /{}\n\nWhat should this command do?\nExamples:\n- 'run git status'\n- 'deploy to staging'\n- 'create alias for ls -la'\n- 'save current context'",
                    self.name
                )
            }
            CreationType::Agent => {
                format!(
                    "🛠️ Creation Assistant - Agent\nCreating agent: {}\n\nWhat role should this agent play?\nExamples:\n- 'code reviewer for Python'\n- 'documentation writer'\n- 'DevOps troubleshooter'",
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
                "Please provide the command or prompt for your skill:".to_string()
            }
            CreationType::CustomCommand => {
                // Determine command type from user input
                let input_lower = user_input.to_lowercase();
                if input_lower.contains("alias") {
                    self.command_type = Some(CommandType::Alias);
                    "What command should this alias point to?\nExample: 'ls -la' or 'git status --short'"
                } else if input_lower.contains("builtin") || input_lower.contains("save") || input_lower.contains("context") {
                    self.command_type = Some(CommandType::Builtin);
                    "Which builtin function?\nOptions: save_context, clear_context, show_stats"
                } else {
                    self.command_type = Some(CommandType::Script);
                    "What shell command should this execute?\nUse {{param}} for parameters.\nExample: 'git checkout {{branch}}' or 'echo Hello {{name}}'"
                }.to_string()
            }
            CreationType::Agent => {
                "What instructions should guide this agent's behavior?\nExample: 'Review code for security issues and suggest improvements'"
                .to_string()
            }
        }
    }

    pub fn handle_configuration_response(&mut self, user_input: &str) -> String {
        self.command = Some(user_input.to_string());

        match &self.creation_type {
            CreationType::Skill(_) => {
                self.state = CreationState::Completion;
                self.generate_completion_message()
            }
            CreationType::CustomCommand => {
                // Check if command has parameters
                if user_input.contains("{{") && user_input.contains("}}") {
                    "I see your command uses parameters. Let's configure them.\nFor each {{param}} in your command, provide:\nname: required/optional, description\n\nExample:\nbranch: required, Git branch to checkout\nforce: optional, Force checkout".to_string()
                } else {
                    self.state = CreationState::Completion;
                    self.generate_completion_message()
                }
            }
            CreationType::Agent => {
                self.state = CreationState::Completion;
                self.generate_completion_message()
            }
        }
    }

    pub fn handle_parameter_configuration(&mut self, user_input: &str) -> String {
        // Parse parameter configuration
        for line in user_input.lines() {
            if let Some((name, config)) = line.split_once(':') {
                let name = name.trim();
                let parts: Vec<&str> = config.split(',').collect();
                if parts.len() >= 2 {
                    let required = parts[0].trim().to_lowercase().contains("required");
                    let description = parts[1].trim();

                    self.parameters.push(Parameter {
                        name: name.to_string(),
                        description: description.to_string(),
                        required,
                        default_value: if required { None } else { Some("".to_string()) },
                    });
                }
            }
        }

        self.state = CreationState::Completion;
        self.generate_completion_message()
    }

    fn generate_completion_message(&self) -> String {
        match &self.creation_type {
            CreationType::Skill(skill_type) => {
                let skill_type_name = match skill_type {
                    SkillType::CodeInline => "Command",
                    SkillType::PromptInline => "Template",
                    SkillType::Conversation => "Assistant",
                    SkillType::CodeSession => "REPL",
                };

                let mut message = format!(
                    "✅ Skill Ready!\n\nName: {}\nType: {}\nDescription: {}\n",
                    self.name, skill_type_name, self.description
                );

                if let Some(cmd) = &self.command {
                    message.push_str(&format!("Content: {}\n", cmd));
                }

                message.push_str("\nSave this skill? (yes/no)");
                message
            }
            CreationType::CustomCommand => {
                let type_name = match self.command_type.as_ref().unwrap() {
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
            CreationType::Agent => {
                let mut message = format!(
                    "✅ Agent Ready!\n\nName: {}\nDescription: {}\n",
                    self.name, self.description
                );

                if let Some(instructions) = &self.command {
                    message.push_str(&format!("Instructions: {}\n", instructions));
                }

                message.push_str("\nSave this agent? (yes/no)");
                message
            }
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

                let custom_params: Vec<CommandParameter> = self.parameters.iter().map(|p| {
                    CommandParameter {
                        name: p.name.clone(),
                        description: p.description.clone(),
                        required: p.required,
                        default_value: p.default_value.clone(),
                    }
                }).collect();

                let cmd = CustomCommand {
                    name: self.name.clone(),
                    description: self.description.clone(),
                    handler,
                    parameters: custom_params,
                    created_at: chrono::Utc::now().to_rfc3339(),
                    usage_count: 0,
                };

                Ok(CreationArtifact::CustomCommand(cmd))
            }
            CreationType::Skill(_) => {
                // Future: Create skill artifact
                Err("Skill creation not yet implemented in unified assistant".to_string())
            }
            CreationType::Agent => {
                // Future: Create agent artifact
                Err("Agent creation not yet implemented".to_string())
            }
        }
    }

    pub fn is_complete(&self) -> bool {
        matches!(self.state, CreationState::Completion)
    }
}

#[derive(Debug)]
pub enum CreationArtifact {
    Skill(Value), // Future: proper skill type
    CustomCommand(CustomCommand),
    Agent(Value), // Future: proper agent type
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

    pub fn new_agent(name: &str) -> Self {
        Self {
            assistant: UnifiedCreationAssistant::new_agent(name),
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
        match self.assistant.create_artifact()? {
            CreationArtifact::CustomCommand(cmd) => {
                let commands_dir = std::env::current_dir()?.join(".q-commands");
                let mut registry = CustomCommandRegistry::new(commands_dir)?;
                registry.add_command(cmd)?;
            }
            CreationArtifact::Skill(_) => {
                println!("Skill creation not yet implemented in unified assistant");
            }
            CreationArtifact::Agent(_) => {
                println!("Agent creation not yet implemented");
            }
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
