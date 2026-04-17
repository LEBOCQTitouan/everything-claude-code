# Solution: Fix shell-eval injection in slash-command templates

## Spec Reference
Concern: `fix`, Feature: `spec command shell escaping bug`
Spec: `docs/specs/2026-04-17-spec-command-shell-escaping/spec.md`

## File Changes (dependency order)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `crates/ecc-workflow/src/commands/feature_input.rs` | create | New module: `read_feature_from_stdin<R: Read>(reader, is_tty) -> Result<String, FeatureInputError>` free function; `thiserror` enum (`Empty`, `InvalidUtf8`, `TooLarge`, `IsTty`, `Io`). Each variant's `Display` emits pinned diagnostic strings. **TTY check FIRST** before any read (Security REQUIRED). `Read::take(65537)` then `len > 65536` check. `Io(_)` wraps `std::io::Error` with fixed `"stdin read error: …"` prefix so AC assertions don't flake. Module-level `//!` docs explain four guards and ordering invariant. | AC-001.3/9/10/11/12 |
| 2 | `crates/ecc-workflow/src/commands/mod.rs` | modify | `pub mod feature_input;` | scaffolding |
| 3 | `crates/ecc-workflow/src/main.rs` | modify | Add `--feature-stdin` flag via clap `ArgGroup` on `Init` and `WorktreeName`; `feature: Option<String>`; dispatches `FeatureSource::{Positional(String), Stdin}`. `stdin_is_tty()` helper uses `std::io::IsTerminal::is_terminal(&io::stdin())` — zero `unsafe`, zero `libc`. Flag includes `help = "Read feature from stdin. Limits: 64KB, UTF-8, rejects TTY, strips single trailing LF."`. | AC-001.3/4/5 |
| 4 | `crates/ecc-workflow/src/commands/init.rs` | modify | Accept `FeatureSource`; `Positional(s)` → existing path; `Stdin` → `feature_input::read_feature_from_stdin(io::stdin().lock(), stdin_is_tty())`. Map errors to `WorkflowOutput::block`. No state write on any error path. | AC-001.2a/2b, 3, 9, 10, 11, 12 |
| 5 | `crates/ecc-workflow/src/commands/worktree_name.rs` | modify | Same `FeatureSource` branching; feeds `WorktreeName::generate` untouched (Constraint 4). | AC-001.2a |
| 6 | `crates/ecc-cli/src/commands/workflow.rs` | modify | Mirror `--feature-stdin` on `WorkflowCommand::Init`/`::WorktreeName`; `build_args` forwards flag; existing `Stdio::inherit()` streams stdin to child without buffering (CRLF, NUL preserved). Same `help` string. | AC-001.6 |
| 7 | `crates/ecc-app/src/validate/commands.rs` | modify | Add per-line scan via `regex::Regex::new(r"^[[:space:]]*!.*\$ARGUMENTS")` (POSIX-ERE-compatible, matches the spec-pinned `VALIDATE_REGEX` verbatim) compiled once through `std::sync::LazyLock`. Use `content.lines()` (handles `\n` and `\r\n`). Emits `ERROR: <path>:<lineno>: …`; sets `has_errors`. Added unit tests (fixture-positive, prose-negative, fenced-code conservative, CRLF line numbers). | AC-001.1a, 7 |
| 8 | `commands/spec-dev.md` | modify | Replace L14 `!`-prefix init and L19 `!`-prefix worktree-name with prose: *"Use the Bash tool with an `sh -c` command that passes the feature via env-var + stdin (`env FEATURE_PAYLOAD='<feature>' sh -c 'printf %s "$FEATURE_PAYLOAD" \| ecc-workflow init dev --feature-stdin'`) — do NOT interpolate the feature text directly into the command string."* | AC-001.1a/1b |
| 9 | `commands/spec-fix.md` | modify | Same substitution (concern = `fix`). | AC-001.1a/1b |
| 10 | `commands/spec-refactor.md` | modify | Same substitution (concern = `refactor`). | AC-001.1a/1b |
| 11 | `commands/project-foundation.md` | modify | Same substitution at L18 (concern = `foundation`). | AC-001.1a/1b |
| 12 | `crates/ecc-workflow/tests/init.rs` | modify | Append the integration PC tests per the TDD table. Existing tests untouched (AC-001.5). Pin `SAFE_REPRO_PAYLOAD: &str = "feat-\"unterminated"` in a shared test constant — MUST NOT contain executable metacharacters. | AC-001.2a, 2b, 3, 4, 8, 9, 10, 11, 12 |
| 13 | `crates/ecc-workflow/tests/worktree_name.rs` | create | Parallel test matrix for `worktree-name`. | AC-001.2a, 11 |
| 14 | `crates/ecc-workflow/tests/proptest-regressions/` | create (dir) | Created on first failure by proptest; not pre-committed (empty committed file would be lint noise). | AC-001.8 |
| 15 | `crates/ecc-integration-tests/tests/workflow_cli_parity.rs` | modify | Append `feature_stdin_parity_{exit_code,state_bytes,stderr_class}` + `init_stdin_property_round_trip_via_delegator`. | AC-001.6, 8 |
| 16 | `crates/ecc-workflow/Cargo.toml` | modify | Add `[dev-dependencies]`: `proptest = "1"`, `rexpect = "0.5"` (narrow PTY dep for PC-020 real-TTY integration test). Existing `tempfile` + workspace `regex`/`std::sync::LazyLock` suffice. **No `libc`, no `portable-pty`, no `rustix`.** | enables AC-001.8, 11 |

