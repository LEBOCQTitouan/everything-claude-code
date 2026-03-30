# Design: Audit 2026-03-29 Full Remediation

## Overview

This design decomposes the spec into 8 independently-deliverable phases, ordered by dependency. Each phase is a vertical slice testable in isolation. The grouping follows spec Decision #3 (merge.rs split + typed tests + legacy dedup as one atomic unit) and Decision #4 (all other smells ship independently).

## File Changes Table

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `crates/ecc-domain/src/config/merge.rs` | modify | Add tests for `merge_hooks_typed`, `remove_legacy_hooks_typed`, `is_legacy_ecc_hook_typed` (the typed path has zero coverage) | AC-001.1..AC-001.5 |
| 2 | `crates/ecc-domain/src/config/merge.rs` | modify | Extract `is_legacy_command(cmd: &str) -> bool` to deduplicate 2x detection logic (Value vs typed) | AC-001.7 |
| 3 | `crates/ecc-domain/src/config/merge.rs` | modify | Replace 4-tuple return of `merge_hooks_pure` and `merge_hooks_typed` with `MergeHooksResult` struct | AC-001.8 |
| 4 | `crates/ecc-domain/src/config/merge/` | create (directory) | Decompose 868-line merge.rs into submodules: `mod.rs`, `types.rs`, `legacy.rs`, `merge_value.rs`, `merge_typed.rs`, `format.rs` | AC-001.6 |
| 5 | `crates/ecc-domain/src/config/mod.rs` | modify | Change `pub mod merge;` (file) to `pub mod merge;` (directory) — no API change | AC-001.6 |
| 6 | `crates/ecc-app/src/config/merge.rs` | modify | Update imports if needed after domain split (verify `use ecc_domain::config::merge::*` still resolves) | AC-001.9 |
| 7 | `crates/ecc-app/src/install/global/steps.rs` | modify | Change `step_hooks_and_settings` return type from `()` to `Result<(), Vec<String>>`. Change `step_clean` to return errors. `step_merge_artifacts` error accumulation. | AC-002.1..AC-002.5 |
| 8 | `crates/ecc-app/src/install/global/mod.rs` | modify | Handle `Result` from `step_hooks_and_settings`, accumulate errors into `combined.errors` | AC-002.3, AC-002.7 |
| 9 | `crates/ecc-app/src/config/merge.rs` | modify | `pre_scan_directory`: return errors in `MergeReport.errors` instead of silently returning empty; propagate file read failures | AC-002.1, AC-002.2 |
| 10 | `crates/ecc-cli/src/commands/install.rs` | modify | Already handles `summary.success == false` — verify it works with new error paths | AC-002.7 |
| 11 | `crates/ecc-app/src/detection/language.rs` | modify | Promote 6 `Regex::new().unwrap()` to `LazyLock<Regex>` statics | AC-003.1 |
| 12 | `crates/ecc-domain/src/ansi.rs` | modify | Promote `Regex::new().expect()` in `strip_ansi` to `LazyLock<Regex>` static | AC-003.1 |
| 13 | `crates/ecc-app/src/version.rs` | modify | Accept `&dyn Environment` instead of calling `std::env::var` directly | AC-003.3 |
| 14 | `crates/ecc-app/src/validate/statusline.rs` | modify | Accept `&dyn Environment` param, replace 2x `std::env::var("HOME")` with `env.var("HOME")` | AC-003.3 |
| 15 | `crates/ecc-app/src/worktree.rs` | modify | Define `WorktreeError` enum with `thiserror`, replace `anyhow::Error` return | AC-003.4 |
| 16 | `crates/ecc-workflow/src/commands/memory_write.rs` | modify | Extract `insert_after_heading(content, heading, entry)` generic, replace both `insert_after_activity` and `insert_after_daily_heading` | AC-003.5 |
| 17 | `crates/ecc-domain/src/ansi.rs` | modify | Add `ColorMode` enum (`Enabled`, `Disabled`) replacing `bool` param | AC-003.6 |
| 18 | `crates/ecc-app/src/hook/handlers/mod.rs` | modify | Replace 4 glob re-exports with explicit named re-exports | AC-003.7 |
| 19 | `crates/ecc-app/src/hook/handlers/tier1_simple/mod.rs` | modify | Replace 5 glob re-exports with explicit named re-exports | AC-003.7 |
| 20 | `crates/ecc-app/src/hook/handlers/tier2_notify.rs` | modify | Fix UTF-8 safe truncation with `is_char_boundary()` walk-back in `sanitize_osascript` and `sanitize_powershell` | AC-003.8 |
| 21 | `crates/ecc-app/src/install/global/steps.rs` | modify | Change `log::warn!` to `log::error!` for write/serialize/manifest failures | AC-005.1 |
| 22 | `crates/ecc-app/src/install/global/steps.rs` | modify | Add `log::info!` at step boundaries (clean, detect, merge, hooks, manifest) | AC-005.2, AC-005.3 |
| 23 | `CLAUDE.md` | modify | Fix test count (both locations), fix crate count (8→9), add `ecc sources` command | AC-004.1, AC-004.2, AC-004.5 |
| 24 | `docs/ARCHITECTURE.md` | modify | Regenerate to include ecc-flock, correct test count | AC-004.3 |
| 25 | `docs/MODULE-SUMMARIES.md` | modify | Fix subcommand count (17→20) | AC-004.6 |

