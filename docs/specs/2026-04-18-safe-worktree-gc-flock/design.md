# Solution: Safe Worktree GC via PID + Heartbeat (BL-156)

## Spec Reference

- **Concern**: fix
- **Feature**: Fix ecc worktree gc to skip live sessions via POSIX flock lock files (BL-156) — pivoted to PID + heartbeat hybrid per Decision #1
- **Spec**: `docs/specs/2026-04-18-safe-worktree-gc-flock/spec.md`

## File Changes (dependency order — US-004 ships FIRST per Decision #14)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `crates/ecc-app/src/worktree/shell_manager.rs` | modify | Replace 5 stubbed methods with real `git` invocations via `ShellExecutor` port. Standalone bug-fix; revertable independent of heartbeat. Decision #14. | US-004 |
| 2 | `crates/ecc-integration-tests/tests/shell_worktree_manager_real_git.rs` | create | `#[ignore]` real-git tempdir test exercising all 5 methods. | US-004 AC-004.1, AC-004.7 |
| 3 | `crates/ecc-domain/src/worktree.rs` | modify (folder-mod conversion) | Promote to `worktree/mod.rs` to host new `liveness` submodule. | US-001 |
| 4 | `crates/ecc-domain/src/worktree/liveness.rs` | create | `LivenessRecord` VO + `LivenessParseError` + `is_live` pure fn + `MIN_VALID_PID = 2` + future-skew guard. Zero I/O. | US-001 AC-001.6, .7, .8, .9, .10 |
| 5 | `crates/ecc-app/src/worktree/gc.rs` | modify | New `GcOptions { force, kill_live, dry_run, self_skip, liveness_disabled, ttl_secs, self_skip_fallback_secs }`. Consults `LivenessChecker::check` (US-005) before stale-timer. Self-skip early-continue. `--dry-run` short-circuit. | US-001, US-003, US-006, US-008 |
| 6 | `crates/ecc-app/src/worktree/heartbeat.rs` | create | `write_heartbeat(fs, worktree_path, pid, now_fn) -> Result<(), HeartbeatError>`. Canonicalize+prefix-check **before** `now_fn()` call (SEC-001). Atomic tmpfile+rename. | US-002 AC-002.1..10 |
| 7 | `crates/ecc-app/src/worktree/self_identity.rs` | create | `current_worktree(env, fs, shell) -> Option<WorktreeName>`. Reads `CLAUDE_PROJECT_DIR`, walks `.git` for `gitdir:`, **canonicalizes resolved path** (SEC-002). Returns `None` for main repo. | US-003 AC-003.1, .5 |
| 8 | `crates/ecc-app/src/worktree/checker.rs` | create | `LivenessChecker { fs, shell, clock, policy }` struct + `.check(&path) -> LivenessVerdict { Live, Stale, Dead, MissingFile, Malformed }`. SOLID-002 remediation. | US-005 AC-005.1 |
| 9 | `crates/ecc-app/src/worktree/status.rs` | modify | Replace inline `kill -0` with `LivenessChecker`. `--json` emits `liveness_reason`. | US-005 AC-005.2..5 |
| 10 | `crates/ecc-app/src/worktree/mod.rs` | modify | Export new submodules + `LivenessVerdict` enum. | US-001..US-005 |
| 11 | `crates/ecc-app/src/bypass_mgmt.rs` | modify | `gc()` consults `LivenessChecker::check` before token-dir deletion. | US-007 AC-007.1, .2, .3 |
| 12 | `crates/ecc-app/src/hook/handlers/tier3_session/lifecycle.rs` | modify | Pass `self_skip` into `gc()`. Write heartbeat on SessionStart and Stop (fire-and-forget, `tracing::warn!` on err). | US-002 AC-002.1, .3, .5; US-003 AC-003.2 |
| 13 | `crates/ecc-app/src/hook/handlers/tier2_post_tool_use/mod.rs` | modify | Write heartbeat on PostToolUse (single fire-and-forget call). | US-002 AC-002.2 |
| 14 | `crates/ecc-cli/src/commands/worktree.rs` | modify | Add `--dry-run`, `--kill-live`, `--yes` flags. Clap `requires = "force"` for `--kill-live` (AC-006.4). Read env vars here; validate (AC-009.2/.3). Non-TTY check (AC-006.5). Interactive prompt (AC-006.2). | US-006, US-008, US-009 |
| 15 | `crates/ecc-cli/src/commands/bypass.rs` | modify | Plumb new `bypass_mgmt::gc` args (worktrees + shell + ttl). | US-007 |
| 16 | `crates/ecc-test-support/src/mock_worktree.rs` | modify | Extend mock if needed for shell-failure scenarios. | US-004 AC-004.9 |
| 17 | `crates/ecc-integration-tests/tests/worktree_gc_concurrent.rs` | create | All concurrent-session scenarios + kill switch + dry-run + gitignore E2E. | US-001..US-009 |
| 18 | `.gitignore` | modify | Add `.ecc-session` line. | Decision #11, AC-009.4 |
| 19 | `docs/adr/NNNN-pid-heartbeat-liveness.md` | create | Status/Context/Decision/Consequences. Number resolved at commit time via `ls docs/adr/`. | Decision #8 |
| 20 | `CLAUDE.md` | modify | Restore multi-session safety Gotcha line. | Doc Impact |
| 21 | `CHANGELOG.md` | modify | `## [Unreleased]` Fixed/Added/Changed entries. | Doc Impact |
| 22 | `docs/commands-reference.md` | modify | New flags + env vars. | Doc Impact |
| 23 | `docs/backlog/BL-156-safe-worktree-gc-session-aware.md` | modify | `status: open` → `status: implemented`. | Doc Impact |

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | `unmerged_commit_count` invokes `git rev-list --count <base>..HEAD` | AC-004.1 | `cargo test -p ecc-app shell_manager::tests::unmerged_commit_count_invokes_git_rev_list` | PASS |
| PC-002 | unit | `has_uncommitted_changes` checks porcelain | AC-004.2 | `cargo test -p ecc-app shell_manager::tests::has_uncommitted_changes_checks_porcelain` | PASS |
| PC-003 | unit | `has_untracked_files` checks `git ls-files --others` | AC-004.3 | `cargo test -p ecc-app shell_manager::tests::has_untracked_files_checks_ls_files` | PASS |
| PC-004 | unit | `has_stash` checks `git stash list` | AC-004.4 | `cargo test -p ecc-app shell_manager::tests::has_stash_checks_stash_list` | PASS |
| PC-005 | unit | `is_pushed_to_remote` checks count vs `origin/<branch>` | AC-004.5 | `cargo test -p ecc-app shell_manager::tests::is_pushed_checks_count_against_origin` | PASS |
| PC-006 | unit | Shell failure → `Err(WorktreeError)` propagation | AC-004.6, AC-004.9 | `cargo test -p ecc-app shell_manager::tests::shell_failure_propagates_as_err` | PASS |
| PC-007 | unit | Existing OsWorktreeManager behavior unchanged | AC-004.7 | `cargo test -p ecc-infra worktree` | PASS |
| PC-008 | unit | Non-numeric stdout → `Err(ParseError)` | AC-004.8 | `cargo test -p ecc-app shell_manager::tests::non_numeric_stdout_propagates_err` | PASS |
| PC-009 | unit | BOM/locale noise → `Err` | AC-004.8 | `cargo test -p ecc-app shell_manager::tests::bom_in_stdout_propagates_err` | PASS |
| PC-010 | e2e | Real-git tempdir: 5 methods correct | AC-004.1 | `cargo test -p ecc-integration-tests -- --ignored shell_worktree_manager_real_git` | PASS |
| PC-011 | git-log | US-004 commits land before US-001 | AC-004.10 | `git log --oneline | awk '/US-001|US-004/ {print}' | head -20` | US-004 commits precede US-001 |
| PC-012 | unit | Remote-not-found in `is_pushed_to_remote` → `Ok(false)` | AC-004.5, AC-004.6 | `cargo test -p ecc-app shell_manager::tests::remote_not_found_returns_unpushed` | PASS |
| PC-013 | unit | `LivenessRecord::parse` round-trips schema_version 1 | AC-001.6 | `cargo test -p ecc-domain worktree::liveness::parse_round_trip` | PASS |
| PC-014 | unit | `parse` rejects `schema_version != 1` | AC-001.10 | `cargo test -p ecc-domain worktree::liveness::parse_rejects_unknown_schema` | PASS |
| PC-015 | unit | `parse` rejects PID 0 | AC-001.8 | `cargo test -p ecc-domain worktree::liveness::parse_rejects_pid_0` | PASS |
| PC-016 | unit | `parse` rejects PID 1 | AC-001.8 | `cargo test -p ecc-domain worktree::liveness::parse_rejects_pid_1` | PASS |
| PC-017 | unit | `is_live` false when stale | AC-001.7 | `cargo test -p ecc-domain worktree::liveness::is_live_false_for_stale` | PASS |
| PC-018 | unit | `is_live` false for future timestamp > now+60 | AC-001.9 | `cargo test -p ecc-domain worktree::liveness::is_live_false_for_future_ts` | PASS |
| PC-019 | unit | GC skips fresh+alive with stderr | AC-001.1 | `cargo test -p ecc-app gc::tests::skips_when_fresh_heartbeat_and_pid_alive` | PASS |
| PC-020 | unit | GC removes when PID reaped despite fresh heartbeat | AC-001.2 | `cargo test -p ecc-app gc::tests::removes_when_pid_reaped` | PASS |
| PC-021 | unit | GC removes when heartbeat stale (>60min) | AC-001.3 | `cargo test -p ecc-app gc::tests::removes_when_heartbeat_stale` | PASS |
| PC-022 | unit | GC falls back to 24h timer when no `.ecc-session` | AC-001.4 | `cargo test -p ecc-app gc::tests::missing_session_file_falls_back` | PASS |
| PC-023 | unit | GC treats malformed JSON as missing | AC-001.5 | `cargo test -p ecc-app gc::tests::malformed_session_file_falls_back` | PASS |
| PC-024 | unit | `WorktreeName` comparison case-insensitive on darwin | AC-001.11 | `cargo test -p ecc-domain worktree::eq_platform_case_normalized` | PASS |
| PC-025 | grep | `ecc-domain::worktree::liveness` zero I/O imports | AC-001.6 | `! grep -rE 'use std::(fs\|env\|process)\|use tokio' crates/ecc-domain/src/worktree/liveness.rs` | empty |
| PC-026 | unit | SessionStart writes heartbeat via tmpfile+rename | AC-002.1 | `cargo test -p ecc-app heartbeat::tests::session_start_writes_heartbeat` | PASS |
| PC-027 | unit | PostToolUse refreshes `last_seen` | AC-002.2 | `cargo test -p ecc-app heartbeat::tests::post_tool_use_refreshes_last_seen` | PASS |
| PC-028 | unit | Stop hook writes final heartbeat | AC-002.3 | `cargo test -p ecc-app heartbeat::tests::stop_writes_final` | PASS |
| PC-029 | e2e | Concurrent writes never tear (atomic rename) | AC-002.4 | `cargo test -p ecc-integration-tests -- --ignored concurrent_heartbeat_writes_no_tear` | PASS |
| PC-030 | unit | FS write failure logs WARN + does NOT block hook | AC-002.5, AC-002.9 | `cargo test -p ecc-app heartbeat::tests::fs_failure_logs_warn_nonblocking` | PASS |
| PC-031 | unit | Write outside worktree is no-op | AC-002.6 | `cargo test -p ecc-app heartbeat::tests::noop_outside_worktree` | PASS |
| PC-032 | unit | Stale `.ecc-session` overwritten on SessionStart | AC-002.7 | `cargo test -p ecc-app heartbeat::tests::overwrites_stale_session_file` | PASS |
| PC-033 | unit | Concurrent PostToolUse: write-time timestamp prevents stale-overwrites-newer | AC-002.8 | `cargo test -p ecc-app heartbeat::tests::write_time_not_read_time` | PASS |
| PC-033b | unit | `now_fn()` invoked AFTER canonicalize (counting mock) — SEC-001 closure | AC-002.8 | `cargo test -p ecc-app heartbeat::tests::now_fn_called_after_canonicalize` | PASS |
| PC-034 | unit | Symlink escape rejected | AC-002.10 | `cargo test -p ecc-app heartbeat::tests::rejects_symlink_escape` | PASS |
| PC-035 | unit | `..` traversal rejected | AC-002.10 | `cargo test -p ecc-app heartbeat::tests::rejects_path_traversal` | PASS |
| PC-036 | unit | `current_worktree()` returns `Some` when in worktree | AC-003.1 | `cargo test -p ecc-app self_identity::tests::reads_claude_project_dir` | PASS |
| PC-037 | unit | GC early-continue on self-skip match | AC-003.2 | `cargo test -p ecc-app gc::tests::skips_self_worktree` | PASS |
| PC-038 | unit | Resolver `None` → all session worktrees < 60 min skipped | AC-003.3 | `cargo test -p ecc-app gc::tests::skips_young_when_resolver_none` | PASS |
| PC-039 | unit | Non-session worktree unaffected by resolver None | AC-003.4 | `cargo test -p ecc-app gc::tests::non_session_unaffected_by_none_resolver` | PASS |
| PC-040 | unit | `current_worktree` returns `None` for main repo | AC-003.5 | `cargo test -p ecc-app self_identity::tests::returns_none_for_main_repo` | PASS |
| PC-040b | unit | `current_worktree` canonicalizes resolved gitdir path — SEC-002 closure | AC-003.5 | `cargo test -p ecc-app self_identity::tests::canonicalizes_gitdir_path_against_symlinks` | PASS |
| PC-041 | unit | Fallback window honors `ECC_WORKTREE_SELF_SKIP_FALLBACK_SECS` default | AC-003.6 | `cargo test -p ecc-app gc::tests::fallback_default_3600` | PASS |
| PC-042 | unit | Fallback window honors env override | AC-003.6 | `cargo test -p ecc-app gc::tests::fallback_env_override_applies` | PASS |
| PC-043 | unit | `LivenessChecker::check` is single source of truth | AC-005.1 | `cargo test -p ecc-app worktree::checker::tests::gc_and_status_use_same_checker` | PASS |
| PC-044 | unit | `status` shows `live` for fresh heartbeat + live PID | AC-005.2 | `cargo test -p ecc-app status::tests::shows_live` | PASS |
| PC-045 | unit | `status` shows `stale`/`dead` for stale or dead PID | AC-005.3 | `cargo test -p ecc-app status::tests::shows_stale_or_dead` | PASS |
| PC-046 | unit | `status` falls back to `kill -0` when no `.ecc-session` | AC-005.4 | `cargo test -p ecc-app status::tests::falls_back_to_kill_0` | PASS |
| PC-047 | unit | `status --json` emits `liveness_reason` | AC-005.5 | `cargo test -p ecc-app status::tests::json_emits_liveness_reason` | PASS |
| PC-048 | integration | `--force` without `--kill-live` respects liveness | AC-006.1 | `cargo test -p ecc-integration-tests -- --ignored force_respects_liveness` | PASS |
| PC-049 | integration | Interactive `--force --kill-live` shows confirmation prompt | AC-006.2 | `cargo test -p ecc-integration-tests -- --ignored kill_live_prompts` | PASS |
| PC-050 | integration | `--force --kill-live --yes` bypasses prompt | AC-006.3 | `cargo test -p ecc-integration-tests -- --ignored kill_live_yes_bypasses_prompt` | PASS |
| PC-051 | cli | `--kill-live` without `--force` rejected by clap | AC-006.4 | `cargo run -p ecc-cli -- worktree gc --kill-live 2>&1 | grep 'requires --force'` | matches |
| PC-052 | integration | Non-TTY `--force --kill-live` without `--yes` exits non-zero | AC-006.5 | `cargo test -p ecc-integration-tests -- --ignored kill_live_non_tty_requires_yes` | PASS |
| PC-053 | unit | `bypass::gc` consults `LivenessChecker::check` | AC-007.1 | `cargo test -p ecc-app bypass_mgmt::tests::gc_consults_checker` | PASS |
| PC-054 | unit | Bypass-token preserved when sibling worktree live | AC-007.2 | `cargo test -p ecc-app bypass_mgmt::tests::preserves_live_sibling` | PASS |
| PC-055 | unit | Existing bypass::gc tests pass unchanged | AC-007.3 | `cargo test -p ecc-app bypass_mgmt` | PASS (no regressions) |
| PC-056 | unit | `--dry-run` prints `WOULD DELETE: ...` | AC-008.1 | `cargo test -p ecc-app gc::tests::dry_run_prints_would_delete` | PASS |
| PC-057 | unit | `--dry-run` makes zero `remove_worktree` calls | AC-008.2 | `cargo test -p ecc-app gc::tests::dry_run_no_side_effects` | PASS |
| PC-058 | unit | `--dry-run --force --kill-live` includes live worktrees | AC-008.3 | `cargo test -p ecc-app gc::tests::dry_run_kill_live_preview` | PASS |
| PC-059 | integration | `--dry-run --json` emits `[{name, action, reason}]` | AC-008.4 | `cargo test -p ecc-integration-tests -- --ignored dry_run_json_schema` | PASS |
| PC-060 | integration | `ECC_WORKTREE_LIVENESS_DISABLED=1` disables read AND write | AC-009.1 | `cargo test -p ecc-integration-tests -- --ignored liveness_disabled_kill_switch` | PASS |
| PC-061 | unit | Malformed `ECC_WORKTREE_LIVENESS_TTL_SECS` → WARN + default | AC-009.2 | `cargo test -p ecc-cli worktree::tests::invalid_ttl_env_warns_and_defaults` | PASS |
| PC-062 | unit | Malformed `ECC_WORKTREE_SELF_SKIP_FALLBACK_SECS` → WARN + default | AC-009.3 | `cargo test -p ecc-cli worktree::tests::invalid_fallback_env_warns_and_defaults` | PASS |
| PC-063 | integration | `.ecc-session` present → `git status --porcelain` empty | AC-009.4 | `cargo test -p ecc-integration-tests -- --ignored ecc_session_gitignored` | PASS |
| PC-064 | grep | `.gitignore` contains `.ecc-session` literal line | AC-009.4 | `grep -x '.ecc-session' .gitignore` | match |
| PC-065 | build | Workspace compiles clean | all | `cargo build --release` | exit 0 |
| PC-066 | lint | Zero clippy warnings | all | `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| PC-067 | fmt | rustfmt clean | all | `cargo fmt --all --check` | exit 0 |
| PC-068 | test | All workspace tests pass | all | `cargo nextest run --workspace` | exit 0 |
| PC-069 | grep | `ecc-domain` zero I/O imports (hook enforcement) | all domain | `grep -rE 'use std::(fs\|env\|process)\|use tokio' crates/ecc-domain/src/` | empty |
| PC-070 | semver | `cargo semver-checks` passes | all | `cargo semver-checks --workspace` | exit 0 |
| PC-071 | unit | Heartbeat write is no-op when `ECC_WORKTREE_LIVENESS_DISABLED=1` (write-suppression unit) — closes adversary R1 finding | AC-009.1 | `cargo test -p ecc-app heartbeat::tests::write_suppressed_when_kill_switch_set` | PASS |
| PC-072 | unit | `is_live` rejects negative-delta clock skew (now < last_seen by 1s, NTP jitter) | AC-001.9 | `cargo test -p ecc-domain worktree::liveness::is_live_handles_negative_skew` | PASS (record treated as stale per defensive policy) |
| PC-073 | integration | `--dry-run --force --kill-live --yes` previews live worktrees with no prompt + no destructive calls | AC-008.3, AC-006.3 | `cargo test -p ecc-integration-tests -- --ignored dry_run_kill_live_yes_no_destructive` | PASS |
| PC-074 | unit | `LivenessChecker` is `Send+Sync`; concurrent `check()` across threads returns consistent verdicts | AC-005.1 | `cargo test -p ecc-app worktree::checker::tests::checker_send_sync_concurrent_safe` | PASS |
| PC-075 | git-log | Reverting commits 5..29 leaves repo at green build (US-004 separability) — closes Decision #14 verifiability | AC-004.10 | `git revert --no-commit HEAD~25..HEAD && cargo build --release && cargo nextest run --workspace` | exit 0 |
| PC-076 | unit | `current_worktree` parser tolerates trailing whitespace / CRLF in `.git` `gitdir:` line | AC-003.1, AC-003.5 | `cargo test -p ecc-app self_identity::tests::gitdir_parser_strips_whitespace` | PASS |
| PC-077 | unit | PostToolUse heartbeat write is non-blocking under slow-fs scenarios (timeout < 100ms) | AC-002.5, AC-002.9 | `cargo test -p ecc-app heartbeat::tests::posttooluse_nonblocking_under_slow_fs` | PASS |
| PC-078 | grep | `LivenessVerdict` enum defined in exactly one location (no duplicates) | AC-005.1 | `grep -rEc '^(pub )?enum LivenessVerdict' crates/` | total count = 1 |

### Coverage Check

**All 58 ACs covered by ≥1 PC. Plus 2 design-review-remediation PCs (PC-033b for SEC-001, PC-040b for SEC-002).**

Per-AC mapping (from planner output):
- US-001 (11 ACs): PC-013..025 + PC-019..023
- US-002 (10 ACs): PC-026..035 + PC-033b
- US-003 (6 ACs): PC-036..042 + PC-040b
- US-004 (10 ACs): PC-001..012
- US-005 (5 ACs): PC-043..047
- US-006 (5 ACs): PC-048..052
- US-007 (3 ACs): PC-053..055
- US-008 (4 ACs): PC-056..059
- US-009 (4 ACs): PC-060..064

### E2E Test Plan

| # | Boundary | Adapter | Port | Test Description | Default State | Run When |
|---|----------|---------|------|------------------|---------------|----------|
| 1 | ShellWorktreeManager → git CLI | real git | `ShellExecutor` | 5 methods against real tempdir repo | ignored | Always in CI |
| 2 | GC → FileSystem (.ecc-session read) | OsFileSystem | `FileSystem` | `gc_skips_worktree_with_fresh_heartbeat` (concurrent sessions) | ignored | Always in CI |
| 3 | Heartbeat → FileSystem atomic rename | OsFileSystem | `FileSystem` | `concurrent_heartbeat_writes_no_tear` | ignored | Always in CI |
| 4 | CLI → Env subprocess | std::env | n/a | `liveness_disabled_kill_switch` (AC-009.1) | ignored | Always in CI |
| 5 | CLI clap parsing | clap | n/a | `kill_live_non_tty_requires_yes`, `kill_live_yes_bypasses_prompt`, `force_respects_liveness`, `dry_run_json_schema` | ignored | Always in CI |
| 6 | Gitignore enforcement | real git | n/a | `ecc_session_gitignored` (AC-009.4) | ignored | Always in CI |

### E2E Activation Rules

All 6 un-ignored for this implementation: feature ships with full concurrent-session E2E coverage from day 1.

## Test Strategy (TDD Order — 29 commits)

| # | Commit | PCs |
|---|--------|-----|
| 1 | `test: add ShellWorktreeManager real-git unit tests (US-004)` | PC-001..009, 012 RED |
| 2 | `fix: replace ShellWorktreeManager stubs with real git invocations (US-004)` | PC-001..009, 012 GREEN |
| 3 | `refactor: extract run_git_count helper in ShellWorktreeManager (US-004)` | REFACTOR |
| 4 | `test: add ShellWorktreeManager real-git integration test (US-004)` | PC-010 |
| 5 | `test: add LivenessRecord unit tests (US-001)` | PC-013..018 RED |
| 6 | `feat: add LivenessRecord domain VO + is_live (US-001)` | PC-013..018 GREEN, PC-025 |
| 7 | `refactor: add WorktreeName::eq_platform for darwin case-insensitivity (US-001)` | PC-024 |
| 8 | `test: add GC heartbeat consultation tests (US-001)` | PC-019..023 RED |
| 9 | `feat: GC consults .ecc-session heartbeat via shared helper (US-001)` | PC-019..023 GREEN |
| 10 | `refactor: update GC call sites to new GcOptions signature (US-001)` | call-site cleanup |
| 11 | `test: add heartbeat write unit tests (US-002)` | PC-026..035 + PC-033b RED |
| 12 | `feat: add write_heartbeat with canonicalize+now_fn ordering AND ECC_WORKTREE_LIVENESS_DISABLED gate (US-002, US-009)` | PC-026..035 + PC-033b + PC-071 GREEN |
| 13 | `feat: wire heartbeat into SessionStart/PostToolUse/Stop hooks (US-002)` | hook integration; PC-071 ensures hook writes are no-op when kill switch set from this commit forward |
| 14 | `test: add self-identity resolver + GC self-skip tests (US-003)` | PC-036..042 + PC-040b RED |
| 15 | `feat: add current_worktree resolver + GC self-skip with canonicalize (US-003)` | PC-036..042 + PC-040b GREEN |
| 16 | `test: add status liveness consistency tests (US-005)` | PC-043..047 RED |
| 17 | `refactor: extract LivenessChecker struct (SOLID-002, US-005)` | PC-043..047 GREEN |
| 18 | `test: add CLI flag integration tests for --kill-live (US-006)` | PC-048..052 RED |
| 19 | `feat: add --kill-live / --yes flags with TTY-aware prompting (US-006)` | PC-048..052 GREEN |
| 20 | `test: add bypass::gc liveness-aware tests (US-007)` | PC-053..055 RED |
| 21 | `feat: bypass::gc consults LivenessChecker before token deletion (US-007)` | PC-053..055 GREEN |
| 22 | `test: add --dry-run preview tests (US-008)` | PC-056..059 RED |
| 23 | `feat: add --dry-run flag with plain+JSON output (US-008)` | PC-056..059 GREEN |
| 24 | `test: add kill-switch + env-validation + gitignore tests (US-009)` | PC-060..064 RED |
| 25 | `feat: add ECC_WORKTREE_LIVENESS_* env vars with WARN-on-malformed (US-009)` | PC-060..062 GREEN |
| 26 | `chore: add .ecc-session to .gitignore (US-009)` | PC-063, PC-064 |
| 27 | `docs: ADR for PID+heartbeat liveness + restore multi-session safety (US-009)` | docs |
| 28 | `docs: update commands-reference + CHANGELOG for BL-156 (US-009)` | docs |
| 29 | `chore: mark BL-156 as implemented (US-009)` | docs |

**Gate between commits**: `cargo build --release && cargo nextest run --workspace && cargo clippy --workspace --all-targets -- -D warnings` must all pass. Final verification: PC-065..070.

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | `docs/adr/NNNN-pid-heartbeat-liveness.md` | upper | create | Status/Context/Decision/Consequences ADR. Documents flock rejection (no long-lived process), heartbeat design, .ecc-session JSON format, self-skip policy, ShellWorktreeManager stub fix justification. | Decision #8 |
| 2 | `CLAUDE.md` | upper | modify | Restore Gotchas line: "Multi-session safety: parallel Claude Code sessions in the same repo are now safe — `ecc worktree gc` consults a PID + heartbeat liveness check (`<worktree>/.ecc-session`); kill switch via `ECC_WORKTREE_LIVENESS_DISABLED=1`. See ADR NNNN." | Doc Impact |
| 3 | `CHANGELOG.md` | upper | modify | `## [Unreleased]` Fixed (BL-156 data-loss), Added (heartbeat + flags + kill switch), Changed (ShellWorktreeManager real git). | All USes |
| 4 | `docs/commands-reference.md` | upper | modify | Document `--dry-run`, `--force --kill-live`, `--yes`, `ECC_WORKTREE_LIVENESS_DISABLED`, `ECC_WORKTREE_LIVENESS_TTL_SECS`, `ECC_WORKTREE_SELF_SKIP_FALLBACK_SECS`. | Doc Impact |
| 5 | `.gitignore` | repo | modify | Add `.ecc-session`. | Decision #11 |
| 6 | `docs/backlog/BL-156-safe-worktree-gc-session-aware.md` | backlog | modify | `status: open` → `status: implemented`. | Doc Impact |
| 7 | `docs/MODULE-SUMMARIES.md` | spec-artifact | modify (Phase 7.5) | New entries for `liveness`, `heartbeat`, `self_identity`, `checker`. | Phase 7.5 |
| 8 | `docs/cartography/elements/worktree-gc-liveness.md` | spec-artifact | create (Phase 7.5) | Component diagram of heartbeat-write + GC-consult flow. | Phase 7.5 |

