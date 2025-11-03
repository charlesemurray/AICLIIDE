# Parallel Sessions - Remediation Plan

**Date**: 2025-11-03  
**Total Estimated Effort**: 12-16 hours  
**Priority**: High - Required before production deployment

---

## Phase 1: Critical Fixes (P0) - 6 hours

**Goal**: Fix issues that could cause data loss or system instability

### Task 1.1: Consolidate WorktreeInfo Definitions (1.5h)

**Problem**: Two different `WorktreeInfo` structs with different fields

**Solution**:
```rust
// git/worktree.rs - Rename to GitWorktreeInfo
pub struct GitWorktreeInfo {
    pub path: PathBuf,
    pub branch: String,
    pub commit: String,
}

// session/metadata.rs - Keep as WorktreeInfo (no changes)

// Add conversion in git/worktree.rs
impl GitWorktreeInfo {
    pub fn to_session_info(&self, repo_root: PathBuf, merge_target: String) -> session::WorktreeInfo {
        session::WorktreeInfo {
            path: self.path.clone(),
            branch: self.branch.clone(),
            repo_root,
            is_temporary: false,
            merge_target,
        }
    }
}
```

**Files to modify**:
- `crates/chat-cli/src/git/worktree.rs` - Rename struct, add conversion
- `crates/chat-cli/src/git/mod.rs` - Update exports
- Update all imports (5-6 files)

**Testing**: Verify compilation, run existing tests

---

### Task 1.2: Fix Silent Error Swallowing (2h)

**Problem**: Errors silently ignored in session scanning and cleanup

**Solution**:

```rust
// session_scanner.rs
pub fn scan_worktree_sessions(repo_root: &Path) -> Result<(Vec<SessionMetadata>, Vec<String>)> {
    let worktrees = list_worktrees(repo_root)?;
    let mut sessions = Vec::new();
    let mut errors = Vec::new();
    
    for wt in worktrees {
        match load_from_worktree(&wt.path) {
            Ok(metadata) => sessions.push(metadata),
            Err(e) => errors.push(format!("{}: {}", wt.path.display(), e)),
        }
    }
    
    Ok((sessions, errors))
}

// sessions.rs - Cleanup command
if let Ok((sessions, scan_errors)) = get_current_repo_sessions() {
    if !scan_errors.is_empty() {
        eprintln!("⚠️  Scan warnings:");
        for err in scan_errors.iter().take(3) {
            eprintln!("  • {}", err);
        }
    }
    
    for session in sessions {
        // ... cleanup logic ...
        if let Err(e) = remove_worktree(&wt.path) {
            eprintln!("  ✗ Failed to remove {}: {}", wt.branch, e);
        } else {
            println!("  ✓ Removed worktree: {}", wt.branch);
            cleaned += 1;
        }
    }
}
```

**Files to modify**:
- `crates/chat-cli/src/cli/chat/session_scanner.rs` - Return errors
- `crates/chat-cli/src/cli/chat/cli/sessions.rs` - Handle errors
- Update all callers (3-4 locations)

**Testing**: Add test for error reporting

---

### Task 1.3: Fix Incomplete SessionMetadata Creation (1h)

**Problem**: `persist_to_worktree()` creates metadata with empty first_message

**Solution**:

```rust
// worktree_session.rs - Change signature
pub fn persist_to_worktree(
    worktree_path: &Path,
    metadata: &SessionMetadata,  // Accept full metadata
) -> Result<()> {
    let session_dir = worktree_path.join(".amazonq");
    std::fs::create_dir_all(&session_dir)?;
    let session_file = session_dir.join("session.json");
    let json = serde_json::to_string_pretty(metadata)?;
    std::fs::write(session_file, json)?;
    Ok(())
}

// mod.rs - Update callers
let metadata = SessionMetadata::new(&conversation_id, &first_message)
    .with_worktree(wt_info.clone());
persist_to_worktree(&path, &metadata)?;
```

**Files to modify**:
- `crates/chat-cli/src/cli/chat/worktree_session.rs` - Change signature
- `crates/chat-cli/src/cli/chat/mod.rs` - Update 2 call sites

