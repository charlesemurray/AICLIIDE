# Security Integration Implementation Plan

## Context

**Design Document**: `docs/skills-system-design-v2.md`
**Phase**: Phase 1 - Security & Resilience (Critical)
**Current Status**: 0% complete - security code exists but not integrated

## Design Requirements (Phase 1)

From the design document:

1. ✅ **Sandboxed execution environment** - Code exists, not integrated
2. ✅ **Resource limits and timeouts** - Code exists, not integrated
3. ⚠️ **Error recovery and retry logic** - Partially implemented
4. ✅ **Basic security permissions** - Code exists, not integrated

## Current State

### What Exists
- `security.rs` (256 lines) - SecurityContext, TrustLevel, PermissionSet
- `security_logging.rs` (338 lines) - SecurityLogger, audit events
- `security_tools.rs` (548 lines) - File access validation, monitoring
- `security_testing.rs` (216 lines) - Security test framework

### What's Missing
- ❌ Security not called in skill execution path
- ❌ Skills run without sandboxing
- ❌ No resource limits enforced
- ❌ No permission checks
- ❌ No security logging

### Execution Path
```rust
// Current: crates/chat-cli/src/cli/chat/tools/skill_tool.rs
impl Tool for SkillTool {
    async fn execute(&self) -> Result<ToolResult> {
        let skill = registry.get(&self.skill_name)?;
        let result = skill.execute(self.params).await?;  // ← NO SECURITY
        Ok(result)
    }
}
```

## Implementation Plan

### Phase 1.1: Basic Security Integration (Week 1)

#### Step 1: Add SecurityContext to Skill Execution (Day 1)
**File**: `crates/chat-cli/src/cli/chat/tools/skill_tool.rs`

**Changes**:
```rust
use crate::cli::skills::security::{SecurityContext, TrustLevel};

impl Tool for SkillTool {
    async fn execute(&self) -> Result<ToolResult> {
        // Create security context
        let security_context = SecurityContext::for_trust_level(
            TrustLevel::UserVerified
        );
        
        // Get skill
        let skill = registry.get(&self.skill_name)?;
        
        // Execute with security context
        let result = skill.execute_with_security(
            self.params,
            &security_context
        ).await?;
        
        Ok(result)
    }
}
```

**Tests**:
- [ ] Security context created for each execution
- [ ] Trust level correctly set
- [ ] Execution fails without security context

**Git Commit**: `feat(skills): add SecurityContext to skill execution`

---

#### Step 2: Implement execute_with_security (Day 1-2)
**File**: `crates/chat-cli/src/cli/skills/mod.rs`

**Changes**:
```rust
#[async_trait]
pub trait Skill: Send + Sync {
    // Existing
    async fn execute(&self, params: Value) -> Result<SkillResult, SkillError>;
    
    // New - with security
    async fn execute_with_security(
        &self,
        params: Value,
        security_context: &SecurityContext,
    ) -> Result<SkillResult, SkillError> {
        // Validate permissions
        security_context.validate_execution(self)?;
        
        // Execute with monitoring
        self.execute(params).await
    }
}
```

**Tests**:
- [ ] Permission validation works
- [ ] Execution blocked if permissions denied
- [ ] Error messages are clear

**Git Commit**: `feat(skills): implement execute_with_security trait method`

---

#### Step 3: Add Permission Checks (Day 2-3)
**File**: `crates/chat-cli/src/cli/skills/security.rs`

**Changes**:
```rust
impl SecurityContext {
    pub fn validate_execution(&self, skill: &dyn Skill) -> Result<(), SecurityError> {
        // Check file permissions
        if let Some(file_access) = skill.required_file_access() {
            self.permissions.validate_file_access(&file_access)?;
        }
        
        // Check network permissions
        if skill.requires_network() {
            self.permissions.validate_network_access()?;
        }
        
        // Check process permissions
        if skill.requires_process_spawn() {
            self.permissions.validate_process_spawn()?;
        }
        
        Ok(())
    }
}
```

**Tests**:
- [ ] File access validated
- [ ] Network access validated
- [ ] Process spawn validated
- [ ] Violations blocked

**Git Commit**: `feat(skills): implement permission validation`

---

#### Step 4: Add Resource Limits (Day 3-4)
**File**: `crates/chat-cli/src/cli/skills/security.rs`

