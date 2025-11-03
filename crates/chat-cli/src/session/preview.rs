use std::path::PathBuf;
use tokio::sync::OnceCell;
use tracing::warn;
use super::error::SessionError;
use super::metadata::SessionMetadata;

/// Fast approximation data computed upfront
#[derive(Debug, Clone)]
pub struct SessionApproximation {
    pub estimated_size: u64,      // From metadata file size * 10
    pub estimated_files: usize,   // From directory entry count (no recursion)
    pub has_conversation: bool,   // Just check if file exists
}

/// Lazy-loading session preview with smart precomputation
#[derive(Debug)]
pub struct SessionPreview {
    pub metadata: SessionMetadata,
    pub approximation: SessionApproximation,  // Fast data for sorting/filtering
    session_path: PathBuf,
    
    // Lazy-loaded precise fields
    last_message: OnceCell<String>,
    precise_file_count: OnceCell<usize>,
    precise_workspace_size: OnceCell<u64>,
}

impl SessionPreview {
    pub fn new(metadata: SessionMetadata, session_path: PathBuf) -> Result<Self, SessionError> {
        // Compute fast approximations upfront (still O(1) per session)
        let approximation = Self::compute_approximation(&session_path)?;
        
        Ok(Self {
            metadata,
            approximation,
            session_path,
            last_message: OnceCell::new(),
            precise_file_count: OnceCell::new(),
            precise_workspace_size: OnceCell::new(),
        })
    }

    /// Fast approximation - O(1) operation
    fn compute_approximation(path: &PathBuf) -> Result<SessionApproximation, SessionError> {
        let metadata_size = std::fs::metadata(path.join("metadata.json"))
            .map(|m| m.len())
            .unwrap_or(0);
        
        let files_dir = path.join("files");
        let estimated_files = if files_dir.exists() {
            std::fs::read_dir(&files_dir)?.count()  // Just count entries, no recursion
        } else { 0 };
        
        let has_conversation = path.join("conversation.json").exists();
        
        Ok(SessionApproximation {
            estimated_size: metadata_size * 10,  // Rough estimate
            estimated_files,
            has_conversation,
        })
    }

    /// Get sortable size (fast approximation or cached precise)
    pub async fn sortable_size(&self) -> u64 {
        // Use precise if already loaded, otherwise approximation
        if let Some(precise) = self.precise_workspace_size.get() {
            *precise
        } else {
            self.approximation.estimated_size
        }
    }

    /// Get searchable content preview (fast)
    pub async fn searchable_preview(&self) -> Result<String, SessionError> {
        // Return first 500 chars of conversation for search
        if !self.approximation.has_conversation {
            return Ok(String::new());
        }
        
        let conversation_file = self.session_path.join("conversation.json");
        let content = tokio::fs::read_to_string(&conversation_file).await?;
        Ok(content.chars().take(500).collect())
    }

    /// Get precise workspace size (expensive, lazy-loaded)
    pub async fn precise_workspace_size(&self) -> Result<u64, SessionError> {
        self.precise_workspace_size.get_or_try_init(|| async {
            calculate_dir_size(&self.session_path).await
        }).await.copied()
    }

    /// Get last message (lazy-loaded)
    pub async fn last_message(&self) -> Result<&str, SessionError> {
        self.last_message.get_or_try_init(|| async {
            let conversation_file = self.session_path.join("conversation.json");
            if !conversation_file.exists() {
                return Ok("No conversation".to_string());
            }
            
            let content = tokio::fs::read_to_string(&conversation_file).await?;
            let last_line = content.lines().last().unwrap_or("Empty conversation");
            Ok(last_line.chars().take(100).collect::<String>())
        }).await.map(|s| s.as_str())
    }
}

fn calculate_dir_size(path: &PathBuf) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<u64, SessionError>> + Send + '_>> {
    Box::pin(async move {
        let mut total = 0;
        let mut entries = tokio::fs::read_dir(path).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let metadata = entry.metadata().await?;
            if metadata.is_file() {
                total += metadata.len();
            } else if metadata.is_dir() {
                total += calculate_dir_size(&entry.path()).await?;
            }
        }
        
        Ok(total)
    })
}
