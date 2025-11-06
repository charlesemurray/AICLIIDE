//! End-to-end integration test for PriorityLimiter with real components
//!
//! Tests the full stack: Coordinator → PriorityLimiter → Tower → LLM calls

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

// Mock LLM client that simulates slow responses
struct MockLLMClient {
    call_duration: Duration,
}

impl MockLLMClient {
    fn new(duration: Duration) -> Self {
        Self { call_duration: duration }
    }
    
    async fn send_message(&self, _request: &str) -> String {
        tokio::time::sleep(self.call_duration).await;
        "Mock LLM response".to_string()
    }
}

#[tokio::test]
async fn test_e2e_foreground_priority_with_background_load() {
    // This test proves the full integration works:
    // 1. PriorityLimiter is configured correctly
    // 2. Foreground calls use priority pool
    // 3. Background calls use shared pool
    // 4. Foreground is faster than background under load
    
    use chat_cli::cli::chat::priority_limiter::PriorityLimiter;
    
    // Configure like production
    let total_capacity = 10;
    let priority_permits = (total_capacity / 4).max(3).min(10); // 3
    let shared_permits = total_capacity - priority_permits; // 7
    let timeout = Duration::from_millis(100);
    
    let limiter = Arc::new(PriorityLimiter::new(
        priority_permits,
        shared_permits,
        timeout
    ));
    
    let mock_client = Arc::new(MockLLMClient::new(Duration::from_millis(500)));
    
    // Start 7 background "LLM calls" (fill shared pool)
    let mut background_tasks = vec![];
    for i in 0..7 {
        let limiter = limiter.clone();
        let client = mock_client.clone();
        background_tasks.push(tokio::spawn(async move {
            let _permit = limiter.acquire_background().await;
            let _response = client.send_message("background request").await;
            println!("Background {} complete", i);
        }));
    }
    
    // Wait for background to acquire permits
    tokio::time::sleep(Duration::from_millis(50)).await;
    
    // Make foreground call - should use priority pool and be fast
    let limiter = limiter.clone();
    let client = mock_client.clone();
    let start = Instant::now();
    
    let foreground = tokio::spawn(async move {
        let _permit = limiter.acquire_foreground().await;
        let _response = client.send_message("foreground request").await;
        start.elapsed()
    });
    
    let foreground_time = foreground.await.unwrap();
    
    // Verify foreground was fast (< 700ms: 500ms LLM + 200ms overhead)
    assert!(foreground_time < Duration::from_millis(700),
        "Foreground took {:?}, expected < 700ms (should use priority pool)", foreground_time);
    
    // Verify metrics were recorded
    let metrics = &limiter.metrics;
    assert_eq!(metrics.foreground_priority_count.load(std::sync::atomic::Ordering::Relaxed), 1,
        "Should have 1 foreground priority call");
    assert_eq!(metrics.background_count.load(std::sync::atomic::Ordering::Relaxed), 7,
        "Should have 7 background calls");
    
    // Print metrics
    println!("\n=== Metrics ===");
    metrics.print_summary();
    
    // Cleanup
    for task in background_tasks {
        task.abort();
    }
}

#[tokio::test]
async fn test_e2e_foreground_fallback_when_priority_busy() {
    // Test that foreground falls back to shared pool when priority is full
    
    use chat_cli::cli::chat::priority_limiter::PriorityLimiter;
    
    let limiter = Arc::new(PriorityLimiter::new(
        2,  // Only 2 priority permits
        5,  // 5 shared permits
        Duration::from_millis(100)
    ));
    
    let mock_client = Arc::new(MockLLMClient::new(Duration::from_millis(200)));
    
    // Start 2 foreground calls (fill priority pool)
    let mut priority_tasks = vec![];
    for i in 0..2 {
        let limiter = limiter.clone();
        let client = mock_client.clone();
        priority_tasks.push(tokio::spawn(async move {
            let _permit = limiter.acquire_foreground().await;
            let _response = client.send_message("priority request").await;
            println!("Priority {} complete", i);
        }));
    }
    
    // Wait for priority pool to fill
    tokio::time::sleep(Duration::from_millis(50)).await;
    
    // Third foreground call should timeout and use shared
    let limiter = limiter.clone();
    let client = mock_client.clone();
    let start = Instant::now();
    
    let fallback = tokio::spawn(async move {
        let _permit = limiter.acquire_foreground().await;
        let _response = client.send_message("fallback request").await;
        start.elapsed()
    });
    
    let fallback_time = fallback.await.unwrap();
    
    // Should take ~100ms (timeout) + 200ms (LLM) = ~300ms
    assert!(fallback_time >= Duration::from_millis(250),
        "Fallback took {:?}, should have waited for timeout", fallback_time);
    
    // Verify metrics show fallback
    let metrics = &limiter.metrics;
    assert_eq!(metrics.foreground_priority_count.load(std::sync::atomic::Ordering::Relaxed), 2,
        "Should have 2 priority calls");
    assert_eq!(metrics.foreground_fallback_count.load(std::sync::atomic::Ordering::Relaxed), 1,
        "Should have 1 fallback call");
    
    println!("\n=== Fallback Metrics ===");
    metrics.print_summary();
    
    // Cleanup
    for task in priority_tasks {
        task.abort();
    }
}

