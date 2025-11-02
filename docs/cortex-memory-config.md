# Cortex Memory - Configuration Design

## Q CLI Settings System

### Existing Infrastructure

**Settings Storage**: `~/.q/config/settings.json`

**Settings Enum**: `crates/chat-cli/src/database/settings.rs`
- Centralized enum for all settings
- Type-safe access via `Setting` enum
- Stored as JSON with dot-notation keys
- Supports: boolean, string, number, array

**Usage Pattern**:
```rust
// Get setting
let enabled = os.database.settings.get(Setting::EnabledKnowledge)
    .and_then(|v| v.as_bool())
    .unwrap_or(false);

// Set setting
os.database.settings.set(Setting::EnabledKnowledge, Value::Bool(true))?;
```

---

## Memory Configuration Settings

### New Settings to Add

Add to `Setting` enum in `database/settings.rs`:

```rust
#[derive(Clone, Copy, Debug, strum::EnumIter, strum::EnumMessage)]
pub enum Setting {
    // ... existing settings ...
    
    #[strum(message = "Enable memory system (boolean)")]
    MemoryEnabled,
    
    #[strum(message = "Memory retention period in days (number, 0 = unlimited)")]
    MemoryRetentionDays,
    
    #[strum(message = "Maximum memory storage size in MB (number)")]
    MemoryMaxSizeMb,
    
    #[strum(message = "Enable cross-session memory recall (boolean)")]
    MemoryCrossSession,
    
    #[strum(message = "Auto-promote memories to long-term storage (boolean)")]
    MemoryAutoPromote,
    
    #[strum(message = "Warn when memory usage reaches percentage (number, 0-100)")]
    MemoryWarnThreshold,
}

impl AsRef<str> for Setting {
    fn as_ref(&self) -> &'static str {
        match self {
            // ... existing mappings ...
            Self::MemoryEnabled => "memory.enabled",
            Self::MemoryRetentionDays => "memory.retentionDays",
            Self::MemoryMaxSizeMb => "memory.maxSizeMb",
            Self::MemoryCrossSession => "memory.crossSession",
            Self::MemoryAutoPromote => "memory.autoPromote",
            Self::MemoryWarnThreshold => "memory.warnThreshold",
        }
    }
}
```

### Default Values

**In `~/.q/config/settings.json`**:
```json
{
  "memory.enabled": true,
  "memory.retentionDays": 30,
  "memory.maxSizeMb": 100,
  "memory.crossSession": false,
  "memory.autoPromote": true,
  "memory.warnThreshold": 80
}
```

---

## Configuration API

### Reading Configuration

```rust
// In CortexMemory initialization
pub struct MemoryConfig {
    pub enabled: bool,
    pub retention_days: u32,
    pub max_size_mb: u32,
    pub cross_session: bool,
    pub auto_promote: bool,
    pub warn_threshold: u8,
}

impl MemoryConfig {
    pub fn from_settings(settings: &Settings) -> Self {
        Self {
            enabled: settings.get(Setting::MemoryEnabled)
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
            retention_days: settings.get(Setting::MemoryRetentionDays)
                .and_then(|v| v.as_u64())
                .unwrap_or(30) as u32,
            max_size_mb: settings.get(Setting::MemoryMaxSizeMb)
                .and_then(|v| v.as_u64())
                .unwrap_or(100) as u32,
            cross_session: settings.get(Setting::MemoryCrossSession)
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            auto_promote: settings.get(Setting::MemoryAutoPromote)
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
            warn_threshold: settings.get(Setting::MemoryWarnThreshold)
                .and_then(|v| v.as_u64())
                .unwrap_or(80) as u8,
        }
    }
}
```

### Setting Configuration

