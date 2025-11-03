# Critical Gaps Implementation Plan

**Priority**: BLOCKER  
**Estimated Time**: 8-12 hours  
**Status**: Ready to implement

---

## Gap 1: Resume Workflow (CRITICAL)

### Implementation Steps:

1. **Add worktree detection on startup** (1h)
```rust
// In ChatArgs::execute(), before creating conversation_id
let (conversation_id, resume_from_worktree) = {
    let current_dir = os.env.current_dir()?;
    if let Ok(git_ctx) = detect_git_context(&current_dir) {
        if git_ctx.is_worktree {
            // Try to load session from worktree
            if let Ok(metadata) = load_from_worktree(&current_dir) {
                eprintln!("✓ Resuming session in worktree: {}", git_ctx.branch_name);
                (metadata.id, true)
            } else {
                (uuid::Uuid::new_v4().to_string(), false)
            }
        } else {
            (uuid::Uuid::new_v4().to_string(), false)
        }
    } else {
        (uuid::Uuid::new_v4().to_string(), false)
    }
};
```

2. **Add prompt for resume** (30min)
```rust
if git_ctx.is_worktree && session_file.exists() {
    eprint!("Found existing session. Resume? [Y/n]: ");
    // ... handle input
}
```

3. **Test resume workflow** (30min)

---

## Gap 2: Conflict Resolution Guide (HIGH)

### Implementation Steps:

1. **Enhance conflict detection output** (1h)
```rust
if !conflicts.is_empty() {
    println!("⚠️  Conflicts detected in {} file(s):", conflicts.len());
    for file in conflicts.iter().take(5) {
        println!("  • {}", file);
    }
    println!("\nTo resolve conflicts:");
    println!("  1. Edit the conflicting files above");
    println!("  2. Mark as resolved: git add <file>");
    println!("  3. Complete merge: git merge --continue");
    println!("  4. Or abort: git merge --abort");
    println!("\nOr use --force to merge anyway (manual resolution required)");
    return Ok(ChatState::PromptUser { skip_printing_tools: true });
}
```

2. **Add --continue flag** (2h)
```rust
Merge {
    branch: Option<String>,
    #[arg(long)]
    force: bool,
    #[arg(long)]
    continue_merge: bool,  // NEW
}
```

3. **Implement continue logic** (1h)
```rust
if *continue_merge {
    // Check if merge is in progress
    // Complete the merge
    // Cleanup worktree
}
```

---

## Gap 3: Improve Error Messages (MEDIUM)

### Implementation Steps:

1. **Add context to all errors** (2h)
```rust
// Before
bail!("Branch name cannot be empty");

// After
bail!("Branch name cannot be empty.\n\
       Try: q chat --worktree feature-name\n\
       Or: q chat --worktree auto (for AI-generated name)");
```

2. **Create error message helper** (1h)
```rust
fn error_with_help(msg: &str, help: &str) -> eyre::Error {
    eyre::eyre!("{}\n\nHelp: {}", msg, help)
}
```

3. **Update all error sites** (1h)

---

## Gap 4: Session Lifecycle Management (HIGH)

### Implementation Steps:

1. **Add session state transitions** (2h)
```rust
impl SessionMetadata {
    pub fn mark_completed(&mut self) {
        self.status = SessionStatus::Completed;
        self.last_active = OffsetDateTime::now_utc();
    }
    
    pub fn archive(&mut self) {
        self.status = SessionStatus::Archived;
    }
}
```

2. **Update merge to mark completed** (30min)
```rust
// In cleanup_after_merge
let mut metadata = load_from_worktree(&wt.path)?;
metadata.mark_completed();
// Save to archive location
```

3. **Add /sessions close command** (1h)
```rust
Close {
    /// Session ID or branch name
    name: String,
}
```

4. **Implement close handler** (1h)
```rust
SessionsSubcommand::Close { name } => {
    // Find session
    // Mark as completed
    // Optionally cleanup worktree
}
```

---

## Implementation Order

### Day 1 (4 hours)
1. Gap 1: Resume Workflow (2h)
2. Gap 2: Conflict Resolution Guide (2h)

### Day 2 (4 hours)
3. Gap 3: Improve Error Messages (2h)
4. Gap 4: Session Lifecycle (2h)

---

## Testing Plan

### Resume Workflow
```bash
# Test 1: Create and resume
q chat --worktree test-feature
# ... work ...
exit
cd /repo-test-feature
q chat  # Should resume

# Test 2: Prompt for resume
q chat --worktree test-feature
# ... work ...
exit
cd /repo-test-feature
q chat  # Should prompt: "Resume? [Y/n]"
```

### Conflict Resolution
```bash
# Test 1: Detect conflicts
# Create conflicting changes
/sessions merge
# Should show clear instructions

# Test 2: Continue after resolution
# Resolve conflicts manually
/sessions merge --continue
# Should complete merge
```

### Error Messages
```bash
# Test 1: Empty branch name
q chat --worktree ""
# Should show helpful error with examples

# Test 2: Invalid path
# Should show what makes it invalid
```

### Session Lifecycle
```bash
# Test 1: Close session
/sessions close test-feature
# Should mark as completed

# Test 2: List shows status
/sessions list
# Should show Active/Completed/Archived
```

---

## Success Criteria

- ✅ Users can resume sessions in worktrees automatically
- ✅ Conflict resolution has clear next steps
- ✅ All error messages include actionable guidance
- ✅ Sessions have proper lifecycle (Active → Completed → Archived)
- ✅ All tests pass
- ✅ Documentation updated

---

## Files to Modify

1. `crates/chat-cli/src/cli/chat/mod.rs` - Resume logic
2. `crates/chat-cli/src/cli/chat/cli/sessions.rs` - Merge improvements, Close command
3. `crates/chat-cli/src/cli/chat/merge_workflow.rs` - Conflict guidance
4. `crates/chat-cli/src/cli/chat/branch_naming.rs` - Better errors
5. `crates/chat-cli/src/session/metadata.rs` - Lifecycle methods

---

## Risk Mitigation

- Each gap is independent - can be implemented separately
- All changes are additive - no breaking changes
- Comprehensive testing plan for each gap
- Can deploy incrementally (Gap 1 first, then 2, etc.)

---

## Post-Implementation

After these 4 gaps are closed:
- Feature will be truly production-ready
- User experience will be complete
- No critical workflows missing
- Clear path for users in all scenarios

**Estimated to raise grade from B- (80%) to A- (92%)**
