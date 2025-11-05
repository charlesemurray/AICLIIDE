# Worktree UI Audit & Improvement Recommendations

## Executive Summary

The worktree feature currently uses basic `println!` and `eprintln!` for output, missing opportunities to leverage the new theme/UI library that provides:
- Semantic color coding
- Consistent formatting
- Status indicators
- Command highlighting
- Session type displays

**Recommendation**: Refactor worktree output to use `CommandOutputFormatter` and `SessionDisplay` for a professional, consistent UX.

---

## Current State Analysis

### What We Have (New UI Library)

**Location**: `crates/chat-cli/src/theme/`

#### 1. CommandOutputFormatter
```rust
// Semantic formatting
formatter.success("message")     // ✓ Green
formatter.error("message")       // ✗ Red  
formatter.warning("message")     // ⚠ Yellow
formatter.info("message")        // ℹ Blue
formatter.command("git merge")   // Cyan highlight
formatter.file_path("/path")     // Cyan
formatter.header("Section")      // Bold emphasis
formatter.list_item("item")      // • Bullet
```

#### 2. SessionDisplay & SessionType
```rust
SessionType::Feature      // Requires worktree
SessionType::Refactor     // Requires worktree
SessionType::Experiment   // Requires worktree
SessionStatus::Active     // With state transitions
```

#### 3. TerminalUI
```rust
terminal_ui.show_session_list()    // Formatted list
terminal_ui.show_switch_message()  // Transition messages
terminal_ui.render_indicator()     // Status indicators
```

### What Worktrees Currently Use

**Raw output examples**:
```rust
eprintln!("✓ Created worktree at: {}", path.display());
eprintln!("📂 Existing worktrees:");
println!("⚠️  Conflicts detected in {} file(s):", conflicts.len());
println!("❌ No worktree session found to merge");
```

**Problems**:
- ❌ Inconsistent emoji usage
- ❌ No semantic color coding
- ❌ Manual formatting
- ❌ No command highlighting
- ❌ No session type integration
- ❌ Plain text paths

---

## Improvement Opportunities

### 1. Worktree Creation (HIGH IMPACT)

#### Current
```
📂 Existing worktrees:
  1. feature-auth (/repo/.worktrees/feature-auth)
  2. fix-login (/repo/.worktrees/fix-login)

Create or select worktree [number/name/auto/N]:
```

#### Proposed
```rust
use crate::theme::command_output::CommandOutputFormatter;
use crate::theme::session::SessionType;

fn display_worktree_selection(formatter: &CommandOutputFormatter, worktrees: &[WorktreeInfo]) {
    println!("{}", formatter.header("📂 Existing Worktrees"));
    println!();
    
    for (idx, wt) in worktrees.iter().enumerate() {
        let session_type = detect_session_type(&wt.branch);
        let type_badge = format!("[{}]", session_type.display_name());
        
        println!("  {}. {} {} {}",
            formatter.emphasis((idx + 1).to_string()),
            formatter.primary(&wt.branch),
            formatter.secondary(type_badge),
            formatter.file_path(wt.path.display().to_string())
        );
    }
    
    println!();
    println!("{}", formatter.secondary("Create or select worktree [number/name/auto/N]:"));
}
```

**Output**:
```
📂 Existing Worktrees

  1. feature-auth [Feature] /repo/.worktrees/feature-auth
  2. fix-login [Hotfix] /repo/.worktrees/fix-login
  3. refactor-api [Refactor] /repo/.worktrees/refactor-api

Create or select worktree [number/name/auto/N]:
```

**Benefits**:
- ✅ Session type badges
- ✅ Color-coded paths
- ✅ Numbered emphasis
- ✅ Consistent formatting

---

### 2. Worktree Creation Success (HIGH IMPACT)

#### Current
```
✓ Created worktree at: /repo/.worktrees/feature-auth
✓ Branch: feature-auth
✓ Changed to worktree directory
```

