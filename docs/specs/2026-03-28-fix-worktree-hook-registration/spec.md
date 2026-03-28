# Spec: BL-085 — Fix WorktreeCreate/WorktreeRemove Hook Registration

## Problem Statement

The `EnterWorktree` tool fails with `"WorktreeCreate hook failed: no successful output"` in any project with ECC hooks installed. The root cause is a semantic mismatch: `WorktreeCreate` is a **delegation hook** (Claude Code expects it to create the worktree), but the registered handler (`worktree_create_init` at `crates/ecc-app/src/hook/handlers/tier3_session/worktree.rs:12`) only logs to the session file and passthroughs stdin. The `WorktreeRemove` hook has the same issue — `worktree_cleanup_reminder` at `crates/ecc-app/src/hook/handlers/tier1_simple/dev_hooks.rs:272` only emits a warning. This blocks all worktree-based isolation, including BL-065 Sub-Spec C.

## Research Summary

- Claude Code's `WorktreeCreate`/`WorktreeRemove` hooks are delegation hooks — when registered, Claude Code skips native `git worktree` operations and expects the hook to handle creation/removal
- The `ecc install` merge logic (`crates/ecc-domain/src/config/merge.rs:65-138`) already supports legacy hook detection via `is_legacy_ecc_hook()` with 5 pattern categories
- PostToolUse hooks receive `{"tool_name":"EnterWorktree","tool_input":{...}}` JSON on stdin — different from WorktreeCreate's `{"worktree_path":"..."}` format
- No prior audit findings flagged this issue

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Remove `WorktreeCreate`/`WorktreeRemove` from hooks.json | They are delegation hooks but only log — blocking EnterWorktree | No |
| 2 | Add `PostToolUse` hooks matching `EnterWorktree`/`ExitWorktree` | Preserves session logging without claiming worktree creation ownership | No |
| 3 | Add legacy detection for old worktree hooks | Auto-cleans stale entries from users' settings.json on next `ecc install` | No |
| 4 | Repurpose handler code for PostToolUse, don't delete | Handlers still useful for logging; only the event registration and stdin format changes | No |
| 5 | Keep `WorktreeCreate`/`WorktreeRemove` in `VALID_HOOK_EVENTS` | They are legitimate Claude Code events; ECC just shouldn't register delegation hooks for them | No |
| 6 | New hook IDs: `post:enter-worktree:session-log` and `post:exit-worktree:cleanup-reminder` | Follows existing naming convention (e.g., `post:bash:pr-created`, `post:bash:build-complete`) | No |
| 7 | Legacy detection via command-pattern matching (option b) | Extend `is_legacy_ecc_hook()` to match `"worktree:create:init"` and `"stop:worktree-cleanup-reminder"` command strings. Fits existing architecture — no changes to `merge_hooks_pure` needed | No |
| 8 | PostToolUse stdin: best-effort extraction with fallback | Handlers extract path from `$.tool_input.worktree_path` or `$.tool_input.name`; fallback to `"unknown"` if not present. PostToolUse format differs from WorktreeCreate format | No |

## User Stories

### US-001: Remove Broken Worktree Delegation Hooks

**As a** developer using Claude Code with ECC, **I want** `EnterWorktree` to work natively, **so that** I can create isolated worktrees for parallel work.

#### Acceptance Criteria

- AC-001.1: Given `hooks/hooks.json` is installed, then no `WorktreeCreate` or `WorktreeRemove` top-level event keys exist, ensuring Claude Code falls back to native git worktree creation.
- AC-001.2: Given `is_legacy_ecc_hook()` receives a command string containing `"worktree:create:init"`, then it returns `true`.
- AC-001.3: Given `is_legacy_ecc_hook()` receives a command string containing `"stop:worktree-cleanup-reminder"`, then it returns `true`.
- AC-001.4: Given a user's `settings.json` has old `WorktreeCreate`/`WorktreeRemove` entries with `ecc-hook` commands, when `remove_legacy_hooks()` runs, then those entries are removed.
- AC-001.5: Given `ecc-domain` config validation, then `WorktreeCreate` and `WorktreeRemove` remain in the `VALID_HOOK_EVENTS` list.

#### Dependencies

- Depends on: none

### US-002: Preserve Worktree Session Logging via PostToolUse

**As a** developer, **I want** worktree creation and removal logged to my session file, **so that** session history tracks worktree operations.

#### Acceptance Criteria

- AC-002.1: Given PostToolUse stdin `{"tool_name":"EnterWorktree","tool_input":{"worktree_path":"/tmp/wt"}}`, when the `post:enter-worktree:session-log` handler runs, then the session file contains `[Worktree] Created: /tmp/wt`. Path is extracted from `$.tool_input.worktree_path` with fallback to `$.tool_input.name`, then `"unknown"`.
- AC-002.2: Given PostToolUse stdin `{"tool_name":"ExitWorktree","tool_input":{"worktree_path":"/tmp/wt"}}`, when the `post:exit-worktree:cleanup-reminder` handler runs, then stderr contains `"Worktree removed"` and `"/tmp/wt"`. Output is via `HookResult::warn()`.
- AC-002.3: Given `hooks/hooks.json`, then `PostToolUse` entries exist with `matcher: "EnterWorktree"` and `matcher: "ExitWorktree"`, calling `ecc-hook "post:enter-worktree:session-log"` and `ecc-hook "post:exit-worktree:cleanup-reminder"` respectively.
- AC-002.4: Given no active session file exists, when the `post:enter-worktree:session-log` handler runs, then it returns `HookResult::passthrough(stdin)` without error.

