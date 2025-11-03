use std::path::PathBuf;
use tokio::sync::OnceCell;
use super::error::SessionError;
use super::metadata::SessionMetadata;

/// Lazy-loading session preview that computes expensive data on-demand
#[derive(Debug)]
pub struct SessionPreview {
    pub metadata: SessionMetadata,
    session_path: PathBuf,
    
    // Lazy-loaded fields
    last_message: OnceCell<String>,
    file_count: OnceCell<usize>,
    workspace_size: OnceCell<u64>,
}

impl SessionPreview {
    pub fn new(metadata: SessionMetadata, session_path: PathBuf) -> Self {
        Self {
            metadata,
            session_path,
            last_message: OnceCell::new(),
            file_count: OnceCell::new(),
            workspace_size: OnceCell::new(),
        }
    }

    /// Get last message (lazy-loaded)
    pub async fn last_message(&self) -> Result<&str, SessionError> {
        self.last_message.get_or_try_init(|| async {
            let conversation_file = self.session_path.join("conversation.json");
            if !conversation_file.exists() {
                return Ok("No conversation".to_string());
            }
            
            // Read only the last few KB to get last message
            let content = tokio::fs::read_to_string(&conversation_file).await?;
            let last_line = content.lines().last().unwrap_or("Empty conversation");
            Ok(last_line.chars().take(100).collect::<String>())
        }).await.map(|s| s.as_str())
    }

    /// Get file count (lazy-loaded)
    pub async fn file_count(&self) -> Result<usize, SessionError> {
        self.file_count.get_or_try_init(|| async {
            let files_dir = self.session_path.join("files");
            if !files_dir.exists() {
                return Ok(0);
            }
            
            let mut count = 0;
            let mut entries = tokio::fs::read_dir(&files_dir).await?;
            while entries.next_entry().await?.is_some() {
                count += 1;
            }
            Ok(count)
        }).await.copied()
    }

    /// Get workspace size (lazy-loaded)
    pub async fn workspace_size(&self) -> Result<u64, SessionError> {
        self.workspace_size.get_or_try_init(|| async {
            calculate_dir_size(&self.session_path).await
        }).await.copied()
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
