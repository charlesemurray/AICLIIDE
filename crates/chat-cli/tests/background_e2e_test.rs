/// End-to-end test for background processing

use std::sync::Arc;

#[tokio::test]
async fn test_worker_actually_starts() {
    use chat_cli::cli::chat::queue_manager::QueueManager;
    use chat_cli::cli::chat::message_queue::MessagePriority;
    
    // Create queue manager
    let manager = Arc::new(QueueManager::new());
    
    // Start worker
    manager.clone().start_background_worker();
    
    // Submit message
    let mut rx = manager.submit_message(
        "test-session".to_string(),
        "test message".to_string(),
        MessagePriority::High,
    ).await;
    
    // Worker should process it
    let response = tokio::time::timeout(
        tokio::time::Duration::from_secs(2),
        rx.recv()
    ).await;
    
    assert!(response.is_ok(), "Worker should process message");
    assert!(response.unwrap().is_some(), "Should receive response");
}

#[tokio::test]
async fn test_coordinator_starts_worker() {
    use chat_cli::cli::chat::coordinator::{MultiSessionCoordinator, CoordinatorConfig};
    use chat_cli::cli::chat::message_queue::MessagePriority;
    
    // Create coordinator (should start worker)
    let coord = MultiSessionCoordinator::new(CoordinatorConfig::default());
    
    // Submit message through coordinator's queue manager
    let mut rx = coord.queue_manager.submit_message(
        "test-session".to_string(),
        "test".to_string(),
        MessagePriority::High,
    ).await;
    
    // Should receive response from worker
    let response = tokio::time::timeout(
        tokio::time::Duration::from_secs(2),
        rx.recv()
    ).await;
    
    assert!(response.is_ok(), "Coordinator's worker should process message");
}

#[tokio::test]
async fn test_notification_flow() {
    use chat_cli::cli::chat::coordinator::{MultiSessionCoordinator, CoordinatorConfig};
    
    let coord = MultiSessionCoordinator::new(CoordinatorConfig::default());
    let session_id = "test-session".to_string();
    
    // No notification initially
    assert!(!coord.has_notification(&session_id).await);
    
    // Post notification
    coord.notify_background_complete(
        session_id.clone(),
        "Work complete".to_string()
    ).await;
    
    // Should have notification
    assert!(coord.has_notification(&session_id).await);
    
    // Retrieve it
    let msg = coord.take_notification(&session_id).await;
    assert_eq!(msg, Some("Work complete".to_string()));
    
    // Should be gone
    assert!(!coord.has_notification(&session_id).await);
}
