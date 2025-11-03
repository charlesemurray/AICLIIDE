# Memory System Release Notes

## Version 1.0.0 - Initial Release

### Overview

The Cortex Memory System brings intelligent context retention to Amazon Q CLI, enabling more natural and efficient conversations by remembering important information across sessions.

### Key Features

#### 🧠 Automatic Context Retention
- Q automatically stores and recalls relevant context from your conversations
- Smart semantic search finds the most relevant memories
- Session-scoped by default for focused context

#### 💬 Memory Commands
- `/memory config` - View current settings
- `/memory list` - Browse stored memories
- `/memory search <query>` - Search for specific content
- `/memory stats` - View usage statistics
- `/memory cleanup --force` - Clear all memories
- `/memory toggle` - Enable/disable memory
- `/recall <query>` - Explicitly recall context

#### 🔒 Privacy Controls
- `--no-memory` / `--ephemeral` flag for sensitive conversations
- Local storage only (no cloud sync)
- Easy cleanup with `/memory cleanup --force`
- Can be disabled entirely with `/memory toggle --disable`

#### ⚙️ Configuration
- Configurable retention period (default: 30 days)
- Storage limits (default: 100MB)
- Session-scoped or cross-session recall
- Verbose mode for detailed operation visibility

### Technical Details

- **Storage**: Local SQLite database at `~/.q/memory/cortex.db`
- **Search**: HNSW vector search with semantic embeddings
- **Performance**: < 100ms recall latency, < 50ms store latency
- **Capacity**: Configurable, default 100MB (~10,000 interactions)

### Getting Started

Memory is enabled by default. Just start chatting:

```bash
q chat "Let's build a REST API"
# Q will remember this context

# Later...
q chat
> /recall "REST API"
# Q recalls the previous conversation
```

### Settings

Configure in `~/.q/settings.json`:

```json
{
  "memory.enabled": true,
  "memory.retentionDays": 30,
  "memory.maxSizeMb": 100,
  "memory.verbose": false
}
```

### Documentation

- [User Guide](memory-user-guide.md) - Complete usage instructions
- [Developer Guide](memory-developer-guide.md) - Technical architecture
- [Performance](memory-performance.md) - Benchmarks and optimization
- [Integration Testing](memory-integration-testing.md) - Test scenarios

### Known Limitations

- Memory is local only (no sync across machines)
- Embeddings require model files (bundled with Q CLI)
- Large datasets (100k+ memories) may require cleanup

### Future Enhancements

Planned for future releases:
- Long-term memory with automatic promotion
- Memory importance scoring
- Cross-device sync (optional)
- Memory visualization tools
- Export/import functionality

### Migration

No migration needed - this is a new feature. Existing Q CLI installations will automatically enable memory on first use.

### Feedback

We'd love to hear your feedback! Please report issues or suggestions through the standard Q CLI channels.

---

## Changelog

### Added
- Cortex memory system with automatic context retention
- 7 memory management commands
- Session-scoped and global recall
- Ephemeral mode for privacy
- Verbose mode for detailed operations
- Welcome message for first-time users
- Comprehensive documentation

### Performance
- Store operations: ~10-20ms average
- Recall operations: ~30-60ms average
- All operations meet performance targets

### Testing
- 47 tests passing (41 unit + 6 integration)
- Performance benchmarks included
- Integration test scenarios documented
- Tested on macOS and Linux (x86_64, ARM64)
