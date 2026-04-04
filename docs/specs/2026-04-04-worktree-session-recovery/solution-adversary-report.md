# Solution Adversary Report

## Summary
Verdict: PASS (avg: 86/100)
Rounds: 2 of 3

## Dimension Results
| # | Dimension | Score | Verdict | Critical Findings |
|---|-----------|-------|---------|-------------------|
| 1 | AC Coverage | 95 | PASS | All 9 ACs mapped; coverage table complete |
| 2 | Execution Order | 82 | PASS | Round 1 project_dir gap resolved; TDD order valid |
| 3 | Fragility | 75 | PASS | Mock-based approach is stable; PC-006 existing test needs verification |
| 4 | Rollback Adequacy | 85 | PASS | Reverse-order plan covers all 5 files; purely subtractive critical path |
| 5 | Architecture Compliance | 90 | PASS | Intra-crate gc call is clean; hex rules respected |
| 6 | Blast Radius | 90 | PASS | 5 files, 2 crates, minimal cross-boundary risk |
| 7 | Missing Pass Conditions | 82 | PASS | Round 1 gap resolved: PC-013 cargo fmt added |
| 8 | Doc Plan Completeness | 85 | PASS | CHANGELOG and CLAUDE.md covered; no ADRs needed |

## Uncovered ACs
| AC | Description (from spec) | Suggested PC |
|----|------------------------|--------------|
| (none) | All 9 ACs have covering PCs | N/A |

## Detailed Findings

### 1. AC Coverage
- **Finding**: All 9 ACs (AC-001.1 through AC-002.4) remain fully mapped in the Coverage Check table (design lines 32-43). No regression from round 1.
- **Evidence**: Each AC has at least one PC; AC-001.5 has triple coverage (PC-001, PC-002, PC-004). AC-002.4 is covered by PC-008 via the alive-PID check which prevents gc from touching in-merge worktrees.
- **Recommendation**: None.

### 2. Execution Order
- **Finding**: Round 1 flagged that `project_dir` acquisition was unspecified. Round 2 design now specifies: "Resolve `project_dir` via `ports.shell.run_command("git", &["rev-parse", "--show-toplevel"])`". This is correct -- `HookPorts` provides `shell: &dyn ShellExecutor`, and the gc-failure-is-non-blocking requirement (AC-002.3) covers the case where the git command fails (e.g., non-git directory).
- **Evidence**: FC-3 description now includes the resolution method.
- **Recommendation**: None. Gap closed.

- **Finding**: TDD order remains valid. Steps 1-2 (session_merge.rs, merge.rs) are independent. Step 3 (PC-006) verifies an existing test. Step 4 (lifecycle.rs) depends on `worktree::gc()` which already exists. Step 5 (lint/build/suite) is correctly last.
- **Evidence**: Test Strategy lines 57-63.
- **Recommendation**: None.

### 3. Fragility
- **Finding**: PC-006 references existing test `rebase_conflict_preserves_worktree`. The design modifies `session_merge.rs` to change the zero-commit path, but the rebase conflict test exercises the non-zero-exit-code path from `ecc-workflow merge`. These are distinct code paths -- the zero-commit check happens before merge execution, while rebase conflicts occur during merge. No overlap.
- **Evidence**: session_merge.rs zero-commit path is reached via `git rev-list --count` returning "0"; rebase conflict path is reached via non-zero exit from the merge subprocess.
- **Recommendation**: Low risk, but the implementer should run PC-006 explicitly after FC-1 changes to confirm no side effects.

- **Finding**: PC commands use `cargo test -p <crate> <test_name_filter>`. These filters match substrings, so a test named `empty_worktree_defers_to_gc_extra` would also match PC-001's filter. This is a minor fragility -- unlikely to cause false positives given the descriptive naming convention.
- **Evidence**: All PC commands use substring-match test filters.
- **Recommendation**: Not actionable for this design; standard cargo test behavior.

### 4. Rollback Adequacy
- **Finding**: Rollback plan lists 4 steps in reverse dependency order: lifecycle.rs -> merge.rs -> session_merge.rs -> docs. This correctly mirrors the File Changes order reversed. No data migrations or irreversible state changes exist.
- **Evidence**: Design lines 79-84.
- **Recommendation**: None.