#### Dependencies

- Depends on: US-001

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `hooks/hooks.json` | Config | Remove WorktreeCreate/WorktreeRemove; add PostToolUse entries |
| `crates/ecc-domain/src/config/merge.rs` | Domain | Add legacy detection patterns for old worktree hooks |
| `crates/ecc-app/src/hook/mod.rs` | Application | Update dispatch: replace old hook IDs with new PostToolUse hook IDs |
| `crates/ecc-app/src/hook/handlers/tier3_session/worktree.rs` | Application | Repurpose handler for PostToolUse stdin format |
| `crates/ecc-app/src/hook/handlers/tier3_session/mod.rs` | Application | Update re-exports |
| `crates/ecc-app/src/hook/handlers/tier1_simple/dev_hooks.rs` | Application | Repurpose worktree_cleanup_reminder for PostToolUse |
| `crates/ecc-integration-tests/tests/hook_dispatch.rs` | Test | Update integration tests for new hook IDs |

## Constraints

- Must not remove `WorktreeCreate`/`WorktreeRemove` from `VALID_HOOK_EVENTS` in ecc-domain
- PostToolUse hooks receive different stdin JSON than WorktreeCreate — handlers must parse `{"tool_name":"EnterWorktree","tool_input":{...}}`
- `ecc install` must auto-clean old entries — users shouldn't need manual intervention
- Existing `is_legacy_ecc_hook()` pattern list must be extended, not replaced

## Non-Requirements

- Not implementing actual worktree creation in hooks (Option B rejected)
- Not changing the `ecc install` merge algorithm itself
- Not addressing BL-065 Sub-Spec C in this fix (unblocking only)
- Not adding E2E tests that require a real `EnterWorktree` invocation

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| Hook dispatch (ecc-app) | Modified routing | Integration tests verify new hook IDs dispatch correctly |
| Legacy detection (ecc-domain) | Extended patterns | Unit test for merge cleanup of old worktree hooks |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| Bug fix | Minor | CHANGELOG.md | Add BL-085 entry |
| Config | Minor | hooks/hooks.json | Self-documenting (description fields) |

## Open Questions

None — all resolved during investigation and grill-me interview.

## Phase Summary

### Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|
| 1 | Root cause vs symptom? | Root cause — hook semantic mismatch confirmed | Recommended |
| 2 | Minimal vs proper fix? | Proper structural fix — remove delegation hooks, add PostToolUse, legacy detection | Recommended |
| 3 | Missing test scenarios? | Mirror existing coverage + regression test for hooks.json keys | Recommended |
| 4 | Legacy detection for existing users? | Yes, add to is_legacy_ecc_hook() | Recommended |
| 5 | Related audit findings? | No prior audits flagged this issue | N/A |
| 6 | Reproduction steps? | Yes, include in spec | Recommended |
| 7 | Data impact? | No data migration needed | Recommended |

### User Stories

| ID | Title | AC Count | Dependencies |
|----|-------|----------|--------------|
| US-001 | Remove Broken Worktree Delegation Hooks | 5 | none |
| US-002 | Preserve Worktree Session Logging via PostToolUse | 4 | US-001 |

### Acceptance Criteria

| AC ID | Description | Source US |
|-------|-------------|----------|
| AC-001.1 | No WorktreeCreate/WorktreeRemove keys in hooks.json | US-001 |
| AC-001.2 | is_legacy_ecc_hook detects worktree:create:init | US-001 |
| AC-001.3 | is_legacy_ecc_hook detects stop:worktree-cleanup-reminder | US-001 |
| AC-001.4 | remove_legacy_hooks cleans old entries | US-001 |
| AC-001.5 | VALID_HOOK_EVENTS retains WorktreeCreate/WorktreeRemove | US-001 |
| AC-002.1 | PostToolUse handler logs worktree creation to session file | US-002 |
| AC-002.2 | PostToolUse handler emits cleanup reminder on ExitWorktree | US-002 |
| AC-002.3 | hooks.json has PostToolUse entries for EnterWorktree/ExitWorktree | US-002 |
| AC-002.4 | Graceful degradation when no session file exists | US-002 |

### Adversary Findings

| Dimension | Score | Verdict | Key Rationale |
|-----------|-------|---------|---------------|
| Completeness | 90 | PASS | All 8 decisions documented, affected modules verified |
| Correctness | 88 | PASS | PostToolUse stdin format and JSON path change specified |
| Testability | 85 | PASS | All ACs have deterministic test paths |
| Feasibility | 95 | PASS | Legacy detection fits existing is_legacy_ecc_hook patterns |
| Consistency | 92 | PASS | Hook IDs follow post:<tool>:<purpose> convention |
| Security | 95 | PASS | No new attack surface, paths only used for logging |
| Clarity | 90 | PASS | Decisions numbered, rationale provided, non-requirements explicit |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-03-28-fix-worktree-hook-registration/spec.md | Full spec + phase summary |
