# Cortex Memory - Feedback System Integration Plan

**Date**: 2025-11-03  
**Status**: Ready to Implement  
**Estimated Time**: 55 minutes

## Overview

Complete the integration of the Cortex Memory feedback system and expose circuit breaker status to users. This closes the gap between the implemented backend and CLI user interface.

## Current State

### ✅ Implemented (Backend)
- FeedbackManager with SQLite storage
- Circuit breaker with 3-state pattern
- Quality filtering and deduplication
- All core memory operations

### ❌ Missing (CLI Integration)
- Feedback command is a stub (prints success but doesn't record)
- FeedbackManager not instantiated in ChatSession
- Circuit breaker status not visible to users
- Memory IDs not displayed (needed for feedback)
- Feedback stats not shown in stats command

## Implementation Plan

### Phase 1: Wire Up Feedback System (30 min)

#### Task 1.1: Add FeedbackManager to ChatSession (15 min)
**File**: `crates/chat-cli/src/cli/chat/session.rs`

**Changes**:
1. Add `feedback_manager: Option<FeedbackManager>` field to ChatSession struct
2. Initialize FeedbackManager when cortex is initialized
3. Use same database directory as cortex memory

**Code**:
```rust
pub struct ChatSession {
    // ... existing fields
    pub cortex: Option<CortexMemory>,
    pub feedback_manager: Option<FeedbackManager>,  // NEW
}

// In initialization:
let feedback_manager = if memory_enabled {
    let feedback_db = memory_dir.join("feedback.db");
    Some(FeedbackManager::new(feedback_db)?)
} else {
    None
};
```

#### Task 1.2: Implement Feedback Command Handler (10 min)
**File**: `crates/chat-cli/src/cli/chat/cli/mod.rs`

**Location**: Line ~540 (MemorySubcommand::Feedback)

**Replace stub with**:
```rust
MemorySubcommand::Feedback(args) => {
    if args.helpful == args.not_helpful {
        execute!(
            session.stderr,
            StyledText::error_fg(),
            style::Print("Error: Specify either --helpful or --not-helpful\n"),
            StyledText::reset(),
        )?;
    } else {
        if let Some(ref feedback_mgr) = session.feedback_manager {
            let helpful = args.helpful;
            match feedback_mgr.record_feedback(&args.memory_id, helpful) {
                Ok(_) => {
                    execute!(
                        session.stderr,
                        StyledText::success_fg(),
                        style::Print(format!(
                            "✓ Feedback recorded for memory {}\n",
                            args.memory_id
                        )),
                        StyledText::reset(),
                    )?;
                },
                Err(e) => {
                    execute!(
                        session.stderr,
                        StyledText::error_fg(),
                        style::Print(format!("Error: {}\n", e)),
                        StyledText::reset(),
                    )?;
                },
            }
        } else {
            execute!(
                session.stderr,
                StyledText::warning_fg(),
                style::Print("Memory system not initialized\n"),
                StyledText::reset(),
            )?;
        }
    }
}
```

#### Task 1.3: Add Feedback Stats to Stats Command (5 min)
**File**: `crates/chat-cli/src/cli/chat/cli/mod.rs`

**Location**: Line ~450 (MemorySubcommand::Stats)

**Add after existing stats**:
```rust
// Add feedback statistics
if let Some(ref feedback_mgr) = session.feedback_manager {
    if let Ok((helpful, not_helpful)) = feedback_mgr.get_stats() {
        execute!(
            session.stderr,
            style::Print(format!(
                "  Feedback: {} helpful, {} not helpful\n",
                helpful, not_helpful
            )),
        )?;
    }
}
```

---

### Phase 2: Expose Circuit Breaker Status (15 min)

#### Task 2.1: Add Circuit Breaker Getters (5 min)
**File**: `crates/cortex-memory/src/qcli_api.rs`

**Add to CortexMemory impl**:
```rust
/// Get circuit breaker state
pub fn circuit_breaker_state(&self) -> CircuitState {
    self.circuit_breaker.state()
}

/// Get circuit breaker failure count
pub fn circuit_breaker_failures(&self) -> u32 {
    self.circuit_breaker.failure_count()
}
```

#### Task 2.2: Display Circuit Status in Stats (10 min)
**File**: `crates/chat-cli/src/cli/chat/cli/mod.rs`

**Location**: Line ~450 (MemorySubcommand::Stats)

**Add after memory stats**:
```rust
// Add circuit breaker status
let cb_state = cortex.circuit_breaker_state();
let cb_failures = cortex.circuit_breaker_failures();

execute!(
    session.stderr,
    style::Print(format!(
        "  Circuit Breaker: {:?} ({} failures)\n",
        cb_state, cb_failures
    )),
)?;

// Warning if circuit is open
if cb_state == cortex_memory::CircuitState::Open {
    execute!(
        session.stderr,
        StyledText::warning_fg(),
        style::Print("  ⚠️  Memory operations temporarily disabled\n"),
        StyledText::reset(),
    )?;
}
```

---

### Phase 3: Display Memory IDs (10 min)

#### Task 3.1: Show IDs in List Command (5 min)
**File**: `crates/chat-cli/src/cli/chat/cli/mod.rs`

**Location**: Line ~384 (MemorySubcommand::List)

**Change**:
```rust
for (i, item) in items.iter().enumerate() {
    let preview = if item.content.len() > 80 {
        format!("{}...", &item.content[..77])
    } else {
        item.content.clone()
    };
    execute!(
        session.stderr,
        style::Print(format!(
            "{}. [{}] {}\n",
            i + 1,
            &item.id[..8],  // Show first 8 chars
            preview
        )),
    )?;
}
```

#### Task 3.2: Show IDs in Search Results (5 min)
**File**: `crates/chat-cli/src/cli/chat/cli/mod.rs`

**Location**: Line ~425 (MemorySubcommand::Search)

**Change**:
```rust
for item in items {
    execute!(
        session.stderr,
        style::Print(format!(
            "  • [{}] {} (score: {:.2})\n",
            &item.id[..8],
            item.content,
            item.score
        )),
    )?;
}
```

---

## Testing Checklist

### Test 1: Feedback Recording
- [ ] `/memory list` shows memory IDs
- [ ] `/memory feedback <id> --helpful` records positive feedback
- [ ] `/memory feedback <id> --not-helpful` records negative feedback
- [ ] `/memory stats` shows feedback counts
- [ ] Error shown if neither --helpful nor --not-helpful specified
- [ ] Error shown if memory system disabled

### Test 2: Circuit Breaker Status
- [ ] `/memory stats` shows circuit breaker state (Closed)
- [ ] Circuit breaker state visible when Open
- [ ] Warning shown when circuit is Open
- [ ] Failure count displayed

### Test 3: Memory IDs
- [ ] `/memory list` shows IDs in format `[abc12345]`
- [ ] `/memory search` shows IDs in results
- [ ] IDs are copyable for feedback command

### Test 4: Integration
- [ ] Build succeeds: `cargo check`
- [ ] Tests pass: `cargo test -p cortex-memory`
- [ ] No clippy warnings: `cargo clippy -p cortex-memory`

---

## Files Modified

1. `crates/chat-cli/src/cli/chat/session.rs`
   - Add FeedbackManager field
   - Initialize FeedbackManager

2. `crates/chat-cli/src/cli/chat/cli/mod.rs`
   - Implement Feedback command handler
   - Add feedback stats to Stats command
   - Add circuit breaker status to Stats command
   - Show memory IDs in List command
   - Show memory IDs in Search command

3. `crates/cortex-memory/src/qcli_api.rs`
   - Add circuit_breaker_state() method
   - Add circuit_breaker_failures() method

**Total**: 3 files, 7 changes

---

## Success Criteria

1. ✅ Feedback command records to database (no stub)
2. ✅ Feedback stats visible in `/memory stats`
3. ✅ Circuit breaker status visible in `/memory stats`
4. ✅ Memory IDs displayed in list and search
5. ✅ All tests passing
6. ✅ No compilation errors or warnings

---

## Rollback Plan

If issues arise:
1. Revert changes to `session.rs` (remove FeedbackManager field)
2. Revert changes to `mod.rs` (restore stub)
3. Revert changes to `qcli_api.rs` (remove getters)

All changes are additive and non-breaking.

---

## Post-Implementation

### Documentation Updates
- [ ] Update `memory-user-guide.md` with feedback examples
- [ ] Update `memory-developer-guide.md` with FeedbackManager integration
- [ ] Add feedback workflow to README

### Future Enhancements
- Use feedback data to improve recall ranking
- Add feedback-based memory pruning
- Expose evaluation framework via CLI
- Add configurable circuit breaker thresholds

---

**Status**: Ready to implement  
**Risk**: Low (additive changes only)  
**Impact**: High (completes feedback system)
