# Cortex Memory - Visual Mockups

## Terminal Output Examples

### 1. First Run Experience

```
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

### 2. First Memory Store

```
You: How do I deploy a Python Lambda function?

Q: To deploy a Python Lambda function, you have several options:

1. Using AWS Console:
   - Navigate to Lambda service
   - Click "Create function"
   - Choose "Author from scratch"
   - Select Python 3.11 runtime
   ...

2. Using AWS CLI:
   ```
   aws lambda create-function \
     --function-name my-function \
     --runtime python3.11 \
     --handler lambda_function.lambda_handler \
     --zip-file fileb://function.zip
   ```

[💾 Memory saved - Q will remember this conversation]
   Disable: /memory toggle --disable | Configure: /memory config

You: 
```

### 3. Recall with Results

```
You: /recall Lambda deployment

▰▰▰▱▱▱▱ Recalling context...

Found 3 relevant memories:

  1. session-abc123 (2 days ago) - 95% match
     "How to deploy a Python Lambda function with environment variables
     and proper IAM roles..."

  2. session-xyz789 (1 week ago) - 87% match
     "AWS Lambda deployment using SAM CLI for local testing and
     automated deployment..."

  3. session-def456 (2 weeks ago) - 82% match
     "Lambda function timeout configuration and memory allocation
     best practices..."

You: Can you remind me about the SAM CLI approach?

Q: Based on our previous discussion about Lambda deployment using SAM CLI...
```

### 4. Recall - No Results

```
You: /recall Kubernetes deployment

[No memories found matching "Kubernetes deployment"]

Tips:
  • Try broader terms: /recall Kubernetes
  • Search all sessions: /recall --global Kubernetes
  • Check if memory is enabled: /memory config

You: 
```

### 5. Recall - First Use (Empty State)

```
You: /recall Lambda

[No memories stored yet]

Memory will automatically save your conversations.
Ask a few questions, then try /recall again!

Tips:
  • Memory saves after each Q response
  • Use /memory config to view settings
  • Use /help to see all commands

You: 
```

### 6. Recall - Global Search

```
You: /recall --global Lambda deployment

▰▰▰▱▱▱▱ Searching all sessions...

Found 5 relevant memories across 3 sessions:

  session-abc123 (Today):
    • "Lambda deployment with Python" (95% match)
    • "Lambda environment variables" (88% match)

  session-xyz789 (3 days ago):
    • "SAM CLI for Lambda" (91% match)
    • "Lambda CI/CD pipeline" (85% match)

  session-def456 (1 week ago):
    • "Lambda timeout configuration" (82% match)

You: Tell me about the CI/CD pipeline approach

Q: Based on our discussion 3 days ago about Lambda CI/CD pipelines...
```

### 7. List Sessions

```
You: /recall --list-sessions

Sessions with memories:

  1. session-abc123 (Today, 3:45 PM)
     "AWS Lambda deployment" - 15 memories

  2. session-xyz789 (Yesterday, 2:30 PM)
     "React component patterns" - 8 memories

  3. session-def456 (3 days ago)
     "Database optimization" - 12 memories

  4. session-ghi789 (1 week ago)
     "Python async programming" - 6 memories

Total: 41 memories across 4 sessions

Use: /recall --session <number|id> <query>
Example: /recall --session 1 environment variables

You: 
```

### 8. Memory Config

```
You: /memory config

📊 Memory Configuration:
  Enabled: ✓
  Retention: 30 days
  Max Size: 100 MB
  Cross-Session: ✗
  Auto-Promote: ✓
  Warn Threshold: 80%

Current Usage:
  Memories: 1,247
  Storage: 45.2 MB / 100 MB (45%)
  Oldest: 28 days ago

Commands:
  /memory set retention <days>  - Change retention period
  /memory set max-size <mb>     - Change storage limit
  /memory cleanup               - Remove old memories
  /memory toggle --disable      - Disable memory

You: 
```

### 9. Memory Stats

```
You: /memory stats

Memory Statistics:

Total Memories: 1,247
Storage: 45.2 MB / 100 MB (45%)
Sessions: 23 active

