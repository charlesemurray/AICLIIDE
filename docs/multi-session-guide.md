# Multi-Session User Guide

## Overview

Multi-session support allows you to run multiple Q CLI chat sessions simultaneously, enabling parallel workflows without losing context.

## Quick Start

### Enable Multi-Session Mode

```bash
q settings set multiSession.enabled true
```

### Basic Commands

```bash
# List all sessions
/sessions

# Create a new session
/new

# Create a session with a specific type
/new debug

# Switch to a session
/switch my-session
# or use the short alias
/s my-session

# Rename current session
/rename new-name

# Close a session
/close my-session
```

## Session Types

- **debug** - For debugging and troubleshooting
- **planning** - For planning and design discussions
- **development** - For general development work (default)
- **review** - For code review sessions

## Session Names

Session names are automatically generated based on your conversation context. You can also set custom names:

- Max 20 characters
- Alphanumeric, dashes, and underscores only
- Example: `debug-api-fix`, `plan_feature_x`

## Configuration

```bash
# Maximum active sessions (default: 10)
q settings set multiSession.maxActive 10

# Buffer size in MB (default: 10)
q settings set multiSession.bufferSizeMb 10
```

## Tips

1. Use descriptive session names for easy identification
2. Close sessions when done to free resources
3. Use `/sessions --waiting` to see sessions waiting for input
4. Switch between sessions with `/s <name>` for quick navigation

## Troubleshooting

**Sessions not appearing?**
- Ensure multi-session mode is enabled
- Check `q settings get multiSession.enabled`

**Can't create more sessions?**
- Check your maxActive limit
- Close unused sessions with `/close`

**Session names too long?**
- Names are limited to 20 characters
- Use abbreviations or shorter names