**Testing**: Verify persisted metadata is complete

---

### Task 1.4: Add Rollback to Merge Operations (1h)

**Problem**: Merge failure leaves repo in inconsistent state

**Solution**:

```rust
// merge_workflow.rs
pub fn merge_branch(repo_root: &Path, branch: &str, target: &str) -> Result<()> {
    // Save current branch for rollback
    let current_branch = get_current_branch(repo_root)?;
    
    // Switch to target branch
    if let Err(e) = checkout_branch(repo_root, target) {
        return Err(e);
    }
    
    // Attempt merge
    let merge_result = Command::new("git")
        .arg("-C").arg(repo_root)
        .arg("merge").arg(branch)
        .arg("--no-ff")
        .arg("-m").arg(format!("Merge branch '{}'", branch))
        .status();
    
    match merge_result {
        Ok(status) if status.success() => Ok(()),
        _ => {
            // Rollback: return to original branch
            let _ = checkout_branch(repo_root, &current_branch);
            bail!("Merge failed - conflicts need resolution. Returned to {}", current_branch)
        }
    }
}

fn get_current_branch(repo_root: &Path) -> Result<String> {
    let output = Command::new("git")
        .arg("-C").arg(repo_root)
        .arg("branch").arg("--show-current")
        .output()?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn checkout_branch(repo_root: &Path, branch: &str) -> Result<()> {
    let status = Command::new("git")
        .arg("-C").arg(repo_root)
        .arg("checkout").arg(branch)
        .status()?;
    if !status.success() {
        bail!("Failed to checkout {}", branch);
    }
    Ok(())
}
```

**Files to modify**:
- `crates/chat-cli/src/cli/chat/merge_workflow.rs` - Add rollback logic

**Testing**: Test merge failure scenarios

---

### Task 1.5: Add Confirmation for Destructive Operations (0.5h)

**Problem**: Cleanup deletes worktrees without confirmation

**Solution**:

```rust
// sessions.rs
SessionsSubcommand::Cleanup { completed, older_than, force } => {
    // ... scan logic ...
    
    if sessions_to_clean.is_empty() {
        println!("  No sessions to clean up");
        return Ok(ChatState::PromptUser { skip_printing_tools: true });
    }
    
    // Show what will be deleted
    println!("  Will remove {} worktree(s):", sessions_to_clean.len());
    for session in &sessions_to_clean {
        if let Some(wt) = &session.worktree_info {
            println!("    • {} ({})", wt.branch, wt.path.display());
        }
    }
    
    // Require confirmation unless --force
    if !force {
        eprint!("\nProceed with cleanup? [y/N]: ");
        io::stderr().flush().ok();
        let mut input = String::new();
        io::stdin().read_line(&mut input).ok();
        if input.trim().to_lowercase() != "y" {
            println!("Cleanup cancelled");
            return Ok(ChatState::PromptUser { skip_printing_tools: true });
        }
    }
    
    // Proceed with cleanup...
}

// Update CLI definition
Cleanup {
    #[arg(long)]
    completed: bool,
    #[arg(long)]
    older_than: Option<u32>,
    #[arg(long)]
    force: bool,  // Add force flag
}
```

**Files to modify**:
- `crates/chat-cli/src/cli/chat/cli/sessions.rs` - Add confirmation

**Testing**: Test interactive confirmation

---

## Phase 2: High Priority Fixes (P1) - 4 hours

**Goal**: Improve reliability and maintainability

### Task 2.1: Remove All unwrap() Calls (1.5h)

**Files to audit and fix**:
- `git/worktree.rs` - 5 unwrap() calls in parsing
- `cli/chat/merge_workflow.rs` - 1 unwrap_or()
- `cli/chat/mod.rs` - 1 unwrap_or()

**Pattern**:
```rust
// Before
let value = line.strip_prefix("worktree ").unwrap().to_string();

// After
let value = line.strip_prefix("worktree ")
    .ok_or_else(|| GitError::ParseError("Invalid worktree line".into()))?
    .to_string();
```

