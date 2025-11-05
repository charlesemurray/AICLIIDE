# Worktree TUI Features - Visual Demo

## Feature 1: Interactive Worktree Selector

### What You See on Startup

```
┌─ 📂 Select Worktree ────────────────────────────────────────────────┐
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
┌──────────────────────────────────────────────────────────────────────┐
│                                                                      │
│ → feature-auth [Feature]                                            │
│     /Users/dev/myproject/.worktrees/feature-auth                    │
│                                                                      │
│   fix-login-bug [Hotfix]                                            │
│     /Users/dev/myproject/.worktrees/fix-login-bug                   │
│                                                                      │
│   refactor-api [Refactor]                                           │
│     /Users/dev/myproject/.worktrees/refactor-api                    │
│                                                                      │
│   experiment-new-ui [Experiment]                                    │
│     /Users/dev/myproject/.worktrees/experiment-new-ui               │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
┌──────────────────────────────────────────────────────────────────────┐
│         ↑↓/jk: Navigate | Enter: Select | n: New | q: Cancel        │
└──────────────────────────────────────────────────────────────────────┘
```

### Keyboard Controls

| Key | Action |
|-----|--------|
| `↑` or `k` | Move up |
| `↓` or `j` | Move down |
| `Enter` | Select highlighted worktree |
| `n` | Create new worktree |
| `q` or `Esc` | Cancel / Skip |

### Creating New Worktree

Press `n`:
```
┌─ 📂 Select Worktree ────────────────────────────────────────────────┐
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
┌──────────────────────────────────────────────────────────────────────┐
│                                                                      │
│   feature-auth [Feature]                                            │
│     /Users/dev/myproject/.worktrees/feature-auth                    │
│                                                                      │
│   fix-login-bug [Hotfix]                                            │
│     /Users/dev/myproject/.worktrees/fix-login-bug                   │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
┌─ Create New ─────────────────────────────────────────────────────────┐
│ New worktree name: add-dark-mode_                                   │
└──────────────────────────────────────────────────────────────────────┘
```

Type name and press `Enter` to create.

---

## Feature 2: Context Stats Widget

### Always Visible in Top-Right Corner

While chatting, you'll see this in the top-right:

```
                                                    ┌────────────────────────┐
                                                    │ 🌳 feature-auth        │
                                                    │    [Feature]           │
                                                    │                        │
                                                    │ Context: 15%           │
                                                    │   30.0K/200.0K         │
                                                    │ Messages: 3            │
                                                    └────────────────────────┘

You: Let's add authentication to the app
Q: I can help with that...
```

### As Context Fills Up

```
                                                    ┌────────────────────────┐
                                                    │ 🌳 feature-auth        │
                                                    │    [Feature]           │
                                                    │                        │
                                                    │ Context: 75%           │  ← Yellow warning
                                                    │   150.0K/200.0K        │
                                                    │ Messages: 18           │
                                                    └────────────────────────┘
```

### Near Limit

```
                                                    ┌────────────────────────┐
                                                    │ 🌳 feature-auth        │
                                                    │    [Feature]           │
                                                    │                        │
                                                    │ Context: 95%           │  ← Red alert
                                                    │   190.0K/200.0K        │
                                                    │ Messages: 42           │
                                                    └────────────────────────┘
```

### Color Coding

| Usage | Color | Meaning |
|-------|-------|---------|
| 0-70% | 🟢 Green | Plenty of space |
| 70-90% | 🟡 Yellow | Getting full |
| 90-100% | 🔴 Red | Almost full |

---

## Complete Workflow Example

### 1. Start Q Chat
```bash
$ cd ~/myproject
$ q chat
```

### 2. Interactive Selector Appears
```
┌─ 📂 Select Worktree ────────────────────────────────────────────────┐
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
┌──────────────────────────────────────────────────────────────────────┐
│                                                                      │
│ → feature-auth [Feature]                                            │
│     /Users/dev/myproject/.worktrees/feature-auth                    │
│                                                                      │
│   fix-login-bug [Hotfix]                                            │
│     /Users/dev/myproject/.worktrees/fix-login-bug                   │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
┌──────────────────────────────────────────────────────────────────────┐
│         ↑↓/jk: Navigate | Enter: Select | n: New | q: Cancel        │
└──────────────────────────────────────────────────────────────────────┘
```

### 3. Press `n` to Create New
```
┌─ Create New ─────────────────────────────────────────────────────────┐
│ New worktree name: add-notifications_                               │
└──────────────────────────────────────────────────────────────────────┘
```

### 4. Type Name and Press Enter
```
✓ Created worktree at: /Users/dev/myproject/.worktrees/add-notifications
✓ Branch: add-notifications
✓ Changed to worktree directory
```

