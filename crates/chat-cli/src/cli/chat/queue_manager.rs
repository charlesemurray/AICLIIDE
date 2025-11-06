//! Queue manager for processing LLM messages with priority

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use eyre::Result;

use super::message_queue::{MessageQueue, QueuedMessage, MessagePriority};

/// Callback for processing messages
pub type ProcessCallback = Arc<dyn Fn(String) -> String + Send + Sync>;

/// Response from LLM processing
#[derive(Debug, Clone)]
pub enum LLMResponse {
    /// Text chunk from assistant
    Chunk(String),
    /// Tool use request
    ToolUse {
        id: String,
        name: String,
        params: serde_json::Value,
    },
    /// Tool result
    ToolResult {
        id: String,
        result: String,
    },
    /// Processing complete
    Complete,
    /// Error occurred
    Error(String),
    /// Processing interrupted for higher priority
    Interrupted,
}

/// Manages message queue and LLM processing
pub struct QueueManager {
    queue: Arc<Mutex<MessageQueue>>,
    response_channels: Arc<Mutex<HashMap<String, mpsc::UnboundedSender<LLMResponse>>>>,
}

impl QueueManager {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(MessageQueue::new())),
            response_channels: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Start background worker to process queued messages
    pub fn start_background_worker(self: Arc<Self>) {
        tokio::spawn(async move {
            loop {
                // Check for messages to process
                let msg = {
                    let mut queue = self.queue.lock().await;
                    queue.dequeue()
                };
                
                if let Some(queued_msg) = msg {
                    eprintln!("[WORKER] Processing message from session {}", queued_msg.session_id);
                    
                    // Get response channel
                    let tx = {
                        let channels = self.response_channels.lock().await;
                        channels.get(&queued_msg.session_id).cloned()
                    };
                    
                    if let Some(tx) = tx {
                        // Send processing indicator
                        let _ = tx.send(LLMResponse::Chunk("Processing in background...".to_string()));
                        
                        // Check for interruption
                        if self.should_interrupt().await {
                            eprintln!("[WORKER] Interrupted for higher priority");
                            let _ = tx.send(LLMResponse::Interrupted);
                            continue;
                        }
                        
                        // Simulate work (in real impl, this would call LLM API)
                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                        
                        // Send completion
                        let _ = tx.send(LLMResponse::Chunk(format!(
                            "Background processing complete for: {}",
                            queued_msg.message
                        )));
                        let _ = tx.send(LLMResponse::Complete);
                        
                        eprintln!("[WORKER] Completed processing for session {}", queued_msg.session_id);
                    }
                } else {
                    // No messages, sleep briefly
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            }
        });
    }
    
    /// Submit a message to the queue and get response channel
    pub async fn submit_message(
        &self,
        session_id: String,
        message: String,
        priority: MessagePriority,
    ) -> mpsc::UnboundedReceiver<LLMResponse> {
        let (tx, rx) = mpsc::unbounded_channel();
        
        eprintln!("[QUEUE] Submitting message from session {} (priority: {:?})", session_id, priority);
        
        // Register response channel
        {
            let mut channels = self.response_channels.lock().await;
            channels.insert(session_id.clone(), tx);
        }
        
        // Enqueue message
        {
            let mut queue = self.queue.lock().await;
            queue.enqueue(QueuedMessage {
                session_id,
                message,
                priority,
                timestamp: std::time::Instant::now(),
            });
        }
        
        rx
    }
    
    /// Check if should interrupt current processing
    pub async fn should_interrupt(&self) -> bool {
        let queue = self.queue.lock().await;
        queue.should_interrupt()
    }
    
    /// Get next message from queue
    pub async fn dequeue(&self) -> Option<QueuedMessage> {
        let mut queue = self.queue.lock().await;
        queue.dequeue()
    }
    
    /// Mark current message as complete
    pub async fn complete_current(&self) {
        let mut queue = self.queue.lock().await;
        queue.complete_current();
    }
    
    /// Send response to session
    pub async fn send_response(&self, session_id: &str, response: LLMResponse) -> Result<()> {
        let channels = self.response_channels.lock().await;
        if let Some(tx) = channels.get(session_id) {
            let _ = tx.send(response);
        }
        Ok(())
    }
    
    /// Get queue statistics
    pub async fn stats(&self) -> super::message_queue::QueueStats {
        let queue = self.queue.lock().await;
        queue.stats()
    }
    
    /// Remove response channel for session
    pub async fn remove_channel(&self, session_id: &str) {
        let mut channels = self.response_channels.lock().await;
        channels.remove(session_id);
    }
}

