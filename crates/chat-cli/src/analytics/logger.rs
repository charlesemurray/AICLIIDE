use std::fs::{
    File,
    OpenOptions,
};
use std::io::{
    BufWriter,
    Write,
};
use std::path::{
    Path,
    PathBuf,
};
use std::time::Instant;

use serde_json;
use uuid::Uuid;

use super::types::ConversationAnalyticsEvent;

const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024; // 10MB

#[derive(Debug)]
pub struct ConversationAnalytics {
    session_id: String,
    start_time: Instant,
    message_count: u32,
    writer: BufWriter<File>,
    file_path: PathBuf,
}

impl ConversationAnalytics {
    pub fn new(analytics_dir: &Path) -> Result<Self, std::io::Error> {
        // Create analytics directory if it doesn't exist
        std::fs::create_dir_all(analytics_dir)?;

        let file_path = analytics_dir.join("conversation_flow.jsonl");

        // Check if file exists and is too large
        if file_path.exists() {
            let metadata = std::fs::metadata(&file_path)?;
            if metadata.len() > MAX_FILE_SIZE {
                // Rotate the file
                let backup_path = analytics_dir.join(format!(
                    "conversation_flow_{}.jsonl",
                    chrono::Utc::now().format("%Y%m%d_%H%M%S")
                ));
                std::fs::rename(&file_path, backup_path)?;
            }
        }

        // Open file for appending
        let file = OpenOptions::new().create(true).append(true).open(&file_path)?;

        // Set file permissions on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = file.metadata()?.permissions();
            permissions.set_mode(0o600);
            file.set_permissions(permissions)?;
        }

        let writer = BufWriter::new(file);
        let session_id = Uuid::new_v4().to_string();

        Ok(Self {
            session_id,
            start_time: Instant::now(),
            message_count: 0,
            writer,
            file_path,
        })
    }

    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    pub fn file_path(&self) -> &Path {
        &self.file_path
    }

    pub fn log_event(&mut self, event: ConversationAnalyticsEvent) -> Result<(), std::io::Error> {
        let json_line = serde_json::to_string(&event)?;
        writeln!(self.writer, "{}", json_line)?;
        self.writer.flush()?;
        Ok(())
    }

    pub fn start_session(&mut self, _initial_request: &str) -> Result<(), std::io::Error> {
        use super::types::{
            AnalyticsEventType,
            SessionEventType,
        };

        let event = ConversationAnalyticsEvent::new(self.session_id.clone(), AnalyticsEventType::SessionFlow {
            event_type: SessionEventType::Started,
            at_message_count: 0,
            duration_ms: None,
        });

        self.log_event(event)?;
        self.message_count = 0;
        Ok(())
    }

    pub fn end_session(&mut self, completion_status: SessionCompletionStatus) -> Result<(), std::io::Error> {
        use super::types::{
            AnalyticsEventType,
            SessionEventType,
        };

        let duration_ms = self.start_time.elapsed().as_millis() as u64;
        let event_type = match completion_status {
            SessionCompletionStatus::Completed => SessionEventType::Completed,
            SessionCompletionStatus::Abandoned => SessionEventType::Abandoned,
        };

        let event = ConversationAnalyticsEvent::new(self.session_id.clone(), AnalyticsEventType::SessionFlow {
            event_type,
            at_message_count: self.message_count,
            duration_ms: Some(duration_ms),
        });

        self.log_event(event)?;
        Ok(())
    }

    pub fn increment_message_count(&mut self) {
        self.message_count += 1;
    }

    pub fn message_count(&self) -> u32 {
        self.message_count
    }
}

#[derive(Debug, Clone)]
pub enum SessionCompletionStatus {
    Completed,
    Abandoned,
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;

    #[test]
    fn test_new_creates_analytics_file() {
        let temp_dir = TempDir::new().unwrap();
        let analytics_dir = temp_dir.path();

        let analytics = ConversationAnalytics::new(analytics_dir).unwrap();

        assert!(!analytics.session_id().is_empty());
        assert_eq!(analytics.message_count(), 0);

        let file_path = analytics_dir.join("conversation_flow.jsonl");
        assert!(file_path.exists());
    }

