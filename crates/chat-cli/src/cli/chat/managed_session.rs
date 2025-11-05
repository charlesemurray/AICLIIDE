//! Managed session linking display, conversation, and execution state

use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Instant;

use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::cli::chat::ConversationState;
use crate::theme::session::{
    SessionDisplay,
    SessionStatus,
};

/// Metadata for session lifecycle tracking
#[derive(Debug, Clone)]
pub struct SessionMetadata {
    pub created_at: Instant,
    pub last_active: Instant,
    pub message_count: usize,
}

/// Events that can be buffered for background sessions
#[derive(Debug, Clone)]
pub enum OutputEvent {
    Text(String),
    StyledText(String, String), // text, style_description
    ToolStart(String),
    ToolEnd(String, String), // tool_name, result
    Error(String),
}

impl OutputEvent {
    /// Estimate size in bytes for buffer management
    pub fn size_bytes(&self) -> usize {
        match self {
            OutputEvent::Text(s) => s.len(),
            OutputEvent::StyledText(s, style) => s.len() + style.len(),
            OutputEvent::ToolStart(s) => s.len(),
            OutputEvent::ToolEnd(name, result) => name.len() + result.len(),
            OutputEvent::Error(s) => s.len(),
        }
    }
}

/// Output buffer for background sessions
#[derive(Debug)]
pub struct OutputBuffer {
    events: VecDeque<OutputEvent>,
    max_size_bytes: usize,
    current_size_bytes: usize,
}

impl OutputBuffer {
    pub fn new(max_size_bytes: usize) -> Self {
        Self {
            events: VecDeque::new(),
            max_size_bytes,
            current_size_bytes: 0,
        }
    }

    /// Add event to buffer, evicting old events if necessary
    pub fn push(&mut self, event: OutputEvent) {
        let event_size = event.size_bytes();

        // Evict old events if needed
        while self.current_size_bytes + event_size > self.max_size_bytes && !self.events.is_empty() {
            if let Some(old_event) = self.events.pop_front() {
                self.current_size_bytes -= old_event.size_bytes();
            }
        }

        self.current_size_bytes += event_size;
        self.events.push_back(event);
    }

    /// Get current buffer size in bytes
    pub fn current_size(&self) -> usize {
        self.current_size_bytes
    }

    /// Replay buffered events to writer
    pub fn replay<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        for event in &self.events {
            match event {
                OutputEvent::Text(s) => writeln!(writer, "{}", s)?,
                OutputEvent::StyledText(s, _) => writeln!(writer, "{}", s)?,
                OutputEvent::ToolStart(name) => writeln!(writer, "→ Tool: {}", name)?,
                OutputEvent::ToolEnd(name, result) => writeln!(writer, "✓ {}: {}", name, result)?,
                OutputEvent::Error(e) => writeln!(writer, "Error: {}", e)?,
            }
        }
        Ok(())
    }

    /// Get all buffered events
    pub fn events(&self) -> &VecDeque<OutputEvent> {
        &self.events
    }

    /// Clear the buffer
    pub fn clear(&mut self) {
        self.events.clear();
        self.current_size_bytes = 0;
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Get current buffer size in bytes
    pub fn size_bytes(&self) -> usize {
        self.current_size_bytes
    }
}

/// State of a managed session
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SessionState {
    Active,
    WaitingForInput,
    Processing,
}

/// A managed session linking display, conversation, and execution
pub struct ManagedSession {
    /// Display information (name, type, colors)
    pub display: SessionDisplay,
    /// Conversation state (history, messages)
    pub conversation: ConversationState,
    /// Conversation ID
    pub conversation_id: String,
    /// Current state
    pub state: SessionState,
    /// Output buffer for background execution
    pub output_buffer: Arc<Mutex<OutputBuffer>>,
    /// Task handle for background execution (if running)
    pub task_handle: Option<JoinHandle<()>>,
    /// Last error encountered
    pub last_error: Option<String>,
    /// Session metadata for lifecycle tracking
    pub metadata: SessionMetadata,
}

