# Solution: CLAUDE.md `TEMPORARY (BL-NNN)` Marker Lint

## Spec Reference

- **Concern**: dev
- **Feature**: Lint rule: flag TEMPORARY (BL-NNN) markers in CLAUDE.md whose backlog IDs do not exist on disk, plus one-time audit of existing markers
- **Spec**: `docs/specs/2026-04-18-claude-md-temp-marker-lint/spec.md`

## File Changes (dependency order)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `crates/ecc-domain/src/docs/claude_md.rs` | modify | Add `TemporaryMarker { backlog_id: u32, line_number: usize, raw_text: String }` VO + `extract_temporary_markers(content: &str) -> Vec<TemporaryMarker>`. Static `LazyLock<regex::Regex>` for `(?i)TEMPORARY\s*\(BL-0*(\d{1,6})\)`. Reuses existing `in_code_block` fence-skip. Zero I/O. | US-001 AC-001.3, .4, .7, .8, .9, .14 |
| 2 | `crates/ecc-domain/src/backlog/entry.rs` | modify | Add `matches_backlog_filename(filename: &str, id: u32) -> bool` pure predicate. `BL-{:03}` for ≤999; `BL-{id}` for ≥1000. | US-001 AC-001.3; decision #10 |
| 3 | `crates/ecc-infra/src/fs_backlog.rs` | modify | Refactor `find_entry_path` to delegate to `matches_backlog_filename`. `strip_prefix("BL-").and_then(\|s\| s.parse::<u32>().ok())` with literal fallback. Behavior-preserving. | AC-006.6; decision #10 |
| 4 | `crates/ecc-app/src/validate_claude_md.rs` | modify | Add `run_validate_temporary_markers(fs, terminal, project_root, disabled, strict, audit_report) -> bool`. Decomposed into helpers: `WalkPlan::discover()`, `BacklogIndex::new()` / `contains(id)`, `MarkerDiagnostics::emit_*`. Top-level fn <30 LOC. `disabled` param threaded from CLI (no `std::env` in app). Walker: depth 16, deny-list, `is_symlink()` skip. Path sanitizer strips non-printables before emission. | US-001, US-002, US-003, US-004 |
| 5 | `crates/ecc-cli/src/commands/validate.rs` | modify | Restructure `ClaudeMd { counts: bool }` → `ClaudeMd { #[command(subcommand)] cmd: Option<ClaudeMdSubcommand>, #[arg(long)] counts: bool }` where `ClaudeMdSubcommand = Counts \| Markers { strict, audit_report } \| All { strict, audit_report }`. Read `ECC_CLAUDE_MD_MARKERS_DISABLED` via `std::env::var` here; pass as `disabled: bool` to app. `--counts` flag → `DEPRECATED:` stderr then dispatch to `Counts`. | decision #1; AC-001.13, AC-002.4, AC-006.5 |
| 6 | `crates/ecc-integration-tests/tests/validate_claude_md_markers.rs` | create | End-to-end CLI coverage: `--counts` deprecation, markers happy/fail, strict-scoping, kill-switch subprocess. | US-001..US-004, US-006 |
| 7 | `crates/ecc-integration-tests/tests/validate_claude_md_repo_audit.rs` | create | Real-repo anchor: post-fix worktree exits 0 + success stdout; audit-report zero `missing` rows. | AC-004.3, AC-005.2 |
| 8 | `CLAUDE.md` | modify | Delete line 108 (stale TEMPORARY BL-150); update CLI Commands top-10 to mention `markers` subcommand. | US-005 AC-005.1; doc impact |
| 9 | `docs/backlog/BL-158-frontmatter-aware-temporary-marker-v2.md` | create | Companion entry capturing decision-#4 governance loophole (archived=resolved). Status: open. Filed via `ecc backlog add`-style write (ID confirmed via `ecc backlog next-id`). | spec Constraints; AC-001.12 |
| 10 | `docs/specs/2026-04-18-claude-md-temp-marker-lint/audit-report.md` | create | Post-fix audit table: zero `missing` rows. Regression anchor per AC-004.2 (pre-fix BL-150 row documented in body). | AC-004.4 |
| 11 | `CHANGELOG.md` | modify | Added: `markers` subcommand + `--audit-report`; Removed: stale `TEMPORARY (BL-150)` line; Deprecated: `--counts` flag (removed next minor). | AC-005.3 |
| 12 | `docs/commands-reference.md` | modify | Add `markers` subcommand section with flags + `ECC_CLAUDE_MD_MARKERS_DISABLED` kill-switch warning. | doc impact |
| 13 | `rules/ecc/github-actions.md` | modify | Update `ci.yml` validate-job description to list the new markers step. | doc impact |
| 14 | `.github/workflows/ci.yml` | modify | Add step `ecc validate claude-md markers --strict` after existing `--counts` step in `validate` job. **Atomic single-concern commit per AC-006.4.** | US-006 AC-006.1, .3, .4 |

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | Domain: canonical `TEMPORARY (BL-150)` | AC-001.3 | `cargo test -p ecc-domain docs::claude_md::tests::extract_temporary_marker_canonical` | pass |
| PC-002 | unit | Domain: variants (mixed case, `BL-0150`, `BL-100000`) + fence-skip | AC-001.3, .4 | `cargo test -p ecc-domain docs::claude_md::tests::extract_temporary_marker_variants` | pass |
| PC-003 | unit | Domain: negative (prose, `BL-`, `BL-ABC`, `TEMPORARY: note`) | AC-001.9 | `cargo test -p ecc-domain docs::claude_md::tests::extract_temporary_marker_negative` | pass |
| PC-004 | unit | Domain: duplicate markers → two entries | AC-001.8 | `cargo test -p ecc-domain docs::claude_md::tests::extract_temporary_marker_duplicates` | pass |
| PC-005 | unit | Domain: line numbers ascending | AC-001.14 | `cargo test -p ecc-domain docs::claude_md::tests::extract_temporary_marker_order` | pass |
| PC-006 | unit | Domain: `matches_backlog_filename` ≤999 (BL-001-foo.md, BL-100.md) | AC-001.3 | `cargo test -p ecc-domain backlog::entry::tests::matches_backlog_filename_padded` | pass |
| PC-007 | unit | Domain: `matches_backlog_filename` ≥1000 (BL-1000-bar.md) | AC-001.3 | `cargo test -p ecc-domain backlog::entry::tests::matches_backlog_filename_unpadded` | pass |
| PC-008 | unit | Infra regression: `fs_backlog::tests::load_entries_reads_bl_files` passes | AC-006.6 | `cargo test -p ecc-infra fs_backlog::tests::load_entries_reads_bl_files` | pass |
| PC-009 | unit | Infra regression: `next_id_computes_max_plus_one` passes | AC-006.6 | `cargo test -p ecc-infra fs_backlog::tests::next_id_computes_max_plus_one` | pass |
| PC-010 | unit | Infra regression: `update_entry_status_atomic_write` passes | AC-006.6 | `cargo test -p ecc-infra fs_backlog::tests::update_entry_status_atomic_write` | pass |
| PC-011 | unit | App: `disabled=true` → exit true + stderr notice `markers: disabled via ECC_CLAUDE_MD_MARKERS_DISABLED` | AC-006.5 | `cargo test -p ecc-app validate_claude_md::tests::markers_kill_switch_emits_notice` | pass |
| PC-012 | unit | App: missing `docs/backlog/` → all markers missing | AC-001.11 | `cargo test -p ecc-app validate_claude_md::tests::markers_missing_backlog_dir` | pass |
| PC-013 | unit | App: walker deny-list + symlink-skip | AC-003.3, .4 | `cargo test -p ecc-app validate_claude_md::tests::markers_walker_denylist_and_symlink` | pass |
| PC-014 | unit | App: depth cap 16 → WARN | AC-003.7 | `cargo test -p ecc-app validate_claude_md::tests::markers_depth_limit` | pass |
| PC-015 | unit | App: non-UTF8 → `WARN: <path>: skipping non-UTF8 file` + skip; I/O error (permission denied) → distinct `WARN: <path>: read error: ...` + skip. Exit code unaffected either way. | AC-003.6 | `cargo test -p ecc-app validate_claude_md::tests::markers_non_utf8_and_io_errors_distinguished` | pass |
| PC-016 | unit | App: AGENTS.md walked identically | AC-003.2 | `cargo test -p ecc-app validate_claude_md::tests::markers_agents_md_scanned` | pass |
| PC-017 | unit | App: zero markers + no `--strict` → silent stdout | AC-001.10, AC-002.5 | `cargo test -p ecc-app validate_claude_md::tests::markers_baseline_silent` | pass |
| PC-018 | unit | App: zero missing + `--strict` → success stdout | AC-002.3 | `cargo test -p ecc-app validate_claude_md::tests::markers_strict_success` | pass |
| PC-019 | unit | App: missing + `--strict` → `ERROR:` prefix + return false | AC-002.2 | `cargo test -p ecc-app validate_claude_md::tests::markers_strict_error_prefix` | pass |
| PC-020 | unit | App: missing + default → `WARN:` prefix + return true | AC-001.2, AC-001.6, AC-002.1 | `cargo test -p ecc-app validate_claude_md::tests::markers_warn_default` | pass |
| PC-021 | unit | App: `--audit-report` emits one row per marker, correct status, archived files = resolved | AC-001.5, AC-004.1 | `cargo test -p ecc-app validate_claude_md::tests::markers_audit_report_table` | pass |
| PC-022 | unit | App: lexicographic file order + within-file line ordering | AC-003.5, AC-003.1 | `cargo test -p ecc-app validate_claude_md::tests::markers_file_order_deterministic` | pass |
| PC-023 | integration | CLI: `--counts` emits EXACT `DEPRECATED: use 'ecc validate claude-md counts' (subcommand form); --counts will be removed in the next minor release.` on stderr | AC-001.13 | `cargo test -p ecc-integration-tests validate_claude_md_markers::counts_flag_deprecation_warning` | pass |
| PC-024 | integration | CLI: `markers --strict` happy (AC-001.1 BL-156 present) + missing-BL fail (exit 1, `ERROR:` prefix, stderr contains file path AND `:<line>:` AND `BL-999` simultaneously) | AC-001.1, AC-001.2, AC-002.2, AC-006.2 | `cargo test -p ecc-integration-tests validate_claude_md_markers::markers_strict_happy_and_fail_message_composition` | pass |
| PC-025 | integration | CLI: `counts --strict` rejected by clap (strict scoped to markers) | AC-002.4 | `cargo test -p ecc-integration-tests validate_claude_md_markers::strict_scoped_to_markers` | pass |
| PC-026 | integration | CLI subprocess: `ECC_CLAUDE_MD_MARKERS_DISABLED=1` → exit 0 + stderr notice | AC-006.5 | `cargo test -p ecc-integration-tests validate_claude_md_markers::kill_switch_env_subprocess` | pass |
| PC-027 | integration | Existing `ecc backlog next-id/list/update-status` tests unchanged | AC-006.6 | `cargo test -p ecc-integration-tests backlog` | pass |
| PC-028 | e2e | Post-fix worktree: `ecc validate claude-md markers --strict` → exit 0 + success stdout | AC-004.3, AC-005.2, AC-006.3 | `./target/release/ecc validate claude-md markers --strict` | exit 0; stdout contains `All TEMPORARY markers reference valid backlog entries` |
| PC-029 | lint | `CLAUDE.md` has zero `TEMPORARY (BL-` occurrences | AC-005.1 | `test "$(grep -c 'TEMPORARY (BL-' CLAUDE.md \|\| true)" = "0"` | exit 0 |
| PC-030 | lint | `.github/workflows/ci.yml` contains markers step | AC-006.1, AC-006.4 | `grep -q 'ecc validate claude-md markers --strict' .github/workflows/ci.yml` | exit 0 |
| PC-031 | lint | Companion `BL-158` file exists + title | spec Constraints, AC-001.12 | `test -f docs/backlog/BL-158-frontmatter-aware-temporary-marker-v2.md && grep -q 'Frontmatter-aware TEMPORARY marker validation (v2)' docs/backlog/BL-158-frontmatter-aware-temporary-marker-v2.md` | exit 0 |
| PC-032 | lint | audit-report.md: zero `missing` table rows in the After section (scoped line-start regex; quoted references in prose are excluded) | AC-004.3, AC-004.4 | `test "$(awk '/^## After/,0' docs/specs/2026-04-18-claude-md-temp-marker-lint/audit-report.md \| grep -cE '^\| .* \| missing \|' \|\| true)" = "0"` | exit 0 |
| PC-033 | lint | CHANGELOG.md has Added + Removed + Deprecated under current release | AC-005.3 | `grep -E '^### (Added\|Removed\|Deprecated)' CHANGELOG.md \| head -20 \| wc -l \| grep -qE '^[[:space:]]*[3-9]'` | ≥3 headers |
| PC-034 | lint | commands-reference.md documents subcommand + kill switch | doc impact, AC-006.5 | `grep -qE 'claude-md markers\|ECC_CLAUDE_MD_MARKERS_DISABLED' docs/commands-reference.md` | ≥1 match |
| PC-035 | build | Clippy zero-warning gate | coding-style | `cargo clippy --all-targets -- -D warnings` | exit 0 |
| PC-036 | build | Release build passes | global | `cargo build --release` | exit 0 |
| PC-037 | build | Workspace test suite passes | global | `cargo test` | exit 0 |
| PC-038 | lint | Domain I/O purity guard: `ecc-domain/src/docs/claude_md.rs` has no `use std::{fs,io,env,process}` or `tokio` imports | AC-001.7 | `! grep -qE 'use std::(fs\|io\|env\|process)\|use tokio' crates/ecc-domain/src/docs/claude_md.rs` | exit 0 |
| PC-039 | lint | Audit-report regression anchor: pre-fix body contains the literal BL-150 row (Before table documents AC-004.2) | AC-004.2 | `grep -qE '^\\\| CLAUDE.md \\\| 108 \\\| BL-150 \\\| missing \\\|' docs/specs/2026-04-18-claude-md-temp-marker-lint/audit-report.md` | exit 0 |
| PC-040 | unit | App: ANSI/non-printable sanitizer strips `\x1b` + control bytes from emitted paths | security (Round 1 mitigation) | `cargo test -p ecc-app validate_claude_md::tests::markers_path_sanitizer_strips_control_bytes` | pass |
| PC-041 | integration | CLI smoke: `--counts` (legacy), `counts` (subcommand), `markers --strict`, `all` all parse successfully (clap compatibility guard for `Option<Subcommand>` + sibling flag) | decision #1; fragility mitigation | `cargo test -p ecc-integration-tests validate_claude_md_markers::clap_surface_smoke` | pass |

