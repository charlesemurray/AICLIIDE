/// Tests for visual indicators

use chat_cli::cli::chat::status_bar::StatusBar;
use chat_cli::cli::chat::live_indicator::LiveIndicator;

#[test]
fn test_status_bar_updates() {
    let mut bar = StatusBar::new("test".to_string(), 2);
    assert_eq!(bar.notification_count, 0);
    assert!(!bar.background_active);
    
    bar.update(3, true);
    assert_eq!(bar.notification_count, 3);
    assert!(bar.background_active);
}

#[test]
fn test_live_indicator_state_changes() {
    let mut indicator = LiveIndicator::new();
    
    // Initial state
    assert_eq!(indicator.notification_count, 0);
    assert!(!indicator.background_active);
    
    // Update state
    let mut output = Vec::new();
    indicator.update_and_render(2, true, &mut output).unwrap();
    
    assert_eq!(indicator.notification_count, 2);
    assert!(indicator.background_active);
}

#[test]
fn test_indicator_only_renders_on_change() {
    let mut indicator = LiveIndicator::new();
    let mut output = Vec::new();
    
    // First render
    indicator.update_and_render(1, false, &mut output).unwrap();
    let first_len = output.len();
    
    // Same state - should not render
    output.clear();
    indicator.update_and_render(1, false, &mut output).unwrap();
    assert_eq!(output.len(), 0, "Should not render when state unchanged");
    
    // Different state - should render
    indicator.update_and_render(2, false, &mut output).unwrap();
    assert!(output.len() > 0, "Should render when state changes");
}

#[tokio::test]
async fn test_coordinator_notification_count() {
    use chat_cli::cli::chat::coordinator::{MultiSessionCoordinator, CoordinatorConfig};
    
    let coord = MultiSessionCoordinator::new(CoordinatorConfig::default());
    
    // Initially zero
    assert_eq!(coord.notification_count().await, 0);
    
    // Add notifications
    coord.notify_background_complete("session1".to_string(), "msg1".to_string()).await;
    assert_eq!(coord.notification_count().await, 1);
    
    coord.notify_background_complete("session2".to_string(), "msg2".to_string()).await;
    assert_eq!(coord.notification_count().await, 2);
    
    // Take one
    coord.take_notification("session1").await;
    assert_eq!(coord.notification_count().await, 1);
}

#[tokio::test]
async fn test_background_work_detection() {
    use chat_cli::cli::chat::coordinator::{MultiSessionCoordinator, CoordinatorConfig};
    use chat_cli::cli::chat::message_queue::MessagePriority;
    
    let coord = MultiSessionCoordinator::new(CoordinatorConfig::default());
    
    // Initially no work
    assert!(!coord.has_background_work().await);
    
    // Submit message
    let _rx = coord.queue_manager.submit_message(
        "test".to_string(),
        "msg".to_string(),
        MessagePriority::Low,
    ).await;
    
    // Should have work now
    assert!(coord.has_background_work().await);
}