### 5. Chat Starts with Stats Widget
```
                                                    ┌────────────────────────┐
                                                    │ 🌳 add-notifications   │
                                                    │    [Feature]           │
                                                    │                        │
                                                    │ Context: 0%            │
                                                    │   0/200.0K             │
                                                    │ Messages: 0            │
                                                    └────────────────────────┘

You: _
```

### 6. As You Chat, Stats Update
```
                                                    ┌────────────────────────┐
                                                    │ 🌳 add-notifications   │
                                                    │    [Feature]           │
                                                    │                        │
                                                    │ Context: 12%           │
                                                    │   24.0K/200.0K         │
                                                    │ Messages: 5            │
                                                    └────────────────────────┘

You: Create a notification service
Q: I'll help you create a notification service...
```

---

## Session Type Detection

The selector automatically detects session types from branch names:

| Branch Pattern | Detected Type | Badge Color |
|---------------|---------------|-------------|
| `feature/*`, `feat/*` | Feature | 🟡 Yellow |
| `fix/*`, `hotfix/*` | Hotfix | 🔴 Red |
| `refactor/*` | Refactor | 🔵 Blue |
| `experiment/*` | Experiment | 🟣 Purple |
| Other | Development | ⚪ Gray |

### Examples
- `feature/add-auth` → `[Feature]`
- `fix/login-bug` → `[Hotfix]`
- `refactor/api-cleanup` → `[Refactor]`
- `experiment/new-ui` → `[Experiment]`
- `my-branch` → `[Development]`

---

## Fallback Behavior

### When TUI is Not Available

If the interactive selector can't run (no TTY, piped input, etc.), it automatically falls back to text input:

```
📂 Existing worktrees:
  1. feature-auth (/Users/dev/myproject/.worktrees/feature-auth)
  2. fix-login-bug (/Users/dev/myproject/.worktrees/fix-login-bug)

Create or select worktree [number/name/auto/N]: _
```

You can still:
- Type `1` or `2` to select by number
- Type a name to create new
- Type `auto` for auto-generated name
- Press `N` or `Enter` to skip

---

## Configuration

### Enable Stats Widget
```bash
export Q_SHOW_STATS=1
q chat
```

### Disable Interactive Selector
```bash
export Q_NO_TUI=1
q chat
```

### Both
```bash
export Q_SHOW_STATS=1
export Q_NO_TUI=1
q chat
```

---

## Benefits

### Interactive Selector
✅ **Faster** - No typing, just arrow keys
✅ **Visual** - See all options at once
✅ **Informative** - Session types and paths visible
✅ **Keyboard-driven** - Vim-style navigation (j/k)
✅ **Professional** - Modern TUI experience

### Context Stats
✅ **Awareness** - Always know your context usage
✅ **Warning** - Color-coded alerts when getting full
✅ **Tracking** - See message count grow
✅ **Context** - Know which worktree you're in
✅ **Non-intrusive** - Small widget, doesn't block view

---

## Comparison

### Before (Text-based)
```
📂 Existing worktrees:
  1. feature-auth (/repo/.worktrees/feature-auth)
  2. fix-login-bug (/repo/.worktrees/fix-login-bug)
  3. refactor-api (/repo/.worktrees/refactor-api)

Create or select worktree [number/name/auto/N]: _
```
- Must type number or name
- No visual feedback
- Easy to mistype
- No session type info
- No context stats

### After (TUI)
```
┌─ 📂 Select Worktree ────────────────────────────────────────────────┐
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
┌──────────────────────────────────────────────────────────────────────┐
│ → feature-auth [Feature]                                            │
│     /repo/.worktrees/feature-auth                                   │
│   fix-login-bug [Hotfix]                                            │
│     /repo/.worktrees/fix-login-bug                                  │
│   refactor-api [Refactor]                                           │
│     /repo/.worktrees/refactor-api                                   │
└──────────────────────────────────────────────────────────────────────┘
┌──────────────────────────────────────────────────────────────────────┐
│         ↑↓/jk: Navigate | Enter: Select | n: New | q: Cancel        │
└──────────────────────────────────────────────────────────────────────┘

                                                    ┌────────────────────────┐
                                                    │ 🌳 feature-auth        │
                                                    │    [Feature]           │
                                                    │ Context: 15%           │
                                                    │   30.0K/200.0K         │
                                                    │ Messages: 3            │
                                                    └────────────────────────┘
```
- Arrow key navigation
- Visual selection
- Session type badges
- Full paths visible
- Context stats always visible
- Professional appearance

---

## Next Steps

1. **Try it out** - Start `q chat` in a repo with worktrees
2. **Navigate** - Use arrow keys or j/k
3. **Create new** - Press `n` to create a worktree
4. **Watch stats** - Enable `Q_SHOW_STATS=1` to see context usage
5. **Provide feedback** - Let us know what you think!
