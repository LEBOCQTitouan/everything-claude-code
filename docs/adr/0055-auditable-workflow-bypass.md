# ADR-0055: Auditable Workflow Bypass

## Status
Accepted (2026-04-06)

## Context
ECC uses `ECC_WORKFLOW_BYPASS=1` as a binary kill-switch that disables all workflow hooks with zero audit trail. No granularity, no audit, no feedback loop.

## Decision
Introduce granular, session-scoped bypass with consent and audit:
- Bypass tokens as JSON files at `~/.ecc/bypass-tokens/<session_id>/<encoded_hook_id>.json`
- BypassStore port trait with SQLite adapter at `~/.ecc/bypass.db` (WAL, busy_timeout)
- Single bypass entry point in `dispatch()` — handler-level checks removed
- CLI: `ecc bypass grant|list|summary|prune|gc`
- Verdict enum: Accepted, Refused, Applied

## Consequences
- Granular per-hook bypass with full audit trail
- Session-scoped tokens auto-expire
- Multi-step LLM protocol (ask user → grant → retry)
- 31 HookPorts sites updated
