# Task 2.1: Refactor God Object - IN PROGRESS ⏳

**Date**: 2025-11-03  
**Sprint**: 2 (Refactoring)  
**Status**: ⏳ Partial - Components created, integration incomplete  
**Time**: ~20 minutes

---

## Objective

Refactor `MultiSessionCoordinator` to split responsibilities into focused components.

---

## Problem

The coordinator has 10 fields and too many responsibilities:
- Session storage
- Persistence
- Lock management
- Rate limiting
- Memory monitoring
- Resource cleanup
- State change handling
- Configuration

---

## Solution Approach

Split into focused components:
1. `SessionRegistry` - Session storage and lifecycle
2. `SessionResources` - Resource management (rate limiting, memory, cleanup)
3. `MultiSessionCoordinator` - Coordination and orchestration

---

## Progress

### ✅ Components Created

**SessionRegistry** (`session_registry.rs`):
- Manages session storage (HashMap)
- Handles persistence
- Manages locks
- Cleanup of inactive sessions

**SessionResources** (`session_resources.rs`):
- Rate limiting
- Memory monitoring
- Resource cleanup
- Dropped events tracking

### ⏳ Integration Incomplete

**Challenges**:
- 50+ method calls need updating to delegate to components
- `self.state` references throughout codebase
- External code accesses `coordinator.state` directly
- Backward compatibility needed

---

## Recommendation

**Defer Task 2.1** for now because:

1. **Sprint 1 achieved production readiness** - System is already Grade A
2. **High refactoring cost** - 50+ methods need updates
3. **Breaking changes** - External code needs updates
4. **Low immediate value** - Code works well as-is
5. **Better done incrementally** - Can refactor one subsystem at a time

---

## Alternative Approach

Instead of full refactoring, consider:

### Option 1: Incremental Refactoring
- Keep current structure
- Extract one component at a time
- Maintain backward compatibility
- Do over multiple PRs

### Option 2: Defer Until Pain Point
- Current code is maintainable
- Refactor when adding new features
- Let real use cases drive design

### Option 3: Document Current Structure
- Add clear documentation
- Group related methods
- Use comments to show logical sections

---

## Files Created

- `crates/chat-cli/src/cli/chat/session_registry.rs` - Session storage component
- `crates/chat-cli/src/cli/chat/session_resources.rs` - Resource management component

**Note**: These files are created but not integrated. Can be deleted or completed later.

---

## Decision Point

**Recommendation**: Skip Task 2.1 and move to Task 2.2 (Fix Parameter Explosion)

**Rationale**:
- Task 2.2 has higher immediate value
- Parameter explosion is more painful in daily use
- Easier to implement (just create config structs)
- Less risk of breaking changes

---

## Next Steps

Would you like to:
1. **Skip Task 2.1** and move to Task 2.2 (Fix Parameter Explosion)?
2. **Complete Task 2.1** (will take significant time)?
3. **Delete partial work** and document decision?
4. **Review Sprint 2 priorities**?
