# Design: Audit Findings Remediation — All 24 Smells

## Overview

This design addresses 50 acceptance criteria across 8 user stories, remediating 24 audit smells. The dependency order is: US-001/002/003 (parallel) -> US-007 -> US-004/006/008 (parallel) -> US-005.

---

## 1. File Changes Table

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| **US-001: Observable CLI Operations** | | | | |
| 1 | `crates/ecc-cli/src/main.rs` | modify | Change default log level from unset to `warn` via `env_logger::Builder::from_env(Env::default().default_filter_or("warn"))` | AC-001.1, AC-001.2 |
| 2 | `crates/ecc-cli/src/commands/install.rs` | modify | Add `eprintln!("Error: {e}")` before `std::process::exit(1)` on failure paths | AC-001.5 |
| 3 | `crates/ecc-cli/src/commands/dev.rs` | modify | Add `eprintln!("Error: {e}")` before exit(1) on failure paths | AC-001.5 |
| 4 | `crates/ecc-workflow/Cargo.toml` | modify | Add `log = "0.4"` and `env_logger = "0.11"` to dependencies | AC-001.3, AC-002.5 |
| 5 | `crates/ecc-workflow/src/main.rs` | modify | Add `env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init()` after bypass check | AC-001.3, AC-002.5 |
| 6 | `crates/ecc-app/src/validate.rs` -> submodules (deferred to US-007) | — | Error-discard sites addressed after split | AC-001.4 |
| 7 | `crates/ecc-app/src/config/audit/mod.rs` | modify | Replace `Err(_) =>` with `Err(e) => { log::warn!("load_hooks_json: {e}"); ... }` | AC-001.4 |
| 8 | `crates/ecc-app/src/config/clean.rs` | modify | Add `log::warn!` at error-discard sites | AC-001.4 |
| 9 | `crates/ecc-app/src/config/merge.rs` | modify | Add `log::warn!` at error-discard sites | AC-001.4 |
| 10 | `crates/ecc-app/src/detect.rs` | modify | Add `log::warn!` at error-discard sites | AC-001.4 |
| 11 | `crates/ecc-app/src/install/global.rs` -> submodules (deferred to US-007) | — | Error-discard sites addressed after split | AC-001.4 |
| 12 | `crates/ecc-app/src/install/helpers/settings.rs` | modify | Add `log::warn!` at error-discard sites | AC-001.4 |
| 13 | `crates/ecc-app/src/merge/mod.rs` | modify | Add `log::warn!` at error-discard sites | AC-001.4 |
| 14 | `crates/ecc-app/src/smart_merge.rs` | modify | Add `log::warn!` at error-discard sites | AC-001.4 |
| 15 | `crates/ecc-app/src/hook/handlers/tier1_simple/helpers.rs` | modify | Add `log::warn!` at error-discard sites | AC-001.4 |
| 16 | `crates/ecc-app/src/hook/handlers/tier3_session/helpers.rs` | modify | Add `log::warn!` at error-discard sites | AC-001.4 |
| **US-002: Testable Workflow Commands** | | | | |
| 17 | `crates/ecc-workflow/src/commands/memory_write.rs` | modify | Extract pure functions: `build_action_entry`, `build_work_item_content`, `build_daily_content`, `build_memory_index_content` as `pub(crate)` fns taking typed params, returning String | AC-002.1 |
| 18 | `crates/ecc-workflow/src/commands/memory_write.rs` | modify | Add `#[cfg(test)] mod tests` with >= 3 tests per extracted function | AC-002.2 |
| 19 | `crates/ecc-workflow/src/io.rs` | modify | Replace `read_stdin()` with bounded version: `read_stdin_bounded(limit: usize)` using `Read::take(limit as u64 + 1)`, log::warn on truncation | AC-002.3, AC-002.3a, AC-002.3b |
| 20 | `crates/ecc-workflow/tests/integration.rs` | delete | Replaced by per-command test files | AC-002.4 |
| 21 | `crates/ecc-workflow/tests/common/mod.rs` | create | Shared helpers: `binary_path()`, `assert_structured_json_output()`, `valid_statuses()` | AC-002.4 |
| 22 | `crates/ecc-workflow/tests/init.rs` | create | Init command integration tests (< 400 lines) | AC-002.4 |
| 23 | `crates/ecc-workflow/tests/transition.rs` | create | Transition command integration tests (< 400 lines) | AC-002.4 |
| 24 | `crates/ecc-workflow/tests/memory_write.rs` | create | Memory-write integration tests (< 400 lines) | AC-002.4 |
| 25 | `crates/ecc-workflow/tests/phase_gate.rs` | create | Phase-gate integration tests (< 400 lines) | AC-002.4 |
| 26 | `crates/ecc-workflow/tests/status.rs` | create | Status/artifact/reset integration tests (< 400 lines) | AC-002.4 |
| 27 | `crates/ecc-workflow/tests/hooks.rs` | create | Hook-related commands integration tests (< 400 lines) | AC-002.4 |
| 28 | `crates/ecc-workflow/tests/backlog.rs` | create | Backlog integration tests (< 400 lines) | AC-002.4 |
| **US-003: Secure Notification Hooks** | | | | |
| 29 | `crates/ecc-app/src/hook/handlers/tier2_notify.rs` | modify | Add `sanitize_osascript(s: &str) -> String` and `sanitize_powershell(s: &str) -> String` functions; call before building notification commands | AC-003.1, AC-003.2, AC-003.3 |
| 30 | `crates/ecc-app/src/hook/handlers/tier2_notify.rs` | modify | Add >= 5 adversarial unit tests per platform in `#[cfg(test)]` | AC-003.4 |
| **US-007: File Size Compliance** | | | | |
| 31 | `crates/ecc-app/src/validate.rs` | delete | Replaced by validate/ module directory | AC-007.1 |
| 32 | `crates/ecc-app/src/validate/mod.rs` | create | Re-exports `ValidateTarget`, `run_validate`, dispatches to submodules | AC-007.1 |
| 33 | `crates/ecc-app/src/validate/agents.rs` | create | `validate_agents()` (< 400 lines) | AC-007.1 |
| 34 | `crates/ecc-app/src/validate/commands.rs` | create | `validate_commands()` (< 400 lines) | AC-007.1 |
| 35 | `crates/ecc-app/src/validate/hooks.rs` | create | `validate_hooks()` (< 400 lines) | AC-007.1 |
| 36 | `crates/ecc-app/src/validate/skills.rs` | create | `validate_skills()` (< 400 lines) | AC-007.1 |
| 37 | `crates/ecc-app/src/validate/rules.rs` | create | `validate_rules()` (< 400 lines) | AC-007.1 |
| 38 | `crates/ecc-app/src/validate/paths.rs` | create | `validate_paths()` (< 400 lines) | AC-007.1 |
| 39 | `crates/ecc-app/src/validate/statusline.rs` | create | `validate_statusline()` + tests (< 400 lines) | AC-007.1 |
| 40 | `crates/ecc-app/src/dev.rs` | delete | Replaced by dev/ module directory | AC-007.2 |
| 41 | `crates/ecc-app/src/dev/mod.rs` | create | Re-exports types, dispatches to submodules | AC-007.2 |
| 42 | `crates/ecc-app/src/dev/switch.rs` | create | `dev_switch()` + `DevSwitchResult` (< 400 lines) | AC-007.2 |
| 43 | `crates/ecc-app/src/dev/status.rs` | create | `dev_status()` + `DevStatus` (< 400 lines) | AC-007.2 |
| 44 | `crates/ecc-app/src/dev/toggle.rs` | create | `dev_on()`, `dev_off()` + `DevOffResult` (< 400 lines) | AC-007.2 |
| 45 | `crates/ecc-app/src/dev/format.rs` | create | `format_status_output()` + display helpers (< 400 lines) | AC-007.2 |
| 46 | `crates/ecc-app/src/merge/helpers.rs` | modify | Extract `#[cfg(test)] mod tests` into `crates/ecc-app/src/merge/helpers_test.rs` (or use `#[path]`), keeping production code < 500 lines | AC-007.3 |
| 47 | `crates/ecc-app/src/install/global.rs` | delete | Replaced by install/global/ module directory | AC-007.4 |
| 48 | `crates/ecc-app/src/install/global/mod.rs` | create | Re-exports `install_global`, orchestration entry point (< 400 lines) | AC-007.4 |
| 49 | `crates/ecc-app/src/install/global/steps.rs` | create | Individual step functions: `step_clean`, `step_detect`, `step_merge_artifacts`, `step_hooks_and_settings`, `step_write_manifest` (< 400 lines) | AC-007.4 |
| **US-004: Typed Error Handling in ecc-app** | | | | |
| 50 | `crates/ecc-app/src/claw/error.rs` | create | `ClawError` enum with variants: `NoHomeDir`, `SessionDirCreate`, `SessionSave`, `SessionLoad`, `SkillLoad`, `ClaudeRun`, `InputError`, `StorageError` | AC-004.1 |
| 51 | `crates/ecc-app/src/claw/mod.rs` | modify | Replace `anyhow::Result<()>` with `Result<(), ClawError>` in `run_repl` | AC-004.1 |
| 52 | `crates/ecc-app/src/claw/dispatch.rs` | modify | Replace `anyhow::Result<()>` with `Result<(), ClawError>` | AC-004.1 |
| 53 | `crates/ecc-app/src/claw/storage.rs` | modify | Replace `Result<(), String>` with `Result<(), ClawError>` | AC-004.1 |
| 54 | `crates/ecc-app/src/claw/skill_loader.rs` | modify | Replace `Result<String, String>` with `Result<String, ClawError>` | AC-004.1 |
| 55 | `crates/ecc-app/src/claw/claude_runner.rs` | modify | Replace `Result<String, String>` with `Result<String, ClawError>` | AC-004.1 |
| 56 | `crates/ecc-app/src/merge/error.rs` | create | `MergeError` enum with variants: `ReadJson`, `WriteJson`, `CopyDir`, `PromptFailed`, `MergeConflict` | AC-004.2 |
| 57 | `crates/ecc-app/src/merge/helpers.rs` | modify | Replace `Result<T, String>` with `Result<T, MergeError>` | AC-004.2 |
| 58 | `crates/ecc-app/src/merge/mod.rs` | modify | Replace `Result<T, String>` with `Result<T, MergeError>` in `merge_hooks_json` | AC-004.2 |
| 59 | `crates/ecc-app/src/merge/prompt.rs` | modify | Replace `Result<T, String>` with `Result<T, MergeError>` | AC-004.2 |
| 60 | `crates/ecc-app/src/config/error.rs` | create | `ConfigAppError` enum with variants: `AuditLoad`, `CleanFailed`, `MergeFailed` | AC-004.3 |
| 61 | `crates/ecc-app/src/config/audit/mod.rs` | modify | Replace `Result<T, String>` with `Result<T, ConfigAppError>` | AC-004.3 |
| 62 | `crates/ecc-app/src/config/clean.rs` | modify | Replace `Result<T, String>` with `Result<T, ConfigAppError>` | AC-004.3 |
| 63 | `crates/ecc-app/src/config/merge.rs` | modify | Replace `Result<T, String>` with `Result<T, ConfigAppError>` | AC-004.3 |
| 64 | `crates/ecc-app/src/install/error.rs` | create | `InstallError` enum with variants: `ResolveRoot`, `ManifestRead`, `ManifestWrite`, `StepFailed` | AC-004.4 |
| 65 | `crates/ecc-app/src/install/resolve.rs` | modify | Replace `Result<PathBuf, String>` with `Result<PathBuf, InstallError>` | AC-004.4 |
| 66 | `crates/ecc-app/src/hook/handlers/tier2_tools/helpers.rs` | modify | Replace `Result<(), String>` with typed error (inline or local) | AC-004.4 |
| 67 | `crates/ecc-app/Cargo.toml` | modify | Remove `anyhow = { workspace = true }` from `[dependencies]` | AC-004.5 |
| 68 | `crates/ecc-cli/src/commands/claw.rs` | modify | Map `ClawError` variants to user-friendly messages with operation name + remediation hint | AC-004.6 |
| **US-006: Convention Consistency** | | | | |
| 69 | `crates/ecc-domain/src/config/validate.rs` | modify | Replace manual `extract_frontmatter` with `serde_yaml::from_str` (benchmark first; retain manual if >3x slower on p95) | AC-006.1 |
| 70 | `crates/ecc-app/src/smart_merge.rs` | modify | Remove `is_claude_available`, import from `claw::claude_runner::is_claude_available` instead (adapting to take `&dyn ShellExecutor` param) | AC-006.2 |
| 71 | `crates/ecc-app/src/claw/claude_runner.rs` | modify | Add `pub fn is_claude_available_shell(shell: &dyn ShellExecutor) -> bool` as canonical implementation | AC-006.2 |
| 72 | `crates/ecc-domain/src/workflow/state.rs` | modify | Change `Completion.phase` from `String` to `Phase` enum; add `#[serde(deserialize_with = ...)]` for unknown-phase fallback | AC-006.3, AC-006.3a, AC-006.3b |
| 73 | `crates/ecc-domain/src/workflow/phase.rs` | modify | Add `Unknown` variant with custom serde (deserialize any unrecognized string -> Unknown, serialize -> "unknown") | AC-006.3a, AC-006.3b |
| 74 | `crates/ecc-domain/src/workflow/state.rs` | modify | Add `Concern` enum (`Dev`, `Fix`, `Refactor`) with serde; add `Timestamp` newtype wrapping `String` with validation | AC-006.4 |
| 75 | `crates/ecc-ports/src/fs.rs` | modify | Add `///` doc comments to every method on `FileSystem` trait | AC-006.5 |
| 76 | `crates/ecc-ports/src/shell.rs` | modify | Add `///` doc comments to every method on `ShellExecutor` trait | AC-006.5 |
| 77 | `crates/ecc-ports/src/env.rs` | modify | Add `///` doc comments to every method on `Environment` trait | AC-006.5 |
| 78 | `crates/ecc-ports/src/terminal.rs` | modify | Add `///` doc comments to every method on `TerminalIO` trait | AC-006.5 |
| **US-008: Domain Model Improvement** | | | | |
| 79 | `crates/ecc-domain/src/traits.rs` | create | `Validatable<E>` trait: `fn validate(&self) -> Result<(), Vec<E>>` | AC-008.1, AC-008.3 |
| 80 | `crates/ecc-domain/src/traits.rs` | create (same file) | `Transitionable` trait: `fn transition_to(&self, target: Phase) -> Result<Self, WorkflowError>` | AC-008.2, AC-008.3 |
| 81 | `crates/ecc-domain/src/lib.rs` | modify | Add `pub mod traits;` | AC-008.1, AC-008.2 |
| 82 | `crates/ecc-domain/src/config/validate.rs` | modify | Implement `Validatable` for frontmatter types (Agent, Hook, Skill config structs) | AC-008.1 |
| 83 | `crates/ecc-domain/src/workflow/state.rs` | modify | Implement `Transitionable` for `WorkflowState` delegating to existing `transition::try_transition` | AC-008.2 |
| 84 | `crates/ecc-app/src/session/aliases.rs` | modify | In `load_aliases`, on `Err(_)` from `serde_json::from_str`, emit `log::warn!("load_aliases: corrupt JSON at {}: {e}", path.display())` | AC-008.5 |
| 85 | `crates/ecc-app/src/dev/switch.rs` (or `dev.rs` pre-split) | modify | In rollback path, on `remove_file` failure, emit `log::error!("dev_switch rollback: failed to remove {}: {e}", path.display())` | AC-008.6 |
| **US-005: Accurate Documentation** | | | | |
| 86 | `docs/DEPENDENCY-GRAPH.md` | modify | Rewrite with 9 Rust crate nodes and correct edges | AC-005.1 |
| 87 | `docs/domain/glossary.md` | modify | Replace all 27 `.ts` references with `.rs` paths | AC-005.2 |
| 88 | `docs/ARCHITECTURE.md` | modify | Regenerate with actual counts: agents, commands, skills | AC-005.3 |
| 89 | `docs/domain/bounded-contexts.md` | modify | Add backlog and workflow bounded context entries | AC-005.4 |
| 90 | `docs/MODULE-SUMMARIES.md` | modify | Add ecc-integration-tests and ecc-workflow summaries | AC-005.5 |
| 91 | `docs/commands-reference.md` | modify | Add /spec, /spec-dev, /spec-fix, /spec-refactor, /design, /implement | AC-005.6 |
| 92 | `docs/diagrams/module-dependency-graph.md` | modify | Add all 9 crate nodes | AC-005.7 |
| 93 | `docs/getting-started.md` | modify | Update repo tree to show 9 crates, fix skill count | AC-005.8 |
| 94 | `docs/adr/NNN-workflow-direct-io.md` | create | ADR: ecc-workflow keeps direct I/O | Spec Decision 1 |
| 95 | `docs/adr/NNN-domain-traits.md` | create | ADR: Domain abstractness via behavioral traits | Spec Decision 2 |
| 96 | `docs/adr/NNN-error-type-strategy.md` | create | ADR: thiserror per module, anyhow in binaries only | Spec Decision 3 |
| 97 | `CLAUDE.md` | modify | Update test count after all new tests added | AC-005.0 |