---

### Task 2.2: Add Atomic Writes for Session Persistence (1h)

**Solution**:
```rust
// worktree_session.rs
pub fn persist_to_worktree(worktree_path: &Path, metadata: &SessionMetadata) -> Result<()> {
    let session_dir = worktree_path.join(".amazonq");
    std::fs::create_dir_all(&session_dir)?;
    
    let session_file = session_dir.join("session.json");
    let temp_file = session_dir.join(".session.json.tmp");
    
    // Write to temp file
    let json = serde_json::to_string_pretty(metadata)?;
    std::fs::write(&temp_file, json)?;
    
    // Atomic rename
    std::fs::rename(&temp_file, &session_file)?;
    
    Ok(())
}
```

**Files to modify**:
- `crates/chat-cli/src/cli/chat/worktree_session.rs`

---

### Task 2.3: Add Input Validation (1h)

**Solution**:
```rust
// worktree_session.rs
pub fn persist_to_worktree(worktree_path: &Path, metadata: &SessionMetadata) -> Result<()> {
    // Validate inputs
    if !worktree_path.exists() {
        bail!("Worktree path does not exist: {}", worktree_path.display());
    }
    if metadata.id.is_empty() {
        bail!("Session ID cannot be empty");
    }
    if metadata.worktree_info.is_none() {
        bail!("Cannot persist non-worktree session to worktree");
    }
    
    // ... rest of function
}

// branch_naming.rs
pub fn sanitize_branch_name(input: &str) -> Result<String> {
    if input.trim().is_empty() {
        bail!("Branch name cannot be empty");
    }
    
    let sanitized = /* ... sanitization logic ... */;
    
    // Validate result
    if sanitized.is_empty() {
        bail!("Branch name '{}' contains no valid characters", input);
    }
    if sanitized.starts_with('-') || sanitized.ends_with('-') {
        bail!("Branch name cannot start or end with '-'");
    }
    
    Ok(sanitized)
}
```

**Files to modify**:
- `crates/chat-cli/src/cli/chat/worktree_session.rs`
- `crates/chat-cli/src/cli/chat/branch_naming.rs`

---

### Task 2.4: Extract Constants for Magic Numbers (0.5h)

**Solution**:
```rust
// branch_naming.rs
const MIN_WORD_LENGTH: usize = 3;
const MAX_CONTEXT_WORDS: usize = 4;
const MAX_BRANCH_NAME_LENGTH: usize = 50;
const MAX_CONFLICT_RETRIES: u32 = 100;

pub fn generate_from_conversation(first_message: &str, session_type: Option<&str>) -> String {
    let words: Vec<&str> = first_message
        .split_whitespace()
        .filter(|w| w.len() > MIN_WORD_LENGTH)
        .take(MAX_CONTEXT_WORDS)
        .collect();
    // ...
}
```

**Files to modify**:
- `crates/chat-cli/src/cli/chat/branch_naming.rs`
- `crates/chat-cli/src/git/worktree.rs`

---

## Phase 3: Documentation & Testing (P1) - 3 hours

### Task 3.1: Add Doc Comments (1.5h)

**Pattern**:
```rust
/// Creates a new git worktree for isolated development.
///
/// # Arguments
/// * `repo_root` - Path to the main repository
/// * `name` - Branch name for the worktree
/// * `base_branch` - Branch to base the new worktree on
/// * `path` - Optional custom path (defaults to sibling directory)
///
/// # Returns
/// Path to the created worktree
///
/// # Errors
/// * `GitError::BranchExists` - If branch already exists
/// * `GitError::WorktreeExists` - If worktree path already exists
/// * `GitError::CommandFailed` - If git command fails
///
/// # Example
/// ```no_run
/// let path = create_worktree(
///     Path::new("/repo"),
///     "feature-branch",
///     "main",
///     None
/// )?;
/// ```
pub fn create_worktree(...) -> Result<PathBuf> { ... }
```

**Files to document**:
- All public functions in all 7 files
- Module-level docs for each file

---

### Task 3.2: Add Integration Tests (1.5h)

**Tests to add**:
```rust
// tests/worktree_integration_test.rs