**Via `q settings` command**:
```bash
# Enable/disable memory
$ q settings set memory.enabled true
$ q settings set memory.enabled false

# Set retention period
$ q settings set memory.retentionDays 90
$ q settings set memory.retentionDays 0  # Unlimited

# Set storage limit
$ q settings set memory.maxSizeMb 200

# Enable cross-session recall by default
$ q settings set memory.crossSession true

# Disable auto-promotion
$ q settings set memory.autoPromote false

# Set warning threshold
$ q settings set memory.warnThreshold 90
```

**Via `q memory config` command** (convenience wrapper):
```bash
# Show current config
$ q memory config
Memory Configuration:
  Enabled: true
  Retention: 30 days
  Max Size: 100 MB
  Cross-Session: false
  Auto-Promote: true
  Warn Threshold: 80%

# Set retention
$ q memory config --retention 90d
$ q memory config --retention unlimited

# Set max size
$ q memory config --max-size 200MB

# Enable cross-session
$ q memory config --cross-session

# Disable memory
$ q memory config --disable
```

---

## Hybrid Retention Logic

### Implementation

```rust
pub struct MemoryRetentionManager {
    config: MemoryConfig,
}

impl MemoryRetentionManager {
    pub fn should_cleanup(&self, db_path: &Path) -> Result<bool> {
        // Check time-based retention
        let time_exceeded = if self.config.retention_days > 0 {
            self.has_old_memories(db_path)?
        } else {
            false
        };
        
        // Check size-based retention
        let size_exceeded = self.get_db_size_mb(db_path)? >= self.config.max_size_mb;
        
        Ok(time_exceeded || size_exceeded)
    }
    
    pub fn cleanup_old_memories(&self, cortex: &mut CortexMemory) -> Result<usize> {
        let cutoff_date = if self.config.retention_days > 0 {
            Some(Utc::now() - Duration::days(self.config.retention_days as i64))
        } else {
            None
        };
        
        let current_size_mb = self.get_db_size_mb(&cortex.db_path)?;
        
        // Delete oldest memories until both constraints satisfied
        let mut deleted = 0;
        
        // Time-based cleanup
        if let Some(cutoff) = cutoff_date {
            deleted += cortex.delete_before(cutoff)?;
        }
        
        // Size-based cleanup (if still over limit)
        while self.get_db_size_mb(&cortex.db_path)? >= self.config.max_size_mb {
            let oldest = cortex.get_oldest_memory()?;
            if let Some(memory) = oldest {
                cortex.delete(&memory.id)?;
                deleted += 1;
            } else {
                break;
            }
        }
        
        Ok(deleted)
    }
    
    pub fn should_warn(&self, db_path: &Path) -> Result<bool> {
        let current_size_mb = self.get_db_size_mb(db_path)?;
        let threshold_size = (self.config.max_size_mb as f32 
            * self.config.warn_threshold as f32 / 100.0) as u32;
        
        Ok(current_size_mb >= threshold_size)
    }
}
```

### Cleanup Triggers

**Automatic cleanup**:
1. On Q CLI startup (background check)
2. After each memory store (if threshold exceeded)
3. Daily background task

**Manual cleanup**:
```bash
$ q memory cleanup
Cleaning up old memories...
- Deleted 127 memories older than 30 days
- Freed 15.3 MB of storage
Current usage: 45.2 MB / 100 MB (45%)
```

**Warning display**:
```bash
You: How do I deploy to Lambda?

⚠️  Memory storage at 85 MB / 100 MB (85%)
    Run 'q memory cleanup' or adjust limits with 'q memory config'

Q: Here's how to deploy to Lambda...
```

---

## Configuration File Format

**`~/.q/config/settings.json`**:
```json
{
  "telemetry.enabled": true,
  "chat.enableKnowledge": true,
  "memory.enabled": true,
  "memory.retentionDays": 30,
  "memory.maxSizeMb": 100,
  "memory.crossSession": false,
  "memory.autoPromote": true,
  "memory.warnThreshold": 80
}
```

**Validation**:
- `retentionDays`: 0 (unlimited) or 1-365
- `maxSizeMb`: 10-1000 MB
- `warnThreshold`: 50-100%

