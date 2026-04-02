# Solution: Deterministic Hook System Redesign

## Spec Reference
Concern: refactor, Feature: deterministic hook system redesign

## File Changes (dependency order)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `crates/ecc-domain/src/workflow/path.rs` | Create | Pure lexical path normalization `normalize_path(path: &str) -> String` -- strips `.` and `..` without filesystem access | US-002, AC-002.1 |
| 2 | `crates/ecc-domain/src/workflow/staleness.rs` | Create | Pure staleness detection `is_stale(started_at, now, threshold_secs) -> bool` with injectable timestamps | US-006, AC-006.1, AC-006.2 |
| 3 | `crates/ecc-domain/src/workflow/phase_verify.rs` | Create | Pure `verify_phase(current, expected) -> Result<(), PhaseError>` with hint messages | US-005, AC-005.5 |
| 4 | `crates/ecc-domain/src/workflow/state.rs` | Modify | Add `#[serde(default = "default_version")] pub version: u32` field to WorkflowState | US-003, AC-003.5 |
| 5 | `crates/ecc-domain/src/workflow/mod.rs` | Modify | Add `pub mod path; pub mod staleness; pub mod phase_verify;` | US-002, US-005, US-006 |
| 6 | `crates/ecc-ports/src/git.rs` | Create | `trait GitInfo { fn git_dir(&self, working_dir: &Path) -> Result<PathBuf, GitError>; fn is_inside_worktree(&self, working_dir: &Path) -> bool; }` | US-001, AC-001.1 |
| 7 | `crates/ecc-ports/src/clock.rs` | Create | `trait Clock { fn now_iso8601(&self) -> String; fn now_epoch_secs(&self) -> u64; }` | US-006, AC-006.2 |
| 8 | `crates/ecc-ports/src/lib.rs` | Modify | Add `pub mod git; pub mod clock;` | US-001, US-006 |
| 9 | `crates/ecc-infra/src/os_git.rs` | Create | `OsGitInfo` implementing GitInfo via `git rev-parse --git-dir` | US-001, AC-001.1 |
| 10 | `crates/ecc-infra/src/system_clock.rs` | Create | `SystemClock` implementing Clock via `SystemTime` | US-006, AC-006.2 |
| 11 | `crates/ecc-infra/src/lib.rs` | Modify | Add `pub mod os_git; pub mod system_clock;` | US-001, US-006 |
| 12 | `crates/ecc-test-support/src/mock_git.rs` | Create | Configurable `MockGitInfo` test double | US-001, AC-001.5, AC-001.7 |
| 13 | `crates/ecc-test-support/src/mock_clock.rs` | Create | Fixed/advancing `MockClock` test double | US-006, AC-006.2 |
| 14 | `crates/ecc-test-support/src/lib.rs` | Modify | Add mock re-exports | US-001, US-006 |
| 15 | `crates/ecc-app/src/workflow/mod.rs` | Create | Module root: `pub mod state_resolver; pub mod recover;` | US-001, US-006 |
| 16 | `crates/ecc-app/src/workflow/state_resolver.rs` | Create | `resolve_state_dir(env, git, fs) -> (PathBuf, Vec<Warning>)` with fallback chain | US-001, AC-001.1-7 |
| 17 | `crates/ecc-app/src/workflow/recover.rs` | Create | `detect_staleness()` and `recover()` with mock-friendly ports | US-006, AC-006.1, AC-006.3 |
| 18 | `crates/ecc-app/src/lib.rs` | Modify | Add `pub mod workflow;` | US-001, US-003, US-006 |
| 19 | `crates/ecc-workflow/src/commands/phase_gate.rs` | Modify | Call `normalize_path()` on file_path before `is_allowed_path()` | US-002, AC-002.1-3 |
| 20 | `crates/ecc-workflow/src/io.rs` | Modify | Add `resolve_state_dir_legacy()` using git-dir resolution with fallback | US-001, AC-001.1-7 |
| 21 | `crates/ecc-workflow/src/commands/status.rs` | Modify | Append " (STALE)" when threshold exceeded | US-006, AC-006.4 |
| 22 | `crates/ecc-workflow/src/commands/mod.rs` | Modify | Add `pub mod recover;` | US-006 |
| 23 | `crates/ecc-workflow/src/commands/recover.rs` | Create | `ecc-workflow recover` subcommand | US-006, AC-006.3 |
| 24 | `crates/ecc-workflow/src/main.rs` | Modify | Add Recover variant; later convert to thin wrapper | US-003, US-006 |
| 25 | `crates/ecc-cli/src/commands/workflow.rs` | Create | `ecc workflow` subcommand group mirroring all 22+1 commands | US-003, AC-003.1-4 |
| 26 | `crates/ecc-cli/src/commands/mod.rs` | Modify | Add `pub mod workflow;` | US-003 |
| 27 | `crates/ecc-cli/src/main.rs` | Modify | Add Workflow variant to Command enum | US-003, AC-003.1 |
| 28 | `crates/ecc-cli/Cargo.toml` | Modify | Add ecc-flock dependency | US-003 |
| 29 | `crates/ecc-workflow/src/main.rs` | Modify | Convert to thin wrapper (exec `ecc workflow`) with fallback | US-003, AC-003.2 |
| 30 | `hooks/hooks.json` | Modify | Replace `ecc-hook` with `ecc hook` in all command strings | US-004, AC-004.3-4 |
| 31 | `crates/ecc-app/src/install/hooks_migration.rs` | Create | `migrate_hooks_json()` -- idempotent, preserves custom hooks | US-004, AC-004.3-5 |
| 32 | `crates/ecc-integration-tests/tests/characterization_session_hooks.rs` | Create | Characterization tests for session hooks | US-008, AC-008.1 |
| 33 | `crates/ecc-integration-tests/tests/characterization_typed_merge.rs` | Create | Characterization tests for typed hook merge | US-008, AC-008.2 |
| 34 | `crates/ecc-integration-tests/tests/characterization_workflow_lifecycle.rs` | Create | Full lifecycle E2E test | US-008, AC-008.3 |
| 35 | `crates/ecc-integration-tests/tests/characterization_worktree_isolation.rs` | Create | Worktree state isolation test | US-008, AC-008.4 |
| 36 | `commands/spec-dev.md`, `spec-fix.md`, `spec-refactor.md`, `design.md`, `implement.md` | Modify | Replace `!ecc-workflow` with `!ecc workflow` | US-003 |

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | normalize_path strips `..` components | AC-002.1, AC-002.3 | `cargo test -p ecc-domain workflow::path::tests` | PASS |
| PC-002 | unit | normalize_path strips `.` components | AC-002.1 | `cargo test -p ecc-domain workflow::path::tests` | PASS |
| PC-003 | unit | normalize_path preserves absolute paths | AC-002.2 | `cargo test -p ecc-domain workflow::path::tests` | PASS |
| PC-004 | unit | normalize_path handles complex traversal | AC-002.1 | `cargo test -p ecc-domain workflow::path::tests` | PASS |
| PC-005 | unit | Phase gate blocks traversal attack after normalization | AC-002.3 | `cargo test -p ecc-workflow phase_gate::tests::phase_gate_blocks_traversal_attack` | PASS |
| PC-006 | unit | Phase gate blocks absolute outside path | AC-002.2 | `cargo test -p ecc-workflow phase_gate::tests::phase_gate_blocks_absolute_outside` | PASS |
| PC-007 | unit | is_stale returns true when threshold exceeded | AC-006.1, AC-006.2 | `cargo test -p ecc-domain workflow::staleness::tests` | PASS |
| PC-008 | unit | is_stale returns false within threshold | AC-006.2 | `cargo test -p ecc-domain workflow::staleness::tests` | PASS |
| PC-009 | unit | verify_phase rejects Idle when Solution expected | AC-005.1, AC-005.5 | `cargo test -p ecc-domain workflow::phase_verify::tests` | PASS |
| PC-010 | unit | verify_phase rejects Plan when Implement expected | AC-005.2, AC-005.5 | `cargo test -p ecc-domain workflow::phase_verify::tests` | PASS |
| PC-011 | unit | verify_phase accepts correct phase | AC-005.3, AC-005.5 | `cargo test -p ecc-domain workflow::phase_verify::tests` | PASS |
| PC-012 | unit | verify_phase allows None state for init | AC-005.4, AC-005.5 | `cargo test -p ecc-domain workflow::phase_verify::tests` | PASS |
| PC-013 | unit | WorkflowState deserializes without version field (default 1) | AC-003.5 | `cargo test -p ecc-domain workflow::state::tests::version_field_default` | PASS |
| PC-014 | unit | WorkflowState serializes with version field | AC-003.5 | `cargo test -p ecc-domain workflow::state::tests::version_field_serialized` | PASS |
| PC-015 | unit | WorkflowState ignores unknown fields | AC-003.5 | `cargo test -p ecc-domain workflow::state::tests::ignores_unknown_fields` | PASS |
| PC-016 | unit | resolve_state_dir returns worktree git-dir path | AC-001.1, AC-001.3 | `cargo test -p ecc-app workflow::state_resolver::tests::worktree_returns_git_dir` | PASS |
| PC-017 | unit | resolve_state_dir returns independent path for worktree vs main | AC-001.2 | `cargo test -p ecc-app workflow::state_resolver::tests::worktree_independent_from_main` | PASS |
| PC-018 | unit | resolve_state_dir uses CLAUDE_PROJECT_DIR | AC-001.4 | `cargo test -p ecc-app workflow::state_resolver::tests::uses_claude_project_dir` | PASS |
| PC-019 | unit | resolve_state_dir falls back for non-git with warning | AC-001.5 | `cargo test -p ecc-app workflow::state_resolver::tests::non_git_fallback` | PASS |
| PC-020 | unit | resolve_state_dir reads from old location with migration warning | AC-001.6 | `cargo test -p ecc-app workflow::state_resolver::tests::old_location_fallback` | PASS |
| PC-021 | unit | resolve_state_dir uses bare repo git-dir | AC-001.7 | `cargo test -p ecc-app workflow::state_resolver::tests::bare_repo_support` | PASS |
| PC-022 | unit | recover archives state then resets to idle | AC-006.3 | `cargo test -p ecc-app workflow::recover::tests::recover_archives_and_resets` | PASS |
| PC-023 | unit | recover fails if archive write fails, state unchanged | AC-006.3 | `cargo test -p ecc-app workflow::recover::tests::recover_fails_if_archive_fails` | PASS |
| PC-024 | unit | status output includes STALE when threshold exceeded | AC-006.4 | `cargo test -p ecc-workflow status::tests::status_shows_stale` | PASS |
| PC-025 | unit | migrate_hooks_json replaces ecc-hook with ecc hook | AC-004.3 | `cargo test -p ecc-app install::hooks_migration::tests::replaces_ecc_hook` | PASS |
| PC-026 | unit | migrate_hooks_json is idempotent | AC-004.3 | `cargo test -p ecc-app install::hooks_migration::tests::idempotent` | PASS |
| PC-027 | unit | migrate_hooks_json preserves custom hooks | AC-004.4 | `cargo test -p ecc-app install::hooks_migration::tests::preserves_custom_hooks` | PASS |
| PC-028 | integration | Session start hook characterization | AC-008.1 | `cargo test -p ecc-integration-tests characterization_session_hooks::session_start_characterization` | PASS |
| PC-029 | integration | Session end hook characterization | AC-008.1 | `cargo test -p ecc-integration-tests characterization_session_hooks::session_end_characterization` | PASS |
| PC-030 | integration | merge_hooks_typed adds new hooks | AC-008.2 | `cargo test -p ecc-integration-tests characterization_typed_merge::add_new_hooks` | PASS |
| PC-031 | integration | merge_hooks_typed updates existing | AC-008.2 | `cargo test -p ecc-integration-tests characterization_typed_merge::update_existing_hooks` | PASS |
| PC-032 | integration | remove_legacy_hooks_typed removes legacy | AC-008.2 | `cargo test -p ecc-integration-tests characterization_typed_merge::remove_legacy_hooks` | PASS |
| PC-033 | integration | merge_hooks_typed preserves customizations | AC-008.2 | `cargo test -p ecc-integration-tests characterization_typed_merge::preserve_user_customizations` | PASS |
| PC-034 | e2e | Full workflow lifecycle init->done | AC-008.3 | `cargo test -p ecc-integration-tests characterization_workflow_lifecycle` | PASS |
| PC-035 | e2e | Worktree isolation: independent state | AC-008.4 | `cargo test -p ecc-integration-tests characterization_worktree_isolation -- --ignored` | PASS |
| PC-036 | integration | `ecc workflow init` succeeds | AC-003.1 | `cargo test -p ecc-integration-tests workflow_cli::init_succeeds` | PASS |
| PC-037 | integration | `ecc workflow status` matches ecc-workflow output | AC-003.1 | `cargo test -p ecc-integration-tests workflow_cli::status_parity` | PASS |
| PC-038 | integration | `ecc workflow transition` succeeds | AC-003.1 | `cargo test -p ecc-integration-tests workflow_cli::transition_parity` | PASS |
| PC-039 | integration | All 22 subcommands accessible | AC-003.1 | `cargo test -p ecc-integration-tests workflow_cli::all_subcommands_exist` | PASS |
| PC-040 | integration | ecc-workflow thin wrapper delegates | AC-003.2 | `cargo test -p ecc-integration-tests workflow_cli::thin_wrapper_delegation` | PASS |
| PC-041 | unit | Workflow commands use port traits | AC-003.3 | `cargo test -p ecc-app workflow::state_resolver::tests` | PASS |
| PC-042 | integration | `ecc workflow --verbose` emits tracing | AC-003.4 | `cargo test -p ecc-integration-tests workflow_cli::verbose_tracing` | PASS |
| PC-043 | integration | `ecc hook` matches `ecc-hook` behavior | AC-004.1 | `cargo test -p ecc-integration-tests hook_parity::check_hook_enabled_parity` | PASS |
| PC-044 | integration | ecc-hook thin wrapper with fallback | AC-004.2 | `cargo test -p ecc-integration-tests hook_parity::thin_wrapper_fallback` | PASS |
| PC-045 | unit | hooks.json migration safety gate | AC-004.5 | `cargo test -p ecc-app install::hooks_migration::tests::safety_gate` | PASS |
| PC-046 | unit | detect_staleness uses injectable clock | AC-006.1, AC-006.2 | `cargo test -p ecc-app workflow::recover::tests::staleness_with_mock_clock` | PASS |
| PC-047 | lint | Clippy passes with zero warnings | all | `cargo clippy --workspace -- -D warnings` | exit 0 |
| PC-048 | build | Full workspace builds | all | `cargo build --workspace` | exit 0 |
| PC-049 | unit | All existing tests pass (regression) | all | `cargo test --workspace` | PASS |
| PC-050 | lint | Format check passes | all | `cargo fmt --workspace -- --check` | exit 0 |

