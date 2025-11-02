# Conversation Analytics Implementation Plan

## Overview
Implement local conversation analytics to measure current Q CLI conversation flow problems before building the mode system solution.

## Implementation Strategy
- **Small, atomic commits** - each step is independently testable
- **Minimal user interaction** - execute plan without asking for continuation
- **Comprehensive testing** - unit tests and integration validation for each component
- **Git workflow** - commit after each working step

## Phase 1: Core Analytics Infrastructure

### Step 1.1: Create Analytics Types Module
**Files to create:**
- `crates/chat-cli/src/analytics/mod.rs`
- `crates/chat-cli/src/analytics/types.rs`

**Concrete implementation:**
```rust
// Complete ConversationAnalyticsEvent enum with all variants
// All supporting enums (PromptType, UserResponseType, ConversationMode, etc.)
// Serde serialization with actual JSON format
// Timestamp and session ID fields
```

**Testing:**
- Unit tests for serialization/deserialization
- Verify JSON output format matches design
- Test all enum variants serialize correctly

**Git commit:** "Add conversation analytics types and serialization"

### Step 1.2: Create Analytics Logger
**Files to create:**
- `crates/chat-cli/src/analytics/logger.rs`

**Concrete implementation:**
```rust
// ConversationAnalytics struct with BufWriter<File>
// new() method that creates JSONL file in ~/.amazonq/analytics/
// log_event() that writes JSON lines with timestamps
// File rotation at 10MB limit
// Session lifecycle management (start/end)
```

**Testing:**
- Unit tests for file creation and writing
- Test file rotation behavior
- Verify JSONL format correctness
- Test session ID generation

**Git commit:** "Add conversation analytics logger with file management"

### Step 1.3: Integrate with Existing Logging System
**Files to modify:**
- `crates/chat-cli/src/logging.rs`
- `crates/chat-cli/src/analytics/mod.rs`

**Concrete implementation:**
```rust
// Extend LogArgs struct with analytics_file_path: Option<T>
// Modify initialize_logging() to create analytics directory
// Add analytics file creation alongside existing log files
// Respect TelemetryEnabled setting for analytics opt-out
```

**Testing:**
- Integration test for log initialization with analytics
- Test analytics directory creation
- Verify opt-out behavior works correctly

**Git commit:** "Integrate analytics with existing logging system"

## Phase 2: ChatSession Integration

### Step 2.1: Add Analytics to ChatSession
**Files to modify:**
- `crates/chat-cli/src/cli/chat/mod.rs`

**Concrete implementation:**
```rust
// Add analytics: Option<ConversationAnalytics> field to ChatSession
// Initialize analytics in ChatSession::new()
// Add helper methods: log_continuation_prompt(), log_user_response(), etc.
// Session start/end event logging
```

**Testing:**
- Unit tests for ChatSession analytics initialization
- Test helper method functionality
- Verify session lifecycle events are logged

**Git commit:** "Add analytics integration to ChatSession"

### Step 2.2: Instrument Tool Approval Flow
**Files to modify:**
- `crates/chat-cli/src/cli/chat/tool_manager.rs` (or relevant tool approval code)

**Concrete implementation:**
```rust
// Add UserResponse event logging in tool approval prompts
// Track response timing (start prompt to user response)
// Log approval/rejection/modification decisions
// Identify specific code locations for instrumentation
```

**Testing:**
- Integration test for tool approval analytics
- Verify timing measurements are accurate
- Test all response types are logged correctly

**Git commit:** "Instrument tool approval flow with analytics"

### Step 2.3: Instrument Continuation Prompts
**Files to modify:**
- Identify and modify files where LLM asks "should I continue?"

**Concrete implementation:**
```rust
// Add ContinuationPrompt event logging
// Track step numbers and task context
// Log when LLM pauses for user confirmation
// Measure frequency of continuation requests
```