### Coverage Check

**All 39 ACs covered by ≥1 PC.**

| AC | PCs |
|----|-----|
| AC-001.1 | PC-024 |
| AC-001.2 | PC-020, PC-024 |
| AC-001.3 | PC-001, PC-002, PC-006, PC-007 |
| AC-001.4 | PC-002 |
| AC-001.5 | PC-021 |
| AC-001.6 | PC-020 |
| AC-001.7 | PC-038 (executable grep guard — no `use std::{fs,io,env,process}` or `tokio` in `claude_md.rs`) |
| AC-001.8 | PC-004 |
| AC-001.9 | PC-003 |
| AC-001.10 | PC-017 |
| AC-001.11 | PC-012 |
| AC-001.12 | PC-031 (companion BL filed) |
| AC-001.13 | PC-023 (exact string assertion) |
| AC-001.14 | PC-005, PC-022 |
| AC-002.1 | PC-020 |
| AC-002.2 | PC-019, PC-024 |
| AC-002.3 | PC-018 |
| AC-002.4 | PC-025 |
| AC-002.5 | PC-017, PC-018 |
| AC-003.1 | PC-022 |
| AC-003.2 | PC-016 |
| AC-003.3 | PC-013 |
| AC-003.4 | PC-013 (symlink-skip) |
| AC-003.5 | PC-022 |
| AC-003.6 | PC-015 |
| AC-003.7 | PC-014 |
| AC-004.1 | PC-021 |
| AC-004.2 | PC-039 (executable grep for `\| CLAUDE.md \| 108 \| BL-150 \| missing \|` in audit-report.md Before-table section) |
| AC-004.3 | PC-028, PC-032 |
| AC-004.4 | PC-032 |
| AC-005.1 | PC-029 |
| AC-005.2 | PC-028 |
| AC-005.3 | PC-033 |
| AC-006.1 | PC-030 |
| AC-006.2 | PC-024 |
| AC-006.3 | PC-028 |
| AC-006.4 | PC-030 (single-commit granularity enforced by TDD ordering) |
| AC-006.5 | PC-011, PC-026 |
| AC-006.6 | PC-008, PC-009, PC-010, PC-027 |

