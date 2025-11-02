#[cfg(test)]
mod tests {
    use crate::cli::custom_commands::*;
    use crate::cli::custom_commands::types::ParameterType;
    use std::collections::HashMap;
    use tempfile::TempDir;

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

        cmd.add_parameter(CommandParameter::new(
            "name".to_string(),
            ParameterType::String,
        ).with_description("Person to greet".to_string()));

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

        cmd.add_parameter(CommandParameter::new(
            "name".to_string(),
            ParameterType::String,
        ).with_description("Name to greet".to_string()));

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
                function_name: "save_context".to_string() 
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
        let required = CommandParameter::new(
            "name".to_string(),
            ParameterType::String,
        ).with_description("Required name".to_string());
        assert!(required.required);
        assert!(required.default_value.is_none());

        let optional = CommandParameter::optional(
            "greeting".to_string(),
            ParameterType::String,
            Some("Hello".to_string()),
        ).with_description("Optional greeting".to_string());
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
        let enum_param = CommandParameter::enum_param(
            "environment".to_string(),
            vec!["dev".to_string(), "staging".to_string(), "prod".to_string()]
        );
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
}
