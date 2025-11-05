# Workflow CLI ToolManager Integration Fix

## Problem

The CLI `/workflows run` command was using the synchronous `invoke_with_definition()` method which doesn't have access to ToolManager. This meant:

- Workflows run from CLI couldn't invoke skills
- Only basic tools (fs_read, fs_write, execute_bash) worked
- Skills would fail with "tool not found" errors

## Solution

Changed the `Run` command in `crates/chat-cli/src/cli/chat/cli/workflows.rs` to use the async `invoke_with_definition_and_manager()` method with access to the session's ToolManager.

### Code Change

**Before:**
```rust
match tool.invoke_with_definition(workflow, params_map) {
    Ok(result) => format!("🔄 Workflow '{}' completed\n\n{}", workflow_name, result),
    Err(e) => format!("❌ Workflow '{}' failed: {}", workflow_name, e),
}
```

**After:**
```rust
match tool.invoke_with_definition_and_manager(workflow, params_map, Some(&mut _session.conversation.tool_manager)).await {
    Ok(result) => format!("🔄 Workflow '{}' completed\n\n{}", workflow_name, result),
    Err(e) => format!("❌ Workflow '{}' failed: {}", workflow_name, e),
}
```

## Impact

- ✅ Workflows can now invoke skills from CLI
- ✅ Full ToolManager access for skill execution
- ✅ Consistent behavior between LLM-invoked and CLI-invoked workflows
- ✅ No breaking changes to API

## Testing

Test workflow created at `.q-workflows/test-skill-invocation.json`:

```json
{
  "name": "test-skill-invocation",
  "version": "1.0.0",
  "description": "Test workflow that invokes calculator skill",
  "steps": [
    {
      "name": "calculate",
      "tool": "calculator",
      "parameters": {
        "operation": "add",
        "a": 5,
        "b": 3
      }
    }
  ]
}
```

Run with:
```bash
/workflows run test-skill-invocation
```

Expected output:
```
🔄 Workflow 'test-skill-invocation' completed

Executed 1 steps successfully in X.XXms

Step 'calculate': 8 (completed in X.XXms)
```

## Remaining Limitations

1. **Creation assistant state persistence** - Cannot continue conversations across messages
2. **Async tool support** - Cannot invoke AWS tools, MCP tools, code_search
3. **Permission checking** - Workflows bypass user approval for dangerous operations

## Related Files

- `crates/chat-cli/src/cli/chat/cli/workflows.rs` - CLI command implementation
- `crates/chat-cli/src/cli/chat/tools/workflow.rs` - WorkflowTool with both sync and async methods
- `crates/chat-cli/src/cli/chat/conversation.rs` - ConversationState with tool_manager field