**Changes**:
```rust
impl SecurityContext {
    pub async fn execute_with_limits<F, T>(
        &self,
        future: F,
    ) -> Result<T, SecurityError>
    where
        F: Future<Output = Result<T, SkillError>>,
    {
        // Apply timeout
        let timeout_duration = Duration::from_secs(
            self.resource_limits.max_execution_time
        );
        
        // Execute with timeout
        match tokio::time::timeout(timeout_duration, future).await {
            Ok(result) => result.map_err(SecurityError::from),
            Err(_) => Err(SecurityError::Timeout(
                self.resource_limits.max_execution_time
            )),
        }
    }
}
```

**Tests**:
- [ ] Timeout enforced
- [ ] Long-running skills killed
- [ ] Error message clear

**Git Commit**: `feat(skills): implement resource limits and timeouts`

---

#### Step 5: Add Security Logging (Day 4-5)
**File**: `crates/chat-cli/src/cli/chat/tools/skill_tool.rs`

**Changes**:
```rust
use crate::cli::skills::security_logging::SecurityLogger;

impl Tool for SkillTool {
    async fn execute(&self) -> Result<ToolResult> {
        let security_context = SecurityContext::for_trust_level(TrustLevel::UserVerified);
        let security_logger = SecurityLogger::new(log_dir);
        
        // Log execution start
        security_logger.log_execution_start(&self.skill_name, &security_context).await?;
        
        let skill = registry.get(&self.skill_name)?;
        
        // Execute with security
        let result = match skill.execute_with_security(self.params, &security_context).await {
            Ok(r) => {
                security_logger.log_execution_success(&self.skill_name).await?;
                r
            }
            Err(e) => {
                security_logger.log_execution_failure(&self.skill_name, &e).await?;
                return Err(e.into());
            }
        };
        
        Ok(result)
    }
}
```

**Tests**:
- [ ] Execution logged
- [ ] Success logged
- [ ] Failures logged
- [ ] Log format correct

**Git Commit**: `feat(skills): add security logging to skill execution`

---

### Phase 1.2: Retry Logic (Week 2)

#### Step 6: Implement Retry Logic (Day 6-7)
**File**: `crates/chat-cli/src/cli/skills/mod.rs`

**Changes**:
```rust
impl Skill {
    async fn execute_with_retry(
        &self,
        params: Value,
        security_context: &SecurityContext,
    ) -> Result<SkillResult, SkillError> {
        let max_retries = self.config().retry_attempts.unwrap_or(0);
        let mut last_error = None;
        
        for attempt in 0..=max_retries {
            match self.execute_with_security(params.clone(), security_context).await {
                Ok(result) => return Ok(result),
                Err(e) if e.is_retryable() && attempt < max_retries => {
                    last_error = Some(e);
                    tokio::time::sleep(Duration::from_secs(2_u64.pow(attempt))).await;
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
        
        Err(last_error.unwrap())
    }
}
```

**Tests**:
- [ ] Retries on transient failures
- [ ] Exponential backoff works
- [ ] Max retries respected
- [ ] Non-retryable errors fail immediately

**Git Commit**: `feat(skills): implement retry logic with exponential backoff`

---

#### Step 7: Implement Fallback Commands (Day 7-8)
**File**: `crates/chat-cli/src/cli/skills/mod.rs`

**Changes**:
```rust
impl Skill {
    async fn execute_with_fallback(
        &self,
        params: Value,
        security_context: &SecurityContext,
    ) -> Result<SkillResult, SkillError> {
        match self.execute_with_retry(params.clone(), security_context).await {
            Ok(result) => Ok(result),
            Err(e) => {
                if let Some(fallback) = self.config().fallback_command {
                    // Execute fallback
                    self.execute_fallback(fallback, params, security_context).await
                } else {
                    Err(e)
                }
            }
        }
    }
}
```

**Tests**:
- [ ] Fallback executed on failure
- [ ] Fallback has same security context
- [ ] No fallback if not configured

**Git Commit**: `feat(skills): implement fallback command execution`

---

### Phase 1.3: Integration & Testing (Week 2)

#### Step 8: Wire Everything Together (Day 9)
**File**: `crates/chat-cli/src/cli/chat/tools/skill_tool.rs`