---

## Migration Strategy

### First Run

**If no memory settings exist**:
```rust
// On first run, initialize with defaults
if !settings.has(Setting::MemoryEnabled) {
    settings.set(Setting::MemoryEnabled, Value::Bool(true))?;
    settings.set(Setting::MemoryRetentionDays, Value::Number(30.into()))?;
    settings.set(Setting::MemoryMaxSizeMb, Value::Number(100.into()))?;
    settings.set(Setting::MemoryCrossSession, Value::Bool(false))?;
    settings.set(Setting::MemoryAutoPromote, Value::Bool(true))?;
    settings.set(Setting::MemoryWarnThreshold, Value::Number(80.into()))?;
}
```

### Upgrade Path

**If user has existing Q CLI installation**:
- Settings file is updated with new memory.* keys
- Defaults applied automatically
- No user action required
- Existing settings preserved

---

## Concrete Integration Steps

### Step 1: Add Settings to Enum

**File**: `crates/chat-cli/src/database/settings.rs`

```rust
#[derive(Clone, Copy, Debug, strum::EnumIter, strum::EnumMessage)]
pub enum Setting {
    // ... existing settings ...
    
    #[strum(message = "Enable memory system (boolean)")]
    MemoryEnabled,
    
    #[strum(message = "Memory retention period in days (number, 0 = unlimited)")]
    MemoryRetentionDays,
    
    #[strum(message = "Maximum memory storage size in MB (number)")]
    MemoryMaxSizeMb,
    
    #[strum(message = "Enable cross-session memory recall (boolean)")]
    MemoryCrossSession,
    
    #[strum(message = "Auto-promote memories to long-term storage (boolean)")]
    MemoryAutoPromote,
    
    #[strum(message = "Warn when memory usage reaches percentage (number, 0-100)")]
    MemoryWarnThreshold,
}
```

### Step 2: Add String Mappings

**In same file**, add to `AsRef<str>` impl:

```rust
impl AsRef<str> for Setting {
    fn as_ref(&self) -> &'static str {
        match self {
            // ... existing mappings ...
            Self::MemoryEnabled => "memory.enabled",
            Self::MemoryRetentionDays => "memory.retentionDays",
            Self::MemoryMaxSizeMb => "memory.maxSizeMb",
            Self::MemoryCrossSession => "memory.crossSession",
            Self::MemoryAutoPromote => "memory.autoPromote",
            Self::MemoryWarnThreshold => "memory.warnThreshold",
        }
    }
}
```

### Step 3: Add Reverse Mappings

**In same file**, add to `TryFrom<&str>` impl:

```rust
impl TryFrom<&str> for Setting {
    type Error = DatabaseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            // ... existing mappings ...
            "memory.enabled" => Ok(Self::MemoryEnabled),
            "memory.retentionDays" => Ok(Self::MemoryRetentionDays),
            "memory.maxSizeMb" => Ok(Self::MemoryMaxSizeMb),
            "memory.crossSession" => Ok(Self::MemoryCrossSession),
            "memory.autoPromote" => Ok(Self::MemoryAutoPromote),
            "memory.warnThreshold" => Ok(Self::MemoryWarnThreshold),
            _ => Err(DatabaseError::InvalidSetting(value.to_string())),
        }
    }
}
```

### Step 4: Use in Cortex Code

**File**: `crates/cortex-memory/src/config.rs` (new file)

