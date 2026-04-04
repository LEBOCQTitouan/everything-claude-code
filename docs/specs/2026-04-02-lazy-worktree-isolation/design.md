# Design: Lazy Worktree Isolation with Session-End Merge

## Overview

Two new Rust hook handlers (write-guard + session-end merge) and configuration entries to enforce worktree isolation lazily. When Claude attempts a file write outside a worktree, the PreToolUse handler blocks the operation (exit 2), causing Claude to call `EnterWorktree` before retrying. At session end, a SessionEnd handler calls `ecc-workflow merge` to rebase, verify, and fast-forward merge back to main.

All hooks follow existing ECC patterns: Rust handler functions dispatched via `ecc-hook`, using port traits for testability.

## Architecture Decision

**ADR-0042: Lazy Worktree via Write-Guard Pattern** -- hooks cannot call agent tools (`EnterWorktree`), so we use a blocking PreToolUse hook that forces Claude to call `EnterWorktree` itself. This is the only feasible approach given Claude Code's architecture.

## File Changes (Dependency Order)

| # | File | Change | Layer | AC Coverage |
|---|------|--------|-------|-------------|
| 1 | `crates/ecc-app/src/hook/handlers/tier1_simple/worktree_guard.rs` | **New**: write-guard handler (`pre_worktree_write_guard`) | Adapter | AC-001.1 through AC-001.7 |
| 2 | `crates/ecc-app/src/hook/handlers/tier1_simple/mod.rs` | **Modify**: add `mod worktree_guard` + re-export | Adapter | -- |
| 3 | `crates/ecc-app/src/hook/handlers/mod.rs` | **Modify**: re-export new handler | Adapter | -- |
| 4 | `crates/ecc-app/src/hook/mod.rs` | **Modify**: add dispatch route for `"pre:write-edit:worktree-guard"` | Adapter | -- |
| 5 | `crates/ecc-app/src/hook/handlers/tier3_session/session_merge.rs` | **New**: session-end merge handler (`session_end_merge`) | Adapter | AC-002.1 through AC-002.9 |
| 6 | `crates/ecc-app/src/hook/handlers/tier3_session/mod.rs` | **Modify**: add `mod session_merge` + re-export | Adapter | -- |
| 7 | `crates/ecc-app/src/hook/handlers/mod.rs` | **Modify**: re-export session_end_merge | Adapter | -- |
| 8 | `crates/ecc-app/src/hook/mod.rs` | **Modify**: add dispatch route for `"session:end:worktree-merge"` | Adapter | -- |
| 9 | `.claude/settings.json` | **Modify**: add PreToolUse + SessionEnd hook entries | Configuration | AC-003.1, AC-003.2, AC-003.3 |
| 10 | `hooks/hooks.json` | **Modify**: add hook definitions for pipeline context | Configuration | AC-003.4 |
| 11 | `CLAUDE.md` | **Modify**: add Gotchas entries | Documentation | AC-004.1, AC-004.2 |
| 12 | `docs/adr/0042-lazy-worktree-write-guard.md` | **New**: ADR | Documentation | AC-004.4 |
| 13 | `docs/glossary.md` or inline in CLAUDE.md | **Modify**: define terms | Documentation | AC-004.3 |
| 14 | Final gate: `cargo clippy` + `cargo build` | -- | -- | Build integrity |

## Detailed Design

### PC-001: Write-Guard Handler (File 1)

**Hook ID**: `pre:write-edit:worktree-guard`
**Trigger**: PreToolUse on `Write|Edit|MultiEdit`
**Profile**: `standard,strict`

Logic (in order):
1. Check `ECC_WORKFLOW_BYPASS` env var via `ports.env.var()` -- if `"1"`, return `passthrough(stdin)` (AC-001.3)
2. Run `git rev-parse --show-toplevel` via `ports.shell` -- if it fails (non-git dir), return `passthrough(stdin)` (AC-001.4)
3. Run `git rev-parse --git-common-dir` via `ports.shell` -- get the common git dir
4. Resolve the parent of `git-common-dir` to get the main repo root
5. Compare `show-toplevel` vs resolved parent of `git-common-dir`:
   - If they differ: we are in a worktree, return `passthrough(stdin)` (AC-001.2, AC-001.6)
   - If they match: we are NOT in a worktree, return `block(stdin, message)` with exit code 2 (AC-001.1)
