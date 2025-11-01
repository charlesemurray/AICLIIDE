//! Tests for command builder functionality

use super::super::*;
use eyre::Result;

#[test]
fn test_command_builder_basic() -> Result<()> {
    let builder = CommandBuilder::new()
        .with_name("git-status".to_string())
        .with_description("Show git status".to_string())
        .with_command("git status".to_string());
    
    let validation = builder.validate()?;
    assert!(validation.is_valid);
    
    Ok(())
}

#[test]
fn test_command_builder_parameters() -> Result<()> {
    let config = CommandBuilder::new()
        .with_name("docker-logs".to_string())
        .with_command("docker logs".to_string())
        .add_parameter("--follow".to_string())
        .add_parameter("--tail=100".to_string())
        .build()?;
    
    assert_eq!(config.name, "docker-logs");
    assert_eq!(config.command, "docker logs");
    assert_eq!(config.parameters.len(), 2);
    assert!(config.parameters.contains(&"--follow".to_string()));
    assert!(config.parameters.contains(&"--tail=100".to_string()));
    
    Ok(())
}

#[test]
fn test_command_builder_with_parameters() -> Result<()> {
    let params = vec!["--verbose".to_string(), "--output=json".to_string()];
    let config = CommandBuilder::new()
        .with_name("test-cmd".to_string())
        .with_command("test".to_string())
        .with_parameters(params.clone())
        .build()?;
    
    assert_eq!(config.parameters, params);
    
    Ok(())
}

#[test]
fn test_command_builder_working_directory() -> Result<()> {
    let config = CommandBuilder::new()
        .with_name("pwd-test".to_string())
        .with_command("pwd".to_string())
        .with_working_directory("/tmp".to_string())
        .build()?;
    
    assert_eq!(config.working_directory, Some("/tmp".to_string()));
    
    Ok(())
}

#[test]
fn test_command_builder_timeout() -> Result<()> {
    let config = CommandBuilder::new()
        .with_name("long-cmd".to_string())
        .with_command("sleep 10".to_string())
        .with_timeout(30)
        .build()?;
    
    assert_eq!(config.timeout, Some(30));
    
    Ok(())
}

#[test]
fn test_command_builder_environment() -> Result<()> {
    let config = CommandBuilder::new()
        .with_name("env-test".to_string())
        .with_command("env".to_string())
        .with_environment("DEBUG".to_string(), "1".to_string())
        .with_environment("PATH".to_string(), "/usr/bin".to_string())
        .build()?;
    
    assert_eq!(config.environment.get("DEBUG"), Some(&"1".to_string()));
    assert_eq!(config.environment.get("PATH"), Some(&"/usr/bin".to_string()));
    
    Ok(())
}

#[test]
fn test_command_builder_validation_success() -> Result<()> {
    let builder = CommandBuilder::new()
        .with_name("valid-cmd".to_string())
        .with_description("A valid command".to_string())
        .with_command("echo hello".to_string())
        .add_parameter("world".to_string())
        .with_timeout(10);
    
    let validation = builder.validate()?;
    assert!(validation.is_valid);
    assert!(validation.score > 0.5);
    
    Ok(())
}

#[test]
fn test_command_builder_validation_failure() -> Result<()> {
    let builder = CommandBuilder::new(); // Empty command
    
    let validation = builder.validate()?;
    assert!(!validation.is_valid);
    assert!(!validation.issues.is_empty());
    
    // Should have errors for empty name and command
    let has_name_error = validation.issues
        .iter()
        .any(|issue| issue.severity == IssueSeverity::Error && 
                    issue.message.contains("name cannot be empty"));
    let has_command_error = validation.issues
        .iter()
        .any(|issue| issue.severity == IssueSeverity::Error && 
                    issue.message.contains("executable cannot be empty"));
    
    assert!(has_name_error);
    assert!(has_command_error);
    
    Ok(())
}

#[test]
fn test_command_builder_build_success() -> Result<()> {
    let config = CommandBuilder::new()
        .with_name("working-cmd".to_string())
        .with_description("A working command".to_string())
        .with_command("ls -la".to_string())
        .build()?;
    
    assert_eq!(config.name, "working-cmd");
    assert_eq!(config.command, "ls -la");
    
    Ok(())
}

#[test]
fn test_command_builder_build_failure() {
    let result = CommandBuilder::new().build(); // Empty command should fail
    assert!(result.is_err());
}

#[test]
fn test_command_builder_preview() -> Result<()> {
    let builder = CommandBuilder::new()
        .with_name("preview-cmd".to_string())
        .with_command("docker run".to_string())
        .add_parameter("-it".to_string())
        .add_parameter("ubuntu:latest".to_string())
        .add_parameter("bash".to_string());
    
    let preview = builder.preview();
    
    assert!(preview.contains("docker run"));
    assert!(preview.contains("-it"));
    assert!(preview.contains("ubuntu:latest"));
    assert!(preview.contains("bash"));
    
    Ok(())
}

#[test]
fn test_command_builder_timeout_validation() -> Result<()> {
    let builder = CommandBuilder::new()
        .with_name("timeout-test".to_string())
        .with_command("test".to_string())
        .with_timeout(0); // Invalid timeout
    
    let validation = builder.validate()?;
    assert!(validation.is_valid); // Still valid, just a warning
    
    let has_timeout_warning = validation.issues
        .iter()
        .any(|issue| issue.severity == IssueSeverity::Warning && 
                    issue.message.contains("Timeout should be"));
    assert!(has_timeout_warning);
    
    Ok(())
}

#[test]
fn test_command_builder_chaining() -> Result<()> {
    let config = CommandBuilder::new()
        .with_name("chained-cmd".to_string())
        .with_description("Built with method chaining".to_string())
        .with_command("git".to_string())
        .add_parameter("log".to_string())
        .add_parameter("--oneline".to_string())
        .add_parameter("--graph".to_string())
        .with_working_directory("/repo".to_string())
        .with_timeout(60)
        .with_environment("GIT_PAGER".to_string(), "cat".to_string())
        .build()?;
    
    assert_eq!(config.name, "chained-cmd");
    assert_eq!(config.command, "git");
    assert_eq!(config.parameters.len(), 3);
    assert_eq!(config.working_directory, Some("/repo".to_string()));
    assert_eq!(config.timeout, Some(60));
    assert_eq!(config.environment.get("GIT_PAGER"), Some(&"cat".to_string()));
    
    Ok(())
}
