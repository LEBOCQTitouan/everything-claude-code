# Design: Fix Phase-Gate Hook State Resolution in Worktrees (BL-131)

## Spec Reference
`docs/specs/2026-04-07-phase-gate-worktree-state-fix/spec.md`

## File Changes

| # | File | Change | Layer | Spec Ref |
|---|------|--------|-------|----------|
| 1 | `crates/ecc-app/src/workflow/state_resolver.rs` | Add `read_anchor()` helper; call before git resolution; 3 new tests | App (UseCase) | AC-001.2/3/6/7 |
| 2 | `crates/ecc-workflow/src/commands/init.rs` | Add `write_anchor()` helper; call after state.json write; 2 new tests | Adapter | AC-001.1/8 |
| 3 | `crates/ecc-workflow/src/commands/reset.rs` | Add `project_dir` param; delete .state-dir best-effort; 1 new test | Adapter | AC-001.4/9 |
| 4 | `crates/ecc-workflow/src/main.rs` | Pass `&proj` to reset dispatch (line 300) | Adapter | — |
| 5 | `.claude/workflow/implement-done.md` | `git rm --cached` (one-time) | Git index | AC-002.1 |

## Pass Conditions

| PC | Type | Command | Expected | Verifies |
|----|------|---------|----------|----------|
| PC-001 | Unit | `cargo test -p ecc-app -- anchor_file_overrides_git_resolution` | PASS | AC-001.2, AC-001.6 |
| PC-002 | Regression | `cargo test -p ecc-app -- worktree_returns_git_dir` | PASS | AC-001.3 |
| PC-003 | Unit | `cargo test -p ecc-app -- corrupt_anchor_falls_back` | PASS | AC-001.7 |
| PC-004 | Unit | `cargo test -p ecc-app -- stale_anchor_falls_back` | PASS | AC-001.7 |
| PC-005 | Integration | `cargo test -p ecc-workflow -- init_writes_state_dir_anchor` | PASS | AC-001.1, AC-001.8 |
| PC-006 | Integration | `cargo test -p ecc-workflow -- init_succeeds_without_anchor` | PASS | AC-001.8 |
| PC-007 | Integration | `cargo test -p ecc-workflow -- reset_deletes_state_dir_anchor` | PASS | AC-001.4 |
| PC-008 | Regression | `cargo test -p ecc-workflow -- reset_force_deletes` | PASS | AC-001.9 |
| PC-009 | Regression | `cargo test -p ecc-app -- workflow::state_resolver` | all PASS | AC-001.5 |
| PC-010 | Regression | `cargo test -p ecc-workflow -- init` | all PASS | AC-001.5 |
| PC-011 | Regression | `cargo test -p ecc-workflow -- reset` | all PASS | AC-001.5 |
| PC-012 | Lint | `cargo clippy -p ecc-app -p ecc-workflow -- -D warnings` | exit 0 | — |
| PC-013 | Build | `cargo build -p ecc-app -p ecc-workflow` | exit 0 | — |
| PC-014 | Verify | `git check-ignore .claude/workflow/.state-dir` | exit 0, prints path | AC-002.2 |
| PC-015 | Verify | `test -z "$(git ls-files .claude/workflow/implement-done.md)"` | exit 0 | AC-002.1 |

## Coverage Check

| AC | Covered by PC |
|----|---------------|
| AC-001.1 | PC-005 |
| AC-001.2 | PC-001 |
| AC-001.3 | PC-002 |
| AC-001.4 | PC-007 |
| AC-001.5 | PC-009, PC-010, PC-011 |
| AC-001.6 | PC-001 |
| AC-001.7 | PC-003, PC-004 |
| AC-001.8 | PC-005, PC-006 |
| AC-001.9 | PC-007, PC-008 |
| AC-002.1 | PC-015 |
| AC-002.2 | PC-014 |

**11/11 ACs covered.**

## Implementation Strategy

### Wave 1: state_resolver.rs (App layer)
1. Write 3 failing tests (RED): anchor overrides git, corrupt fallback, stale fallback
2. Implement `read_anchor()` helper + insert before git resolution (GREEN)
3. Extract `ANCHOR_FILE_NAME` constant (REFACTOR)

### Wave 2: init.rs (Adapter layer)
1. Write 2 failing tests: anchor written, init survives failure
2. Implement `write_anchor()` helper after state.json write (GREEN)
3. Remove dead `let _ = project_dir` line

### Wave 3: reset.rs + main.rs (Adapter layer)
1. Write 1 failing test: anchor deleted on reset
2. Add `project_dir` param to `reset::run()`, update main.rs dispatch, delete anchor (GREEN)
3. Update all existing reset test call sites

### Wave 4: Git cleanup
1. `git rm --cached .claude/workflow/implement-done.md`

## Design Highlights

- **Anchor read in app layer** uses `FileSystem` port (testable with `InMemoryFileSystem`)
- **Anchor write/delete in adapter layer** uses `std::fs` directly (consistent with existing init.rs/reset.rs patterns — Uncle Bob: CLEAN)
- **Fail-open**: corrupt/stale/missing anchor -> git resolution fallback + warning
- **Atomic write**: temp + rename for .state-dir (Security: CLEAR)
- **Read cap**: 4096 bytes max to prevent DoS from corrupted anchor (Security rec)
- **Function extraction**: `read_anchor()` and `write_anchor()` helpers to keep functions under 50 lines (Robert: Oath 2)

## E2E Test Plan

No new E2E tests needed. Unit tests with in-memory doubles + tempdir integration tests cover all ACs.

## E2E Activation Rules

| Boundary | Adapter | Port | Description | Status | Activation |
|----------|---------|------|-------------|--------|------------|
| FileSystem | OsFileSystem | FileSystem | Read .state-dir anchor | ignored | Only if anchor read crosses process boundary |

## Test Strategy

- **Unit tests** (Wave 1): `state_resolver.rs` — 3 new tests with `InMemoryFileSystem` + `MockGitInfo`
- **Integration tests** (Wave 2-3): `init.rs` — 2 new, `reset.rs` — 1 new, all using `tempfile::TempDir`
- **Regression**: All existing state_resolver + init + reset tests

## SOLID Assessment

Uncle Bob: **CLEAN**. std::fs in adapter layer is correct. Anchor location is project_dir-relative. No SOLID violations.

## Robert's Oath Check

1 WARNING (Oath 2): Extract `read_anchor()` and `write_anchor()` helpers to keep functions under 50 lines.

## Security Notes

Security reviewer: **CLEAR**. Path traversal blocked by `is_absolute()` check. Symlinks acceptable in threat model. TOCTOU benign (fail-open). 2 LOW hardening recs: atomic write, 4096 byte read cap.

## Rollback Plan

Deploy previous binary version — it ignores .state-dir (doesn't read it). Falls back to git resolution. Orphaned .state-dir files are harmless (gitignored, next init overwrites). `git rm --cached` reversal requires `git checkout` of the file into the index.

## Doc Update Plan

| Doc File | Level | Action | Content Summary | Spec Ref |
|----------|-------|--------|-----------------|----------|
| CHANGELOG.md | root | Add entry | fix: resolve phase-gate state in worktrees via .state-dir anchor | US-001 |
| CLAUDE.md | Gotchas | Update | Add .state-dir anchor to worktree state resolution gotcha | US-001 |

## Bounded Contexts Affected

- **Workflow** (state lifecycle): init, reset, state resolution — all within same context
