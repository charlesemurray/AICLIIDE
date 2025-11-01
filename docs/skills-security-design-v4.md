# Skills System Security Design v4.0 - Leveraging Q CLI Security Infrastructure

## Design Philosophy: Extend Existing Security Tools

Instead of building security from scratch, we leverage and extend Q CLI's existing security infrastructure:
- `fs_write` with built-in path validation and safeguards
- `execute_bash` with command sanitization
- `fs_read` with access controls
- Built-in input validation and error handling

## Enhanced Q CLI Security Tools for Skills

### 1. Secure File Operations (`fs_write_secure`)

```rust
pub async fn fs_write_secure(
    path: &str,
    content: &str,
    security_context: &SecurityContext,
) -> SecurityResult<()> {
    // Leverage existing fs_write safeguards + skills-specific validation
    validate_skill_file_access(path, security_context)?;
    
    // Use existing fs_write with enhanced logging
    let result = fs_write(path, content).await;
    log_file_operation("write", path, &result, security_context).await;
    
    result.map_err(|e| SecurityError::FileAccessDenied(e.to_string()))
}
```

### 2. Secure Command Execution (`execute_bash_secure`)

```rust
pub async fn execute_bash_secure(
    command: &str,
    security_context: &SecurityContext,
) -> SecurityResult<String> {
    // Enhanced command validation beyond existing execute_bash
    validate_skill_command(command, security_context)?;
    
    // Use existing execute_bash with resource limits
    let result = tokio::time::timeout(
        Duration::from_secs(security_context.resource_limits.timeout_seconds),
        execute_bash(command)
    ).await;
    
    match result {
        Ok(output) => {
            log_command_execution(command, &output, security_context).await;
            Ok(output)
        }
        Err(_) => Err(SecurityError::Timeout(security_context.resource_limits.timeout_seconds))
    }
}
```

### 3. Secure File Reading (`fs_read_secure`)

```rust
pub async fn fs_read_secure(
    path: &str,
    security_context: &SecurityContext,
) -> SecurityResult<String> {
    validate_skill_file_access(path, security_context)?;
    
    let result = fs_read(path).await;
    log_file_operation("read", path, &result, security_context).await;
    
    result.map_err(|e| SecurityError::FileAccessDenied(e.to_string()))
}
```

## Extended Security Tool Collection

### 1. Skills-Specific Validation Tools

```rust
pub struct SkillSecurityTools {
    pub validator: SkillValidator,
    pub sanitizer: InputSanitizer,
    pub monitor: ResourceMonitor,
    pub logger: SecurityLogger,
}

impl SkillSecurityTools {
    pub fn validate_skill_input(&self, input: &serde_json::Value) -> SecurityResult<()> {
        // Leverage existing JSON validation + skills-specific checks
        self.validator.validate_json_structure(input)?;
        self.sanitizer.check_for_malicious_patterns(input)?;
        Ok(())
    }
    
    pub fn validate_skill_output(&self, output: &str) -> SecurityResult<String> {
        // Sanitize output using existing tools
        let sanitized = self.sanitizer.sanitize_output(output)?;
        Ok(sanitized)
    }
    
    pub async fn monitor_skill_execution<F, T>(&self, future: F) -> SecurityResult<T>
    where F: Future<Output = SecurityResult<T>> {
        let start_time = Instant::now();
        let result = future.await;
        let duration = start_time.elapsed();
        
        self.monitor.record_execution_metrics(duration, &result).await;
        result
    }
}
```

### 2. Enhanced Path Validation (Building on fs_write safeguards)

```rust
pub fn validate_skill_file_access(path: &str, context: &SecurityContext) -> SecurityResult<()> {
    // Use existing path validation from fs_write
    if !is_safe_path(path) {
        return Err(SecurityError::InvalidPath(path.to_string()));
    }
    
    // Add skills-specific restrictions
    match context.trust_level {
        TrustLevel::Untrusted => {
            // Only allow temp directory access
            if !path.starts_with("/tmp/q-skills-untrusted/") {
                return Err(SecurityError::PermissionDenied(
                    "Untrusted skills limited to temp directory".to_string()
                ));
            }
        }
        TrustLevel::UserVerified => {
            // Allow workspace access only
            if !path.starts_with("./") && !path.starts_with("/tmp/q-skills-user/") {
                return Err(SecurityError::PermissionDenied(
                    "User skills limited to workspace".to_string()
                ));
            }
        }
        TrustLevel::SystemTrusted => {
            // Full access (but still use existing fs_write safeguards)
        }
    }
    
    Ok(())
}
```

### 3. Command Sanitization (Building on execute_bash safeguards)

