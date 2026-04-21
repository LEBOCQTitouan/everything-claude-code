# Tasks: Solution: CLAUDE.md `TEMPORARY (BL-NNN)` Marker Lint

## Pass Conditions

- [ ] PC-001: Domain: canonical `TEMPORARY (BL-150)` | ``cargo test -p ecc-domain docs::claude_md::tests::extract_temporary_marker_canonical`` | pending@2026-04-18T11:44:31Z
- [ ] PC-002: Domain: variants (mixed case, `BL-0150`, `BL-100000`) + fence-skip | ``cargo test -p ecc-domain docs::claude_md::tests::extract_temporary_marker_variants`` | pending@2026-04-18T11:44:31Z
- [ ] PC-003: Domain: negative (prose, `BL-`, `BL-ABC`, `TEMPORARY: note`) | ``cargo test -p ecc-domain docs::claude_md::tests::extract_temporary_marker_negative`` | pending@2026-04-18T11:44:31Z
- [ ] PC-004: Domain: duplicate markers → two entries | ``cargo test -p ecc-domain docs::claude_md::tests::extract_temporary_marker_duplicates`` | pending@2026-04-18T11:44:31Z
- [ ] PC-005: Domain: line numbers ascending | ``cargo test -p ecc-domain docs::claude_md::tests::extract_temporary_marker_order`` | pending@2026-04-18T11:44:31Z
- [ ] PC-006: Domain: `matches_backlog_filename` ≤999 (BL-001-foo.md, BL-100.md) | ``cargo test -p ecc-domain backlog::entry::tests::matches_backlog_filename_padded`` | pending@2026-04-18T11:44:31Z
- [ ] PC-007: Domain: `matches_backlog_filename` ≥1000 (BL-1000-bar.md) | ``cargo test -p ecc-domain backlog::entry::tests::matches_backlog_filename_unpadded`` | pending@2026-04-18T11:44:31Z
- [ ] PC-008: Infra regression: `fs_backlog::tests::load_entries_reads_bl_files` passes | ``cargo test -p ecc-infra fs_backlog::tests::load_entries_reads_bl_files`` | pending@2026-04-18T11:44:31Z
- [ ] PC-009: Infra regression: `next_id_computes_max_plus_one` passes | ``cargo test -p ecc-infra fs_backlog::tests::next_id_computes_max_plus_one`` | pending@2026-04-18T11:44:31Z
- [ ] PC-010: Infra regression: `update_entry_status_atomic_write` passes | ``cargo test -p ecc-infra fs_backlog::tests::update_entry_status_atomic_write`` | pending@2026-04-18T11:44:31Z
- [ ] PC-011: App: `disabled=true` → exit true + stderr notice `markers: disabled via ECC_CLAUDE_MD_MARKERS_DISABLED` | ``cargo test -p ecc-app validate_claude_md::tests::markers_kill_switch_emits_notice`` | pending@2026-04-18T11:44:31Z
- [ ] PC-012: App: missing `docs/backlog/` → all markers missing | ``cargo test -p ecc-app validate_claude_md::tests::markers_missing_backlog_dir`` | pending@2026-04-18T11:44:31Z
- [ ] PC-013: App: walker deny-list + symlink-skip | ``cargo test -p ecc-app validate_claude_md::tests::markers_walker_denylist_and_symlink`` | pending@2026-04-18T11:44:31Z
- [ ] PC-014: App: depth cap 16 → WARN | ``cargo test -p ecc-app validate_claude_md::tests::markers_depth_limit`` | pending@2026-04-18T11:44:31Z
- [ ] PC-015: App: non-UTF8 → `WARN: <path>: skipping non-UTF8 file` + skip; I/O error (permission denied) → distinct `WARN: <path>: read error: ...` + skip. Exit code unaffected either way. | ``cargo test -p ecc-app validate_claude_md::tests::markers_non_utf8_and_io_errors_distinguished`` | pending@2026-04-18T11:44:31Z
- [ ] PC-016: App: AGENTS.md walked identically | ``cargo test -p ecc-app validate_claude_md::tests::markers_agents_md_scanned`` | pending@2026-04-18T11:44:31Z
- [ ] PC-017: App: zero markers + no `--strict` → silent stdout | ``cargo test -p ecc-app validate_claude_md::tests::markers_baseline_silent`` | pending@2026-04-18T11:44:31Z
- [ ] PC-018: App: zero missing + `--strict` → success stdout | ``cargo test -p ecc-app validate_claude_md::tests::markers_strict_success`` | pending@2026-04-18T11:44:31Z
- [ ] PC-019: App: missing + `--strict` → `ERROR:` prefix + return false | ``cargo test -p ecc-app validate_claude_md::tests::markers_strict_error_prefix`` | pending@2026-04-18T11:44:31Z
- [ ] PC-020: App: missing + default → `WARN:` prefix + return true | ``cargo test -p ecc-app validate_claude_md::tests::markers_warn_default`` | pending@2026-04-18T11:44:31Z
- [ ] PC-021: App: `--audit-report` emits one row per marker, correct status, archived files = resolved | ``cargo test -p ecc-app validate_claude_md::tests::markers_audit_report_table`` | pending@2026-04-18T11:44:31Z
- [ ] PC-022: App: lexicographic file order + within-file line ordering | ``cargo test -p ecc-app validate_claude_md::tests::markers_file_order_deterministic`` | pending@2026-04-18T11:44:31Z
- [ ] PC-023: CLI: `--counts` emits EXACT `DEPRECATED: use 'ecc validate claude-md counts' (subcommand form); --counts will be removed in the next minor release.` on stderr | ``cargo test -p ecc-integration-tests validate_claude_md_markers::counts_flag_deprecation_warning`` | pending@2026-04-18T11:44:31Z
- [ ] PC-024: CLI: `markers --strict` happy (AC-001.1 BL-156 present) + missing-BL fail (exit 1, `ERROR:` prefix, stderr contains file path AND `:<line>:` AND `BL-999` simultaneously) | ``cargo test -p ecc-integration-tests validate_claude_md_markers::markers_strict_happy_and_fail_message_composition`` | pending@2026-04-18T11:44:31Z
- [ ] PC-025: CLI: `counts --strict` rejected by clap (strict scoped to markers) | ``cargo test -p ecc-integration-tests validate_claude_md_markers::strict_scoped_to_markers`` | pending@2026-04-18T11:44:31Z
- [ ] PC-026: CLI subprocess: `ECC_CLAUDE_MD_MARKERS_DISABLED=1` → exit 0 + stderr notice | ``cargo test -p ecc-integration-tests validate_claude_md_markers::kill_switch_env_subprocess`` | pending@2026-04-18T11:44:31Z
- [ ] PC-027: Existing `ecc backlog next-id/list/update-status` tests unchanged | ``cargo test -p ecc-integration-tests backlog`` | pending@2026-04-18T11:44:31Z
- [ ] PC-028: Post-fix worktree: `ecc validate claude-md markers --strict` → exit 0 + success stdout | ``./target/release/ecc validate claude-md markers --strict`` | pending@2026-04-18T11:44:31Z
- [ ] PC-029: `CLAUDE.md` has zero `TEMPORARY (BL-` occurrences | ``test "$(grep -c 'TEMPORARY (BL-' CLAUDE.md \` | pending@2026-04-18T11:44:31Z
- [ ] PC-030: `.github/workflows/ci.yml` contains markers step | ``grep -q 'ecc validate claude-md markers --strict' .github/workflows/ci.yml`` | pending@2026-04-18T11:44:31Z
- [ ] PC-031: Companion `BL-158` file exists + title | ``test -f docs/backlog/BL-158-frontmatter-aware-temporary-marker-v2.md && grep -q 'Frontmatter-aware TEMPORARY marker validation (v2)' docs/backlog/BL-158-frontmatter-aware-temporary-marker-v2.md`` | pending@2026-04-18T11:44:31Z
- [ ] PC-032: audit-report.md: zero `missing` rows | ``test "$(grep -c '\` | pending@2026-04-18T11:44:31Z
- [ ] PC-033: CHANGELOG.md has Added + Removed + Deprecated under current release | ``grep -E '^### (Added\` | pending@2026-04-18T11:44:31Z
- [ ] PC-034: commands-reference.md documents subcommand + kill switch | ``grep -qE 'claude-md markers\` | pending@2026-04-18T11:44:31Z
- [ ] PC-035: Clippy zero-warning gate | ``cargo clippy --all-targets -- -D warnings`` | pending@2026-04-18T11:44:31Z
- [ ] PC-036: Release build passes | ``cargo build --release`` | pending@2026-04-18T11:44:31Z
- [ ] PC-037: Workspace test suite passes | ``cargo test`` | pending@2026-04-18T11:44:31Z
- [ ] PC-038: Domain I/O purity guard: `ecc-domain/src/docs/claude_md.rs` has no `use std::{fs,io,env,process}` or `tokio` imports | ``! grep -qE 'use std::(fs\` | pending@2026-04-18T11:44:31Z
- [ ] PC-039: Audit-report regression anchor: pre-fix body contains the literal BL-150 row (Before table documents AC-004.2) | ``grep -qE '^\\\` | pending@2026-04-18T11:44:31Z
- [ ] PC-040: App: ANSI/non-printable sanitizer strips `\x1b` + control bytes from emitted paths | ``cargo test -p ecc-app validate_claude_md::tests::markers_path_sanitizer_strips_control_bytes`` | pending@2026-04-18T11:44:31Z
- [ ] PC-041: CLI smoke: `--counts` (legacy), `counts` (subcommand), `markers --strict`, `all` all parse successfully (clap compatibility guard for `Option<Subcommand>` + sibling flag) | ``cargo test -p ecc-integration-tests validate_claude_md_markers::clap_surface_smoke`` | pending@2026-04-18T11:44:31Z

## Post-TDD

- [ ] E2E tests | pending@2026-04-18T11:44:31Z
- [ ] Code review | pending@2026-04-18T11:44:31Z
- [ ] Doc updates | pending@2026-04-18T11:44:31Z
- [ ] Supplemental docs | pending@2026-04-18T11:44:31Z
- [ ] Write implement-done.md | pending@2026-04-18T11:44:31Z