impl Default for QueueManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_submit_and_dequeue() {
        let manager = QueueManager::new();
        
        // Submit message
        let mut rx = manager.submit_message(
            "session1".to_string(),
            "test message".to_string(),
            MessagePriority::High,
        ).await;
        
        // Should be able to dequeue
        let msg = manager.dequeue().await;
        assert!(msg.is_some());
        let msg = msg.unwrap();
        assert_eq!(msg.session_id, "session1");
        assert_eq!(msg.message, "test message");
        assert_eq!(msg.priority, MessagePriority::High);
        
        // Send response
        manager.send_response("session1", LLMResponse::Chunk("test".to_string())).await.unwrap();
        
        // Should receive response
        let response = rx.recv().await;
        assert!(response.is_some());
        matches!(response.unwrap(), LLMResponse::Chunk(_));
    }
    
    #[tokio::test]
    async fn test_priority_ordering() {
        let manager = QueueManager::new();
        
        // Submit low priority
        let _rx1 = manager.submit_message(
            "session1".to_string(),
            "low".to_string(),
            MessagePriority::Low,
        ).await;
        
        // Submit high priority
        let _rx2 = manager.submit_message(
            "session2".to_string(),
            "high".to_string(),
            MessagePriority::High,
        ).await;
        
        // High priority should come first
        let msg = manager.dequeue().await.unwrap();
        assert_eq!(msg.session_id, "session2");
        assert_eq!(msg.priority, MessagePriority::High);
        
        let msg = manager.dequeue().await.unwrap();
        assert_eq!(msg.session_id, "session1");
        assert_eq!(msg.priority, MessagePriority::Low);
    }
    
    #[tokio::test]
    async fn test_interruption_detection() {
        let manager = QueueManager::new();
        
        // Submit and start processing low priority
        let _rx1 = manager.submit_message(
            "session1".to_string(),
            "low".to_string(),
            MessagePriority::Low,
        ).await;
        
        manager.dequeue().await;
        
        // Should not interrupt yet
        assert!(!manager.should_interrupt().await);
        
        // Submit high priority
        let _rx2 = manager.submit_message(
            "session2".to_string(),
            "high".to_string(),
            MessagePriority::High,
        ).await;
        
        // Should interrupt now
        assert!(manager.should_interrupt().await);
    }
    
    #[tokio::test]
    async fn test_background_worker() {
        let manager = Arc::new(QueueManager::new());
        
        // Start worker
        manager.clone().start_background_worker();
        
        // Submit message
        let mut rx = manager.submit_message(
            "session1".to_string(),
            "test".to_string(),
            MessagePriority::High,
        ).await;
        
        // Should receive responses from worker
        let response = tokio::time::timeout(
            tokio::time::Duration::from_secs(1),
            rx.recv()
        ).await;
        
        assert!(response.is_ok());
        assert!(response.unwrap().is_some());
    }
    
    #[tokio::test]
    async fn test_stats() {
        let manager = QueueManager::new();
        
        manager.submit_message(
            "session1".to_string(),
            "msg1".to_string(),
            MessagePriority::High,
        ).await;
        
        manager.submit_message(
            "session2".to_string(),
            "msg2".to_string(),
            MessagePriority::Low,
        ).await;
        
        let stats = manager.stats().await;
        assert_eq!(stats.high_priority_count, 1);
        assert_eq!(stats.low_priority_count, 1);
        assert!(!stats.is_processing);
        
        manager.dequeue().await;
        let stats = manager.stats().await;
        assert!(stats.is_processing);
    }
}
