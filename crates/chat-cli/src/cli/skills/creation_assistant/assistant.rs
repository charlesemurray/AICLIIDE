use std::path::Path;

use serde_json::{Value, json};

use super::types::*;
use crate::cli::skills::types::SkillType;

pub struct SkillCreationAssistant {
    session: SkillCreationSession,
}

impl SkillCreationAssistant {
    pub fn new(skill_name: &str, skill_type: SkillType) -> Self {
        Self {
            session: SkillCreationSession::new(skill_name, skill_type),
        }
    }

    pub fn session(&self) -> &SkillCreationSession {
        &self.session
    }

    pub fn session_mut(&mut self) -> &mut SkillCreationSession {
        &mut self.session
    }

    // Discovery phase - understand what user wants to build
    pub fn start_discovery(&mut self) -> String {
        let skill_type_name = match self.session.skill_type() {
            SkillType::Command => "command",
            SkillType::CodeInline => "command",
            SkillType::PromptInline => "template",
            SkillType::Conversation => "assistant",
            SkillType::CodeSession => "REPL",
        };

        format!(
            "🛠️ Skill Creation Assistant\nCreating {} skill: {}\n\nWhat are you trying to accomplish with this skill?",
            skill_type_name,
            self.session.skill_name()
        )
    }

    // Configuration phase - set up skill parameters
    pub fn handle_discovery_response(&mut self, user_input: &str) -> String {
        // Store user's goal as description
        self.session.set_description(user_input.to_string());
        self.session.advance_to_configuration();

        let constraints = SkillTypeConstraints::for_type(self.session.skill_type());

        if constraints.requires_command() {
            self.prompt_for_command()
        } else if constraints.supports_prompt_testing() {
            self.prompt_for_template()
        } else {
            "Configuration complete. Ready to test!".to_string()
        }
    }

    fn prompt_for_command(&self) -> String {
        match self.session.skill_type() {
            SkillType::CodeInline => {
                "Since this is a command skill, it will execute a single command.\nWhat command should it run? (e.g., python script, existing tool)"
            },
            SkillType::CodeSession => {
                "This skill starts an interactive session.\nWhat interpreter should it use? (e.g., python3, node, bash)"
            },
            _ => "What command should this skill execute?"
        }.to_string()
    }

    fn prompt_for_template(&self) -> String {
        match self.session.skill_type() {
            SkillType::PromptInline => {
                "What should this template generate?\nUse {parameter_name} for variables.\nExample: Generate documentation for {function_name}"
            },
            SkillType::Conversation => {
                "What role should this AI assistant have?\nExample: You are a code reviewer who focuses on security and best practices"
            },
            _ => "What template should this skill use?"
        }.to_string()
    }

    pub fn handle_configuration_response(&mut self, user_input: &str) -> String {
        let constraints = SkillTypeConstraints::for_type(self.session.skill_type());

        if constraints.requires_command() {
            self.session.set_command(user_input.to_string());

            if matches!(self.session.skill_type(), SkillType::CodeSession) {
                self.session.set_session_timeout(3600);
                self.session.set_max_sessions(5);
            }

            self.session.advance_to_testing();
            "Command configured! Ready to test the skill.".to_string()
        } else if constraints.supports_prompt_testing() {
            self.session.set_prompt_template(user_input.to_string());

            if matches!(self.session.skill_type(), SkillType::Conversation) {
                self.session.add_context_pattern("*.rs".to_string());
                self.session.add_context_pattern("*.py".to_string());
            }

            self.session.advance_to_testing();
            self.start_testing()
        } else {
            "Configuration complete!".to_string()
        }
    }

    // Testing phase - test prompts and functionality
    fn start_testing(&mut self) -> String {
        let constraints = SkillTypeConstraints::for_type(self.session.skill_type());

        if constraints.supports_prompt_testing() {
            self.create_initial_test_case();
            self.run_test_and_show_results()
        } else {
            self.session.advance_to_completion();
            self.complete_skill_creation()
        }
    }

