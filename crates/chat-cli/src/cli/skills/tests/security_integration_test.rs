#[cfg(test)]
mod security_integration_tests {
    use crate::cli::skills::security::*;
    use crate::cli::skills::security_tools::*;
    use tempfile::TempDir;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_complete_security_framework() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().to_path_buf();
        
        // Initialize git repo for testing
        std::process::Command::new("git")
            .args(&["init"])
            .current_dir(&repo_path)
            .output()
            .ok();
        
        std::process::Command::new("git")
            .args(&["config", "user.email", "test@example.com"])
            .current_dir(&repo_path)
            .output()
            .ok();
            
        std::process::Command::new("git")
            .args(&["config", "user.name", "Test User"])
            .current_dir(&repo_path)
            .output()
            .ok();
        
        let security_tools = SkillSecurityTools::new(
            temp_dir.path().to_path_buf(),
            repo_path.clone()
        );
        
        // Test 1: User signoff for dangerous operations
        let untrusted_context = SecurityContext::for_trust_level(TrustLevel::Untrusted);
        let signoff_decision = security_tools.signoff_manager
            .check_signoff_required("rm -rf /tmp/test", &untrusted_context)
            .await;
        
        // Should require signoff for dangerous command
        assert!(signoff_decision.is_ok());
        let decision = signoff_decision.unwrap();
        assert!(decision.required, "Dangerous operations should require signoff");
        
        // Test 2: File access validation
        let file_validation = security_tools.validate_skill_file_access(
            "../../../etc/passwd", 
            &untrusted_context
        );
        assert!(file_validation.is_err(), "Directory traversal should be blocked");
        
        // Test 3: Command validation
        let command_validation = security_tools.validate_skill_command(
            "echo hello; rm -rf /", 
            &untrusted_context
        );
        assert!(command_validation.is_err(), "Command injection should be blocked");
        
        // Test 4: Security health monitoring
        let health_status = security_tools.get_security_health();
        assert!(matches!(health_status, SecurityHealthStatus::Excellent));
        
        println!("✅ All security framework tests passed!");
        println!("🔐 User signoff integration: Working");
        println!("📁 File access validation: Working");
        println!("⚡ Command validation: Working");
        println!("📊 Security monitoring: Working");
        println!("{} Security health: {} {}", 
            health_status.as_emoji(),
            format!("{:?}", health_status),
            health_status.as_description()
        );
    }
    
    #[tokio::test]
    async fn test_trust_level_permissions() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().to_path_buf();
        
        let security_tools = SkillSecurityTools::new(
            temp_dir.path().to_path_buf(),
            repo_path
        );
        
        // Test different trust levels
        let trust_levels = [
            TrustLevel::Untrusted,
            TrustLevel::UserVerified,
            TrustLevel::SystemTrusted,
        ];
        
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
        
        println!("✅ Trust level permission tests passed!");
    }
    
    #[test]
    fn test_security_design_principles() {
        // Verify our security design principles are implemented
        
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
        assert!(system_context.resource_limits.timeout_seconds < 600); // Still has limits
        
        println!("✅ Security design principles verified!");
        println!("🛡️  Zero Trust: Implemented");
        println!("🔒 Least Privilege: Implemented");
        println!("🏰 Defense in Depth: Implemented");
        println!("🚫 Fail Secure: Implemented");
    }
}