6. Block message includes:
   - Primary: `"[Hook] BLOCKED: Not in a worktree. Call EnterWorktree before making changes."`
   - Fallback: `"If EnterWorktree is unavailable, set ECC_WORKFLOW_BYPASS=1 to proceed on main"` (AC-001.7)

**Worktree Detection Logic** (AC-001.5):
```
toplevel = git rev-parse --show-toplevel     # e.g., /repo/.claude/worktrees/ecc-session-xxx
common   = git rev-parse --git-common-dir    # e.g., /repo/.git
parent   = dirname(common)                   # e.g., /repo

if toplevel != parent => in worktree => allow
if toplevel == parent => NOT in worktree => block
```

When `git-common-dir` returns a relative path (e.g., `.` or `../../.git`), the handler resolves it relative to `show-toplevel` before comparing. This handles both absolute and relative `git-common-dir` outputs.

### PC-002: Session-End Merge Handler (File 5)

**Hook ID**: `session:end:worktree-merge`
**Trigger**: SessionEnd

Logic (in order):
1. Check `ECC_WORKFLOW_BYPASS` env var -- if `"1"`, return `passthrough(stdin)` (AC-002.7)
2. Detect worktree status (same logic as write-guard) -- if NOT in worktree, return `passthrough(stdin)` (AC-002.2)
3. Check if there are commits ahead of main: `git rev-list HEAD ^main --count`
   - If `"0"`: no commits, clean up empty worktree via `git worktree remove --force` and return `passthrough(stdin)` (AC-002.6)
4. Set up a trap/cleanup for the `.ecc-merge-recovery` file (AC-002.8)
5. Call `ecc-workflow merge` via `ports.shell.run_command("ecc-workflow", &["merge"])`
6. Handle exit codes from `ecc-workflow merge` (AC-002.9):
   - Exit 0: success, return `passthrough(stdin)` with success message on stderr (AC-002.1)
   - Exit 2 with "Rebase conflict" in stderr: worktree preserved, return `warn(stdin, remediation)` (AC-002.3)
   - Exit 2 with "verify failed" in stderr: worktree preserved, return `warn(stdin, remediation)` (AC-002.4)
   - Exit 2 with "lock held" in stderr: return `warn(stdin, "retry later")` (AC-002.5)
   - Other failures: write `.ecc-merge-recovery` file and warn (AC-002.8)

**Important**: The `ecc-workflow merge` binary already handles rebase + verify + ff-only + cleanup internally. The SessionEnd handler is a thin orchestrator that:
- Decides whether to invoke merge (worktree check, empty check)
- Maps `ecc-workflow merge` output to `HookResult`
- Writes recovery file on unexpected failures

**Recovery file** (AC-002.8):
Written to `$WORKTREE_ROOT/.ecc-merge-recovery` via `ports.fs.write()`:
```
# ECC Merge Recovery
# Merge was interrupted at: <timestamp>
# Worktree: <path>
# Branch: <branch>
#
# To retry: cd <path> && ecc-workflow merge
# To clean up: ecc worktree gc --force
```

### PC-003: Write-Guard in settings.json (File 9)

Add to `.claude/settings.json` under `hooks.PreToolUse`:
```json
{
  "matcher": "Write|Edit|MultiEdit",
  "hooks": [
    {
      "type": "command",
      "command": "ecc-hook \"pre:write-edit:worktree-guard\" \"standard,strict\""
    }
  ],
  "description": "Lazy worktree: block file writes outside a worktree"
}
```

### PC-004: Session-End Merge in settings.json (File 9)

Add to `.claude/settings.json` under `hooks.SessionEnd`:
```json
{
  "matcher": "*",
  "hooks": [
    {
      "type": "command",
      "command": "ecc-hook \"session:end:worktree-merge\" \"standard,strict\"",
      "timeout": 300
    }
  ],
  "description": "Lazy worktree: merge session worktree back to main at session end"
}
```

Note the 300s timeout to accommodate cargo build+test+clippy (~3 min).

### PC-005: hooks.json Definitions (File 10)

No changes needed to `hooks/hooks.json`. That file is for `ecc-workflow` hooks (pipeline state machine), not for `ecc-hook` hooks. The write-guard and merge hooks are `ecc-hook` handlers configured in `.claude/settings.json`.

### PC-006: CLAUDE.md Gotchas (File 11)