---

## 2. Pass Conditions Table

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| **US-001: Observable CLI Operations** | | | | | |
| PC-001 | integration | Default ecc invocation emits warn! on stderr | AC-001.1 | `cd /Users/titouanlebocq/code/everything-claude-code && cargo test -p ecc-cli --test '*' -- --include-ignored warn_on_stderr 2>&1` | PASS |
| PC-002 | integration | `--verbose` flag produces debug-level output | AC-001.2 | `cd /Users/titouanlebocq/code/everything-claude-code && cargo build --release && ./target/release/ecc --verbose version 2>&1 \| grep -i debug` | exit 0 |
| PC-003 | integration | ecc-workflow with RUST_LOG=debug produces diagnostic output | AC-001.3 | `cd /Users/titouanlebocq/code/everything-claude-code && cargo build --release && RUST_LOG=debug CLAUDE_PROJECT_DIR=/tmp/nonexistent ./target/release/ecc-workflow status 2>&1 \| grep -iE 'debug\|WARN\|INFO'` | exit 0 |
| PC-004 | unit | Error-discard sites in ecc-app emit log::warn! (programmatic test counts bare Err(_) sites) | AC-001.4 | `cd /Users/titouanlebocq/code/everything-claude-code && cargo test -p ecc-app no_bare_error_discards` | PASS |
| PC-005 | integration | Failure banner appears on stderr before exit(1) | AC-001.5 | `cd /Users/titouanlebocq/code/everything-claude-code && cargo build --release && ./target/release/ecc install --ecc-root /nonexistent 2>&1 \| grep '^Error:'` | exit 0 |
| **US-002: Testable Workflow Commands** | | | | | |
| PC-006 | unit | Pure functions extracted from memory_write.rs are unit-testable | AC-002.1 | `cd /Users/titouanlebocq/code/everything-claude-code && cargo test -p ecc-workflow build_action_entry` | PASS |
| PC-007 | unit | Each extracted function has >= 3 test cases | AC-002.2 | `cd /Users/titouanlebocq/code/everything-claude-code && test $(cargo test -p ecc-workflow -- memory_write::tests 2>&1 \| grep -c 'test .* \.\.\. ok') -ge 12` | exit 0 |
| PC-008 | unit | stdin bounded at 1MB byte boundary (> 1MB truncated) | AC-002.3 | `cd /Users/titouanlebocq/code/everything-claude-code && cargo test -p ecc-workflow read_stdin_bounded_truncates` | PASS |
| PC-009 | unit | Exactly 1MB stdin returns full content without truncation | AC-002.3a | `cd /Users/titouanlebocq/code/everything-claude-code && cargo test -p ecc-workflow read_stdin_bounded_exact` | PASS |
| PC-010 | unit | Truncation logs warning with byte count | AC-002.3b | `cd /Users/titouanlebocq/code/everything-claude-code && cargo test -p ecc-workflow read_stdin_bounded_logs_truncation` | PASS |
| PC-011 | lint | Each split integration test file < 400 lines | AC-002.4 | `cd /Users/titouanlebocq/code/everything-claude-code && find crates/ecc-workflow/tests -name '*.rs' -not -path '*/common/*' -exec wc -l {} + \| awk '$1 > 400 && !/total/ {exit 1}'` | exit 0 |
| PC-012 | integration | ecc-workflow gets log + env_logger (RUST_LOG=debug works) | AC-002.5 | `cd /Users/titouanlebocq/code/everything-claude-code && grep 'log\b' crates/ecc-workflow/Cargo.toml && grep 'env_logger' crates/ecc-workflow/Cargo.toml` | exit 0 |
| **US-003: Secure Notification Hooks** | | | | | |
| PC-013 | unit | osascript builder escapes single and double quotes | AC-003.1 | `cd /Users/titouanlebocq/code/everything-claude-code && cargo test -p ecc-app sanitize_osascript_escapes_quotes` | PASS |
| PC-014 | unit | PowerShell builder escapes single quotes | AC-003.2 | `cd /Users/titouanlebocq/code/everything-claude-code && cargo test -p ecc-app sanitize_powershell_escapes_quotes` | PASS |
| PC-015 | unit | Adversarial injection input blocked | AC-003.3 | `cd /Users/titouanlebocq/code/everything-claude-code && cargo test -p ecc-app adversarial_injection_blocked` | PASS |
| PC-016 | unit | >= 5 adversarial inputs tested per platform | AC-003.4 | `cd /Users/titouanlebocq/code/everything-claude-code && test $(cargo test -p ecc-app -- tier2_notify::tests::adversarial 2>&1 \| grep -c 'test .* \.\.\. ok') -ge 10` | exit 0 |
| **US-007: File Size Compliance** | | | | | |
| PC-017 | lint | validate.rs split: each submodule < 400 lines | AC-007.1 | `cd /Users/titouanlebocq/code/everything-claude-code && find crates/ecc-app/src/validate -name '*.rs' -exec wc -l {} + \| awk '$1 > 400 && !/total/ {exit 1}'` | exit 0 |
| PC-018 | lint | dev.rs split: each submodule < 400 lines | AC-007.2 | `cd /Users/titouanlebocq/code/everything-claude-code && find crates/ecc-app/src/dev -name '*.rs' -exec wc -l {} + \| awk '$1 > 400 && !/total/ {exit 1}'` | exit 0 |
| PC-019 | lint | merge/helpers.rs production code < 500 lines | AC-007.3 | `cd /Users/titouanlebocq/code/everything-claude-code && wc -l crates/ecc-app/src/merge/helpers.rs \| awk '$1 > 500 {exit 1}'` | exit 0 |
| PC-020 | lint | install/global/ each file < 400 lines | AC-007.4 | `cd /Users/titouanlebocq/code/everything-claude-code && find crates/ecc-app/src/install/global -name '*.rs' -exec wc -l {} + \| awk '$1 > 400 && !/total/ {exit 1}'` | exit 0 |
| PC-021 | build | All existing tests pass after splits | AC-007.5 | `cd /Users/titouanlebocq/code/everything-claude-code && cargo test` | PASS |
| **US-004: Typed Error Handling** | | | | | |
| PC-022 | unit | ClawError enum covers all 8 functions in claw/ | AC-004.1 | `cd /Users/titouanlebocq/code/everything-claude-code && cargo test -p ecc-app claw::error` | PASS |
| PC-023 | unit | MergeError enum covers all 4 functions in merge/ | AC-004.2 | `cd /Users/titouanlebocq/code/everything-claude-code && cargo test -p ecc-app merge::error` | PASS |
| PC-024 | unit | ConfigAppError enum covers all 3 functions in config/ | AC-004.3 | `cd /Users/titouanlebocq/code/everything-claude-code && cargo test -p ecc-app config::error` | PASS |
| PC-025 | unit | InstallError + hook/helpers typed errors | AC-004.4 | `cd /Users/titouanlebocq/code/everything-claude-code && cargo test -p ecc-app install::error` | PASS |
| PC-026 | build | anyhow removed from ecc-app/Cargo.toml | AC-004.5 | `cd /Users/titouanlebocq/code/everything-claude-code && ! grep 'anyhow' crates/ecc-app/Cargo.toml` | exit 0 |
| PC-027 | unit | Error messages contain operation name + remediation hint | AC-004.6 | `cd /Users/titouanlebocq/code/everything-claude-code && cargo test -p ecc-cli error_message_format` | PASS |
| PC-028 | build | cargo check passes after each module migration | AC-004.7 | `cd /Users/titouanlebocq/code/everything-claude-code && cargo check` | exit 0 |
| **US-006: Convention Consistency** | | | | | |
| PC-029 | unit | Frontmatter parsing uses serde_yaml (or manual if >3x slower) | AC-006.1 | `cd /Users/titouanlebocq/code/everything-claude-code && cargo test -p ecc-domain extract_frontmatter` | PASS |
| PC-030 | lint | Single canonical is_claude_available (no duplicates) | AC-006.2 | `cd /Users/titouanlebocq/code/everything-claude-code && test $(grep -rn 'pub fn is_claude_available' crates/ecc-app/src --include='*.rs' \| wc -l) -eq 1` | exit 0 |
| PC-031 | unit | Completion.phase is Phase enum, serializes as lowercase string | AC-006.3, AC-006.3b | `cd /Users/titouanlebocq/code/everything-claude-code && cargo test -p ecc-domain completion_phase_is_typed` | PASS |
| PC-032 | unit | Unknown phase string deserializes to Unknown variant + warning | AC-006.3a | `cd /Users/titouanlebocq/code/everything-claude-code && cargo test -p ecc-domain unknown_phase_fallback` | PASS |
| PC-033 | unit | Phase serializes backward-compatible (plain lowercase string) | AC-006.3b | `cd /Users/titouanlebocq/code/everything-claude-code && cargo test -p ecc-domain phase_backward_compat` | PASS |
| PC-034 | unit | WorkflowState concern/started_at typed with backward-compat serde | AC-006.4 | `cd /Users/titouanlebocq/code/everything-claude-code && cargo test -p ecc-domain concern_enum_roundtrip` | PASS |
| PC-035 | lint | All port trait methods have /// doc comments | AC-006.5 | `cd /Users/titouanlebocq/code/everything-claude-code && cargo doc -p ecc-ports --no-deps 2>&1 \| grep -c 'missing documentation' \| xargs test 0 -eq` | exit 0 |
| **US-008: Domain Model Improvement** | | | | | |
| PC-036 | unit | Validatable trait added, config types implement it | AC-008.1 | `cd /Users/titouanlebocq/code/everything-claude-code && cargo test -p ecc-domain validatable_impl` | PASS |
| PC-037 | unit | Transitionable trait added, WorkflowState implements it | AC-008.2 | `cd /Users/titouanlebocq/code/everything-claude-code && cargo test -p ecc-domain transitionable_impl` | PASS |
| PC-038 | build | Traits use generics (no dyn dispatch) - compile-time check | AC-008.3 | `cd /Users/titouanlebocq/code/everything-claude-code && ! grep -rn 'dyn Validatable\|dyn Transitionable' crates/ecc-domain/src --include='*.rs' \| head -1` | exit 0 |
| PC-039 | unit | D < 0.80 computed from public items | AC-008.4 | `cd /Users/titouanlebocq/code/everything-claude-code && cargo test -p ecc-domain domain_abstractness_score` | PASS |
| PC-040 | unit | Corrupt aliases.json emits log::warn! | AC-008.5 | `cd /Users/titouanlebocq/code/everything-claude-code && cargo test -p ecc-app corrupt_aliases_warns` | PASS |
| PC-041 | unit | dev_switch rollback emits log::error! on remove_file failure | AC-008.6 | `cd /Users/titouanlebocq/code/everything-claude-code && cargo test -p ecc-app dev_switch_rollback_logs_error` | PASS |
| **US-005: Accurate Documentation** | | | | | |
| PC-042 | lint | DEPENDENCY-GRAPH.md contains all 9 crate names | AC-005.1 | `cd /Users/titouanlebocq/code/everything-claude-code && for c in ecc-domain ecc-ports ecc-infra ecc-app ecc-cli ecc-workflow ecc-flock ecc-test-support ecc-integration-tests; do grep -q "$c" docs/DEPENDENCY-GRAPH.md \|\| exit 1; done` | exit 0 |
| PC-043 | lint | glossary.md has zero .ts references | AC-005.2 | `cd /Users/titouanlebocq/code/everything-claude-code && ! grep -n '\.ts\b' docs/domain/glossary.md \| head -1` | exit 0 |
| PC-044 | lint | ARCHITECTURE.md agent/command/skill counts match actual | AC-005.3 | `cd /Users/titouanlebocq/code/everything-claude-code && AGENTS=$(find agents -maxdepth 1 -name '*.md' ! -name 'README*' \| wc -l \| tr -d ' ') && grep -q "$AGENTS" docs/ARCHITECTURE.md` | exit 0 |
| PC-045 | lint | bounded-contexts.md mentions backlog and workflow | AC-005.4 | `cd /Users/titouanlebocq/code/everything-claude-code && grep -q 'backlog' docs/domain/bounded-contexts.md && grep -q 'workflow' docs/domain/bounded-contexts.md` | exit 0 |
| PC-046 | lint | MODULE-SUMMARIES.md mentions ecc-integration-tests and ecc-workflow | AC-005.5 | `cd /Users/titouanlebocq/code/everything-claude-code && grep -q 'ecc-integration-tests' docs/MODULE-SUMMARIES.md && grep -q 'ecc-workflow' docs/MODULE-SUMMARIES.md` | exit 0 |
| PC-047 | lint | commands-reference.md lists 6 spec pipeline commands | AC-005.6 | `cd /Users/titouanlebocq/code/everything-claude-code && for cmd in '/spec' '/spec-dev' '/spec-fix' '/spec-refactor' '/design' '/implement'; do grep -q "$cmd" docs/commands-reference.md \|\| exit 1; done` | exit 0 |
| PC-048 | lint | Module dependency diagram has 9 nodes | AC-005.7 | `cd /Users/titouanlebocq/code/everything-claude-code && for c in ecc-domain ecc-ports ecc-infra ecc-app ecc-cli ecc-workflow ecc-flock ecc-test-support ecc-integration-tests; do grep -q "$c" docs/diagrams/module-dependency-graph.md \|\| exit 1; done` | exit 0 |
| PC-049 | lint | getting-started.md shows 9 crates in repo tree | AC-005.8 | `cd /Users/titouanlebocq/code/everything-claude-code && grep -c 'ecc-' docs/getting-started.md \| xargs test 9 -le` | exit 0 |
| PC-050 | lint | Docs verified after all code stories (no .ts refs in glossary, counts match) | AC-005.0 | `cd /Users/titouanlebocq/code/everything-claude-code && ! grep -rn '\.ts\b' docs/domain/glossary.md` | exit 0 |
| **Cross-cutting gates** | | | | | |
| PC-051 | lint | Zero clippy warnings | all | `cd /Users/titouanlebocq/code/everything-claude-code && cargo clippy -- -D warnings` | exit 0 |
| PC-052 | build | Release build succeeds | all | `cd /Users/titouanlebocq/code/everything-claude-code && cargo build --release` | exit 0 |
| PC-053 | build | Full test suite passes | all | `cd /Users/titouanlebocq/code/everything-claude-code && cargo test` | PASS |
| PC-054 | unit | Phase::Unknown rejected by transition logic | AC-006.3a, Security | `cd /Users/titouanlebocq/code/everything-claude-code && cargo test -p ecc-domain unknown_phase_transition_rejected` | PASS |
| PC-055 | unit | sanitize_osascript escapes backslashes + caps length at 256 | AC-003.1, Security | `cd /Users/titouanlebocq/code/everything-claude-code && cargo test -p ecc-app sanitize_osascript_backslash_and_length` | PASS |

