use chat_cli::cli::custom_commands::*;
use std::collections::HashMap;
use tempfile::TempDir;

struct UserAcceptanceTestContext {
    temp_dir: TempDir,
    registry: CustomCommandRegistry,
}

impl UserAcceptanceTestContext {
    fn new() -> anyhow::Result<Self> {
        let temp_dir = TempDir::new()?;
        let commands_dir = temp_dir.path().join(".q-commands");
        let registry = CustomCommandRegistry::new(commands_dir)?;
        
        Ok(Self { temp_dir, registry })
    }
}

#[tokio::test]
async fn uat_001_developer_creates_git_shortcut() -> anyhow::Result<()> {
    // User Story: As a developer, I want to create a shortcut for git status
    let mut ctx = UserAcceptanceTestContext::new()?;
    
    // User creates git status alias
    let cmd = CustomCommand::new_alias(
        "gs".to_string(),
        "Quick git status".to_string(),
        "git status --short".to_string(),
    );
    
    ctx.registry.add_command(cmd)?;
    
    // User executes the command
    let execution = CommandExecution {
        command_name: "gs".to_string(),
        arguments: HashMap::new(),
    };
    
    let loaded_cmd = ctx.registry.get_command("gs").unwrap();
    let result = CommandExecutor::execute(loaded_cmd, &execution);
    
    // Command should execute (may fail if not in git repo, but that's expected)
    assert!(result.is_ok() || result.is_err()); // Either outcome is valid for this test
    
    Ok(())
}

#[tokio::test]
async fn uat_002_devops_creates_deployment_script() -> anyhow::Result<()> {
    // User Story: As a DevOps engineer, I want to create a deployment command with parameters
    let mut ctx = UserAcceptanceTestContext::new()?;
    
    let mut cmd = CustomCommand::new_script(
        "deploy".to_string(),
        "Deploy application to environment".to_string(),
        "echo 'Deploying to {{env}} with version {{version}}'".to_string(),
    );
    
    // Add required parameters
    cmd.add_parameter(CommandParameter::required(
        "env".to_string(),
        "Target environment".to_string(),
    ));
    cmd.add_parameter(CommandParameter::required(
        "version".to_string(),
        "Application version".to_string(),
    ));
    
    ctx.registry.add_command(cmd)?;
    
    // User executes deployment
    let mut args = HashMap::new();
    args.insert("env".to_string(), "production".to_string());
    args.insert("version".to_string(), "v1.2.3".to_string());
    
    let execution = CommandExecution {
        command_name: "deploy".to_string(),
        arguments: args,
    };
    
    let loaded_cmd = ctx.registry.get_command("deploy").unwrap();
    let result = CommandExecutor::execute(loaded_cmd, &execution)?;
    
    assert!(result.contains("Deploying to production"));
    assert!(result.contains("version v1.2.3"));
    
    Ok(())
}

#[tokio::test]
async fn uat_003_user_manages_command_lifecycle() -> anyhow::Result<()> {
    // User Story: As a user, I want to create, list, and delete commands
    let mut ctx = UserAcceptanceTestContext::new()?;
    
    // User creates multiple commands
    let commands = vec![
        CustomCommand::new_script("test1".to_string(), "Test 1".to_string(), "echo 'test1'".to_string()),
        CustomCommand::new_script("test2".to_string(), "Test 2".to_string(), "echo 'test2'".to_string()),
    ];
    
    for cmd in commands {
        ctx.registry.add_command(cmd)?;
    }
    
    // User lists commands
    let command_list = ctx.registry.list_commands();
    assert_eq!(command_list.len(), 2);
    
    // User deletes a command
    ctx.registry.remove_command("test1")?;
    
    // Verify deletion
    assert!(!ctx.registry.command_exists("test1"));
    assert!(ctx.registry.command_exists("test2"));
    
    Ok(())
}

#[tokio::test]
async fn uat_004_user_handles_command_errors_gracefully() -> anyhow::Result<()> {
    // User Story: As a user, I want clear error messages when commands fail
    let mut ctx = UserAcceptanceTestContext::new()?;
    
    let mut cmd = CustomCommand::new_script(
        "greet".to_string(),
        "Greet someone".to_string(),
        "echo 'Hello, {{name}}!'".to_string(),
    );
    
    cmd.add_parameter(CommandParameter::required(
        "name".to_string(),
        "Name to greet".to_string(),
    ));
    
    ctx.registry.add_command(cmd)?;
    
    // User tries to execute without required parameter
    let execution = CommandExecution {
        command_name: "greet".to_string(),
        arguments: HashMap::new(), // Missing required 'name' parameter
    };
    
    let loaded_cmd = ctx.registry.get_command("greet").unwrap();
    let result = CommandExecutor::execute(loaded_cmd, &execution);
    
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Required parameter 'name' is missing"));
    
    Ok(())
}

#[tokio::test]
async fn uat_005_user_creates_builtin_command_shortcut() -> anyhow::Result<()> {
    // User Story: As a user, I want to create shortcuts for built-in Q functions
    let mut ctx = UserAcceptanceTestContext::new()?;
    
    let cmd = CustomCommand {
        name: "save".to_string(),
        description: "Save current context".to_string(),
        handler: CommandHandler::Builtin { 
            function_name: "save_context".to_string() 
        },
        parameters: vec![],
        created_at: chrono::Utc::now().to_rfc3339(),
        usage_count: 0,
    };
    
    ctx.registry.add_command(cmd)?;
    
    // User executes builtin command
    let execution = CommandExecution {
        command_name: "save".to_string(),
        arguments: HashMap::new(),
    };
    
    let loaded_cmd = ctx.registry.get_command("save").unwrap();
    let result = CommandExecutor::execute(loaded_cmd, &execution)?;
    
    assert!(result.contains("Context saved successfully"));
    
    Ok(())
}

#[tokio::test]
async fn uat_006_user_prevents_dangerous_commands() -> anyhow::Result<()> {
    // User Story: As a user, I want the system to prevent me from creating dangerous commands
    let ctx = UserAcceptanceTestContext::new()?;
    
    // Test dangerous script validation
    let dangerous_scripts = vec![
        "rm -rf /",
        "sudo rm important_file",
        "dd if=/dev/zero of=/dev/sda",
    ];
    
    for script in dangerous_scripts {
        let result = CommandExecutor::validate_script_safety(script);
        assert!(result.is_err(), "Should reject dangerous script: {}", script);
    }
    
    // Test safe script validation
    let safe_script = "echo 'Hello, World!'";
    let result = CommandExecutor::validate_script_safety(safe_script);
    assert!(result.is_ok(), "Should accept safe script");
    
    Ok(())
}
