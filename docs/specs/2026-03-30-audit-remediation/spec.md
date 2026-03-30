# Spec: Audit 2026-03-29 Full Remediation

## Problem Statement

The March 2026 full audit revealed three compound risks: (1) the live production path for typed hook merging has zero test coverage while sitting in the project's #1 hotspot file that exceeds the 800-line limit, (2) the install pipeline silently succeeds even when hooks fail to install due to three distinct error-swallowing patterns, and (3) convention violations (.unwrap() in production, direct env access, stale docs) accumulate across the codebase. Left unaddressed, these create a fragile install surface where the core ECC value proposition (hooks, phase gates, TDD enforcement) can be silently absent.

## Research Summary

- Rust module decomposition: use `mod.rs` + sibling files in a directory; re-export public items from `mod.rs` to maintain API compatibility
- Error propagation refactoring: change `()` return types to `Result<(), Error>` incrementally; callers that used `.ok()` or `let _ =` must handle the new Result
- Install rollback patterns: accumulator pattern (collect errors, report at end) preferred over fail-fast for multi-step installers
- LazyLock regex promotion: move `Regex::new().unwrap()` to `static RE: LazyLock<Regex> = LazyLock::new(|| ...)` at file scope
- thiserror migration from anyhow: define error enum, add `#[from]` variants, change return types
- UTF-8 safe truncation: walk backward with `is_char_boundary()` — standard Rust pattern

## Decisions Made

| # | Decision | Rationale | ADR? |
|---|----------|-----------|------|
| 1 | Tests first, then structural changes | User preference — TDD safety net before refactoring | No |
| 2 | Submodule directory pattern for decomposition | Established codebase convention, approved in interview | No |
| 3 | Group merge.rs split + typed tests + legacy dedup | Tightly coupled — splitting merge.rs changes test file paths | No |
| 4 | All other smells ship independently | Each is behavior-preserving and atomic | No |
| 5 | No new ADRs | All changes follow existing documented conventions | No |
| 6 | Behavioral changes accepted | Install errors surfacing, warn→error, unwrap→propagation | No |
| 7 | Accumulator pattern for install errors | Collect all errors, report at end — not fail-fast | No |

## User Stories

### US-001: Typed Hook Merge Test Coverage + merge.rs Decomposition

**As a** developer, **I want** the typed hook merge path to have full test coverage and config/merge.rs to be decomposed into focused submodules, **so that** regressions in hook installation are caught automatically and the #1 hotspot stays maintainable.

#### Acceptance Criteria

- AC-001.1: Given `merge_hooks_typed` with empty source and destination, when merging, then result is empty
- AC-001.2: Given `merge_hooks_typed` with new ECC hooks, when merging into empty destination, then all hooks are added
- AC-001.3: Given `merge_hooks_typed` with duplicate hooks, when merging, then duplicates are deduplicated
- AC-001.4: Given `merge_hooks_typed` with legacy hooks in destination, when merging, then legacy hooks are removed
- AC-001.5: Given `remove_legacy_hooks_typed` with known legacy patterns, when called, then all legacy patterns are detected
- AC-001.6: Given `config/merge.rs` at 868 lines, when decomposed, then each submodule is under 400 lines
- AC-001.7: Given legacy detection logic duplicated 3x, when refactored, then a single `is_legacy_command(cmd: &str) -> bool` serves all callers
- AC-001.8: Given `merge_hooks_pure` returns 4-tuple, when refactored, then it returns `MergeHooksResult` struct
- AC-001.9: All existing tests pass after decomposition (zero behavioral change)

#### Dependencies
- None (first unit of work)

### US-002: Install Pipeline Error Propagation

**As a** user, **I want** install failures to be visible and reported, **so that** I know when hooks, agents, or commands failed to install.

#### Acceptance Criteria

- AC-002.1: Given `pre_scan_directory` fails to read source dir, when install runs, then `InstallSummary.success` is `false` and error is in `errors` list
- AC-002.2: Given file read fails during content comparison, when comparing, then the file is added to `MergeReport.errors` (not classified as "changed")
- AC-002.3: Given `step_hooks_and_settings` returns `Result`, when hook merge fails, then error propagates to `InstallSummary`
- AC-002.4: Given install completes with errors, when summary is displayed, then user sees which steps failed
- AC-002.5: Unit tests exist for `step_clean`, `step_merge_artifacts`, and `step_hooks_and_settings` error paths using in-memory FS doubles
- AC-002.6: All existing install integration tests still pass
- AC-002.7: Given `step_hooks_and_settings` return type changes to `Result`, when `ecc-cli/src/commands/install.rs` calls it, then the CLI handles the Result and surfaces errors to the user

#### Dependencies
- None

### US-003: Convention Violations Cleanup

**As a** developer, **I want** convention violations fixed (unwrap, env access, anyhow leak, boolean params, glob re-exports), **so that** the codebase follows its own documented rules consistently.

#### Acceptance Criteria

