# 0012. Wave-Based Parallel TDD Execution

Date: 2026-03-22

## Status
Accepted

## Context
The /implement command dispatched PCs strictly sequentially. For implementations with many independent PCs (touching different files), this wasted time. A wave-based model was needed to execute independent PCs concurrently while preserving correctness for dependent PCs.

## Decision
Group PCs into waves using left-to-right scan with adjacent + no file overlap. Dispatch up to 4 concurrent tdd-executor subagents per wave using Claude Code's `isolation: "worktree"`. Merge worktree branches in PC-ID order after each wave. Run regression verification at wave boundaries. Create git tags at wave boundaries for rollback. Single-PC waves preserve current sequential behavior (backward compatible).

## Consequences
- **Positive**: Implementation time reduced proportionally to parallelism factor for independent PCs
- **Positive**: Backward compatible — fully sequential implementations behave identically
- **Positive**: Git tags enable clean rollback to pre-wave state
- **Negative**: Wave grouping adds complexity to implement.md's orchestration logic
- **Negative**: Merge conflicts between wave PCs (if dependency analysis misses a shared file) require manual resolution