---

## 3. TDD Order

The order follows spec dependency graph: US-001/002/003 parallel -> US-007 -> US-004/006/008 parallel -> US-005.

| Order | PC IDs | Story | Rationale |
|-------|--------|-------|-----------|
| 1 | PC-001..PC-005 | US-001 | Logging infrastructure required by US-008 (AC-008.5/6). No deps. |
| 2 | PC-006..PC-012 | US-002 | Pure extraction + test file split. No deps. Can parallel with US-001/003. |
| 3 | PC-013..PC-016 | US-003 | Security fix, self-contained. No deps. Can parallel with US-001/002. |
| 4 | PC-017..PC-021 | US-007 | File splits. Must complete before US-004/006 to reduce merge conflicts. |
| 5 | PC-022..PC-028 | US-004 | Typed errors. Depends on US-007 (split files). Module-by-module with cargo check gate. |
| 6 | PC-029..PC-035 | US-006 | Convention fixes. Depends on US-004 (error types may affect validate.rs) and US-007 (splits). |
| 7 | PC-036..PC-041 | US-008 | Domain traits + corrupt JSON warning. Depends on US-001 (logging). |
| 8 | PC-042..PC-050 | US-005 | Documentation. Must reflect final code state. Last. |
| 9 | PC-051..PC-053 | Cross-cut | Final gates: clippy, build, full test suite. |