    #[test]
    fn test_log_event_writes_json() {
        let temp_dir = TempDir::new().unwrap();
        let analytics_dir = temp_dir.path();

        let mut analytics = ConversationAnalytics::new(analytics_dir).unwrap();

        let event = ConversationAnalyticsEvent::continuation_prompt(
            analytics.session_id().to_string(),
            1,
            "test task".to_string(),
            Some(5),
        );

        analytics.log_event(event).unwrap();

        // Read the file and verify JSON was written
        let file_path = analytics_dir.join("conversation_flow.jsonl");
        let content = fs::read_to_string(file_path).unwrap();

        assert!(content.contains("ContinuationPrompt"));
        assert!(content.contains("test task"));
        assert!(content.ends_with('\n'));
    }

    #[test]
    fn test_start_session_logs_started_event() {
        let temp_dir = TempDir::new().unwrap();
        let analytics_dir = temp_dir.path();

        let mut analytics = ConversationAnalytics::new(analytics_dir).unwrap();
        analytics.start_session("initial request").unwrap();

        let file_path = analytics_dir.join("conversation_flow.jsonl");
        let content = fs::read_to_string(file_path).unwrap();

        assert!(content.contains("SessionFlow"));
        assert!(content.contains("Started"));
    }

    #[test]
    fn test_end_session_logs_completion_event() {
        let temp_dir = TempDir::new().unwrap();
        let analytics_dir = temp_dir.path();

        let mut analytics = ConversationAnalytics::new(analytics_dir).unwrap();
        analytics.increment_message_count();
        analytics.increment_message_count();

        analytics.end_session(SessionCompletionStatus::Completed).unwrap();

        let file_path = analytics_dir.join("conversation_flow.jsonl");
        let content = fs::read_to_string(file_path).unwrap();

        assert!(content.contains("SessionFlow"));
        assert!(content.contains("Completed"));
        assert!(content.contains("\"at_message_count\":2"));
        assert!(content.contains("duration_ms"));
    }

    #[test]
    fn test_file_rotation_when_too_large() {
        let temp_dir = TempDir::new().unwrap();
        let analytics_dir = temp_dir.path();
        let file_path = analytics_dir.join("conversation_flow.jsonl");

        // Create analytics directory
        fs::create_dir_all(analytics_dir).unwrap();

        // Create a large file (larger than MAX_FILE_SIZE)
        let large_content = "x".repeat((MAX_FILE_SIZE + 1) as usize);
        fs::write(&file_path, large_content).unwrap();

        // Creating new analytics should rotate the file
        let _analytics = ConversationAnalytics::new(analytics_dir).unwrap();

        // Original file should be smaller now (new empty file)
        let new_size = fs::metadata(&file_path).unwrap().len();
        assert_eq!(new_size, 0);

        // Should have a backup file
        let backup_files: Vec<_> = fs::read_dir(analytics_dir)
            .unwrap()
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("conversation_flow_") && name.ends_with(".jsonl") {
                    Some(name)
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(backup_files.len(), 1);
    }

    #[test]
    fn test_message_count_tracking() {
        let temp_dir = TempDir::new().unwrap();
        let analytics_dir = temp_dir.path();

        let mut analytics = ConversationAnalytics::new(analytics_dir).unwrap();

        assert_eq!(analytics.message_count(), 0);

        analytics.increment_message_count();
        assert_eq!(analytics.message_count(), 1);

        analytics.increment_message_count();
        analytics.increment_message_count();
        assert_eq!(analytics.message_count(), 3);
    }

    #[test]
    fn test_session_id_consistency() {
        let temp_dir = TempDir::new().unwrap();
        let analytics_dir = temp_dir.path();

        let analytics = ConversationAnalytics::new(analytics_dir).unwrap();
        let session_id1 = analytics.session_id().to_string();
        let session_id2 = analytics.session_id().to_string();

        assert_eq!(session_id1, session_id2);

        // Different instances should have different session IDs
        let analytics2 = ConversationAnalytics::new(analytics_dir).unwrap();
        let session_id3 = analytics2.session_id().to_string();

        assert_ne!(session_id1, session_id3);
    }
}
