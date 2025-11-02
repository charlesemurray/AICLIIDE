# Cortex Memory - Implementation Checklist

## Overview

This checklist provides a step-by-step guide to implement the Cortex memory system in Q CLI.

**Total Estimated Time**: 3-4 weeks
**Phases**: 4 phases with incremental delivery

---

## Pre-Implementation (1 day)

### Design Review
- [x] All design documents reviewed
- [x] UX gaps addressed (error states, mockups, empty states)
- [x] Technical decisions documented
- [x] Team alignment on approach

### Environment Setup
- [ ] Development branch created: `feature/cortex-memory`
- [ ] Dependencies verified: `semantic-search-client`, `rusqlite`
- [ ] Test environment prepared
- [ ] Documentation reviewed

---

## Phase 1: Core Integration (Week 1)

**Goal**: Basic memory storage and recall working in Q CLI

### 1.1 Add Settings (Day 1)

**File**: `crates/chat-cli/src/database/settings.rs`

- [ ] Add 6 memory settings to `Setting` enum
  - `MemoryEnabled`
  - `MemoryRetentionDays`
  - `MemoryMaxSizeMb`
  - `MemoryCrossSession`
  - `MemoryAutoPromote`
  - `MemoryWarnThreshold`
- [ ] Add string mappings in `AsRef<str>` impl
- [ ] Add reverse mappings in `TryFrom<&str>` impl
- [ ] Test: Settings can be read/written
- [ ] Commit: `feat(cortex): add memory settings to Q CLI`

**Reference**: `docs/cortex-memory-config.md` (Step 1)

### 1.2 Create Config Module (Day 1)

**File**: `crates/cortex-memory/src/config.rs` (new)

- [ ] Create `MemoryConfig` struct
- [ ] Implement `from_q_settings()` method
- [ ] Implement `default()` method
- [ ] Add unit tests (3 tests)
- [ ] Commit: `feat(cortex): add memory configuration module`

**Reference**: `docs/cortex-memory-config.md` (Step 4)

### 1.3 Create Embedder Wrapper (Day 2)

**File**: `crates/cortex-memory/src/embedder.rs` (new)

- [ ] Add `semantic-search-client` dependency
- [ ] Create `CortexEmbedder` wrapper
- [ ] Implement `embed()` method
- [ ] Add fallback to BM25 on error
- [ ] Add unit tests (2 tests)
- [ ] Commit: `feat(cortex): add embedder wrapper using Q CLI's CandleTextEmbedder`

**Reference**: `docs/cortex-embedding-research.md`

### 1.4 Create High-Level API (Day 2-3)

**File**: `crates/cortex-memory/src/qcli_api.rs` (new)

- [ ] Create `CortexMemory` struct
- [ ] Implement `new()` with config
- [ ] Implement `store_interaction()`
- [ ] Implement `recall_context()`
- [ ] Add `InteractionMetadata` struct
- [ ] Add `ContextItem` struct
- [ ] Add unit tests (5 tests)
- [ ] Commit: `feat(cortex): add high-level API for Q CLI integration`

**Reference**: `docs/cortex-qcli-integration-design.md` (API Design section)

### 1.5 Integrate with ChatSession (Day 3-4)

**File**: `crates/chat-cli/src/cli/chat/mod.rs`

- [ ] Add `cortex: Option<CortexMemory>` field to `ChatSession`
- [ ] Initialize in `new()` with config
- [ ] Add `recall_context()` before LLM request
- [ ] Add `store_interaction()` after response
- [ ] Handle errors gracefully (continue without memory)
- [ ] Add integration test
- [ ] Commit: `feat(cortex): integrate memory with chat session`

**Reference**: `docs/cortex-qcli-integration-design.md` (Component Integration)

### 1.6 Add Visual Indicators (Day 4)

**File**: `crates/chat-cli/src/cli/chat/mod.rs`

- [ ] Add spinner during recall using Q CLI's `Spinner`
- [ ] Add first-save notification (one-time)
- [ ] Add storage warning at 80% threshold
- [ ] Test visual output
- [ ] Commit: `feat(cortex): add visual indicators for memory operations`

**Reference**: `docs/cortex-visual-indicators.md`

### 1.7 Phase 1 Testing (Day 5)

- [ ] Run all unit tests: `cargo test -p cortex-memory`
- [ ] Run integration tests: `cargo test -p chat-cli`
- [ ] Manual testing: Store and recall in chat
- [ ] Verify embeddings work
- [ ] Verify storage limits
- [ ] Test error handling (database locked, storage full)
- [ ] Document any issues

**Phase 1 Complete** ✅

---

## Phase 2: Memory Commands (Week 2)

**Goal**: User can view and manage memories