### Within-story TDD sequence

**US-001 (5 phases):**
1. RED: Test default warn visibility -> GREEN: init env_logger with default_filter_or("warn") -> REFACTOR
2. RED: Test --verbose debug output -> GREEN: already wired (verify) -> REFACTOR
3. RED: Test ecc-workflow RUST_LOG -> GREEN: add env_logger to ecc-workflow -> REFACTOR
4. Scan ecc-app for `Err(_) =>` patterns, add `log::warn!` (batch, test via grep PC-004)
5. RED: Test failure banner -> GREEN: add eprintln!("Error: ...") before exit(1) -> REFACTOR

**US-002 (5 phases):**
1. RED: Tests for `build_action_entry`, `build_work_item_content`, `build_daily_content`, `build_memory_index_content` -> GREEN: extract pure functions -> REFACTOR
2. RED: Tests for `read_stdin_bounded` (truncation, exact, logging) -> GREEN: implement bounded reader -> REFACTOR
3. Split integration.rs into per-command files + common/mod.rs (structural, run full test suite to verify)
4. Verify env_logger in ecc-workflow (covered by US-001 phase 3)

**US-003 (2 phases):**
1. RED: Tests for `sanitize_osascript` and `sanitize_powershell` with >= 5 adversarial inputs each -> GREEN: implement sanitization -> REFACTOR
2. RED: Integration test verifying end-to-end sanitization in `send_notification` -> GREEN: wire sanitizers into notification builder -> REFACTOR

