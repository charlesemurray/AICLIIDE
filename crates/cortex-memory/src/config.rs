pub struct MemoryConfig {
    pub enabled: bool,
    pub retention_days: u32,
    pub max_size_mb: u32,
    pub cross_session: bool,
    pub auto_promote: bool,
    pub warn_threshold: u8,
}

impl MemoryConfig {
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    
    pub fn with_retention_days(mut self, days: u32) -> Self {
        self.retention_days = days;
        self
    }
    
    pub fn with_max_size_mb(mut self, mb: u32) -> Self {
        self.max_size_mb = mb;
        self
    }
    
    pub fn with_cross_session(mut self, cross_session: bool) -> Self {
        self.cross_session = cross_session;
        self
    }
    
    pub fn with_auto_promote(mut self, auto_promote: bool) -> Self {
        self.auto_promote = auto_promote;
        self
    }
    
    pub fn with_warn_threshold(mut self, threshold: u8) -> Self {
        self.warn_threshold = threshold;
        self
    }
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            retention_days: 30,
            max_size_mb: 100,
            cross_session: false,
            auto_promote: true,
            warn_threshold: 80,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = MemoryConfig::default();
        
        assert_eq!(config.enabled, true);
        assert_eq!(config.retention_days, 30);
        assert_eq!(config.max_size_mb, 100);
        assert_eq!(config.cross_session, false);
        assert_eq!(config.auto_promote, true);
        assert_eq!(config.warn_threshold, 80);
    }
    
    #[test]
    fn test_config_builder() {
        let config = MemoryConfig::default()
            .with_enabled(false)
            .with_retention_days(60)
            .with_max_size_mb(200)
            .with_cross_session(true)
            .with_auto_promote(false)
            .with_warn_threshold(90);
        
        assert_eq!(config.enabled, false);
        assert_eq!(config.retention_days, 60);
        assert_eq!(config.max_size_mb, 200);
        assert_eq!(config.cross_session, true);
        assert_eq!(config.auto_promote, false);
        assert_eq!(config.warn_threshold, 90);
    }
}
