# ADR-0056: Deprecation of ECC_WORKFLOW_BYPASS

## Status
Accepted (2026-04-06)

## Context
`ECC_WORKFLOW_BYPASS=1` exists in three code paths. The new auditable bypass system (ADR-0055) replaces it.

## Decision
Deprecate `ECC_WORKFLOW_BYPASS=1`:
- Handler-level checks removed from worktree_guard.rs and session_merge.rs
- Deprecation warning in dispatch() — hooks still pass (backward compat)
- Timeline: deprecated v4.3, removal planned v5.0

## Consequences
- Single bypass entry point — auditable, consistent
- Backward compatible during transition
- .envrc users see deprecation warnings until migrated
