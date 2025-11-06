/// Full flow test: submit -> process -> notify -> retrieve

use std::sync::Arc;

#[tokio::test]
async fn test_full_background_flow() {
    use chat_cli::cli::chat::coordinator::{MultiSessionCoordinator, CoordinatorConfig};
    use chat_cli::cli::chat::message_queue::MessagePriority;
    
    // Create coordinator (starts worker)
    let coord = MultiSessionCoordinator::new(CoordinatorConfig::default());
    let session_id = "test-session".to_string();
    
    // Submit message to background
    let mut rx = coord.queue_manager.submit_message(
        session_id.clone(),
        "test message".to_string(),
        MessagePriority::Low,
    ).await;
    
    // Collect responses
    let mut responses = Vec::new();
    while let Ok(Some(response)) = tokio::time::timeout(
        tokio::time::Duration::from_secs(2),
        rx.recv()
    ).await {
        use chat_cli::cli::chat::queue_manager::LLMResponse;
        match response {
            LLMResponse::Chunk(text) => responses.push(text),
            LLMResponse::Complete => break,
            _ => {}
        }
    }
    
    // Should have received responses
    assert!(!responses.is_empty(), "Should receive responses from worker");
    assert!(responses.iter().any(|r| r.contains("Background processing")), 
            "Should contain background processing message");
}

#[tokio::test]
async fn test_notification_after_completion() {
    use chat_cli::cli::chat::coordinator::{MultiSessionCoordinator, CoordinatorConfig};
    use chat_cli::cli::chat::message_queue::MessagePriority;
    
    let coord = Arc::new(MultiSessionCoordinator::new(CoordinatorConfig::default()));
    let session_id = "test-session".to_string();
    
    // Submit message
    let mut rx = coord.queue_manager.submit_message(
        session_id.clone(),
        "test".to_string(),
        MessagePriority::Low,
    ).await;
    
    // Wait for completion
    while let Ok(Some(response)) = tokio::time::timeout(
        tokio::time::Duration::from_secs(2),
        rx.recv()
    ).await {
        use chat_cli::cli::chat::queue_manager::LLMResponse;
        if matches!(response, LLMResponse::Complete) {
            break;
        }
    }
    
    // Manually post notification (in real impl, worker would do this)
    coord.notify_background_complete(
        session_id.clone(),
        "Work complete".to_string()
    ).await;
    
    // Should have notification
    assert!(coord.has_notification(&session_id).await);
    
    // Retrieve it
    let msg = coord.take_notification(&session_id).await;
    assert_eq!(msg, Some("Work complete".to_string()));
}