### 2.1 Add Slash Commands (Day 1-2)

**File**: `crates/chat-cli/src/cli/chat/cli/mod.rs`

- [ ] Add `Recall` command to `SlashCommand` enum
- [ ] Add `Memory` subcommand to `SlashCommand` enum
- [ ] Export memory module

**File**: `crates/chat-cli/src/cli/chat/cli/memory.rs` (new)

- [ ] Create `MemorySubcommand` enum
  - `Config`, `Set`, `List`, `Search`, `Stats`, `Cleanup`, `Toggle`
- [ ] Create `MemorySetting` enum
  - `Retention`, `MaxSize`, `CrossSession`
- [ ] Implement `execute()` for each subcommand
- [ ] Add command parsing tests
- [ ] Commit: `feat(cortex): add memory slash commands`

**Reference**: `docs/cortex-memory-config.md` (Step 6)

### 2.2 Implement Config Command (Day 2)

- [ ] Implement `/memory config` display
- [ ] Show all settings with current values
- [ ] Show current usage stats
- [ ] Test output formatting
- [ ] Commit: `feat(cortex): implement /memory config command`

**Reference**: `docs/cortex-visual-mockups.md` (Memory Config section)

### 2.3 Implement Set Commands (Day 2)

- [ ] Implement `/memory set retention <days>`
- [ ] Implement `/memory set max-size <mb>`
- [ ] Implement `/memory set cross-session`
- [ ] Add validation for values
- [ ] Add success messages
- [ ] Test each command
- [ ] Commit: `feat(cortex): implement /memory set commands`

**Reference**: `docs/cortex-visual-mockups.md` (Memory Settings Change)

### 2.4 Implement List Command (Day 3)

- [ ] Implement `/memory list`
- [ ] Add `--limit` option
- [ ] Add `--session` filter
- [ ] Format output nicely
- [ ] Handle empty state
- [ ] Test with various data
- [ ] Commit: `feat(cortex): implement /memory list command`

**Reference**: `docs/cortex-visual-mockups.md` (Memory List section)

### 2.5 Implement Search Command (Day 3)

- [ ] Implement `/memory search <query>`
- [ ] Add `--limit` option
- [ ] Show relevance scores
- [ ] Format results
- [ ] Handle no results
- [ ] Test search quality
- [ ] Commit: `feat(cortex): implement /memory search command`

**Reference**: `docs/cortex-visual-mockups.md` (Memory Search section)

### 2.6 Implement Stats Command (Day 4)

- [ ] Implement `/memory stats`
- [ ] Show total memories, storage, sessions
- [ ] Add `--by-session` breakdown
- [ ] Show age distribution
- [ ] Add warnings for old memories
- [ ] Test with various data sizes
- [ ] Commit: `feat(cortex): implement /memory stats command`

**Reference**: `docs/cortex-visual-mockups.md` (Memory Stats section)

### 2.7 Implement Cleanup Command (Day 4)

- [ ] Implement `/memory cleanup`
- [ ] Add confirmation prompt
- [ ] Add `--force` flag
- [ ] Show progress bar
- [ ] Show results (deleted count, freed space)
- [ ] Test cleanup logic
- [ ] Commit: `feat(cortex): implement /memory cleanup command`

**Reference**: `docs/cortex-visual-mockups.md` (Memory Cleanup section)

### 2.8 Implement Toggle Command (Day 5)

- [ ] Implement `/memory toggle`
- [ ] Add `--disable` flag
- [ ] Update settings
- [ ] Show confirmation
- [ ] Test enable/disable flow
- [ ] Commit: `feat(cortex): implement /memory toggle command`

**Reference**: `docs/cortex-visual-mockups.md` (Memory Toggle section)

### 2.9 Phase 2 Testing (Day 5)

- [ ] Test all commands manually
- [ ] Test error states
- [ ] Test empty states
- [ ] Verify help text
- [ ] Test command discovery
- [ ] Document any issues

**Phase 2 Complete** ✅

---

## Phase 3: Advanced Features (Week 3)

**Goal**: Session management, recall enhancements, error handling

### 3.1 Session Integration (Day 1-2)

**File**: `crates/cortex-memory/src/qcli_api.rs`

- [ ] Add session_id to metadata
- [ ] Implement session filtering in recall
- [ ] Add `recall_from_session()` method
- [ ] Add `list_sessions_with_memories()` method
- [ ] Test session isolation
- [ ] Commit: `feat(cortex): add session-aware memory operations`

**Reference**: `docs/cortex-session-integration.md`

### 3.2 Recall Command Enhancements (Day 2-3)

**File**: `crates/chat-cli/src/cli/chat/cli/mod.rs`