**Pre-implementation step 0**: Run `rg -n 'ecc-workflow init' hooks/` to confirm no hook shells out with interpolated feature. Spec-adversary confirmed clean earlier; re-verify at implementation time.

**MSRV check**: `std::io::IsTerminal` requires Rust 1.70+. Verify `rust-toolchain.toml` MSRV (project currently at Rust 2024 edition ≥1.85; safe).

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit (lint) | `commands_validate_rule_detects_dollar_arguments_fixture` | AC-001.7 | `cargo test --package ecc-app --lib validate::commands::tests::commands_validate_rule_detects_dollar_arguments_fixture -- --exact` | PASS |
| PC-002 | unit (lint) | `commands_validate_rule_ignores_prose_dollar_arguments` | AC-001.7 | `cargo test --package ecc-app --lib validate::commands::tests::commands_validate_rule_ignores_prose_dollar_arguments -- --exact` | PASS |
| PC-003 | unit | `feature_input_strips_single_trailing_lf` — `b"foo\n"` → `Ok("foo")` | AC-001.12 | `cargo test --package ecc-workflow --lib commands::feature_input::tests::feature_input_strips_single_trailing_lf -- --exact` | PASS |
| PC-004 | unit | `feature_input_preserves_second_lf` — `b"foo\n\n"` → `Ok("foo\n")` | AC-001.12 | `cargo test --package ecc-workflow --lib commands::feature_input::tests::feature_input_preserves_second_lf -- --exact` | PASS |
| PC-005 | unit | `feature_input_preserves_cr_before_lf` — `b"foo\r\n"` → `Ok("foo\r")` | AC-001.12 | `cargo test --package ecc-workflow --lib commands::feature_input::tests::feature_input_preserves_cr_before_lf -- --exact` | PASS |
| PC-006 | unit | `feature_input_rejects_empty_after_strip` → `feature is empty` | AC-001.3 | `cargo test --package ecc-workflow --lib commands::feature_input::tests::feature_input_rejects_empty_after_strip -- --exact` | PASS |
| PC-007 | unit | `feature_input_rejects_invalid_utf8` → `invalid UTF-8 on stdin` | AC-001.9 | `cargo test --package ecc-workflow --lib commands::feature_input::tests::feature_input_rejects_invalid_utf8 -- --exact` | PASS |
| PC-008 | unit | `feature_input_accepts_64kb_exactly` — 65_536 bytes OK | AC-001.10 | `cargo test --package ecc-workflow --lib commands::feature_input::tests::feature_input_accepts_64kb_exactly -- --exact` | PASS |
| PC-009 | unit | `feature_input_rejects_64kb_plus_one` — 65_537 bytes → `stdin exceeds 64KB limit` | AC-001.10 | `cargo test --package ecc-workflow --lib commands::feature_input::tests::feature_input_rejects_64kb_plus_one -- --exact` | PASS |
| PC-010 | unit | `feature_input_rejects_tty_before_any_read` — counting-`Read` impl asserts zero `read()` calls when `is_tty=true` | AC-001.11 | `cargo test --package ecc-workflow --lib commands::feature_input::tests::feature_input_rejects_tty_before_any_read -- --exact` | PASS |
| PC-011 | unit | `feature_input_preserves_metachars_control_and_nul` — byte-preservation across METACHAR ∪ CONTROL ∪ {NUL} | AC-001.2a | `cargo test --package ecc-workflow --lib commands::feature_input::tests::feature_input_preserves_metachars_control_and_nul -- --exact` | PASS |
| PC-012 | integration | `init_positional_round_trips_metachars` via argv with canonical payload (NUL excluded) | AC-001.2b | `cargo test --package ecc-workflow --test init init_positional_round_trips_metachars -- --exact` | PASS |
| PC-013 | integration | `init_stdin_round_trips_metachars_and_nul` via piped stdin | AC-001.2a | `cargo test --package ecc-workflow --test init init_stdin_round_trips_metachars_and_nul -- --exact` | PASS |
| PC-014 | integration | `init_empty_stdin_exits_two` → `feature is empty`; no state.json | AC-001.3 | `cargo test --package ecc-workflow --test init init_empty_stdin_exits_two -- --exact` | PASS |
| PC-015 | integration | `init_stdin_and_positional_exits_two` → conflict diagnostic | AC-001.4 | `cargo test --package ecc-workflow --test init init_stdin_and_positional_exits_two -- --exact` | PASS |
| PC-016 | integration | `init_stdin_invalid_utf8_rejected_no_state` — no state.json file after failure | AC-001.9 | `cargo test --package ecc-workflow --test init init_stdin_invalid_utf8_rejected_no_state -- --exact` | PASS |
| PC-017 | integration | `init_stdin_65537_bytes_rejected` — no partial write | AC-001.10 | `cargo test --package ecc-workflow --test init init_stdin_65537_bytes_rejected -- --exact` | PASS |
| PC-018 | integration | `init_stdin_65536_bytes_accepted` — exact cap succeeds | AC-001.10 | `cargo test --package ecc-workflow --test init init_stdin_65536_bytes_accepted -- --exact` | PASS |
| PC-019 | integration | `init_stdin_trailing_newline_policy` — three sub-assertions | AC-001.12 | `cargo test --package ecc-workflow --test init init_stdin_trailing_newline_policy -- --exact` | PASS |
| PC-020 | integration | `init_stdin_tty_rejected_within_100ms_median_of_5` — `rexpect`-backed PTY, 5 back-to-back trials, PASS if ≥3 of 5 jointly satisfy (exit_code != 0) AND (elapsed < 100ms) | AC-001.11 | `cargo test --package ecc-workflow --test init init_stdin_tty_rejected_within_100ms_median_of_5 -- --exact` | PASS |
| PC-021 | integration (prop) | `init_stdin_property_round_trip` — `proptest` 1024 cases, ≤4KB UTF-8, regex `(?s-u:\\PC|\\s){0,4096}` | AC-001.8 | `cargo test --package ecc-workflow --test init init_stdin_property_round_trip -- --exact --ignored` | PASS |
| PC-022 | integration | `worktree_name_positional_round_trips` | AC-001.2b | `cargo test --package ecc-workflow --test worktree_name worktree_name_positional_round_trips -- --exact` | PASS |
| PC-023 | integration | `worktree_name_stdin_round_trips` | AC-001.2a | `cargo test --package ecc-workflow --test worktree_name worktree_name_stdin_round_trips -- --exact` | PASS |
| PC-024 | integration | `worktree_name_stdin_tty_rejected` | AC-001.11 | `cargo test --package ecc-workflow --test worktree_name worktree_name_stdin_tty_rejected -- --exact` | PASS |
| PC-025 | integration | `workflow_cli_parity_feature_stdin_exit_code` | AC-001.6 | `cargo test --package ecc-integration-tests --test workflow_cli_parity workflow_cli_parity_feature_stdin_exit_code -- --exact` | PASS |
| PC-026 | integration | `workflow_cli_parity_feature_stdin_state_bytes` | AC-001.6 | `cargo test --package ecc-integration-tests --test workflow_cli_parity workflow_cli_parity_feature_stdin_state_bytes -- --exact` | PASS |
| PC-027 | integration | `workflow_cli_parity_feature_stdin_stderr_class` | AC-001.6 | `cargo test --package ecc-integration-tests --test workflow_cli_parity workflow_cli_parity_feature_stdin_stderr_class -- --exact` | PASS |
| PC-028 | integration | `canonical_payload_survives_sh_c_with_env_var` — `Command::new("sh").env("FEATURE_PAYLOAD", canonical).args(["-c", "printf %s \"$FEATURE_PAYLOAD\" \| ecc-workflow init dev --feature-stdin"])`. Defense-in-depth proxy for `--feature-stdin` under shell-hostile input; NOT proof of Claude Code template-engine behavior | AC-001.1b (proxy) | `cargo test --package ecc-workflow --test init canonical_payload_survives_sh_c_with_env_var -- --exact` | PASS |
| PC-028b | integration (deleted post step 21b) | `pre_fix_sh_c_direct_interpolation_fails` — `sh -c 'ecc-workflow init dev "$SAFE_REPRO_PAYLOAD"'` where `SAFE_REPRO_PAYLOAD = "feat-\"unterminated"` triggers `unmatched "`. Asserts non-zero exit with `unmatched` in stderr. **Zero destructive subshells; no `rm`, no `$(…)`, no backticks.** | AC-001.1b (negative) | `cargo test --package ecc-workflow --test init pre_fix_sh_c_direct_interpolation_fails -- --exact` | PASS (asserts FAILURE) |
| PC-029 | integration | `existing_positional_tests_unchanged` — behavioral half of AC-001.5 (structural half is self-enforced by code review) | AC-001.5 | `cargo test --package ecc-workflow --test init missing_state_exits_zero_with_warning output_is_structured_json -- --exact` | PASS |
| PC-030 | lint (CLI) | `ecc validate commands` → exit 0 on post-fix repo | AC-001.1a | `cargo run --quiet --package ecc-cli -- validate commands` | exit 0 |
| PC-032 | lint | Clippy zero-warnings | all | `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| PC-033 | build | Release build | all | `cargo build --release --workspace` | exit 0 |
| PC-034 | full suite | Full nextest run | all (esp. AC-001.5) | `cargo nextest run --workspace` | all green |
| PC-035 | format | rustfmt check | all | `cargo fmt --all -- --check` | exit 0 |
| PC-036 | unit (lint) | `commands_validate_rule_flags_fenced_code_examples` — conservative: flag fenced-code occurrences | AC-001.7 | `cargo test --package ecc-app --lib validate::commands::tests::commands_validate_rule_flags_fenced_code_examples -- --exact` | PASS |
| PC-036a | unit (lint) | `commands_validate_rule_crlf_line_numbers_correct` — CRLF-terminated fixture yields correct 1-based line number | AC-001.7 | `cargo test --package ecc-app --lib validate::commands::tests::commands_validate_rule_crlf_line_numbers_correct -- --exact` | PASS |
| PC-037 | integration (prop) | `init_stdin_property_round_trip_via_delegator` — mirrors PC-021 against `ecc workflow init --feature-stdin` | AC-001.6, 8 | `cargo test --package ecc-integration-tests --test workflow_cli_parity init_stdin_property_round_trip_via_delegator -- --exact --ignored` | PASS |
| PC-038 | unit | `feature_input_preserves_bom_prefix` — `b"\xEF\xBB\xBFfoo"` → `Ok("\u{FEFF}foo")` | AC-001.2a | `cargo test --package ecc-workflow --lib commands::feature_input::tests::feature_input_preserves_bom_prefix -- --exact` | PASS |
| PC-039 | integration | `init_feature_stdin_without_concern_fails_cleanly` — clap rejects missing concern before reading stdin | AC-001.4 (adjacent) | `cargo test --package ecc-workflow --test init init_feature_stdin_without_concern_fails_cleanly -- --exact` | PASS |

**Total PCs**: 41 (40 pass-expected + PC-028b which asserts expected failure).

### Coverage Check

| AC | Covering PCs |
|----|--------------|
| AC-001.1a | PC-001, PC-002, PC-030 (narrow spec-pinned regex achieves zero violations post-fix) |
| AC-001.1b | PC-028 (proxy), PC-028b (negative characterization), PC-030 (structural) — plus manual smoke gate documented in `/implement` checklist |
| AC-001.2a | PC-011 (unit), PC-013 (int), PC-023 (worktree-name), PC-038 (BOM) |
| AC-001.2b | PC-012, PC-022 |
| AC-001.3 | PC-006 (unit), PC-014 (int) |
| AC-001.4 | PC-015, PC-039 |
| AC-001.5 | PC-029 (behavioral half); structural half self-enforced |
| AC-001.6 | PC-025, PC-026, PC-027, PC-037 |
| AC-001.7 | PC-001, PC-002, PC-036, PC-036a |
| AC-001.8 | PC-021, PC-037 (via delegator) |
| AC-001.9 | PC-007 (unit), PC-016 (int) |
| AC-001.10 | PC-008, PC-009, PC-017, PC-018 |
| AC-001.11 | PC-010 (unit), PC-020 (PTY 100ms median-of-5), PC-024 (worktree-name) |
| AC-001.12 | PC-003, PC-004, PC-005, PC-019 |

**All 14 ACs covered. Zero uncovered.**

### E2E Test Plan

| # | Boundary | Adapter | Port | Test Description | Default State | Run When |
|---|----------|---------|------|------------------|---------------|----------|
| 1 | `ecc-workflow` stdin | `std::io::Stdin` via `feature_input` | none | Pipe bytes to `ecc-workflow init --feature-stdin`; assert state.json contains exact bytes minus trailing LF | ignored → **activated** | `--feature-stdin` changes (this spec) |
| 2 | Slash-command template | `commands/*.md` | n/a | Simulate `/spec-dev` via `sh -c` with env-var + stdin; assert state.json.feature matches | ignored → **activated** | Template rewrites (this spec) |
| 3 | `ecc validate commands` | CLI | n/a | `cargo run -p ecc-cli -- validate commands` → exit 0 on post-fix repo | active (CI) | Changes to `commands/*.md` or `validate/commands.rs` |
| 4 | `ecc-cli` → `ecc-workflow` delegator | subprocess spawn | none | Parity on three axes (exit, stderr, state bytes) with identical stdin | ignored → **activated** | Delegator or flag changes (this spec) |

### E2E Activation Rules

All four boundaries activated for this implementation. PC-013/PC-017/PC-018/PC-019/PC-020 → boundary #1. PC-028/PC-030 → boundaries #2+3. PC-025/PC-026/PC-027/PC-037 → boundary #4. Post-fix, boundaries #1/#2/#4 revert to `ignored` until their respective files change again.

## Test Strategy — TDD Order

**Step 0 (pre-implementation)**: Run `rg -n 'ecc-workflow init' hooks/` — document findings. Confirm MSRV ≥1.70.

1. **PC-001** — validate rule RED against pre-fix repo (narrow spec-pinned regex matches 3 of 7 offending lines at top-level `!`-prefix).
2. **PC-002** — prose-exclusion negative.
3. **PC-036** — fenced-code conservative.
4. **PC-036a** — CRLF line-numbering.
5. **PC-003 → PC-004 → PC-005** — trailing-LF trio.
6. **PC-006** — empty-after-strip.
7. **PC-007** — invalid UTF-8.
8. **PC-008 → PC-009** — size cap pair.
9. **PC-010** — TTY guard with counting Reader.
10. **PC-011** — byte-preservation across METACHAR ∪ CONTROL ∪ NUL.
11. **PC-038** — BOM preservation.
12. **PC-015** — clap mutex.
13. **PC-039** — missing concern fails clap.
14. **PC-014** — empty-stdin integration.
15. **PC-012** — positional metachars integration.
16. **PC-013** — stdin metachars + NUL integration.
17. **PC-016** — invalid UTF-8 no-partial-write.
18. **PC-017 → PC-018** — size-cap integration.
19. **PC-019** — trailing-LF integration.
20. **PC-020** — PTY 100ms median-of-5 (hard gate).
21. **PC-022 → PC-023 → PC-024** — worktree-name parallel matrix.
22. **PC-021 → PC-037** — proptest + delegator proptest (`--ignored`, CI runs both).
23. **PC-025 → PC-026 → PC-027** — delegator parity trio.
24. **PC-028b** — pre-fix `sh -c` reproduction with `SAFE_REPRO_PAYLOAD`. RED.
25. **Step 21a — Validate rule implementation ships**: PC-001/002/036/036a go GREEN at unit level; **PC-030 (`ecc validate commands` on full repo) is RED** — pre-fix templates still have 3 offending `!`-prefix lines. Proves rule has teeth.
26. **Step 21b — Template rewrite batched**: rewrite 7 `!`-prefix lines across 4 templates (all removed); PC-030 flips GREEN; PC-028 (shell-proxy with canonical payload via env-var + stdin) goes GREEN; **delete PC-028b** after this step (its pre-fix reproduction no longer applies).
27. **Manual smoke gate**: invoke `/spec-dev` in a test session with the canonical payload; verify state.json; confirm no command substitution executed. Documented in `/implement` checklist.
28. **PC-029** — pre-existing-test characterization.
29. **PC-035 → PC-032 → PC-033 → PC-034** — fmt, clippy, build, full nextest.

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | `CHANGELOG.md` | project | Unreleased/fix | `fix(workflow): remove $ARGUMENTS from !-prefix shell-eval lines; add --feature-stdin; add validate rule` | US-001 |
| 2 | `docs/commands-reference.md` | project | Document `--feature-stdin` on `init` + `worktree-name` | Flag semantics: 64KB, UTF-8, TTY, single-LF strip; mutex with positional; examples | AC-001.3/9/10/11/12 |
| 3 | `CLAUDE.md` | project | New Gotcha entry | Templates MUST NOT use `$ARGUMENTS` in `!`-prefix shell-eval; enforced by `ecc validate commands` | US-001 |
| 4 | `rules/ecc/development.md` | project | Anti-Pattern entry | DO NOT interpolate `$ARGUMENTS` into `!`-prefix lines | AC-001.7 |
| 5 | `docs/backlog/BL-NNN-slash-command-audit.md` | follow-up | Create | Broader audit of all slash-command templates for user-text interpolation | Non-Req |
| 6 | `docs/backlog/BL-NNN-campaign-append-decision-shell-eval.md` | follow-up | Create | Secondary grill-me shell-eval risk | Non-Req |
| 7 | `docs/backlog/BL-NNN-tracing-feature-redaction.md` | follow-up | Create | Redact/truncate `feature` in `tracing::info!` at `transition.rs:391` | Security |
| 8 | `MODULE-SUMMARIES.md` | project | Add `feature_input.rs` entry | Boundary validator; four-guard policy | US-001 |
| 9 | `docs/adr/00NN-feature-input-boundary-validation.md` | project | New ADR | Records four-guard policy (TTY-first, 64KB, UTF-8-only, single-LF strip) and ordering invariant for future refactors | Decisions 11–14 |
| 10 | `docs/backlog/BL-NNN-widen-validate-rule-backtick.md` | follow-up | Create | Widen `ecc validate commands` rule to catch backtick-embedded `!$ARGUMENTS` inline-code patterns (defense-in-depth; post-fix has zero such occurrences already) | Round-3 adversary |

## SOLID Assessment

**CLEAN (uncle-bob verdict)**. 5 LOW observations; 4 folded into design (IsTerminal replaces unsafe/libc, Io-error diagnostic wrapping, LazyLock over once_cell, fenced-code conservative flagging); 2 deferred to REFACTOR phase (`resolve_feature` helper dedup, `FeatureSource::Stdin{is_tty}` enrichment).

## Robert's Oath Check

**CLEAN (robert verdict)**. 1 low WARNING on Oath 4 (16-file count at upper edge of "small release"), mitigated by Constraint 11 release coupling. Rework ratio 0.40 noted as trending at threshold (unrelated to this design).

## Security Notes

**No CRITICAL/HIGH (security-reviewer verdict)**. 9 LOW findings:
- **Resolved**: #2 unsafe eliminated (IsTerminal), #4 TTY-first ordering enforced, #6 fenced-code test added, #7 portable-pty dropped in favor of narrower rexpect.
- **Deferred to follow-up**: #1, #9 (tracing redaction of `feature` field → Doc Plan entry #7).
- **Accepted**: #3 (take-first invariant), #5 (delegator fail-fast), #8 (release coupling satisfied by template pattern).

## Rollback Plan

Ships as **one atomic series**. Recommended merge strategy: squash-merge to single revert target. **Partial revert within the series is UNSUPPORTED** — such a partial state leaves the validate rule RED on some templates or the flag dangling. To revert, revert the entire squash commit. No data migration unwind required (state.json schema unchanged; all feature strings on disk are clean).

## Bounded Contexts Affected

**No bounded contexts affected.** All file changes are in the CLI adapter layer (`crates/ecc-workflow/src/commands/*`, `crates/ecc-cli/src/commands/*`) or the app-layer validator (`crates/ecc-app/src/validate/commands.rs`) or slash-command templates (`commands/*.md`). Zero modifications to `crates/ecc-domain/src/*`.

The `workflow` bounded context (owns `WorkflowState`, `Phase`, `Concern`, `Timestamp`, `Completion`) is in the invocation chain but its types transit through the dispatch layer unchanged.

(No bounded contexts table rows — empty set.)

## Adversarial Review History

| Round | Verdict | Score | Key Findings Addressed |
|-------|---------|-------|------------------------|
| 1 | CONDITIONAL | 71 | TDD order batched template+validate, TTY ghost test, unverifiable AC-001.1b, rollback narrative, missing ADR, delegator proptest, hooks sweep, AC-001.5 non-modification |
| 2 | CONDITIONAL | 74 | PC-028 bypassed shell, VALIDATE_REGEX under-fit, PC-020 1s vs 100ms, AC-001.5 human-eyeball check |
| 3 | CONDITIONAL | 77 | PC-028 needed shell-eval proxy, VALIDATE_REGEX spec deviation, PC-020 relaxed 10×, PC-028b dangerous payload |
| 4 | **PASS** | **84** | All deviations reconciled with spec-as-written; safe payloads; honest framing |

## Phase Summary

### Design Reviews

| Review Type | Verdict | Finding Count |
|-------------|---------|---------------|
| SOLID (uncle-bob) | CLEAN | 5 LOW (4 folded, 2 deferred) |
| Robert | CLEAN | 1 WARNING (mitigated) |
| Security | CLEAR (no CRITICAL/HIGH) | 9 LOW (4 resolved, 2 deferred to backlog, 3 accepted) |

### Adversary Findings

| Dimension | Round | Score | Verdict | Key Rationale |
|-----------|-------|-------|---------|---------------|
| AC Coverage | 4 | 85 | PASS | 14 ACs covered; AC-001.1b has structural + behavioral-proxy + negative coverage |
| Execution Order | 4 | 85 | PASS | TDD split validated; validate-rule RED before template rewrite proves teeth |
| Fragility | 4 | 80 | PASS | 100ms bound honored via median-of-5; safe test payloads; dropped portable-pty |
| Rollback | 4 | 82 | PASS | Squash-merge single revert target; zero data migration |
| Architecture | 4 | 88 | PASS | Adapter-layer fix; zero domain touches; ADR records ordering invariant |
| Blast Radius | 4 | 80 | PASS | ~12 files, 3 crates; hooks/ sweep confirmed clean |
| Missing PCs | 4 | 75 | PASS | Delegator proptest added; AC-001.5 structural honestly self-enforced |
| Doc Plan | 4 | 82 | PASS | CHANGELOG + ADR + Gotcha + anti-pattern + 3 backlog entries + MODULE-SUMMARIES |
| **Round 4 avg** | — | **84** | **PASS** | **Advance to `/implement`** |

### File Changes Summary

| # | File | Action | Spec Ref |
|---|------|--------|----------|
| 1 | `crates/ecc-workflow/src/commands/feature_input.rs` | create | US-001, AC-001.3/9/10/11/12 |
| 2 | `crates/ecc-workflow/src/commands/mod.rs` | modify | scaffolding |
| 3 | `crates/ecc-workflow/src/main.rs` | modify | US-001, AC-001.3/4/5 |
| 4 | `crates/ecc-workflow/src/commands/init.rs` | modify | US-001, AC-001.2a/2b, 3, 9, 10, 11, 12 |
| 5 | `crates/ecc-workflow/src/commands/worktree_name.rs` | modify | US-001, AC-001.2a |
| 6 | `crates/ecc-cli/src/commands/workflow.rs` | modify | US-001, AC-001.6 |
| 7 | `crates/ecc-app/src/validate/commands.rs` | modify | US-001, AC-001.1a, 7 |
| 8 | `commands/spec-dev.md` | modify | US-001, AC-001.1a/1b |
| 9 | `commands/spec-fix.md` | modify | US-001, AC-001.1a/1b |
| 10 | `commands/spec-refactor.md` | modify | US-001, AC-001.1a/1b |
| 11 | `commands/project-foundation.md` | modify | US-001, AC-001.1a/1b |
| 12 | `crates/ecc-workflow/tests/init.rs` | modify | US-001, AC-001.2a/2b/3/4/8/9/10/11/12 |
| 13 | `crates/ecc-workflow/tests/worktree_name.rs` | create | US-001, AC-001.2a/11 |
| 14 | `crates/ecc-workflow/tests/proptest-regressions/` | create (dir, on first failure) | AC-001.8 |
| 15 | `crates/ecc-integration-tests/tests/workflow_cli_parity.rs` | modify | US-001, AC-001.6/8 |
| 16 | `crates/ecc-workflow/Cargo.toml` | modify | enables AC-001.8/11 |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| `docs/specs/2026-04-17-spec-command-shell-escaping/design.md` | Full design (File Changes, 41 PCs, Coverage, E2E, TDD order, Doc Plan, Reviews, Rollback, Bounded Contexts, Adversary History, Phase Summary) |
| `docs/specs/2026-04-17-spec-command-shell-escaping/campaign.md` | 9 decisions across grill-me + 4 adversary rounds |
| `/Users/titouanlebocq/.claude/plans/design-shell-escape-fix-2026-04-17.md` | Plan-mode architecture preview (approved) |
