/// Test partial response save/resume functionality
use chat_cli::cli::chat::conversation::ConversationState;

#[test]
fn test_partial_response_save_and_take() {
    let mut conv = ConversationState::new("test-conv-id".to_string());
    
    // Initially no partial response
    assert!(!conv.has_partial_response());
    assert_eq!(conv.take_partial_response(), None);
    
    // Save a partial response
    let partial_text = "This is a partial response from the LLM...".to_string();
    conv.save_partial_response(partial_text.clone());
    
    // Should now have partial response
    assert!(conv.has_partial_response());
    
    // Take should return the saved text and clear it
    assert_eq!(conv.take_partial_response(), Some(partial_text));
    
    // After taking, should be empty again
    assert!(!conv.has_partial_response());
    assert_eq!(conv.take_partial_response(), None);
}

#[test]
fn test_partial_response_overwrite() {
    let mut conv = ConversationState::new("test-conv-id".to_string());
    
    // Save first partial
    conv.save_partial_response("First partial".to_string());
    assert!(conv.has_partial_response());
    
    // Save second partial (should overwrite)
    conv.save_partial_response("Second partial".to_string());
    assert!(conv.has_partial_response());
    
    // Should get the second one
    assert_eq!(conv.take_partial_response(), Some("Second partial".to_string()));
}

#[test]
fn test_partial_response_empty_string() {
    let mut conv = ConversationState::new("test-conv-id".to_string());
    
    // Save empty string
    conv.save_partial_response(String::new());
    
    // Should still be considered as having a partial (even if empty)
    assert!(conv.has_partial_response());
    assert_eq!(conv.take_partial_response(), Some(String::new()));
}
