# Memory System User Guide

Amazon Q CLI's memory system helps maintain context across conversations, making interactions more natural and efficient.

## Quick Start

Memory is enabled by default. When you start a chat session, Q will automatically:
- Remember important context from your conversations
- Recall relevant information when needed
- Keep memories organized by session

## Commands

### View Configuration

```bash
/memory config
```

Shows current memory status and settings.

### List Memories

```bash
/memory list [--limit N] [--session ID]
```

View recent memories. Use `--limit` to control how many are shown (default: 10).

### Search Memories

```bash
/memory search <query> [--limit N]
```

Search for specific memories by content.

### Recall Context

```bash
/recall <query> [--limit N] [--global] [--session ID]
```

Explicitly recall memories matching your query. By default, searches current session only.
- `--global`: Search across all sessions
- `--session ID`: Search specific session

### View Statistics

```bash
/memory stats
```

Shows memory usage and statistics.

### Clean Up

```bash
/memory cleanup --force
```

Clear all stored memories. Requires `--force` flag for safety.

### Toggle Memory

```bash
/memory toggle [--disable]
```

Enable or disable the memory system.

### Verbose Mode

```bash
/memory set verbose true
```

Enable detailed output showing what memories are recalled during conversations.

## Ephemeral Sessions

Start a session without memory:

```bash
q chat --no-memory "your question"
# or
q chat --ephemeral "your question"
```

Useful for sensitive topics or one-off queries.

## How It Works

1. **Automatic Storage**: Q stores user-assistant interactions during conversations
2. **Smart Recall**: Before responding, Q recalls relevant context from past conversations
3. **Session Isolation**: By default, memories are scoped to the current session
4. **Cross-Session**: Enable with settings to recall from all sessions

## Settings

Configure memory behavior in `~/.q/settings.json`:

```json
{
  "memory.enabled": true,
  "memory.retentionDays": 30,
  "memory.maxSizeMb": 100,
  "memory.crossSession": false,
  "memory.verbose": false
}
```

## Privacy

- Memories are stored locally in `~/.q/memory/cortex.db`
- Use `--no-memory` for sensitive conversations
- Use `/memory cleanup --force` to clear all data
- Disable entirely with `/memory toggle --disable`

## Tips

- Use `/recall` to check what Q remembers about a topic
- Enable verbose mode to see memory operations in action
- Use session-specific recall for focused context
- Clean up old memories periodically to save space

## Troubleshooting

**Memory not working?**
- Check status with `/memory config`
- Ensure it's enabled with `/memory toggle`

**Too much/too little context?**
- Adjust `--limit` parameter in recall commands
- Use session-scoped vs global recall appropriately

**Storage concerns?**
- Check usage with `/memory stats`
- Adjust `memory.maxSizeMb` in settings
- Run `/memory cleanup --force` to clear old data