## Pass Conditions Table

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | `merge_hooks_typed` with both empty maps returns empty map, 0 added, 0 existing, 0 legacy | AC-001.1 | `cargo test -p ecc-domain merge::tests::merge_hooks_typed_empty_both` | PASS |
| PC-002 | unit | `merge_hooks_typed` adds new hooks into empty destination | AC-001.2 | `cargo test -p ecc-domain merge::tests::merge_hooks_typed_adds_new_to_empty` | PASS |
| PC-003 | unit | `merge_hooks_typed` deduplicates identical hooks | AC-001.3 | `cargo test -p ecc-domain merge::tests::merge_hooks_typed_deduplicates` | PASS |
| PC-004 | unit | `merge_hooks_typed` removes legacy hooks from destination | AC-001.4 | `cargo test -p ecc-domain merge::tests::merge_hooks_typed_removes_legacy` | PASS |
| PC-005 | unit | `remove_legacy_hooks_typed` detects all legacy patterns (package IDs, scripts/hooks, placeholders, run-with-flags, node -e, dist/hooks, worktree delegation) | AC-001.5 | `cargo test -p ecc-domain merge::tests::remove_legacy_hooks_typed` | PASS |
| PC-006 | unit | `is_legacy_command` shared helper matches same patterns as both `is_legacy_ecc_hook` and `is_legacy_ecc_hook_typed` | AC-001.7 | `cargo test -p ecc-domain merge::tests::is_legacy_command` | PASS |
| PC-007 | unit | `merge_hooks_pure` and `merge_hooks_typed` return `MergeHooksResult` struct instead of 4-tuple | AC-001.8 | `cargo test -p ecc-domain merge::tests::merge_hooks_result_struct` | PASS |
| PC-008 | build | After merge.rs decomposition, all submodules are under 400 lines | AC-001.6 | `wc -l crates/ecc-domain/src/config/merge/*.rs \| tail -1` | total under 868 lines, each file < 400 |
| PC-009 | unit | All 868 lines of existing merge.rs tests still pass after decomposition | AC-001.9 | `cargo test -p ecc-domain config::merge` | PASS, same test count |
| PC-010 | unit | `pre_scan_directory` returns error in report when source dir unreadable | AC-002.1 | `cargo test -p ecc-app config::merge::tests::pre_scan_directory_unreadable_src_reports_error` | PASS |
| PC-011 | unit | `pre_scan_directory` adds file read failure to `MergeReport.errors` | AC-002.2 | `cargo test -p ecc-app config::merge::tests::pre_scan_file_read_failure_reports_error` | PASS |
| PC-012 | unit | `step_hooks_and_settings` returns `Result`, error propagates to `InstallSummary.errors` | AC-002.3, AC-002.7 | `cargo test -p ecc-app install::global::tests` | PASS |
| PC-013 | unit | Install completes with accumulated errors when multiple steps fail (not fail-fast) | AC-002.4 | `cargo test -p ecc-app install::global::tests::install_accumulates_errors` | PASS |
| PC-014 | unit | `step_clean`, `step_merge_artifacts`, `step_hooks_and_settings` error paths tested with in-memory FS | AC-002.5 | `cargo test -p ecc-app install::global` | PASS |
| PC-015 | integration | Existing install integration tests still pass | AC-002.6 | `cargo test -p ecc-integration-tests install` | PASS |
| PC-016 | unit | `get_python_deps`, `get_go_deps`, `get_rust_deps`, `get_elixir_deps` use `LazyLock<Regex>` statics — no `.unwrap()` in production code | AC-003.1 | `cargo test -p ecc-app detection::language` | PASS |
| PC-017 | lint | No `Regex::new(` followed by `.unwrap()` in production code of `language.rs` | AC-003.1 | `grep -c 'Regex::new.*\.unwrap()' crates/ecc-app/src/detection/language.rs` | 0 |
| PC-018 | unit | `strip_ansi` uses `LazyLock<Regex>` — no `.expect()` in production | AC-003.1 | `cargo test -p ecc-domain ansi::tests` | PASS |
| PC-019 | unit | `version()` accepts `&dyn Environment` and routes through trait | AC-003.3 | `cargo test -p ecc-app version::tests::version_dev_mode_via_trait` | PASS |
| PC-020 | unit | `validate_statusline` uses `&dyn Environment` for HOME lookup | AC-003.3 | `cargo test -p ecc-app validate::statusline::tests` | PASS |
| PC-021 | unit | `worktree::gc` returns `WorktreeError` instead of `anyhow::Error` | AC-003.4 | `cargo test -p ecc-app worktree::tests` | PASS |
| PC-022 | unit | `insert_after_heading` generic works for both `## Activity` and `## Daily` headings | AC-003.5 | `cargo test -p ecc-workflow memory_write::tests::insert_after_heading` | PASS |
| PC-023 | unit | `ColorMode` enum works with ansi functions; `bold("x", ColorMode::Enabled)` produces ANSI | AC-003.6 | `cargo test -p ecc-domain ansi::tests::color_mode_enum` | PASS |
| PC-024 | build | `handlers/mod.rs` uses explicit named re-exports, no `pub use *` | AC-003.7 | `grep -c 'pub use.*\*;' crates/ecc-app/src/hook/handlers/mod.rs crates/ecc-app/src/hook/handlers/tier1_simple/mod.rs` | 0 |
| PC-025 | unit | `sanitize_osascript` and `sanitize_powershell` use `is_char_boundary()` walk-back for multi-byte truncation | AC-003.8 | `cargo test -p ecc-app hook::handlers::tier2_notify::tests::sanitize_osascript_multibyte_truncation` | PASS |
| PC-026 | unit | `sanitize_powershell` multi-byte truncation safe | AC-003.8 | `cargo test -p ecc-app hook::handlers::tier2_notify::tests::sanitize_powershell_multibyte_truncation` | PASS |
| PC-027 | unit | Install step boundaries emit `log::info!` (at least one per step) | AC-005.2, AC-005.3 | `grep -c 'log::info!' crates/ecc-app/src/install/global/steps.rs` | >= 5 |
| PC-028 | lint | Write/serialize/manifest failures use `log::error!` not `log::warn!` | AC-005.1 | `grep -c 'log::warn.*manifest\|log::warn.*write\|log::warn.*serialize' crates/ecc-app/src/install/global/steps.rs` | 0 |
| PC-029 | build | Full workspace builds with zero clippy warnings | AC-001.9, AC-003.9 | `cargo clippy --workspace -- -D warnings` | exit 0 |
| PC-030 | build | Full test suite passes | AC-001.9, AC-002.6, AC-003.9 | `cargo test --workspace` | exit 0 |
| PC-031 | lint | CLAUDE.md test count matches actual `cargo test` output | AC-004.1 | `cargo test --workspace 2>&1 \| grep 'test result' \| head -1` | count matches CLAUDE.md |
| PC-032 | lint | CLAUDE.md mentions 9 crates consistently | AC-004.2 | `grep -c '9 crates\|9-crate' CLAUDE.md` | >= 1 |
| PC-033 | lint | CLAUDE.md documents `ecc sources` command | AC-004.5 | `grep -c 'ecc sources' CLAUDE.md` | >= 1 |