#### Proposed
```rust
fn display_worktree_created(formatter: &CommandOutputFormatter, session: &SessionMetadata) {
    let wt = session.worktree_info.as_ref().unwrap();
    
    println!();
    println!("{}", formatter.status_ok("Worktree created successfully"));
    println!();
    println!("  {} {}", 
        formatter.secondary("Type:"),
        formatter.emphasis(session.session_type.display_name())
    );
    println!("  {} {}", 
        formatter.secondary("Branch:"),
        formatter.command(&wt.branch)
    );
    println!("  {} {}", 
        formatter.secondary("Path:"),
        formatter.file_path(wt.path.display().to_string())
    );
    println!("  {} {}", 
        formatter.secondary("Target:"),
        formatter.command(&wt.merge_target)
    );
    println!();
    println!("{}", formatter.info("💡 Use /sessions complete when work is done"));
}
```

**Output**:
```
✓ Worktree created successfully

  Type:   Feature
  Branch: feature-auth
  Path:   /repo/.worktrees/feature-auth
  Target: main

💡 Use /sessions complete when work is done
```

**Benefits**:
- ✅ Structured information
- ✅ Color-coded commands
- ✅ Helpful next steps
- ✅ Professional appearance

---

### 3. Conflict Detection (CRITICAL IMPACT)

#### Current
```
⚠️  Conflicts detected in 2 file(s):
  • src/auth.rs
  • src/login.rs

📋 Resolution options:
  1. Resolve manually:
     git checkout main
     git merge feature-auth
     # Fix conflicts, then:
     git add .
     git commit

  2. Force merge (requires manual resolution):
     /sessions merge --force

  3. Cancel and continue working:
     /sessions list
```

#### Proposed
```rust
fn display_merge_conflicts(formatter: &CommandOutputFormatter, conflicts: &[String], branch: &str, target: &str) {
    println!();
    println!("{}", formatter.warning(format!("⚠  Conflicts detected in {} file(s)", conflicts.len())));
    println!();
    
    for file in conflicts.iter().take(5) {
        println!("{}", formatter.list_item(formatter.file_path(file)));
    }
    if conflicts.len() > 5 {
        println!("{}", formatter.secondary(format!("  ... and {} more", conflicts.len() - 5)));
    }
    
    println!();
    println!("{}", formatter.header("📋 Resolution Options"));
    println!();
    
    println!("{}", formatter.emphasis("1. Resolve manually:"));
    println!("   {}", formatter.command(format!("git checkout {}", target)));
    println!("   {}", formatter.command(format!("git merge {}", branch)));
    println!("   {}", formatter.secondary("# Fix conflicts, then:"));
    println!("   {}", formatter.command("git add ."));
    println!("   {}", formatter.command("git commit"));
    println!();
    
    println!("{}", formatter.emphasis("2. Force merge:"));
    println!("   {}", formatter.command("/sessions merge --force"));
    println!();
    
    println!("{}", formatter.emphasis("3. Continue working:"));
    println!("   {}", formatter.command("/sessions list"));
    println!();
}
```

**Output**:
```
⚠  Conflicts detected in 2 file(s)

  • src/auth.rs
  • src/login.rs

📋 Resolution Options

1. Resolve manually:
   git checkout main
   git merge feature-auth
   # Fix conflicts, then:
   git add .
   git commit

2. Force merge:
   /sessions merge --force

3. Continue working:
   /sessions list
```

**Benefits**:
- ✅ Commands highlighted in cyan
- ✅ File paths color-coded
- ✅ Clear visual hierarchy
- ✅ Easier to scan

---

### 4. Merge Success (HIGH IMPACT)

#### Current
```
🔀 Preparing to merge worktree session...
Merging feature-auth into main...
✓ Merge successful!
✓ Cleaned up worktree and branch
✓ Session marked as completed
```

