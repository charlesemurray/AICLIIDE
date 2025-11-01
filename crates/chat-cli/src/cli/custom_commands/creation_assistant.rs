use super::*;
use std::io::{self, Write};

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
    Completion,     // Finalizing and saving
}

pub struct CustomCommandCreationAssistant {
    state: CreationState,
    name: String,
    description: String,
    command_type: CommandType,
    command: Option<String>,
    parameters: Vec<CommandParameter>,
}

impl CustomCommandCreationAssistant {
    pub fn new(name: &str) -> Self {
        Self {
            state: CreationState::Discovery,
            name: name.to_string(),
            description: String::new(),
            command_type: CommandType::Script,
            command: None,
            parameters: Vec::new(),
        }
    }

    pub fn start_discovery(&self) -> String {
        format!(
            "🛠️ Custom Command Creation Assistant\nCreating command: /{}\n\nWhat should this command do?\nExamples:\n- 'run git status'\n- 'deploy to staging'\n- 'create alias for ls -la'\n- 'save current context'",
            self.name
        )
    }

    pub fn handle_discovery_response(&mut self, user_input: &str) -> String {
        self.description = user_input.to_string();
        self.state = CreationState::Configuration;

        // Determine command type from user input
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("alias") {
            self.command_type = CommandType::Alias;
            "What command should this alias point to?\nExample: 'ls -la' or 'git status --short'"
        } else if input_lower.contains("builtin") || input_lower.contains("save") || input_lower.contains("context") {
            self.command_type = CommandType::Builtin;
            "Which builtin function?\nOptions: save_context, clear_context, show_stats"
        } else {
            self.command_type = CommandType::Script;
            "What shell command should this execute?\nUse {{param}} for parameters.\nExample: 'git checkout {{branch}}' or 'echo Hello {{name}}'"
        }.to_string()
    }

    pub fn handle_configuration_response(&mut self, user_input: &str) -> String {
        self.command = Some(user_input.to_string());

        // Check if command has parameters
        if user_input.contains("{{") && user_input.contains("}}") {
            "I see your command uses parameters. Let's configure them.\nFor each {{param}} in your command, provide:\nname: required/optional, description\n\nExample:\nbranch: required, Git branch to checkout\nforce: optional, Force checkout".to_string()
        } else {
            self.state = CreationState::Completion;
            self.generate_completion_message()
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

                    self.parameters.push(CommandParameter {
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
        let type_name = match self.command_type {
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

    pub fn create_command(&self) -> Result<CustomCommand, CommandError> {
        let command = self.command.as_ref().ok_or_else(|| 
            CommandError::InvalidParameter("No command specified".to_string()))?;

        let handler = match self.command_type {
            CommandType::Script => {
                CommandHandler::Script { 
                    command: command.clone(), 
                    args: vec![] 
                }
            }
            CommandType::Alias => {
                CommandHandler::Alias { 
                    target: command.clone() 
                }
            }
            CommandType::Builtin => {
                CommandHandler::Builtin { 
                    function_name: command.clone() 
                }
            }
        };

        Ok(CustomCommand {
            name: self.name.clone(),
            description: self.description.clone(),
            handler,
            parameters: self.parameters.clone(),
            created_at: chrono::Utc::now().to_rfc3339(),
            usage_count: 0,
        })
    }

    pub fn is_complete(&self) -> bool {
        matches!(self.state, CreationState::Completion)
    }
}

pub struct CustomCommandCreationCLI {
    assistant: CustomCommandCreationAssistant,
}

impl CustomCommandCreationCLI {
    pub fn new(name: &str) -> Self {
        Self {
            assistant: CustomCommandCreationAssistant::new(name),
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
                self.save_command().await?;
                println!("✅ Command created successfully!");
                println!("You can now use it with: /{}", self.assistant.name);
            } else {
                println!("❌ Command creation cancelled.");
            }
        }
        
        Ok(())
    }

    async fn save_command(&self) -> Result<(), Box<dyn std::error::Error>> {
        let cmd = self.assistant.create_command()?;
        let commands_dir = std::env::current_dir()?.join(".q-commands");
        let mut registry = CustomCommandRegistry::new(commands_dir)?;
        registry.add_command(cmd)?;
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
