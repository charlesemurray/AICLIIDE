//! File-based logging for priority limiter using tracing

use once_cell::sync::Lazy;
use tracing_subscriber::fmt::writer::MakeWriterExt;

/// Initialize file logging for priority limiter
/// Logs go to ~/.amazonq/logs/priority_limiter.log
pub static PRIORITY_LOGGER_INIT: Lazy<()> = Lazy::new(|| {
    let log_dir = dirs::home_dir()
        .map(|home| home.join(".amazonq").join("logs"))
        .unwrap_or_else(|| std::path::PathBuf::from("logs"));
    
    // Create directory
    let _ = std::fs::create_dir_all(&log_dir);
    
    // Create file appender (rotates daily)
    let file_appender = tracing_appender::rolling::daily(&log_dir, "priority_limiter.log");
    
    // Create subscriber that writes to file
    let subscriber = tracing_subscriber::fmt()
        .with_writer(file_appender.with_max_level(tracing::Level::DEBUG))
        .with_ansi(false)
        .with_target(false)
        .finish();
    
    // Set as global default
    let _ = tracing::subscriber::set_global_default(subscriber);
    
    eprintln!("[PRIORITY] Logging to: {}/priority_limiter.log", log_dir.display());
});

/// Ensure logger is initialized
pub fn init() {
    Lazy::force(&PRIORITY_LOGGER_INIT);
}