```rust
use serde_json::Value;

pub struct MemoryConfig {
    pub enabled: bool,
    pub retention_days: u32,
    pub max_size_mb: u32,
    pub cross_session: bool,
    pub auto_promote: bool,
    pub warn_threshold: u8,
}

impl MemoryConfig {
    /// Load from Q CLI's settings system
    pub fn from_q_settings(settings: &chat_cli::database::settings::Settings) -> Self {
        use chat_cli::database::settings::Setting;
        
        Self {
            enabled: settings.get(Setting::MemoryEnabled)
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
            retention_days: settings.get(Setting::MemoryRetentionDays)
                .and_then(|v| v.as_u64())
                .unwrap_or(30) as u32,
            max_size_mb: settings.get(Setting::MemoryMaxSizeMb)
                .and_then(|v| v.as_u64())
                .unwrap_or(100) as u32,
            cross_session: settings.get(Setting::MemoryCrossSession)
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            auto_promote: settings.get(Setting::MemoryAutoPromote)
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
            warn_threshold: settings.get(Setting::MemoryWarnThreshold)
                .and_then(|v| v.as_u64())
                .unwrap_or(80) as u8,
        }
    }
    
    /// Get default configuration
    pub fn default() -> Self {
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
```

### Step 5: Initialize in Chat Session

**File**: `crates/chat-cli/src/cli/chat/mod.rs` (or wherever ChatSession is)

```rust
use cortex_memory::{CortexMemory, MemoryConfig};

impl ChatSession {
    pub fn new(os: &Os) -> Result<Self> {
        // Load memory config from Q CLI settings
        let memory_config = MemoryConfig::from_q_settings(&os.database.settings);
        
        // Initialize Cortex if enabled
        let cortex = if memory_config.enabled {
            let db_path = os.paths.global().memory_dir()?.join("cortex.db");
            Some(CortexMemory::new(db_path, memory_config)?)
        } else {
            None
        };
        
        Ok(Self {
            cortex,
            // ... other fields
        })
    }
}
```

### Step 6: Add In-Chat Commands

**File**: `crates/chat-cli/src/cli/chat/cli/mod.rs`

Add to `SlashCommand` enum:

```rust
#[derive(Debug, PartialEq, Parser)]
pub enum SlashCommand {
    // ... existing commands ...
    
    /// Recall memories from current or other sessions
    Recall {
        /// Search query
        query: String,
        
        /// Search all sessions instead of current only
        #[arg(long, short)]
        global: bool,
        
        /// Search specific session by ID or name
        #[arg(long, short)]
        session: Option<String>,
        
        /// List sessions with memories
        #[arg(long)]
        list_sessions: bool,
        
        /// Maximum results to return
        #[arg(long, short, default_value = "5")]
        limit: usize,
    },
    
    /// Manage memory settings and data
    #[command(subcommand)]
    Memory(MemorySubcommand),
}
```

**File**: `crates/chat-cli/src/cli/chat/cli/memory.rs` (new file)

