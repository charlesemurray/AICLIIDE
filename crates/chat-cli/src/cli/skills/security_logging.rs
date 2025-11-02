use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
    pub timestamp: DateTime<Utc>,
    pub event_id: String,
    pub event_type: SecurityEventType,
    pub skill_name: String,
    pub skill_trust_level: String,
    pub user_context: Option<String>,
    pub details: serde_json::Value,
    pub risk_level: RiskLevel,
    pub action_taken: SecurityAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityEventType {
    SkillExecutionStarted,
    SkillExecutionCompleted,
    SkillExecutionFailed,
    SecurityViolationBlocked,
    ResourceLimitExceeded,
    SuspiciousActivity,
    PermissionDenied,
    InputValidationFailed,
    OutputValidationFailed,
    SandboxViolation,
    UnauthorizedFileAccess,
    UnauthorizedNetworkAccess,
    PrivilegeEscalationAttempt,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityAction {
    Allowed,
    Blocked,
    Sanitized,
    Terminated,
    Quarantined,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTrace {
    pub execution_id: String,
    pub skill_name: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub commands_executed: Vec<String>,
    pub files_accessed: Vec<PathBuf>,
    pub network_requests: Vec<String>,
    pub resource_usage: ResourceUsageTrace,
    pub exit_code: Option<i32>,
    pub security_violations: Vec<SecurityViolation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsageTrace {
    pub peak_memory_mb: u64,
    pub peak_cpu_percent: f32,
    pub total_disk_io_mb: u64,
    pub network_bytes_sent: u64,
    pub network_bytes_received: u64,
    pub execution_duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityViolation {
    pub timestamp: DateTime<Utc>,
    pub violation_type: String,
    pub description: String,
    pub attempted_action: String,
    pub blocked: bool,
}

pub struct SecurityLogger {
    log_file_path: PathBuf,
    trace_file_path: PathBuf,
    alert_threshold: RiskLevel,
}

impl SecurityLogger {
    pub fn new(log_dir: PathBuf) -> Self {
        Self {
            log_file_path: log_dir.join("security_events.jsonl"),
            trace_file_path: log_dir.join("execution_traces.jsonl"),
            alert_threshold: RiskLevel::High,
        }
    }

    pub async fn log_security_event(&self, event: SecurityEvent) -> Result<(), std::io::Error> {
        // Log to file
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file_path)
            .await?;

        let event_json = serde_json::to_string(&event)?;
        file.write_all(format!("{}\n", event_json).as_bytes()).await?;
        file.flush().await?;

        // Console logging for high-risk events
        if event.risk_level >= self.alert_threshold {
            eprintln!("🚨 SECURITY ALERT: {:?} - {}", event.event_type, event.details);
        }

        // Structured logging for debugging
        tracing::info!(
            event_type = ?event.event_type,
            risk_level = ?event.risk_level,
            skill_name = %event.skill_name,
            action_taken = ?event.action_taken,
            "Security event logged"
        );

        Ok(())
    }

    pub async fn log_execution_trace(&self, trace: ExecutionTrace) -> Result<(), std::io::Error> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.trace_file_path)
            .await?;

        let trace_json = serde_json::to_string(&trace)?;
        file.write_all(format!("{}\n", trace_json).as_bytes()).await?;
        file.flush().await?;

        // Log summary to console
        println!(
            "📊 Execution trace: {} - Duration: {}ms, Memory: {}MB, Violations: {}",
            trace.skill_name,
            trace.resource_usage.execution_duration_ms,
            trace.resource_usage.peak_memory_mb,
            trace.security_violations.len()
        );

        Ok(())
    }

    pub fn create_security_event(
        &self,
        event_type: SecurityEventType,
        skill_name: String,
        details: serde_json::Value,
    ) -> SecurityEvent {
        let risk_level = self.assess_risk_level(&event_type, &details);
        let action_taken = self.determine_action(&event_type, &risk_level);

        SecurityEvent {
            timestamp: Utc::now(),
            event_id: uuid::Uuid::new_v4().to_string(),
            event_type,
            skill_name,
            skill_trust_level: "unknown".to_string(), // Will be filled by caller
            user_context: None,
            details,
            risk_level,
            action_taken,
        }
    }

    fn assess_risk_level(&self, event_type: &SecurityEventType, _details: &serde_json::Value) -> RiskLevel {
        match event_type {
            SecurityEventType::PrivilegeEscalationAttempt => RiskLevel::Critical,
            SecurityEventType::UnauthorizedFileAccess => RiskLevel::High,
            SecurityEventType::UnauthorizedNetworkAccess => RiskLevel::High,
            SecurityEventType::SandboxViolation => RiskLevel::High,
            SecurityEventType::SecurityViolationBlocked => RiskLevel::Medium,
            SecurityEventType::ResourceLimitExceeded => RiskLevel::Medium,
            SecurityEventType::InputValidationFailed => RiskLevel::Medium,
            SecurityEventType::SuspiciousActivity => RiskLevel::Medium,
            SecurityEventType::PermissionDenied => RiskLevel::Low,
            SecurityEventType::SkillExecutionStarted => RiskLevel::Low,
            SecurityEventType::SkillExecutionCompleted => RiskLevel::Low,
            SecurityEventType::SkillExecutionFailed => RiskLevel::Low,
            SecurityEventType::OutputValidationFailed => RiskLevel::Medium,
        }
    }

    fn determine_action(&self, event_type: &SecurityEventType, risk_level: &RiskLevel) -> SecurityAction {
        match (event_type, risk_level) {
            (SecurityEventType::PrivilegeEscalationAttempt, _) => SecurityAction::Blocked,
            (SecurityEventType::UnauthorizedFileAccess, _) => SecurityAction::Blocked,
            (SecurityEventType::UnauthorizedNetworkAccess, _) => SecurityAction::Blocked,
            (SecurityEventType::SandboxViolation, _) => SecurityAction::Terminated,
            (SecurityEventType::ResourceLimitExceeded, _) => SecurityAction::Terminated,
            (SecurityEventType::InputValidationFailed, _) => SecurityAction::Sanitized,
            (SecurityEventType::OutputValidationFailed, _) => SecurityAction::Sanitized,
            (_, RiskLevel::Critical) => SecurityAction::Blocked,
            (_, RiskLevel::High) => SecurityAction::Blocked,
            _ => SecurityAction::Allowed,
        }
    }
}

pub struct SecurityMetrics {
    pub total_executions: u64,
    pub blocked_executions: u64,
    pub security_violations: u64,
    pub high_risk_events: u64,
    pub average_execution_time_ms: f64,
    pub resource_limit_violations: u64,
}

impl SecurityMetrics {
    pub fn new() -> Self {
        Self {
            total_executions: 0,
            blocked_executions: 0,
            security_violations: 0,
            high_risk_events: 0,
            average_execution_time_ms: 0.0,
            resource_limit_violations: 0,
        }
    }

    pub fn record_execution(&mut self, trace: &ExecutionTrace) {
        self.total_executions += 1;

        if !trace.security_violations.is_empty() {
            self.security_violations += trace.security_violations.len() as u64;
        }

        // Update average execution time
        let current_avg = self.average_execution_time_ms;
        let new_time = trace.resource_usage.execution_duration_ms as f64;
        self.average_execution_time_ms =
            (current_avg * (self.total_executions - 1) as f64 + new_time) / self.total_executions as f64;
    }

    pub fn record_security_event(&mut self, event: &SecurityEvent) {
        match event.action_taken {
            SecurityAction::Blocked | SecurityAction::Terminated => {
                self.blocked_executions += 1;
            },
            _ => {},
        }

        if event.risk_level >= RiskLevel::High {
            self.high_risk_events += 1;
        }

        if matches!(event.event_type, SecurityEventType::ResourceLimitExceeded) {
            self.resource_limit_violations += 1;
        }
    }

    pub fn security_score(&self) -> f64 {
        if self.total_executions == 0 {
            return 100.0;
        }

        let violation_rate = self.security_violations as f64 / self.total_executions as f64;
        let block_rate = self.blocked_executions as f64 / self.total_executions as f64;
        let high_risk_rate = self.high_risk_events as f64 / self.total_executions as f64;

        // Higher scores are better (fewer violations)
        let score = 100.0 - (violation_rate * 50.0 + block_rate * 30.0 + high_risk_rate * 20.0);
        score.max(0.0).min(100.0)
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[tokio::test]
    async fn test_security_logging() {
        let temp_dir = TempDir::new().unwrap();
        let logger = SecurityLogger::new(temp_dir.path().to_path_buf());

        let event = logger.create_security_event(
            SecurityEventType::SecurityViolationBlocked,
            "test-skill".to_string(),
            serde_json::json!({"violation": "command injection attempt"}),
        );

        assert!(logger.log_security_event(event).await.is_ok());

        // Verify log file was created
        assert!(temp_dir.path().join("security_events.jsonl").exists());
    }

    #[test]
    fn test_security_metrics() {
        let mut metrics = SecurityMetrics::new();

        let trace = ExecutionTrace {
            execution_id: "test".to_string(),
            skill_name: "test-skill".to_string(),
            start_time: Utc::now(),
            end_time: Some(Utc::now()),
            commands_executed: vec![],
            files_accessed: vec![],
            network_requests: vec![],
            resource_usage: ResourceUsageTrace {
                peak_memory_mb: 100,
                peak_cpu_percent: 50.0,
                total_disk_io_mb: 10,
                network_bytes_sent: 0,
                network_bytes_received: 0,
                execution_duration_ms: 1000,
            },
            exit_code: Some(0),
            security_violations: vec![],
        };

        metrics.record_execution(&trace);
        assert_eq!(metrics.total_executions, 1);
        assert_eq!(metrics.average_execution_time_ms, 1000.0);

        let initial_score = metrics.security_score();
        assert_eq!(initial_score, 100.0); // Perfect score with no violations
    }
}
