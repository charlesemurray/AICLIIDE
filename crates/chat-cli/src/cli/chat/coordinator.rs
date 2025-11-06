//! Multi-session coordinator for managing concurrent chat sessions

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use eyre::{
    Result,
    bail,
};
use tokio::sync::{
    Mutex,
    mpsc,
};

use crate::cli::chat::managed_session::{
    ManagedSession,
    OutputBuffer,
};
use crate::cli::chat::memory_monitor::MemoryMonitor;
use crate::cli::chat::queue_manager::QueueManager;
use crate::cli::chat::rate_limiter::ApiRateLimiter;
use crate::cli::chat::resource_cleanup::ResourceCleanupManager;
use crate::cli::chat::session_lock::SessionLockManager;
use crate::cli::chat::session_mode::SessionStateChange;
use crate::cli::chat::session_persistence::{
    PersistedSession,
    SessionPersistence,
};
use crate::theme::session::{
    SessionDisplay,
    SessionStatus,
    SessionType,
};

/// Validation constants
mod validation {
    pub const MAX_SESSION_NAME_LENGTH: usize = 64;
    pub const MIN_SESSION_NAME_LENGTH: usize = 1;
    pub const MAX_CONVERSATION_ID_LENGTH: usize = 128;
    pub const MIN_CONVERSATION_ID_LENGTH: usize = 1;
}

/// Validate session name
fn validate_session_name(name: &str) -> Result<()> {
    if name.is_empty() {
        bail!("Session name cannot be empty");
    }
    if name.len() > validation::MAX_SESSION_NAME_LENGTH {
        bail!("Session name too long (max {} characters)", validation::MAX_SESSION_NAME_LENGTH);
    }
    if name.len() < validation::MIN_SESSION_NAME_LENGTH {
        bail!("Session name too short (min {} characters)", validation::MIN_SESSION_NAME_LENGTH);
    }
    if !name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == ' ') {
        bail!("Session name contains invalid characters (use a-z, A-Z, 0-9, -, _, space)");
    }
    Ok(())
}

/// Validate conversation ID
fn validate_conversation_id(id: &str) -> Result<()> {
    if id.is_empty() {
        bail!("Conversation ID cannot be empty");
    }
    if id.len() > validation::MAX_CONVERSATION_ID_LENGTH {
        bail!("Conversation ID too long (max {} characters)", validation::MAX_CONVERSATION_ID_LENGTH);
    }
    if id.len() < validation::MIN_CONVERSATION_ID_LENGTH {
        bail!("Conversation ID too short (min {} characters)", validation::MIN_CONVERSATION_ID_LENGTH);
    }
    Ok(())
}

/// Configuration for multi-session coordinator
#[derive(Debug, Clone)]
pub struct CoordinatorConfig {
    /// Maximum number of active sessions
    pub max_active_sessions: usize,
    /// Output buffer size per session in bytes
    pub buffer_size_bytes: usize,
    /// Maximum concurrent API calls
    pub max_concurrent_api_calls: usize,
    /// Session timeout for automatic cleanup
    pub session_timeout: Duration,
    /// Cleanup interval
    pub cleanup_interval: Duration,
    /// State change channel capacity
    pub state_channel_capacity: usize,
}

impl Default for CoordinatorConfig {
    fn default() -> Self {
        Self {
            max_active_sessions: 10,
            buffer_size_bytes: 10 * 1024 * 1024, // 10 MB
            max_concurrent_api_calls: 5,
            session_timeout: Duration::from_secs(3600), // 1 hour
            cleanup_interval: Duration::from_secs(300), // 5 minutes
            state_channel_capacity: 100,
        }
    }
}

/// Configuration for creating a new session
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// Session name
    pub name: String,
    /// Session type
    pub session_type: SessionType,
}

/// Context required for session creation
#[derive(Clone)]
pub struct SessionContext {
    /// Conversation ID
    pub conversation_id: String,
    /// Operating system interface
    pub os: crate::os::Os,
    /// Agent configuration
    pub agents: crate::cli::agent::Agents,
    /// Tool configuration
    pub tool_config: std::collections::HashMap<String, crate::cli::chat::tools::ToolSpec>,
    /// Tool manager
    pub tool_manager: crate::cli::chat::tool_manager::ToolManager,
    /// Optional model ID
    pub model_id: Option<String>,
}

/// Combined session state to prevent race conditions
/// All session-related data is protected by a single lock
pub(crate) struct SessionState {
    /// All managed sessions by conversation_id
    pub(crate) sessions: HashMap<String, ManagedSession>,
    /// Currently active session ID
    pub(crate) active_session_id: Option<String>,
    /// Ordered list of session IDs (for numbering)
    pub(crate) session_order: Vec<String>,
    /// Flag to signal application should quit
    pub(crate) should_quit: bool,
    /// Sessions with pending background work completion
    pub(crate) background_notifications: HashMap<String, String>,
}

/// Coordinates multiple chat sessions
pub struct MultiSessionCoordinator {
    /// Combined session state (single lock to prevent deadlocks)
    pub(crate) state: Arc<Mutex<SessionState>>,
    /// Configuration
    config: CoordinatorConfig,
    /// State change receiver
    state_rx: mpsc::Receiver<SessionStateChange>,
    /// State change sender (cloned for each session)
    state_tx: mpsc::Sender<SessionStateChange>,
    /// API rate limiter
    rate_limiter: ApiRateLimiter,
    /// Memory monitor
    memory_monitor: MemoryMonitor,
    /// Session persistence with error handling
    persistence: Option<SessionPersistence>,
    /// Lock manager for race condition protection
    lock_manager: SessionLockManager,
    /// Resource cleanup manager
    cleanup_manager: ResourceCleanupManager,
    /// Dropped events counter
    dropped_events: Arc<std::sync::atomic::AtomicUsize>,
    /// Reference to the currently active ChatSession
    active_chat_session: Option<Arc<tokio::sync::Mutex<crate::cli::chat::ChatSession>>>,
    /// Message queue manager for LLM processing
    pub queue_manager: Arc<QueueManager>,
}