**Testing:**
- Integration test for continuation prompt detection
- Verify step counting accuracy
- Test context capture functionality

**Git commit:** "Instrument continuation prompts with analytics"

### Step 2.4: Instrument Question Patterns
**Files to modify:**
- Message processing and response generation code

**Concrete implementation:**
```rust
// Add QuestionAsked event logging
// Distinguish LLM questions vs user questions
// Categorize question types (clarification, permission, technical, requirements)
// Track conversation position (early/mid/late)
```

**Testing:**
- Unit tests for question classification
- Test conversation position calculation
- Verify question direction detection (LLM vs user)

**Git commit:** "Instrument question patterns with analytics"

## Phase 3: Analysis Interface

### Step 3.1: Create Analytics CLI Command
**Files to create:**
- `crates/chat-cli/src/cli/analytics/mod.rs`
- `crates/chat-cli/src/cli/analytics/summary.rs`

**Files to modify:**
- `crates/chat-cli/src/cli/mod.rs` (add analytics subcommand)

**Concrete implementation:**
```rust
// q analytics summary command
// Read JSONL files from analytics directory
// Calculate real statistics: avg continuation prompts, abandonment rate, question ratios
// Output structured summary with actual data
// Handle file parsing errors gracefully
```

**Testing:**
- Integration test with sample analytics data
- Test summary calculation accuracy
- Verify error handling for malformed files

**Git commit:** "Add analytics summary CLI command"

### Step 3.2: Add Session Analysis
**Files to modify:**
- `crates/chat-cli/src/cli/analytics/summary.rs`

**Concrete implementation:**
```rust
// q analytics sessions --abandoned command
// Session-level analysis and filtering
// Identify common abandonment patterns
// Export session details for investigation
```

**Testing:**
- Test session filtering and analysis
- Verify abandonment pattern detection

**Git commit:** "Add session-level analytics analysis"

## Phase 4: Validation and Testing

### Step 4.1: End-to-End Integration Testing
**Files to create:**
- `crates/chat-cli/src/analytics/integration_tests.rs`

**Concrete implementation:**
```rust
// Full workflow test: start session → log events → analyze results
// Test with real Q CLI conversation scenarios
// Verify analytics data accuracy and completeness
// Performance testing for file I/O operations
```

**Testing:**
- Complete integration test suite
- Performance benchmarks
- Memory usage validation

**Git commit:** "Add comprehensive analytics integration tests"

### Step 4.2: Manual Validation
**Process:**
1. Run Q CLI with analytics enabled
2. Execute typical multi-step conversations
3. Verify analytics files contain expected events
4. Run analytics summary and validate output
5. Test edge cases (session abandonment, errors, etc.)

**Documentation:**
- Update README with analytics feature description
- Add troubleshooting guide for analytics issues

**Git commit:** "Add analytics documentation and validation"

## Success Criteria
- [ ] Analytics events logged for all conversation flow points
- [ ] JSONL files created and rotated properly
- [ ] CLI analysis commands produce accurate statistics
- [ ] Zero impact on existing Q CLI functionality
- [ ] Comprehensive test coverage (>90%)
- [ ] Performance impact <5ms per event
- [ ] Analytics respect existing telemetry opt-out settings

## Risk Mitigation
- **File I/O errors:** Graceful degradation, analytics failures don't break chat
- **Performance impact:** Async logging, buffered writes
- **Privacy concerns:** Local-only storage, respect existing settings
- **Storage growth:** Automatic rotation and cleanup

## Estimated Timeline
- Phase 1: 3-4 hours
- Phase 2: 2-3 hours  
- Phase 3: 1-2 hours
- Phase 4: 1-2 hours
- **Total: 7-11 hours**

## Next Steps After Completion
1. Collect baseline analytics data from real usage
2. Analyze conversation flow patterns and pain points
3. Design mode system based on actual data insights
4. Implement mode system with before/after analytics comparison