By Session:
  session-abc123: 156 memories (12.5%), 5.8 MB
  session-xyz789: 89 memories (7.1%), 3.2 MB
  session-def456: 67 memories (5.4%), 2.4 MB
  ... (20 more sessions)

By Age:
  Last 7 days: 342 memories (27.4%)
  Last 30 days: 1,120 memories (89.8%)
  Older than 30 days: 127 memories (10.2%)

⚠️  127 memories will be auto-deleted soon (older than 30 days)
    Run '/memory cleanup' to clean now

You: 
```

### 10. Memory Cleanup

```
You: /memory cleanup

Analyzing memories...

Found 127 memories older than 30 days:
  • 45 memories from sessions older than 30 days
  • 82 memories from active sessions (older than retention)

This will free approximately 4.8 MB of storage.

Proceed with cleanup? (y/N): y

[Cleaning up old memories...]
▰▰▰▰▰▰▰ 100% (127/127)

✓ Deleted 127 memories
✓ Freed 4.8 MB of storage

Current usage: 40.4 MB / 100 MB (40%)

You: 
```

### 11. Memory List

```
You: /memory list

Recent memories (last 10):

  1. [2 min ago] session-abc123
     "Discussion about AWS Lambda deployment with Python runtime
     and environment variable configuration..."

  2. [15 min ago] session-abc123
     "Explanation of Lambda function timeout settings and how to
     configure memory allocation..."

  3. [1 hour ago] session-abc123
     "Overview of SAM CLI for local Lambda testing and automated
     deployment pipelines..."

  4. [3 hours ago] session-xyz789
     "React component lifecycle methods and when to use useEffect
     vs useLayoutEffect hooks..."

  5. [5 hours ago] session-xyz789
     "State management patterns in React using Context API and
     custom hooks..."

  ... (5 more)

Use: /memory list --limit 20 to see more
Use: /memory search <query> to search memories

You: 
```

### 12. Memory Search

```
You: /memory search "Lambda timeout"

🔍 Searching for: "Lambda timeout"

Found 4 relevant memories:

  1. session-abc123 (15 min ago) - 98% match
     "Lambda function timeout settings and how to configure memory
     allocation for optimal performance..."

  2. session-def456 (2 days ago) - 92% match
     "Debugging Lambda timeout errors and increasing execution time
     limits in AWS Console..."

  3. session-abc123 (1 week ago) - 85% match
     "Best practices for Lambda configuration including timeout,
     memory, and retry settings..."

  4. session-ghi789 (2 weeks ago) - 78% match
     "Comparison of Lambda timeout limits across different AWS
     regions and service quotas..."

You: 
```

### 13. Storage Warning (80% threshold)

```
You: How do I configure Lambda environment variables?

⚠️  Memory storage at 85 MB / 100 MB (85%)
    Run '/memory cleanup' to free space
    Or increase limit: /memory set max-size 200

▰▰▰▱▱▱▱ Recalling context...

Q: Based on our previous discussion about Lambda deployment...
```

### 14. Storage Full Error

```
You: How do I deploy to Lambda?

[Warning: Memory storage full (100 MB / 100 MB)]
Cannot store new memories until space is freed.

Options:
  1. Clean old memories: /memory cleanup
  2. Increase limit: /memory set max-size 200
  3. Disable memory: /memory toggle --disable

Continuing without storing this conversation...

Q: Here's how to deploy to Lambda...
```

### 15. Database Locked Error

```
You: /recall Lambda deployment

▰▰▰▱▱▱▱ Recalling context...

[Error: Memory database is locked]
The memory database is currently in use by another process.

Try:
  • Wait a moment and try again
  • Close other Q CLI instances
  • If problem persists: /memory toggle --disable

Continuing without memory for this query...

Q: How can I help you with Lambda deployment?
```

### 16. Memory Toggle

```
You: /memory toggle --disable

✓ Memory disabled

Q will no longer store or recall conversations.
Your existing memories are preserved.

To re-enable: /memory toggle

You: 

---

You: /memory toggle

