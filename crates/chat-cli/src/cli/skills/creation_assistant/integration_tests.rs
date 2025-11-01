#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::cli::skills::types::SkillType;
    use serde_json::json;
    use tempfile::TempDir;
    use std::fs;

    #[tokio::test]
    async fn test_complete_command_skill_creation_workflow() {
        let temp_dir = TempDir::new().unwrap();
        let skills_dir = temp_dir.path().join(".q-skills");
        
        let mut assistant = SkillCreationAssistant::new("backup-db", SkillType::CodeInline);
        
        // Discovery phase
        let discovery_prompt = assistant.start_discovery();
        assert!(discovery_prompt.contains("Creating command skill: backup-db"));
        assert!(discovery_prompt.contains("What are you trying to accomplish"));
        
        // User responds with their goal
        let config_prompt = assistant.handle_discovery_response("Create database backups");
        assert!(config_prompt.contains("What command should it run"));
        
        // User provides command
        let test_prompt = assistant.handle_configuration_response("pg_dump myapp_production");
        assert!(test_prompt.contains("Ready to test"));
        
        // Save the skill
        assistant.save_skill(&skills_dir).unwrap();
        
        // Verify skill file was created
        let skill_file = skills_dir.join("backup-db.json");
        assert!(skill_file.exists());
        
        // Verify skill content
        let skill_content: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(&skill_file).unwrap()
        ).unwrap();
        
        assert_eq!(skill_content["name"], "backup-db");
        assert_eq!(skill_content["type"], "code_inline");
        assert_eq!(skill_content["command"], "pg_dump myapp_production");
        assert_eq!(skill_content["description"], "Create database backups");
    }

    #[tokio::test]
    async fn test_complete_template_skill_creation_with_testing() {
        let temp_dir = TempDir::new().unwrap();
        let skills_dir = temp_dir.path().join(".q-skills");
        
        let mut assistant = SkillCreationAssistant::new("email-gen", SkillType::PromptInline);
        
        // Discovery phase
        let discovery_prompt = assistant.start_discovery();
        assert!(discovery_prompt.contains("Creating template skill: email-gen"));
        
        // User describes what they want
        let config_prompt = assistant.handle_discovery_response("Generate professional email responses");
        assert!(config_prompt.contains("What should this template generate"));
        
        // User provides template
        let test_prompt = assistant.handle_configuration_response("Dear {recipient}, Thank you for {reason}. Best regards, {sender}");
        assert!(test_prompt.contains("Testing your skill"));
        assert!(test_prompt.contains("Dear Alice, Thank you for"));
        
        // User approves the template
        let completion = assistant.handle_testing_response("looks good");
        assert!(completion.contains("Skill created successfully"));
        assert!(completion.contains("email-gen"));
        
        // Save and verify
        assistant.save_skill(&skills_dir).unwrap();
        
        let skill_file = skills_dir.join("email-gen.json");
        assert!(skill_file.exists());
        
        let test_file = skills_dir.join("email-gen.tests.json");
        assert!(test_file.exists());
        
        // Verify skill content
        let skill_content: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(&skill_file).unwrap()
        ).unwrap();
        
        assert_eq!(skill_content["name"], "email-gen");
        assert_eq!(skill_content["type"], "prompt_inline");
        assert_eq!(skill_content["prompt"], "Dear {recipient}, Thank you for {reason}. Best regards, {sender}");
    }

    #[tokio::test]
    async fn test_template_refinement_workflow() {
        let mut assistant = SkillCreationAssistant::new("greeting", SkillType::PromptInline);
        
        // Go through discovery and configuration
        assistant.handle_discovery_response("Generate personalized greetings");
        let test_prompt = assistant.handle_configuration_response("Hello {name}!");
        assert!(test_prompt.contains("Hello Alice!"));
        
        // User wants to refine
        let refinement_prompt = assistant.handle_testing_response("Add more warmth to the greeting");
        assert!(refinement_prompt.contains("What changes would you like"));
        
        // User provides refined template
        let new_test = assistant.handle_refinement("", "Hello {name}! It's wonderful to see you today!");
        assert!(new_test.contains("Hello Alice! It's wonderful to see you today!"));
        
        // User approves refined version
        let completion = assistant.handle_testing_response("perfect");
        assert!(completion.contains("Skill created successfully"));
    }

    #[tokio::test]
    async fn test_assistant_skill_creation_with_context() {
        let mut assistant = SkillCreationAssistant::new("code-reviewer", SkillType::Conversation);
        
        // Discovery and configuration
        assistant.handle_discovery_response("Review code for best practices and security");
        let test_prompt = assistant.handle_configuration_response("You are a senior code reviewer. Analyze code for best practices, security issues, and maintainability.");
        
        // Should automatically add context files for conversation skills
        let skill_json = assistant.session().generate_skill_json();
        let context_files = &skill_json["context_files"];
        
        assert_eq!(context_files["patterns"], json!(["*.rs", "*.py"]));
        assert_eq!(context_files["max_files"], 10);
        assert_eq!(context_files["max_file_size_kb"], 100);
    }

    #[tokio::test]
    async fn test_repl_skill_creation_with_session_config() {
        let mut assistant = SkillCreationAssistant::new("python-env", SkillType::CodeSession);
        
        // Discovery and configuration
        assistant.handle_discovery_response("Interactive Python environment for data analysis");
        assistant.handle_configuration_response("python3");
        
        let skill_json = assistant.session().generate_skill_json();
        let session_config = &skill_json["session_config"];
        
        assert_eq!(session_config["session_timeout"], 3600);
        assert_eq!(session_config["max_sessions"], 5);
        assert_eq!(session_config["cleanup_on_exit"], true);
    }

    #[tokio::test]
    async fn test_test_case_addition_during_creation() {
        let mut assistant = SkillCreationAssistant::new("formatter", SkillType::PromptInline);
        
        // Set up template
        assistant.session_mut().set_prompt_template("Format: {text} -> {style}".to_string());
        assistant.session_mut().advance_to_testing();
        
        // Add custom test case
        let test_inputs = json!({"text": "hello world", "style": "uppercase"});
        let result = assistant.add_test_case("uppercase_test", "Test uppercase formatting", test_inputs);
        
        assert!(result.contains("Test case 'uppercase_test' added"));
        assert!(result.contains("Format: hello world -> uppercase"));
        assert_eq!(assistant.session().test_cases().len(), 1);
    }

    #[tokio::test]
    async fn test_skill_validation_prevents_incomplete_skills() {
        let mut assistant = SkillCreationAssistant::new("incomplete", SkillType::CodeInline);
        
        // Try to complete without setting required command
        let validation = assistant.session().validate();
        assert!(validation.is_err());
        assert!(validation.unwrap_err().contains("command is required"));
        
        // Add required command
        assistant.session_mut().set_command("echo".to_string());
        let validation = assistant.session().validate();
        assert!(validation.is_ok());
    }

    #[tokio::test]
    async fn test_file_creation_and_tracking() {
        let temp_dir = TempDir::new().unwrap();
        let mut assistant = SkillCreationAssistant::new("script-runner", SkillType::CodeInline);
        
        // Create supporting script
        let script_path = temp_dir.path().join("helper-script.sh");
        assistant.session_mut().create_supporting_file(&script_path, "#!/bin/bash\necho 'Helper script'").unwrap();
        
        // Verify file was created and tracked
        assert!(script_path.exists());
        assert!(assistant.session().created_files().contains(&script_path));
        
        // Verify script content
        let content = fs::read_to_string(&script_path).unwrap();
        assert!(content.contains("Helper script"));
    }

    #[tokio::test]
    async fn test_end_to_end_skill_creation_and_loading() {
        let temp_dir = TempDir::new().unwrap();
        let skills_dir = temp_dir.path().join(".q-skills");
        
        // Create skill through assistant
        let mut assistant = SkillCreationAssistant::new("test-integration", SkillType::CodeInline);
        assistant.handle_discovery_response("Test integration workflow");
        assistant.handle_configuration_response("echo 'Integration test'");
        assistant.save_skill(&skills_dir).unwrap();
        
        // Verify skill can be loaded by registry
        let registry = crate::cli::skills::SkillRegistry::with_workspace_skills(temp_dir.path()).await.unwrap();
        let loaded_skill = registry.get("test-integration");
        
        assert!(loaded_skill.is_some());
        assert_eq!(loaded_skill.unwrap().name(), "test-integration");
        assert_eq!(loaded_skill.unwrap().description(), "Test integration workflow");
    }
}