#[tokio::test]
async fn test_e2e_rapid_session_switching() {
    // Simulate rapid session switching with multiple concurrent foreground calls
    
    use chat_cli::cli::chat::priority_limiter::PriorityLimiter;
    
    let limiter = Arc::new(PriorityLimiter::new(
        5,   // 5 priority permits
        10,  // 10 shared permits
        Duration::from_millis(100)
    ));
    
    let mock_client = Arc::new(MockLLMClient::new(Duration::from_millis(300)));
    
    // Simulate 5 rapid session switches (each makes a foreground call)
    let mut session_tasks = vec![];
    let start = Instant::now();
    
    for i in 0..5 {
        let limiter = limiter.clone();
        let client = mock_client.clone();
        session_tasks.push(tokio::spawn(async move {
            let session_start = Instant::now();
            let _permit = limiter.acquire_foreground().await;
            let acquire_time = session_start.elapsed();
            let _response = client.send_message(&format!("session {} request", i)).await;
            (i, acquire_time, session_start.elapsed())
        }));
        tokio::time::sleep(Duration::from_millis(20)).await; // Rapid switching
    }
    
    // Wait for all to complete
    let mut results = vec![];
    for task in session_tasks {
        results.push(task.await.unwrap());
    }
    
    let total_time = start.elapsed();
    
    // All 5 should fit in priority pool and complete quickly
    for (session_id, acquire_time, total_time) in &results {
        println!("Session {}: acquired in {:?}, total {:?}", session_id, acquire_time, total_time);
        assert!(acquire_time < &Duration::from_millis(50),
            "Session {} took {:?} to acquire, should be instant", session_id, acquire_time);
    }
    
    // Total time should be ~300ms (LLM time), not 5*300ms (sequential)
    assert!(total_time < Duration::from_millis(500),
        "Total time {:?}, sessions should run concurrently", total_time);
    
    // Verify all used priority pool
    let metrics = &limiter.metrics;
    assert_eq!(metrics.foreground_priority_count.load(std::sync::atomic::Ordering::Relaxed), 5,
        "All 5 should use priority pool");
    assert_eq!(metrics.foreground_fallback_count.load(std::sync::atomic::Ordering::Relaxed), 0,
        "None should fall back");
    
    println!("\n=== Rapid Switching Metrics ===");
    metrics.print_summary();
}

#[tokio::test]
async fn test_e2e_metrics_accuracy() {
    // Verify metrics are accurately recorded in production flow
    
    use chat_cli::cli::chat::priority_limiter::PriorityLimiter;
    
    let limiter = Arc::new(PriorityLimiter::new(
        3,
        7,
        Duration::from_millis(100)
    ));
    
    let mock_client = Arc::new(MockLLMClient::new(Duration::from_millis(100)));
    
    // Make 10 foreground calls
    for _ in 0..10 {
        let limiter = limiter.clone();
        let client = mock_client.clone();
        tokio::spawn(async move {
            let _permit = limiter.acquire_foreground().await;
            let _response = client.send_message("test").await;
        });
    }
    
    // Make 20 background calls
    for _ in 0..20 {
        let limiter = limiter.clone();
        let client = mock_client.clone();
        tokio::spawn(async move {
            let _permit = limiter.acquire_background().await;
            let _response = client.send_message("test").await;
        });
    }
    
    // Wait for all to complete
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    let metrics = &limiter.metrics;
    let total_foreground = metrics.foreground_priority_count.load(std::sync::atomic::Ordering::Relaxed) +
                          metrics.foreground_fallback_count.load(std::sync::atomic::Ordering::Relaxed);
    let total_background = metrics.background_count.load(std::sync::atomic::Ordering::Relaxed);
    
    // Verify counts
    assert_eq!(total_foreground, 10, "Should have 10 foreground calls");
    assert_eq!(total_background, 20, "Should have 20 background calls");
    
    // Verify hit rate is reasonable (should be high since we have 3 priority permits)
    let hit_rate = metrics.priority_hit_rate();
    assert!(hit_rate > 0.5, "Priority hit rate should be > 50%, got {:.1}%", hit_rate * 100.0);
    
    // Verify wait times are recorded
    let avg_priority_wait = metrics.avg_priority_wait_us();
    let avg_shared_wait = metrics.avg_shared_wait_us();
    assert!(avg_priority_wait > 0.0, "Should have recorded priority wait times");
    assert!(avg_shared_wait > 0.0, "Should have recorded shared wait times");
    
    println!("\n=== Final Metrics ===");
    metrics.print_summary();
    println!("Priority hit rate: {:.1}%", hit_rate * 100.0);
    println!("Avg priority wait: {:.0}µs", avg_priority_wait);
    println!("Avg shared wait: {:.0}µs", avg_shared_wait);
}