#[test]
fn test_full_worktree_lifecycle() {
    // 1. Create worktree
    // 2. Persist session
    // 3. Load session
    // 4. Merge worktree
    // 5. Cleanup
}

#[test]
fn test_merge_with_conflicts() {
    // 1. Create worktree
    // 2. Make conflicting changes
    // 3. Attempt merge
    // 4. Verify rollback
}

#[test]
fn test_cleanup_with_errors() {
    // 1. Create multiple worktrees
    // 2. Corrupt one session file
    // 3. Run cleanup
    // 4. Verify partial success + error reporting
}

#[test]
fn test_session_persistence_atomic() {
    // 1. Start writing session
    // 2. Simulate crash
    // 3. Verify no corruption
}
```

---

## Phase 4: Medium Priority (P2) - 2-3 hours

### Task 4.1: Improve Error Messages (1h)

Add context to all errors:
```rust
.context(format!("Failed to create worktree '{}' at {}", branch, path.display()))?
```

### Task 4.2: Add Caching for Git Operations (1h)

```rust
pub struct WorktreeCache {
    worktrees: Arc<Mutex<Option<(Instant, Vec<GitWorktreeInfo>)>>>,
    ttl: Duration,
}
```

### Task 4.3: Standardize Error Handling (1h)

Use `thiserror` consistently across all modules.

---

## Implementation Schedule

### Day 1 (6 hours)
- Morning: Phase 1 Tasks 1.1-1.3 (4.5h)
- Afternoon: Phase 1 Tasks 1.4-1.5 (1.5h)

### Day 2 (5 hours)
- Morning: Phase 2 Tasks 2.1-2.2 (2.5h)
- Afternoon: Phase 2 Tasks 2.3-2.4 (1.5h)
- Evening: Phase 3 Task 3.1 start (1h)

### Day 3 (4 hours)
- Morning: Phase 3 Task 3.1 complete (0.5h)
- Morning: Phase 3 Task 3.2 (1.5h)
- Afternoon: Phase 4 (2h)

**Total: 15 hours over 3 days**

---

## Success Criteria

### Phase 1 Complete When:
- ✅ No duplicate type definitions
- ✅ All errors logged or reported
- ✅ Session metadata always complete
- ✅ Merge failures rollback cleanly
- ✅ Destructive ops require confirmation
- ✅ All tests pass

### Phase 2 Complete When:
- ✅ No unwrap() calls in production code
- ✅ All file writes are atomic
- ✅ All inputs validated
- ✅ No magic numbers
- ✅ All tests pass

### Phase 3 Complete When:
- ✅ All public APIs documented
- ✅ Integration tests cover main workflows
- ✅ Error paths tested
- ✅ Test coverage >80%

### Ready for Production When:
- ✅ All Phase 1 & 2 complete
- ✅ Phase 3 at least 80% complete
- ✅ Code review approved
- ✅ Manual testing completed
- ✅ Performance acceptable

---

## Risk Mitigation

### Risk: Breaking existing functionality
**Mitigation**: 
- Run full test suite after each task
- Manual testing of happy paths
- Keep changes minimal and focused

### Risk: Schedule overrun
**Mitigation**:
- Phase 1 is mandatory, others can be deferred
- Each task is independently valuable
- Can pause after any phase

### Risk: Introducing new bugs
**Mitigation**:
- Add tests before refactoring
- Use compiler to catch issues
- Incremental changes with verification

---

## Post-Remediation

### Monitoring
- Track error rates in production
- Monitor worktree creation/cleanup success rates
- Watch for performance issues

### Future Improvements
- Add metrics/telemetry
- Implement undo functionality
- Add progress indicators
- Support custom merge strategies

---

## Approval Required

**Before starting**: Review this plan with team lead
**After Phase 1**: Demo to stakeholders
**After Phase 2**: Security review
**After Phase 3**: Final approval for production
