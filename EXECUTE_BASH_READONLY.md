# Execute Bash Readonly Tool

## Overview

This document describes the implementation of the `execute_bash_readonly` tool, a safer alternative to `execute_bash` for read-only operations.

## Motivation

The original `execute_bash` tool can execute any bash command, including destructive operations. This creates friction when the LLM just wants to inspect the system state (e.g., `ls`, `cat`, `grep`). The readonly variant provides:

1. **Safety**: Prevents accidental modifications to the filesystem or system state
2. **Intent signaling**: Makes it clear when inspection vs. modification is happening
3. **Better UX**: Reduces permission prompts for safe read-only commands

## Implementation

### Files Modified

1. **`crates/chat-cli/src/cli/chat/tools/tool_index.json`**
   - Added `execute_bash_readonly` tool specification
   - Updated `execute_bash` description to clarify it's for commands that may modify state

2. **`crates/chat-cli/src/cli/chat/tools/mod.rs`**
   - Added `execute_bash_readonly` to `NATIVE_TOOLS` array

3. **`crates/chat-cli/src/cli/chat/tools/execute/mod.rs`**
   - Added `is_readonly: bool` field to `ExecuteCommand` struct
   - Updated `eval_perm()` to use `is_readonly` flag for permission checking

4. **`crates/chat-cli/src/cli/chat/tool_manager.rs`**
   - Added parsing case for `execute_bash_readonly` that sets `is_readonly = true`

### How It Works

1. **Tool Selection**: The LLM chooses between `execute_bash` and `execute_bash_readonly` based on the command intent
2. **Parsing**: When `execute_bash_readonly` is called, the tool manager sets the `is_readonly` flag
3. **Permission Check**: The `eval_perm()` method uses this flag to automatically allow safe readonly commands
4. **Execution**: Both tools use the same underlying execution logic

### Readonly Commands

The following commands are considered safe and don't require user confirmation when using `execute_bash_readonly`:

- `ls` - List directory contents
- `cat` - Display file contents
- `echo` - Print text
- `pwd` - Print working directory
- `which` - Locate commands
- `head` - Display first lines of file
- `tail` - Display last lines of file
- `find` - Search for files (without `-exec`, `-delete`, etc.)
- `grep` - Search text (without `-P` flag)
- `dir` - List directory (Windows)
- `type` - Display file contents (Windows)

### Safety Features

The readonly tool still validates commands for:
- Multi-line commands (always require confirmation)
- Dangerous patterns (`$(`, `<(`, backticks, redirects, etc.)
- Piped commands (each command in the chain must be readonly)
- Special `find` flags that modify files (`-exec`, `-delete`, `-ok`, etc.)
- Perl regex in `grep` (RCE vulnerability)

## Usage

The LLM will automatically choose the appropriate tool:

**Inspection tasks** → `execute_bash_readonly`
- "Show me the files in this directory"
- "What's in the README?"
- "Search for TODO comments"

**Modification tasks** → `execute_bash`
- "Delete the temp files"
- "Create a new directory"
- "Move this file"

## Configuration

Users can still control behavior via agent settings:

```json
{
  "execute_bash": {
    "autoAllowReadonly": true,  // Auto-allow readonly commands even with execute_bash
    "allowedCommands": [".*"],   // Regex patterns for allowed commands
    "deniedCommands": [],        // Regex patterns for denied commands
    "denyByDefault": false       // Require explicit allowlist
  }
}
```

## Testing

Build the project:
```bash
cargo build --bin chat_cli
```

The tool will be available in chat sessions when the agent configuration includes `@builtin` or `@builtin/execute_bash_readonly`.

## Future Enhancements

Potential improvements:
1. Add more readonly commands to the safe list
2. Implement command sandboxing for additional safety
3. Add telemetry to track readonly vs. write tool usage
4. Consider a "dry-run" mode for write commands