impl ManagedSession {
    pub fn new(
        display: SessionDisplay,
        conversation: ConversationState,
        conversation_id: String,
        max_buffer_size: usize,
    ) -> Self {
        let now = Instant::now();
        Self {
            display,
            conversation,
            conversation_id,
            state: SessionState::Active,
            output_buffer: Arc::new(Mutex::new(OutputBuffer::new(max_buffer_size))),
            task_handle: None,
            last_error: None,
            metadata: SessionMetadata {
                created_at: now,
                last_active: now,
                message_count: 0,
            },
        }
    }

    /// Update session state with validation
    pub fn update_state(&mut self, new_state: SessionState) -> Result<(), String> {
        // Convert to SessionStatus for validation
        let current_status = match self.state {
            SessionState::Active => SessionStatus::Active,
            SessionState::WaitingForInput => SessionStatus::WaitingForInput,
            SessionState::Processing => SessionStatus::Processing,
        };

        let new_status = match new_state {
            SessionState::Active => SessionStatus::Active,
            SessionState::WaitingForInput => SessionStatus::WaitingForInput,
            SessionState::Processing => SessionStatus::Processing,
        };

        if !current_status.can_transition_to(&new_status) {
            return Err(format!("Invalid state transition: {:?} -> {:?}", self.state, new_state));
        }

        self.state = new_state;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::session::{
        SessionDisplay,
        SessionType,
    };

    #[test]
    fn test_output_event_size() {
        let event = OutputEvent::Text("hello".to_string());
        assert_eq!(event.size_bytes(), 5);

        let event = OutputEvent::StyledText("hello".to_string(), "bold".to_string());
        assert_eq!(event.size_bytes(), 9);
    }

    #[test]
    fn test_output_buffer_push() {
        let mut buffer = OutputBuffer::new(100);
        buffer.push(OutputEvent::Text("test".to_string()));
        assert_eq!(buffer.size_bytes(), 4);
        assert_eq!(buffer.events().len(), 1);
    }

    #[test]
    fn test_output_buffer_overflow_evicts_oldest() {
        let mut buffer = OutputBuffer::new(10);
        buffer.push(OutputEvent::Text("12345".to_string())); // 5 bytes
        buffer.push(OutputEvent::Text("67890".to_string())); // 5 bytes, total 10
        buffer.push(OutputEvent::Text("abc".to_string())); // 3 bytes, should evict first

        assert_eq!(buffer.events().len(), 2);
        assert_eq!(buffer.size_bytes(), 8); // "67890" + "abc"
    }

    #[test]
    fn test_output_buffer_clear() {
        let mut buffer = OutputBuffer::new(100);
        buffer.push(OutputEvent::Text("test".to_string()));
        buffer.clear();
        assert!(buffer.is_empty());
        assert_eq!(buffer.size_bytes(), 0);
    }

    #[test]
    fn test_session_state_can_transition() {
        use crate::theme::session::SessionStatus;

        // Active can transition to any state
        assert!(SessionStatus::Active.can_transition_to(&SessionStatus::WaitingForInput));
        assert!(SessionStatus::Active.can_transition_to(&SessionStatus::Processing));
        assert!(SessionStatus::Active.can_transition_to(&SessionStatus::Paused));

        // WaitingForInput can become Active or Paused
        assert!(SessionStatus::WaitingForInput.can_transition_to(&SessionStatus::Active));
        assert!(SessionStatus::WaitingForInput.can_transition_to(&SessionStatus::Paused));
        assert!(!SessionStatus::WaitingForInput.can_transition_to(&SessionStatus::Processing));

        // Processing can become WaitingForInput or Paused
        assert!(SessionStatus::Processing.can_transition_to(&SessionStatus::WaitingForInput));
        assert!(SessionStatus::Processing.can_transition_to(&SessionStatus::Paused));
        assert!(!SessionStatus::Processing.can_transition_to(&SessionStatus::Active));

        // Completed is terminal
        assert!(!SessionStatus::Completed.can_transition_to(&SessionStatus::Active));
        assert!(!SessionStatus::Completed.can_transition_to(&SessionStatus::WaitingForInput));
    }
}

impl Clone for ManagedSession {
    fn clone(&self) -> Self {
        Self {
            display: self.display.clone(),
            conversation: self.conversation.clone(),
            conversation_id: self.conversation_id.clone(),
            state: self.state.clone(),
            output_buffer: self.output_buffer.clone(),
            task_handle: None, // Can't clone JoinHandle
            last_error: self.last_error.clone(),
            metadata: self.metadata.clone(),
        }
    }
}
