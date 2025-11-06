/// Test response storage and retrieval

use std::sync::Arc;

#[tokio::test]
async fn test_worker_generates_realistic_response() {
    use chat_cli::cli::chat::coordinator::{MultiSessionCoordinator, CoordinatorConfig};
    use chat_cli::cli::chat::message_queue::MessagePriority;
    
    let coord = Arc::new(MultiSessionCoordinator::new(CoordinatorConfig::default()));
    
    // Submit message
    let mut rx = coord.queue_manager.submit_message(
        "test-session".to_string(),
        "What is 2+2?".to_string(),
        MessagePriority::Low,
    ).await;
    
    // Collect all responses
    let mut all_text = String::new();
    while let Ok(Some(response)) = tokio::time::timeout(
        tokio::time::Duration::from_secs(3),
        rx.recv()
    ).await {
        use chat_cli::cli::chat::queue_manager::LLMResponse;
        match response {
            LLMResponse::Chunk(text) => all_text.push_str(&text),
            LLMResponse::Complete => break,
            _ => {}
        }
    }
    
    // Should contain the original message
    assert!(all_text.contains("What is 2+2?"), "Response should quote original message");
    
    // Should contain processing indicator
    assert!(all_text.contains("background"), "Response should mention background processing");
    
    // Should be substantial
    assert!(all_text.len() > 100, "Response should be substantial");
}

#[tokio::test]
async fn test_response_streaming() {
    use chat_cli::cli::chat::coordinator::{MultiSessionCoordinator, CoordinatorConfig};
    use chat_cli::cli::chat::message_queue::MessagePriority;
    
    let coord = Arc::new(MultiSessionCoordinator::new(CoordinatorConfig::default()));
    
    // Submit message
    let mut rx = coord.queue_manager.submit_message(
        "test-session".to_string(),
        "test".to_string(),
        MessagePriority::Low,
    ).await;
    
    // Should receive multiple chunks (streaming)
    let mut chunk_count = 0;
    while let Ok(Some(response)) = tokio::time::timeout(
        tokio::time::Duration::from_secs(3),
        rx.recv()
    ).await {
        use chat_cli::cli::chat::queue_manager::LLMResponse;
        match response {
            LLMResponse::Chunk(_) => chunk_count += 1,
            LLMResponse::Complete => break,
            _ => {}
        }
    }
    
    // Should have multiple chunks (simulating streaming)
    assert!(chunk_count > 1, "Should receive multiple chunks");
}

#[tokio::test]
async fn test_notification_system() {
    use chat_cli::cli::chat::coordinator::{MultiSessionCoordinator, CoordinatorConfig};
    
    let coord = Arc::new(MultiSessionCoordinator::new(CoordinatorConfig::default()));
    let session_id = "test-session";
    
    // No notification initially
    assert!(!coord.has_notification(session_id).await);
    
    // Post notification
    coord.notify_background_complete(
        session_id.to_string(),
        "Work complete".to_string()
    ).await;
    
    // Should have notification
    assert!(coord.has_notification(session_id).await);
    
    // Retrieve it
    let notif = coord.take_notification(session_id).await;
    assert_eq!(notif, Some("Work complete".to_string()));
    
    // Should be gone
    assert!(!coord.has_notification(session_id).await);
}
