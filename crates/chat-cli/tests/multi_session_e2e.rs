//! End-to-end test for multi-session feature

use chat_cli::cli::chat::coordinator::{
    CoordinatorConfig,
    MultiSessionCoordinator,
};
use chat_cli::cli::chat::session_integration;
use chat_cli::theme::session::SessionType;

#[tokio::test]
async fn test_multi_session_basic_flow() {
    let mut coordinator = MultiSessionCoordinator::new(CoordinatorConfig::default());
    let mut output = Vec::new();

    // Test /sessions list (should be empty)
    let result = session_integration::handle_session_command("/sessions", &mut coordinator, &mut output).await;
    assert!(result.is_ok());
    assert!(result.unwrap());

    // Test /switch (should fail - no sessions)
    output.clear();
    let result = session_integration::handle_session_command("/switch test", &mut coordinator, &mut output).await;
    assert!(result.is_err()); // Should fail because no sessions exist

    // Test /close (should fail - no sessions)
    output.clear();
    let result = session_integration::handle_session_command("/close test", &mut coordinator, &mut output).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_session_persistence() {
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let mut coordinator = MultiSessionCoordinator::new(CoordinatorConfig::default());

    // Enable persistence
    coordinator.enable_persistence(temp_dir.path().to_path_buf()).unwrap();

    // Load sessions (should be empty)
    let count = coordinator.load_sessions().await.unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
async fn test_coordinator_integration() {
    let mut coordinator = MultiSessionCoordinator::new(CoordinatorConfig::default());

    // Test list sessions
    let sessions = coordinator.list_sessions().await;
    assert_eq!(sessions.len(), 0);

    // Test get_session_info
    let info = coordinator.get_session_info().await;
    assert_eq!(info.len(), 0);
}