## Implementation Phases

### Phase 1: Typed Hook Merge Tests (RED)
**Layers**: Entity
**Files**: `crates/ecc-domain/src/config/merge.rs`
**Commit**: `test: add typed hook merge coverage (AC-001.1..AC-001.5)`

Add test cases for `merge_hooks_typed` and `remove_legacy_hooks_typed`:
- Empty source + empty destination -> empty result
- New hooks into empty destination -> all added
- Duplicate hooks -> deduplicated
- Legacy hooks in destination -> removed
- All legacy patterns detected by typed path

These are pure additions to the existing `#[cfg(test)] mod tests` block. No structural change yet.

**Pass**: PC-001, PC-002, PC-003, PC-004, PC-005, PC-030

### Phase 2: Legacy Dedup + MergeHooksResult (GREEN+REFACTOR)
**Layers**: Entity
**Files**: `crates/ecc-domain/src/config/merge.rs`
**Commit**: `refactor: extract is_legacy_command, add MergeHooksResult struct`

1. Extract `is_legacy_command(cmd: &str) -> bool` with the shared pattern-matching logic
2. Rewrite `is_legacy_ecc_hook` and `is_legacy_ecc_hook_typed` to delegate to it
3. Define `MergeHooksResult { merged, added, existing, legacy_removed }` struct
4. Change `merge_hooks_pure` and `merge_hooks_typed` to return `MergeHooksResult`
5. Update all callers (destructuring patterns)