Add two entries to the Gotchas section:

```markdown
- `pre:write-edit:worktree-guard` blocks Write/Edit/MultiEdit on main branch -- Claude must call EnterWorktree first; bypass with `ECC_WORKFLOW_BYPASS=1`
- `session:end:worktree-merge` auto-merges worktree to main at session end -- if merge fails, the worktree is preserved; retry with `ecc-workflow merge` or clean up with `ecc worktree gc`
```

### PC-007: ADR-0042 (File 12)

ADR documenting:
- **Context**: hooks cannot call agent tools; need to enforce worktree isolation
- **Decision**: PreToolUse write-guard blocks writes, causing Claude to call EnterWorktree naturally
- **Consequences**: read-only sessions have zero overhead; write sessions get lazy isolation; SessionEnd merge automates integration

### PC-008: Glossary Terms (File 13)

Define in CLAUDE.md or docs:
- **Write-guard**: PreToolUse hook that blocks file writes outside a worktree (exit 2)
- **Lazy worktree**: Worktree created on-demand when first write is attempted, not at session start
- **Session merge**: Automatic rebase + verify + ff-merge of worktree branch into main at session end

## Pass Conditions Table

| PC | Description | Verification Command | AC |
|----|-------------|---------------------|----|
| PC-001a | Write-guard blocks outside worktree | `cargo test -p ecc-app --lib -- hook::handlers::tier1_simple::worktree_guard::tests::blocks_write_outside_worktree` | AC-001.1 |
| PC-001b | Write-guard allows inside worktree | `cargo test -p ecc-app --lib -- hook::handlers::tier1_simple::worktree_guard::tests::allows_write_inside_worktree` | AC-001.2 |
| PC-001c | Write-guard bypass | `cargo test -p ecc-app --lib -- hook::handlers::tier1_simple::worktree_guard::tests::bypass_allows_write` | AC-001.3 |
| PC-001d | Write-guard non-git passthrough | `cargo test -p ecc-app --lib -- hook::handlers::tier1_simple::worktree_guard::tests::non_git_passthrough` | AC-001.4 |
| PC-001e | Worktree detection logic | `cargo test -p ecc-app --lib -- hook::handlers::tier1_simple::worktree_guard::tests::detects_worktree_via_git_common_dir` | AC-001.5 |
| PC-001f | Pipeline worktree passes | `cargo test -p ecc-app --lib -- hook::handlers::tier1_simple::worktree_guard::tests::pipeline_worktree_passes` | AC-001.6 |
| PC-001g | Fallback message | `cargo test -p ecc-app --lib -- hook::handlers::tier1_simple::worktree_guard::tests::block_message_includes_fallback` | AC-001.7 |
| PC-002a | Merge called in worktree | `cargo test -p ecc-app --lib -- hook::handlers::tier3_session::session_merge::tests::calls_merge_in_worktree` | AC-002.1 |
| PC-002b | Skip when not in worktree | `cargo test -p ecc-app --lib -- hook::handlers::tier3_session::session_merge::tests::skips_when_not_in_worktree` | AC-002.2 |
| PC-002c | Rebase conflict preserves | `cargo test -p ecc-app --lib -- hook::handlers::tier3_session::session_merge::tests::rebase_conflict_preserves_worktree` | AC-002.3 |
| PC-002d | Verify failure preserves | `cargo test -p ecc-app --lib -- hook::handlers::tier3_session::session_merge::tests::verify_failure_preserves_worktree` | AC-002.4 |
| PC-002e | Lock held warns | `cargo test -p ecc-app --lib -- hook::handlers::tier3_session::session_merge::tests::lock_held_warns` | AC-002.5 |
| PC-002f | Empty worktree cleanup | `cargo test -p ecc-app --lib -- hook::handlers::tier3_session::session_merge::tests::empty_worktree_cleaned_up` | AC-002.6 |
| PC-002g | Bypass skips merge | `cargo test -p ecc-app --lib -- hook::handlers::tier3_session::session_merge::tests::bypass_skips_merge` | AC-002.7 |
| PC-002h | Recovery file on failure | `cargo test -p ecc-app --lib -- hook::handlers::tier3_session::session_merge::tests::writes_recovery_file_on_unexpected_failure` | AC-002.8 |
| PC-002i | Exit code mapping | `cargo test -p ecc-app --lib -- hook::handlers::tier3_session::session_merge::tests::maps_exit_codes_correctly` | AC-002.9 |
| PC-003a | settings.json PreToolUse entry | `grep -q 'pre:write-edit:worktree-guard' .claude/settings.json` | AC-003.1 |
| PC-003b | settings.json SessionEnd entry | `grep -q 'session:end:worktree-merge' .claude/settings.json` | AC-003.2 |
| PC-003c | Hook conventions | `grep -q 'ECC_WORKFLOW_BYPASS' crates/ecc-app/src/hook/handlers/tier1_simple/worktree_guard.rs` (bypass is handled by dispatch, but we verify in tests) | AC-003.3 |
| PC-004a | CLAUDE.md write-guard gotcha | `grep -q 'worktree-guard' CLAUDE.md` | AC-004.1 |
| PC-004b | CLAUDE.md session merge gotcha | `grep -q 'worktree-merge' CLAUDE.md` | AC-004.2 |
| PC-004c | Glossary terms | `grep -qi 'write-guard\|lazy worktree\|session merge' CLAUDE.md` | AC-004.3 |
| PC-004d | ADR exists | `test -f docs/adr/0042-lazy-worktree-write-guard.md` | AC-004.4 |
| PC-FINAL-a | Clippy clean | `cargo clippy -- -D warnings` | Build integrity |
| PC-FINAL-b | Build succeeds | `cargo build` | Build integrity |
| PC-FINAL-c | All tests pass | `cargo test` | Build integrity |