- [ ] Implement `/recall <query>`
- [ ] Add `--global` flag
- [ ] Add `--session <id>` option
- [ ] Add `--list-sessions` flag
- [ ] Add `--limit` option
- [ ] Format results nicely
- [ ] Test all options
- [ ] Commit: `feat(cortex): implement /recall command with session support`

**Reference**: `docs/cortex-session-integration.md` (Recall Command section)

### 3.3 Error Handling (Day 3-4)

**Files**: Various

- [ ] Implement database locked error handling
- [ ] Implement storage full error handling
- [ ] Implement embedder failure fallback
- [ ] Implement search timeout
- [ ] Add all error messages from design
- [ ] Test each error scenario
- [ ] Commit: `feat(cortex): implement comprehensive error handling`

**Reference**: `docs/cortex-error-states.md`

### 3.4 Empty States (Day 4)

**Files**: Various

- [ ] Implement first-use empty state
- [ ] Implement no-results empty state
- [ ] Implement empty-session state
- [ ] Implement after-cleanup state
- [ ] Test all empty states
- [ ] Commit: `feat(cortex): implement empty state messages`

**Reference**: `docs/cortex-empty-states.md`

### 3.5 Retention & Cleanup (Day 5)

**File**: `crates/cortex-memory/src/retention.rs` (new)

- [ ] Create `MemoryRetentionManager`
- [ ] Implement `should_cleanup()` logic
- [ ] Implement `cleanup_old_memories()`
- [ ] Implement `should_warn()` logic
- [ ] Add automatic cleanup on startup
- [ ] Test retention logic
- [ ] Commit: `feat(cortex): implement automatic memory retention and cleanup`

**Reference**: `docs/cortex-memory-config.md` (Hybrid Retention Logic)

### 3.6 Phase 3 Testing (Day 5)

- [ ] Test session isolation
- [ ] Test cross-session recall
- [ ] Test all error scenarios
- [ ] Test all empty states
- [ ] Test retention and cleanup
- [ ] End-to-end testing
- [ ] Document any issues

**Phase 3 Complete** ✅

---

## Phase 4: Polish & Launch (Week 4)

**Goal**: Production-ready with documentation and monitoring

### 4.1 Welcome Message (Day 1)

**File**: `crates/chat-cli/src/cli/chat/mod.rs`

- [ ] Add first-run detection
- [ ] Show welcome message with memory info
- [ ] Add to `/help` output
- [ ] Test first-run experience
- [ ] Commit: `feat(cortex): add welcome message and help text`

**Reference**: `docs/cortex-privacy-design.md` (Welcome Message)

### 4.2 Verbose Mode (Day 1)

**File**: `crates/chat-cli/src/database/settings.rs`

- [ ] Add `MemoryVerbose` setting
- [ ] Implement verbose recall output
- [ ] Implement verbose store output
- [ ] Add `/memory set verbose` command
- [ ] Test verbose mode
- [ ] Commit: `feat(cortex): add verbose mode for memory operations`

**Reference**: `docs/cortex-visual-indicators.md` (Phase 2: Verbose Mode)

### 4.3 Ephemeral Sessions (Day 2)

**File**: `crates/chat-cli/src/cli/chat/mod.rs`

- [ ] Add `--no-memory` flag to chat command
- [ ] Add `--ephemeral` alias
- [ ] Skip memory initialization when flag set
- [ ] Show indicator in prompt
- [ ] Test ephemeral mode
- [ ] Commit: `feat(cortex): add ephemeral session support`

**Reference**: `docs/cortex-privacy-design.md` (Ephemeral Sessions)

### 4.4 Documentation (Day 2-3)

- [ ] Update README with memory features
- [ ] Create user guide: `docs/memory-user-guide.md`
- [ ] Create developer guide: `docs/memory-developer-guide.md`
- [ ] Update `/help` text
- [ ] Add examples to documentation
- [ ] Commit: `docs(cortex): add user and developer documentation`

### 4.5 Performance Testing (Day 3)

- [ ] Benchmark embedding generation
- [ ] Benchmark recall latency
- [ ] Test with large datasets (10k+ memories)
- [ ] Verify < 100ms recall target
- [ ] Optimize if needed
- [ ] Document performance results
- [ ] Commit: `perf(cortex): performance testing and optimization`

### 4.6 Integration Testing (Day 4)

- [ ] Test with real Q CLI workflows
- [ ] Test multi-session scenarios
- [ ] Test long-running sessions
- [ ] Test storage limits
- [ ] Test error recovery
- [ ] Test on all platforms (macOS, Linux, Windows)
- [ ] Document any platform-specific issues

### 4.7 Final Polish (Day 4-5)

