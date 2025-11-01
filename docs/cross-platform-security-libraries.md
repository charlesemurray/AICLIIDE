# Cross-Platform Security Libraries for Skills System

## Recommended Crates

### 1. **tokio-process** + **nix** (Linux/Mac) + **winapi** (Windows)
```toml
tokio-process = "0.2"
nix = "0.27"           # Linux/Mac process control
winapi = "0.3"         # Windows process control
libc = "0.2"           # Cross-platform system calls
```

**Capabilities:**
- Process spawning with resource limits
- Signal handling and process termination
- Cross-platform process isolation
- Resource monitoring (CPU, memory)

### 2. **bubblewrap-rs** (Linux) + **sandbox-exec** (Mac) + **Windows Sandbox**
```toml
# Linux sandboxing
bubblewrap = "0.1"     # Linux namespace sandboxing

# Cross-platform alternatives
jail = "0.2"           # FreeBSD jails (limited)
```

**Capabilities:**
- Filesystem isolation
- Network isolation  
- Process namespace isolation
- User/group isolation

### 3. **Resource Monitoring: sysinfo**
```toml
sysinfo = "0.29"       # Cross-platform system information
```

**Capabilities:**
- CPU usage monitoring
- Memory usage monitoring
- Process monitoring
- Disk I/O monitoring
- Network monitoring

### 4. **Security: seccomp-sys** (Linux) + **pledge** (OpenBSD-style)
```toml
seccomp-sys = "0.1"    # Linux syscall filtering
caps = "0.5"           # Linux capabilities
```

**Capabilities:**
- System call filtering
- Capability dropping
- Privilege reduction

## Cross-Platform Strategy

### Platform-Specific Implementations

```rust
#[cfg(target_os = "linux")]
mod linux_sandbox {
    use bubblewrap::Sandbox;
    use seccomp_sys::*;
    
    pub struct LinuxSandbox {
        // Linux-specific sandboxing using namespaces
    }
}

#[cfg(target_os = "macos")]
mod macos_sandbox {
    use std::process::Command;
    
    pub struct MacOSSandbox {
        // macOS sandbox-exec wrapper
    }
}

#[cfg(target_os = "windows")]
mod windows_sandbox {
    use winapi::um::processthreadsapi::*;
    
    pub struct WindowsSandbox {
        // Windows job objects and restricted tokens
    }
}
```

### Unified Interface

```rust
pub trait PlatformSandbox {
    async fn execute_sandboxed<F, T>(&self, future: F, config: &SandboxConfig) -> SecurityResult<T>
    where
        F: Future<Output = SecurityResult<T>> + Send,
        T: Send;
}

pub fn create_platform_sandbox() -> Box<dyn PlatformSandbox> {
    #[cfg(target_os = "linux")]
    return Box::new(linux_sandbox::LinuxSandbox::new());
    
    #[cfg(target_os = "macos")]
    return Box::new(macos_sandbox::MacOSSandbox::new());
    
    #[cfg(target_os = "windows")]
    return Box::new(windows_sandbox::WindowsSandbox::new());
}
```

## Recommended Dependencies

```toml
[dependencies]
# Core async runtime
tokio = { version = "1.0", features = ["full"] }

# Cross-platform system info and monitoring
sysinfo = "0.29"
libc = "0.2"

# Process management
tokio-process = "0.2"

# Platform-specific security
[target.'cfg(unix)'.dependencies]
nix = "0.27"
caps = "0.5"

[target.'cfg(target_os = "linux")'.dependencies]
seccomp-sys = "0.1"

[target.'cfg(windows)'.dependencies]
winapi = { version = "0.3", features = ["processthreadsapi", "jobapi2", "securitybaseapi"] }
```

## Implementation Approach

### Phase 1: Basic Resource Limits (Cross-Platform)
- Use `sysinfo` for monitoring
- Use `tokio::time::timeout` for time limits
- Use process spawning with limits

### Phase 2: Process Isolation (Platform-Specific)
- Linux: Use namespaces via `nix` crate
- macOS: Use `sandbox-exec` command wrapper
- Windows: Use job objects via `winapi`

### Phase 3: Advanced Sandboxing (Platform-Specific)
- Linux: Implement seccomp filters
- macOS: Use macOS sandbox profiles
- Windows: Use restricted tokens and AppContainer

### Phase 4: Unified Security Layer
- Abstract platform differences
- Provide consistent security guarantees
- Fallback to basic limits if advanced features unavailable

## Security Guarantees by Platform

| Feature | Linux | macOS | Windows |
|---------|-------|-------|---------|
| Process Limits | ✅ | ✅ | ✅ |
| Memory Limits | ✅ | ✅ | ✅ |
| CPU Limits | ✅ | ✅ | ✅ |
| Filesystem Isolation | ✅ | ✅ | ⚠️ |
| Network Isolation | ✅ | ✅ | ⚠️ |
| Syscall Filtering | ✅ | ⚠️ | ❌ |

✅ = Full support, ⚠️ = Limited support, ❌ = Not available

This approach provides the best security possible on each platform while maintaining a consistent interface.