### E2E Test Plan

| # | Boundary | Adapter | Port | Test Description | Default State | Run When |
|---|----------|---------|------|------------------|---------------|----------|
| 1 | CLI → app (`markers --strict`) | real binary | `FileSystem`+`TerminalIO` | Post-fix worktree exits 0 + success stdout | ignored | always in CI |
| 2 | Subprocess env | real shell | `std::env` (CLI-layer) | Kill-switch env var short-circuits exit 0 | ignored | always in CI |
| 3 | CLI dispatch | clap parser | n/a | `--counts` flag → `DEPRECATED:` stderr | ignored | always in CI |
| 4 | CLI dispatch | clap parser | n/a | `counts --strict` rejected | ignored | always in CI |
| 5 | `find_entry_path` refactor | `fs_backlog` | `FileSystem` | Existing `ecc backlog` tests unchanged | un-ignored (already active) | always in CI |

### E2E Activation Rules

All 5 un-ignored for this implementation: adapter-layer touched (#5), CLI restructured (#1-4), refactor behavior-preserving but proof-bearing (#5).

## Test Strategy (TDD Order)

| Step | PCs | Commit message |
|------|-----|----------------|
| 1 (RED) | PC-001..005 | `test: add TemporaryMarker extraction tests` |
| 2 (GREEN) | PC-001..005 | `feat: add TemporaryMarker domain extractor` |
| 3 (RED) | PC-006..007 | `test: add matches_backlog_filename predicate tests` |
| 4 (GREEN) | PC-006..007 | `feat: add matches_backlog_filename domain predicate` |
| 5 (REFACTOR) | PC-008..010 | `refactor: delegate fs_backlog::find_entry_path to matches_backlog_filename` |
| 6 (RED) | PC-011..022 | `test: add run_validate_temporary_markers use-case tests` |
| 7 (GREEN) | PC-011..022 | `feat: add run_validate_temporary_markers with WalkPlan/BacklogIndex/MarkerDiagnostics helpers` |
| 8 (RED) | PC-023..026 | `test: add claude-md markers CLI integration tests` |
| 9 (GREEN) | PC-023..026 | `feat(cli): add claude-md markers subcommand with --counts deprecation and kill-switch env var` |
| 10 | PC-029 | `fix: remove stale CLAUDE.md TEMPORARY (BL-150) line` (PC-027 already verified as part of step 5 regression gate — no separate commit for re-verification) |
| 11 | — | `docs: update CLAUDE.md CLI top-10 for markers subcommand` |
| 12 | PC-031 | `docs(backlog): file BL-158 frontmatter-aware TEMPORARY marker v2` |
| 13 | PC-032, PC-039 | `docs(spec): add claude-md temp-marker-lint audit report (Before/After tables)` |
| 14 | PC-033 | `docs: add CHANGELOG Unreleased entry for claude-md markers lint` |
| 15 | PC-034 | `docs: document validate claude-md markers subcommand + kill-switch env var` |
| 16 | — | `docs(rules): update ci.yml validate-job for markers step` |
| 17 | PC-028, PC-030 | `feat(ci): validate CLAUDE.md TEMPORARY markers` *(atomic single-concern per AC-006.4)* |
| 18 | PC-035..041 | `chore: verify clippy + build + full test suite` |

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | `CHANGELOG.md` | upper | modify | Target section: `## [Unreleased]` (do NOT backport to 4.2.0 which already shipped). Added: `markers` subcommand + `--audit-report`. Removed: `TEMPORARY (BL-150)` from CLAUDE.md. Deprecated: `--counts` flag (removed in next minor). | AC-005.3 |
| 2 | `CLAUDE.md` | upper | modify | Delete line 108; update CLI Commands top-10 | US-005 AC-005.1 |
| 3 | `docs/commands-reference.md` | upper | modify | Add `markers` section + kill-switch note | doc impact |
| 4 | `rules/ecc/github-actions.md` | upper | modify | Update ci.yml validate-job description | doc impact |
| 5 | `docs/specs/2026-04-18-claude-md-temp-marker-lint/audit-report.md` | artifact | create | Post-fix audit table (zero `missing`) | AC-004.4 |
| 6 | `docs/backlog/BL-158-frontmatter-aware-temporary-marker-v2.md` | artifact | create | Companion v2 backlog | AC-001.12 |

Must include CHANGELOG.md ✓. No ADRs required (all 15 decisions marked `ADR Needed? No`).

## SOLID Assessment

**PASS with 2 MEDIUM findings addressed in design**:

1. SRP — `run_validate_temporary_markers` bundled 5 concerns → decomposed into `WalkPlan`, `BacklogIndex`, `MarkerDiagnostics` helpers. Top-level fn <30 LOC. Each helper independently testable.
2. ISP/DIP — `std::env::var` leak avoided by reading env in CLI layer (argv-adjacent, idiomatic) and threading `disabled: bool` into app. App layer stays env-pure.

OCP (3-tier severity extension), LSP (`FileSystem` port reuse), DIP (dependency direction), Clean Arch (ACL via primitive `u32`) all CLEAN.

## Robert's Oath Check

**CLEAN with 2 warnings addressed**:

1. Depth cap tightened from 64 → **16** with justifying comment (2× observed-max repo depth).
2. Kill-switch stderr notice emitted unconditionally when honored; PC-011 tightened to assert exact stderr content (not just boolean return).

18-commit TDD cadence, 37 PCs over 39 ACs, small releases all CLEAN.

## Security Notes

**CLEAR with 2 LOW findings addressed**:

1. Symlink escape — walker skips symlinks at `file_type().is_symlink()` boundary. Protects against `/etc/passwd` read via malicious symlink; also forestalls symlink loops.
2. ANSI escape in diagnostics — paths sanitized (`path.display()` → strip `\x00..\x1f` + `\x7f`) before stderr/stdout emission.

ReDoS CLEAR (bounded quantifiers), integer bounds CLEAR (`\d{1,6}` << u32), path traversal CLEAR (project-root-anchored), env-var exposure CLEAR (non-sensitive).

## Rollback Plan (reverse dependency order)

If implementation fails mid-stream, undo in reverse:

1. Revert #18 (CI step) — `git revert` the atomic commit; CI returns to prior validate job.
2. Revert #17 (`rules/ecc/github-actions.md`) — doc-only.
3. Revert #16, #15, #14, #13, #12 (doc commits) — independent, no blast radius.
4. Revert #11 (CLAUDE.md line removal) — restores the stale warning; acceptable interim state.
5. Revert #10 (backlog integration regression confirmation) — test-only, no code.
6. Revert #9, #8 (CLI restructure) — `--counts` flag still works in prior form.
7. Revert #7, #6 (app use case + helpers) — removes new functionality; existing `--counts` unaffected.
8. Revert #5 (infra refactor) — behavior-preserving per AC-006.6; reverting is safe.
9. Revert #4, #3 (domain predicate + tests) — zero external dependencies.
10. Revert #2, #1 (domain extractor + tests) — zero external dependencies.

Emergency rollback (production): set `ECC_CLAUDE_MD_MARKERS_DISABLED=1` in CI env. Lint no-ops with stderr notice; no revert needed.

## Bounded Contexts Affected

| Context | Role | Files Modified |
|---------|------|----------------|
| `docs` | value object (new: `TemporaryMarker`) + pure extractor | `crates/ecc-domain/src/docs/claude_md.rs` |
| `backlog` | value-object predicate (new: `matches_backlog_filename`) | `crates/ecc-domain/src/backlog/entry.rs`, `crates/ecc-infra/src/fs_backlog.rs` (refactor) |

Other domain modules (not registered as bounded contexts): none.

Cross-context reference: `docs::TemporaryMarker.backlog_id: u32` holds a primitive from the `backlog` context's identifier space. ACL boundary — no import of `BacklogId` VO across contexts (primitive is the translation layer).

---

## Phase Summary

### Design Reviews

| Review Type | Verdict | Finding Count | Remediated |
|-------------|---------|---------------|------------|
| SOLID (uncle-bob) | PASS with 2 MEDIUM | 2 | Both (SRP decomposition into WalkPlan/BacklogIndex/MarkerDiagnostics; kill-switch threaded as bool from CLI) |
| Robert's Oath | CLEAN with 2 warnings | 2 | Both (depth cap 16 with comment; stderr notice always emitted + PC-011 tightened) |
| Security | CLEAR with 2 LOW | 2 | Both (symlink-skip via is_symlink(); ANSI/control-byte sanitizer on emitted paths) |

### Adversary Findings

| Dimension | Round 1 | Round 2 | Verdict | Key Rationale |
|-----------|---------|---------|---------|---------------|
| AC Coverage | 78 | 92 | PASS | AC-001.7 + AC-004.2 gaps closed via PC-038/PC-039 executable grep guards |
| Execution Order | 85 | 85 | PASS | 18-commit TDD order coherent; step 10 empty commit eliminated |
| Fragility | 72 | 82 | PASS | PC-024 tightened (path+line+ID simultaneous assertion); PC-015 UTF-8 vs I/O distinguished; PC-041 clap smoke guard |
| Rollback | 88 | 85 | PASS | 10-step reverse order; kill-switch env var = emergency brake |
| Architecture | 90 | 92 | PASS | Domain I/O purity now executably enforced (PC-038); ACL via primitive u32 |
| Blast Radius | 82 | 78 | PASS | 14 files, hexagonal-mandated scope; backward-compat preserved |
| Missing PCs | 62 | 88 | PASS | PC-038 (domain purity), PC-039 (regression anchor), PC-040 (sanitizer), PC-041 (clap smoke) added |
| Doc Plan | 88 | 82 | PASS | CHANGELOG target clarified to `## [Unreleased]`, no 4.2.0 backport |
| **Average** | **82** | **84** | **PASS** | — |

### File Changes Summary

| # | File | Action | Spec Ref |
|---|------|--------|----------|
| 1 | `crates/ecc-domain/src/docs/claude_md.rs` | modify | US-001 AC-001.3, .4, .7, .8, .9, .14 |
| 2 | `crates/ecc-domain/src/backlog/entry.rs` | modify | US-001 AC-001.3; decision #10 |
| 3 | `crates/ecc-infra/src/fs_backlog.rs` | modify | AC-006.6; decision #10 |
| 4 | `crates/ecc-app/src/validate_claude_md.rs` | modify | US-001..US-004 |
| 5 | `crates/ecc-cli/src/commands/validate.rs` | modify | decision #1; AC-001.13, AC-002.4, AC-006.5 |
| 6 | `crates/ecc-integration-tests/tests/validate_claude_md_markers.rs` | create | US-001..US-004, US-006 |
| 7 | `crates/ecc-integration-tests/tests/validate_claude_md_repo_audit.rs` | create | AC-004.3, AC-005.2 |
| 8 | `CLAUDE.md` | modify | US-005 AC-005.1; doc impact |
| 9 | `docs/backlog/BL-158-frontmatter-aware-temporary-marker-v2.md` | create | AC-001.12; spec Constraints |
| 10 | `docs/specs/2026-04-18-claude-md-temp-marker-lint/audit-report.md` | create | AC-004.2, AC-004.4 |
| 11 | `CHANGELOG.md` | modify | AC-005.3 |
| 12 | `docs/commands-reference.md` | modify | doc impact |
| 13 | `rules/ecc/github-actions.md` | modify | doc impact |
| 14 | `.github/workflows/ci.yml` | modify | US-006 AC-006.1, .3, .4 |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| `docs/specs/2026-04-18-claude-md-temp-marker-lint/design.md` | Full design (this file) |
| `docs/specs/2026-04-18-claude-md-temp-marker-lint/spec.md` | Spec (persisted prior phase) |
| `docs/specs/2026-04-18-claude-md-temp-marker-lint/campaign.md` | 12 decisions (9 grill-me + 2 spec-adversary rounds + 1 solution-adversary round pair) |
