# Cortex Memory - Privacy & Transparency Design

## Decision: Enabled by Default ✅

Memory system is enabled by default with clear transparency and easy opt-out.

---

## Rationale

**Why enabled by default**:
1. **Local storage only** - All data stays on user's machine in `~/.q/memory/`
2. **Expected behavior** - Users expect AI assistants to remember context
3. **Immediate value** - Users benefit from day one without setup
4. **Easy opt-out** - Simple `/memory toggle --disable` command
5. **Industry standard** - ChatGPT, Claude, etc. all remember by default

**Privacy safeguards**:
- ✅ Local storage (no cloud sync)
- ✅ Clear disclosure on first use
- ✅ Easy disable mechanism
- ✅ Configurable retention
- ✅ Manual cleanup available
- ✅ Per-session isolation by default

---

## Transparency Mechanisms

### 1. Welcome Message (First Run)

**On first Q CLI launch with memory**:
```bash
$ q chat

Welcome to Amazon Q Developer CLI!

💡 Q now remembers context to provide better help:
   • Memories stored locally in ~/.q/memory/
   • Automatically cleaned after 30 days
   • Disable anytime: /memory toggle --disable
   • View settings: /memory config

Type /help for available commands

You: 
```

### 2. First Memory Store Notification

**When first memory is saved**:
```bash
You: How do I deploy to Lambda?

[💾 Memory saved - Q will remember this conversation]
   Disable: /memory toggle --disable | Configure: /memory config

Q: Here's how to deploy to Lambda...
```

**After first notification, silent operation** (no more notifications unless warning threshold reached).

### 3. Settings Visibility

**In `/help` output**:
```bash
You: /help

Memory Commands:
  /memory config          View memory settings
  /memory toggle          Enable/disable memory
  /recall <query>         Search past conversations
  
Type /memory config to see current settings
```

### 4. Status Indicators (Optional)

**In prompt (configurable)**:
```bash
You [💾]: How do I...
```

Or minimal:
```bash
You: How do I...
```

**Recommendation**: No indicator by default (silent operation), but available via setting.

---

## Opt-Out Mechanisms

### Easy Disable

**In-chat**:
```bash
You: /memory toggle --disable
✓ Memory disabled - Q will not store or recall conversations

You: /memory toggle
✓ Memory enabled
```

**Via settings**:
```bash
$ q settings set memory.enabled false
```

**Via config file**:
```json
{
  "memory.enabled": false
}
```

### Ephemeral Sessions

**Temporary disable for sensitive conversations**:
```bash
$ q chat --no-memory
# or
$ q chat --ephemeral

# Nothing stored or recalled for this session
```

### Selective Deletion

**Delete specific memories**:
```bash
You: /memory search "sensitive topic"
Found 3 memories...

You: /memory delete <id>
✓ Memory deleted
```

**Clear all memories**:
```bash
You: /memory cleanup --all
⚠️  This will delete ALL memories. Type 'yes' to confirm: yes
✓ Deleted 1,247 memories
```

---

## Privacy Features

### 1. Local Storage Only

**No cloud sync**:
- All data in `~/.q/memory/cortex.db`
- SQLite database on local filesystem
- No network transmission of memories
- User has full control

### 2. Automatic Cleanup

**Default retention**:
- 30 days automatic deletion
- 100 MB storage limit
- Configurable via settings

### 3. Session Isolation

**Default behavior**:
- Memories tagged with session_id
- Recall searches current session only
- Cross-session requires explicit flag

### 4. Encryption (Future)

**Phase 2 enhancement**:
- Optional database encryption
- User-provided passphrase
- Transparent encryption/decryption

---

## User Control

### Configuration Options

```bash
# View all settings
/memory config

# Disable memory
/memory toggle --disable

# Set retention
/memory set retention 90      # 90 days
/memory set retention 0       # unlimited

# Set storage limit
/memory set max-size 200      # 200 MB

# Disable cross-session
/memory set cross-session --disable
```

### Data Export

**Export memories**:
```bash
$ q memory export memories.json
✓ Exported 1,247 memories to memories.json
```

**Import memories**:
```bash
$ q memory import memories.json
✓ Imported 1,247 memories
```

### Manual Deletion

**Delete database**:
```bash
$ rm -rf ~/.q/memory/
# Memory system will recreate on next use
```

---

## Documentation

### In-App Help

**`/help` includes memory section**:
```
Memory & Context:
  Q remembers conversations to provide better help.
  
  Commands:
    /memory config     - View settings
    /memory toggle     - Enable/disable
    /recall <query>    - Search past conversations
  
  Privacy:
    • Stored locally in ~/.q/memory/
    • Auto-deleted after 30 days
    • Disable anytime with /memory toggle --disable
  
  Learn more: https://docs.aws.amazon.com/q/memory
```

### README Update

**Add to Q CLI README**:
```markdown
## Memory & Context

Q CLI now remembers context across conversations to provide more relevant help.

**Privacy**:
- All memories stored locally on your machine
- Automatically cleaned after 30 days
- Disable anytime: `q settings set memory.enabled false`

**Commands**:
- `/memory config` - View settings
- `/recall <query>` - Search past conversations
- `/memory toggle --disable` - Disable memory

See [Memory Documentation](docs/memory.md) for details.
```

---

## Telemetry (Optional)

**Anonymous usage metrics** (if telemetry enabled):
- Memory enabled/disabled count
- Average memories per user
- Average retention period
- Feature usage (recall, cleanup, etc.)

**Never collected**:
- ❌ Memory content
- ❌ User queries
- ❌ Conversation data
- ❌ Personal information

---

## Default Configuration

**`~/.q/config/settings.json` defaults**:
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

## Migration for Existing Users

**On Q CLI upgrade**:
1. Memory feature added to settings with defaults
2. Welcome message shown on next `q chat` launch
3. First memory store shows notification
4. No action required from user

**Opt-out for existing users**:
```bash
$ q settings set memory.enabled false
```

---

## Compliance Considerations

### GDPR (if applicable)

**Right to access**:
- ✅ Users can export: `q memory export`

**Right to deletion**:
- ✅ Users can delete: `q memory cleanup --all`
- ✅ Users can disable: `/memory toggle --disable`

**Data minimization**:
- ✅ Only conversation context stored
- ✅ Automatic 30-day retention
- ✅ Local storage only

**Transparency**:
- ✅ Clear disclosure on first use
- ✅ Documentation available
- ✅ Easy access to settings

---

## Summary

**✅ Decision: Enabled by default**

**Transparency**:
- Clear welcome message
- First-save notification
- Visible in `/help`
- Documentation

**Privacy**:
- Local storage only
- Automatic cleanup
- Easy opt-out
- Session isolation
- Data export/delete

**User Control**:
- `/memory toggle --disable`
- Configurable retention
- Manual cleanup
- Ephemeral sessions

**Compliance**:
- GDPR-friendly
- User data control
- Transparent operation
- Easy deletion
