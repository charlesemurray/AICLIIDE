//! Queue manager for processing LLM messages with Tower-based rate limiting

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use eyre::Result;

use super::message_queue::{MessageQueue, QueuedMessage, MessagePriority};
use super::llm_tower::LLMTower;
use crate::api_client::ApiClient;

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

/// Manages message queue and LLM processing with Tower
pub struct QueueManager {
    queue: Arc<Mutex<MessageQueue>>,
    response_channels: Arc<Mutex<HashMap<String, mpsc::UnboundedSender<LLMResponse>>>>,
    llm_tower: Option<Arc<Mutex<LLMTower>>>,
    num_workers: usize,
}

impl QueueManager {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(MessageQueue::new())),
            response_channels: Arc::new(Mutex::new(HashMap::new())),
            llm_tower: None,
            num_workers: 3,
        }
    }
    
    /// Create with shared Tower instance (MUST be same instance as coordinator uses)
    pub fn with_shared_tower(tower: Arc<Mutex<LLMTower>>, num_workers: usize) -> Self {
        Self {
            queue: Arc::new(Mutex::new(MessageQueue::new())),
            response_channels: Arc::new(Mutex::new(HashMap::new())),
            llm_tower: Some(tower),
            num_workers,
        }
    }
    
    /// Start background workers to process queued messages
    pub fn start_background_worker(self: Arc<Self>) {
        eprintln!("[WORKER] Starting {} background worker threads with Tower rate limiting", 
            self.num_workers);
        
        // Spawn multiple workers
        for worker_id in 0..self.num_workers {
            let self_clone = Arc::clone(&self);
            
            tokio::spawn(async move {
                eprintln!("[WORKER-{}] Started", worker_id);
                let mut iteration = 0;
                
                loop {
                    iteration += 1;
                    
                    // Check for messages to process
                    let msg = {
                        let mut queue = self_clone.queue.lock().await;
                        if iteration % 100 == 0 {
                            let stats = queue.stats();
                            eprintln!("[WORKER-{}] Iteration {} - Queue stats: high={}, low={}", 
                                worker_id, iteration, stats.high_priority_count, stats.low_priority_count);
                        }
                        queue.dequeue()
                    };
                    
                    if let Some(queued_msg) = msg {
                        let elapsed = queued_msg.timestamp.elapsed();
                        eprintln!("[WORKER-{}] Processing message from session {} (waited: {:?}, priority: {:?})", 
                            worker_id, queued_msg.session_id, elapsed, queued_msg.priority);
                        
                        // Get response channel
                        let tx = {
                            let channels = self_clone.response_channels.lock().await;
                            channels.get(&queued_msg.session_id).cloned()
                        };
                        
                        if let Some(tx) = tx {
                            // Send processing indicator
                            let _ = tx.send(LLMResponse::Chunk("Processing your request in background...\n\n".to_string()));
                            
                            // Check for interruption
                            if self_clone.should_interrupt().await {
                                eprintln!("[WORKER-{}] Interrupted for higher priority (session: {})", 
                                    worker_id, queued_msg.session_id);
                                let _ = tx.send(LLMResponse::Interrupted);
                                continue;
                            }
                            
                            // Process with Tower (handles rate limiting automatically)
                            self_clone.process_message(worker_id, queued_msg, tx).await;
                        }
                    } else {
                        // No messages, sleep briefly
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    }
                }
            });
        }
    }
    
    
    /// Process a single message using Tower
    async fn process_message(
        &self,
        worker_id: usize,
        queued_msg: QueuedMessage,
        tx: mpsc::UnboundedSender<LLMResponse>,
    ) {
        // Use Tower if available
        if let Some(ref tower) = self.llm_tower {
            eprintln!("[WORKER-{}] Using Tower LLM service for session {}", worker_id, queued_msg.session_id);
            
            // Create conversation state
            use crate::api_client::model::{ConversationState, UserInputMessage};
            let conv_state = ConversationState {
                conversation_id: Some(queued_msg.session_id.clone()),
                user_input_message: UserInputMessage {
                    content: queued_msg.message.clone(),
                    user_input_message_context: None,
                    user_intent: None,
                    images: None,
                    model_id: None,
                },
                history: None,
            };
            
            // Call Tower service (handles rate limiting automatically)
            let mut tower_guard = tower.lock().await;
            match tower_guard.call_low_priority(conv_state).await {
                Ok(mut stream) => {
                    eprintln!("[WORKER-{}] Tower LLM streaming started for session {}", worker_id, queued_msg.session_id);
                    let mut chunk_count = 0;
                    
                    // Stream responses
                    loop {
                        // Check for interruption
                        if self.should_interrupt().await {
                            eprintln!("[WORKER-{}] Interrupted during streaming (session: {})", worker_id, queued_msg.session_id);
                            let _ = tx.send(LLMResponse::Interrupted);
                            break;
                        }
                        
                        match stream.recv().await {
                            Some(Ok(event)) => {
                                use crate::cli::chat::parser::ResponseEvent;
                                match event {
                                    ResponseEvent::AssistantText(text) => {
                                        if tx.send(LLMResponse::Chunk(text)).is_err() {
                                            break;
                                        }
                                        chunk_count += 1;
                                    },
                                    ResponseEvent::ToolUseStart { name } => {
                                        eprintln!("[WORKER-{}] Tool use starting: {}", worker_id, name);
                                    },
                                    ResponseEvent::ToolUse(tool_use) => {
                                        if tx.send(LLMResponse::ToolUse { 
                                            id: tool_use.id, 
                                            name: tool_use.name, 
                                            params: tool_use.args 
                                        }).is_err() {
                                            break;
                                        }
                                    },
                                    ResponseEvent::EndStream { .. } => {
                                        break;
                                    },
                                }
                            },
                            Some(Err(e)) => {
                                eprintln!("[WORKER-{}] Stream error: {}", worker_id, e);
                                let _ = tx.send(LLMResponse::Error(format!("Stream error: {}", e)));
                                break;
                            },
                            None => {
                                break;
                            }
                        }
                    }
                    
                    let _ = tx.send(LLMResponse::Complete);
                    eprintln!("[WORKER-{}] Completed Tower LLM processing for session {} (sent {} chunks)", 
                        worker_id, queued_msg.session_id, chunk_count);
                    return;
                },
                Err(e) => {
                    eprintln!("[WORKER-{}] Tower LLM call failed for session {}: {}", worker_id, queued_msg.session_id, e);
                    let _ = tx.send(LLMResponse::Error(format!("LLM API error: {}", e)));
                    return;
                }
            }
        }
        
        // Fallback to simulation
        eprintln!("[WORKER-{}] No Tower service, using simulation for session {}", worker_id, queued_msg.session_id);
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                        
        // Generate response based on message
        let response = format!(
            "I've processed your message in the background:\n\n\
            > {}\n\n\
            This is a simulated response. In production, this would be \
            the actual LLM response from the API.\n\n\
            The background processing system is working correctly!",
            queued_msg.message
        );
        
        eprintln!("[WORKER-{}] Sending response to session {} ({} bytes)", 
            worker_id, queued_msg.session_id, response.len());
        
        // Send response chunks (simulate streaming)
        let mut chunk_count = 0;
        for chunk in response.split('\n') {
            if tx.send(LLMResponse::Chunk(format!("{}\n", chunk))).is_err() {
                eprintln!("[WORKER-{}] ERROR: Failed to send chunk {} to session {}", 
                    worker_id, chunk_count, queued_msg.session_id);
                break;
            }
            chunk_count += 1;
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }
        
        // Send completion
        if tx.send(LLMResponse::Complete).is_err() {
            eprintln!("[WORKER-{}] ERROR: Failed to send completion to session {}", worker_id, queued_msg.session_id);
        } else {
            eprintln!("[WORKER-{}] Completed processing for session {} (sent {} chunks)", 
                worker_id, queued_msg.session_id, chunk_count);
        }
    }
    
    /// Submit a message to the queue and get response channel
    pub async fn submit_message(
        &self,
        session_id: String,
        message: String,
        priority: MessagePriority,
    ) -> mpsc::UnboundedReceiver<LLMResponse> {
        let (tx, rx) = mpsc::unbounded_channel();
        
        eprintln!("[QUEUE] Submitting message from session {} (priority: {:?}, msg_len: {})", 
            session_id, priority, message.len());
        
        // Register response channel
        {
            let mut channels = self.response_channels.lock().await;
            channels.insert(session_id.clone(), tx);
            eprintln!("[QUEUE] Registered response channel for session {} (total channels: {})", 
                session_id, channels.len());
        }
        
        // Enqueue message
        {
            let mut queue = self.queue.lock().await;
            queue.enqueue(QueuedMessage {
                session_id: session_id.clone(),
                message,
                priority,
                timestamp: std::time::Instant::now(),
            });
            let stats = queue.stats();
            eprintln!("[QUEUE] Enqueued message for session {} (queue size: high={}, low={})", 
                session_id, stats.high_priority_count, stats.low_priority_count);
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