## TDD Order

### Phase 1: Write-Guard Handler (US-001)
**Layers**: Adapter

1. **RED**: Create `worktree_guard.rs` with test module containing tests for PC-001a through PC-001g. All tests use `MockExecutor` to simulate git commands.
2. **GREEN**: Implement `pre_worktree_write_guard` function.
3. **REFACTOR**: Extract worktree detection into a shared helper function (will be reused by Phase 2).
4. Wire up: add `mod worktree_guard` to tier1_simple/mod.rs, re-export in handlers/mod.rs, add dispatch route in hook/mod.rs.
5. Verify: `cargo test -p ecc-app --lib -- worktree_guard`

### Phase 2: Session-End Merge Handler (US-002)
**Layers**: Adapter

1. **RED**: Create `session_merge.rs` with test module containing tests for PC-002a through PC-002i. Tests mock `ecc-workflow merge` via `MockExecutor`.
2. **GREEN**: Implement `session_end_merge` function, reusing the worktree detection helper from Phase 1.
3. **REFACTOR**: Clean up error message formatting.
4. Wire up: add `mod session_merge` to tier3_session/mod.rs, re-export in handlers/mod.rs, add dispatch route in hook/mod.rs.
5. Verify: `cargo test -p ecc-app --lib -- session_merge`

### Phase 3: Configuration (US-003)
**Layers**: Configuration

1. Add PreToolUse entry to `.claude/settings.json` (PC-003a)
2. Add SessionEnd entry to `.claude/settings.json` (PC-003b)
3. Verify: `grep` checks for PC-003a, PC-003b

### Phase 4: Documentation (US-004)
**Layers**: Documentation

1. Add Gotchas entries to `CLAUDE.md` (PC-004a, PC-004b)
2. Add glossary terms to `CLAUDE.md` (PC-004c)
3. Create `docs/adr/0042-lazy-worktree-write-guard.md` (PC-004d)
4. Verify: `grep` and `test -f` checks

### Phase 5: Final Gate

1. `cargo clippy -- -D warnings` (PC-FINAL-a)
2. `cargo build` (PC-FINAL-b)
3. `cargo test` (PC-FINAL-c)

## Shared Helper: Worktree Detection

Extract into a function reusable by both handlers:

```rust
// In a helpers module accessible to both tier1_simple and tier3_session
/// Detect whether the current working directory is inside a git worktree.
///
/// Returns:
/// - Ok(true)  if in a worktree (show-toplevel != parent of git-common-dir)
/// - Ok(false) if NOT in a worktree (show-toplevel == parent of git-common-dir)
/// - Err(())   if not a git repository (graceful degradation)
fn is_in_worktree(shell: &dyn ShellExecutor) -> Result<bool, ()>
```

Place this in `crates/ecc-app/src/hook/handlers/helpers.rs` (new file) or in an existing shared helpers module.

