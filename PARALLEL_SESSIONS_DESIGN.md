# Parallel Q CLI Sessions - Design Document

## Problem Statement

Users want to run multiple interactive Q CLI sessions simultaneously on the same codebase to maximize productivity. Instead of waiting for one session to complete work, they want to context-switch between multiple active sessions, answering questions and providing guidance as needed.

## Use Case

```
Developer working on a large codebase wants to:
1. Implement feature A (Session 1)
2. Refactor module B (Session 2)  
3. Fix bug C (Session 3)

All simultaneously, switching between sessions as each needs input or approval.
Eventually, all changes need to merge back to the main branch.
```

## Problems to Solve

### P1: File Conflict Prevention
**Problem**: Multiple sessions cannot modify the same files simultaneously without conflicts.

**Constraints**:
- Git (and most VCS) don't allow multiple working copies in same directory
- Uncommitted changes block branch switching
- File system doesn't prevent concurrent writes

**Questions**:
- How do we physically isolate working directories?
- How do we ensure changes can merge back together?
- What happens when sessions modify the same file?

---

### P2: Conversation Identity & Isolation
**Problem**: Each session needs its own conversation history, but they're all related to the same project.

**Constraints**:
- Current Q CLI uses absolute path as conversation key
- Multiple directories = multiple conversations (already works)
- No concept of "related" conversations

**Questions**:
- How do we identify which conversations belong together?
- How do we prevent conversation cross-contamination?
- How do we maintain conversation history when directories change?

---

### P3: Cross-Session Context Awareness
**Problem**: Sessions working on the same codebase should be aware of each other's changes and decisions.

**Constraints**:
- Sessions are independent processes
- No shared memory or state
- Changes are in different directories/branches

**Questions**:
- Should sessions know about each other?
- How do they discover related sessions?
- What information should be shared vs isolated?
- When should sharing happen (real-time vs on-demand)?

---

### P4: Merge Coordination
**Problem**: Multiple branches with changes need to eventually merge to main without conflicts.

**Constraints**:
- Git handles merging, not Q CLI
- Conflicts are inevitable with parallel work
- Order of merges may matter
- Some changes may depend on others

**Questions**:
- Should Q CLI help orchestrate merges?
- How do we detect potential conflicts early?
- Should sessions be aware of merge order?
- Who decides when to merge (user or Q)?

---

### P5: Session Discovery & Management
**Problem**: User needs to know which sessions exist, their status, and which need attention.

**Constraints**:
- Sessions are separate processes in different terminals
- No central registry of active sessions
- User may have many terminal windows open

**Questions**:
- How does user discover active sessions?
- How do they know which session needs attention?
- Should there be a "session manager" or is that external (tmux)?
- How do we handle session lifecycle (create, pause, resume, destroy)?

---

### P6: State Synchronization
**Problem**: When one session makes changes (especially merges), other sessions may need to update.

**Constraints**:
- Sessions are independent
- Git state can change outside Q CLI
- Stale branches can cause conflicts

**Questions**:
- Should sessions auto-sync with main branch?
- How do we detect when sync is needed?
- What happens to conversation history after sync?
- Should Q CLI handle rebasing/merging or just notify?

---

### P7: Resource Management
**Problem**: Multiple sessions consume API quota, system resources, and may hit rate limits.

**Constraints**:
- API has rate limits and quotas
- System has finite CPU/memory
- User has finite attention

**Questions**:
- Should there be a limit on concurrent sessions?
- How do we prioritize resource allocation?
- Should sessions coordinate API usage?
- How do we prevent resource exhaustion?

---

### P8: User Mental Model
**Problem**: User needs to understand the system - which session is which, what state they're in, how they relate.

**Constraints**:
- Multiple terminal windows are hard to track
- Context switching is cognitively expensive
- Easy to lose track of what each session is doing

**Questions**:
- How do we make sessions easily identifiable?
- What visual/naming conventions help?
- How do we reduce cognitive load?
- Should there be a unified view of all sessions?

---

### P9: Failure & Recovery
**Problem**: Sessions may crash, hang, or need to be paused/resumed.

**Constraints**:
- Processes can crash
- Network can fail
- User may need to stop/restart
- Work should not be lost

**Questions**:
- How do we persist session state?
- Can sessions be resumed after crash?
- What happens to orphaned sessions?
- How do we clean up abandoned work?

---

### P10: Integration with Existing Features
**Problem**: This needs to work with existing Q CLI features (checkpoints, agents, MCP, etc.)

**Constraints**:
- Checkpoints use git
- Agents run in background
- MCP servers are per-session
- Must maintain backward compatibility

**Questions**:
- How do checkpoints work across sessions?
- How do agents relate to sessions?
- Are MCP servers shared or per-session?
- Can we do this without breaking existing workflows?

---

## Non-Problems (Out of Scope)

- **Terminal multiplexing**: Use tmux/screen/etc (external tool)
- **Git operations**: Git handles commits, branches, merges
- **Code review**: External to Q CLI
- **CI/CD integration**: Separate concern

---

## Success Criteria

A successful design should:
1. Allow multiple interactive Q sessions on same codebase
2. Prevent file conflicts between sessions
3. Enable eventual merge of all changes
4. Minimize cognitive load on user
5. Work with or without git
6. Not break existing Q CLI functionality
7. Be implementable incrementally

---

## Next Steps

1. Prioritize problems (which are critical vs nice-to-have)
2. Generate design alternatives for each problem
3. Evaluate alternatives against constraints
4. Select best approach for each problem
5. Ensure solutions work together coherently
6. Create implementation plan

---

## Open Questions for Discussion

- Is this the right set of problems?
- Are there other problems we're missing?
- Which problems are most critical to solve?
- Which problems can be deferred or handled externally?
- What are the hard constraints we cannot violate?
