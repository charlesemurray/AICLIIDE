use std::collections::HashMap;
use std::path::PathBuf;
use std::future::Future;
use async_trait::async_trait;
use crate::cli::skills::{SkillResult, SkillUI};

// Re-export sysinfo for cross-platform monitoring
pub use sysinfo::{System, Pid};

#[derive(Debug, thiserror::Error)]
pub enum SecurityError {
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),
    #[error("Sandbox violation: {0}")]
    SandboxViolation(String),
    #[error("Input validation failed: {0}")]
    InputValidationFailed(String),
    #[error("Output validation failed: {0}")]
    OutputValidationFailed(String),
}

pub type SecurityResult<T> = std::result::Result<T, SecurityError>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TrustLevel {
    Untrusted,      // External/unknown skills
    UserVerified,   // User-created skills  
    SystemTrusted,  // Built-in skills
}

#[derive(Debug, Clone)]
pub struct SecurityContext {
    pub trust_level: TrustLevel,
    pub permissions: PermissionSet,
    pub resource_limits: ResourceLimits,
    pub sandbox_config: SandboxConfig,
}

#[derive(Debug, Clone)]
pub struct PermissionSet {
    pub file_access: FilePermissions,
    pub network_access: NetworkPermissions,
    pub process_spawn: ProcessPermissions,
}

#[derive(Debug, Clone)]
pub enum FilePermissions {
    None,
    ReadOnlyTemp,
    WorkspaceOnly,
    Full,
}

#[derive(Debug, Clone)]
pub enum NetworkPermissions {
    None,
    HttpsOnly,
    Full,
}

#[derive(Debug, Clone)]
pub enum ProcessPermissions {
    None,
    Limited,
    Full,
}

#[derive(Debug, Clone)]
pub struct ResourceLimits {
    pub timeout_seconds: u64,
    pub max_memory_mb: Option<u64>,
    pub max_cpu_percent: Option<u64>,
    pub max_disk_io_mb: Option<u64>,
    pub max_network_requests: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct SandboxConfig {
    pub enable_network: bool,
    pub allowed_file_paths: Vec<PathBuf>,
    pub temp_directory: Option<PathBuf>,
    pub environment_variables: HashMap<String, String>,
    pub max_processes: u32,
    pub max_open_files: u32,
}

impl SecurityContext {
    pub fn for_trust_level(trust_level: TrustLevel) -> Self {
        Self {
            permissions: PermissionSet::for_trust_level(&trust_level),
            resource_limits: ResourceLimits::for_trust_level(&trust_level),
            sandbox_config: SandboxConfig::for_trust_level(&trust_level),
            trust_level,
        }
    }
}

impl PermissionSet {
    pub fn for_trust_level(trust_level: &TrustLevel) -> Self {
        match trust_level {
            TrustLevel::Untrusted => Self {
                file_access: FilePermissions::ReadOnlyTemp,
                network_access: NetworkPermissions::None,
                process_spawn: ProcessPermissions::None,
            },
            TrustLevel::UserVerified => Self {
                file_access: FilePermissions::WorkspaceOnly,
                network_access: NetworkPermissions::HttpsOnly,
                process_spawn: ProcessPermissions::Limited,
            },
            TrustLevel::SystemTrusted => Self {
                file_access: FilePermissions::Full,
                network_access: NetworkPermissions::Full,
                process_spawn: ProcessPermissions::Full,
            },
        }
    }
}

impl ResourceLimits {
    pub fn for_trust_level(trust_level: &TrustLevel) -> Self {
        match trust_level {
            TrustLevel::Untrusted => Self {
                timeout_seconds: 10,
                max_memory_mb: Some(64),
                max_cpu_percent: Some(25),
                max_disk_io_mb: Some(10),
                max_network_requests: Some(0),
            },
            TrustLevel::UserVerified => Self {
                timeout_seconds: 60,
                max_memory_mb: Some(256),
                max_cpu_percent: Some(50),
                max_disk_io_mb: Some(100),
                max_network_requests: Some(10),
            },
            TrustLevel::SystemTrusted => Self {
                timeout_seconds: 300,
                max_memory_mb: Some(1024),
                max_cpu_percent: Some(80),
                max_disk_io_mb: Some(1000),
                max_network_requests: None,
            },
        }
    }
}

impl SandboxConfig {
    pub fn for_trust_level(trust_level: &TrustLevel) -> Self {
        match trust_level {
            TrustLevel::Untrusted => Self {
                enable_network: false,
                allowed_file_paths: vec![],
                temp_directory: Some(PathBuf::from("/tmp/q-skills-untrusted")),
                environment_variables: HashMap::new(),
                max_processes: 1,
                max_open_files: 10,
            },
            TrustLevel::UserVerified => Self {
                enable_network: true,
                allowed_file_paths: vec![PathBuf::from(".")], // Current workspace
                temp_directory: Some(PathBuf::from("/tmp/q-skills-user")),
                environment_variables: HashMap::new(),
                max_processes: 5,
                max_open_files: 50,
            },
            TrustLevel::SystemTrusted => Self {
                enable_network: true,
                allowed_file_paths: vec![PathBuf::from("/")], // Full access
                temp_directory: None,
                environment_variables: std::env::vars().collect(),
                max_processes: 100,
                max_open_files: 1000,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeContext {
    pub execution_id: String,
    pub user_context: UserContext,
    pub workspace_path: PathBuf,
    pub temp_directory: PathBuf,
}

#[derive(Debug, Clone)]
pub struct UserContext {
    pub user_id: String,
    pub permissions: Vec<String>,
}

#[async_trait]
pub trait SecureSkill: Send + Sync {
    // Security metadata
    fn security_context(&self) -> &SecurityContext;
    fn required_permissions(&self) -> &PermissionSet;
    fn trust_level(&self) -> TrustLevel;
    
    // Basic skill info
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn aliases(&self) -> Vec<String> { vec![] }
    
    // Secure execution
    async fn execute_secure(
        &self, 
        params: serde_json::Value,
        runtime_context: &RuntimeContext
    ) -> SecurityResult<SkillResult>;
    
    // Security validation
    fn validate_input(&self, params: &serde_json::Value) -> SecurityResult<()>;
    fn validate_output(&self, result: &SkillResult) -> SecurityResult<()>;
    
    // UI rendering
    async fn render_ui(&self) -> SecurityResult<SkillUI>;
    fn supports_interactive(&self) -> bool { false }
}

// Cross-platform sandbox abstraction
#[async_trait]
pub trait PlatformSandbox: Send + Sync {
    async fn execute_with_timeout(&self, timeout_secs: u64) -> SecurityResult<()>;
    fn monitor_resources(&self, pid: u32) -> SecurityResult<ResourceUsage>;
    fn terminate_process(&self, pid: u32) -> SecurityResult<()>;
}

#[derive(Debug, Clone)]
pub struct ResourceUsage {
    pub cpu_percent: f32,
    pub memory_mb: u64,
    pub disk_io_mb: u64,
}

// Factory function for platform-specific sandbox
pub fn create_platform_sandbox() -> Box<dyn PlatformSandbox> {
    #[cfg(target_os = "linux")]
    return Box::new(crate::cli::skills::platform::linux::LinuxSandbox::new());
    
    #[cfg(target_os = "macos")]
    return Box::new(crate::cli::skills::platform::macos::MacOSSandbox::new());
    
    #[cfg(target_os = "windows")]
    return Box::new(crate::cli::skills::platform::windows::WindowsSandbox::new());
    
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    return Box::new(crate::cli::skills::platform::generic::GenericSandbox::new());
}