```rust
use clap::Subcommand;
use crate::cli::chat::{ChatError, ChatSession, ChatState};
use crate::os::Os;
use crate::database::settings::Setting;
use serde_json::Value;

#[derive(Debug, PartialEq, Subcommand)]
pub enum MemorySubcommand {
    /// Show current memory configuration
    Config,
    
    /// Set memory configuration
    Set {
        #[command(subcommand)]
        setting: MemorySetting,
    },
    
    /// List recent memories
    List {
        /// Maximum memories to show
        #[arg(long, short, default_value = "10")]
        limit: usize,
        
        /// Filter by session ID
        #[arg(long, short)]
        session: Option<String>,
    },
    
    /// Search memories
    Search {
        /// Search query
        query: String,
        
        /// Maximum results
        #[arg(long, short, default_value = "5")]
        limit: usize,
    },
    
    /// Show memory statistics
    Stats {
        /// Show per-session breakdown
        #[arg(long)]
        by_session: bool,
    },
    
    /// Clean up old memories
    Cleanup {
        /// Force cleanup without confirmation
        #[arg(long, short)]
        force: bool,
    },
    
    /// Enable or disable memory
    Toggle {
        /// Disable memory
        #[arg(long)]
        disable: bool,
    },
}

#[derive(Debug, PartialEq, Subcommand)]
pub enum MemorySetting {
    /// Set retention period in days (0 = unlimited)
    Retention { days: u32 },
    
    /// Set maximum storage size in MB
    MaxSize { mb: u32 },
    
    /// Enable/disable cross-session recall
    CrossSession { 
        #[arg(long)]
        disable: bool 
    },
}

impl MemorySubcommand {
    pub async fn execute(
        self,
        os: &Os,
        session: &mut ChatSession,
    ) -> Result<ChatState, ChatError> {
        match self {
            Self::Config => {
                let config = cortex_memory::MemoryConfig::from_q_settings(&os.database.settings);
                println!("\n📊 Memory Configuration:");
                println!("  Enabled: {}", if config.enabled { "✓" } else { "✗" });
                println!("  Retention: {} days", config.retention_days);
                println!("  Max Size: {} MB", config.max_size_mb);
                println!("  Cross-Session: {}", if config.cross_session { "✓" } else { "✗" });
                println!("  Auto-Promote: {}", if config.auto_promote { "✓" } else { "✗" });
                println!("  Warn Threshold: {}%\n", config.warn_threshold);
            }
            Self::Set { setting } => {
                match setting {
                    MemorySetting::Retention { days } => {
                        os.database.settings.set(
                            Setting::MemoryRetentionDays,
                            Value::Number(days.into())
                        )?;
                        println!("✓ Memory retention set to {} days", days);
                    }
                    MemorySetting::MaxSize { mb } => {
                        os.database.settings.set(
                            Setting::MemoryMaxSizeMb,
                            Value::Number(mb.into())
                        )?;
                        println!("✓ Memory max size set to {} MB", mb);
                    }
                    MemorySetting::CrossSession { disable } => {
                        os.database.settings.set(
                            Setting::MemoryCrossSession,
                            Value::Bool(!disable)
                        )?;
                        if disable {
                            println!("✓ Cross-session recall disabled");
                        } else {
                            println!("✓ Cross-session recall enabled");
                        }
                    }
                }
            }
            Self::List { limit, session } => {
                // Implementation in Phase 2
                println!("📝 Recent memories (limit: {}):", limit);
                if let Some(sid) = session {
                    println!("   Filtered by session: {}", sid);
                }
                // TODO: List memories from cortex
            }
            Self::Search { query, limit } => {
                // Implementation in Phase 2
                println!("🔍 Searching for: \"{}\" (limit: {})", query, limit);
                // TODO: Search memories from cortex
            }
            Self::Stats { by_session } => {
                // Implementation in Phase 2
                println!("📊 Memory Statistics:");
                if by_session {
                    println!("   (per-session breakdown)");
                }
                // TODO: Get stats from cortex
            }
            Self::Cleanup { force } => {
                if !force {
                    println!("⚠️  This will delete old memories. Use --force to confirm.");
                } else {
                    println!("🧹 Cleaning up old memories...");
                    // TODO: Cleanup from cortex
                }
            }
            Self::Toggle { disable } => {
                os.database.settings.set(
                    Setting::MemoryEnabled,
                    Value::Bool(!disable)
                )?;
                if disable {
                    println!("✓ Memory disabled");
                } else {
                    println!("✓ Memory enabled");
                }
            }
        }
        
        Ok(ChatState::PromptUser {
            skip_printing_tools: true,
        })
    }
}
```

### In-Chat Usage Examples

**View configuration**:
```bash
You: /memory config
📊 Memory Configuration:
  Enabled: ✓
  Retention: 30 days
  Max Size: 100 MB
  Cross-Session: ✗
  Auto-Promote: ✓
  Warn Threshold: 80%
```

**Change settings**:
```bash
You: /memory set retention 90
✓ Memory retention set to 90 days

You: /memory set max-size 200
✓ Memory max size set to 200 MB

You: /memory set cross-session
✓ Cross-session recall enabled
```

**Recall memories**:
```bash
You: /recall Lambda deployment
[searches current session]

You: /recall --global Lambda deployment
[searches all sessions]

You: /recall --session abc123 that bug fix
[searches specific session]

You: /recall --list-sessions
Sessions with memories:
1. session-abc123 (Today) - "AWS Lambda deployment"
2. session-xyz789 (Yesterday) - "React patterns"
```

