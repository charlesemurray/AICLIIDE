use crate::cli::skills::security::*;
use crate::cli::skills::security_logging::*;
use std::path::PathBuf;
use std::time::Instant;
use tokio::fs;
use tokio::io::{self, AsyncBufReadExt, BufReader};

#[derive(Debug, Clone)]
pub struct SignoffRequest {
    pub operation_description: String,
    pub risk_level: RiskLevel,
    pub details: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct UserSignoffDecision {
    pub approved: bool,
    pub conditions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SignoffDecision {
    pub required: bool,
    pub approved: bool,
    pub conditions: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum SignoffTrigger {
    TrustLevelElevation,
    FileSystemWrite(String),
    NetworkAccess,
    SystemCommand(String),
    ResourceLimitExceed,
}

/// Enhanced security tools that build on Q CLI's existing infrastructure
pub struct SkillSecurityTools {
    pub logger: SecurityLogger,
    pub metrics: SecurityMetrics,
    pub signoff_manager: SkillSignoffManager,
    pub git_manager: SkillGitManager,
}

impl SkillSecurityTools {
    pub fn new(log_dir: std::path::PathBuf, repo_path: PathBuf) -> Self {
        Self {
            logger: SecurityLogger::new(log_dir),
            metrics: SecurityMetrics::new(),
            signoff_manager: SkillSignoffManager::new(),
            git_manager: SkillGitManager::new(repo_path),
        }
    }
    
    /// Secure file write building on fs_write safeguards
    pub async fn fs_write_secure(
        &self,
        path: &str,
        content: &str,
        security_context: &SecurityContext,
    ) -> SecurityResult<()> {
        // Validate file access permissions
        self.validate_skill_file_access(path, security_context)?;
        
        // Log the operation
        let event = self.logger.create_security_event(
            SecurityEventType::SkillExecutionStarted,
            "file_write".to_string(),
            serde_json::json!({
                "operation": "write",
                "path": path,
                "trust_level": format!("{:?}", security_context.trust_level)
            }),
        );
        self.logger.log_security_event(event).await.ok();
        
        // Use existing fs_write functionality with enhanced error handling
        match fs::write(path, content).await {
            Ok(()) => {
                let success_event = self.logger.create_security_event(
                    SecurityEventType::SkillExecutionCompleted,
                    "file_write".to_string(),
                    serde_json::json!({"path": path, "bytes_written": content.len()}),
                );
                self.logger.log_security_event(success_event).await.ok();
                Ok(())
            }
            Err(e) => {
                let error_event = self.logger.create_security_event(
                    SecurityEventType::SkillExecutionFailed,
                    "file_write".to_string(),
                    serde_json::json!({"path": path, "error": e.to_string()}),
                );
                self.logger.log_security_event(error_event).await.ok();
                Err(SecurityError::PermissionDenied(format!("File write failed: {}", e)))
            }
        }
    }
    
    /// Secure file read building on fs_read safeguards
    pub async fn fs_read_secure(
        &self,
        path: &str,
        security_context: &SecurityContext,
    ) -> SecurityResult<String> {
        // Validate file access permissions
        self.validate_skill_file_access(path, security_context)?;
        
        // Log the operation
        let event = self.logger.create_security_event(
            SecurityEventType::SkillExecutionStarted,
            "file_read".to_string(),
            serde_json::json!({
                "operation": "read",
                "path": path,
                "trust_level": format!("{:?}", security_context.trust_level)
            }),
        );
        self.logger.log_security_event(event).await.ok();
        
        // Use existing fs_read functionality
        match fs::read_to_string(path).await {
            Ok(content) => {
                let success_event = self.logger.create_security_event(
                    SecurityEventType::SkillExecutionCompleted,
                    "file_read".to_string(),
                    serde_json::json!({"path": path, "bytes_read": content.len()}),
                );
                self.logger.log_security_event(success_event).await.ok();
                Ok(content)
            }
            Err(e) => {
                let error_event = self.logger.create_security_event(
                    SecurityEventType::SkillExecutionFailed,
                    "file_read".to_string(),
                    serde_json::json!({"path": path, "error": e.to_string()}),
                );
                self.logger.log_security_event(error_event).await.ok();
                Err(SecurityError::PermissionDenied(format!("File read failed: {}", e)))
            }
        }
    }
    
    /// Validate file access based on trust level and path
    pub fn validate_skill_file_access(&self, path: &str, context: &SecurityContext) -> SecurityResult<()> {
        // Basic path validation (similar to fs_write safeguards)
        if path.contains("..") || path.contains("~") {
            let event = self.logger.create_security_event(
                SecurityEventType::SecurityViolationBlocked,
                "path_validation".to_string(),
                serde_json::json!({"violation": "directory_traversal", "path": path}),
            );
            tokio::spawn(async move { event });
            return Err(SecurityError::InputValidationFailed(
                "Directory traversal detected".to_string()
            ));
        }
        
        // Trust-level based restrictions
        match context.trust_level {
            TrustLevel::Untrusted => {
                if !path.starts_with("/tmp/q-skills-untrusted/") {
                    return Err(SecurityError::PermissionDenied(
                        "Untrusted skills limited to temp directory".to_string()
                    ));
                }
            }
            TrustLevel::UserVerified => {
                let allowed_prefixes = ["./", "/tmp/q-skills-user/"];
                if !allowed_prefixes.iter().any(|prefix| path.starts_with(prefix)) {
                    return Err(SecurityError::PermissionDenied(
                        "User skills limited to workspace and temp directories".to_string()
                    ));
                }
            }
            TrustLevel::SystemTrusted => {
                // Full access allowed
            }
        }
        
        Ok(())
    }
    
    /// Validate command execution based on trust level
    pub fn validate_skill_command(&self, command: &str, context: &SecurityContext) -> SecurityResult<()> {
        // Basic command injection protection
        let injection_patterns = [";", "&&", "||", "|", "`", "$("];
        for pattern in injection_patterns {
            if command.contains(pattern) {
                let event = self.logger.create_security_event(
                    SecurityEventType::SecurityViolationBlocked,
                    "command_validation".to_string(),
                    serde_json::json!({"violation": "command_injection", "pattern": pattern, "command": command}),
                );
                tokio::spawn(async move { event });
                return Err(SecurityError::InputValidationFailed(
                    format!("Command injection pattern detected: {}", pattern)
                ));
            }
        }
        
        // Trust-level based command restrictions
        let dangerous_commands = match context.trust_level {
            TrustLevel::Untrusted => vec![
                "rm", "del", "format", "sudo", "su", "chmod", "chown", 
                "curl", "wget", "nc", "ssh", "scp", "rsync", "dd"
            ],
            TrustLevel::UserVerified => vec![
                "sudo", "su", "chmod +s", "chown root", "dd"
            ],
            TrustLevel::SystemTrusted => vec![], // No restrictions
        };
        
        for dangerous in dangerous_commands {
            if command.to_lowercase().contains(dangerous) {
                let event = self.logger.create_security_event(
                    SecurityEventType::PermissionDenied,
                    "command_validation".to_string(),
                    serde_json::json!({
                        "violation": "dangerous_command", 
                        "command": dangerous, 
                        "trust_level": format!("{:?}", context.trust_level)
                    }),
                );
                tokio::spawn(async move { event });
                return Err(SecurityError::PermissionDenied(
                    format!("Command '{}' not allowed for trust level {:?}", dangerous, context.trust_level)
                ));
            }
        }
        
        Ok(())
    }
    
    /// Monitor skill execution with resource tracking
    pub async fn monitor_skill_execution<F, T>(&mut self, future: F, skill_name: &str) -> SecurityResult<T>
    where 
        F: std::future::Future<Output = SecurityResult<T>>,
    {
        let start_time = Instant::now();
        
        // Log execution start
        let start_event = self.logger.create_security_event(
            SecurityEventType::SkillExecutionStarted,
            skill_name.to_string(),
            serde_json::json!({"start_time": chrono::Utc::now().to_rfc3339()}),
        );
        self.logger.log_security_event(start_event).await.ok();
        
        // Execute with monitoring
        let result = future.await;
        let duration = start_time.elapsed();
        
        // Log execution completion
        let end_event = match &result {
            Ok(_) => self.logger.create_security_event(
                SecurityEventType::SkillExecutionCompleted,
                skill_name.to_string(),
                serde_json::json!({
                    "duration_ms": duration.as_millis(),
                    "status": "success"
                }),
            ),
            Err(e) => self.logger.create_security_event(
                SecurityEventType::SkillExecutionFailed,
                skill_name.to_string(),
                serde_json::json!({
                    "duration_ms": duration.as_millis(),
                    "status": "failed",
                    "error": e.to_string()
                }),
            ),
        };
        self.logger.log_security_event(end_event).await.ok();
        
        // Update metrics
        let trace = ExecutionTrace {
            execution_id: uuid::Uuid::new_v4().to_string(),
            skill_name: skill_name.to_string(),
            start_time: chrono::Utc::now() - chrono::Duration::from_std(duration).unwrap_or_default(),
            end_time: Some(chrono::Utc::now()),
            commands_executed: vec![], // Would be populated by actual execution
            files_accessed: vec![],
            network_requests: vec![],
            resource_usage: ResourceUsageTrace {
                peak_memory_mb: 0, // Would be populated by actual monitoring
                peak_cpu_percent: 0.0,
                total_disk_io_mb: 0,
                network_bytes_sent: 0,
                network_bytes_received: 0,
                execution_duration_ms: duration.as_millis() as u64,
            },
            exit_code: if result.is_ok() { Some(0) } else { Some(1) },
            security_violations: vec![],
        };
        
        self.metrics.record_execution(&trace);
        self.logger.log_execution_trace(trace).await.ok();
        
        result
    }
    
    /// Get current security health status
    pub fn get_security_health(&self) -> SecurityHealthStatus {
        let score = self.metrics.security_score();
        match score {
            90.0..=100.0 => SecurityHealthStatus::Excellent,
            75.0..=89.9 => SecurityHealthStatus::Good,
            50.0..=74.9 => SecurityHealthStatus::Warning,
            _ => SecurityHealthStatus::Critical,
        }
    }
}

#[derive(Debug, Clone)]
pub enum SecurityHealthStatus {
    Excellent,
    Good,
    Warning,
    Critical,
}

impl SecurityHealthStatus {
    pub fn as_emoji(&self) -> &'static str {
        match self {
            SecurityHealthStatus::Excellent => "🟢",
            SecurityHealthStatus::Good => "🟡",
            SecurityHealthStatus::Warning => "🟠",
            SecurityHealthStatus::Critical => "🔴",
        }
    }
    
    pub fn as_description(&self) -> &'static str {
        match self {
            SecurityHealthStatus::Excellent => "Security systems operating normally",
            SecurityHealthStatus::Good => "Minor security events detected",
            SecurityHealthStatus::Warning => "Elevated security risk - review recommended",
            SecurityHealthStatus::Critical => "High security risk - immediate attention required",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_secure_file_operations() {
        let temp_dir = TempDir::new().unwrap();
        let tools = SkillSecurityTools::new(temp_dir.path().to_path_buf());
        let context = SecurityContext::for_trust_level(TrustLevel::UserVerified);
        
        // Test valid file write
        let test_file = temp_dir.path().join("test.txt");
        let result = tools.fs_write_secure(
            test_file.to_str().unwrap(),
            "test content",
            &context
        ).await;
        
        // Should fail due to path restrictions for UserVerified
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_command_validation() {
        let temp_dir = TempDir::new().unwrap();
        let tools = SkillSecurityTools::new(temp_dir.path().to_path_buf());
        let context = SecurityContext::for_trust_level(TrustLevel::Untrusted);
        
        // Test dangerous command blocking
        let result = tools.validate_skill_command("rm -rf /", &context);
        assert!(result.is_err());
        
        // Test command injection blocking
        let result = tools.validate_skill_command("echo hello; rm -rf /", &context);
        assert!(result.is_err());
    }
}

pub struct SkillSignoffManager {
    signoff_required_operations: Vec<SignoffTrigger>,
}

impl SkillSignoffManager {
    pub fn new() -> Self {
        Self {
            signoff_required_operations: vec![
                SignoffTrigger::FileSystemWrite("/".to_string()),
                SignoffTrigger::NetworkAccess,
                SignoffTrigger::SystemCommand("sudo".to_string()),
                SignoffTrigger::SystemCommand("rm".to_string()),
                SignoffTrigger::ResourceLimitExceed,
            ],
        }
    }
    
    pub async fn check_signoff_required(
        &self,
        operation: &str,
        context: &SecurityContext,
    ) -> SecurityResult<SignoffDecision> {
        let triggers = self.evaluate_signoff_triggers(operation, context);
        
        if !triggers.is_empty() {
            let signoff_request = SignoffRequest {
                operation_description: format!("Skill operation: {}", operation),
                risk_level: self.assess_risk_level(&triggers),
                details: serde_json::json!({
                    "triggers": format!("{:?}", triggers),
                    "trust_level": format!("{:?}", context.trust_level),
                }),
            };
            
            let user_decision = self.request_user_signoff(signoff_request).await?;
            
            Ok(SignoffDecision {
                required: true,
                approved: user_decision.approved,
                conditions: user_decision.conditions,
            })
        } else {
            Ok(SignoffDecision {
                required: false,
                approved: true,
                conditions: vec![],
            })
        }
    }
    
    fn evaluate_signoff_triggers(&self, operation: &str, context: &SecurityContext) -> Vec<SignoffTrigger> {
        let mut triggers = vec![];
        
        if operation.contains("sudo") || operation.contains("su") {
            triggers.push(SignoffTrigger::SystemCommand("privilege_escalation".to_string()));
        }
        
        if operation.contains("rm") || operation.contains("del") {
            triggers.push(SignoffTrigger::SystemCommand("file_deletion".to_string()));
        }
        
        if operation.contains("curl") || operation.contains("wget") || operation.contains("nc") {
            triggers.push(SignoffTrigger::NetworkAccess);
        }
        
        if context.trust_level == TrustLevel::Untrusted && operation.contains("write") {
            triggers.push(SignoffTrigger::FileSystemWrite("untrusted_write".to_string()));
        }
        
        triggers
    }
    
    fn assess_risk_level(&self, triggers: &[SignoffTrigger]) -> RiskLevel {
        if triggers.iter().any(|t| matches!(t, SignoffTrigger::SystemCommand(_))) {
            RiskLevel::High
        } else if triggers.len() > 1 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        }
    }
    
    async fn request_user_signoff(&self, request: SignoffRequest) -> SecurityResult<UserSignoffDecision> {
        println!("🔐 Skill Security Review Required");
        println!("Operation: {}", request.operation_description);
        println!("Risk Level: {:?}", request.risk_level);
        println!();
        println!("Do you want to allow this operation? (y/N)");
        
        let stdin = io::stdin();
        let mut reader = BufReader::new(stdin);
        let mut response = String::new();
        
        match reader.read_line(&mut response).await {
            Ok(_) => {
                let response = response.trim().to_lowercase();
                match response.as_str() {
                    "y" | "yes" => Ok(UserSignoffDecision {
                        approved: true,
                        conditions: vec![],
                    }),
                    _ => Ok(UserSignoffDecision {
                        approved: false,
                        conditions: vec![],
                    }),
                }
            }
            Err(_) => Ok(UserSignoffDecision {
                approved: false,
                conditions: vec![],
            }),
        }
    }
}

pub struct SkillGitManager {
    repo_path: PathBuf,
}

impl SkillGitManager {
    pub fn new(repo_path: PathBuf) -> Self {
        Self { repo_path }
    }
    
    pub async fn backup_before_execution(&self, skill_name: &str, trust_level: &TrustLevel) -> SecurityResult<String> {
        let commit_message = format!(
            "Pre-execution backup: {} (trust: {:?})",
            skill_name,
            trust_level
        );
        
        self.git_commit_changes(&commit_message).await
    }
    
    pub async fn create_security_checkpoint(&self, event: &SecurityEvent) -> SecurityResult<()> {
        if event.risk_level >= RiskLevel::High {
            let commit_message = format!(
                "Security checkpoint: {:?} - {}",
                event.event_type,
                event.skill_name
            );
            
            self.git_commit_changes(&commit_message).await?;
        }
        
        Ok(())
    }
    
    async fn git_commit_changes(&self, message: &str) -> SecurityResult<String> {
        use tokio::process::Command;
        
        let add_output = Command::new("git")
            .args(&["add", "-A"])
            .current_dir(&self.repo_path)
            .output()
            .await
            .map_err(|e| SecurityError::SandboxViolation(format!("Git add failed: {}", e)))?;
        
        if !add_output.status.success() {
            return Err(SecurityError::SandboxViolation("Git add failed".to_string()));
        }
        
        let commit_output = Command::new("git")
            .args(&["commit", "-m", message])
            .current_dir(&self.repo_path)
            .output()
            .await
            .map_err(|e| SecurityError::SandboxViolation(format!("Git commit failed: {}", e)))?;
        
        if commit_output.status.success() {
            let hash_output = Command::new("git")
                .args(&["rev-parse", "HEAD"])
                .current_dir(&self.repo_path)
                .output()
                .await
                .map_err(|e| SecurityError::SandboxViolation(format!("Git hash failed: {}", e)))?;
            
            let commit_hash = String::from_utf8_lossy(&hash_output.stdout).trim().to_string();
            Ok(commit_hash)
        } else {
            Ok("no-changes".to_string())
        }
    }
}
