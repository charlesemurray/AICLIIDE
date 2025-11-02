// Temporary stub for analytics module
// TODO: Replace with actual implementation from analytics session

use std::path::Path;

#[derive(Debug, Clone)]
pub struct ConversationAnalytics;

impl ConversationAnalytics {
    pub fn new(_path: &Path) -> Self {
        Self
    }
}

#[derive(Debug, Clone)]
pub enum ConversationAnalyticsEvent {
    // Placeholder
}