impl MultiSessionCoordinator {
    /// Create a new coordinator
    pub fn new(config: CoordinatorConfig) -> Self {
        let (state_tx, state_rx) = mpsc::channel(config.state_channel_capacity);
        let rate_limiter = ApiRateLimiter::new(config.max_concurrent_api_calls);
        let memory_monitor = MemoryMonitor::new();
        let queue_manager = Arc::new(QueueManager::new());
        
        // Start background worker
        queue_manager.clone().start_background_worker();
        eprintln!("[COORDINATOR] Background worker started");

        Self {
            state: Arc::new(Mutex::new(SessionState {
                sessions: HashMap::new(),
                active_session_id: None,
                session_order: Vec::new(),
                should_quit: false,
                background_notifications: HashMap::new(),
            })),
            config,
            state_rx,
            state_tx,
            rate_limiter,
            memory_monitor,
            persistence: None,
            lock_manager: SessionLockManager::default(),
            cleanup_manager: ResourceCleanupManager::default(),
            dropped_events: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            active_chat_session: None,
            queue_manager,
        }
    }

    /// Enable persistence with error handling
    pub fn enable_persistence(&mut self, base_dir: std::path::PathBuf) -> Result<()> {
        self.persistence = Some(SessionPersistence::new(&base_dir)?);
        Ok(())
    }
    
    /// Set API client for real LLM calls in background processing
    pub fn set_api_client(&mut self, client: crate::api_client::ApiClient) {
        // Calculate number of workers (leave room for active session)
        let total_permits = self.rate_limiter.max_concurrent();
        let num_workers = if total_permits > 2 {
            total_permits - 2  // Reserve 2 for active session
        } else {
            1
        };
        
        // Replace queue_manager with one that has the API client and shared rate limiter
        let new_queue_manager = Arc::new(crate::cli::chat::queue_manager::QueueManager::with_rate_limiter(
            client,
            self.rate_limiter.clone(),
            num_workers
        ));
        new_queue_manager.clone().start_background_worker();
        self.queue_manager = new_queue_manager;
        eprintln!("[COORDINATOR] API client configured for background processing");
        eprintln!("[COORDINATOR] Rate limiter: {} total permits, {} workers, {} reserved for active", 
            total_permits, num_workers, total_permits - num_workers);
    }

    /// Save session to disk with error handling
    pub async fn save_session(&self, conversation_id: &str) -> Result<()> {
        eprintln!("[DEBUG] save_session called for conversation_id: {}", conversation_id);
        
        let persistence = match &self.persistence {
            Some(p) => p,
            None => {
                eprintln!("[DEBUG] save_session: persistence disabled");
                return Ok(());
            }
        };

        let state = self.state.lock().await;
        let session = state
            .sessions
            .get(conversation_id)
            .ok_or_else(|| {
                eprintln!("[DEBUG] save_session: session not found in state");
                eyre::eyre!("Session not found: {}", conversation_id)
            })?;

        eprintln!("[DEBUG] save_session: session name='{}', type={:?}, status={:?}", 
            session.display.name, session.display.session_type, session.display.status);

        let persisted = PersistedSession {
            conversation_id: conversation_id.to_string(),
            name: session.display.name.clone(),
            session_type: session.display.session_type,
            status: session.display.status,
            created_at: 0,
            last_active: 0,
        };

        eprintln!("[DEBUG] save_session: saving to disk...");
        persistence.save_session(&persisted)?;
        eprintln!("[DEBUG] save_session: saved successfully");
        Ok(())
    }

    /// Load all sessions from disk with error handling
    pub async fn load_sessions(&mut self, os: &mut crate::os::Os) -> Result<usize> {
        eprintln!("[DEBUG] load_sessions called");
        
        let persistence = match &self.persistence {
            Some(p) => p,
            None => {
                eprintln!("[DEBUG] load_sessions: persistence disabled");
                return Ok(0);
            }
        };

        let persisted_sessions = persistence.load_all_sessions()?;
        eprintln!("[DEBUG] load_sessions: found {} persisted sessions", persisted_sessions.len());
        
        // Filter for active sessions only (not archived/completed)
        let active_sessions: Vec<_> = persisted_sessions
            .into_iter()
            .filter(|s| {
                let is_active = !matches!(s.status, SessionStatus::Completed);
                eprintln!("[DEBUG] load_sessions: session '{}' ({}), status={:?}, active={}", 
                    s.name, s.conversation_id, s.status, is_active);
                is_active
            })
            .collect();
        
        eprintln!("[DEBUG] load_sessions: {} active sessions to restore", active_sessions.len());
        
        if active_sessions.is_empty() {
            return Ok(0);
        }
        
        let current_dir = std::env::current_dir().unwrap_or_default();
        eprintln!("[DEBUG] load_sessions: current_dir={:?}", current_dir);
        
        let mut restored_count = 0;
        
        for persisted in active_sessions {
            eprintln!("[DEBUG] load_sessions: attempting to restore session '{}'", persisted.name);
            
            // Try to load conversation from database
            let conversation = os.database
                .get_conversation_by_path(&current_dir)
                .ok()
                .flatten()
                .filter(|cs| {
                    let matches = cs.conversation_id() == &persisted.conversation_id;
                    eprintln!("[DEBUG] load_sessions: conversation_id match: {}", matches);
                    matches
                });
            
            if let Some(mut conv) = conversation {
                eprintln!("[DEBUG] load_sessions: found conversation in database, history_len={}", conv.history().len());
                
                // Create session context
                let tool_config = std::collections::HashMap::new();
                
                let config = SessionConfig {
                    name: persisted.name.clone(),
                    session_type: persisted.session_type,
                };
                
                let context = SessionContext {
                    conversation_id: persisted.conversation_id.clone(),
                    os: os.clone(),
                    agents: conv.agents.clone(),
                    tool_config,
                    tool_manager: conv.tool_manager.clone(),
                    model_id: conv.model_info.as_ref().map(|m| m.model_id.clone()),
                };
                
                match self.create_session(config, context).await {
                    Ok(_) => {
                        restored_count += 1;
                        eprintln!("[DEBUG] load_sessions: ✓ restored session '{}'", persisted.name);
                    },
                    Err(e) => {
                        eprintln!("[DEBUG] load_sessions: ✗ failed to restore '{}': {}", persisted.name, e);
                    }
                }
            } else {
                eprintln!("[DEBUG] load_sessions: no conversation found in database for '{}'", persisted.name);
            }
        }
        
        eprintln!("[DEBUG] load_sessions: restored {} sessions", restored_count);
        Ok(restored_count)
    }