## E2E Assessment

- **Touches user-facing flows?** Yes -- PreToolUse blocks writes, SessionEnd triggers merge
- **Crosses 3+ modules end-to-end?** No -- hook handlers + config only (2 modules)
- **New E2E tests needed?** No -- unit tests with MockExecutor cover the logic; the existing `ecc-workflow merge` already has integration tests with real git repos. Run existing E2E suite as gate.

## Risks & Mitigations

| Risk | Severity | Mitigation |
|------|----------|------------|
| SessionEnd hook timeout (build+test+clippy > 300s) | Medium | 300s timeout; if exceeded, worktree preserved for manual merge |
| `git rev-parse --git-common-dir` returns relative path | Low | Resolve relative to show-toplevel before comparing |
| Double-guard interaction (write-guard + branch-guard) | Low | Write-guard checks worktree only; branch-guard checks branch name. Independent concerns. |

## Adversarial Round 2 Additions

### Additional PCs

| PC | Description | Command | AC |
|----|-------------|---------|-----|
| PC-001h | Relative git-common-dir path resolution | `cargo test -p ecc-app --lib -- worktree_guard::tests::handles_relative_git_common_dir` | AC-001.5 |
| PC-001i | Double-guard coexistence | `cargo test -p ecc-app --lib -- worktree_guard::tests::coexists_with_branch_guard` | AC-001.6 |

### Exit Code Handling Fix

`ecc-workflow merge` exit codes mapped without fragile string matching:
- Exit 0: success (AC-002.1)
- Any non-zero: preserve worktree, warn with stderr content forwarded verbatim (AC-002.3, AC-002.4, AC-002.5)
- Recovery file written on any non-zero exit (AC-002.8)

### CHANGELOG.md

Added to Phase 4 file changes (was missing from round 1).

## Phase Summary

### Design Reviews

| Review Type | Verdict | Finding Count |
|-------------|---------|---------------|
| SOLID | PASS | 0 |
| Robert | PASS | 0 |
| Security | PASS | 0 |

### Adversary Findings

| Dimension | Score | Verdict | Key Rationale |
|-----------|-------|---------|---------------|
| Completeness | 92 | PASS | 29 PCs cover all 25 ACs |
| Fragility | 88 | PASS | Relative path resolution + exit-code propagation |
| Correctness | 90 | PASS | Exit 0 success, non-zero preserve+warn |
| Testability | 90 | PASS | Double-guard interaction test |
| Security | 95 | PASS | No new attack surface |
| Consistency | 92 | PASS | Follows hook conventions |
| Dependency Order | 94 | PASS | Phase sequence respects crate DAG |
| Documentation | 88 | PASS | CHANGELOG added to Phase 4 |

### File Changes Summary

| # | File | Action | Spec Ref |
|---|------|--------|----------|
| 1 | crates/ecc-app/src/hook/handlers/tier1_simple/worktree_guard.rs | Create | US-001 |
| 2 | crates/ecc-app/src/hook/handlers/tier3_session/session_merge.rs | Create | US-002 |
| 3 | crates/ecc-app/src/hook/handlers/helpers.rs | Create | US-001, US-002 |
| 4 | crates/ecc-app/src/hook/handlers/tier1_simple/mod.rs | Modify | -- |
| 5 | crates/ecc-app/src/hook/handlers/tier3_session/mod.rs | Modify | -- |
| 6 | crates/ecc-app/src/hook/handlers/mod.rs | Modify | -- |
| 7 | crates/ecc-app/src/hook/mod.rs | Modify | -- |
| 8 | .claude/settings.json | Modify | US-003 |
| 9 | CLAUDE.md | Modify | US-004 |
| 10 | docs/adr/0042-lazy-worktree-write-guard.md | Create | US-004 |
| 11 | CHANGELOG.md | Modify | Convention |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-04-02-lazy-worktree-isolation/spec.md | Full spec + Phase Summary |
| docs/specs/2026-04-02-lazy-worktree-isolation/design.md | Full design + Phase Summary |
| Hook fires in non-ECC projects | Low | Non-git passthrough (AC-001.4); bypass env var |
| `ecc-workflow merge` not on PATH | Low | Shell error → recovery file written (AC-002.8) |
| Write-guard + existing workflow-branch-guard double-blocking | Low | Both return exit 2; Claude sees the first block message and acts on it |