### Coverage Check

All 33 ACs covered. Zero uncovered ACs.

| AC | PCs |
|----|-----|
| AC-001.1 | PC-016 |
| AC-001.2 | PC-017 |
| AC-001.3 | PC-016 |
| AC-001.4 | PC-018 |
| AC-001.5 | PC-019 |
| AC-001.6 | PC-020 |
| AC-001.7 | PC-021 |
| AC-002.1 | PC-001, PC-002, PC-004, PC-005 |
| AC-002.2 | PC-003, PC-006 |
| AC-002.3 | PC-001, PC-005 |
| AC-003.1 | PC-036, PC-037, PC-038, PC-039 |
| AC-003.2 | PC-040 |
| AC-003.3 | PC-041 |
| AC-003.4 | PC-042 |
| AC-003.5 | PC-013, PC-014, PC-015 |
| AC-004.1 | PC-043 |
| AC-004.2 | PC-044 |
| AC-004.3 | PC-025, PC-026 |
| AC-004.4 | PC-027 |
| AC-004.5 | PC-045 |
| AC-005.1 | PC-009 |
| AC-005.2 | PC-010 |
| AC-005.3 | PC-011 |
| AC-005.4 | PC-012 |
| AC-005.5 | PC-009, PC-010, PC-011, PC-012 |
| AC-006.1 | PC-007, PC-046 |
| AC-006.2 | PC-007, PC-008, PC-046 |
| AC-006.3 | PC-022, PC-023 |
| AC-006.4 | PC-024 |
| AC-008.1 | PC-028, PC-029 |
| AC-008.2 | PC-030, PC-031, PC-032, PC-033 |
| AC-008.3 | PC-034 |
| AC-008.4 | PC-035 |

