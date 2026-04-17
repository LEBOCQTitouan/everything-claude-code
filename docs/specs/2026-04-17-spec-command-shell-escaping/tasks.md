# Tasks: Fix shell-eval injection in slash-command templates

**Spec**: `docs/specs/2026-04-17-spec-command-shell-escaping/spec.md`
**Design**: `docs/specs/2026-04-17-spec-command-shell-escaping/design.md`
**Started**: 2026-04-17
**Status**: pre-TDD (tasks.md generated; resume via `/implement` in fresh session)

## Pre-Implementation Checks

- [x] Step 0a: `rg -n 'ecc-workflow init' hooks/` → no matches. Confirmed clean. `done@2026-04-17T17:XX:XXZ`
- [x] Step 0b: MSRV check → Rust 1.85 (edition 2024), `std::io::IsTerminal` stable since 1.70. Safe. `done@2026-04-17T17:XX:XXZ`

## Wave 1 — Validate rule unit tests (serialized; same file `crates/ecc-app/src/validate/commands.rs`)

- [ ] PC-001: `commands_validate_rule_detects_dollar_arguments_fixture` — fixture with forbidden pattern → error. Verifies AC-001.7. Cmd: `cargo test --package ecc-app --lib validate::commands::tests::commands_validate_rule_detects_dollar_arguments_fixture -- --exact`
- [ ] PC-002: `commands_validate_rule_ignores_prose_dollar_arguments` — prose `$ARGUMENTS` without `!`-prefix → zero violations. Verifies AC-001.7.
- [ ] PC-036: `commands_validate_rule_flags_fenced_code_examples` — conservative flagging of fenced-code-block occurrences. Verifies AC-001.7.
- [ ] PC-036a: `commands_validate_rule_crlf_line_numbers_correct` — CRLF fixture yields correct 1-based line number. Verifies AC-001.7.

## Wave 2 — feature_input unit tests (serialized; new file `crates/ecc-workflow/src/commands/feature_input.rs`)

- [ ] PC-003: `feature_input_strips_single_trailing_lf` — `b"foo\n"` → `Ok("foo")`. Verifies AC-001.12.
- [ ] PC-004: `feature_input_preserves_second_lf` — `b"foo\n\n"` → `Ok("foo\n")`. Verifies AC-001.12.
- [ ] PC-005: `feature_input_preserves_cr_before_lf` — `b"foo\r\n"` → `Ok("foo\r")`. Verifies AC-001.12.
- [ ] PC-006: `feature_input_rejects_empty_after_strip` → `FeatureInputError::Empty`. Verifies AC-001.3.
- [ ] PC-007: `feature_input_rejects_invalid_utf8` — lone `0xFF` byte. Verifies AC-001.9.
- [ ] PC-008: `feature_input_accepts_64kb_exactly` — 65_536 bytes OK. Verifies AC-001.10.
- [ ] PC-009: `feature_input_rejects_64kb_plus_one` — 65_537 bytes → `TooLarge`. Verifies AC-001.10.
- [ ] PC-010: `feature_input_rejects_tty_before_any_read` — counting-Reader asserts zero `read()` calls. Verifies AC-001.11.
- [ ] PC-011: `feature_input_preserves_metachars_control_and_nul` — byte-preservation. Verifies AC-001.2a.
- [ ] PC-038: `feature_input_preserves_bom_prefix` — `b"\xEF\xBB\xBFfoo"` → `Ok("\u{FEFF}foo")`. Verifies AC-001.2a.

## Wave 3 — CLI integration: argument parsing

- [ ] PC-015: `init_stdin_and_positional_exits_two` — clap `ArgGroup` mutex. Verifies AC-001.4.
- [ ] PC-039: `init_feature_stdin_without_concern_fails_cleanly`. Verifies AC-001.4 (adjacent).
- [ ] PC-014: `init_empty_stdin_exits_two`. Verifies AC-001.3.

## Wave 4 — Init round-trip integration (`crates/ecc-workflow/tests/init.rs`)

- [ ] PC-012: `init_positional_round_trips_metachars`. Verifies AC-001.2b.
- [ ] PC-013: `init_stdin_round_trips_metachars_and_nul`. Verifies AC-001.2a.
- [ ] PC-016: `init_stdin_invalid_utf8_rejected_no_state`. Verifies AC-001.9.
- [ ] PC-017: `init_stdin_65537_bytes_rejected`. Verifies AC-001.10.
- [ ] PC-018: `init_stdin_65536_bytes_accepted`. Verifies AC-001.10.
- [ ] PC-019: `init_stdin_trailing_newline_policy` — 3 sub-assertions. Verifies AC-001.12.
- [ ] PC-020: `init_stdin_tty_rejected_within_100ms_median_of_5` — `rexpect`-backed PTY. Verifies AC-001.11.

## Wave 5 — worktree-name parallel matrix (new `crates/ecc-workflow/tests/worktree_name.rs`)

- [ ] PC-022: `worktree_name_positional_round_trips`. Verifies AC-001.2b.
- [ ] PC-023: `worktree_name_stdin_round_trips`. Verifies AC-001.2a.
- [ ] PC-024: `worktree_name_stdin_tty_rejected`. Verifies AC-001.11.