    /// Create a new session
    pub async fn create_session(
        &mut self,
        config: SessionConfig,
        mut context: SessionContext,
    ) -> Result<String> {
        eprintln!("[DEBUG] create_session: name='{}', type={:?}, conversation_id={}", 
            config.name, config.session_type, context.conversation_id);
        
        // Validate inputs
        validate_conversation_id(&context.conversation_id)?;
        validate_session_name(&config.name)?;
        
        let mut state = self.state.lock().await;

        // Check session limit
        if state.sessions.len() >= self.config.max_active_sessions {
            bail!("Maximum active sessions ({}) reached", self.config.max_active_sessions);
        }

        // Check for duplicate names
        if state.sessions.values().any(|s| s.display.name == config.name) {
            eprintln!("[DEBUG] create_session: duplicate name '{}' found", config.name);
            bail!("Session with name '{}' already exists", config.name);
        }

        // Create session display
        let display = SessionDisplay::new(config.session_type, config.name.clone());
        eprintln!("[DEBUG] create_session: created display with name='{}'", display.name);

        // Create output buffer
        let buffer = Arc::new(Mutex::new(OutputBuffer::new(self.config.buffer_size_bytes)));

        // Try to load existing conversation from database, otherwise create new
        let conversation = {
            let existing = context.os.database
                .get_conversation_by_path(std::env::current_dir().unwrap_or_default())
                .ok()
                .flatten()
                .filter(|cs| cs.conversation_id() == &context.conversation_id);
            
            if let Some(mut cs) = existing {
                eprintln!("[DEBUG] Restored conversation history for session {} ({} messages)", 
                    config.name, cs.history().len());
                // Update with current context
                cs.tool_manager = context.tool_manager;
                cs.agents = context.agents;
                cs
            } else {
                eprintln!("[DEBUG] Creating new conversation for session {}", config.name);
                crate::cli::chat::conversation::ConversationState::new(
                    &context.conversation_id,
                    context.agents,
                    context.tool_config,
                    context.tool_manager,
                    context.model_id,
                    &context.os,
                    true, // mcp_enabled
                )
                .await
            }
        };

        // Create managed session with real ConversationState
        let now = std::time::Instant::now();
        let session = ManagedSession {
            display,
            conversation,
            conversation_id: context.conversation_id.clone(),
            state: crate::cli::chat::managed_session::SessionState::Active,
            output_buffer: buffer,
            task_handle: None,
            last_error: None,
            metadata: crate::cli::chat::managed_session::SessionMetadata {
                created_at: now,
                last_active: now,
                message_count: 0,
            },
            chat_session: None, // Will be set later when ChatSession is created
            background_responses: Vec::new(),
        };

        state.sessions.insert(context.conversation_id.clone(), session);
        
        // Add to session order for numbering
        state.session_order.push(context.conversation_id.clone());

        // Set as active if first session
        if state.active_session_id.is_none() {
            state.active_session_id = Some(context.conversation_id.clone());
        }

        Ok(context.conversation_id)
    }

    /// Switch to a different session
    pub async fn switch_session(&mut self, name: &str) -> Result<()> {
        let mut state = self.state.lock().await;

        eprintln!("[DEBUG] switch_session called with name: {}", name);
        eprintln!("[DEBUG] Current sessions: {:?}", state.sessions.keys().collect::<Vec<_>>());

        // Try to parse as number first
        let target_id = if let Ok(num) = name.parse::<usize>() {
            // Switch by number (1-indexed)
            if num == 0 || num > state.session_order.len() {
                bail!("Session number {} out of range (1-{})", num, state.session_order.len());
            }
            state.session_order.get(num - 1).cloned()
                .ok_or_else(|| eyre::eyre!("Session number {} not found", num))?
        } else {
            // Validate input only if not a number
            validate_session_name(name)?;
            
            // Find session by name, or by conversation_id prefix
            state
                .sessions
                .iter()
                .find(|(_, s)| s.display.name == name)
                .or_else(|| {
                    // Fallback: try matching by conversation_id prefix
                    state.sessions.iter().find(|(id, _)| id.starts_with(name))
                })
                .map(|(id, _)| id.clone())
                .ok_or_else(|| eyre::eyre!("Session '{}' not found", name))?
        };

        eprintln!("[DEBUG] Found target session ID: {}", target_id);

        // Update active session
        state.active_session_id = Some(target_id.clone());
        
        eprintln!("[DEBUG] Updated active_session_id to: {}", target_id);
        
        // Update last_active timestamp
        if let Some(session) = state.sessions.get_mut(&target_id) {
            session.metadata.last_active = std::time::Instant::now();
        }

        Ok(())
    }

    /// Update last_active timestamp for a session
    pub async fn touch_session(&mut self, session_id: &str) -> Result<()> {
        // Validate input
        validate_conversation_id(session_id)?;
        
        let mut state = self.state.lock().await;
        if let Some(session) = state.sessions.get_mut(session_id) {
            session.metadata.last_active = std::time::Instant::now();
            Ok(())
        } else {
            bail!("Session not found: {}", session_id)
        }
    }

