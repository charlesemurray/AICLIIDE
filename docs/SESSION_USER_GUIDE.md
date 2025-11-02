# Session Management User Guide

Amazon Q CLI automatically creates isolated session workspaces for each conversation, helping you organize analysis, research, and planning documents separately from your code.

## Overview

Each chat session has its own workspace directory at `.amazonq/sessions/{conversation_id}/`. This keeps temporary work isolated between different conversations while your code stays in the main repository.

### What Goes Where?

- **Session Workspace** (`.amazonq/sessions/{conversation_id}/`): Analysis, research notes, planning documents, design drafts
- **Repository**: Code, tests, committed documentation, production files

## Commands

### List Active Sessions

View all currently active sessions:

```bash
/sessions list
```

Example output:
```
💬 Active Sessions:
  1. My Feature - "Implement user authentication" (2 hours ago, 5 messages, 3 files)
  2. 8a3f2b1c - "Debug performance issue" (1 day ago, 12 messages, 7 files)
```

### View Session History

See archived sessions:

```bash
/sessions history
```

With options:
```bash
/sessions history --limit 20 --search "authentication"
```

### View Background Sessions

List sessions running background tasks:

```bash
/sessions background
```

With options:
```bash
/sessions background --limit 5 --search "deployment"
```

### Name a Session

Give a session a memorable name:

```bash
/sessions name 8a3f2b1c "Auth Feature Work"
```

Names must be:
- 1-256 characters
- Non-empty

### Archive a Session

Move a session to history:

```bash
/sessions archive 8a3f2b1c
```

Archived sessions:
- No longer appear in active list
- Remain searchable in history
- Keep all files and metadata

## Session Types

### Active Sessions
Interactive work in progress. These appear in `/sessions list`.

### Background Sessions
Autonomous tasks running in the background. View with `/sessions background`.

### Archived Sessions
Completed or inactive sessions. Access via `/sessions history`.

## File Organization

Amazon Q automatically uses the session workspace for temporary documents:

```
.amazonq/
└── sessions/
    └── {conversation_id}/
        ├── metadata.json          # Session info
        ├── analysis.md            # Analysis documents
        ├── research_notes.md      # Research findings
        └── implementation_plan.md # Planning docs
```

Your repository structure remains clean:

```
your-project/
├── src/                # Source code
├── tests/              # Test files
├── docs/               # Committed documentation
└── .amazonq/           # Q CLI workspace (add to .gitignore)
```

## Best Practices

### 1. Name Important Sessions

Give sessions descriptive names for easy reference:
```bash
/sessions name abc123 "OAuth Integration Sprint"
```

### 2. Archive Completed Work

Keep your active list clean:
```bash
/sessions archive abc123
```

### 3. Search History

Find past sessions quickly:
```bash
/sessions history --search "database migration"
```

### 4. Add .amazonq to .gitignore

Session workspaces are temporary and shouldn't be committed:
```bash
echo ".amazonq/" >> .gitignore
```

## Metadata

Each session stores metadata in `metadata.json`:

```json
{
  "version": 1,
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "active",
  "created": "2025-11-02T20:00:00Z",
  "last_active": "2025-11-02T23:00:00Z",
  "first_message": "Help me implement authentication",
  "name": "Auth Feature Work",
  "file_count": 5,
  "message_count": 23
}
```

## Troubleshooting

### Session Not Found

If a session ID doesn't exist:
```
Error: Session 'abc123' not found
```

Solution: Use `/sessions list` to see available sessions.

### Invalid Session Name

If a name is invalid:
```
Error: Session name must be between 1 and 256 characters
```

Solution: Provide a name within the valid length range.

### Permission Issues

If you can't write to `.amazonq/`:
```
Error: Failed to save session metadata: Permission denied
```

Solution: Check directory permissions or run from a writable location.

## FAQ

**Q: Are session files backed up?**
A: Session workspaces are local and temporary. Commit important documents to your repository.

**Q: Can I manually edit session files?**
A: Yes, but avoid editing `metadata.json` directly. Use commands to modify session state.

**Q: How do I delete a session?**
A: Currently, archive it with `/sessions archive`. Deletion support coming soon.

**Q: Can I restore an archived session?**
A: Not yet, but the files remain in `.amazonq/sessions/{id}/` and can be accessed directly.

**Q: Do sessions sync across machines?**
A: No, sessions are local to each machine. Use git for code synchronization.

## Related Documentation

- [Session Management Design](SESSION_MANAGEMENT_DESIGN_V2.md) - Architecture details
- [Implementation Plan](SESSION_IMPLEMENTATION_PLAN_V2.md) - Development approach
- [Skills & Workflows](SKILLS_WORKFLOWS_INTEGRATION.md) - Extending Q CLI