**List and search**:
```bash
You: /memory list
📝 Recent memories (limit: 10):
1. [2 min ago] Discussion about Rust structs
2. [5 min ago] AWS Lambda deployment

You: /memory search "rust structs"
🔍 Searching for: "rust structs"
Found 3 relevant memories...
```

**Statistics and cleanup**:
```bash
You: /memory stats
📊 Memory Statistics:
- Total memories: 1,247
- Storage: 45.2 MB / 100 MB (45%)

You: /memory cleanup --force
🧹 Cleaning up old memories...
Deleted 127 memories older than 30 days
```

**Toggle memory**:
```bash
You: /memory toggle --disable
✓ Memory disabled

You: /memory toggle
✓ Memory enabled
```

---

## Command Availability

**All commands work in chat**:
- ✅ `/memory config` - View settings
- ✅ `/memory set <setting>` - Change settings
- ✅ `/memory list` - List memories
- ✅ `/memory search <query>` - Search memories
- ✅ `/memory stats` - View statistics
- ✅ `/memory cleanup` - Clean old memories
- ✅ `/memory toggle` - Enable/disable
- ✅ `/recall <query>` - Quick recall with options

**No need to exit chat** - all configuration and management happens in-chat with slash commands.

---

## Implementation Priority

**Phase 1** (Core):
- ✅ `/memory config` - View settings
- ✅ `/memory set` - Change settings
- ✅ `/memory toggle` - Enable/disable
- ✅ `/recall` - Basic recall

**Phase 2** (Management):
- `/memory list` - List memories
- `/memory search` - Search memories
- `/memory stats` - Statistics
- `/memory cleanup` - Cleanup

**Phase 3** (Advanced):
- `/recall --list-sessions` - Session discovery
- `/recall --session <id>` - Session-specific recall
- `/memory stats --by-session` - Per-session stats

---

```rust
#[derive(Debug, Subcommand)]
pub enum SettingsSubcommand {
    // ... existing subcommands ...
    
    /// Configure memory settings
    Memory {
        #[command(subcommand)]
        command: MemorySettingsCommand,
    },
}

#[derive(Debug, Subcommand)]
pub enum MemorySettingsCommand {
    /// Show current memory configuration
    Show,
    
    /// Set retention period
    Retention {
        /// Days to retain (0 = unlimited)
        days: u32,
    },
    
    /// Set maximum storage size
    MaxSize {
        /// Size in MB
        mb: u32,
    },
    
    /// Enable/disable memory
    Enable {
        #[arg(long)]
        disable: bool,
    },
    
    /// Enable/disable cross-session recall
    CrossSession {
        #[arg(long)]
        disable: bool,
    },
}

impl MemorySettingsCommand {
    pub fn execute(&self, os: &Os) -> Result<()> {
        use crate::database::settings::Setting;
        
        match self {
            Self::Show => {
                let config = MemoryConfig::from_q_settings(&os.database.settings);
                println!("Memory Configuration:");
                println!("  Enabled: {}", config.enabled);
                println!("  Retention: {} days", config.retention_days);
                println!("  Max Size: {} MB", config.max_size_mb);
                println!("  Cross-Session: {}", config.cross_session);
                println!("  Auto-Promote: {}", config.auto_promote);
                println!("  Warn Threshold: {}%", config.warn_threshold);
            }
            Self::Retention { days } => {
                os.database.settings.set(
                    Setting::MemoryRetentionDays,
                    Value::Number((*days).into())
                )?;
                println!("✓ Memory retention set to {} days", days);
            }
            Self::MaxSize { mb } => {
                os.database.settings.set(
                    Setting::MemoryMaxSizeMb,
                    Value::Number((*mb).into())
                )?;
                println!("✓ Memory max size set to {} MB", mb);
            }
            Self::Enable { disable } => {
                os.database.settings.set(
                    Setting::MemoryEnabled,
                    Value::Bool(!disable)
                )?;
                if *disable {
                    println!("✓ Memory disabled");
                } else {
                    println!("✓ Memory enabled");
                }
            }
            Self::CrossSession { disable } => {
                os.database.settings.set(
                    Setting::MemoryCrossSession,
                    Value::Bool(!disable)
                )?;
                if *disable {
                    println!("✓ Cross-session recall disabled");
                } else {
                    println!("✓ Cross-session recall enabled");
                }
            }
        }
        Ok(())
    }
}
```