    fn create_initial_test_case(&mut self) {
        match self.session.skill_type() {
            SkillType::PromptInline => {
                let test_case = TestCase {
                    name: "basic_test".to_string(),
                    description: "Basic template test".to_string(),
                    inputs: json!({"name": "Alice", "place": "Wonderland"}),
                    expected_output: None,
                };
                self.session.add_test_case(test_case);
            },
            SkillType::Conversation => {
                let test_case = TestCase {
                    name: "sample_input".to_string(),
                    description: "Test assistant response".to_string(),
                    inputs: json!({"input": "Review this function: def add(a, b): return a + b"}),
                    expected_output: None,
                };
                self.session.add_test_case(test_case);
            },
            _ => {},
        }
    }

    fn run_test_and_show_results(&mut self) -> String {
        let results = self.session.run_all_tests();

        if results.is_empty() {
            return "No tests to run.".to_string();
        }

        let mut output = String::from("🧪 Testing your skill:\n\n");

        for result in &results {
            output.push_str(&format!("Test: {}\n", result.test_case_name));
            output.push_str(&format!("Result: {}\n", result.actual_output));

            if let Some(error) = &result.error {
                output.push_str(&format!("Error: {}\n", error));
            }
            output.push('\n');
        }

        output.push_str("Does this look right, or should we refine the template?\n");
        output.push_str("(Type 'looks good' to continue, or describe changes needed)");

        output
    }

    pub fn handle_testing_response(&mut self, user_input: &str) -> String {
        if user_input.to_lowercase().contains("looks good")
            || user_input.to_lowercase().contains("perfect")
            || user_input.to_lowercase().contains("ready")
        {
            self.session.advance_to_completion();
            self.complete_skill_creation()
        } else {
            // User wants to refine the template
            format!(
                "I understand you want to refine the template. What changes would you like?\nCurrent template: {:?}",
                self.session.prompt_template()
            )
        }
    }

    pub fn handle_refinement(&mut self, user_input: &str, new_template: &str) -> String {
        self.session.set_prompt_template(new_template.to_string());

        // Re-run tests with new template
        self.run_test_and_show_results()
    }

    // Completion phase - save skill and provide usage info
    fn complete_skill_creation(&mut self) -> String {
        let skill_json = self.session.generate_skill_json();
        let skill_name = self.session.skill_name();

        // In a real implementation, this would save to .q-skills/
        // For now, we'll just show what would be saved

        let mut output = String::from("✅ Skill created successfully!\n\n");
        output.push_str(&format!("Skill: {}\n", skill_name));
        output.push_str(&format!("Type: {:?}\n", self.session.skill_type()));

        if let Some(desc) = skill_json.get("description") {
            output.push_str(&format!("Description: {}\n", desc.as_str().unwrap_or("")));
        }

        if let Some(cmd) = skill_json.get("command") {
            output.push_str(&format!("Command: {}\n", cmd.as_str().unwrap_or("")));
        }

        if let Some(template) = skill_json.get("prompt") {
            output.push_str(&format!("Template: {}\n", template.as_str().unwrap_or("")));
        }

        output.push_str(&format!("\nUse: /skills run {}", skill_name));

        if !self.session.test_cases().is_empty() {
            output.push_str(" --params '{\"param\":\"value\"}'");
        }

        output.push_str("\n\nReturning to main interface...");
        output
    }

    // Add test case during testing phase
    pub fn add_test_case(&mut self, name: &str, description: &str, inputs: Value) -> String {
        let test_case = TestCase {
            name: name.to_string(),
            description: description.to_string(),
            inputs,
            expected_output: None,
        };

        self.session.add_test_case(test_case);
        format!(
            "✅ Test case '{}' added. Running all tests...\n\n{}",
            name,
            self.run_test_and_show_results()
        )
    }

    // Save skill to filesystem
    pub fn save_skill(&self, skills_dir: &Path) -> Result<(), std::io::Error> {
        let skill_json = self.session.generate_skill_json();
        let skill_file = skills_dir.join(format!("{}.json", self.session.skill_name()));

        std::fs::create_dir_all(skills_dir)?;
        std::fs::write(&skill_file, serde_json::to_string_pretty(&skill_json)?)?;

        // Save test cases if any exist
        if !self.session.test_cases().is_empty() {
            let test_file = skills_dir.join(format!("{}.tests.json", self.session.skill_name()));
            let test_data = json!({
                "test_cases": self.session.test_cases()
            });
            std::fs::write(&test_file, serde_json::to_string_pretty(&test_data)?)?;
        }

        Ok(())
    }
}