#### Proposed
```rust
fn display_merge_success(formatter: &CommandOutputFormatter, branch: &str, target: &str) {
    println!();
    println!("{}", formatter.status_ok("Merge completed successfully"));
    println!();
    
    println!("  {} {} {} {}",
        formatter.command(branch),
        formatter.secondary("→"),
        formatter.command(target),
        formatter.success("✓")
    );
    println!();
    
    println!("{}", formatter.list_item("Worktree removed"));
    println!("{}", formatter.list_item("Branch deleted"));
    println!("{}", formatter.list_item("Session marked as completed"));
    println!();
    
    println!("{}", formatter.info("💡 Changes are now in your main branch"));
}
```

**Output**:
```
✓ Merge completed successfully

  feature-auth → main ✓

  • Worktree removed
  • Branch deleted
  • Session marked as completed

💡 Changes are now in your main branch
```

**Benefits**:
- ✅ Clear success indication
- ✅ Visual merge flow
- ✅ Checklist of actions
- ✅ Helpful context

---

### 5. Session List with Types (MEDIUM IMPACT)

#### Current
```
🔍 Scanning for worktree sessions...
  Found 3 worktree session(s):
  🟢 abc123 (branch: feature-auth, status: Active)
  ✅ def456 (branch: bugfix-login, status: Completed)
  📦 ghi789 (branch: refactor-api, status: Archived)
```

#### Proposed
```rust
fn display_session_list(formatter: &CommandOutputFormatter, sessions: &[SessionMetadata]) {
    println!();
    println!("{}", formatter.header("📋 Active Sessions"));
    println!();
    
    if sessions.is_empty() {
        println!("{}", formatter.secondary("  No active sessions"));
        println!();
        println!("{}", formatter.info("💡 Create one with: /worktree or q chat --worktree <name>"));
        return;
    }
    
    for session in sessions {
        let status_icon = match session.status {
            SessionStatus::Active => formatter.success("●"),
            SessionStatus::Completed => formatter.success("✓"),
            SessionStatus::Archived => formatter.secondary("□"),
            _ => formatter.secondary("○"),
        };
        
        let type_badge = format!("[{}]", session.session_type.display_name());
        let wt = session.worktree_info.as_ref().unwrap();
        
        println!("  {} {} {} {}",
            status_icon,
            formatter.emphasis(&wt.branch),
            formatter.secondary(type_badge),
            formatter.file_path(wt.path.display().to_string())
        );
        
        if session.message_count > 0 {
            println!("    {} {} messages",
                formatter.secondary("└─"),
                formatter.secondary(session.message_count.to_string())
            );
        }
    }
    
    println!();
}
```

**Output**:
```
📋 Active Sessions

  ● feature-auth [Feature] /repo/.worktrees/feature-auth
    └─ 12 messages
  ✓ bugfix-login [Hotfix] /repo/.worktrees/bugfix-login
    └─ 5 messages
  □ refactor-api [Refactor] /repo/.worktrees/refactor-api
    └─ 23 messages
```

**Benefits**:
- ✅ Session type badges
- ✅ Message counts
- ✅ Tree-style formatting
- ✅ Color-coded status

---

### 6. Error Messages (HIGH IMPACT)

#### Current
```
❌ No worktree session found to merge
   Create one with: /sessions create <name>
   Or list existing: /sessions list
```

#### Proposed
```rust
fn display_no_session_error(formatter: &CommandOutputFormatter) {
    println!();
    println!("{}", formatter.status_error("No worktree session found"));
    println!();
    println!("{}", formatter.secondary("Available actions:"));
    println!("{}", formatter.list_item(format!("Create: {}", formatter.command("/worktree"))));
    println!("{}", formatter.list_item(format!("List: {}", formatter.command("/sessions list"))));
    println!("{}", formatter.list_item(format!("Scan: {}", formatter.command("/sessions scan"))));
    println!();
}
```