- **Finding**: FC-2 now explicitly mentions removing associated test(s) for dead `cleanup_worktree`. The rollback plan step 2 ("restore cleanup_worktree call and success message") implicitly covers restoring those tests, but does not say so explicitly.
- **Evidence**: Rollback step 2: "Revert merge.rs (restore cleanup_worktree call and success message)" -- no mention of tests.
- **Recommendation**: Minor gap. A git revert of the merge.rs commit would restore everything including tests, so this is operationally safe even if the description is incomplete.

### 5. Architecture Compliance
- **Finding**: `lifecycle.rs` (hook handler, adapter layer in ecc-app) calling `crate::worktree::gc()` (application layer in ecc-app) with `ports.shell` (port trait from ecc-ports) -- this is clean. Dependencies flow inward: adapter -> application -> port trait.
- **Evidence**: `gc` signature: `gc(executor: &dyn ShellExecutor, project_dir: &Path, _force: bool)`. `HookPorts.shell: &dyn ShellExecutor`.
- **Recommendation**: None.

- **Finding**: `ecc-domain` is not touched by this design. No I/O imports introduced anywhere in the domain layer.
- **Evidence**: File Changes table has no entries for ecc-domain.
- **Recommendation**: None.

- **Finding**: `ecc-workflow` binary crate uses `std::process::Command` directly. This is existing behavior, acceptable per crate CLAUDE.md for standalone binaries. The design removes code from this crate but adds nothing new.
- **Evidence**: FC-2 is purely subtractive for ecc-workflow.
- **Recommendation**: None.

### 6. Blast Radius
- **Finding**: 5 files across 2 code crates (ecc-app, ecc-workflow) plus 2 documentation files. Well under the 20-file threshold. No public API changes -- `ecc-workflow merge` still accepts the same arguments and produces the same exit codes. Only the success message text changes and the worktree-removal side effect is removed.
- **Evidence**: File Changes table has 5 entries. Spec constraint: "Must not change public API of ecc-workflow merge."
- **Recommendation**: None.

### 7. Missing Pass Conditions
- **Finding**: Round 1 CONDITIONAL was due to missing `cargo fmt --check`. Round 2 adds PC-013 (`cargo fmt --check`). All structural PCs now present: lint (PC-010 clippy, PC-013 fmt), build (PC-011), suite (PC-012).
- **Evidence**: PC-013 in pass conditions table.
- **Recommendation**: None. Gap closed.

- **Finding**: No integration test PC. Round 1 recommended PC-014 as optional. For a primarily subtractive change where the "new" behavior is the absence of a side effect, unit tests with MockExecutor are sufficient. The gc trigger in lifecycle.rs is additive but calls an existing, already-tested function (`worktree::gc`). The design's argument that "Fully testable as unit tests with MockExecutor" is defensible.
- **Evidence**: E2E Test Plan marks both boundaries as "ignored" for default state.
- **Recommendation**: Accepted without integration test. Not blocking.

### 8. Doc Plan Completeness
- **Finding**: CHANGELOG.md entry is planned (Doc Update #2). CLAUDE.md gotchas update is planned (Doc Update #1). No ADRs needed per spec (all 3 decisions marked "ADR Needed? No"). Doc levels are appropriate.
- **Evidence**: Doc Update Plan table, design lines 64-68.
- **Recommendation**: None.

## Suggested PCs

None required. All round 1 CONDITIONAL items have been addressed:
1. PC-013 (cargo fmt) -- added
2. project_dir acquisition -- specified in FC-3
3. FC-2 test removal -- explicitly mentioned

## Verdict Rationale

All 8 dimensions score above 70. The round 1 CONDITIONAL items (missing cargo fmt PC, unspecified project_dir resolution, implicit test removal) have been addressed in the updated design. The average score of 86/100 reflects a clean, focused solution with minimal blast radius, correct architecture, and full AC coverage.

Remaining minor observations (not blocking):
- Rollback step 2 description does not explicitly mention restoring cleanup_worktree tests, but a git revert covers this operationally
- PC-006 should be verified after FC-1 implementation to confirm no side effects on the rebase conflict test
- No integration test, which is acceptable for a subtractive change

The solution is ready for `/implement`.
