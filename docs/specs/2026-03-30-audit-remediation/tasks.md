# Tasks: Audit 2026-03-29 Full Remediation

## Pass Conditions

| PC | Description | Status | Trail |
|----|-------------|--------|-------|
| PC-001 | merge_hooks_typed empty both | done | green@2026-03-30T00:15:00Z |
| PC-002 | merge_hooks_typed adds new to empty | done | green@2026-03-30T00:15:00Z |
| PC-003 | merge_hooks_typed deduplicates | done | green@2026-03-30T00:15:00Z |
| PC-004 | merge_hooks_typed removes legacy | done | green@2026-03-30T00:15:00Z |
| PC-005 | remove_legacy_hooks_typed patterns | done | green@2026-03-30T00:15:00Z |
| PC-006 | is_legacy_command shared helper | done | green@2026-03-30T00:45:00Z |
| PC-007 | MergeHooksResult struct | done | green@2026-03-30T00:45:00Z |
| PC-008 | Submodules under 400 lines | done | green@2026-03-30T01:30:00Z |
| PC-009 | Existing tests pass after split | done | green@2026-03-30T01:30:00Z |
| PC-010 | pre_scan_directory error reporting | done | green@2026-03-30T02:30:00Z |
| PC-011 | File read failure error reporting | done | green@2026-03-30T02:30:00Z |
| PC-012 | step_hooks_and_settings Result | done | green@2026-03-30T02:30:00Z |
| PC-013 | Accumulator error pattern | done | green@2026-03-30T02:30:00Z |
| PC-014 | Step error paths with in-memory FS | done | green@2026-03-30T02:30:00Z |
| PC-015 | Existing install tests pass | done | green@2026-03-30T02:30:00Z |
| PC-016 | LazyLock in language.rs | done | green@2026-03-30T03:00:00Z |
| PC-017 | No Regex::new().unwrap() grep | done | green@2026-03-30T03:00:00Z |
| PC-018 | LazyLock in ansi.rs | done | green@2026-03-30T03:00:00Z |
| PC-019 | version() via Environment trait | done | green@2026-03-30T03:30:00Z |
| PC-020 | statusline via Environment trait | done | green@2026-03-30T04:00:00Z |
| PC-021 | WorktreeError replaces anyhow | done | green@2026-03-30T05:00:00Z |
| PC-022 | insert_after_heading generic | done | green@2026-03-30T03:00:00Z |
| PC-023 | ColorMode enum | done | green@2026-03-30T03:30:00Z |
| PC-024 | No glob re-exports | done | green@2026-03-30T03:30:00Z |
| PC-025 | UTF-8 safe osascript truncation | done | green@2026-03-30T03:00:00Z |
| PC-026 | UTF-8 safe powershell truncation | done | green@2026-03-30T03:00:00Z |
| PC-027 | Info logging at step boundaries | done | green@2026-03-30T05:30:00Z |
| PC-028 | Error level for write failures | done | green@2026-03-30T05:30:00Z |
| PC-029 | Zero clippy warnings | done | green@2026-03-30T06:00:00Z |
| PC-030 | Full test suite passes | done | green@2026-03-30T06:00:00Z |
| PC-031 | CLAUDE.md test count correct | done | green@2026-03-30T06:00:00Z |
| PC-032 | CLAUDE.md says 9 crates | done | green@2026-03-30T06:00:00Z |
| PC-033 | CLAUDE.md has ecc sources | done | green@2026-03-30T06:00:00Z |
| PC-034 | No duplicate ADR 0030 | n/a | worktree-divergence — ADR 0030+ exist only on main |
| PC-035 | MODULE-SUMMARIES count fixed | done | green@2026-03-30T06:00:00Z |
| PC-036 | ARCHITECTURE.md has ecc-flock | done | green@2026-03-30T06:00:00Z |
| PC-037 | RUST_LOG documented | done | already-present@2026-03-30T06:00:00Z |
| PC-038 | flock_lock.rs unwrap removed | done | prod-clean@2026-03-30T03:30:00Z |
| PC-039 | toggle.rs unwrap removed | done | prod-clean@2026-03-30T03:30:00Z |
| PC-040 | DryRun enum | deferred | low-priority — ColorMode covers AC-003.6 primary |

## Post-TDD

| Task | Status | Trail |
|------|--------|-------|
| E2E tests | done | no-e2e-required@2026-03-30T06:30:00Z |
| Code review | in-progress | dispatched@2026-03-30T06:30:00Z |
| Doc updates | done | green@2026-03-30T06:00:00Z |
| Supplemental docs | pending | |
| Write implement-done.md | pending | |
