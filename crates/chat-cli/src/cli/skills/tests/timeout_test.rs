#[cfg(test)]
mod timeout_test {
    use crate::cli::skills::{ResourceLimits, execute_with_timeout, SkillResult, SkillError};
    use std::time::{Duration, Instant};
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_timeout_functionality() {
        let limits = ResourceLimits {
            timeout_seconds: 1,
            max_memory_mb: Some(512),
            max_cpu_percent: Some(80),
        };

        // Test that a quick operation completes successfully
        let quick_future = async {
            sleep(Duration::from_millis(100)).await;
            Ok(SkillResult {
                output: "Quick operation".to_string(),
                ui_updates: None,
                state_changes: None,
            })
        };

        let start = Instant::now();
        let result = execute_with_timeout(quick_future, &limits).await;
        let duration = start.elapsed();

        assert!(result.is_ok(), "Quick operation should succeed");
        assert!(duration < Duration::from_secs(1), "Quick operation should complete fast");

        // Test that a slow operation times out
        let slow_future = async {
            sleep(Duration::from_secs(2)).await;
            Ok(SkillResult {
                output: "Slow operation".to_string(),
                ui_updates: None,
                state_changes: None,
            })
        };

        let start = Instant::now();
        let result = execute_with_timeout(slow_future, &limits).await;
        let duration = start.elapsed();

        assert!(result.is_err(), "Slow operation should timeout");
        if let Err(SkillError::Timeout(timeout_secs)) = result {
            assert_eq!(timeout_secs, 1, "Should report correct timeout duration");
        } else {
            panic!("Expected timeout error");
        }
        assert!(duration >= Duration::from_secs(1), "Should wait for timeout");
        assert!(duration < Duration::from_millis(1100), "Should timeout promptly");
    }
}
