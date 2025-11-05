# Worktree TUI Implementation Guide

## Features Implemented

### 1. Interactive Worktree Selector ✅
**File**: `crates/chat-cli/src/cli/chat/worktree_selector.rs`

**Features**:
- Visual list with arrow key navigation (↑↓ or j/k)
- Session type badges `[Feature]`, `[Hotfix]`, `[Refactor]`
- Highlighted selection with `→` indicator
- Create new worktree with `n` key
- Cancel with `q` or `Esc`
- Enter to select

**Usage**:
```rust
use crate::cli::chat::worktree_selector::{WorktreeSelector, SelectorAction};

let worktrees = list_worktrees(&repo_root)?;
let selector = WorktreeSelector::new(worktrees);

match selector.run()? {
    SelectorAction::Selected(idx) => {
        // User selected existing worktree at index
    },
    SelectorAction::CreateNew(name) => {
        // User wants to create new worktree with name
    },
    SelectorAction::Cancel => {
        // User cancelled
    }
}
```

### 2. Context Stats Widget ✅
**File**: `crates/chat-cli/src/cli/chat/context_stats_widget.rs`

**Features**:
- Shows current worktree name and type
- Context window usage (tokens used/limit)
- Color-coded usage: Green (<70%), Yellow (70-90%), Red (>90%)
- Message count
- Renders in top-right corner

**Usage**:
```rust
use crate::cli::chat::context_stats_widget::ContextStats;

let mut stats = ContextStats::new();

// Update worktree info
stats.update_worktree("feature-auth".to_string(), "Feature".to_string());

// Update token usage
stats.update_tokens(125_000);

// Increment message count
stats.increment_messages();

// Render in top-right corner
stats.render()?;
```

---

## Integration Steps

### Step 1: Add Dependencies

**File**: `crates/chat-cli/Cargo.toml`

```toml
[dependencies]
atty = "0.2"  # For detecting if stdin is a TTY
```

### Step 2: Update mod.rs

**File**: `crates/chat-cli/src/cli/chat/mod.rs`

Add module declarations:
```rust
mod worktree_selector;
mod context_stats_widget;
```

### Step 3: Integrate Selector at Startup

**Location**: `crates/chat-cli/src/cli/chat/mod.rs` around line 640

**Current code**:
```rust
// List existing worktrees
let existing_worktrees = list_worktrees(&ctx.repo_root).unwrap_or_default();

if !existing_worktrees.is_empty() {
    eprintln!("\n📂 Existing worktrees:");
    for (idx, wt) in existing_worktrees.iter().enumerate() {
        eprintln!("  {}. {} ({})", idx + 1, wt.branch, wt.path.display());
    }
}

eprint!("Create or select worktree [number/name/auto/N]: ");
// ... text input handling
```

**Replace with**:
```rust
use crate::cli::chat::worktree_selector::{WorktreeSelector, SelectorAction};

// List existing worktrees
let existing_worktrees = list_worktrees(&ctx.repo_root).unwrap_or_default();

// Use interactive selector if worktrees exist and stdin is a TTY
let selection = if !existing_worktrees.is_empty() && atty::is(atty::Stream::Stdin) {
    let selector = WorktreeSelector::new(existing_worktrees.clone());
    match selector.run() {
        Ok(action) => Some(action),
        Err(e) => {
            eprintln!("Selector error: {}, falling back to text input", e);
            None
        }
    }
} else {
    None
};

match selection {
    Some(SelectorAction::Selected(idx)) => {
        let selected = &existing_worktrees[idx];
        eprintln!("✓ Using existing worktree: {}", selected.branch);
        
        if std::env::set_current_dir(&selected.path).is_ok() {
            eprintln!("✓ Changed to worktree directory");
        }
        
        Some(selected.path.clone())
    },
    Some(SelectorAction::CreateNew(branch_name)) => {
        // Create new worktree with branch_name
        // ... existing creation logic
    },
    Some(SelectorAction::Cancel) | None => {
        // Fall back to text input
        // ... existing text input logic
    }
}
```

### Step 4: Add Context Stats to ChatSession

**Option A: Simple Integration (Recommended)**

Add stats as a field in ChatSession:
```rust
pub struct ChatSession {
    // ... existing fields ...
    context_stats: Option<Arc<Mutex<ContextStats>>>,
}
```

Initialize in `new()`:
```rust
let context_stats = if std::env::var("Q_SHOW_STATS").is_ok() {
    Some(Arc::new(Mutex::new(ContextStats::new())))
} else {
    None
};
```

Update stats after each message:
```rust
if let Some(ref stats) = self.context_stats {
    let mut stats = stats.lock().unwrap();
    stats.increment_messages();
    stats.update_tokens(self.conversation.token_count());
    let _ = stats.render();
}
```

