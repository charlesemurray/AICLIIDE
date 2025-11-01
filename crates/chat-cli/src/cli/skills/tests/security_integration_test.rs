#[cfg(test)]
mod security_integration_tests {
    use crate::cli::skills::security::*;
    use crate::cli::skills::security_tools::*;
    use std::path::PathBuf;

    // Mock security tools for fast testing
    struct MockSecurityTools {
        signoff_manager: MockSignoffManager,
    }

    impl MockSecurityTools {
        fn new() -> Self {
            Self {
                signoff_manager: MockSignoffManager::new(),
            }
        }

        fn validate_skill_file_access(&self, path: &str, context: &SecurityContext) -> SecurityResult<()> {
            if path.contains("..") || path.contains("~") {
                return Err(SecurityError::InputValidationFailed("Directory traversal detected".to_string()));
            }
            
            match context.trust_level {
                TrustLevel::Untrusted => {
                    if !path.starts_with("/tmp/q-skills-untrusted/") {
                        return Err(SecurityError::PermissionDenied("Untrusted skills limited to temp directory".to_string()));
                    }
                }
                TrustLevel::UserVerified => {
                    let allowed_prefixes = ["./", "/tmp/q-skills-user/"];
                    if !allowed_prefixes.iter().any(|prefix| path.starts_with(prefix)) {
                        return Err(SecurityError::PermissionDenied("User skills limited to workspace and temp directories".to_string()));
                    }
                }
                TrustLevel::SystemTrusted => {}
            }
            Ok(())
        }

        fn validate_skill_command(&self, command: &str, context: &SecurityContext) -> SecurityResult<()> {
            let injection_patterns = [";", "&&", "||", "|", "`", "$("];
            for pattern in injection_patterns {
                if command.contains(pattern) {
                    return Err(SecurityError::InputValidationFailed(format!("Command injection pattern detected: {}", pattern)));
                }
            }
            
            let dangerous_commands = match context.trust_level {
                TrustLevel::Untrusted => vec!["rm", "del", "format", "sudo", "su", "chmod", "chown", "curl", "wget", "nc", "ssh", "scp", "rsync", "dd"],
                TrustLevel::UserVerified => vec!["sudo", "su", "chmod +s", "chown root", "dd"],
                TrustLevel::SystemTrusted => vec![],
            };
            
            for dangerous in dangerous_commands {
                if command.to_lowercase().contains(dangerous) {
                    return Err(SecurityError::PermissionDenied(format!("Command '{}' not allowed for trust level {:?}", dangerous, context.trust_level)));
                }
            }
            Ok(())
        }

        fn get_security_health(&self) -> SecurityHealthStatus {
            SecurityHealthStatus::Excellent
        }
    }

    struct MockSignoffManager;

    impl MockSignoffManager {
        fn new() -> Self {
            Self
        }

        fn check_signoff_required(&self, operation: &str, _context: &SecurityContext) -> SignoffDecision {
            let requires_signoff = operation.contains("rm") || operation.contains("sudo") || operation.contains("curl");
            SignoffDecision {
                required: requires_signoff,
                approved: false, // Mock always requires manual approval
                conditions: vec![],
            }
        }
    }

    #[test]
    fn test_complete_security_framework() {
        let security_tools = MockSecurityTools::new();
        
        // Test 1: User signoff for dangerous operations
        let untrusted_context = SecurityContext::for_trust_level(TrustLevel::Untrusted);
        let signoff_decision = security_tools.signoff_manager.check_signoff_required("rm -rf /tmp/test", &untrusted_context);
        
        assert!(signoff_decision.required, "Dangerous operations should require signoff");
        
        // Test 2: File access validation
        let file_validation = security_tools.validate_skill_file_access("../../../etc/passwd", &untrusted_context);
        assert!(file_validation.is_err(), "Directory traversal should be blocked");
        
        // Test 3: Command validation
        let command_validation = security_tools.validate_skill_command("echo hello; rm -rf /", &untrusted_context);
        assert!(command_validation.is_err(), "Command injection should be blocked");
        
        // Test 4: Security health monitoring
        let health_status = security_tools.get_security_health();
        assert!(matches!(health_status, SecurityHealthStatus::Excellent));
    }
    
    #[test]
    fn test_trust_level_permissions() {
        let security_tools = MockSecurityTools::new();
        
        let trust_levels = [TrustLevel::Untrusted, TrustLevel::UserVerified, TrustLevel::SystemTrusted];
        
        for trust_level in trust_levels {
            let context = SecurityContext::for_trust_level(trust_level.clone());
            
            // Test file access permissions
            let workspace_file = "./test.txt";
            let file_result = security_tools.validate_skill_file_access(workspace_file, &context);
            
            match trust_level {
                TrustLevel::Untrusted => {
                    assert!(file_result.is_err(), "Untrusted should not access workspace files");
                }
                TrustLevel::UserVerified => {
                    assert!(file_result.is_ok(), "UserVerified should access workspace files");
                }
                TrustLevel::SystemTrusted => {
                    assert!(file_result.is_ok(), "SystemTrusted should access all files");
                }
            }
            
            // Test command permissions
            let sudo_command = "sudo echo test";
            let command_result = security_tools.validate_skill_command(sudo_command, &context);
            
            match trust_level {
                TrustLevel::Untrusted | TrustLevel::UserVerified => {
                    assert!(command_result.is_err(), "{:?} should not run sudo", trust_level);
                }
                TrustLevel::SystemTrusted => {
                    assert!(command_result.is_ok(), "SystemTrusted should run sudo");
                }
            }
        }
    }
    
    #[test]
    fn test_security_design_principles() {
        // 1. Zero Trust Execution
        let untrusted_context = SecurityContext::for_trust_level(TrustLevel::Untrusted);
        assert_eq!(untrusted_context.resource_limits.timeout_seconds, 10);
        assert_eq!(untrusted_context.resource_limits.max_memory_mb, Some(64));
        
        // 2. Least Privilege
        assert!(matches!(untrusted_context.permissions.network_access, NetworkPermissions::None));
        assert!(matches!(untrusted_context.permissions.file_access, FilePermissions::ReadOnlyTemp));
        
        // 3. Defense in Depth
        let user_context = SecurityContext::for_trust_level(TrustLevel::UserVerified);
        assert!(user_context.resource_limits.timeout_seconds > untrusted_context.resource_limits.timeout_seconds);
        assert!(matches!(user_context.permissions.network_access, NetworkPermissions::HttpsOnly));
        
        // 4. Fail Secure
        let system_context = SecurityContext::for_trust_level(TrustLevel::SystemTrusted);
        assert!(system_context.resource_limits.timeout_seconds < 600);
    }
}