**US-007 (4 phases, structural — test suite is the safety net):**
1. Split validate.rs -> validate/{mod,agents,commands,hooks,skills,rules,paths,statusline}.rs -> cargo test
2. Split dev.rs -> dev/{mod,switch,status,toggle,format}.rs -> cargo test
3. Extract merge/helpers.rs test code -> cargo test
4. Split install/global.rs -> install/global/{mod,steps}.rs -> cargo test

**US-004 (4 phases, one per module + final cleanup):**
1. RED: ClawError enum tests -> GREEN: create claw/error.rs, migrate claw/ -> cargo check -> REFACTOR
2. RED: MergeError enum tests -> GREEN: create merge/error.rs, migrate merge/ -> cargo check -> REFACTOR
3. RED: ConfigAppError enum tests -> GREEN: create config/error.rs, migrate config/ -> cargo check -> REFACTOR
4. RED: InstallError enum tests -> GREEN: create install/error.rs, migrate install/ + hook/helpers -> cargo check -> Remove anyhow from Cargo.toml -> REFACTOR

**US-006 (5 phases):**
1. Benchmark frontmatter parsers -> decide -> RED: test serde_yaml-based extract_frontmatter -> GREEN: implement -> REFACTOR
2. RED: test canonical is_claude_available -> GREEN: deduplicate -> REFACTOR
3. RED: test Phase with Unknown variant, Completion.phase typed -> GREEN: implement -> REFACTOR
4. RED: test Concern enum + Timestamp newtype -> GREEN: implement -> REFACTOR
5. Add /// doc comments to port traits -> cargo doc check

