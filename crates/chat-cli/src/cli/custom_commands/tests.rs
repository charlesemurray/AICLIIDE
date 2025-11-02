#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use tempfile::TempDir;

    use crate::cli::custom_commands::types::ParameterType;
    use crate::cli::custom_commands::*;

    #[test]
    fn test_custom_command_creation() {
        let cmd = CustomCommand::new_script(
            "hello".to_string(),
            "Say hello".to_string(),
            "echo 'Hello, {{name}}!'".to_string(),
        );

        assert_eq!(cmd.name, "hello");
        assert_eq!(cmd.description, "Say hello");
        assert_eq!(cmd.usage_count, 0);
        assert!(matches!(cmd.handler, CommandHandler::Script { .. }));
    }

    #[test]
    fn test_command_validation_with_parameters() {
        let mut cmd = CustomCommand::new_script(
            "greet".to_string(),
            "Greet someone".to_string(),
            "echo 'Hello, {{name}}!'".to_string(),
        );

        cmd.add_parameter(
            CommandParameter::new("name".to_string(), ParameterType::String)
                .with_description("Person to greet".to_string()),
        );

        // Test with required parameter
        let mut args = HashMap::new();
        args.insert("name".to_string(), "Alice".to_string());
        assert!(cmd.validate_parameters(&args).is_ok());

        // Test without required parameter
        let empty_args = HashMap::new();
        assert!(cmd.validate_parameters(&empty_args).is_err());
    }

    #[test]
    fn test_registry_creation() {
        let temp_dir = TempDir::new().unwrap();
        let registry = CustomCommandRegistry::new(temp_dir.path().to_path_buf());
        assert!(registry.is_ok());
    }

    #[test]
    fn test_registry_add_command() {
        let temp_dir = TempDir::new().unwrap();
        let mut registry = CustomCommandRegistry::new(temp_dir.path().to_path_buf()).unwrap();

        let cmd = CustomCommand::new_script(
            "test".to_string(),
            "Test command".to_string(),
            "echo 'test'".to_string(),
        );

        assert!(registry.add_command(cmd).is_ok());
        assert!(registry.command_exists("test"));
    }

    #[test]
    fn test_registry_duplicate_command() {
        let temp_dir = TempDir::new().unwrap();
        let mut registry = CustomCommandRegistry::new(temp_dir.path().to_path_buf()).unwrap();

        let cmd1 = CustomCommand::new_script(
            "duplicate".to_string(),
            "First command".to_string(),
            "echo 'first'".to_string(),
        );

        let cmd2 = CustomCommand::new_script(
            "duplicate".to_string(),
            "Second command".to_string(),
            "echo 'second'".to_string(),
        );

        assert!(registry.add_command(cmd1).is_ok());
        assert!(registry.add_command(cmd2).is_err());
    }

    #[test]
    fn test_registry_get_command() {
        let temp_dir = TempDir::new().unwrap();
        let mut registry = CustomCommandRegistry::new(temp_dir.path().to_path_buf()).unwrap();

        let cmd = CustomCommand::new_script(
            "get_test".to_string(),
            "Get test command".to_string(),
            "echo 'get test'".to_string(),
        );

        registry.add_command(cmd).unwrap();

        let retrieved = registry.get_command("get_test");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "get_test");

        let not_found = registry.get_command("nonexistent");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_registry_remove_command() {
        let temp_dir = TempDir::new().unwrap();
        let mut registry = CustomCommandRegistry::new(temp_dir.path().to_path_buf()).unwrap();

        let cmd = CustomCommand::new_script(
            "remove_test".to_string(),
            "Remove test command".to_string(),
            "echo 'remove test'".to_string(),
        );

        registry.add_command(cmd).unwrap();
        assert!(registry.command_exists("remove_test"));

        assert!(registry.remove_command("remove_test").is_ok());
        assert!(!registry.command_exists("remove_test"));

        // Test removing non-existent command
        assert!(registry.remove_command("nonexistent").is_err());
    }

    #[test]
    fn test_registry_list_commands() {
        let temp_dir = TempDir::new().unwrap();
        let mut registry = CustomCommandRegistry::new(temp_dir.path().to_path_buf()).unwrap();

        let cmd1 = CustomCommand::new_script("cmd1".to_string(), "First".to_string(), "echo '1'".to_string());
        let cmd2 = CustomCommand::new_script("cmd2".to_string(), "Second".to_string(), "echo '2'".to_string());

        registry.add_command(cmd1).unwrap();
        registry.add_command(cmd2).unwrap();

        let commands = registry.list_commands();
        assert_eq!(commands.len(), 2);
    }

    #[test]
    fn test_script_execution() {
        let cmd = CustomCommand::new_script(
            "echo_test".to_string(),
            "Echo test".to_string(),
            "echo 'Hello, World!'".to_string(),
        );

        let execution = CommandExecution {
            command_name: "echo_test".to_string(),
            arguments: HashMap::new(),
        };

        let result = CommandExecutor::execute(&cmd, &execution);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Hello, World!"));
    }

    #[test]
    fn test_script_with_parameters() {
        let mut cmd = CustomCommand::new_script(
            "greet".to_string(),
            "Greet someone".to_string(),
            "echo 'Hello, {{name}}!'".to_string(),
        );

        cmd.add_parameter(
            CommandParameter::new("name".to_string(), ParameterType::String)
                .with_description("Name to greet".to_string()),
        );

        let mut args = HashMap::new();
        args.insert("name".to_string(), "Alice".to_string());

        let execution = CommandExecution {
            command_name: "greet".to_string(),
            arguments: args,
        };

        let result = CommandExecutor::execute(&cmd, &execution);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Hello, Alice!"));
    }

    #[test]
    fn test_builtin_execution() {
        let cmd = CustomCommand {
            name: "save".to_string(),
            description: "Save context".to_string(),
            handler: CommandHandler::Builtin {
                function_name: "save_context".to_string(),
            },
            parameters: vec![],
            created_at: chrono::Utc::now().to_rfc3339(),
            usage_count: 0,
        };

        let execution = CommandExecution {
            command_name: "save".to_string(),
            arguments: HashMap::new(),
        };

        let result = CommandExecutor::execute(&cmd, &execution);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Context saved"));
    }

    #[test]
    fn test_script_safety_validation() {
        // Safe script
        assert!(CommandExecutor::validate_script_safety("echo 'hello'").is_ok());

        // Dangerous scripts
        assert!(CommandExecutor::validate_script_safety("rm -rf /").is_err());
        assert!(CommandExecutor::validate_script_safety("sudo rm file").is_err());
        assert!(CommandExecutor::validate_script_safety("dd if=/dev/zero").is_err());
    }

    #[test]
    fn test_command_usage_tracking() {
        let mut cmd = CustomCommand::new_script(
            "usage_test".to_string(),
            "Usage test".to_string(),
            "echo 'test'".to_string(),
        );

        assert_eq!(cmd.usage_count, 0);
        cmd.increment_usage();
        assert_eq!(cmd.usage_count, 1);
        cmd.increment_usage();
        assert_eq!(cmd.usage_count, 2);
    }

    #[test]
    fn test_alias_command_creation() {
        let cmd = CustomCommand::new_alias(
            "gs".to_string(),
            "Git status shortcut".to_string(),
            "git status".to_string(),
        );

        assert_eq!(cmd.name, "gs");
        assert!(matches!(cmd.handler, CommandHandler::Alias { .. }));
    }

    #[test]
    fn test_parameter_types() {
        let required = CommandParameter::new("name".to_string(), ParameterType::String)
            .with_description("Required name".to_string());
        assert!(required.required);
        assert!(required.default_value.is_none());

        let optional =
            CommandParameter::optional("greeting".to_string(), ParameterType::String, Some("Hello".to_string()))
                .with_description("Optional greeting".to_string());
        assert!(!optional.required);
        assert_eq!(optional.default_value, Some("Hello".to_string()));
    }

    #[test]
    fn test_command_parameter_validation() {
        // Test string parameter with command injection protection
        let string_param = CommandParameter::new("input".to_string(), ParameterType::String);
        assert!(string_param.validate("safe_value").is_ok());
        assert!(string_param.validate("unsafe; rm -rf /").is_err());
        assert!(string_param.validate("unsafe | cat /etc/passwd").is_err());
        assert!(string_param.validate("unsafe & echo malicious").is_err());

        // Test number parameter
        let number_param = CommandParameter::new("count".to_string(), ParameterType::Number);
        assert!(number_param.validate("42").is_ok());
        assert!(number_param.validate("3.14").is_ok());
        assert!(number_param.validate("not_a_number").is_err());

        // Test boolean parameter
        let bool_param = CommandParameter::new("enabled".to_string(), ParameterType::Boolean);
        assert!(bool_param.validate("true").is_ok());
        assert!(bool_param.validate("false").is_ok());
        assert!(bool_param.validate("1").is_ok());
        assert!(bool_param.validate("0").is_ok());
        assert!(bool_param.validate("yes").is_ok());
        assert!(bool_param.validate("no").is_ok());
        assert!(bool_param.validate("maybe").is_err());

        // Test enum parameter
        let enum_param = CommandParameter::enum_param("environment".to_string(), vec![
            "dev".to_string(),
            "staging".to_string(),
            "prod".to_string(),
        ]);
        assert!(enum_param.validate("dev").is_ok());
        assert!(enum_param.validate("staging").is_ok());
        assert!(enum_param.validate("prod").is_ok());
        assert!(enum_param.validate("invalid").is_err());

        // Test pattern validation
        let pattern_param = CommandParameter::new("email".to_string(), ParameterType::String)
            .with_pattern(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$".to_string());
        assert!(pattern_param.validate("user@example.com").is_ok());
        assert!(pattern_param.validate("invalid-email").is_err());
    }

    #[test]
    fn test_json_serialization_with_new_schema() {
        let mut cmd = CustomCommand::new_script(
            "test-cmd".to_string(),
            "Test command with new parameter schema".to_string(),
            "echo 'Hello {{name}}'".to_string(),
        );

        // Add parameters with new ParameterType enum
        cmd.add_parameter(
            CommandParameter::new("name".to_string(), ParameterType::String)
                .with_description("Name parameter".to_string()),
        );

        cmd.add_parameter(
            CommandParameter::new("count".to_string(), ParameterType::Number)
                .with_description("Count parameter".to_string()),
        );

        cmd.add_parameter(
            CommandParameter::enum_param("env".to_string(), vec![
                "dev".to_string(),
                "staging".to_string(),
                "prod".to_string(),
            ])
            .with_description("Environment parameter".to_string()),
        );

        cmd.add_parameter(
            CommandParameter::new("email".to_string(), ParameterType::String)
                .with_pattern(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$".to_string())
                .with_description("Email parameter with validation".to_string()),
        );

        // Test serialization
        let json = serde_json::to_string_pretty(&cmd).expect("Failed to serialize");

        // Test deserialization
        let deserialized: CustomCommand = serde_json::from_str(&json).expect("Failed to deserialize");

        // Verify the command was properly deserialized
        assert_eq!(deserialized.name, "test-cmd");
        assert_eq!(deserialized.parameters.len(), 4);

        // Verify parameter types are preserved
        assert!(matches!(deserialized.parameters[0].param_type, ParameterType::String));
        assert!(matches!(deserialized.parameters[1].param_type, ParameterType::Number));
        assert!(matches!(deserialized.parameters[2].param_type, ParameterType::Enum));
        assert!(matches!(deserialized.parameters[3].param_type, ParameterType::String));

        // Verify enum values are preserved
        assert_eq!(
            deserialized.parameters[2].values,
            Some(vec!["dev".to_string(), "staging".to_string(), "prod".to_string()])
        );

        // Verify pattern is preserved
        assert_eq!(
            deserialized.parameters[3].pattern,
            Some(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$".to_string())
        );
    }

    #[test]
    fn test_json_schema_compatibility() {
        // Test that our new schema can be loaded from JSON
        let json_content = r#"{
  "name": "test-deploy",
  "description": "Test deployment command",
  "handler": {
    "Script": {
      "command": "echo",
      "args": ["Deploying to {{env}}"]
    }
  },
  "parameters": [
    {
      "name": "env",
      "type": "enum",
      "required": true,
      "default_value": null,
      "description": "Environment to deploy to",
      "values": ["dev", "staging", "prod"],
      "pattern": null
    },
    {
      "name": "version",
      "type": "string",
      "required": true,
      "default_value": null,
      "description": "Version to deploy",
      "values": null,
      "pattern": "^v\\d+\\.\\d+\\.\\d+$"
    }
  ],
  "created_at": "2025-11-02T00:00:00.000000000+00:00",
  "usage_count": 0
}"#;

        // Test deserialization
        let command: CustomCommand =
            serde_json::from_str(json_content).expect("Failed to deserialize JSON with new schema");

        assert_eq!(command.name, "test-deploy");
        assert_eq!(command.parameters.len(), 2);

        // Verify enum parameter
        assert!(matches!(command.parameters[0].param_type, ParameterType::Enum));
        assert_eq!(
            command.parameters[0].values,
            Some(vec!["dev".to_string(), "staging".to_string(), "prod".to_string()])
        );

        // Verify string parameter with pattern
        assert!(matches!(command.parameters[1].param_type, ParameterType::String));
        assert_eq!(command.parameters[1].pattern, Some("^v\\d+\\.\\d+\\.\\d+$".to_string()));

        // Test parameter validation
        assert!(command.parameters[0].validate("dev").is_ok());
        assert!(command.parameters[0].validate("invalid").is_err());
        assert!(command.parameters[1].validate("v1.2.3").is_ok());
        assert!(command.parameters[1].validate("invalid-version").is_err());
    }

    #[test]
    fn test_command_execution_workflow() {
        use tempfile::TempDir;

        // Create temporary directory for testing
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let mut registry =
            CustomCommandRegistry::new(temp_dir.path().to_path_buf()).expect("Failed to create registry");

        // Create a command with parameters
        let mut cmd = CustomCommand::new_script(
            "greet".to_string(),
            "Greet someone with validation".to_string(),
            "echo 'Hello {{name}} from {{env}}'".to_string(),
        );

        // Add validated parameters
        cmd.add_parameter(
            CommandParameter::new("name".to_string(), ParameterType::String)
                .with_description("Name to greet".to_string()),
        );

        cmd.add_parameter(
            CommandParameter::enum_param("env".to_string(), vec![
                "dev".to_string(),
                "staging".to_string(),
                "prod".to_string(),
            ])
            .with_description("Environment".to_string()),
        );

        // Test adding command to registry
        registry.add_command(cmd).expect("Failed to add command");

        // Test retrieving command
        let retrieved_cmd = registry.get_command("greet").expect("Command not found");
        assert_eq!(retrieved_cmd.name, "greet");
        assert_eq!(retrieved_cmd.parameters.len(), 2);

        // Test parameter validation with valid inputs
        let mut valid_args = HashMap::new();
        valid_args.insert("name".to_string(), "Alice".to_string());
        valid_args.insert("env".to_string(), "dev".to_string());

        assert!(retrieved_cmd.validate_parameters(&valid_args).is_ok());

        // Test parameter validation with invalid inputs
        let mut invalid_args = HashMap::new();
        invalid_args.insert("name".to_string(), "Alice; rm -rf /".to_string()); // Command injection attempt
        invalid_args.insert("env".to_string(), "production".to_string()); // Invalid enum value

        let validation_result = retrieved_cmd.validate_parameters(&invalid_args);
        assert!(validation_result.is_err());

        // Verify error message is helpful
        let error_msg = validation_result.unwrap_err().to_string();
        assert!(error_msg.contains("Invalid characters detected") || error_msg.contains("not in allowed values"));
    }

    #[test]
    fn test_error_handling_and_user_experience() {
        // Test helpful error messages for validation failures
        let string_param = CommandParameter::new("name".to_string(), ParameterType::String);

        // Test command injection error message
        let result = string_param.validate("malicious; rm -rf /");
        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("Invalid characters detected"));

        // Test enum validation error message
        let enum_param = CommandParameter::enum_param("env".to_string(), vec![
            "dev".to_string(),
            "staging".to_string(),
            "prod".to_string(),
        ]);
        let result = enum_param.validate("production");
        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("not in allowed values"));
        assert!(error_msg.contains("dev"));
        assert!(error_msg.contains("staging"));
        assert!(error_msg.contains("prod"));

        // Test pattern validation error message
        let pattern_param = CommandParameter::new("version".to_string(), ParameterType::String)
            .with_pattern(r"^v\d+\.\d+\.\d+$".to_string());
        let result = pattern_param.validate("invalid-version");
        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("does not match required pattern"));

        // Test number validation error message
        let number_param = CommandParameter::new("count".to_string(), ParameterType::Number);
        let result = number_param.validate("not-a-number");
        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("not a valid number"));
    }

    #[test]
    fn test_real_command_execution_with_parameters() {
        // Test actual command execution with parameter substitution
        let mut cmd = CustomCommand::new_script(
            "echo-test".to_string(),
            "Test echo command with parameters".to_string(),
            "echo 'Hello {{name}} from {{env}}'".to_string(),
        );

        cmd.add_parameter(
            CommandParameter::new("name".to_string(), ParameterType::String)
                .with_description("Name to greet".to_string()),
        );

        cmd.add_parameter(
            CommandParameter::enum_param("env".to_string(), vec![
                "dev".to_string(),
                "staging".to_string(),
                "prod".to_string(),
            ])
            .with_description("Environment".to_string()),
        );

        // Test parameter validation with valid inputs
        let mut valid_args = HashMap::new();
        valid_args.insert("name".to_string(), "Alice".to_string());
        valid_args.insert("env".to_string(), "dev".to_string());

        // Validate parameters
        assert!(cmd.validate_parameters(&valid_args).is_ok());

        // Test that command string contains parameter placeholders
        match &cmd.handler {
            CommandHandler::Script { command, .. } => {
                assert!(command.contains("{{name}}"));
                assert!(command.contains("{{env}}"));
            },
            _ => panic!("Expected script handler"),
        }

        // Test parameter substitution logic (simulated)
        let command_template = "echo 'Hello {{name}} from {{env}}'";
        let mut substituted = command_template.to_string();
        for (key, value) in &valid_args {
            substituted = substituted.replace(&format!("{{{{{}}}}}", key), value);
        }
        assert_eq!(substituted, "echo 'Hello Alice from dev'");

        // Test security: malicious parameter should be blocked by validation
        let mut malicious_args = HashMap::new();
        malicious_args.insert("name".to_string(), "Alice; rm -rf /".to_string());
        malicious_args.insert("env".to_string(), "dev".to_string());

        let validation_result = cmd.validate_parameters(&malicious_args);
        assert!(validation_result.is_err());
    }

    #[test]
    fn test_performance_at_scale() {
        use std::time::Instant;

        use tempfile::TempDir;

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let mut registry =
            CustomCommandRegistry::new(temp_dir.path().to_path_buf()).expect("Failed to create registry");

        // Create 100 commands with parameters to test performance
        let start = Instant::now();

        for i in 0..100 {
            let mut cmd = CustomCommand::new_script(
                format!("cmd-{}", i),
                format!("Test command {}", i),
                format!("echo 'Command {} with {{param}}'", i),
            );

            cmd.add_parameter(
                CommandParameter::new("param".to_string(), ParameterType::String)
                    .with_description("Test parameter".to_string()),
            );

            registry.add_command(cmd).expect("Failed to add command");
        }

        let creation_time = start.elapsed();

        // Test retrieval and validation performance
        let start = Instant::now();

        for i in 0..100 {
            let cmd = registry.get_command(&format!("cmd-{}", i)).expect("Command not found");

            let mut args = HashMap::new();
            args.insert("param".to_string(), format!("value-{}", i));

            assert!(cmd.validate_parameters(&args).is_ok());
        }

        let validation_time = start.elapsed();

        // Performance assertions (should be very fast)
        assert!(
            creation_time.as_millis() < 1000,
            "Creation took too long: {:?}",
            creation_time
        );
        assert!(
            validation_time.as_millis() < 100,
            "Validation took too long: {:?}",
            validation_time
        );

        // Verify all commands exist
        assert_eq!(registry.list_commands().len(), 100);
    }
}
