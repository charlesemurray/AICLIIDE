# Cortex Memory - Visual Indicators Design

## Q CLI UI Capabilities

**Available libraries**:
- ✅ `indicatif` - Progress bars and spinners
- ✅ Custom `Spinner` - Q CLI's spinner implementation with frames
- ✅ `ratatui` - Full TUI framework (for advanced UI)
- ✅ `crossterm` - Terminal manipulation
- ✅ `StyledText` - Q CLI's text styling

---

## Decision: Minimal Indicators with Q CLI's Spinner

**Approach**: Use Q CLI's existing `Spinner` for recall, silent for store

---

## Visual Indicator Options

### Option A: Silent (No Indicators)

```bash
You: How do I deploy to Lambda?
Q: Based on our previous discussion about Lambda...
```

**Pros**: Cleanest UI, no distraction
**Cons**: No feedback, users don't know memory is working

### Option B: Minimal (Q CLI Spinner) ✅ RECOMMENDED

```bash
You: How do I deploy to Lambda?
▰▰▰▱▱▱▱ Recalling context...
Q: Based on our previous discussion about Lambda...
```

**Implementation**:
```rust
use crate::util::spinner::{Spinner, SpinnerComponent};

// During recall
let mut spinner = Spinner::new(vec![
    SpinnerComponent::Spinner,
    SpinnerComponent::Text(" Recalling context...".into()),
]);

// Perform search
let results = cortex.recall_context(query, 5).await?;

// Stop spinner
spinner.stop();
```

**Pros**: 
- Uses Q CLI's existing spinner
- Consistent with other Q CLI operations
- Brief, non-intrusive
- Clear feedback

**Cons**: None significant

### Option C: Detailed (Show Sources)

```bash
You: How do I deploy to Lambda?
▰▰▰▱▱▱▱ Searching memories...

Found relevant context:
  * "Lambda deployment" (2 days ago, 95% match)
  * "AWS credentials" (1 week ago, 82% match)

Q: Based on our previous discussion about Lambda...
```

**Pros**: More informative, helps debugging
**Cons**: Verbose, slows down interaction

### Option D: Configurable Verbosity

```bash
# Default: minimal
You: How do I deploy to Lambda?
▰▰▰▱▱▱▱ Recalling context...
Q: ...

# Verbose mode (q settings set memory.verbose true)
You: How do I deploy to Lambda?
▰▰▰▱▱▱▱ Searching 1,247 memories...

Found 2 relevant matches:
  * session-abc123: "Lambda deployment" (95%)
  * session-xyz789: "AWS credentials" (82%)

Q: ...

[Stored to memory: mem-12345]
```

**Pros**: Flexibility, power users get details
**Cons**: More complex implementation

---

## Recommended Implementation

### Phase 1: Minimal Spinner

**During recall**:
```rust
// In ChatSession::process_message()
if let Some(cortex) = &self.cortex {
    let mut spinner = Spinner::new(vec![
        SpinnerComponent::Spinner,
        SpinnerComponent::Text(" Recalling context...".into()),
    ]);
    
    let context = cortex.recall_context(user_input, 5).await?;
    
    spinner.stop();
    
    // Use context in prompt...
}
```

**During store** (silent):
```rust
// After response, store silently
if let Some(cortex) = &mut self.cortex {
    cortex.store_interaction(user_input, assistant_response, metadata).await?;
    // No visual indicator
}
```

**First-time notification** (one-time):
```rust
// On first memory store
if is_first_memory_store {
    queue!(
        self.stderr,
        StyledText::dim(),
        style::Print("\n[Memory saved - Q will remember this conversation]\n"),
        style::Print("  Disable: /memory toggle --disable | Configure: /memory config\n\n"),
        StyledText::reset(),
    )?;
}
```

### Phase 2: Add Verbose Mode

**Add setting**:
```rust
#[strum(message = "Show detailed memory activity (boolean)")]
MemoryVerbose,
```