### Step 7: Usage Examples

**Reading settings in Cortex code**:
```rust
// In any function that has access to Os
fn should_store_memory(os: &Os) -> bool {
    os.database.settings
        .get(Setting::MemoryEnabled)
        .and_then(|v| v.as_bool())
        .unwrap_or(true)
}

fn get_retention_days(os: &Os) -> u32 {
    os.database.settings
        .get(Setting::MemoryRetentionDays)
        .and_then(|v| v.as_u64())
        .unwrap_or(30) as u32
}
```

**Setting values**:
```rust
// Enable memory
os.database.settings.set(
    Setting::MemoryEnabled,
    Value::Bool(true)
)?;

// Set retention to 90 days
os.database.settings.set(
    Setting::MemoryRetentionDays,
    Value::Number(90.into())
)?;
```

---

## Testing Integration

### Unit Test

**File**: `crates/cortex-memory/src/config.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_config_from_settings() {
        let mut settings = chat_cli::database::settings::Settings::default();
        
        // Set test values
        settings.set(Setting::MemoryEnabled, json!(true)).unwrap();
        settings.set(Setting::MemoryRetentionDays, json!(90)).unwrap();
        settings.set(Setting::MemoryMaxSizeMb, json!(200)).unwrap();
        
        let config = MemoryConfig::from_q_settings(&settings);
        
        assert_eq!(config.enabled, true);
        assert_eq!(config.retention_days, 90);
        assert_eq!(config.max_size_mb, 200);
    }
    
    #[test]
    fn test_config_defaults() {
        let settings = chat_cli::database::settings::Settings::default();
        let config = MemoryConfig::from_q_settings(&settings);
        
        // Should use defaults when settings not present
        assert_eq!(config.enabled, true);
        assert_eq!(config.retention_days, 30);
        assert_eq!(config.max_size_mb, 100);
    }
}
```

---

## Summary of Changes

**Files to modify**:
1. ✅ `crates/chat-cli/src/database/settings.rs` - Add 6 new settings
2. ✅ `crates/cortex-memory/src/config.rs` - New file for config struct
3. ✅ `crates/chat-cli/src/cli/settings.rs` - Add memory subcommands
4. ✅ `crates/chat-cli/src/cli/chat/mod.rs` - Initialize Cortex with config

**No breaking changes**:
- All new settings have defaults
- Existing Q CLI functionality unaffected
- Settings file automatically updated on first use

**Backward compatible**:
- If settings don't exist, defaults are used
- Existing users get memory enabled by default
- Can be disabled via `q settings set memory.enabled false`

**✅ Question 3: Memory Retention**

**Decision**: Hybrid approach (30 days OR 100 MB, whichever first)

**Configuration**:
- Stored in `~/.q/config/settings.json`
- Accessed via Q CLI's existing `Settings` system
- Configurable via `q settings set` or `q memory config`

**Defaults**:
- Retention: 30 days
- Max Size: 100 MB
- Warn Threshold: 80%
- Auto-cleanup: Enabled
- Cross-session: Disabled by default

**Cleanup**:
- Automatic: On startup, after store, daily
- Manual: `q memory cleanup`
- Warning: At 80% threshold

**Benefits**:
- ✅ Predictable storage usage
- ✅ Reasonable time-based retention
- ✅ User configurable
- ✅ Integrates with existing Q CLI settings
- ✅ Graceful warnings before limits