**Final Implementation**:
```rust
impl Tool for SkillTool {
    async fn execute(&self) -> Result<ToolResult> {
        // Setup security
        let security_context = SecurityContext::for_trust_level(TrustLevel::UserVerified);
        let security_logger = SecurityLogger::new(log_dir);
        
        // Log start
        security_logger.log_execution_start(&self.skill_name, &security_context).await?;
        
        // Get skill
        let skill = registry.get(&self.skill_name)?;
        
        // Execute with full security stack
        let result = security_context.execute_with_limits(
            skill.execute_with_fallback(self.params, &security_context)
        ).await;
        
        // Log result
        match &result {
            Ok(_) => security_logger.log_execution_success(&self.skill_name).await?,
            Err(e) => security_logger.log_execution_failure(&self.skill_name, e).await?,
        }
        
        result.map_err(Into::into)
    }
}
```

**Git Commit**: `feat(skills): complete Phase 1 security integration`

---

#### Step 9: Integration Tests (Day 9-10)
**File**: `crates/chat-cli/tests/skills_security_integration.rs`

**Tests**:
```rust
#[tokio::test]
async fn test_skill_respects_file_permissions() {
    // Skill tries to read /etc/passwd
    // Should be blocked
}

#[tokio::test]
async fn test_skill_respects_resource_limits() {
    // Skill tries to use 1GB memory
    // Should be killed
}

#[tokio::test]
async fn test_skill_respects_timeout() {
    // Skill runs for 5 minutes
    // Should timeout at 30s
}

#[tokio::test]
async fn test_retry_logic_works() {
    // Skill fails twice, succeeds third time
    // Should retry and succeed
}

#[tokio::test]
async fn test_fallback_executes() {
    // Skill fails, fallback succeeds
    // Should return fallback result
}

#[tokio::test]
async fn test_security_logging() {
    // Execute skill
    // Check logs exist and are correct
}
```

**Git Commit**: `test(skills): add Phase 1 security integration tests`

---

#### Step 10: Documentation (Day 10)
**Files**: 
- `docs/SECURITY_INTEGRATION_COMPLETE.md`
- Update `docs/skills-system-design-v2.md`

**Content**:
- Phase 1 completion status
- Security features enabled
- How to configure security
- Examples of secure skills
- Troubleshooting guide

**Git Commit**: `docs(skills): document Phase 1 security integration`

---

## Success Criteria

### Functional Requirements
- [ ] All skills execute with SecurityContext
- [ ] Permission checks enforced
- [ ] Resource limits enforced
- [ ] Timeouts enforced
- [ ] Retry logic works
- [ ] Fallback commands work
- [ ] Security logging works

### Non-Functional Requirements
- [ ] Performance impact < 10ms per execution
- [ ] No breaking changes to existing skills
- [ ] Clear error messages
- [ ] Comprehensive test coverage (>80%)

### Documentation
- [ ] Design document updated
- [ ] Integration guide written
- [ ] Security configuration documented
- [ ] Examples provided

## Timeline

**Week 1**: Steps 1-5 (Basic security integration)
**Week 2**: Steps 6-10 (Retry logic, testing, docs)
**Total**: 10 days (2 weeks)

## Risks & Mitigation

### Risk 1: Breaking Existing Skills
**Mitigation**: Default security context allows everything, gradual rollout

### Risk 2: Performance Impact
**Mitigation**: Benchmark each step, optimize if needed

### Risk 3: Complex Error Handling
**Mitigation**: Clear error types, good error messages

## Verification

After completion, run design-aware adversary:
```bash
# Check security integration
grep -c "SecurityContext" crates/chat-cli/src/cli/chat/tools/skill_tool.rs
# Expected: >5

# Check resource limits
grep -c "execute_with_limits" crates/chat-cli/src/cli/chat/tools/skill_tool.rs
# Expected: >1

# Check logging
grep -c "SecurityLogger" crates/chat-cli/src/cli/chat/tools/skill_tool.rs
# Expected: >3

# Run tests
cargo test security_integration
# Expected: All pass
```

## Git Commits Summary

1. `feat(skills): add SecurityContext to skill execution`
2. `feat(skills): implement execute_with_security trait method`
3. `feat(skills): implement permission validation`
4. `feat(skills): implement resource limits and timeouts`
5. `feat(skills): add security logging to skill execution`
6. `feat(skills): implement retry logic with exponential backoff`
7. `feat(skills): implement fallback command execution`
8. `feat(skills): complete Phase 1 security integration`
9. `test(skills): add Phase 1 security integration tests`
10. `docs(skills): document Phase 1 security integration`

**Total**: 10 commits over 2 weeks