### E2E Test Plan

| # | Boundary | Adapter | Port | Test Description | Default State | Run When |
|---|----------|---------|------|------------------|---------------|----------|
| E2E-001 | FileSystem (state.json location) | OsFileSystem | FileSystem | State written to worktree-scoped path | ignored | `state_resolver.rs` or `io.rs` modified |
| E2E-002 | ShellExecutor (hook dispatch) | ProcessExecutor | ShellExecutor | `ecc hook <id>` same output as `ecc-hook <id>` | ignored | `hook.rs` or `hook/mod.rs` modified |
| E2E-003 | Environment (CLAUDE_PROJECT_DIR) | OsEnvironment | Environment | Workflow resolves state via git-dir | ignored | `state_resolver.rs` modified |
| E2E-004 | GitInfo (new) | OsGitInfo | GitInfo | Worktree, non-git, bare repo scenarios | ignored | `os_git.rs` or `state_resolver.rs` modified |
| E2E-005 | Clock (new) | SystemClock | Clock | Staleness fires at correct threshold | ignored | `system_clock.rs` or `recover.rs` modified |

### E2E Activation Rules
All 5 E2E tests un-ignored for this implementation (all boundaries touched).

## Test Strategy

### Test Naming Convention
All unit test modules MUST be named `#[cfg(test)] mod tests` inside their respective files. PC commands use `module::path::tests` filters. Implementors MUST follow this convention. If a test filter runs zero tests, it is a TDD RED failure requiring investigation.

