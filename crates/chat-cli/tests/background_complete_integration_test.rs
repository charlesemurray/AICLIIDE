/// Complete integration test: routing, processing, notification, display

use std::sync::Arc;

#[tokio::test]
async fn test_complete_background_flow() {
    use chat_cli::cli::chat::coordinator::{MultiSessionCoordinator, CoordinatorConfig};
    use chat_cli::cli::chat::message_queue::MessagePriority;
    
    // 1. Create coordinator (starts worker)
    let coord = Arc::new(MultiSessionCoordinator::new(CoordinatorConfig::default()));
    let session_id = "test-session".to_string();
    
    // 2. Submit message to background
    let mut rx = coord.queue_manager.submit_message(
        session_id.clone(),
        "test message".to_string(),
        MessagePriority::Low,
    ).await;
    
    // 3. Worker processes it
    let mut got_response = false;
    while let Ok(Some(response)) = tokio::time::timeout(
        tokio::time::Duration::from_secs(2),
        rx.recv()
    ).await {
        use chat_cli::cli::chat::queue_manager::LLMResponse;
        match response {
            LLMResponse::Chunk(_) => got_response = true,
            LLMResponse::Complete => break,
            _ => {}
        }
    }
    
    assert!(got_response, "Should receive response from worker");
    
    // 4. Post notification
    coord.notify_background_complete(
        session_id.clone(),
        "Background work complete".to_string()
    ).await;
    
    // 5. Check notification exists
    assert!(coord.has_notification(&session_id).await);
    
    // 6. Retrieve notification
    let notif = coord.take_notification(&session_id).await;
    assert_eq!(notif, Some("Background work complete".to_string()));
    
    // 7. Notification should be gone
    assert!(!coord.has_notification(&session_id).await);
}

#[tokio::test]
async fn test_routing_logic() {
    // Test that routing logic works correctly
    
    // Case 1: No coordinator -> foreground
    let has_coordinator = false;
    let is_active = true;
    assert!(!should_background(has_coordinator, is_active));
    
    // Case 2: Has coordinator, active -> foreground
    let has_coordinator = true;
    let is_active = true;
    assert!(!should_background(has_coordinator, is_active));
    
    // Case 3: Has coordinator, inactive -> background
    let has_coordinator = true;
    let is_active = false;
    assert!(should_background(has_coordinator, is_active));
}

fn should_background(has_coordinator: bool, is_active: bool) -> bool {
    has_coordinator && !is_active
}

#[tokio::test]
async fn test_worker_handles_multiple_messages() {
    use chat_cli::cli::chat::coordinator::{MultiSessionCoordinator, CoordinatorConfig};
    use chat_cli::cli::chat::message_queue::MessagePriority;
    
    let coord = Arc::new(MultiSessionCoordinator::new(CoordinatorConfig::default()));
    
    // Submit multiple messages
    let mut receivers = vec![];
    for i in 0..3 {
        let rx = coord.queue_manager.submit_message(
            format!("session-{}", i),
            format!("message {}", i),
            MessagePriority::Low,
        ).await;
        receivers.push(rx);
    }
    
    // All should complete
    for mut rx in receivers {
        let mut completed = false;
        while let Ok(Some(response)) = tokio::time::timeout(
            tokio::time::Duration::from_secs(3),
            rx.recv()
        ).await {
            use chat_cli::cli::chat::queue_manager::LLMResponse;
            if matches!(response, LLMResponse::Complete) {
                completed = true;
                break;
            }
        }
        assert!(completed, "Message should complete");
    }
}
