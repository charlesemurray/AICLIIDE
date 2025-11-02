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

## Decision Summary

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