- [ ] Fix any remaining bugs
- [ ] Improve error messages
- [ ] Optimize performance
- [ ] Clean up code
- [ ] Run full test suite
- [ ] Update documentation
- [ ] Commit: `chore(cortex): final polish and cleanup`

### 4.8 Launch Preparation (Day 5)

- [ ] Create release notes
- [ ] Update changelog
- [ ] Prepare announcement
- [ ] Create demo video/screenshots
- [ ] Review all documentation
- [ ] Final testing
- [ ] Merge to main

**Phase 4 Complete** ✅

---

## Post-Launch (Ongoing)

### Week 1 After Launch
- [ ] Monitor error rates
- [ ] Gather user feedback
- [ ] Track usage metrics
- [ ] Fix critical bugs
- [ ] Update documentation based on feedback

### Week 2-4 After Launch
- [ ] Analyze usage patterns
- [ ] Identify improvement opportunities
- [ ] Plan Phase 2 features
- [ ] Conduct user interviews
- [ ] Iterate based on data

---

## Testing Checklist

### Unit Tests
- [ ] All cortex-memory modules (39 tests)
- [ ] Integration tests (6 tests)
- [ ] Settings integration tests
- [ ] Command parsing tests
- [ ] Error handling tests

### Integration Tests
- [ ] Store and recall flow
- [ ] Session isolation
- [ ] Cross-session recall
- [ ] Cleanup and retention
- [ ] Error recovery

### Manual Tests
- [ ] First-run experience
- [ ] All slash commands
- [ ] All error scenarios
- [ ] All empty states
- [ ] Storage limits
- [ ] Performance with large datasets

### Platform Tests
- [ ] macOS (x86_64, ARM64)
- [ ] Linux (x86_64, ARM64)
- [ ] Windows (x86_64)

---

## Success Criteria

### Phase 1
- ✅ Memory stores automatically
- ✅ Recall works in chat
- ✅ < 100ms recall latency
- ✅ All unit tests pass

### Phase 2
- ✅ All commands implemented
- ✅ Help text complete
- ✅ Error messages clear
- ✅ Empty states helpful

### Phase 3
- ✅ Session isolation works
- ✅ Error handling robust
- ✅ Retention logic correct
- ✅ All edge cases handled

### Phase 4
- ✅ Documentation complete
- ✅ Performance targets met
- ✅ All platforms tested
- ✅ Ready for production

---

## Risk Mitigation

### Technical Risks
- **Embedder fails**: Fallback to BM25 keyword search
- **Database locked**: Retry once, continue without memory
- **Storage full**: Warn user, continue in read-only mode
- **Performance issues**: Optimize or reduce default limits

### UX Risks
- **Users don't discover feature**: Add welcome message, help text
- **Users concerned about privacy**: Clear disclosure, easy opt-out
- **Commands not intuitive**: User testing, iterate on naming
- **Too much visual noise**: Start minimal, add verbose mode

### Rollback Plan
- Feature flag: `memory.enabled = false`
- Can disable via settings
- Data preserved if disabled
- No breaking changes to Q CLI

---

## Resources

### Design Documents
1. `cortex-qcli-integration-design.md` - Main integration design
2. `cortex-implementation-plan-detailed.md` - Detailed step-by-step plan
3. `cortex-memory-config.md` - Configuration system
4. `cortex-session-integration.md` - Session management
5. `cortex-error-states.md` - Error handling
6. `cortex-visual-mockups.md` - UI examples
7. `cortex-empty-states.md` - Empty state design
8. `cortex-privacy-design.md` - Privacy and transparency
9. `cortex-visual-indicators.md` - Visual feedback
10. `cortex-embedding-research.md` - Embedding solution
11. `cortex-verification-results.md` - Test results
12. `cortex-design-review.md` - Design evaluation

### Code References
- `crates/cortex-memory/` - Core implementation
- `crates/semantic-search-client/` - Embedder
- `crates/chat-cli/src/database/settings.rs` - Settings system
- `crates/chat-cli/src/session/` - Session management
- `crates/chat-cli/src/util/spinner.rs` - UI spinner

---

## Timeline Summary

| Phase | Duration | Deliverable |
|-------|----------|-------------|
| Pre-Implementation | 1 day | Setup complete |
| Phase 1 | 1 week | Core integration working |
| Phase 2 | 1 week | All commands implemented |
| Phase 3 | 1 week | Advanced features complete |
| Phase 4 | 1 week | Production-ready |
| **Total** | **4 weeks** | **Launch** |

---

## Status Tracking

**Current Status**: ✅ Design Complete, Ready for Implementation

**Next Steps**:
1. Create feature branch
2. Start Phase 1, Day 1: Add settings
3. Follow checklist step-by-step
4. Test continuously
5. Commit after each step

**Ready to begin implementation!** 🚀
