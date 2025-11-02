use std::io::{
    self,
    Write,
};

use super::*;
use crate::cli::skills::types::SkillType;

pub struct SkillCreationCLI {
    assistant: SkillCreationAssistant,
}

impl SkillCreationCLI {
    pub fn new(skill_name: &str, skill_type: SkillType) -> Self {
        Self {
            assistant: SkillCreationAssistant::new(skill_name, skill_type),
        }
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("{}", self.assistant.start_discovery());

        // Discovery phase
        let user_input = self.get_user_input()?;
        println!("{}", self.assistant.handle_discovery_response(&user_input));

        // Configuration phase
        let user_input = self.get_user_input()?;
        println!("{}", self.assistant.handle_configuration_response(&user_input));

        // Testing phase (if applicable)
        if matches!(self.assistant.session().state(), CreationState::Testing) {
            loop {
                let user_input = self.get_user_input()?;
                let response = self.assistant.handle_testing_response(&user_input);
                println!("{}", response);

                if matches!(self.assistant.session().state(), CreationState::Completion) {
                    break;
                }
            }
        }

        // Save the skill
        let skills_dir = std::env::current_dir()?.join(".q-skills");
        self.assistant.save_skill(&skills_dir)?;

        Ok(())
    }

    fn get_user_input(&self) -> Result<String, std::io::Error> {
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        Ok(input.trim().to_string())
    }
}

// Helper function to map user-friendly types to internal types
pub fn map_user_type_to_skill_type(user_type: &str) -> Option<SkillType> {
    match user_type {
        "command" => Some(SkillType::CodeInline),
        "repl" => Some(SkillType::CodeSession),
        "assistant" => Some(SkillType::Conversation),
        "template" => Some(SkillType::PromptInline),
        _ => None,
    }
}