**US-008 (4 phases):**
1. RED: test Validatable trait + impls -> GREEN: implement trait, impl for config types -> REFACTOR
2. RED: test Transitionable trait + WorkflowState impl -> GREEN: implement -> REFACTOR
3. RED: test corrupt aliases.json warning -> GREEN: add log::warn! in load_aliases -> REFACTOR
4. RED: test dev_switch rollback error logging -> GREEN: add log::error! -> REFACTOR

**US-005 (1 phase, docs only):**
1. Rewrite/update all 8 doc files + create 3 ADRs + update CLAUDE.md test count
2. Verify all PC-042..PC-050

---

## Module Structure for File Splits

### validate/ (from validate.rs, 1240 lines)
```
crates/ecc-app/src/validate/
  mod.rs           — ValidateTarget enum, run_validate dispatch (~40 lines)
  agents.rs        — validate_agents + tests (~200 lines)
  commands.rs      — validate_commands + tests (~150 lines)
  hooks.rs         — validate_hooks + tests (~250 lines)
  skills.rs        — validate_skills + tests (~150 lines)
  rules.rs         — validate_rules + tests (~100 lines)
  paths.rs         — validate_paths + tests (~100 lines)
  statusline.rs    — validate_statusline + tests (~250 lines)
```

### dev/ (from dev.rs, 1065 lines)
```
crates/ecc-app/src/dev/
  mod.rs           — Re-exports types + public API (~60 lines)
  switch.rs        — dev_switch, DevSwitchResult, DevError (~350 lines)
  status.rs        — dev_status, DevStatus, DevProfileStatus (~200 lines)
  toggle.rs        — dev_on, dev_off, DevOffResult (~250 lines)
  format.rs        — format_status_output, display helpers (~200 lines)
```

