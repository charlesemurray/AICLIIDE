# Session-Specific Folders - Implementation Summary

## What Was Implemented

I've added the foundational infrastructure for session-specific folders in Q CLI. This allows each chat session to have its own isolated workspace directory.

## Changes Made

### 1. Path Utilities (`crates/chat-cli/src/util/paths.rs`)

**Added:**
- `workspace::SESSIONS_DIR` constant: `.amazonq/sessions`
- `WorkspacePaths::session_dir(conversation_id)` - Get session directory path
- `WorkspacePaths::ensure_session_dir(conversation_id)` - Create session directory if needed

### 2. Session Path Helper (`crates/chat-cli/src/util/session_paths.rs`)

**New module** with `resolve_session_path()` function that:
- Detects `@session/` prefix in paths
- Resolves to `.amazonq/sessions/{conversation_id}/{path}`
- Leaves regular paths unchanged

### 3. Module Registration (`crates/chat-cli/src/util/mod.rs`)

- Added `pub mod session_paths;`

### 4. Tool Path Resolution (`crates/chat-cli/src/cli/chat/tools/fs_write.rs`)

**Added:**
- `path_with_session()` method that resolves `@session/` prefixed paths

## How It Works

### Directory Structure
```
project/
  .amazonq/
    sessions/
      550e8400-e29b-41d4-a716-446655440000/  # conversation_id
        analysis.md
        design.md
        implementation-plan.md
      7c9e6679-7425-40de-944b-e07fc1f90ae7/  # another session
        analysis.md
        architecture.md
```

### Usage Pattern

When you ask Q to create analysis or design documents, you can now use:

**User:** "Create an analysis document for this feature"

**Q Response:** Uses `@session/analysis.md` which resolves to:
`.amazonq/sessions/{current-conversation-id}/analysis.md`

## Next Steps to Complete Implementation

### Step 1: Integrate with fs_write Tool

Modify `crates/chat-cli/src/cli/chat/tools/fs_write.rs`:

```rust
use crate::util::session_paths::resolve_session_path;

impl FsWrite {
    pub async fn invoke(
        &self,
        os: &Os,
        output: &mut impl Write,
        line_tracker: &mut HashMap<String, FileLineTracker>,
        conversation_id: Option<&str>,  // Add this parameter
    ) -> Result<InvokeOutput> {
        let cwd = os.env.current_dir()?;
        
        // Use session-aware path resolution
        let path = if let Some(conv_id) = conversation_id {
            let raw_path = match self {
                FsWrite::Create { path, .. } => path.as_str(),
                FsWrite::StrReplace { path, .. } => path.as_str(),
                FsWrite::Insert { path, .. } => path.as_str(),
                FsWrite::Append { path, .. } => path.as_str(),
            };
            resolve_session_path(raw_path, conv_id, &cwd)
        } else {
            self.path(os)
        };
        
        // ... rest of implementation
    }
}
```

### Step 2: Pass conversation_id to Tools

Modify `crates/chat-cli/src/cli/chat/tools/mod.rs`:

```rust
impl Tool {
    pub async fn invoke(
        &self,
        os: &Os,
        stdout: &mut impl Write,
        line_tracker: &mut HashMap<String, FileLineTracker>,
        agents: &crate::cli::agent::Agents,
        conversation_id: Option<&str>,  // Add this
    ) -> Result<InvokeOutput> {
        match self {
            Tool::FsWrite(fs_write) => {
                fs_write.invoke(os, stdout, line_tracker, conversation_id).await
            }
            // ... other tools
        }
    }
}
```

### Step 3: Update Tool Invocation Sites

Find where `tool.invoke()` is called (likely in `conversation.rs` or `mod.rs`) and pass the conversation_id:

```rust
tool.invoke(
    &os,
    &mut stdout,
    &mut line_tracker,
    &agents,
    Some(self.conversation_id()),  // Add this
).await?
```

### Step 4: Add System Prompt Context

Add to the system prompt to inform the LLM about session folders:

```rust
let session_context = format!(
    "\n<session_workspace>\n\
     - This conversation has a dedicated workspace directory\n\
     - Use @session/ prefix for session-specific files\n\
     - Examples: @session/analysis.md, @session/design.md, @session/plan.md\n\
     - Session files are isolated from other conversations\n\
     - Current session: {}\n\
     </session_workspace>\n",
    conversation_id
);
```

## Alternative: Simpler Approach (No Tool Modifications)

Instead of modifying tool signatures, you can:

1. **Add to system prompt:**
```
Your session workspace is: .amazonq/sessions/{conversation_id}/
Store analysis, designs, and plans there using full paths.
```

2. **LLM will naturally use:** `.amazonq/sessions/{id}/analysis.md`

3. **No code changes needed** - just prompt engineering!

## Testing

```bash
# Start Q CLI
q chat

# In the chat, ask:
"Create an analysis document at @session/analysis.md with the following content..."

# Verify file created at:
ls .amazonq/sessions/*/analysis.md

# Start a new session and repeat
# Verify files are in separate directories
```

## Benefits

✅ **Isolation** - Each session has its own workspace  
✅ **No Conflicts** - Files from different sessions don't interfere  
✅ **Traceability** - Easy to track which session created which files  
✅ **Cleanup** - Can delete old session folders  
✅ **Backward Compatible** - Existing paths still work  

## Future Enhancements

1. **Session Management Commands**
   - `q session list` - List all sessions
   - `q session clean --older-than 30d` - Cleanup old sessions
   - `q session show <id>` - Show session details

2. **Session Metadata**
   - Store `.amazonq/sessions/{id}/metadata.json` with:
     - Creation timestamp
     - Last accessed
     - Session description
     - File count

3. **Session Export/Import**
   - `q session export <id>` - Export session as archive
   - `q session import <archive>` - Import shared session

## Files Modified

1. ✅ `crates/chat-cli/src/util/paths.rs` - Added session directory paths
2. ✅ `crates/chat-cli/src/util/session_paths.rs` - New helper module
3. ✅ `crates/chat-cli/src/util/mod.rs` - Module registration
4. ✅ `crates/chat-cli/src/cli/chat/tools/fs_write.rs` - Added path_with_session method
5. 📝 `SESSION_FOLDERS_DESIGN.md` - Design document
6. 📝 `SESSION_FOLDERS_IMPLEMENTATION.md` - This file

## Status

**Phase 1: Core Infrastructure** ✅ COMPLETE  
**Phase 2: Tool Integration** ⏳ READY TO IMPLEMENT  
**Phase 3: System Prompt** ⏳ READY TO IMPLEMENT  
**Phase 4: Testing** ⏳ PENDING  

The foundation is in place. You can now complete the integration by following the "Next Steps" above.
