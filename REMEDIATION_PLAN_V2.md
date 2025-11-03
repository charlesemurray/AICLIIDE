# Parallel Sessions - Remediation Plan V2 (TDD + Git Workflow)

**Date**: 2025-11-03  
**Approach**: Test-Driven Development + Small Commits  
**Total Estimated Effort**: 15-18 hours  

---

## Principles

1. **Red-Green-Refactor**: Write failing test → Make it pass → Refactor
2. **Small Commits**: Each commit is atomic and deployable
3. **Test First**: No production code without a test
4. **Incremental**: Each step adds value independently

---

## Phase 1: Critical Fixes (P0) - 8 hours

### Task 1.1: Consolidate WorktreeInfo Definitions

#### Step 1.1.1: Write Test for Type Confusion (15 min)
```rust
// tests/worktree_type_test.rs
#[test]
fn test_git_worktree_to_session_worktree_conversion() {
    let git_wt = GitWorktreeInfo {
        path: PathBuf::from("/tmp/wt"),
        branch: "feature".into(),
        commit: "abc123".into(),
    };
    
    let session_wt = git_wt.to_session_info(
        PathBuf::from("/tmp/repo"),
        "main".into()
    );
    
    assert_eq!(session_wt.path, git_wt.path);
    assert_eq!(session_wt.branch, git_wt.branch);
    assert_eq!(session_wt.repo_root, PathBuf::from("/tmp/repo"));
}
```

**Commit**: `test: add conversion test for WorktreeInfo types`

#### Step 1.1.2: Rename git::WorktreeInfo (20 min)
```rust
// git/worktree.rs
pub struct GitWorktreeInfo {  // Renamed
    pub path: PathBuf,
    pub branch: String,
    pub commit: String,
}
```

**Run tests**: Should fail (compilation errors)  
**Commit**: `refactor: rename git::WorktreeInfo to GitWorktreeInfo`

#### Step 1.1.3: Fix Compilation Errors (30 min)
Update all imports and usages one file at a time:
- `git/mod.rs`
- `git/worktree.rs` internal uses
- `cli/chat/session_scanner.rs`

**Run tests**: Should compile but test still fails  
**Commit**: `fix: update imports for GitWorktreeInfo rename`

#### Step 1.1.4: Implement Conversion (15 min)
```rust
impl GitWorktreeInfo {
    pub fn to_session_info(&self, repo_root: PathBuf, merge_target: String) -> WorktreeInfo {
        WorktreeInfo {
            path: self.path.clone(),
            branch: self.branch.clone(),
            repo_root,
            is_temporary: false,
            merge_target,
        }
    }
}
```

**Run tests**: Should pass  
**Commit**: `feat: add conversion from GitWorktreeInfo to WorktreeInfo`

**Total**: 1.5h, 4 commits

---

### Task 1.2: Fix Silent Error Swallowing

#### Step 1.2.1: Write Test for Error Reporting (20 min)
```rust
// tests/session_scanner_test.rs
#[test]
fn test_scan_reports_corrupted_sessions() {
    let temp_dir = TempDir::new().unwrap();
    let repo = setup_test_repo(&temp_dir);
    
    // Create worktree with corrupted session file
    create_worktree(&repo, "test", "main", None).unwrap();
    let wt_path = repo.parent().unwrap().join("repo-test");
    std::fs::write(wt_path.join(".amazonq/session.json"), "invalid json").unwrap();
    
    let (sessions, errors) = scan_worktree_sessions(&repo).unwrap();
    
    assert_eq!(sessions.len(), 0);
    assert_eq!(errors.len(), 1);
    assert!(errors[0].contains("invalid json"));
}
```