## Wave 6 — Proptest + delegator proptest (both `#[ignore]`)

- [ ] PC-021: `init_stdin_property_round_trip` — 1024 cases, ≤4KB UTF-8. Verifies AC-001.8.
- [ ] PC-037: `init_stdin_property_round_trip_via_delegator` — via `ecc workflow init`. Verifies AC-001.6, 8.

## Wave 7 — Delegator parity (`crates/ecc-integration-tests/tests/workflow_cli_parity.rs`)

- [ ] PC-025: `workflow_cli_parity_feature_stdin_exit_code`. Verifies AC-001.6.
- [ ] PC-026: `workflow_cli_parity_feature_stdin_state_bytes`. Verifies AC-001.6.
- [ ] PC-027: `workflow_cli_parity_feature_stdin_stderr_class`. Verifies AC-001.6.

## Wave 8 — Pre-fix characterization (temporary, deleted after Wave 9)

- [ ] PC-028b: `pre_fix_sh_c_direct_interpolation_fails` — `sh -c` with `SAFE_REPRO_PAYLOAD = "feat-\"unterminated"` → non-zero exit with `unmatched` stderr. **No `rm -rf /`, no `$(…)`, no backticks in payload.** Verifies AC-001.1b (negative).

## Wave 9 — Template rewrite (atomic commit)

- [ ] Rewrite `!`-prefix lines in:
  - [ ] `commands/spec-dev.md:14,19`
  - [ ] `commands/spec-fix.md:14,19`
  - [ ] `commands/spec-refactor.md:14,19`
  - [ ] `commands/project-foundation.md:18`
  Replace each with prose: *"Use the Bash tool with `env FEATURE_PAYLOAD='<feature>' sh -c 'printf %s \"$FEATURE_PAYLOAD\" | ecc-workflow init <concern> --feature-stdin'` — do NOT interpolate the feature directly into the command string."*
- [ ] PC-028: `canonical_payload_survives_sh_c_with_env_var` → GREEN post-rewrite. Verifies AC-001.1b (proxy).
- [ ] PC-030: `cargo run --quiet --package ecc-cli -- validate commands` → exit 0. Verifies AC-001.1a.
- [ ] Delete PC-028b (pre-fix reproduction no longer applicable).

## Wave 10 — Manual smoke test

- [ ] Invoke `/spec-dev` with canonical payload (e.g., `feat-"paste with quotes and \`backticks\`"`) in a test session. Verify `state.json.feature` contains the input bytes. Confirm no command substitution executed. Document outcome in `implement-done.md` under `## Manual Smoke Gate` section.

## Wave 11 — AC-001.5 characterization

- [ ] PC-029: `existing_positional_tests_unchanged` — run `missing_state_exits_zero_with_warning`, `output_is_structured_json` via their existing names; assert PASS. Behavioral half of AC-001.5; structural half self-enforced by code review.

## Final Gates

- [ ] PC-035: `cargo fmt --all -- --check` → exit 0
- [ ] PC-032: `cargo clippy --workspace --all-targets -- -D warnings` → exit 0
- [ ] PC-033: `cargo build --release --workspace` → exit 0
- [ ] PC-034: `cargo nextest run --workspace` → all green

## Post-TDD

- [ ] Phase 4: E2E tests (4 boundaries activated; see design's E2E Test Plan)
- [ ] Phase 5: Code review (`code-reviewer` agent)
- [ ] Phase 6: Doc updates (10 entries per design's Doc Update Plan — CHANGELOG + CLAUDE.md Gotcha + rules/ecc/development.md Anti-Pattern + docs/commands-reference.md + 3 backlog entries + MODULE-SUMMARIES.md + ADR #9 + widen-validate-rule backlog)
- [ ] Phase 7.5: Supplemental docs (module-summary-updater + diagram-updater in parallel)
- [ ] Phase 7: Write implement-done.md
- [ ] Phase 8: Merge via `ecc-workflow merge`

## File Changes (reference)

See `design.md` § File Changes table for the 16 file changes in dependency order.

## Resume Instructions

This tasks.md is the authoritative resume source. When `/implement` is invoked in a fresh session:
1. `ecc-workflow status` confirms phase=`implement`
2. Parse this file; find the first incomplete (non-`done`) entry
3. Rebuild TodoWrite from the checklist
4. Dispatch `tdd-executor` for the resume PC per the wave-dispatch rules in `skills/wave-dispatch/SKILL.md`

## Context Handoff Notes

- **Do NOT skip Plan Mode on resume** — re-enter Plan Mode to re-present the full plan before executing.
- **Spec + design are on disk** at the paths above; no reconstruction needed.
- **campaign.md** has 9 persisted decisions covering grill-me + 4 adversary rounds.
- **Safe test payload pinned**: `SAFE_REPRO_PAYLOAD = "feat-\"unterminated"` (string literal in test constants). Test payloads MUST NOT contain executable metacharacters — no `rm`, no `$(…)`, no backticks, even inside string literals.
- **Release coupling**: Wave 9 is atomic. If it fails mid-way, `ecc validate commands` in CI may go RED for partial states. Squash-merge the full series as one revert target.
