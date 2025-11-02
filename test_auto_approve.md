# Auto-Approve and Batch Mode Enhancement

## New Features Added

### 1. CLI Arguments
- `--auto-approve N`: Auto-approve the next N tool executions
- `--batch-mode`: Auto-approve all tools until user types 'stop'

### 2. Interactive Commands
During tool approval prompts, users can now type:
- `auto N`: Auto-approve next N tools (e.g., `auto 10`)
- `batch`: Enter batch mode (auto-approve until 'stop')
- `stop`/`pause`: Exit batch mode

### 3. Usage Examples

#### For your 102-step implementation plan:

```bash
# Option 1: Auto-approve next 102 tools
q chat --auto-approve 102 "Please implement the 102-step plan we discussed"

# Option 2: Use batch mode
q chat --batch-mode "Please implement the 102-step plan we discussed"
# (Type 'stop' when you want to pause and review)

# Option 3: Start normally, then use interactive commands
q chat "Please implement the 102-step plan we discussed"
# When first tool appears, type: auto 102
# Or type: batch
```

#### Interactive Usage:
```
> create 5 files
Allow this action? Use 't' to trust (always allow) this tool for the session, 'auto N' to auto-approve next N tools, 'batch' for batch mode. [y/n/t]:
auto 5
Auto-approve activated for 5 tools.
✓ Auto-approving tool (4 remaining)
✓ Auto-approving tool (3 remaining)
✓ Auto-approving tool (2 remaining)
✓ Auto-approving tool (1 remaining)
✓ Auto-approving tool (0 remaining)
```

## Benefits

1. **No more manual confirmations** for multi-step operations
2. **Granular control** with auto-approve count
3. **Safety valve** with batch mode stop/pause
4. **Backward compatible** - existing behavior unchanged
5. **Works with existing trust mechanisms** (`--trust-all-tools`, etc.)

This should solve your 102-step confirmation problem while maintaining safety and control!