**Run tests**: Should fail (function signature doesn't match)  
**Commit**: `test: add test for error reporting in session scanner`

#### Step 1.2.2: Change scan_worktree_sessions Signature (15 min)
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
```

**Run tests**: Should fail (callers need updating)  
**Commit**: `refactor: return errors from scan_worktree_sessions`

#### Step 1.2.3: Update get_current_repo_sessions (10 min)
```rust
pub fn get_current_repo_sessions() -> Result<(Vec<SessionMetadata>, Vec<String>)> {
    let current_dir = std::env::current_dir()?;
    let git_ctx = detect_git_context(&current_dir)?;
    scan_worktree_sessions(&git_ctx.repo_root)
}
```

**Run tests**: Should fail (CLI callers need updating)  
**Commit**: `refactor: update get_current_repo_sessions to return errors`

#### Step 1.2.4: Update CLI Scan Command (15 min)
```rust
// sessions.rs
SessionsSubcommand::Scan => {
    match get_current_repo_sessions() {
        Ok((sessions, errors)) => {
            if !errors.is_empty() {
                eprintln!("⚠️  Scan warnings:");
                for err in errors.iter().take(3) {
                    eprintln!("  • {}", err);
                }
            }
            // ... display sessions
        }
    }
}
```

**Run tests**: Should pass  
**Commit**: `feat: display scan errors in CLI`

#### Step 1.2.5: Update CLI Cleanup Command (20 min)
```rust
SessionsSubcommand::Cleanup { ... } => {
    let (sessions, scan_errors) = get_current_repo_sessions()?;
    
    if !scan_errors.is_empty() {
        eprintln!("⚠️  Some sessions could not be scanned:");
        for err in scan_errors.iter().take(3) {
            eprintln!("  • {}", err);
        }
    }
    
    for session in sessions {
        if should_clean {
            match remove_worktree(&wt.path) {
                Ok(_) => {
                    println!("  ✓ Removed: {}", wt.branch);
                    cleaned += 1;
                },
                Err(e) => {
                    eprintln!("  ✗ Failed to remove {}: {}", wt.branch, e);
                    failed += 1;
                }
            }
        }
    }
    
    println!("✓ Cleaned: {}, Failed: {}", cleaned, failed);
}
```

**Run tests**: Should pass  
**Commit**: `feat: report cleanup failures in CLI`

#### Step 1.2.6: Update Worktrees Command (10 min)
Similar pattern for worktrees command.

**Commit**: `feat: report errors in worktrees command`

**Total**: 1.5h, 6 commits

---

### Task 1.3: Fix Incomplete SessionMetadata Creation

#### Step 1.3.1: Write Test for Complete Metadata (15 min)
```rust
#[test]
fn test_persist_preserves_all_metadata_fields() {
    let temp_dir = TempDir::new().unwrap();
    let wt_path = temp_dir.path();
    
    let metadata = SessionMetadata::new("test-id", "First message")
        .with_worktree(WorktreeInfo { /* ... */ });
    
    persist_to_worktree(&wt_path, &metadata).unwrap();
    
    let loaded = load_from_worktree(&wt_path).unwrap();
    assert_eq!(loaded.id, "test-id");
    assert_eq!(loaded.first_message, "First message");
    assert!(loaded.worktree_info.is_some());
}
```

**Run tests**: Should fail (function signature doesn't match)  
**Commit**: `test: verify persist_to_worktree preserves all fields`

#### Step 1.3.2: Change persist_to_worktree Signature (10 min)
```rust
pub fn persist_to_worktree(
    worktree_path: &Path,
    metadata: &SessionMetadata,
) -> Result<()> {
    let session_dir = worktree_path.join(".amazonq");
    std::fs::create_dir_all(&session_dir)?;
    let session_file = session_dir.join("session.json");
    let json = serde_json::to_string_pretty(metadata)?;
    std::fs::write(session_file, json)?;
    Ok(())
}
```

**Run tests**: Should fail (callers need updating)  
**Commit**: `refactor: accept full SessionMetadata in persist_to_worktree`

#### Step 1.3.3: Update First Caller in mod.rs (15 min)
```rust
// mod.rs - Create strategy
let metadata = SessionMetadata::new(&conversation_id, "")  // Will fix message later
    .with_worktree(wt_info.clone());
persist_to_worktree(&path, &metadata)?;
```

**Run tests**: Should pass  
**Commit**: `fix: update Create strategy to use new persist signature`

#### Step 1.3.4: Update Second Caller in mod.rs (15 min)
```rust
// mod.rs - Ask strategy
let metadata = SessionMetadata::new(&conversation_id, "")
    .with_worktree(wt_info.clone());
persist_to_worktree(&path, &metadata)?;
```

**Run tests**: Should pass  
**Commit**: `fix: update Ask strategy to use new persist signature`

#### Step 1.3.5: Add TODO for First Message (5 min)
```rust
// TODO: Pass first_message from user input
let metadata = SessionMetadata::new(&conversation_id, "")
    .with_worktree(wt_info.clone());
```

**Commit**: `docs: add TODO for first_message in worktree creation`

**Total**: 1h, 5 commits

---

### Task 1.4: Add Rollback to Merge Operations

#### Step 1.4.1: Write Test for Merge Failure Rollback (25 min)
```rust
#[test]
fn test_merge_failure_returns_to_original_branch() {
    let temp_dir = TempDir::new().unwrap();
    let repo = setup_test_repo_with_conflict(&temp_dir);
    
    // Start on main
    checkout_branch(&repo, "main").unwrap();
    
    // Attempt merge that will fail
    let result = merge_branch(&repo, "conflicting-branch", "main");
    
    assert!(result.is_err());
    
    // Should still be on main
    let current = get_current_branch(&repo).unwrap();
    assert_eq!(current, "main");
}
```

**Run tests**: Should fail (functions don't exist)  
**Commit**: `test: add test for merge rollback on failure`

#### Step 1.4.2: Implement get_current_branch (10 min)
```rust
fn get_current_branch(repo_root: &Path) -> Result<String> {
    let output = Command::new("git")
        .arg("-C").arg(repo_root)
        .arg("branch").arg("--show-current")
        .output()?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
```

**Run tests**: Should still fail (merge_branch not updated)  
**Commit**: `feat: add get_current_branch helper`

#### Step 1.4.3: Extract checkout_branch (10 min)
```rust
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

**Run tests**: Should still fail  
**Commit**: `refactor: extract checkout_branch helper`

#### Step 1.4.4: Update merge_branch with Rollback (20 min)
```rust
pub fn merge_branch(repo_root: &Path, branch: &str, target: &str) -> Result<()> {
    let original_branch = get_current_branch(repo_root)?;
    
    if let Err(e) = checkout_branch(repo_root, target) {
        return Err(e);
    }
    
    let merge_result = Command::new("git")
        .arg("-C").arg(repo_root)
        .arg("merge").arg(branch)
        .arg("--no-ff")
        .arg("-m").arg(format!("Merge branch '{}'", branch))
        .status();
    
    match merge_result {
        Ok(status) if status.success() => Ok(()),
        _ => {
            let _ = checkout_branch(repo_root, &original_branch);
            bail!("Merge failed. Returned to {}", original_branch)
        }
    }
}
```

**Run tests**: Should pass  
**Commit**: `feat: add rollback to merge_branch on failure`

**Total**: 1h, 4 commits

---

### Task 1.5: Add Confirmation for Destructive Operations

#### Step 1.5.1: Add --force Flag to CLI (10 min)
```rust
Cleanup {
    #[arg(long)]
    completed: bool,
    #[arg(long)]
    older_than: Option<u32>,
    #[arg(long)]
    force: bool,
}
```

**Run tests**: Should fail (handler needs updating)  
**Commit**: `feat: add --force flag to cleanup command`

#### Step 1.5.2: Write Test for Confirmation Logic (20 min)
```rust
#[test]
fn test_cleanup_requires_confirmation_without_force() {
    // Mock stdin with "n"
    // Run cleanup without --force
    // Verify no worktrees removed
}

#[test]
fn test_cleanup_skips_confirmation_with_force() {
    // Run cleanup with --force
    // Verify worktrees removed without prompt
}
```

**Run tests**: Should fail  
**Commit**: `test: add tests for cleanup confirmation`

#### Step 1.5.3: Implement Confirmation Logic (25 min)
```rust
SessionsSubcommand::Cleanup { completed, older_than, force } => {
    let (sessions, _) = get_current_repo_sessions()?;
    
    let to_clean: Vec<_> = sessions.iter()
        .filter(|s| should_clean(s, *completed, *older_than))
        .collect();
    
    if to_clean.is_empty() {
        println!("No sessions to clean up");
        return Ok(ChatState::PromptUser { skip_printing_tools: true });
    }
    
    // Show what will be deleted
    println!("Will remove {} worktree(s):", to_clean.len());
    for session in &to_clean {
        if let Some(wt) = &session.worktree_info {
            println!("  • {} ({})", wt.branch, wt.path.display());
        }
    }
    
    // Require confirmation unless --force
    if !*force {
        use std::io::{self, Write};
        eprint!("\nProceed? [y/N]: ");
        io::stderr().flush().ok();
        let mut input = String::new();
        io::stdin().read_line(&mut input).ok();
        if input.trim().to_lowercase() != "y" {
            println!("Cancelled");
            return Ok(ChatState::PromptUser { skip_printing_tools: true });
        }
    }
    
    // Proceed with cleanup
    // ...
}
```

**Run tests**: Should pass  
**Commit**: `feat: add confirmation prompt to cleanup command`

**Total**: 1h, 3 commits

---

## Phase 1 Summary

**Total Time**: 6 hours  
**Total Commits**: 22 commits  
**Test Coverage**: All critical paths tested

**Git Log Preview**:
```
feat: add confirmation prompt to cleanup command
test: add tests for cleanup confirmation
feat: add --force flag to cleanup command
feat: add rollback to merge_branch on failure
refactor: extract checkout_branch helper
feat: add get_current_branch helper
test: add test for merge rollback on failure
docs: add TODO for first_message in worktree creation
fix: update Ask strategy to use new persist signature
fix: update Create strategy to use new persist signature
refactor: accept full SessionMetadata in persist_to_worktree
test: verify persist_to_worktree preserves all fields
feat: report errors in worktrees command
feat: report cleanup failures in CLI
feat: display scan errors in CLI
refactor: update get_current_repo_sessions to return errors
refactor: return errors from scan_worktree_sessions
test: add test for error reporting in session scanner
feat: add conversion from GitWorktreeInfo to WorktreeInfo
fix: update imports for GitWorktreeInfo rename
refactor: rename git::WorktreeInfo to GitWorktreeInfo
test: add conversion test for WorktreeInfo types
```

---

## Phase 2: High Priority Fixes (P1) - 5 hours

### Task 2.1: Remove unwrap() Calls

#### Step 2.1.1: Write Test for Parse Error Handling (15 min)
```rust
#[test]
fn test_parse_worktree_list_handles_malformed_input() {
    let malformed = "worktree\nHEAD abc123\n";  // Missing path
    let result = parse_worktree_list(malformed);
    assert!(result.is_err());
}
```

**Commit**: `test: add test for malformed worktree list parsing`

#### Step 2.1.2: Fix First unwrap() in parse_worktree_list (15 min)
```rust
if line.starts_with("worktree ") {
    current_path = line.strip_prefix("worktree ")
        .map(|s| s.to_string());
}
```

**Run tests**: Should pass  
**Commit**: `fix: remove unwrap from worktree path parsing`

#### Step 2.1.3-2.1.7: Fix Remaining unwrap() Calls (1h)
One commit per unwrap() removal, following same pattern.

**Total**: 1.5h, 6 commits

---

### Task 2.2: Add Atomic Writes

#### Step 2.2.1: Write Test for Atomic Write (20 min)
```rust
#[test]
fn test_persist_is_atomic() {
    // Start persist
    // Simulate crash (drop temp file)
    // Verify original file unchanged or new file complete
}
```

**Commit**: `test: add test for atomic session persistence`

#### Step 2.2.2: Implement Atomic Write (25 min)
```rust
pub fn persist_to_worktree(worktree_path: &Path, metadata: &SessionMetadata) -> Result<()> {
    let session_dir = worktree_path.join(".amazonq");
    std::fs::create_dir_all(&session_dir)?;
    
    let session_file = session_dir.join("session.json");
    let temp_file = session_dir.join(".session.json.tmp");
    
    let json = serde_json::to_string_pretty(metadata)?;
    std::fs::write(&temp_file, json)?;
    std::fs::rename(&temp_file, &session_file)?;
    
    Ok(())
}
```

**Run tests**: Should pass  
**Commit**: `feat: implement atomic writes for session persistence`

**Total**: 0.75h, 2 commits

---

### Task 2.3: Add Input Validation

#### Step 2.3.1: Write Validation Tests (30 min)
```rust
#[test]
fn test_persist_rejects_empty_session_id() { ... }

#[test]
fn test_persist_rejects_nonexistent_path() { ... }

#[test]
fn test_sanitize_rejects_empty_name() { ... }

#[test]
fn test_sanitize_rejects_invalid_chars_only() { ... }
```

**Commit**: `test: add input validation tests`

#### Step 2.3.2: Add Validation to persist_to_worktree (20 min)
```rust
pub fn persist_to_worktree(...) -> Result<()> {
    if !worktree_path.exists() {
        bail!("Worktree path does not exist: {}", worktree_path.display());
    }
    if metadata.id.is_empty() {
        bail!("Session ID cannot be empty");
    }
    // ...
}
```

**Run tests**: Should pass  
**Commit**: `feat: add input validation to persist_to_worktree`

#### Step 2.3.3: Add Validation to sanitize_branch_name (20 min)
```rust
pub fn sanitize_branch_name(input: &str) -> Result<String> {
    if input.trim().is_empty() {
        bail!("Branch name cannot be empty");
    }
    
    let sanitized = /* ... */;
    
    if sanitized.is_empty() {
        bail!("Branch name contains no valid characters");
    }
    if sanitized.starts_with('-') {
        bail!("Branch name cannot start with '-'");
    }
    
    Ok(sanitized)
}
```

**Run tests**: Should fail (callers need updating)  
**Commit**: `feat: add validation to sanitize_branch_name`

#### Step 2.3.4: Update Callers (30 min)
Update all callers to handle Result return type.

**Commit**: `fix: handle validation errors in branch name callers`

**Total**: 1.5h, 4 commits

---

### Task 2.4: Extract Constants

#### Step 2.4.1: Define Constants (20 min)
```rust
// branch_naming.rs
const MIN_WORD_LENGTH: usize = 3;
const MAX_CONTEXT_WORDS: usize = 4;
const MAX_BRANCH_NAME_LENGTH: usize = 50;
const MAX_CONFLICT_RETRIES: u32 = 100;
```

**Commit**: `refactor: extract magic numbers to named constants`

#### Step 2.4.2: Use Constants (15 min)
Replace all magic numbers with constants.

**Run tests**: Should pass  
**Commit**: `refactor: use named constants instead of magic numbers`

**Total**: 0.5h, 2 commits

---

## Phase 2 Summary

**Total Time**: 4.25h  
**Total Commits**: 14 commits

---

## Phase 3: Documentation & Testing (P1) - 4 hours

### Task 3.1: Add Doc Comments

#### Step 3.1.1: Document git/worktree.rs (30 min)
Add doc comments to all public functions.

**Commit**: `docs: add documentation to git/worktree module`

#### Step 3.1.2-3.1.7: Document Remaining Modules (2.5h)
One commit per module.

**Total**: 3h, 7 commits

---

### Task 3.2: Add Integration Tests

#### Step 3.2.1: Test Full Lifecycle (30 min)
```rust
#[test]
fn test_worktree_full_lifecycle() {
    // Create → Persist → Load → Merge → Cleanup
}
```

**Commit**: `test: add full worktree lifecycle integration test`

#### Step 3.2.2: Test Merge with Conflicts (20 min)
**Commit**: `test: add merge conflict integration test`

#### Step 3.2.3: Test Cleanup with Errors (20 min)
**Commit**: `test: add cleanup error handling integration test`

**Total**: 1h, 3 commits

---

## Phase 3 Summary

**Total Time**: 4h  
**Total Commits**: 10 commits

---

## Complete Summary

**Total Time**: 14.25 hours  
**Total Commits**: 46 commits  
**Average Commit Size**: ~20 minutes of work

### Commit Frequency
- Phase 1: 22 commits (6 hours) = 1 commit every 16 minutes
- Phase 2: 14 commits (4.25 hours) = 1 commit every 18 minutes
- Phase 3: 10 commits (4 hours) = 1 commit every 24 minutes

### Test Coverage
- Unit tests: 25+ new tests
- Integration tests: 3 new tests
- All critical paths covered

### Git Workflow
```bash
# Each task follows:
git checkout -b fix/task-name
# Write test
git add tests/
git commit -m "test: description"
# Implement
git add src/
git commit -m "feat/fix: description"
# Refactor if needed
git commit -m "refactor: description"
git push origin fix/task-name
# Create PR, review, merge
```

### Rollback Strategy
Each commit is independently revertable. If a problem is found:
```bash
git revert <commit-hash>
```

### Continuous Integration
After each commit:
- ✅ Run `cargo test`
- ✅ Run `cargo clippy`
- ✅ Run `cargo fmt --check`
- ✅ Verify binary builds

---

## Daily Schedule

### Day 1 (4 hours)
- Task 1.1: WorktreeInfo consolidation (1.5h, 4 commits)
- Task 1.2: Error reporting (1.5h, 6 commits)
- Task 1.3: Complete metadata (1h, 5 commits)

### Day 2 (5 hours)
- Task 1.4: Merge rollback (1h, 4 commits)
- Task 1.5: Confirmation (1h, 3 commits)
- Task 2.1: Remove unwrap (1.5h, 6 commits)
- Task 2.2: Atomic writes (0.75h, 2 commits)
- Task 2.3: Validation (0.75h, 2 commits)

### Day 3 (5 hours)
- Task 2.3: Validation complete (0.75h, 2 commits)
- Task 2.4: Constants (0.5h, 2 commits)
- Task 3.1: Documentation (3h, 7 commits)
- Task 3.2: Integration tests (1h, 3 commits)

**Total: 14 hours, 46 commits**
