use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;
use crate::cli::skills::types::SkillType;

#[derive(Debug, Clone, PartialEq)]
pub enum CreationState {
    Discovery,      // Understanding what user wants to build
    Configuration,  // Setting up skill parameters
    Testing,        // Testing prompts/functionality
    Completion,     // Finalizing and saving skill
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub name: String,
    pub description: String,
    pub inputs: Value,
    pub expected_output: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TestResult {
    pub test_case_name: String,
    pub passed: bool,
    pub actual_output: String,
    pub expected_output: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SkillTypeConstraints {
    pub requires_command: bool,
    pub supports_prompt_testing: bool,
    pub supports_parameters: bool,
    pub supports_context_files: bool,
    pub supports_session_config: bool,
}

impl SkillTypeConstraints {
    pub fn for_type(skill_type: &SkillType) -> Self {
        match skill_type {
            SkillType::Command => Self {
                requires_command: true,
                supports_prompt_testing: false,
                supports_parameters: true,
                supports_context_files: false,
                supports_session_config: false,
            },
            SkillType::CodeInline => Self {
                requires_command: true,
                supports_prompt_testing: false,
                supports_parameters: false,
                supports_context_files: false,
                supports_session_config: false,
            },
            SkillType::PromptInline => Self {
                requires_command: false,
                supports_prompt_testing: true,
                supports_parameters: true,
                supports_context_files: false,
                supports_session_config: false,
            },
            SkillType::Conversation => Self {
                requires_command: false,
                supports_prompt_testing: true,
                supports_parameters: false,
                supports_context_files: true,
                supports_session_config: false,
            },
            SkillType::CodeSession => Self {
                requires_command: true,
                supports_prompt_testing: false,
                supports_parameters: false,
                supports_context_files: false,
                supports_session_config: true,
            },
        }
    }

    pub fn requires_command(&self) -> bool {
        self.requires_command
    }

    pub fn supports_prompt_testing(&self) -> bool {
        self.supports_prompt_testing
    }
}

#[derive(Debug)]
pub struct SkillCreationSession {
    skill_name: String,
    skill_type: SkillType,
    state: CreationState,
    
    // Common fields
    description: Option<String>,
    
    // Command/REPL fields
    command: Option<String>,
    args: Option<Vec<String>>,
    timeout: Option<u32>,
    
    // Template/Assistant fields
    prompt_template: Option<String>,
    parameters: Vec<crate::cli::skills::types::Parameter>,
    
    // Assistant fields
    context_patterns: Vec<String>,
    max_files: Option<u32>,
    max_file_size_kb: Option<u32>,
    
    // REPL fields
    session_timeout: Option<u64>,
    max_sessions: Option<u32>,
    cleanup_on_exit: Option<bool>,
    
    // Testing
    test_cases: Vec<TestCase>,
    
    // File tracking
    created_files: Vec<PathBuf>,
}

impl SkillCreationSession {
    pub fn new(skill_name: &str, skill_type: SkillType) -> Self {
        Self {
            skill_name: skill_name.to_string(),
            skill_type,
            state: CreationState::Discovery,
            description: None,
            command: None,
            args: None,
            timeout: None,
            prompt_template: None,
            parameters: Vec::new(),
            context_patterns: Vec::new(),
            max_files: None,
            max_file_size_kb: None,
            session_timeout: None,
            max_sessions: None,
            cleanup_on_exit: None,
            test_cases: Vec::new(),
            created_files: Vec::new(),
        }
    }

    // Getters
    pub fn skill_name(&self) -> &str {
        &self.skill_name
    }

    pub fn skill_type(&self) -> &SkillType {
        &self.skill_type
    }

    pub fn state(&self) -> &CreationState {
        &self.state
    }

    pub fn test_cases(&self) -> &[TestCase] {
        &self.test_cases
    }

    pub fn created_files(&self) -> &[PathBuf] {
        &self.created_files
    }

    // State transitions
    pub fn advance_to_configuration(&mut self) {
        self.state = CreationState::Configuration;
    }

    pub fn advance_to_testing(&mut self) {
        self.state = CreationState::Testing;
    }

    pub fn advance_to_completion(&mut self) {
        self.state = CreationState::Completion;
    }

    // Setters
    pub fn set_description(&mut self, description: String) {
        self.description = Some(description);
    }

    pub fn set_command(&mut self, command: String) {
        self.command = Some(command);
    }

    pub fn set_args(&mut self, args: Vec<String>) {
        self.args = Some(args);
    }

    pub fn set_prompt_template(&mut self, template: String) {
        self.prompt_template = Some(template);
    }

    pub fn add_context_pattern(&mut self, pattern: String) {
        self.context_patterns.push(pattern);
    }

    pub fn set_session_timeout(&mut self, timeout: u64) {
        self.session_timeout = Some(timeout);
    }

    pub fn set_max_sessions(&mut self, max: u32) {
        self.max_sessions = Some(max);
    }

    pub fn prompt_template(&self) -> Option<&String> {
        self.prompt_template.as_ref()
    }

    pub fn set_timeout(&mut self, timeout: u32) {
        self.timeout = Some(timeout);
    }

    pub fn set_cleanup_on_exit(&mut self, cleanup: bool) {
        self.cleanup_on_exit = Some(cleanup);
    }

    pub fn set_max_files(&mut self, max: u32) {
        self.max_files = Some(max);
    }

    pub fn set_max_file_size_kb(&mut self, max: u32) {
        self.max_file_size_kb = Some(max);
    }

    // Test case management
    pub fn add_test_case(&mut self, test_case: TestCase) {
        self.test_cases.push(test_case);
    }

    // File creation
    pub fn create_supporting_file(&mut self, path: &PathBuf, content: &str) -> Result<(), std::io::Error> {
        std::fs::write(path, content)?;
        self.created_files.push(path.clone());
        Ok(())
    }

    // Validation
    pub fn validate(&self) -> Result<(), String> {
        let constraints = SkillTypeConstraints::for_type(&self.skill_type);
        
        if constraints.requires_command() && self.command.is_none() {
            return Err("command is required for this skill type".to_string());
        }
        
        Ok(())
    }

    // JSON generation
    pub fn generate_skill_json(&self) -> Value {
        let mut skill = json!({
            "name": self.skill_name,
            "version": "1.0.0",
            "type": match self.skill_type {
                SkillType::Command => "command",
                SkillType::CodeInline => "code_inline",
                SkillType::PromptInline => "prompt_inline", 
                SkillType::Conversation => "conversation",
                SkillType::CodeSession => "code_session",
            }
        });

        if let Some(desc) = &self.description {
            skill["description"] = json!(desc);
        }

        if let Some(cmd) = &self.command {
            skill["command"] = json!(cmd);
        }

        if let Some(args) = &self.args {
            skill["args"] = json!(args);
        }

        if let Some(template) = &self.prompt_template {
            skill["prompt"] = json!(template);
        }

        if !self.context_patterns.is_empty() {
            skill["context_files"] = json!({
                "patterns": self.context_patterns,
                "max_files": self.max_files.unwrap_or(10),
                "max_file_size_kb": self.max_file_size_kb.unwrap_or(100)
            });
        }

        if let Some(timeout) = self.session_timeout {
            skill["session_config"] = json!({
                "session_timeout": timeout,
                "max_sessions": self.max_sessions.unwrap_or(5),
                "cleanup_on_exit": self.cleanup_on_exit.unwrap_or(true)
            });
        }

        skill
    }

    // Template testing
    pub fn test_template(&self, inputs: &Value) -> Result<String, String> {
        let template = self.prompt_template.as_ref()
            .ok_or("No template set")?;
        
        let mut result = template.clone();
        if let Value::Object(map) = inputs {
            for (key, value) in map {
                let placeholder = format!("{{{}}}", key);
                let replacement = match value {
                    Value::String(s) => s.clone(),
                    _ => value.to_string(),
                };
                result = result.replace(&placeholder, &replacement);
            }
        }
        
        Ok(result)
    }

    // Test execution
    pub fn run_all_tests(&self) -> Vec<TestResult> {
        self.test_cases.iter().map(|test_case| {
            let actual_output = match self.test_template(&test_case.inputs) {
                Ok(output) => output,
                Err(e) => return TestResult {
                    test_case_name: test_case.name.clone(),
                    passed: false,
                    actual_output: String::new(),
                    expected_output: test_case.expected_output.clone(),
                    error: Some(e),
                },
            };

            let passed = if let Some(expected) = &test_case.expected_output {
                actual_output == *expected
            } else {
                true // No expected output means we just check it runs
            };

            TestResult {
                test_case_name: test_case.name.clone(),
                passed,
                actual_output,
                expected_output: test_case.expected_output.clone(),
                error: None,
            }
        }).collect()
    }
}