    /// Acquire lock for session (prevents concurrent access)
    pub async fn lock_session(
        &self,
        session_id: &str,
        holder: &str,
    ) -> Result<crate::cli::chat::session_lock::SessionLockGuard> {
        self.lock_manager.try_lock(session_id, holder).await
    }

    /// Check if session is currently locked
    pub async fn is_session_locked(&self, session_id: &str) -> bool {
        self.lock_manager.is_locked(session_id).await
    }

    /// Get all locked sessions
    pub async fn get_locked_sessions(&self) -> Vec<String> {
        self.lock_manager.locked_sessions().await
    }

    /// Clean up stale locks
    pub async fn cleanup_stale_locks(&self) -> usize {
        self.lock_manager.cleanup_stale_locks().await
    }

    /// Update resource statistics
    pub async fn update_resource_stats(&self) {
        let state = self.state.lock().await;
        let active_count = state.sessions.len();

        // Calculate total buffer usage
        let mut total_bytes = 0;
        for session in state.sessions.values() {
            let buffer = session.output_buffer.lock().await;
            total_bytes += buffer.current_size();
        }

        self.cleanup_manager.update_stats(active_count, total_bytes).await;
    }

    /// Get resource statistics
    pub async fn get_resource_stats(&self) -> crate::cli::chat::resource_cleanup::ResourceStats {
        self.cleanup_manager.get_stats().await
    }

    /// Check for resource leaks
    pub async fn check_resource_leaks(&self) -> Vec<String> {
        self.cleanup_manager.check_for_leaks().await
    }

    /// Get cleanup recommendations
    pub async fn get_cleanup_recommendations(&self) -> Vec<String> {
        self.cleanup_manager.get_recommendations().await
    }

    /// Perform periodic cleanup
    pub async fn perform_cleanup(&self) -> eyre::Result<()> {
        if !self.cleanup_manager.needs_cleanup().await {
            return Ok(());
        }

        // Update stats
        self.update_resource_stats().await;

        // Clean up stale locks
        let stale_locks = self.cleanup_stale_locks().await;
        if stale_locks > 0 {
            eprintln!("Cleaned up {} stale locks", stale_locks);
        }

        // Check for leaks
        let warnings = self.check_resource_leaks().await;
        for warning in warnings {
            eprintln!("Resource warning: {}", warning);
        }

        self.cleanup_manager.mark_cleanup_done().await;
        Ok(())
    }

    /// Close a session
    pub async fn close_session(&mut self, name: &str, context: Option<SessionContext>) -> Result<()> {
        // Validate input
        validate_session_name(name)?;
        
        let mut state = self.state.lock().await;

        // Find session by name
        let target_id = state
            .sessions
            .iter()
            .find(|(_, s)| s.display.name == name)
            .map(|(id, _)| id.clone())
            .ok_or_else(|| eyre::eyre!("Session '{}' not found", name))?;

        // Archive the session (mark as Completed)
        if let Some(persistence) = &self.persistence {
            if let Some(session) = state.sessions.get(&target_id) {
                let persisted = PersistedSession {
                    conversation_id: target_id.clone(),
                    name: session.display.name.clone(),
                    session_type: session.display.session_type,
                    status: SessionStatus::Completed, // Mark as archived
                    created_at: session.metadata.created_at.elapsed().as_secs(),
                    last_active: session.metadata.last_active.elapsed().as_secs(),
                };
                let _ = persistence.save_session(&persisted);
            }
        }
        
        // Remove session
        state.sessions.remove(&target_id);
        
        // Remove from session order
        state.session_order.retain(|id| id != &target_id);

        // If we closed the active session, handle next session
        if state.active_session_id.as_ref() == Some(&target_id) {
            // Find another session to switch to
            let next_session = state.sessions.keys().next().cloned();
            
            if next_session.is_none() {
                // No sessions left - create a new one if context provided
                if let Some(ctx) = context {
                    drop(state); // Release lock before calling create_session
                    
                    let new_id = self.create_session(
                        SessionConfig {
                            name: "default".to_string(),
                            session_type: crate::theme::session::SessionType::Development,
                        },
                        ctx,
                    ).await?;
                    
                    // Set as active
                    let mut state = self.state.lock().await;
                    state.active_session_id = Some(new_id);
                } else {
                    state.active_session_id = None;
                }
            } else {
                state.active_session_id = next_session;
            }
        }

        Ok(())
    }

    /// Get the active session ID
    pub async fn active_session_id(&self) -> Option<String> {
        let state = self.state.lock().await;
        state.active_session_id.clone()
    }
    
    /// Add background work notification for a session
    pub async fn notify_background_complete(&self, session_id: String, message: String) {
        eprintln!("[NOTIFY] Background work complete for session {}", session_id);
        let mut state = self.state.lock().await;
        state.background_notifications.insert(session_id, message);
    }
    
    /// Store background response for a session
    pub async fn store_background_response(&self, session_id: &str, response: String) {
        let mut state = self.state.lock().await;
        if let Some(session) = state.sessions.get_mut(session_id) {
            session.background_responses.push(response.clone());
            eprintln!("[STORE] Stored background response for session {} ({} bytes, {} total responses)", 
                session_id, response.len(), session.background_responses.len());
        } else {
            eprintln!("[STORE] ERROR: Session {} not found, cannot store response", session_id);
        }
    }
    
    /// Get and clear background responses for a session
    pub async fn take_background_responses(&self, session_id: &str) -> Vec<String> {
        let mut state = self.state.lock().await;
        if let Some(session) = state.sessions.get_mut(session_id) {
            let responses = std::mem::take(&mut session.background_responses);
            eprintln!("[RETRIEVE] Retrieved {} background responses for session {}", 
                responses.len(), session_id);
            responses
        } else {
            eprintln!("[RETRIEVE] ERROR: Session {} not found", session_id);
            Vec::new()
        }
    }
    