### install/global/ (from install/global.rs, 863 lines)
```
crates/ecc-app/src/install/global/
  mod.rs           — install_global entry point, re-exports (~200 lines)
  steps.rs         — step_clean, step_detect, step_merge_artifacts, step_hooks_and_settings, step_write_manifest (~350 lines)
```

### ecc-workflow/tests/ (from integration.rs, 2750 lines)
```
crates/ecc-workflow/tests/
  common/mod.rs    — binary_path, assert_structured_json_output, valid_statuses (~60 lines)
  init.rs          — init command tests (~300 lines)
  transition.rs    — transition command tests (~400 lines)
  memory_write.rs  — memory-write tests (~350 lines)
  phase_gate.rs    — phase-gate tests (~350 lines)
  status.rs        — status/artifact/reset tests (~350 lines)
  hooks.rs         — stop-gate, grill-me-gate, tdd-enforcement, scope-check, doc-enforcement, doc-level-check tests (~400 lines)
  backlog.rs       — backlog add-entry tests (~200 lines)
```

---

## E2E Test Plan

| # | Boundary | Adapter | Port | Test Description | Default State | Run When |
|---|----------|---------|------|------------------|---------------|----------|
| 1 | stderr output | StdTerminal | TerminalIO | Verify warn! output appears on stderr during install | ignored | env_logger default changed (US-001) |
| 2 | stderr failure banner | StdTerminal | TerminalIO | Verify `Error: <desc>` appears before exit(1) | ignored | CLI error handling changed (US-001) |
| 3 | osascript command | ProcessExecutor | ShellExecutor | Verify notification with special chars in title is safe | ignored | tier2_notify.rs modified (US-003) |
| 4 | stdin pipe | raw stdin | N/A (ecc-workflow) | Verify >1MB payload is truncated | ignored | io.rs modified (US-002) |

### E2E Activation Rules

All 4 E2E tests un-ignored for this implementation (all boundaries touched).

---

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | docs/DEPENDENCY-GRAPH.md | Major | Rewrite | 9-crate Mermaid graph with correct edges | AC-005.1 |
| 2 | docs/domain/glossary.md | Major | Update | Replace 27 .ts refs with .rs paths | AC-005.2 |
| 3 | docs/ARCHITECTURE.md | Major | Regenerate | Update agent/command/skill counts | AC-005.3 |
| 4 | docs/domain/bounded-contexts.md | Medium | Add entries | backlog + workflow modules | AC-005.4 |
| 5 | docs/MODULE-SUMMARIES.md | Medium | Add entries | ecc-integration-tests + ecc-workflow | AC-005.5 |
| 6 | docs/commands-reference.md | Medium | Add entries | 6 spec pipeline commands | AC-005.6 |
| 7 | docs/diagrams/module-dependency-graph.md | Medium | Update | 9 crate nodes | AC-005.7 |
| 8 | docs/getting-started.md | Medium | Update | Repo tree + skill count | AC-005.8 |
| 9 | docs/adr/NNN-workflow-direct-io.md | Minor | Create | ADR: ecc-workflow keeps direct I/O | Decision 1 |
| 10 | docs/adr/NNN-domain-traits.md | Minor | Create | ADR: Domain abstractness via traits | Decision 2 |
| 11 | docs/adr/NNN-error-type-strategy.md | Minor | Create | ADR: thiserror per module, anyhow in binaries | Decision 3 |
| 12 | CLAUDE.md | Minor | Update | Test count after new tests | AC-005.0 |
| 13 | CHANGELOG.md | Minor | Add entry | Audit remediation: 24 smells, 8 stories | — |