✓ Memory enabled

Q will now remember conversations to provide better help.

You: 
```

### 17. Memory Settings Change

```
You: /memory set retention 90

✓ Memory retention set to 90 days

Previous: 30 days
New: 90 days

Memories will now be kept for 90 days before automatic cleanup.

You: 

---

You: /memory set max-size 200

✓ Memory max size set to 200 MB

Previous: 100 MB
New: 200 MB
Current usage: 45.2 MB / 200 MB (23%)

You: 
```

### 18. Help Command

```
You: /help

Available Commands:

Chat:
  /quit, /q, /exit        Quit the application
  /clear                  Clear conversation history
  /help                   Show this help message

Memory:
  /recall <query>         Search past conversations
    --global, -g          Search all sessions
    --session <id>, -s    Search specific session
    --list-sessions       List sessions with memories
  /memory config          View memory settings
  /memory set <setting>   Change memory settings
  /memory list            List recent memories
  /memory search <query>  Search memories
  /memory stats           View memory statistics
  /memory cleanup         Remove old memories
  /memory toggle          Enable/disable memory

Context:
  /context add <path>     Add file to context
  /context list           List context files
  ...

Type /help <command> for detailed help on a specific command

You: 
```

### 19. Verbose Mode

```
You: /memory set verbose

✓ Verbose mode enabled

Memory operations will now show detailed information.

You: /recall Lambda

▰▰▰▱▱▱▱ Searching 1,247 memories...

Search completed in 45ms
Found 3 matches:

  1. mem-abc123 (session-abc123, 2 days ago)
     Score: 0.95 (cosine similarity)
     "How to deploy a Python Lambda function..."

  2. mem-xyz789 (session-xyz789, 1 week ago)
     Score: 0.87
     "AWS Lambda deployment using SAM CLI..."

  3. mem-def456 (session-def456, 2 weeks ago)
     Score: 0.82
     "Lambda function timeout configuration..."

Q: Based on these previous discussions...

[Stored to memory: mem-ghi789]
Storage: 45.3 MB / 100 MB (45%)

You: 
```

### 20. Ephemeral Session

```
$ q chat --no-memory

Amazon Q Developer CLI (Ephemeral Session)

Memory is disabled for this session.
Conversations will not be stored or recalled.

You: How do I deploy to Lambda?

Q: Here's how to deploy to Lambda...

[No memory stored]

You: 
```

---

## Visual Design Specifications

### Spacing
- 1 blank line before memory indicators
- 1 blank line after memory indicators
- 2 blank lines between major sections

### Indentation
- List items: 2 spaces
- Nested items: 4 spaces
- Code blocks: 3 spaces

### Text Styling
- **Errors**: Red text (StyledText::error_fg())
- **Warnings**: Yellow text (StyledText::warning_fg())
- **Success**: Green text (StyledText::success_fg())
- **Dim info**: Gray text (StyledText::dim())
- **Commands**: No styling (plain text)

### Symbols
- ✓ Success checkmark
- ⚠️  Warning triangle
- 💡 Tip/info lightbulb
- 💾 Memory save indicator
- 🔍 Search indicator
- 📊 Stats indicator
- 📝 List indicator

### Progress Indicators
- Spinner: `▰▰▰▱▱▱▱` (Q CLI's custom spinner)
- Progress bar: `▰▰▰▰▰▰▰ 100% (127/127)`

### Line Length
- Max 80 characters for messages
- Wrap long content with ellipsis
- Preserve code blocks without wrapping

---

## Accessibility Notes

- All information conveyed through text (no color-only)
- Screen reader friendly (no ASCII art)
- Keyboard-only navigation
- Clear hierarchy with indentation
- Consistent formatting

---

## Implementation Checklist

- [x] First run welcome message
- [x] First memory save notification
- [x] Recall with results
- [x] Recall no results
- [x] Recall empty state
- [x] Memory config display
- [x] Memory stats display
- [x] Storage warning
- [x] Storage full error
- [x] Database locked error
- [x] All command outputs designed

**Ready for implementation** ✅