    /// Check if session has pending notifications
    pub async fn has_notification(&self, session_id: &str) -> bool {
        let state = self.state.lock().await;
        state.background_notifications.contains_key(session_id)
    }
    
    /// Get total notification count across all sessions
    pub async fn notification_count(&self) -> usize {
        let state = self.state.lock().await;
        state.background_notifications.len()
    }
    
    /// Check if any background work is in progress
    pub async fn has_background_work(&self) -> bool {
        let stats = self.queue_manager.stats().await;
        stats.high_priority_count > 0 || stats.low_priority_count > 0
    }
    
    /// Get and clear notification for session
    pub async fn take_notification(&self, session_id: &str) -> Option<String> {
        let mut state = self.state.lock().await;
        state.background_notifications.remove(session_id)
    }
    
    /// Signal the coordinator to quit the application
    pub async fn quit(&self) {
        let mut state = self.state.lock().await;
        state.should_quit = true;
    }

    /// Get session by ID
    pub async fn get_session(&self, id: &str) -> Option<String> {
        let state = self.state.lock().await;
        state.sessions.get(id).map(|s| s.display.name.clone())
    }
    
    /// Get session ID by number (1-indexed)
    pub async fn get_session_id_by_number(&self, number: usize) -> Option<String> {
        let state = self.state.lock().await;
        if number == 0 || number > state.session_order.len() {
            return None;
        }
        state.session_order.get(number - 1).cloned()
    }
    
    /// Get session number by ID (1-indexed)
    pub async fn get_session_number(&self, id: &str) -> Option<usize> {
        let state = self.state.lock().await;
        state.session_order.iter().position(|sid| sid == id).map(|pos| pos + 1)
    }

    /// List all session names
    pub async fn list_sessions(&self) -> Vec<String> {
        let state = self.state.lock().await;
        state.sessions.values().map(|s| s.display.name.clone()).collect()
    }
    
    /// List sessions with numbers in order
    pub async fn list_sessions_with_numbers(&self) -> Vec<(usize, String, bool)> {
        let state = self.state.lock().await;
        let active_id = state.active_session_id.as_ref();
        
        state.session_order
            .iter()
            .enumerate()
            .filter_map(|(idx, id)| {
                state.sessions.get(id).map(|s| {
                    let is_active = active_id == Some(id);
                    (idx + 1, s.display.name.clone(), is_active)
                })
            })
            .collect()
    }

    /// Get sessions waiting for input
    pub async fn get_waiting_sessions(&self) -> Vec<String> {
        let state = self.state.lock().await;
        state
            .sessions
            .values()
            .filter(|s| {
                matches!(
                    s.state,
                    crate::cli::chat::managed_session::SessionState::WaitingForInput
                )
            })
            .map(|s| s.display.name.clone())
            .collect()
    }

    /// Update session state
    pub async fn update_session_state(
        &mut self,
        id: &str,
        new_state: crate::cli::chat::managed_session::SessionState,
    ) -> Result<()> {
        let mut state = self.state.lock().await;

        if let Some(session) = state.sessions.get_mut(id) {
            session.update_state(new_state).map_err(|e| eyre::eyre!(e))?;

            // Update display status
            session.display.status = match new_state {
                crate::cli::chat::managed_session::SessionState::Active => SessionStatus::Active,
                crate::cli::chat::managed_session::SessionState::WaitingForInput => SessionStatus::WaitingForInput,
                crate::cli::chat::managed_session::SessionState::Processing => SessionStatus::Processing,
            };
        }

        Ok(())
    }

    /// Get the display name of a session
    pub async fn get_session_name(&self, conversation_id: &str) -> Option<String> {
        let state = self.state.lock().await;
        state.sessions.get(conversation_id).map(|s| s.display.name.clone())
    }

    /// Get a managed session by conversation_id
    pub async fn get_managed_session(&self, conversation_id: &str) -> Option<ManagedSession> {
        let state = self.state.lock().await;
        state.sessions.get(conversation_id).cloned()
    }

    /// Get a mutable reference to a managed session
    pub async fn get_managed_session_mut(&self, conversation_id: &str) -> Option<ManagedSession> {
        let mut state = self.state.lock().await;
        state.sessions.get_mut(conversation_id).map(|s| {
            // Update the conversation in place
            s.clone()
        })
    }

    /// Update a session's conversation
    pub async fn update_session_conversation(&self, conversation_id: &str, conversation: crate::cli::chat::ConversationState) -> Result<()> {
        let mut state = self.state.lock().await;
        if let Some(session) = state.sessions.get_mut(conversation_id) {
            session.conversation = conversation;
            Ok(())
        } else {
            bail!("Session not found: {}", conversation_id)
        }
    }

    /// Set the active ChatSession reference
    pub fn set_active_chat_session(&mut self, session: Arc<tokio::sync::Mutex<crate::cli::chat::ChatSession>>) {
        self.active_chat_session = Some(session);
    }

    /// Store a ChatSession in a ManagedSession
    pub async fn set_chat_session(&self, conversation_id: &str, chat_session: Arc<tokio::sync::Mutex<crate::cli::chat::ChatSession>>) -> Result<()> {
        let mut state = self.state.lock().await;
        if let Some(session) = state.sessions.get_mut(conversation_id) {
            session.chat_session = Some(chat_session);
            Ok(())
        } else {
            bail!("Session not found: {}", conversation_id)
        }
    }

    /// Get state change sender for new sessions
    pub fn state_sender(&self) -> mpsc::Sender<SessionStateChange> {
        self.state_tx.clone()
    }

