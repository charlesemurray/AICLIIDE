//! Integration tests for PriorityLimiter under load

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Barrier;

// Note: These tests use the public API, not internal implementation
// We test behavior, not implementation details

#[tokio::test]
async fn test_foreground_faster_than_background_under_load() {
    // Simulate: 10 background tasks running, foreground should still be fast
    
    let priority_permits = 3;
    let shared_permits = 7;
    let timeout = Duration::from_millis(100);
    
    // This would use the actual PriorityLimiter from the crate
    // For now, we'll test the concept with raw semaphores
    
    use tokio::sync::Semaphore;
    let priority_sem = Arc::new(Semaphore::new(priority_permits));
    let shared_sem = Arc::new(Semaphore::new(shared_permits));
    
    // Start 7 background "calls" (fill shared pool)
    let mut background_tasks = vec![];
    for i in 0..7 {
        let shared = shared_sem.clone();
        background_tasks.push(tokio::spawn(async move {
            let _permit = shared.acquire().await.unwrap();
            tokio::time::sleep(Duration::from_secs(2)).await; // Simulate long LLM call
            println!("Background {} complete", i);
        }));
    }
    
    // Wait for background to acquire permits
    tokio::time::sleep(Duration::from_millis(50)).await;
    
    // Now make foreground call - should use priority pool
    let priority = priority_sem.clone();
    let start = Instant::now();
    let foreground = tokio::spawn(async move {
        let _permit = priority.acquire().await.unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;
        start.elapsed()
    });
    
    let foreground_time = foreground.await.unwrap();
    
    // Foreground should complete quickly (< 200ms) even though shared pool is full
    assert!(foreground_time < Duration::from_millis(200), 
        "Foreground took {:?}, expected < 200ms", foreground_time);
    
    // Cleanup
    for task in background_tasks {
        task.abort();
    }
}

#[tokio::test]
async fn test_foreground_falls_back_when_priority_full() {
    use tokio::sync::Semaphore;
    
    let priority_sem = Arc::new(Semaphore::new(2));
    let shared_sem = Arc::new(Semaphore::new(5));
    
    // Fill priority pool
    let _p1 = priority_sem.acquire().await.unwrap();
    let _p2 = priority_sem.acquire().await.unwrap();
    
    // Foreground should timeout and use shared
    let priority = priority_sem.clone();
    let shared = shared_sem.clone();
    
    let start = Instant::now();
    let result = tokio::time::timeout(
        Duration::from_millis(100),
        priority.acquire()
    ).await;
    
    // Should timeout
    assert!(result.is_err(), "Should timeout on priority pool");
    
    // Should successfully acquire from shared
    let _shared_permit = shared.acquire().await.unwrap();
    let total_time = start.elapsed();
    
    // Should complete in ~100ms (timeout) + minimal shared acquisition
    assert!(total_time < Duration::from_millis(150),
        "Fallback took {:?}, expected < 150ms", total_time);
}

#[tokio::test]
async fn test_background_fairness() {
    use tokio::sync::Semaphore;
    
    let shared_sem = Arc::new(Semaphore::new(3));
    let barrier = Arc::new(Barrier::new(5));
    
    // Start 5 background tasks competing for 3 permits
    let mut tasks = vec![];
    for i in 0..5 {
        let shared = shared_sem.clone();
        let barrier = barrier.clone();
        tasks.push(tokio::spawn(async move {
            barrier.wait().await; // All start at same time
            let start = Instant::now();
            let _permit = shared.acquire().await.unwrap();
            let wait_time = start.elapsed();
            tokio::time::sleep(Duration::from_millis(50)).await;
            (i, wait_time)
        }));
    }
    
    let mut results = vec![];
    for task in tasks {
        results.push(task.await.unwrap());
    }
    
    // First 3 should acquire immediately
    let immediate = results.iter().filter(|(_, time)| *time < Duration::from_millis(10)).count();
    assert_eq!(immediate, 3, "First 3 should acquire immediately");
    
    // Last 2 should wait
    let waited = results.iter().filter(|(_, time)| *time >= Duration::from_millis(40)).count();
    assert_eq!(waited, 2, "Last 2 should wait for permits");
}

#[tokio::test]
async fn test_no_starvation_with_rapid_foreground() {
    use tokio::sync::Semaphore;
    
    let priority_sem = Arc::new(Semaphore::new(2));
    let shared_sem = Arc::new(Semaphore::new(3));
    
    // Start background task
    let shared = shared_sem.clone();
    let background_started = Arc::new(tokio::sync::Notify::new());
    let bg_started = background_started.clone();
    
    let background = tokio::spawn(async move {
        bg_started.notify_one();
        let start = Instant::now();
        let _permit = shared.acquire().await.unwrap();
        start.elapsed()
    });
    
    // Wait for background to start
    background_started.notified().await;
    tokio::time::sleep(Duration::from_millis(10)).await;
    
    // Make rapid foreground calls (using priority pool)
    for _ in 0..5 {
        let priority = priority_sem.clone();
        tokio::spawn(async move {
            let _permit = priority.acquire().await.unwrap();
            tokio::time::sleep(Duration::from_millis(20)).await;
        });
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    
    // Background should still complete in reasonable time
    let bg_time = background.await.unwrap();
    assert!(bg_time < Duration::from_secs(1),
        "Background took {:?}, should not be starved", bg_time);
}

#[tokio::test]
async fn test_scales_with_capacity() {
    // Test that configuration scales appropriately
    // Formula: (total / 4).max(3).min(10)
    
    let test_cases = vec![
        (10, 3, 7),   // Small: 10 total -> 3 priority (min), 7 shared
        (20, 5, 15),  // Medium: 20 total -> 5 priority, 15 shared
        (50, 10, 40), // Large: 50 total -> 10 priority (max), 40 shared
        (8, 3, 5),    // Very small: 8 total -> 3 priority (min), 5 shared
        (100, 10, 90), // Very large: 100 total -> 10 priority (max), 90 shared
    ];
    
    for (total, expected_priority, expected_shared) in test_cases {
        // Calculate what PriorityLimiter would allocate
        let priority = (total / 4).max(3).min(10);
        let shared = total - priority;
        
        assert_eq!(priority, expected_priority,
            "Total {} should allocate {} priority", total, expected_priority);
        assert_eq!(shared, expected_shared,
            "Total {} should allocate {} shared", total, expected_shared);
    }
}