**Pass**: PC-006, PC-007, PC-009, PC-030

### Phase 3: merge.rs Decomposition (REFACTOR)
**Layers**: Entity
**Files**: `crates/ecc-domain/src/config/merge.rs` -> `crates/ecc-domain/src/config/merge/` directory
**Commit**: `refactor: decompose merge.rs into submodule directory`

Split into:
- `mod.rs` — re-exports, `MergeReport`, `MergeHooksResult`, `FileToReview`, constants (~80 lines)
- `legacy.rs` — `is_legacy_command`, `is_legacy_ecc_hook`, `is_legacy_ecc_hook_typed`, `remove_legacy_hooks`, `remove_legacy_hooks_typed` (~120 lines)
- `merge_value.rs` — `merge_event_entries`, `merge_hooks_pure` (~60 lines)
- `merge_typed.rs` — `merge_hooks_typed` (~40 lines)
- `format.rs` — `format_merge_report`, `contents_differ`, `empty_report`, `combine_reports` (~80 lines)
- `tests.rs` — all tests (~400 lines, `#[cfg(test)]`)

Each submodule under 400 lines. `mod.rs` re-exports all public items explicitly (not glob).

**Pass**: PC-008, PC-009, PC-029, PC-030

### Phase 4: Install Error Propagation
**Layers**: UseCase, Adapter
**Files**: `crates/ecc-app/src/install/global/steps.rs`, `crates/ecc-app/src/install/global/mod.rs`, `crates/ecc-app/src/config/merge.rs`, `crates/ecc-cli/src/commands/install.rs`
**Commit 1**: `test: add install error path tests (AC-002.1..AC-002.5)`
**Commit 2**: `feat: propagate install errors with accumulator pattern`

Changes:
1. `pre_scan_directory`: return `(Vec<FileToReview>, Vec<String>, Vec<String>)` where third element is errors. When `read_dir` fails, push error message. When `read_to_string` fails for content comparison, push error to errors vec instead of defaulting to empty string.
2. `step_hooks_and_settings`: change return type to `Result<(usize, usize, usize), String>`. On hook merge error, return `Err`.
3. `install_global` in `mod.rs`: match on `step_hooks_and_settings` result; on `Err`, push to `combined.errors` (accumulator pattern, not fail-fast).
4. `step_write_manifest`: change `log::warn!` to `log::error!`.
5. Add `log::info!` at each step boundary.
6. CLI `install.rs` already handles `summary.success == false` — verify no changes needed.

**Pass**: PC-010, PC-011, PC-012, PC-013, PC-014, PC-015, PC-027, PC-028, PC-030

### Phase 5: LazyLock Regex Promotion
**Layers**: UseCase (language.rs), Entity (ansi.rs)
**Files**: `crates/ecc-app/src/detection/language.rs`, `crates/ecc-domain/src/ansi.rs`
**Commit**: `refactor: promote Regex to LazyLock statics`

Replace all `Regex::new(...).unwrap()` in production code with:
```rust
use std::sync::LazyLock;
static RE_BLOCK: LazyLock<regex::Regex> = LazyLock::new(|| regex::Regex::new(r"...").expect("valid regex"));
```

In `language.rs`: 6 regex instances across `get_python_deps`, `get_go_deps`, `get_rust_deps`, `get_elixir_deps`.
In `ansi.rs`: 1 regex in `strip_ansi`.

**Pass**: PC-016, PC-017, PC-018, PC-029, PC-030

