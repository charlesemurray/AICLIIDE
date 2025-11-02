# Session-Specific Folders Implementation Design

## Problem Statement

When working with Q CLI across multiple chat sessions, files created for analysis, design docs, and implementation plans can cause confusion when found by other sessions. We need session-specific folders to isolate artifacts per conversation.

## Solution Overview

Leverage the existing `conversation_id` (UUID v4) that Q CLI generates for each session to create isolated workspace directories.

## Architecture

### Directory Structure
```
.amazonq/
  sessions/
    {conversation-id-1}/
      analysis.md
      design.md
      implementation-plan.md
    {conversation-id-2}/
      analysis.md
      architecture.md
```

### Key Components

1. **Path Resolution Layer** (`util/paths.rs`)
   - Add `SESSIONS_DIR` constant: `.amazonq/sessions`
   - Add `session_dir(conversation_id)` method
   - Add `ensure_session_dir(conversation_id)` method

2. **Session Path Prefix** (`@session/`)
   - Special path prefix that resolves to current session directory
   - Example: `@session/analysis.md` → `.amazonq/sessions/{uuid}/analysis.md`

3. **Tool Integration** (`tools/fs_write.rs`, `tools/fs_read.rs`)
   - Detect `@session/` prefix
   - Resolve to session-specific path
   - Auto-create session directory on first write

## Implementation Plan

### Phase 1: Core Infrastructure (Minimal)

**File: `crates/chat-cli/src/util/paths.rs`**

```rust
// Add to workspace module
pub const SESSIONS_DIR: &str = ".amazonq/sessions";

// Add to WorkspacePaths impl
pub fn session_dir(&self, conversation_id: &str) -> Result<PathBuf> {
    Ok(self.os.env.current_dir()?.join(workspace::SESSIONS_DIR).join(conversation_id))
}

pub async fn ensure_session_dir(&self, conversation_id: &str) -> Result<PathBuf> {
    let dir = self.session_dir(conversation_id)?;
    if !dir.exists() {
        self.os.fs.create_dir_all(&dir).await?;
    }
    Ok(dir)
}
```

### Phase 2: Tool Modifications

**File: `crates/chat-cli/src/cli/chat/tools/mod.rs`**

Modify `Tool::invoke()` signature to accept `conversation_id`:

```rust
pub async fn invoke(
    &self,
    os: &Os,
    stdout: &mut impl Write,
    line_tracker: &mut HashMap<String, FileLineTracker>,
    agents: &crate::cli::agent::Agents,
    conversation_id: Option<&str>,  // NEW
) -> Result<InvokeOutput>
```

**File: `crates/chat-cli/src/cli/chat/tools/fs_write.rs`**

Add session path resolution:

```rust
impl FsWrite {
    fn resolve_path(&self, os: &Os, conversation_id: Option<&str>) -> PathBuf {
        let raw_path = match self {
            FsWrite::Create { path, .. } => path.as_str(),
            FsWrite::StrReplace { path, .. } => path.as_str(),
            FsWrite::Insert { path, .. } => path.as_str(),
            FsWrite::Append { path, .. } => path.as_str(),
        };
        
        // Handle @session/ prefix
        if let Some(stripped) = raw_path.strip_prefix("@session/") {
            if let Some(conv_id) = conversation_id {
                if let Ok(cwd) = os.env.current_dir() {
                    let session_path = cwd
                        .join(".amazonq/sessions")
                        .join(conv_id)
                        .join(stripped);
                    
                    // Ensure parent directory exists
                    if let Some(parent) = session_path.parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    
                    return session_path;
                }
            }
        }
        
        sanitize_path_tool_arg(os, raw_path)
    }
    
    pub async fn invoke(
        &self,
        os: &Os,
        output: &mut impl Write,
        line_tracker: &mut HashMap<String, FileLineTracker>,
        conversation_id: Option<&str>,  // NEW
    ) -> Result<InvokeOutput> {
        let path = self.resolve_path(os, conversation_id);
        // ... rest of implementation
    }
}
```

**File: `crates/chat-cli/src/cli/chat/tools/fs_read.rs`**

Similar changes for reading session files.

### Phase 3: Conversation Integration

**File: `crates/chat-cli/src/cli/chat/conversation.rs`**

Pass conversation_id when invoking tools (already available in `ConversationState`).

**File: `crates/chat-cli/src/cli/chat/mod.rs`**

Update tool invocation calls to pass `conversation_id`.

### Phase 4: System Prompt Enhancement

Add to system prompt to inform LLM about session folders:

```
<session_workspace>
- You have access to a session-specific workspace for this conversation
- Use the @session/ prefix to store files in the session directory
- Examples:
  - @session/analysis.md - Store analysis documents
  - @session/design.md - Store design documents
  - @session/plan.md - Store implementation plans
- Session files are isolated from other conversations
- Session ID: {conversation_id}
</session_workspace>
```

## Alternative: Simpler Approach (Recommended)

Instead of modifying tool signatures, use a **context-based approach**:

1. Add session directory info to the system context
2. Instruct LLM to use explicit paths like `.amazonq/sessions/{id}/`
3. No tool modifications needed
4. LLM naturally uses session paths when instructed

**Implementation:**

```rust
// In conversation.rs or prompt building
let session_context = format!(
    "Session workspace: .amazonq/sessions/{}/\n\
     Store analysis, designs, and plans in your session workspace.",
    self.conversation_id
);
```

## Benefits

1. **Isolation**: Each session has its own workspace
2. **No Conflicts**: Files from different sessions don't interfere
3. **Traceability**: Easy to track which session created which files
4. **Cleanup**: Can delete old session folders
5. **Backward Compatible**: Existing paths still work

## Migration Path

1. Deploy path utilities (Phase 1)
2. Add system prompt enhancement (Phase 4 - simple)
3. Optionally add tool modifications (Phase 2-3) for `@session/` prefix

## Testing

1. Start new session, verify conversation_id
2. Create file with `@session/analysis.md`
3. Verify file created in `.amazonq/sessions/{id}/analysis.md`
4. Start second session
5. Create file with same name
6. Verify files are in separate directories

## Future Enhancements

1. Session metadata file (`.amazonq/sessions/{id}/metadata.json`)
2. Session cleanup command (`q session clean --older-than 30d`)
3. Session listing (`q session list`)
4. Session export/import for sharing
