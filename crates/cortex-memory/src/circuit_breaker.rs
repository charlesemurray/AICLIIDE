//! Circuit breaker for fault tolerance

use std::time::{
    Duration,
    Instant,
};

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircuitState {
    /// Normal operation
    Closed,
    /// Failing, reject requests
    Open,
    /// Testing recovery
    HalfOpen,
}

/// Circuit breaker for memory operations
pub struct CircuitBreaker {
    failure_count: u32,
    success_count: u32,
    state: CircuitState,
    last_failure: Option<Instant>,
    failure_threshold: u32,
    cooldown_duration: Duration,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, cooldown_duration: Duration) -> Self {
        Self {
            failure_count: 0,
            success_count: 0,
            state: CircuitState::Closed,
            last_failure: None,
            failure_threshold,
            cooldown_duration,
        }
    }

    /// Check if operation should be allowed
    pub fn should_allow(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if cooldown period has passed
                if let Some(last_failure) = self.last_failure {
                    if last_failure.elapsed() >= self.cooldown_duration {
                        tracing::info!("Circuit breaker entering half-open state");
                        self.state = CircuitState::HalfOpen;
                        self.failure_count = 0;
                        self.success_count = 0;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            },
            CircuitState::HalfOpen => true,
        }
    }

    /// Record successful operation
    pub fn record_success(&mut self) {
        self.success_count += 1;

        match self.state {
            CircuitState::HalfOpen => {
                // After 3 successes in half-open, close the circuit
                if self.success_count >= 3 {
                    tracing::info!("Circuit breaker closing after successful recovery");
                    self.state = CircuitState::Closed;
                    self.failure_count = 0;
                    self.success_count = 0;
                }
            },
            CircuitState::Closed => {
                // Reset failure count on success
                if self.failure_count > 0 {
                    self.failure_count = 0;
                }
            },
            CircuitState::Open => {},
        }
    }

    /// Record failed operation
    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure = Some(Instant::now());

        match self.state {
            CircuitState::Closed => {
                if self.failure_count >= self.failure_threshold {
                    tracing::warn!(
                        failure_count = self.failure_count,
                        "Circuit breaker opening due to failures"
                    );
                    self.state = CircuitState::Open;
                }
            },
            CircuitState::HalfOpen => {
                // Any failure in half-open goes back to open
                tracing::warn!("Circuit breaker reopening after failure in half-open state");
                self.state = CircuitState::Open;
                self.success_count = 0;
            },
            CircuitState::Open => {},
        }
    }

    /// Get current state
    pub fn state(&self) -> CircuitState {
        self.state
    }

    /// Get failure count
    pub fn failure_count(&self) -> u32 {
        self.failure_count
    }
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new(10, Duration::from_secs(60))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker_opens_after_threshold() {
        let mut cb = CircuitBreaker::new(3, Duration::from_secs(60));

        assert_eq!(cb.state(), CircuitState::Closed);
        assert!(cb.should_allow());

        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Closed);

        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
        assert!(!cb.should_allow());
    }

    #[test]
    fn test_circuit_breaker_recovers() {
        let mut cb = CircuitBreaker::new(2, Duration::from_millis(10));

        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);

        std::thread::sleep(Duration::from_millis(20));
        assert!(cb.should_allow());
        assert_eq!(cb.state(), CircuitState::HalfOpen);

        cb.record_success();
        cb.record_success();
        cb.record_success();
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn test_circuit_breaker_reopens_on_half_open_failure() {
        let mut cb = CircuitBreaker::new(2, Duration::from_millis(10));

        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);

        std::thread::sleep(Duration::from_millis(20));
        cb.should_allow();
        assert_eq!(cb.state(), CircuitState::HalfOpen);

        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
    }
}
