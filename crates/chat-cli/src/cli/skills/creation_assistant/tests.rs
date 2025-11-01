#[cfg(test)]
mod skill_creation_assistant_tests {
    use crate::cli::skills::creation_assistant::types::*;
    use crate::cli::skills::types::SkillType;
    use serde_json::json;
    use tempfile::TempDir;

    #[test]
    fn test_skill_creation_session_initialization() {
        let session = SkillCreationSession::new("test-skill", SkillType::CodeInline);
        
        assert_eq!(session.skill_name(), "test-skill");
        assert_eq!(session.skill_type(), &SkillType::CodeInline);
        assert_eq!(session.state(), &CreationState::Discovery);
        assert!(session.test_cases().is_empty());
    }

    #[test]
    fn test_skill_type_constraints() {
        let command_constraints = SkillTypeConstraints::for_type(&SkillType::CodeInline);
        assert!(command_constraints.requires_command());
        assert!(!command_constraints.supports_prompt_testing());
        
        let template_constraints = SkillTypeConstraints::for_type(&SkillType::PromptInline);
        assert!(!template_constraints.requires_command());
        assert!(template_constraints.supports_prompt_testing());
    }

    #[test]
    fn test_test_case_management() {
        let mut session = SkillCreationSession::new("email-gen", SkillType::PromptInline);
        
        let test_case = TestCase {
            name: "basic_email".to_string(),
            description: "Standard professional email".to_string(),
            inputs: json!({
                "recipient": "John",
                "sender": "Alice"
            }),
            expected_output: Some("Professional email format".to_string()),
        };
        
        session.add_test_case(test_case.clone());
        assert_eq!(session.test_cases().len(), 1);
        assert_eq!(session.test_cases()[0].name, "basic_email");
    }

    #[test]
    fn test_skill_json_generation() {
        let mut session = SkillCreationSession::new("test-cmd", SkillType::CodeInline);
        session.set_command("echo".to_string());
        session.set_args(vec!["Hello".to_string()]);
        session.set_description("Test command skill".to_string());
        
        let skill_json = session.generate_skill_json();
        
        assert_eq!(skill_json["name"], "test-cmd");
        assert_eq!(skill_json["type"], "code_inline");
        assert_eq!(skill_json["command"], "echo");
        assert_eq!(skill_json["args"], json!(["Hello"]));
    }

    #[test]
    fn test_template_skill_prompt_testing() {
        let mut session = SkillCreationSession::new("greeting", SkillType::PromptInline);
        session.set_prompt_template("Hello {name}!".to_string());
        
        let test_inputs = json!({"name": "Alice"});
        let result = session.test_template(&test_inputs);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello Alice!");
    }

    #[test]
    fn test_conversation_flow_state_transitions() {
        let mut session = SkillCreationSession::new("test", SkillType::CodeInline);
        
        assert_eq!(session.state(), &CreationState::Discovery);
        
        session.advance_to_configuration();
        assert_eq!(session.state(), &CreationState::Configuration);
        
        session.advance_to_testing();
        assert_eq!(session.state(), &CreationState::Testing);
        
        session.advance_to_completion();
        assert_eq!(session.state(), &CreationState::Completion);
    }

    #[test]
    fn test_file_creation_tracking() {
        let temp_dir = TempDir::new().unwrap();
        let mut session = SkillCreationSession::new("script-runner", SkillType::CodeInline);
        
        let script_path = temp_dir.path().join("test-script.sh");
        session.create_supporting_file(&script_path, "#!/bin/bash\necho 'test'").unwrap();
        
        assert!(script_path.exists());
        assert!(session.created_files().contains(&script_path));
    }

    #[test]
    fn test_skill_validation() {
        let mut session = SkillCreationSession::new("incomplete", SkillType::CodeInline);
        
        // Missing required command
        let validation = session.validate();
        assert!(validation.is_err());
        assert!(validation.unwrap_err().contains("command is required"));
        
        // Add required fields
        session.set_command("echo".to_string());
        let validation = session.validate();
        assert!(validation.is_ok());
    }

    #[test]
    fn test_test_case_execution() {
        let mut session = SkillCreationSession::new("template-test", SkillType::PromptInline);
        session.set_prompt_template("Welcome {user} to {place}!".to_string());
        
        let test_case = TestCase {
            name: "basic_welcome".to_string(),
            description: "Basic welcome message".to_string(),
            inputs: json!({
                "user": "Alice",
                "place": "Wonderland"
            }),
            expected_output: Some("Welcome Alice to Wonderland!".to_string()),
        };
        
        session.add_test_case(test_case);
        let results = session.run_all_tests();
        
        assert_eq!(results.len(), 1);
        assert!(results[0].passed);
        assert_eq!(results[0].actual_output, "Welcome Alice to Wonderland!");
    }

    #[test]
    fn test_assistant_skill_context_files() {
        let mut session = SkillCreationSession::new("code-reviewer", SkillType::Conversation);
        session.set_prompt_template("Review this code".to_string());
        session.add_context_pattern("*.rs".to_string());
        session.add_context_pattern("*.py".to_string());
        
        let skill_json = session.generate_skill_json();
        let context_files = &skill_json["context_files"];
        
        assert_eq!(context_files["patterns"], json!(["*.rs", "*.py"]));
        assert_eq!(context_files["max_files"], 10);
        assert_eq!(context_files["max_file_size_kb"], 100);
    }

    #[test]
    fn test_repl_skill_session_config() {
        let mut session = SkillCreationSession::new("python-repl", SkillType::CodeSession);
        session.set_command("python3".to_string());
        session.set_session_timeout(3600);
        session.set_max_sessions(5);
        
        let skill_json = session.generate_skill_json();
        let session_config = &skill_json["session_config"];
        
        assert_eq!(session_config["session_timeout"], 3600);
        assert_eq!(session_config["max_sessions"], 5);
        assert_eq!(session_config["cleanup_on_exit"], true);
    }
}