- AC-003.1: Given 6 `Regex::new().unwrap()` in `language.rs`, when refactored, then all use `LazyLock<Regex>` statics
- AC-003.2: Given `.unwrap()` in `flock_lock.rs:149` and `toggle.rs:126`, when refactored, then errors are propagated with `?`
- AC-003.3: Given 3 direct `std::env` reads in ecc-app, when refactored, then all route through the `Environment` trait (`ecc-ports/src/env.rs`)
- AC-003.4: Given `worktree.gc()` returns `anyhow::Error`, when refactored, then it returns `WorktreeError` with thiserror
- AC-003.5: Given `insert_after_activity` and `insert_after_daily_heading` are identical, when refactored, then a single `insert_after_heading` generic serves both
- AC-003.6: Given 10+ boolean params in public APIs, when refactored, then `ColorMode` and `DryRun` enums replace bools
- AC-003.7: Given 13 glob re-exports in module files, when refactored, then explicit named re-exports are used
- AC-003.8: Given `sanitize_osascript` truncation at byte boundary, when fixed, then `is_char_boundary()` walk-back is used
- AC-003.9: All existing tests pass after each change

#### Dependencies
- None

### US-004: Documentation Accuracy

**As a** user/developer, **I want** documentation counts and references to be accurate, **so that** CLAUDE.md and ARCHITECTURE.md are trustworthy.

#### Acceptance Criteria

- AC-004.1: Given CLAUDE.md test count is wrong (1698 and 1562), when fixed, then both locations show actual count
- AC-004.2: Given CLAUDE.md crate count contradicts itself (9 vs 8), when fixed, then consistent "9 crates"
- AC-004.3: Given ARCHITECTURE.md missing ecc-flock and wrong test count, when regenerated, then accurate
- AC-004.4: Given duplicate ADR numbers (three 0030s, two 0031s), when fixed, then all ADR numbers are unique
- AC-004.5: Given `ecc sources` command undocumented, when added to CLAUDE.md, then discoverable
- AC-004.6: Given MODULE-SUMMARIES subcommand count wrong (17 vs 20), when fixed, then accurate

#### Dependencies
- None

### US-005: Observability Improvements

**As a** developer debugging install issues, **I want** `--verbose` to produce useful diagnostic output and error levels to be correct, **so that** I can diagnose failures.

#### Acceptance Criteria

- AC-005.1: Given write failures logged at WARN, when reclassified, then write/serialize/manifest failures are `log::error!`
- AC-005.2: Given 0 `log::info!` calls, when added, then install step boundaries emit info-level logs
- AC-005.3: Given `ecc install --verbose` produces no extra output, when info logs added, then verbose shows at minimum one log line per install step (clean, detect, merge-artifacts, hooks-settings, manifest)
- AC-005.4: Given `ecc-workflow` debug requires undocumented RUST_LOG, when documented, then `--help` mentions RUST_LOG

#### Dependencies
- Depends on US-002 (error propagation changes affect log sites)

## Affected Modules

| Module | Layer | Change Type |
|--------|-------|-------------|
| `ecc-domain/src/config/merge.rs` | Domain | Split into submodule directory, extract shared helper |
| `ecc-app/src/install/global/steps.rs` | App | Change return types to Result, add unit tests |
| `ecc-app/src/config/merge.rs` | App | Update imports after domain split |
| `ecc-app/src/detection/language.rs` | App | Promote Regex to LazyLock |
| `ecc-app/src/version.rs` | App | Route env through `Environment` trait |
| `ecc-app/src/validate/statusline.rs` | App | Route env through `Environment` trait |
| `ecc-app/src/worktree.rs` | App | Add WorktreeError, replace anyhow |
| `ecc-app/src/hook/handlers/tier2_notify.rs` | App | Fix UTF-8 boundary |
| `ecc-app/src/hook/handlers/mod.rs` | App | Replace glob re-exports |
| `ecc-workflow/src/commands/memory_write.rs` | Workflow | Extract insert_after_heading generic |
| `ecc-domain/src/ansi.rs` | Domain | Add ColorMode enum |
| `ecc-cli/src/commands/install.rs` | CLI | Handle new Result from steps |
| `CLAUDE.md` | Docs | Fix counts, add sources command |
| `docs/ARCHITECTURE.md` | Docs | Regenerate |
| `docs/MODULE-SUMMARIES.md` | Docs | Regenerate |
| `docs/adr/` | Docs | Renumber duplicates |

## Constraints

- All refactoring steps must be behavior-preserving (except install error surfacing, which is intentional)
- All 1742+ tests must stay green after every commit
- Tests written BEFORE structural changes (TDD approach)
- `cargo clippy -- -D warnings` must pass after every commit
- No new public API surface unless replacing an existing one
- merge.rs decomposition ships as a single atomic commit; revert if compilation fails
- Install error propagation uses accumulator pattern (collect all errors, report at end) — not fail-fast

## Non-Requirements

- No new features — this is purely remediation
- No CI/CD changes
- No dependency version bumps (serde_yml migration is separate BL-099)
- No ecc-workflow port abstraction (acknowledged architectural debt, separate scope)
- No config/validate.rs decomposition — at 351 lines it is well within the 800-line limit; defer until it actually approaches the threshold

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| `Environment` trait | New callers in ecc-app | Existing tests use InMemoryEnv — no E2E impact |
| Install pipeline | Error propagation | Install integration tests need update for new error paths |
| `FsPort` | No change | No E2E impact |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| Test count fix | CLAUDE.md | CLAUDE.md:12,88 | Update to actual count |
| Crate count fix | CLAUDE.md | CLAUDE.md:93 | Change 8→9 |
| Missing command | CLAUDE.md | CLAUDE.md:27-50 | Add `ecc sources` |
| Regenerate | docs/ | ARCHITECTURE.md, MODULE-SUMMARIES.md | Regenerate |
| ADR renumber | docs/adr/ | 0030, 0031 duplicates | Assign 0032, 0033 |

## Open Questions

None — all resolved during grill-me interview.
