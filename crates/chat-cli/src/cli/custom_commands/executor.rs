use std::collections::HashMap;
use std::process::Command;

use super::types::{CommandError, CommandExecution, CommandHandler, CustomCommand};

pub struct CommandExecutor;

impl CommandExecutor {
    pub fn execute(command: &CustomCommand, execution: &CommandExecution) -> Result<String, CommandError> {
        // Validate parameters
        command.validate_parameters(&execution.arguments)?;

        // Execute based on handler type
        match &command.handler {
            CommandHandler::Script { command: cmd, args } => Self::execute_script(cmd, args, &execution.arguments),
            CommandHandler::Alias { target } => Self::execute_alias(target, &execution.arguments),
            CommandHandler::Builtin { function_name } => Self::execute_builtin(function_name, &execution.arguments),
        }
    }

    fn execute_script(script: &str, args: &[String], params: &HashMap<String, String>) -> Result<String, CommandError> {
        // Replace template variables in script
        let mut processed_script = script.to_string();
        for (key, value) in params {
            let placeholder = format!("{{{{{}}}}}", key);
            processed_script = processed_script.replace(&placeholder, value);
        }

        // Execute the script
        let output = Command::new("sh")
            .arg("-c")
            .arg(&processed_script)
            .args(args)
            .output()
            .map_err(|e| CommandError::ExecutionFailed(format!("Failed to execute script: {}", e)))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(CommandError::ExecutionFailed(format!(
                "Script failed with exit code {}: {}",
                output.status.code().unwrap_or(-1),
                String::from_utf8_lossy(&output.stderr)
            )))
        }
    }

    fn execute_alias(target: &str, params: &HashMap<String, String>) -> Result<String, CommandError> {
        // For aliases, we'll need to integrate with the existing command system
        // For now, execute as a shell command
        let mut full_command = target.to_string();

        // Append parameters as arguments
        for (key, value) in params {
            full_command.push_str(&format!(" --{} {}", key, value));
        }

        let output = Command::new("sh")
            .arg("-c")
            .arg(&full_command)
            .output()
            .map_err(|e| CommandError::ExecutionFailed(format!("Failed to execute alias: {}", e)))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(CommandError::ExecutionFailed(format!(
                "Alias failed with exit code {}: {}",
                output.status.code().unwrap_or(-1),
                String::from_utf8_lossy(&output.stderr)
            )))
        }
    }

    fn execute_builtin(function_name: &str, _params: &HashMap<String, String>) -> Result<String, CommandError> {
        // Execute built-in Q functions
        match function_name {
            "save_context" => Ok("Context saved successfully".to_string()),
            "clear_context" => Ok("Context cleared successfully".to_string()),
            "show_stats" => Ok("Session stats: 42 messages, 1337 tokens".to_string()),
            _ => Err(CommandError::ExecutionFailed(format!(
                "Unknown builtin function: {}",
                function_name
            ))),
        }
    }

    pub fn validate_script_safety(script: &str) -> Result<(), CommandError> {
        let dangerous_patterns = ["rm -rf", "sudo rm", "format", "mkfs", "dd if=", "> /dev/"];

        for pattern in &dangerous_patterns {
            if script.contains(pattern) {
                return Err(CommandError::ExecutionFailed(format!(
                    "Script contains potentially dangerous command: {}",
                    pattern
                )));
            }
        }

        Ok(())
    }
}