### Characterization Test Lifecycle
- PC-028 through PC-034 (characterization tests) capture current ecc-workflow binary behavior
- After Group B converts ecc-workflow to a thin wrapper, PC-034 (full lifecycle) will still pass because the thin wrapper delegates to `ecc workflow` which produces identical output
- If thin wrapper behavior diverges, characterization tests catch the regression immediately
- PC-035 (worktree isolation) starts as `#[ignore]`'d (expected to fail pre-Group-A), un-ignored in Phase 9 after worktree-scoped I/O is implemented

### CI Gating
Each PR Group is a separate PR with its own CI run. Group B MUST NOT begin until Group A's CI is green on main. Group C MUST NOT begin until Group B's CI is green on main.

### Architecture Note: ecc-workflow Legacy I/O
AC-003.3 ("workflow commands use port traits") applies to the NEW `ecc workflow` code path through ecc-cli/ecc-app. The LEGACY ecc-workflow binary (File Change #20, `resolve_state_dir_legacy()`) retains direct `std::fs` and `std::process::Command` usage. This is an accepted deviation because: (1) ecc-workflow becomes a thin wrapper in Group B, (2) the cleanup PR removes it entirely, (3) the port-based path exists in parallel via `ecc-app::workflow::state_resolver`. AC-003.3 is satisfied by the ecc-cli path; the ecc-workflow binary is exempt.

### Existing `ecc hook` Command
`ecc hook` already exists in `crates/ecc-cli/src/commands/hook.rs` with full dispatch logic. AC-004.1 ("ecc hook matches ecc-hook behavior") is satisfied by existing code — no modifications needed to hook.rs. Group C only adds hooks.json migration and the ecc-hook thin wrapper.

TDD order (12 phases in 3 PR Groups):

1. **Phase 0**: Characterization tests (PC-028-035) -- safety net FIRST
2. **Phase 1**: Domain path normalization (PC-001-004, then PC-005-006)
3. **Phase 2**: Domain state version field (PC-013-015)
4. **Phase 3**: Domain staleness detection (PC-007-008)
5. **Phase 4**: Domain phase verification (PC-009-012)
6. **Phase 5**: Port traits GitInfo + Clock (PC-048 build gate)
7. **Phase 6**: Infra adapters + test doubles (PC-048 build gate)
8. **Phase 7**: App state resolver (PC-016-021)
9. **Phase 8**: App recovery (PC-022-023, PC-046, then PC-024)
10. **Phase 9**: ecc-workflow worktree-scoped I/O (PC-035 now passes)
11. **Phase 10**: ecc CLI workflow subcommand (PC-036-042), thin wrapper (PC-040)
12. **Phase 11**: Hook migration (PC-043, PC-025-027, PC-045, PC-044)
13. **Gates**: PC-047 (clippy), PC-048 (build), PC-049 (regression) after each PR Group

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | `CHANGELOG.md` | Top | Add entry | "refactor: unify ecc-hook and ecc-workflow into ecc CLI binary" | All US |
| 2 | `docs/adr/NNNN-binary-unification.md` | ADR | Create | Merge 3 binaries into 1 | Decision #1 |
| 3 | `docs/adr/NNNN-worktree-scoped-state.md` | ADR | Create | State under git-dir | Decision #2 |
| 4 | `docs/adr/NNNN-transition-triggered-hooks.md` | ADR | Create | Hooks as Rust trait objects | Decision #3 |
| 5 | `CLAUDE.md` | Top | Modify | Update CLI Commands, Gotchas, test count | US-003, US-001 |
| 6 | `docs/ARCHITECTURE.md` | Arch | Modify | Remove ecc-workflow standalone, update diagram | US-003 |
| 7 | `commands/spec-dev.md` | Cmd | Modify | `!ecc-workflow` -> `!ecc workflow` | US-003 |
| 8 | `commands/spec-fix.md` | Cmd | Modify | Same | US-003 |
| 9 | `commands/spec-refactor.md` | Cmd | Modify | Same | US-003 |
| 10 | `commands/design.md` | Cmd | Modify | Same | US-003 |
| 11 | `commands/implement.md` | Cmd | Modify | Same | US-003 |
| 12 | `hooks/hooks.json` | Config | Modify | `ecc-hook` -> `ecc hook` | US-004, AC-004.3 |
| 13 | `docs/MODULE-SUMMARIES.md` | Docs | Modify | Add entries for new modules (state_resolver, recover, git, clock, os_git, system_clock, path, staleness, phase_verify, hooks_migration) | All US |

## SOLID Assessment

**Verdict: CLEAN** (uncle-bob)

- **SRP**: PASS -- each new module has a single reason to change
- **OCP**: PASS -- GitInfo/Clock traits open for new adapters; hook migration pattern-scoped
- **LSP**: PASS -- MockGitInfo/MockClock fully substitute for real implementations
- **ISP**: PASS -- GitInfo has 2 methods, Clock has 2 methods -- minimal interfaces
- **DIP**: PASS -- app layer depends on port abstractions, never on concretions
- **Clean Architecture**: PASS -- dependency rule respected throughout; ecc-workflow legacy path accepted as bounded trade-off

## Robert's Oath Check

**Verdict: CLEAN**

- Oath 1 (No harm): Security fix (path traversal) is early in the design. Fallback prevents migration breakage.
- Oath 2 (No mess): Hexagonal architecture extended properly. Legacy path bounded to cleanup PR.
- Oath 3 (Proof): Characterization tests FIRST. All 33 ACs covered by 49 PCs. Injectable mocks.
- Oath 4 (Small releases): 3 PR Groups independently shippable. Compatibility matrix proves each intermediate state works.
- Oath 5 (Fearless improvement): Thin wrappers provide rollback. 1-week observation before cleanup.

## Security Notes

**Verdict: CLEAR** (2 LOW informational findings)

- Path traversal fix: Lexical normalization applied before `is_allowed_path()` -- correct and sufficient.
- Input validation: Clap's typed enum prevents injection in `ecc workflow` subcommands.
- State file: flock + atomic rename prevents TOCTOU.
- Hook migration: Pattern-scoped string replacement, idempotent, safety-gated.
- LOW-1: `is_allowed_path` false-positive matching (permissive, not exploitable).
- LOW-2: Thin wrapper PATH resolution (standard Unix trust model, not a real vulnerability).

## Rollback Plan

Reverse dependency order:

| Order | File | Rollback Action |
|-------|------|-----------------|
| 1 | `hooks/hooks.json` | Revert `ecc hook` -> `ecc-hook` |
| 2 | `crates/ecc-cli/src/commands/workflow.rs` | Delete file |
| 3 | `crates/ecc-cli/src/main.rs` | Remove Workflow variant |
| 4 | `crates/ecc-workflow/src/main.rs` | Revert thin wrapper to standalone |
| 5 | `crates/ecc-app/src/workflow/` | Delete directory |
| 6 | `crates/ecc-app/src/install/hooks_migration.rs` | Delete file |
| 7 | `crates/ecc-infra/src/os_git.rs`, `system_clock.rs` | Delete files |
| 8 | `crates/ecc-ports/src/git.rs`, `clock.rs` | Delete files |
| 9 | `crates/ecc-domain/src/workflow/path.rs`, `staleness.rs`, `phase_verify.rs` | Delete files |
| 10 | `crates/ecc-domain/src/workflow/state.rs` | Remove version field |
| 11 | `crates/ecc-workflow/src/io.rs` | Revert resolve_state_dir_legacy changes |
| 12 | `commands/spec-dev.md`, `spec-fix.md`, etc. | Revert `!ecc workflow` -> `!ecc-workflow` |

Per PR Group: revert the PR. Each group is self-contained. Characterization tests (files 32-35) are additive and safe to leave on revert.

## Phase Summary

### Design Reviews

| Review Type | Verdict | Finding Count |
|-------------|---------|---------------|
| SOLID (uncle-bob) | CLEAN | 0 blocking |
| Robert (conscience) | CLEAN | 0 |
| Security | CLEAR | 2 LOW (informational) |

### Adversary Findings

| Dimension | Score (R1 -> R2) | Verdict | Key Rationale |
|-----------|-----------------|---------|---------------|
| AC Coverage | 92 -> 92 | PASS | All 33 ACs covered by 50 PCs |
| Execution Order | 78 -> 82 | PASS | Sound TDD ordering with documented lifecycle |
| Fragility | 55 -> 68 | PASS | Test naming convention documented; PC-035 lifecycle clarified |
| Rollback Adequacy | 70 -> 80 | PASS | Rollback table expanded with io.rs and command files |
| Architecture Compliance | 62 -> 88 | PASS | AC-003.3 deviation documented; ecc-workflow legacy path accepted |
| Blast Radius | 58 -> 78 | PASS | Per-group CI gating explicit; existing ecc hook noted |
| Missing PCs | 82 -> 90 | PASS | PC-050 (cargo fmt) added |
| Doc Plan Completeness | 75 -> 88 | PASS | MODULE-SUMMARIES.md added |

### File Changes Summary

| # | File | Action | Spec Ref |
|---|------|--------|----------|
| 1-5 | ecc-domain: path.rs, staleness.rs, phase_verify.rs, state.rs, mod.rs | Create/Modify | US-002, US-005, US-006, AC-003.5 |
| 6-8 | ecc-ports: git.rs, clock.rs, lib.rs | Create/Modify | US-001, US-006 |
| 9-11 | ecc-infra: os_git.rs, system_clock.rs, lib.rs | Create/Modify | US-001, US-006 |
| 12-14 | ecc-test-support: mock_git.rs, mock_clock.rs, lib.rs | Create/Modify | US-001, US-006 |
| 15-18 | ecc-app: workflow/mod.rs, state_resolver.rs, recover.rs, lib.rs | Create/Modify | US-001, US-006 |
| 19-24 | ecc-workflow: phase_gate.rs, io.rs, status.rs, mod.rs, recover.rs, main.rs | Modify/Create | US-001, US-002, US-003, US-006 |
| 25-29 | ecc-cli: workflow.rs, mod.rs, main.rs, Cargo.toml, main.rs (wrapper) | Create/Modify | US-003 |
| 30-31 | hooks.json, hooks_migration.rs | Modify/Create | US-004 |
| 32-35 | ecc-integration-tests: 4 characterization test files | Create | US-008 |
| 36 | commands/*.md (5 files) | Modify | US-003 |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-04-01-deterministic-hook-system-redesign/spec.md | Full spec with Phase Summary |
| docs/specs/2026-04-01-deterministic-hook-system-redesign/design.md | Full design with Phase Summary |
