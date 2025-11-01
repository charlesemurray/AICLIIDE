use chat_cli::cli::custom_commands::*;
use std::collections::HashMap;
use tempfile::TempDir;

struct IntegrationTestContext {
    temp_dir: TempDir,
    registry: CustomCommandRegistry,
}

impl IntegrationTestContext {
    fn new() -> anyhow::Result<Self> {
        let temp_dir = TempDir::new()?;
        let commands_dir = temp_dir.path().join(".q-commands");
        let registry = CustomCommandRegistry::new(commands_dir)?;
        
        Ok(Self { temp_dir, registry })
    }
}

#[tokio::test]
async fn test_end_to_end_script_command_workflow() -> anyhow::Result<()> {
    let mut ctx = IntegrationTestContext::new()?;
    
    // Create command
    let cmd = CustomCommand::new_script(
        "hello".to_string(),
        "Say hello to someone".to_string(),
        "echo 'Hello, {{name}}!'".to_string(),
    );
    
    // Add parameter
    let mut cmd_with_param = cmd;
    cmd_with_param.add_parameter(CommandParameter::required(
        "name".to_string(),
        "Name to greet".to_string(),
    ));
    
    // Register command
    ctx.registry.add_command(cmd_with_param)?;
    
    // Verify persistence
    let loaded_cmd = ctx.registry.get_command("hello").unwrap();
    assert_eq!(loaded_cmd.name, "hello");
    
    // Execute command
    let mut args = HashMap::new();
    args.insert("name".to_string(), "World".to_string());
    
    let execution = CommandExecution {
        command_name: "hello".to_string(),
        arguments: args,
    };
    
    let result = CommandExecutor::execute(loaded_cmd, &execution)?;
    assert!(result.contains("Hello, World!"));
    
    Ok(())
}

#[tokio::test]
async fn test_command_persistence_across_registry_reloads() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let commands_dir = temp_dir.path().join(".q-commands");
    
    // Create and save command in first registry
    {
        let mut registry1 = CustomCommandRegistry::new(commands_dir.clone())?;
        let cmd = CustomCommand::new_alias(
            "gs".to_string(),
            "Git status shortcut".to_string(),
            "git status".to_string(),
        );
        registry1.add_command(cmd)?;
    }
    
    // Load command in new registry instance
    {
        let registry2 = CustomCommandRegistry::new(commands_dir)?;
        let loaded_cmd = registry2.get_command("gs");
        assert!(loaded_cmd.is_some());
        assert_eq!(loaded_cmd.unwrap().description, "Git status shortcut");
    }
    
    Ok(())
}

#[tokio::test]
async fn test_multiple_commands_management() -> anyhow::Result<()> {
    let mut ctx = IntegrationTestContext::new()?;
    
    // Create multiple commands
    let commands = vec![
        CustomCommand::new_script("cmd1".to_string(), "First".to_string(), "echo '1'".to_string()),
        CustomCommand::new_script("cmd2".to_string(), "Second".to_string(), "echo '2'".to_string()),
        CustomCommand::new_alias("cmd3".to_string(), "Third".to_string(), "ls -la".to_string()),
    ];
    
    // Add all commands
    for cmd in commands {
        ctx.registry.add_command(cmd)?;
    }
    
    // Verify all commands exist
    assert_eq!(ctx.registry.list_commands().len(), 3);
    assert!(ctx.registry.command_exists("cmd1"));
    assert!(ctx.registry.command_exists("cmd2"));
    assert!(ctx.registry.command_exists("cmd3"));
    
    // Remove one command
    ctx.registry.remove_command("cmd2")?;
    assert_eq!(ctx.registry.list_commands().len(), 2);
    assert!(!ctx.registry.command_exists("cmd2"));
    
    Ok(())
}

#[tokio::test]
async fn test_command_execution_with_error_handling() -> anyhow::Result<()> {
    let mut ctx = IntegrationTestContext::new()?;
    
    // Create command that will fail
    let cmd = CustomCommand::new_script(
        "fail".to_string(),
        "Command that fails".to_string(),
        "exit 1".to_string(),
    );
    
    ctx.registry.add_command(cmd)?;
    
    let execution = CommandExecution {
        command_name: "fail".to_string(),
        arguments: HashMap::new(),
    };
    
    let loaded_cmd = ctx.registry.get_command("fail").unwrap();
    let result = CommandExecutor::execute(loaded_cmd, &execution);
    
    assert!(result.is_err());
    
    Ok(())
}

#[tokio::test]
async fn test_builtin_command_execution() -> anyhow::Result<()> {
    let mut ctx = IntegrationTestContext::new()?;
    
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
    
    ctx.registry.add_command(cmd)?;
    
    let execution = CommandExecution {
        command_name: "save".to_string(),
        arguments: HashMap::new(),
    };
    
    let loaded_cmd = ctx.registry.get_command("save").unwrap();
    let result = CommandExecutor::execute(loaded_cmd, &execution)?;
    
    assert!(result.contains("Context saved"));
    
    Ok(())
}
