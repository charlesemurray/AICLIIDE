# Session Folders - Quick Start Guide

## TL;DR

I've implemented the foundation for session-specific folders. Here's what you need to know:

## What's Done ✅

- Path utilities for session directories (`.amazonq/sessions/{conversation_id}/`)
- Helper function to resolve `@session/` prefix
- Infrastructure ready to use

## Simplest Way to Use It (No Additional Code Needed!)

### Option 1: Prompt Engineering (Recommended - Zero Code Changes)

Just add this to your system prompt in the chat initialization:

```rust
// In crates/chat-cli/src/cli/chat/mod.rs or wherever system prompt is built
let session_workspace_hint = format!(
    "\nYour session workspace directory: .amazonq/sessions/{}/\n\
     When creating analysis, design docs, or plans, store them in your session workspace.\n",
    conversation_id
);
```

The LLM will naturally use paths like:
- `.amazonq/sessions/{id}/analysis.md`
- `.amazonq/sessions/{id}/design.md`
- `.amazonq/sessions/{id}/implementation-plan.md`

**That's it!** No tool modifications needed.

### Option 2: Full Integration (More Work, Better UX)

If you want the `@session/` prefix to work automatically:

1. **Update Tool Invocation** (1 file change)

Find where tools are invoked (search for `tool.invoke(` in `conversation.rs`):

```rust
// Before:
tool.invoke(&os, &mut stdout, &mut line_tracker, &agents).await?

// After:
tool.invoke(&os, &mut stdout, &mut line_tracker, &agents, Some(self.conversation_id())).await?
```

2. **Update Tool Signature** (1 file change)

In `crates/chat-cli/src/cli/chat/tools/mod.rs`:

```rust
pub async fn invoke(
    &self,
    os: &Os,
    stdout: &mut impl Write,
    line_tracker: &mut HashMap<String, FileLineTracker>,
    agents: &crate::cli::agent::Agents,
    conversation_id: Option<&str>,  // ADD THIS LINE
) -> Result<InvokeOutput>
```

3. **Update fs_write** (1 file change)

In `crates/chat-cli/src/cli/chat/tools/fs_write.rs`, replace `self.path(os)` with:

```rust
use crate::util::session_paths::resolve_session_path;

// In invoke method:
let cwd = os.env.current_dir()?;
let raw_path = match self {
    FsWrite::Create { path, .. } => path.as_str(),
    FsWrite::StrReplace { path, .. } => path.as_str(),
    FsWrite::Insert { path, .. } => path.as_str(),
    FsWrite::Append { path, .. } => path.as_str(),
};
let path = conversation_id
    .map(|id| resolve_session_path(raw_path, id, &cwd))
    .unwrap_or_else(|| sanitize_path_tool_arg(os, raw_path));
```

4. **Update fs_read** (similar changes)

Same pattern for reading session files.

## Testing

```bash
# Build
cargo build --bin chat_cli

# Run
cargo run --bin chat_cli

# In chat, try:
> Create an analysis document at @session/analysis.md

# Check it was created:
ls .amazonq/sessions/*/analysis.md

# Start new session and repeat - files should be in different folders
```

## Recommendation

**Start with Option 1 (Prompt Engineering)** - It's the simplest and requires zero code changes. The LLM is smart enough to use the session directory when you tell it to.

**Upgrade to Option 2** later if you want the cleaner `@session/` syntax.

## Example System Prompt Addition

```
<session_workspace>
This conversation has a dedicated workspace for storing artifacts:
- Location: .amazonq/sessions/{conversation_id}/
- Use this directory for:
  - Analysis documents (analysis.md)
  - Design documents (design.md, architecture.md)
  - Implementation plans (plan.md, roadmap.md)
  - Any session-specific files
- Files in this directory are isolated from other chat sessions
- This prevents confusion when multiple sessions work on similar topics
</session_workspace>
```

## Architecture Decision

The implementation uses a **hybrid approach**:

1. **Infrastructure layer** - Path utilities and helpers (✅ Done)
2. **Tool layer** - Optional `@session/` prefix support (⏳ Your choice)
3. **Prompt layer** - LLM guidance to use session dirs (⏳ Recommended)

You can use any combination. The prompt layer alone is sufficient for most use cases.

## Questions?

- **Q: Where are session folders created?**  
  A: `.amazonq/sessions/{conversation_id}/` in your current working directory

- **Q: What's the conversation_id?**  
  A: A UUID v4 generated when you start a new chat session

- **Q: Can I use regular paths too?**  
  A: Yes! Session folders are optional. Regular paths work as before.

- **Q: How do I clean up old sessions?**  
  A: Manually delete folders in `.amazonq/sessions/` or implement a cleanup command

- **Q: Can I share session folders?**  
  A: Yes, just copy the folder. Future enhancement: export/import commands

## Next Steps

1. Choose your approach (Option 1 or 2)
2. Implement it (5-30 minutes depending on option)
3. Test with a few sessions
4. Iterate based on usage patterns

Good luck! 🚀