CHANGELOG.md included ✓. ADR for Decision #8 (`ADR Needed? Yes`) included ✓.

## Implementer Notes (from adversary R1)

- **Clock port choice**: `LivenessChecker { fs, shell, clock, policy }` requires a time source. ECC has no `Clock` port today. Implementer chooses one of: (a) take a `now_fn: Fn() -> u64` closure on `LivenessChecker::new` (zero-port-surface change, mirrors `write_heartbeat`); (b) introduce a tiny `Clock` port in `ecc-ports` (port-surface change, requires production `SystemClock` adapter in `ecc-infra` + `MockClock` in `ecc-test-support`). Decision deferred to implementer per Oath 7 (easy substitution); recommend (a) for minimal scope creep.
- **`LivenessVerdict` location**: lives in `ecc-app::worktree::checker` (consumer-side). PC-078 enforces single-source-of-truth via grep guard.
- **ADR number resolution**: at commit 27 (ADR creation), resolve via `ls docs/adr/ | sort -n | tail -1` and increment. If a parallel session files an ADR concurrently, post-merge rebase will surface the collision and the implementer renames.
- **CHANGELOG cross-reference**: Fixed entry references both `BL-156` (primary) and `BL-150 follow-up` (implicit closure of the parent_id PID fix's incomplete safety net).
- **Phase 7.5 ordering**: `docs/MODULE-SUMMARIES.md` and `docs/cartography/elements/worktree-gc-liveness.md` land in Phase 7.5 (post-implement orchestration), NOT in commits 27-28. Commits 27-28 are the ADR + CHANGELOG + commands-reference docs. Module summaries + diagrams are dispatched to dedicated subagents per `/implement` Phase 7.5 contract.

## SOLID Assessment

**PASS with adjustments incorporated**:

- SRP: clean — `LivenessRecord`/`is_live`/`write_heartbeat`/`current_worktree`/`LivenessChecker` each one reason to change.
- OCP: `LivenessVerdict::Reason` enum extension deferred (SOLID-001) — tracked as future BL when second liveness signal needed.
- LSP: clean — three `WorktreeManager` impls (`Os`, `Shell`, `Mock`) all honor new contract; Decision #14 sequencing enforces this by shipping `Shell` real impls first.
- ISP: `LivenessChecker` struct adopted instead of 5-arg helper fn (SOLID-002 remediation).
- DIP: clean — domain zero I/O (PC-069 enforces), app uses ports only, CLI owns env+clap.

## Robert's Oath Check

**CLEAN with 2 implementer notes**:

- All 9 oaths satisfied. Defensive design throughout (kill switch, symlink/traversal guard, schema validation, future-timestamp guard, PID 0/1 rejection).
- Implementer note 1 (Oath 2 "no mess"): 70 PCs / 29 commits is large but each is single-concern; verify each commit compiles before next.
- Implementer note 2 (Oath 5 "fearless improvement"): GC signature change is broad-blast; rehearse local revert before merging US-001.

## Security Notes

**CLEAR with 2 must-verify items addressed in PCs**:

- SEC-001 (HIGH-verify): `now_fn()` call-ordering — PC-033b added (counting mock asserts `now_fn()` invoked AFTER canonicalize).
- SEC-002 (LOW): `current_worktree` `.git` symlink — PC-040b added (canonicalizes resolved gitdir path).
- All other dimensions clean: input validation (schema/PID/JSON/timestamp), path traversal (canonicalize+prefix), TOCTOU (atomic rename), env handling (WARN-on-malformed), kill switch (operator-set, not default-on, both read+write).

## Rollback Plan

**Important — adversary R1 correction**: rollback is **revert-from-HEAD in dependency order**, NOT "any single commit at any time". Commits 9-13 share signature changes (`gc()` → `GcOptions`); reverting commit 9 alone would leave commits 10-13 calling a removed signature. The correct procedure is to revert sequentially from the latest commit back toward commit 5. Commits 1-4 (US-004) remain independently revertable per Decision #14 — verified by PC-075.

In reverse dependency order from HEAD:

1. Revert commit 29 (BL-156 status). No code impact.
2. Revert commits 28, 27 (CHANGELOG, ADR, CLAUDE.md). Pure docs.
3. Revert commits 26 (gitignore), 25 (env validation). Heartbeat reads/writes stop being created.
4. Revert commits 24-23 (`--dry-run`). Removes preview flag.
5. Revert commits 22-20 (bypass::gc + tests). Returns bypass-gc to existing string-match logic.
6. Revert commits 19-18 (`--kill-live`/`--yes`). Removes destructive override path.
7. Revert commits 17-16 (`LivenessChecker`). Reverts US-005 helper extraction.
8. Revert commits 15-14 (self-identity). GC loses self-skip.
9. Revert commits 13-11 (heartbeat). No more `.ecc-session` files written.
10. Revert commits 10-8 (GC wiring). GC returns to BL-150 logic without heartbeat consultation.
11. Revert commits 7-5 (LivenessRecord). Domain submodule removed.
12. Commits 1-4 (US-004 stub fix) **stay landed** — independently revertable per Decision #14.

**Emergency rollback (production)**: set `ECC_WORKTREE_LIVENESS_DISABLED=1` in CI / shell — fully disables both read and write paths without a code revert (AC-009.1, PC-060).

## Bounded Contexts Affected

| Context | Role | Files Modified |
|---------|------|----------------|
| `worktree` | aggregate (gc, status, shell_manager) + new VO `LivenessRecord` (sub-aggregate `liveness`) + new orchestrators (heartbeat, self_identity, checker) | `crates/ecc-domain/src/worktree.rs` (folder-mod), `crates/ecc-domain/src/worktree/liveness.rs`, `crates/ecc-app/src/worktree/{gc,status,shell_manager,heartbeat,self_identity,checker,mod}.rs` |
| `session` | hook handlers — new responsibility to write heartbeat | `crates/ecc-app/src/hook/handlers/tier3_session/lifecycle.rs`, `crates/ecc-app/src/hook/handlers/tier2_post_tool_use/mod.rs` |

Other domain modules (not registered as bounded contexts):
- none

Cross-context interaction: `session` writes heartbeat consumed by `worktree` GC. Communication via primitive `u32` PID and `u64` timestamp — no cross-context VO leakage.
