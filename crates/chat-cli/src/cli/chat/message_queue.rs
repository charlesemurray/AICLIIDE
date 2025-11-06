//! Message queue for prioritizing LLM requests

use std::collections::VecDeque;

/// Priority level for queued messages
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MessagePriority {
    High = 1,  // Active session
    Low = 2,   // Background session
}

/// A message queued for LLM processing
#[derive(Debug, Clone)]
pub struct QueuedMessage {
    pub session_id: String,
    pub message: String,
    pub priority: MessagePriority,
    pub timestamp: std::time::Instant,
}

/// Priority-based message queue
pub struct MessageQueue {
    high_priority: VecDeque<QueuedMessage>,
    low_priority: VecDeque<QueuedMessage>,
    current_processing: Option<QueuedMessage>,
}

impl MessageQueue {
    pub fn new() -> Self {
        Self {
            high_priority: VecDeque::new(),
            low_priority: VecDeque::new(),
            current_processing: None,
        }
    }
    
    /// Add message to queue
    pub fn enqueue(&mut self, message: QueuedMessage) {
        match message.priority {
            MessagePriority::High => {
                self.high_priority.push_back(message);
            }
            MessagePriority::Low => {
                self.low_priority.push_back(message);
            }
        }
    }
    
    /// Get next message to process (high priority first)
    pub fn dequeue(&mut self) -> Option<QueuedMessage> {
        if let Some(msg) = self.high_priority.pop_front() {
            self.current_processing = Some(msg.clone());
            Some(msg)
        } else if let Some(msg) = self.low_priority.pop_front() {
            self.current_processing = Some(msg.clone());
            Some(msg)
        } else {
            None
        }
    }
    
    /// Check if should interrupt current processing
    pub fn should_interrupt(&self) -> bool {
        // Interrupt if high priority message arrives while processing low priority
        if let Some(current) = &self.current_processing {
            if current.priority == MessagePriority::Low && !self.high_priority.is_empty() {
                return true;
            }
        }
        false
    }
    
    /// Mark current message as complete
    pub fn complete_current(&mut self) {
        self.current_processing = None;
    }
    
    /// Get current processing message
    pub fn current(&self) -> Option<&QueuedMessage> {
        self.current_processing.as_ref()
    }
    
    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.high_priority.is_empty() && self.low_priority.is_empty()
    }
    
    /// Get queue statistics
    pub fn stats(&self) -> QueueStats {
        QueueStats {
            high_priority_count: self.high_priority.len(),
            low_priority_count: self.low_priority.len(),
            is_processing: self.current_processing.is_some(),
        }
    }
}

impl Default for MessageQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// Queue statistics
#[derive(Debug, Clone)]
pub struct QueueStats {
    pub high_priority_count: usize,
    pub low_priority_count: usize,
    pub is_processing: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_ordering() {
        let mut queue = MessageQueue::new();
        
        // Add low priority first
        queue.enqueue(QueuedMessage {
            session_id: "low".to_string(),
            message: "low priority".to_string(),
            priority: MessagePriority::Low,
            timestamp: std::time::Instant::now(),
        });
        
        // Add high priority
        queue.enqueue(QueuedMessage {
            session_id: "high".to_string(),
            message: "high priority".to_string(),
            priority: MessagePriority::High,
            timestamp: std::time::Instant::now(),
        });
        
        // High priority should come first
        let first = queue.dequeue().unwrap();
        assert_eq!(first.session_id, "high");
        assert_eq!(first.priority, MessagePriority::High);
        
        let second = queue.dequeue().unwrap();
        assert_eq!(second.session_id, "low");
        assert_eq!(second.priority, MessagePriority::Low);
    }
    
    #[test]
    fn test_should_interrupt() {
        let mut queue = MessageQueue::new();
        
        // Start processing low priority
        queue.enqueue(QueuedMessage {
            session_id: "low".to_string(),
            message: "low".to_string(),
            priority: MessagePriority::Low,
            timestamp: std::time::Instant::now(),
        });
        queue.dequeue();
        
        // Should not interrupt yet
        assert!(!queue.should_interrupt());
        
        // Add high priority
        queue.enqueue(QueuedMessage {
            session_id: "high".to_string(),
            message: "high".to_string(),
            priority: MessagePriority::High,
            timestamp: std::time::Instant::now(),
        });
        
        // Should interrupt now
        assert!(queue.should_interrupt());
    }
    
    #[test]
    fn test_stats() {
        let mut queue = MessageQueue::new();
        
        queue.enqueue(QueuedMessage {
            session_id: "1".to_string(),
            message: "msg1".to_string(),
            priority: MessagePriority::High,
            timestamp: std::time::Instant::now(),
        });
        
        queue.enqueue(QueuedMessage {
            session_id: "2".to_string(),
            message: "msg2".to_string(),
            priority: MessagePriority::Low,
            timestamp: std::time::Instant::now(),
        });
        
        let stats = queue.stats();
        assert_eq!(stats.high_priority_count, 1);
        assert_eq!(stats.low_priority_count, 1);
        assert!(!stats.is_processing);
        
        queue.dequeue();
        let stats = queue.stats();
        assert!(stats.is_processing);
    }
}