**Output**:
```
✗ No worktree session found

Available actions:
  • Create: /worktree
  • List: /sessions list
  • Scan: /sessions scan
```

**Benefits**:
- ✅ Clear error indication
- ✅ Actionable commands highlighted
- ✅ Consistent formatting
- ✅ Easy to scan

---

## Implementation Plan

### Phase 1: Core Formatter Integration (2 hours)
1. Create `WorktreeOutputFormatter` wrapper
2. Initialize in worktree commands
3. Add helper methods for common patterns

```rust
// crates/chat-cli/src/cli/chat/worktree_output.rs
pub struct WorktreeOutputFormatter {
    formatter: CommandOutputFormatter,
}

impl WorktreeOutputFormatter {
    pub fn display_selection(&self, worktrees: &[WorktreeInfo]) { /* ... */ }
    pub fn display_created(&self, session: &SessionMetadata) { /* ... */ }
    pub fn display_conflicts(&self, conflicts: &[String], branch: &str, target: &str) { /* ... */ }
    pub fn display_merge_success(&self, branch: &str, target: &str) { /* ... */ }
    pub fn display_session_list(&self, sessions: &[SessionMetadata]) { /* ... */ }
}
```

### Phase 2: Replace Output Calls (3 hours)
1. Replace worktree creation output
2. Replace conflict detection output
3. Replace merge output
4. Replace session list output
5. Replace error messages

### Phase 3: Add Session Type Detection (2 hours)
1. Detect session type from branch name patterns
2. Add type badges to displays
3. Color-code by type

```rust
fn detect_session_type(branch: &str) -> SessionType {
    if branch.starts_with("feature/") || branch.starts_with("feat/") {
        SessionType::Feature
    } else if branch.starts_with("fix/") || branch.starts_with("hotfix/") {
        SessionType::Hotfix
    } else if branch.starts_with("refactor/") {
        SessionType::Refactor
    } else {
        SessionType::Development
    }
}
```

### Phase 4: Add Interactive Elements (2 hours)
1. Add progress indicators for long operations
2. Add confirmation prompts with formatting
3. Add help hints

### Phase 5: Testing & Polish (1 hour)
1. Test all output scenarios
2. Verify colors in different terminals
3. Update documentation

**Total Effort**: 10 hours (1.5 days)

---

## Expected Impact

### User Experience
- **Before**: Plain text, inconsistent formatting
- **After**: Professional, color-coded, easy to scan

### Discoverability
- **Before**: Commands mentioned in plain text
- **After**: Commands highlighted, stand out visually

### Error Recovery
- **Before**: Errors with basic suggestions
- **After**: Errors with clear, highlighted action paths

### Consistency
- **Before**: Different styles across features
- **After**: Unified theme across all Q CLI

---

## Quick Wins (Can Do Today)

### 1. Worktree Creation Success (30 min)
Replace basic success message with formatted output

### 2. Conflict Detection (45 min)
Highlight git commands in conflict resolution guide

### 3. Error Messages (30 min)
Use `status_error()` and `command()` formatters

**Total**: 1.75 hours for immediate visual improvement

---

## Recommendation

**Priority**: HIGH

**Rationale**:
1. Worktrees are a key differentiator
2. Current output looks basic compared to new UI
3. Low effort, high visual impact
4. Improves discoverability and UX

**Next Steps**:
1. Approve approach
2. Start with Quick Wins
3. Complete full implementation
4. Update screenshots/docs

---

## Code Example: Before & After

### Before
```rust
eprintln!("✓ Created worktree at: {}", path.display());
eprintln!("✓ Branch: {}", unique_branch);
eprintln!("✓ Changed to worktree directory");
```

### After
```rust
let formatter = WorktreeOutputFormatter::new(&status_colors, &ui_colors);
formatter.display_created(&session);
```

**Result**: Consistent, professional, color-coded output with session type badges and helpful hints.