```rust
pub fn validate_skill_command(command: &str, context: &SecurityContext) -> SecurityResult<()> {
    // Use existing command validation from execute_bash
    if !is_safe_command(command) {
        return Err(SecurityError::UnsafeCommand(command.to_string()));
    }
    
    // Add skills-specific command restrictions
    let dangerous_commands = match context.trust_level {
        TrustLevel::Untrusted => vec![
            "rm", "del", "format", "sudo", "su", "chmod", "chown", 
            "curl", "wget", "nc", "ssh", "scp", "rsync"
        ],
        TrustLevel::UserVerified => vec![
            "sudo", "su", "chmod +s", "chown root"
        ],
        TrustLevel::SystemTrusted => vec![], // No restrictions
    };
    
    for dangerous in dangerous_commands {
        if command.contains(dangerous) {
            return Err(SecurityError::PermissionDenied(
                format!("Command '{}' not allowed for trust level {:?}", dangerous, context.trust_level)
            ));
        }
    }
    
    Ok(())
}
```

## Security-Enhanced Skill Execution Pipeline

```rust
pub struct SecureSkillExecutor {
    security_tools: SkillSecurityTools,
}

impl SecureSkillExecutor {
    pub async fn execute_skill_securely(
        &self,
        skill: &dyn SecureSkill,
        params: serde_json::Value,
    ) -> SecurityResult<SkillResult> {
        let context = skill.security_context();
        
        // 1. Input validation using existing + enhanced tools
        self.security_tools.validate_skill_input(&params)?;
        skill.validate_input(&params)?;
        
        // 2. Create secure execution environment
        let runtime_context = self.create_secure_runtime_context(context).await?;
        
        // 3. Execute with monitoring (leveraging existing tools)
        let result = self.security_tools.monitor_skill_execution(async {
            skill.execute_secure(params, &runtime_context).await
        }).await?;
        
        // 4. Output validation and sanitization
        skill.validate_output(&result)?;
        let sanitized_result = self.security_tools.validate_skill_output(&result.output)?;
        
        // 5. Log execution (using enhanced logging)
        self.security_tools.logger.log_skill_execution(skill, &result).await?;
        
        Ok(SkillResult {
            output: sanitized_result,
            ui_updates: result.ui_updates,
            state_changes: result.state_changes,
        })
    }
}
```

## Enhanced Security Tool Collection

### 1. Skill Sandbox Manager (Using Q CLI Tools)

```rust
pub struct SkillSandboxManager {
    temp_dir_manager: TempDirManager,
    process_manager: ProcessManager,
}

impl SkillSandboxManager {
    pub async fn create_sandbox(&self, context: &SecurityContext) -> SecurityResult<SkillSandbox> {
        // Create isolated temp directory using fs_write safeguards
        let sandbox_dir = self.temp_dir_manager.create_secure_temp_dir(
            &format!("q-skills-{:?}", context.trust_level)
        ).await?;
        
        // Set up secure environment
        let sandbox = SkillSandbox {
            working_directory: sandbox_dir,
            allowed_commands: self.get_allowed_commands(&context.trust_level),
            resource_limits: context.resource_limits.clone(),
        };
        
        Ok(sandbox)
    }
}
```

### 2. Security Audit Dashboard

```rust
pub struct SecurityAuditDashboard {
    logger: SecurityLogger,
    metrics: SecurityMetrics,
}

impl SecurityAuditDashboard {
    pub async fn generate_security_report(&self) -> SecurityReport {
        SecurityReport {
            total_executions: self.metrics.total_executions,
            blocked_executions: self.metrics.blocked_executions,
            security_score: self.metrics.security_score(),
            recent_violations: self.logger.get_recent_violations(24).await,
            risk_trends: self.metrics.get_risk_trends().await,
        }
    }
    
    pub async fn check_security_health(&self) -> SecurityHealthStatus {
        let score = self.metrics.security_score();
        match score {
            90.0..=100.0 => SecurityHealthStatus::Excellent,
            75.0..=89.9 => SecurityHealthStatus::Good,
            50.0..=74.9 => SecurityHealthStatus::Warning,
            _ => SecurityHealthStatus::Critical,
        }
    }
}
```

## Implementation Strategy

### Phase 1: Extend Existing Tools
- Enhance `fs_write`, `execute_bash`, `fs_read` with skills-specific validation
- Add security logging to existing operations
- Create `SkillSecurityTools` wrapper

### Phase 2: Skills-Specific Security
- Implement trust-level based restrictions
- Add resource monitoring using existing infrastructure
- Create secure execution pipeline

### Phase 3: Security Management Tools
- Build security audit dashboard
- Add security health monitoring
- Create security configuration management

### Phase 4: Advanced Security Features
- Implement skill signing and verification
- Add anomaly detection
- Create security incident response

## Benefits of This Approach

1. **Leverage Existing Security**: Build on proven Q CLI safeguards
2. **Consistent Interface**: Same tools users already know
3. **Minimal New Dependencies**: Extend rather than replace
4. **Gradual Enhancement**: Incremental security improvements
5. **Familiar Debugging**: Use existing Q CLI debugging tools

This design ensures security is built on the solid foundation of Q CLI's existing security infrastructure while providing skills-specific enhancements.