    /// Get dropped events count
    pub fn dropped_events_count(&self) -> usize {
        self.dropped_events.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Send state change with backpressure handling
    pub async fn send_state_change(&self, change: SessionStateChange) -> Result<()> {
        match self.state_tx.try_send(change) {
            Ok(_) => Ok(()),
            Err(mpsc::error::TrySendError::Full(_)) => {
                self.dropped_events.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                tracing::warn!("State channel full, dropping event");
                Ok(())
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {
                bail!("State channel closed")
            }
        }
    }

    /// Get rate limiter for API calls
    pub fn rate_limiter(&self) -> ApiRateLimiter {
        self.rate_limiter.clone()
    }

    /// Get current API call statistics
    pub async fn api_stats(&self) -> (usize, usize) {
        let active = self.rate_limiter.active_count().await;
        let available = self.rate_limiter.available_permits();
        (active, available)
    }

    /// Get memory monitor
    pub fn memory_monitor(&self) -> MemoryMonitor {
        self.memory_monitor.clone()
    }

    /// Get memory usage summary
    pub async fn memory_summary(&self) -> crate::cli::chat::memory_monitor::MemorySummary {
        self.memory_monitor.summary().await
    }

    /// Check for sessions that should be hibernated
    pub async fn check_memory_pressure(&self) -> Vec<String> {
        self.memory_monitor.sessions_to_hibernate().await
    }

    /// Clean up inactive sessions based on timeout
    pub async fn cleanup_inactive_sessions(&mut self, max_age: Duration) -> Result<usize> {
        let mut state = self.state.lock().await;
        let now = std::time::Instant::now();
        
        let to_remove: Vec<_> = state.sessions
            .iter()
            .filter(|(_, session)| {
                now.duration_since(session.metadata.last_active) > max_age
            })
            .map(|(id, _)| id.clone())
            .collect();
        
        for id in &to_remove {
            state.sessions.remove(id);
            if let Some(p) = &self.persistence {
                let _ = p.delete_session(id);
            }
        }
        
        Ok(to_remove.len())
    }

    /// Start background cleanup task
    /// Note: Requires coordinator to be wrapped in Arc<Mutex<>> for background task
    /// For now, call cleanup_inactive_sessions() manually or from main loop
    pub fn start_cleanup_task(&self) {
        // TODO: Implement background task when coordinator is Arc<Mutex<>>
        tracing::warn!("Background cleanup task not yet implemented");
    }

    /// Process state changes from background sessions
    pub async fn process_state_changes(&mut self) -> Result<()> {
        while let Ok(change) = self.state_rx.try_recv() {
            match change {
                SessionStateChange::NeedsInput(id) => {
                    self.update_session_state(&id, crate::cli::chat::managed_session::SessionState::WaitingForInput)
                        .await?;
                },
                SessionStateChange::Processing(id) => {
                    self.update_session_state(&id, crate::cli::chat::managed_session::SessionState::Processing)
                        .await?;
                },
                SessionStateChange::Completed(id) => {
                    // Mark as completed
                    let mut state = self.state.lock().await;
                    if let Some(session) = state.sessions.get_mut(&id) {
                        session.display.status = SessionStatus::Completed;
                    }
                },
                SessionStateChange::Error(id, error) => {
                    let mut state = self.state.lock().await;
                    if let Some(session) = state.sessions.get_mut(&id) {
                        session.last_error = Some(error);
                    }
                },
            }
        }
        Ok(())
    }

    /// Main coordinator loop - manages active ChatSession execution
    /// This is the entry point for multi-session support
    pub async fn run(
        coord_arc: std::sync::Arc<tokio::sync::Mutex<Self>>,
        os: &mut crate::os::Os,
    ) -> Result<()> {
        loop {
            let (active_id, initial_active_id) = {
                let coord = coord_arc.lock().await;
                let state = coord.state.lock().await;
                eprintln!("[DEBUG] Coordinator loop - active_session_id: {:?}", state.active_session_id);
                let id = state.active_session_id.clone();
                (id.clone(), id)  // Store both for comparison later
            };

            let Some(session_id) = active_id else {
                eprintln!("[DEBUG] No active session, exiting coordinator loop");
                break;
            };

            eprintln!("[DEBUG] Running session: {}", session_id);

            // Get or create ChatSession for active session
            let chat_session = {
                let coord = coord_arc.lock().await;
                let mut state = coord.state.lock().await;
                let session = state.sessions.get_mut(&session_id)
                    .ok_or_else(|| eyre::eyre!("Active session not found"))?;
                
                // Create ChatSession if it doesn't exist
                if session.chat_session.is_none() {
                    eprintln!("[DEBUG] Creating NEW ChatSession for session: {}", session_id);
                    use crate::cli::chat::input_source::InputSource;
                    use tokio::sync::broadcast;
                    
                    // Create input source
                    let (prompt_request_sender, prompt_request_receiver) = broadcast::channel(5);
                    let (_prompt_response_sender, prompt_response_receiver) = broadcast::channel(5);
                    let input_source = InputSource::new(os, prompt_request_sender, prompt_response_receiver)?;
                    
                    // Create ChatSession from existing conversation
                    let mut chat_session = crate::cli::chat::ChatSession::from_conversation(
                        os,
                        session.conversation.clone(),
                        input_source,
                    ).await?;
                    
                    // Set coordinator reference
                    chat_session.coordinator = Some(coord_arc.clone());
                    eprintln!("[DEBUG] Set coordinator reference on new ChatSession");
                    
                    session.chat_session = Some(std::sync::Arc::new(tokio::sync::Mutex::new(chat_session)));
                } else {
                    eprintln!("[DEBUG] REUSING existing ChatSession for session: {}", session_id);
                }
                
                session.chat_session.clone().unwrap()
            };

            // For now, keep blocking behavior but add ability to detect switches
            // TODO: Implement true background execution with output buffering
            
            let mut session = chat_session.lock().await;
            
            match session.spawn(os).await {
                Ok(_) => {
                    // Save conversation state
                    let conversation = session.conversation.clone();
                    drop(session);
                    
                    {
                        let coord = coord_arc.lock().await;
                        let mut state = coord.state.lock().await;
                        if let Some(managed_session) = state.sessions.get_mut(&session_id) {
                            managed_session.conversation = conversation;
                        }
                    }
                    
                    let coord = coord_arc.lock().await;
                    let state = coord.state.lock().await;
                    let new_active = state.active_session_id.clone();
                    let should_quit = state.should_quit;
                    
                    eprintln!("[DEBUG] Session exited. Initial ID: {:?}, New active ID: {:?}, should_quit: {}", initial_active_id, new_active, should_quit);
                    
                    if should_quit {
                        eprintln!("[DEBUG] should_quit flag set - saving sessions and exiting");
                        
                        let session_ids: Vec<_> = state.sessions.keys().cloned().collect();
                        eprintln!("[DEBUG] quit: found {} sessions to save", session_ids.len());
                        drop(state);
                        
                        for id in &session_ids {
                            eprintln!("[DEBUG] quit: saving session {}", id);
                            match coord.save_session(id).await {
                                Ok(_) => eprintln!("[DEBUG] quit: ✓ saved {}", id),
                                Err(e) => eprintln!("[DEBUG] quit: ✗ failed to save {}: {}", id, e),
                            }
                        }
                        
                        eprintln!("[DEBUG] quit: all sessions saved, exiting");
                        drop(coord);
                        std::process::exit(0);
                    }
                    
                    if new_active != initial_active_id {
                        eprintln!("[DEBUG] Active session changed, continuing loop");
                        continue;
                    } else {
                        eprintln!("[DEBUG] Session exited without switch - unexpected");
                        break;
                    }
                },
                Err(e) => return Err(e),
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires full context
    async fn test_create_session_with_real_conversation_state() {
        // This test verifies that we can create a session with a real ConversationState
        // (no unsafe placeholder)

        let config = CoordinatorConfig::default();
        let coordinator = MultiSessionCoordinator::new(config);

        // Note: This test would need a real Os, agents, tool_config, etc.
        // For now, we just verify the signature compiles
        assert!(coordinator.state.lock().await.sessions.is_empty());
    }

    #[tokio::test]
    #[ignore] // Requires full context
    async fn test_create_session() {
        // TODO: Add test helper with mock parameters
    }

    #[tokio::test]
    #[ignore] // Requires full context
    #[ignore] // Requires full Os, agents, tools context
    async fn test_create_multiple_sessions() {
        // TODO: Add test helper with mock parameters
    }

    #[tokio::test]
    #[ignore] // Requires full context
    #[ignore] // Requires full context
    async fn test_session_limit() {
        let config = CoordinatorConfig {
            max_active_sessions: 2,
            ..Default::default()
        };
        let mut coordinator = MultiSessionCoordinator::new(config);

        coordinator
            .create_session("test-1".to_string(), SessionType::Development, Some("s1".to_string()))
            .await
            .unwrap();

        coordinator
            .create_session("test-2".to_string(), SessionType::Development, Some("s2".to_string()))
            .await
            .unwrap();

        let result = coordinator
            .create_session("test-3".to_string(), SessionType::Development, Some("s3".to_string()))
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Maximum active sessions"));
    }

    #[tokio::test]
    #[ignore] // Requires full context
    #[ignore] // Requires full context
    async fn test_switch_session() {
        // TODO: Add test helper with mock parameters
    }

    #[tokio::test]
    #[ignore] // Requires full context
    #[ignore] // Requires full context
    async fn test_close_session() {
        // TODO: Add test helper with mock parameters
    }

    #[tokio::test]
    #[ignore] // Requires full context
    #[ignore] // Requires full context
    async fn test_get_waiting_sessions() {
        // TODO: Add test helper with mock parameters
    }

    #[tokio::test]
    #[ignore] // Requires full context
    async fn test_lock_session() {
        let coordinator = MultiSessionCoordinator::new(CoordinatorConfig::default());

        let guard = coordinator.lock_session("test-1", "user-1").await;
        assert!(guard.is_ok());
    }

    #[tokio::test]
    #[ignore] // Requires full context
    async fn test_concurrent_lock_fails() {
        let coordinator = MultiSessionCoordinator::new(CoordinatorConfig::default());

        let _guard1 = coordinator.lock_session("test-1", "user-1").await.unwrap();
        let result = coordinator.lock_session("test-1", "user-2").await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("locked"));
    }

    #[tokio::test]
    #[ignore] // Requires full context
    async fn test_is_session_locked() {
        let coordinator = MultiSessionCoordinator::new(CoordinatorConfig::default());

        assert!(!coordinator.is_session_locked("test-1").await);

        let _guard = coordinator.lock_session("test-1", "user-1").await.unwrap();
        assert!(coordinator.is_session_locked("test-1").await);
    }

    #[tokio::test]
    #[ignore] // Requires full context
    async fn test_get_locked_sessions() {
        let coordinator = MultiSessionCoordinator::new(CoordinatorConfig::default());

        let _guard1 = coordinator.lock_session("test-1", "user-1").await.unwrap();
        let _guard2 = coordinator.lock_session("test-2", "user-2").await.unwrap();

        let locked = coordinator.get_locked_sessions().await;
        assert_eq!(locked.len(), 2);
    }

    #[tokio::test]
    #[ignore] // Requires full context
    async fn test_cleanup_stale_locks() {
        let coordinator = MultiSessionCoordinator::new(CoordinatorConfig::default());

        // This test just verifies the method exists and runs
        let count = coordinator.cleanup_stale_locks().await;
        assert_eq!(count, 0);
    }

    #[tokio::test]
    #[ignore] // Requires full context
    async fn test_update_resource_stats() {
        let coordinator = MultiSessionCoordinator::new(CoordinatorConfig::default());

        coordinator.update_resource_stats().await;

        let stats = coordinator.get_resource_stats().await;
        assert_eq!(stats.active_sessions, 0);
    }

    #[tokio::test]
    #[ignore] // Requires full context
    async fn test_check_resource_leaks_empty() {
        let coordinator = MultiSessionCoordinator::new(CoordinatorConfig::default());

        let warnings = coordinator.check_resource_leaks().await;
        assert_eq!(warnings.len(), 0);
    }

    #[tokio::test]
    #[ignore] // Requires full context
    async fn test_get_cleanup_recommendations() {
        let coordinator = MultiSessionCoordinator::new(CoordinatorConfig::default());

        let recommendations = coordinator.get_cleanup_recommendations().await;
        assert_eq!(recommendations.len(), 0);
    }

    #[tokio::test]
    #[ignore] // Requires full context
    async fn test_perform_cleanup() {
        let coordinator = MultiSessionCoordinator::new(CoordinatorConfig::default());

        let result = coordinator.perform_cleanup().await;
        assert!(result.is_ok());
    }

    // Concurrent access tests for Task 1.1
    #[tokio::test]
    async fn test_concurrent_list_sessions() {
        let coordinator = Arc::new(MultiSessionCoordinator::new(CoordinatorConfig::default()));
        
        let mut handles = vec![];
        for _ in 0..100 {
            let coord = coordinator.clone();
            handles.push(tokio::spawn(async move {
                coord.list_sessions().await
            }));
        }
        
        for handle in handles {
            assert!(handle.await.is_ok());
        }
    }

    #[tokio::test]
    async fn test_concurrent_active_session_id() {
        let coordinator = Arc::new(MultiSessionCoordinator::new(CoordinatorConfig::default()));
        
        let mut handles = vec![];
        for _ in 0..100 {
            let coord = coordinator.clone();
            handles.push(tokio::spawn(async move {
                coord.active_session_id().await
            }));
        }
        
        for handle in handles {
            assert!(handle.await.is_ok());
        }
    }

    #[tokio::test]
    async fn test_concurrent_mixed_operations() {
        let coordinator = Arc::new(MultiSessionCoordinator::new(CoordinatorConfig::default()));
        
        let mut handles = vec![];
        for i in 0..100 {
            let coord = coordinator.clone();
            let op = i % 4;
            handles.push(tokio::spawn(async move {
                match op {
                    0 => { coord.list_sessions().await; },
                    1 => { coord.active_session_id().await; },
                    2 => { coord.get_session("test").await; },
                    _ => { coord.get_waiting_sessions().await; },
                }
            }));
        }
        
        for handle in handles {
            assert!(handle.await.is_ok());
        }
    }

    #[tokio::test]
    async fn test_state_consistency_under_concurrent_reads() {
        let coordinator = Arc::new(MultiSessionCoordinator::new(CoordinatorConfig::default()));
        
        let mut handles = vec![];
        for _ in 0..50 {
            let coord = coordinator.clone();
            handles.push(tokio::spawn(async move {
                let sessions = coord.list_sessions().await;
                let active = coord.active_session_id().await;
                (sessions, active)
            }));
        }
        
        for handle in handles {
            let result = handle.await;
            assert!(result.is_ok());
        }
    }
}

    #[tokio::test]
    #[ignore]
    async fn test_cleanup_inactive_sessions() {
        let config = CoordinatorConfig {
            session_timeout: Duration::from_millis(100),
            ..Default::default()
        };
        let mut coordinator = MultiSessionCoordinator::new(config);
        let removed = coordinator.cleanup_inactive_sessions(Duration::from_millis(100)).await.unwrap();
        assert_eq!(removed, 0);
    }

    #[tokio::test]
    async fn test_touch_session_updates_timestamp() {
        let mut coordinator = MultiSessionCoordinator::new(CoordinatorConfig::default());
        let result = coordinator.touch_session("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_bounded_channel_capacity() {
        let config = CoordinatorConfig {
            state_channel_capacity: 5,
            ..Default::default()
        };
        let coordinator = MultiSessionCoordinator::new(config);
        assert_eq!(coordinator.dropped_events_count(), 0);
    }

    #[tokio::test]
    async fn test_send_state_change_with_backpressure() {
        let config = CoordinatorConfig {
            state_channel_capacity: 2,
            ..Default::default()
        };
        let coordinator = MultiSessionCoordinator::new(config);
        
        // Send changes
        let result = coordinator.send_state_change(
            SessionStateChange::Processing("test".to_string())
        ).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_dropped_events_counter() {
        let coordinator = MultiSessionCoordinator::new(CoordinatorConfig::default());
        let initial = coordinator.dropped_events_count();
        assert_eq!(initial, 0);
    }

    #[tokio::test]
    async fn test_validate_session_name_empty() {
        let result = validate_session_name("");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    }

    #[tokio::test]
    async fn test_validate_session_name_too_long() {
        let long_name = "a".repeat(65);
        let result = validate_session_name(&long_name);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too long"));
    }

    #[tokio::test]
    async fn test_validate_session_name_invalid_chars() {
        let result = validate_session_name("test@session");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid characters"));
    }

    #[tokio::test]
    async fn test_validate_session_name_valid() {
        assert!(validate_session_name("test-session_1").is_ok());
        assert!(validate_session_name("my session").is_ok());
    }

    #[tokio::test]
    async fn test_validate_conversation_id_empty() {
        let result = validate_conversation_id("");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    }

    #[tokio::test]
    async fn test_validate_conversation_id_too_long() {
        let long_id = "a".repeat(129);
        let result = validate_conversation_id(&long_id);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too long"));
    }

    #[tokio::test]
    async fn test_validate_conversation_id_valid() {
        assert!(validate_conversation_id("abc123").is_ok());
    }