### Phase 6: Convention Violations Cleanup
**Layers**: Entity (ansi.rs ColorMode), UseCase (version.rs, statusline.rs, worktree.rs, memory_write.rs), Adapter (handlers/mod.rs)
**Files**: Multiple (see File Changes #13-20)

Sub-phases (each independently committable):

**6a: Environment trait routing** (`refactor: route env access through Environment trait`)
- `version.rs`: change `version()` to `version(env: &dyn Environment)`, check `env.var("ECC_DEV_MODE")`
- `validate/statusline.rs`: add `env: &dyn Environment` param, replace `std::env::var("HOME")` calls
- Update callers in CLI

**6b: WorktreeError** (`refactor: replace anyhow with WorktreeError in worktree::gc`)
- Define `WorktreeError` enum in `worktree.rs` with `thiserror`
- Replace `anyhow::Error` return type

**6c: insert_after_heading generic** (`refactor: extract insert_after_heading generic`)
- In `memory_write.rs`, create `insert_after_heading(content, heading, entry)`
- Replace `insert_after_activity` and `insert_after_daily_heading`

**6d: ColorMode enum** (`refactor: add ColorMode enum replacing bool params`)
- Add `ColorMode` enum to `ansi.rs`
- Update `bold`, `dim`, `red`, `green`, `yellow`, `cyan` signatures
- Note: this touches many callers — assess blast radius, consider `From<bool>` impl for migration

**6e: Explicit re-exports** (`refactor: replace glob re-exports with named exports`)
- `handlers/mod.rs`: replace 4 `pub use tier*::*` with explicit names
- `tier1_simple/mod.rs`: replace 5 `pub use *::*` with explicit names

**6f: UTF-8 safe truncation** (`fix: use is_char_boundary for safe truncation`)
- `sanitize_osascript` and `sanitize_powershell`: walk back with `is_char_boundary()`

**Pass**: PC-019, PC-020, PC-021, PC-022, PC-023, PC-024, PC-025, PC-026, PC-029, PC-030

### Phase 7: Observability
**Layers**: UseCase
**Files**: `crates/ecc-app/src/install/global/steps.rs`
**Commit**: `feat: add info-level logging at install step boundaries`

Already partially handled in Phase 4. This phase adds any remaining `log::info!` calls and verifies the `--verbose` path works end-to-end.

**Pass**: PC-027, PC-028, PC-030

### Phase 8: Documentation
**Layers**: (none — docs only)
**Files**: `CLAUDE.md`, `docs/ARCHITECTURE.md`, `docs/MODULE-SUMMARIES.md`
**Commit**: `docs: fix test/crate counts, add ecc sources command`

1. Run `cargo test --workspace 2>&1 | grep 'test result'` to get actual count
2. Update CLAUDE.md test count in both locations
3. Fix crate count to "9 crates" consistently
4. Add `ecc sources` to CLI commands table
5. Regenerate ARCHITECTURE.md and MODULE-SUMMARIES.md

**Pass**: PC-031, PC-032, PC-033, PC-030

## E2E Assessment

- **Touches user-facing flows?** Yes — `ecc install` error reporting changes
- **Crosses 3+ modules end-to-end?** Yes — domain (merge) -> app (install steps) -> CLI (install command)
- **New E2E tests needed?** No — existing install integration tests cover the flow. The behavioral change (errors now surface instead of being swallowed) is verified through unit tests with in-memory FS doubles (PC-012, PC-013, PC-014). The existing integration tests (PC-015) confirm no regression.

## Risks and Mitigations

| Risk | Severity | Mitigation |
|------|----------|------------|
| merge.rs decomposition breaks import paths | High | Phase 3 re-exports all public items from `mod.rs`. Run `cargo test --workspace` immediately. Single atomic commit — revert if anything breaks. |
| `ColorMode` enum changes signature of 6 ansi functions, touching many callers | Medium | Implement `From<bool> for ColorMode` to allow gradual migration. Or defer to a separate PR if blast radius is too large. |
| `version()` signature change propagates to CLI | Low | Only 2 callers. Add `&env` parameter, existing tests use `MockEnvironment`. |
| `step_hooks_and_settings` return type change | Medium | Accumulator pattern means callers push errors but continue — no early exit. Existing tests verify the happy path still works. |
| `pre_scan_directory` return type change breaks callers | Medium | Only 1 caller in `merge/mod.rs`. Update destructuring pattern. |
| ADR renumbering (AC-004.4) may be stale | Low | No duplicate ADRs exist on disk (max is 0029). Skip this AC if confirmed stale. |

## Dependency Graph

```
Phase 1 (typed tests)
    |
    v
Phase 2 (legacy dedup + struct)
    |
    v
Phase 3 (decomposition)

Phase 4 (install errors) -- independent of 1-3
    |
    v
Phase 7 (observability) -- depends on 4

Phase 5 (LazyLock) -- fully independent
Phase 6 (conventions) -- fully independent
Phase 8 (docs) -- last, after test count stabilizes
```

Phases 1-3 must be sequential (grouped per Decision #3).
Phases 4, 5, 6 are independent of each other and of 1-3.
Phase 7 depends on Phase 4.
Phase 8 must be last (test count depends on all other phases).

## Additional Pass Conditions (gap-fill)

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-034 | lint | No duplicate ADR 0030 files | AC-004.4 | `ls docs/adr/0030-*.md \| wc -l` | 1 |
| PC-035 | lint | MODULE-SUMMARIES subcommand count fixed | AC-004.6 | `grep -c '20' docs/MODULE-SUMMARIES.md` | >= 1 |
| PC-036 | lint | ARCHITECTURE.md mentions ecc-flock | AC-004.3 | `grep -c 'ecc-flock' docs/ARCHITECTURE.md` | >= 1 |
| PC-037 | lint | RUST_LOG documented in ecc-workflow help | AC-005.4 | `grep -c 'RUST_LOG' crates/ecc-workflow/src/main.rs` | >= 1 |
| PC-038 | unit | flock_lock.rs unwrap replaced with error propagation | AC-003.2 | `grep -c '\.unwrap()' crates/ecc-infra/src/flock_lock.rs` | 0 (in production code) |
| PC-039 | unit | toggle.rs unwrap replaced with error propagation | AC-003.2 | `grep -c '\.unwrap()' crates/ecc-app/src/dev/toggle.rs` | 0 (in non-test code) |
| PC-040 | lint | DryRun enum replaces boolean dry_run params | AC-003.6 | `cargo test -p ecc-domain config::clean::tests` | PASS |

## Rollback Plan

Reverse dependency order for undoing changes if implementation fails at any phase:

1. **Phase 8 (docs)**: `git revert` the doc commit — no code impact
2. **Phase 7 (observability)**: revert log level and info changes — no API impact
3. **Phase 6f (UTF-8)**: revert truncation fix — isolated to 2 functions
4. **Phase 6e (re-exports)**: revert to glob re-exports — compilation still works
5. **Phase 6d (ColorMode)**: revert enum, restore `bool` params — highest blast radius, revert via `git revert`
6. **Phase 6c (insert_after_heading)**: revert to duplicated functions
7. **Phase 6b (WorktreeError)**: revert to `anyhow::Error` return
8. **Phase 6a (Environment routing)**: revert to `std::env::var` — isolated to 3 call sites
9. **Phase 5 (LazyLock)**: revert to `Regex::new().unwrap()` — no API change
10. **Phase 4 (install errors)**: revert step return types to `()` — most complex rollback, touches 4 files
11. **Phase 3 (decomposition)**: revert merge/ directory to single merge.rs — `git revert` the atomic commit
12. **Phase 2 (dedup + struct)**: revert shared helper and struct — restore 4-tuple
13. **Phase 1 (typed tests)**: revert test additions — no production impact

Each phase is an independent git commit. Rollback = `git revert <commit>` for any phase.

## SOLID Assessment

**PASS** — 4 prescriptions (all LOW/MEDIUM, none blocking):
1. MEDIUM: merge_hooks_pure/typed parallel implementations — consider unifying via trait abstraction (second-pass)
2. LOW: step_write_manifest has `#[allow(clippy::too_many_arguments)]` — consider step context struct
3. LOW: version() DIP fix is correct — ensure `&dyn Environment` parameter
4. LOW: LazyLock statics should be private or `pub(crate)`

## Robert's Oath Check

**CLEAN** — 0 oath warnings. Rework ratio 0.08. Tests-first, atomic phases, Boy Scout Rule at scale.

## Security Notes

**CLEAR** — Net security improvement. No new attack surface. UTF-8 fix prevents panic. Environment trait routing improves testability. Error surfacing is a security positive.

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | CLAUDE.md | Project | Update | Fix test count (both locations), crate count (8→9), add `ecc sources` | AC-004.1, AC-004.2, AC-004.5 |
| 2 | docs/ARCHITECTURE.md | System | Regenerate | Include ecc-flock, correct test count | AC-004.3 |
| 3 | docs/MODULE-SUMMARIES.md | System | Update | Fix subcommand count 17→20 | AC-004.6 |
| 4 | docs/adr/ | Decision | Renumber | Deduplicate 0030 and 0031 entries | AC-004.4 |
| 5 | CHANGELOG.md | Project | Append | Add v4.3.0 audit remediation entry | — |