**Verbose recall**:
```rust
if config.verbose {
    let mut spinner = Spinner::new(vec![
        SpinnerComponent::Spinner,
        SpinnerComponent::Text(format!(" Searching {} memories...", total_count)),
    ]);
    
    let context = cortex.recall_context(user_input, 5).await?;
    
    spinner.stop_with_message(format!(
        "Found {} relevant matches",
        context.len()
    ));
    
    // Show matches
    for (i, item) in context.iter().enumerate() {
        println!("  * {}: \"{}\" ({}%)", 
            i + 1, 
            truncate(&item.content, 50),
            (item.relevance * 100.0) as u8
        );
    }
    println!();
}
```

**Verbose store**:
```rust
if config.verbose {
    queue!(
        self.stderr,
        StyledText::dim(),
        style::Print(format!("\n[Stored to memory: {}]\n", memory_id)),
        StyledText::reset(),
    )?;
}
```

---

## Warning Indicators

**Storage threshold warning**:
```bash
You: How do I deploy to Lambda?
▰▰▰▱▱▱▱ Recalling context...

⚠ Memory storage at 85 MB / 100 MB (85%)
  Run '/memory cleanup' or adjust limits with '/memory config'

Q: Here's how to deploy to Lambda...
```

**Implementation**:
```rust
// Check before recall
if cortex.should_warn()? {
    queue!(
        self.stderr,
        StyledText::warning_fg(),
        style::Print(format!(
            "\n⚠ Memory storage at {} MB / {} MB ({}%)\n",
            current_mb, max_mb, percentage
        )),
        style::Print("  Run '/memory cleanup' or adjust limits with '/memory config'\n\n"),
        StyledText::reset(),
    )?;
}
```

---

## Alternative: Progress Bar (Future)

**Using indicatif for large operations**:
```rust
use indicatif::{ProgressBar, ProgressStyle};

// For cleanup operations
let pb = ProgressBar::new(total_memories as u64);
pb.set_style(
    ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
        .unwrap()
);

for memory in memories_to_delete {
    cortex.delete(&memory.id)?;
    pb.inc(1);
}

pb.finish_with_message("Cleanup complete");
```

---

## Styling Guidelines

**Use Q CLI's existing styles**:

```rust
// Dim text for non-critical info
StyledText::dim()

// Warning for alerts
StyledText::warning_fg()

// Error for problems
StyledText::error_fg()

// Success for confirmations
StyledText::success_fg()

// Reset after styled text
StyledText::reset()
```

**Examples**:
```rust
// Success message
queue!(
    self.stderr,
    StyledText::success_fg(),
    style::Print("✓ Memory enabled\n"),
    StyledText::reset(),
)?;

// Warning
queue!(
    self.stderr,
    StyledText::warning_fg(),
    style::Print("⚠ Storage limit approaching\n"),
    StyledText::reset(),
)?;

// Dim info
queue!(
    self.stderr,
    StyledText::dim(),
    style::Print("[Memory saved]\n"),
    StyledText::reset(),
)?;
```

---

## Decision Summary

**✅ Phase 1: Minimal**
- Use Q CLI's `Spinner` during recall
- Silent during store
- One-time first-save notification
- Warning at storage threshold

**Phase 2: Verbose Mode**
- Add `memory.verbose` setting
- Show detailed match info
- Show store confirmations
- Configurable via `/memory set verbose`

**Implementation**:
```rust
// Minimal (default)
▰▰▰▱▱▱▱ Recalling context...

// Verbose (opt-in)
▰▰▰▱▱▱▱ Searching 1,247 memories...
Found 2 matches:
  * "Lambda deployment" (95%)
  * "AWS credentials" (82%)
[Stored to memory: mem-12345]
```

**Benefits**:
- ✅ Consistent with Q CLI's existing UI
- ✅ Uses proven spinner implementation
- ✅ Non-intrusive by default
- ✅ Configurable for power users
- ✅ Clear feedback without clutter