---

## SOLID Assessment

**Verdict: CLEAN** (uncle-bob)

3 MEDIUM prescriptions incorporated into design:
1. Rename `is_claude_available_shell` -> `is_claude_available` in `claw::claude_runner` (drop `_shell` suffix)
2. `Completion.phase` custom serde deserializer must delegate to `Phase::from_str` for aliases ("spec" -> Plan, "design" -> Solution), falling back to `Unknown` only on unrecognized strings
3. Extract `Concern` to `workflow/concern.rs` and `Timestamp` to `workflow/timestamp.rs` (following `phase.rs` precedent)

1 LOW noted: `sanitize_osascript`/`sanitize_powershell` placement in ecc-app is pragmatic and acceptable.

---

## Robert's Oath Check

**Verdict: CLEAN** — 0 warnings, 0 self-audit findings, rework ratio 0.02.

All 9 applicable oaths satisfied: no harmful code (Oath 1), no mess (Oath 2), proof via 53 PCs (Oath 3), small releases via 8 independent stories (Oath 4), fearless improvement addressing all 24 smells (Oath 5), productivity via benchmark gating (Oath 6), easy substitution via typed errors + traits (Oath 7).

---

## Security Notes

**Verdict: CLEAR** — 0 CRITICAL/HIGH findings.

2 implementation notes:
1. **(MEDIUM)** `sanitize_osascript` must escape backslashes before quotes and cap input length at 256 chars. AC-003.1 wording "escapes quotes" is incomplete — backslash escaping is also required.
2. **(LOW)** `Phase::Unknown` must be explicitly rejected by `Transitionable` / transition logic to prevent phase-gate bypass via crafted state.json.

---

## Rollback Plan

Reverse dependency order of File Changes — if implementation fails, undo in this order:

1. **US-005 (docs)**: Git revert doc files. Zero code impact.
2. **US-008 (domain traits)**: Remove trait impls, then trait definitions, then mod declaration. Additive-only, safe revert.
3. **US-006 (conventions)**: Revert Concern/Timestamp/Phase typing (state.json backward-compat ensures no data loss). Revert frontmatter parser. Re-add duplicate is_claude_available.
4. **US-004 (error types)**: Revert per-module (install -> config -> merge -> claw). Re-add anyhow to Cargo.toml. Each module independently revertable.
5. **US-007 (file splits)**: Recombine submodules into original files. Structural only — no behavioral revert needed. **Permanent improvements; prefer not to revert.**
6. **US-003 (security)**: Remove sanitization functions and tests. Revert to original format! interpolation (re-introduces vulnerability).
7. **US-002 (workflow testing)**: Recombine test files into integration.rs. Remove pure function tests (keep extracted pure functions as they are strictly better).
8. **US-001 (observability)**: Revert env_logger default. Remove log::warn! calls. Revert failure banners.

**Key principle**: File splits (US-007) are permanent structural improvements — they are NOT reverted even if downstream stories fail.

**Blast radius note**: ecc-integration-tests exercises the CLI binary (not the library), so US-004 error type changes do not affect it. Verified: ecc-integration-tests/Cargo.toml depends on no workspace crates — it spawns the ecc binary as a subprocess.

**US-004 rollback addendum**: Also remove thiserror from ecc-app/Cargo.toml if reverting.

---

## Phase Summary

### Design Reviews

| Review Type | Verdict | Finding Count |
|-------------|---------|---------------|
| SOLID (uncle-bob) | CLEAN | 3 MEDIUM, 1 LOW (all incorporated) |
| Robert | CLEAN | 0 warnings |
| Security | CLEAR | 2 implementation notes (incorporated as PC-054, PC-055) |

### Adversary Findings

| Dimension | Score | Verdict | Key Rationale |
|-----------|-------|---------|---------------|
| AC Coverage | 95 | PASS | All 50 ACs covered by 55 PCs |
| Execution Order | 82 | PASS | Dependency graph respected |
| Fragility | 72 | PASS | Brittle grep PCs replaced with programmatic tests |
| Rollback | 75 | PASS | Per-US rollback + thiserror removal addendum |
| Architecture | 85 | PASS | Hexagonal boundaries maintained |
| Blast Radius | 75 | PASS | ecc-integration-tests confirmed subprocess-only |
| Missing PCs | 90 | PASS | Security mitigation PCs added (PC-054, PC-055) |
| Doc Plan | 80 | PASS | CHANGELOG + 3 ADRs + all doc updates |

### File Changes Summary

| # | File | Action | Spec Ref |
|---|------|--------|----------|
| 1-16 | ecc-cli/main.rs, ecc-workflow/*, ecc-app error-discard sites | modify | US-001 (observability) |
| 17-28 | ecc-workflow/commands/memory_write.rs, io.rs, tests/* | modify/create | US-002 (testable workflow) |
| 29-30 | ecc-app/hook/handlers/tier2_notify.rs | modify | US-003 (secure notifications) |
| 31-49 | ecc-app/validate/*, dev/*, merge/helpers.rs, install/global/* | create/modify/delete | US-007 (file splits) |
| 50-68 | ecc-app/claw/error.rs, merge/error.rs, config/error.rs, install/error.rs + callers | create/modify | US-004 (typed errors) |
| 69-78 | ecc-domain/config/validate.rs, workflow/state.rs, ecc-ports/src/*.rs | modify | US-006 (conventions) |
| 79-85 | ecc-domain/traits.rs, workflow/state.rs, ecc-app/session/aliases.rs, dev/switch.rs | create/modify | US-008 (domain model) |
| 86-97 | docs/*, CLAUDE.md, 3 ADRs | create/modify | US-005 (documentation) |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-03-28-audit-findings-remediation/spec.md | Full spec + Phase Summary |
| docs/specs/2026-03-28-audit-findings-remediation/design.md | Full design + Phase Summary |
