use std::collections::HashMap;

use crate::theme::session::{SessionDisplay, SessionStatus, SessionType};

/// Manages multiple active sessions
#[derive(Debug, Default)]
pub struct SessionManager {
    sessions: HashMap<String, SessionDisplay>,
    active_session: Option<String>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            active_session: None,
        }
    }

    /// Start a new session
    pub fn start_session(&mut self, session_type: SessionType, name: impl Into<String>) -> Result<(), String> {
        let name = name.into();

        if self.sessions.contains_key(&name) {
            return Err(format!("Session '{}' already exists", name));
        }

        let session = SessionDisplay::new(session_type, name.clone());
        self.sessions.insert(name.clone(), session);
        self.active_session = Some(name);

        Ok(())
    }

    /// Switch to an existing session
    pub fn switch_session(&mut self, name: &str) -> Result<(), String> {
        if !self.sessions.contains_key(name) {
            return Err(format!("Session '{}' not found", name));
        }

        self.active_session = Some(name.to_string());
        Ok(())
    }

    /// Close a session
    pub fn close_session(&mut self, name: &str) -> Result<SessionDisplay, String> {
        let session = self
            .sessions
            .remove(name)
            .ok_or_else(|| format!("Session '{}' not found", name))?;

        // If we're closing the active session, clear active session
        if self.active_session.as_ref() == Some(&name.to_string()) {
            self.active_session = None;
        }

        Ok(session.with_status(SessionStatus::Completed))
    }

    /// Pause a session
    pub fn pause_session(&mut self, name: &str) -> Result<(), String> {
        let session = self
            .sessions
            .get_mut(name)
            .ok_or_else(|| format!("Session '{}' not found", name))?;

        session.status = SessionStatus::Paused;

        // If this was the active session, clear active session
        if self.active_session.as_ref() == Some(&name.to_string()) {
            self.active_session = None;
        }

        Ok(())
    }

    /// Resume a paused session
    pub fn resume_session(&mut self, name: &str) -> Result<(), String> {
        let session = self
            .sessions
            .get_mut(name)
            .ok_or_else(|| format!("Session '{}' not found", name))?;

        session.status = SessionStatus::Active;
        self.active_session = Some(name.to_string());

        Ok(())
    }

    /// Add a message to a session
    pub fn add_message(&mut self, name: &str) -> Result<(), String> {
        let session = self
            .sessions
            .get_mut(name)
            .ok_or_else(|| format!("Session '{}' not found", name))?;

        session.message_count += 1;
        Ok(())
    }

    /// Get the active session
    pub fn active_session(&self) -> Option<&SessionDisplay> {
        self.active_session.as_ref().and_then(|name| self.sessions.get(name))
    }

    /// Get a specific session
    pub fn get_session(&self, name: &str) -> Option<&SessionDisplay> {
        self.sessions.get(name)
    }

    /// List all sessions
    pub fn list_sessions(&self) -> Vec<&SessionDisplay> {
        self.sessions.values().collect()
    }

    /// List active sessions only
    pub fn list_active_sessions(&self) -> Vec<&SessionDisplay> {
        self.sessions
            .values()
            .filter(|s| s.status == SessionStatus::Active)
            .collect()
    }

    /// Format a message for the active session using theme colors
    pub fn format_active_message(&self, message: impl Into<String>) -> Option<String> {
        use crate::theme;

        self.active_session()
            .map(|session| session.format_message(message, &theme::theme().session))
    }

    /// Check if there are any active sessions
    pub fn has_active_sessions(&self) -> bool {
        self.sessions.values().any(|s| s.status == SessionStatus::Active)
    }

    /// Get colored list of all sessions using theme colors
    pub fn colored_session_list(&self) -> Vec<String> {
        use crate::theme;

        self.sessions
            .values()
            .map(|session| session.colored_list_entry(&theme::theme().session))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_start_session() {
        let mut manager = SessionManager::new();

        let result = manager.start_session(SessionType::Debug, "test-debug");
        assert!(result.is_ok());
        assert_eq!(manager.active_session, Some("test-debug".to_string()));
        assert!(manager.sessions.contains_key("test-debug"));
    }

    #[test]
    fn test_start_duplicate_session() {
        let mut manager = SessionManager::new();

        manager.start_session(SessionType::Debug, "test").unwrap();
        let result = manager.start_session(SessionType::Planning, "test");

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already exists"));
    }

    #[test]
    fn test_switch_session() {
        let mut manager = SessionManager::new();

        manager.start_session(SessionType::Debug, "debug1").unwrap();
        manager.start_session(SessionType::Planning, "plan1").unwrap();

        let result = manager.switch_session("debug1");
        assert!(result.is_ok());
        assert_eq!(manager.active_session, Some("debug1".to_string()));
    }

    #[test]
    fn test_switch_nonexistent_session() {
        let mut manager = SessionManager::new();

        let result = manager.switch_session("nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_close_session() {
        let mut manager = SessionManager::new();

        manager.start_session(SessionType::Debug, "test").unwrap();
        let result = manager.close_session("test");

        assert!(result.is_ok());
        let closed_session = result.unwrap();
        assert_eq!(closed_session.status, SessionStatus::Completed);
        assert!(!manager.sessions.contains_key("test"));
        assert_eq!(manager.active_session, None);
    }

    #[test]
    fn test_pause_and_resume_session() {
        let mut manager = SessionManager::new();

        manager.start_session(SessionType::Debug, "test").unwrap();

        // Pause session
        manager.pause_session("test").unwrap();
        let session = manager.get_session("test").unwrap();
        assert_eq!(session.status, SessionStatus::Paused);
        assert_eq!(manager.active_session, None);

        // Resume session
        manager.resume_session("test").unwrap();
        let session = manager.get_session("test").unwrap();
        assert_eq!(session.status, SessionStatus::Active);
        assert_eq!(manager.active_session, Some("test".to_string()));
    }

    #[test]
    fn test_add_message() {
        let mut manager = SessionManager::new();

        manager.start_session(SessionType::Debug, "test").unwrap();

        manager.add_message("test").unwrap();
        manager.add_message("test").unwrap();

        let session = manager.get_session("test").unwrap();
        assert_eq!(session.message_count, 2);
    }

    #[test]
    fn test_list_sessions() {
        let mut manager = SessionManager::new();

        manager.start_session(SessionType::Debug, "debug1").unwrap();
        manager.start_session(SessionType::Planning, "plan1").unwrap();
        manager.pause_session("plan1").unwrap();

        let all_sessions = manager.list_sessions();
        assert_eq!(all_sessions.len(), 2);

        let active_sessions = manager.list_active_sessions();
        assert_eq!(active_sessions.len(), 1);
        assert_eq!(active_sessions[0].name, "debug1");
    }

    #[test]
    fn test_format_active_message() {
        let mut manager = SessionManager::new();

        manager.start_session(SessionType::Debug, "test").unwrap();

        let formatted = manager.format_active_message("Test message");
        assert!(formatted.is_some());

        let formatted = formatted.unwrap();
        // Check for the prefix and message content, accounting for ANSI color codes
        assert!(formatted.contains("debug:"));
        assert!(formatted.contains("Test message"));
    }

    #[test]
    fn test_format_active_message_no_session() {
        let manager = SessionManager::new();

        let formatted = manager.format_active_message("Test message");
        assert!(formatted.is_none());
    }

    #[test]
    fn test_has_active_sessions() {
        let mut manager = SessionManager::new();
        assert!(!manager.has_active_sessions());

        manager.start_session(SessionType::Debug, "test").unwrap();
        assert!(manager.has_active_sessions());

        manager.pause_session("test").unwrap();
        assert!(!manager.has_active_sessions());
    }

    #[test]
    fn test_colored_session_list() {
        let mut manager = SessionManager::new();

        manager.start_session(SessionType::Debug, "debug1").unwrap();
        manager.start_session(SessionType::Planning, "plan1").unwrap();

        let colored_list = manager.colored_session_list();
        assert_eq!(colored_list.len(), 2);

        // The colored list should contain the session names
        let combined = colored_list.join(" ");
        assert!(combined.contains("debug1"));
        assert!(combined.contains("plan1"));
    }
}
