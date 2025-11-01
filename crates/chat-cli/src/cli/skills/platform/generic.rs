use crate::cli::skills::security::{PlatformSandbox, SecurityResult, SecurityError, SandboxConfig, ResourceUsage};
use async_trait::async_trait;
use std::future::Future;
use tokio::time::{timeout, Duration};
use sysinfo::{System, SystemExt, ProcessExt, Pid};

pub struct GenericSandbox {
    system: System,
}

impl GenericSandbox {
    pub fn new() -> Self {
        Self {
            system: System::new_all(),
        }
    }
}

#[async_trait]
impl PlatformSandbox for GenericSandbox {
    async fn execute_sandboxed<F, T>(&self, future: F, config: &SandboxConfig) -> SecurityResult<T>
    where
        F: Future<Output = SecurityResult<T>> + Send,
        T: Send,
    {
        // Basic timeout-based execution for unsupported platforms
        // This provides minimal security but ensures cross-platform compatibility
        
        let timeout_duration = Duration::from_secs(30); // Default timeout
        
        match timeout(timeout_duration, future).await {
            Ok(result) => result,
            Err(_) => Err(SecurityError::ResourceLimitExceeded(
                "Execution timeout exceeded".to_string()
            )),
        }
    }
    
    fn monitor_resources(&self, pid: u32) -> SecurityResult<ResourceUsage> {
        let mut system = System::new_all();
        system.refresh_all();
        
        if let Some(process) = system.process(Pid::from(pid as usize)) {
            Ok(ResourceUsage {
                cpu_percent: process.cpu_usage(),
                memory_mb: process.memory() / 1024 / 1024,
                disk_io_mb: 0, // Not available in generic implementation
            })
        } else {
            Err(SecurityError::SandboxViolation(
                format!("Process {} not found", pid)
            ))
        }
    }
    
    fn terminate_process(&self, pid: u32) -> SecurityResult<()> {
        let mut system = System::new_all();
        system.refresh_all();
        
        if let Some(process) = system.process(Pid::from(pid as usize)) {
            if process.kill() {
                Ok(())
            } else {
                Err(SecurityError::SandboxViolation(
                    format!("Failed to terminate process {}", pid)
                ))
            }
        } else {
            Err(SecurityError::SandboxViolation(
                format!("Process {} not found", pid)
            ))
        }
    }
}