**Option B: Standalone Renderer**

Create a background task that periodically renders stats:
```rust
// In ChatArgs::execute()
let stats = Arc::new(Mutex::new(ContextStats::new()));
let stats_clone = stats.clone();

// Spawn renderer task
tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(1));
    loop {
        interval.tick().await;
        let stats = stats_clone.lock().unwrap();
        let _ = stats.render();
    }
});
```

### Step 5: Update Stats on Worktree Selection

When a worktree is selected or created:
```rust
if let Some(ref stats) = context_stats {
    let mut stats = stats.lock().unwrap();
    let session_type = detect_session_type(&branch_name);
    stats.update_worktree(branch_name.clone(), session_type.display_name().to_string());
}
```

---

## Testing

### Test Interactive Selector

```bash
# In a git repo with worktrees
cd /path/to/repo
q chat

# Should show interactive selector
# Test:
# - Arrow keys navigate
# - Enter selects
# - n creates new
# - q cancels
```

### Test Context Stats

```bash
# Enable stats
export Q_SHOW_STATS=1

# Start chat in worktree
q chat --worktree feature-test

# Should see stats widget in top-right:
# ┌────────────────────────┐
# │ 🌳 feature-test        │
# │    [Feature]           │
# │                        │
# │ Context: 15%           │
# │   30.0K/200.0K         │
# │ Messages: 5            │
# └────────────────────────┘
```

---

## Configuration

### Environment Variables

- `Q_SHOW_STATS=1` - Enable context stats widget
- `Q_NO_TUI=1` - Disable interactive selector, use text input

### Fallback Behavior

The selector automatically falls back to text input if:
- No TTY detected (piped input)
- Selector fails to initialize
- `Q_NO_TUI=1` is set
- No existing worktrees found

---

## Visual Examples

### Interactive Selector
```
┌─ 📂 Select Worktree ────────────────────────────────┐
│                                                      │
└──────────────────────────────────────────────────────┘
┌──────────────────────────────────────────────────────┐
│                                                      │
│ → feature-auth [Feature]                            │
│     /repo/.worktrees/feature-auth                   │
│                                                      │
│   fix-login [Hotfix]                                │
│     /repo/.worktrees/fix-login                      │
│                                                      │
│   refactor-api [Refactor]                           │
│     /repo/.worktrees/refactor-api                   │
│                                                      │
└──────────────────────────────────────────────────────┘
┌──────────────────────────────────────────────────────┐
│ ↑↓/jk: Navigate | Enter: Select | n: New | q: Cancel│
└──────────────────────────────────────────────────────┘
```

### Context Stats Widget (Top-Right)
```
┌────────────────────────┐
│ 🌳 feature-auth        │
│    [Feature]           │
│                        │
│ Context: 62%           │
│   124.0K/200.0K        │
│ Messages: 12           │
└────────────────────────┘
```

---

## Next Steps

1. **Test the selector** - Verify it works with existing worktrees
2. **Add stats integration** - Choose Option A or B above
3. **Add keyboard shortcut** - Maybe `/stats` to toggle widget
4. **Add to /worktree command** - Use selector for in-chat creation too
5. **Add stats to dashboard** - If implementing full dashboard later

---

## Files Created

- ✅ `crates/chat-cli/src/cli/chat/worktree_selector.rs` (227 lines)
- ✅ `crates/chat-cli/src/cli/chat/context_stats_widget.rs` (186 lines)
- ✅ Integration code in `mod.rs` (partial)

**Total**: ~400 lines of code

**Estimated Integration Time**: 2-3 hours

---

## Benefits

### Interactive Selector
- ✅ No more typing numbers or names
- ✅ Visual navigation
- ✅ Session type badges
- ✅ Professional UX
- ✅ Keyboard-driven workflow

### Context Stats
- ✅ Always visible context usage
- ✅ Know when approaching limit
- ✅ Track message count
- ✅ See current worktree at a glance
- ✅ Color-coded warnings

---

## Troubleshooting

### Selector doesn't appear
- Check if stdin is a TTY: `atty::is(atty::Stream::Stdin)`
- Check for `Q_NO_TUI=1` environment variable
- Verify worktrees exist

### Stats widget not rendering
- Check if `Q_SHOW_STATS=1` is set
- Verify terminal supports cursor positioning
- Check for terminal size errors

### Selector crashes
- Falls back to text input automatically
- Check error message in stderr
- Verify ratatui and crossterm versions match

---

## Future Enhancements

1. **Add file preview** - Show changed files in selector
2. **Add commit count** - Show commits ahead/behind
3. **Add last activity** - Show when worktree was last used
4. **Add stats history** - Graph token usage over time
5. **Add stats export** - Save stats to file
6. **Add stats comparison** - Compare sessions
