# Phase Gate Worktree State Fix Flow

**Type**: Bug Fix & State Management Flow  
**Module**: `ecc-infra` (hooks layer)  
**Layer**: Pre-tool hooks (phase-gate enforcement)  
**Session**: session-1775719302-18786 (timestamp: 1775719305)  
**Status**: Implementation complete

## Overview

This flow documents the fix for BL-131: Phase-gate hook reading wrong state.json in worktrees. The bug prevented the phase-gate hook from correctly reading workflow state, blocking the /implement command during active development.

## Problem

The `phase-gate-hook` (pre:write-edit:phase-gate) was reading `.claude/workflow/state.json` instead of the correct per-worktree state file at `<git-dir>/ecc-workflow/state.json`. This caused:

- Blocking /implement with stale state from previous sessions
- Users unable to transition through spec → design → implement phases in new worktrees
- Confusion about workflow state enforcement

## Root Cause

The hook's state resolution logic didn't account for worktree-isolated state files. It always fell back to `.claude/workflow/` instead of checking the per-worktree location at `<git-dir>/ecc-workflow/`.

## Solution

Updated `ecc-infra` to:

1. **resolve_state_dir()** function now:
   - Detects if running in a git worktree via `git rev-parse --git-dir`
   - Constructs correct state path: `<git-dir>/ecc-workflow/state.json`
   - Falls back to `.claude/workflow/` only for non-git directories
   - Validates state file exists before reading

2. **phase-gate-hook** now:
   - Calls updated `resolve_state_dir()` 
   - Reads correct worktree-scoped state
   - Correctly enforces phase gates during /implement

## Files Changed

- `crates/ecc-infra/Cargo.toml` - Dependency updates for git-dir detection
- `crates/ecc-infra/src/lib.rs` - Updated `resolve_state_dir()` implementation
- `docs/specs/2026-04-07-phase-gate-worktree-state-fix/` - Spec, design, and task documentation

## Impact

- Worktree-isolated development now works correctly
- Phase gates properly respect worktree state
- Users can transition through /spec → /design → /implement in worktrees without blocking

## Related Flows

- **Cartography Delta Processing**: Session start/stop hooks write deltas
- **Worktree Lifecycle**: Session merge runs worktree cleanup
- **Phase Gate System**: Pre-tool hook enforces workflow phases
