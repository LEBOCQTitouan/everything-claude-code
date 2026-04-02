# Design: Universal App Cartography System — Sub-Spec A (BL-064)

## File Changes Table

Files are ordered by dependency: domain types → merge/validation/staleness/coverage/slug → app layer (hook handlers, validate use case) → CLI wiring → integration tests → agents/config.

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| F-001 | `crates/ecc-domain/src/cartography/mod.rs` | Create | Root module; re-exports all public domain types and pure functions. Zero I/O. | US-001, US-002 |
| F-002 | `crates/ecc-domain/src/cartography/types.rs` | Create | `SessionDelta`, `ProjectType`, `ChangedFile`, `CartographyMeta` value types with serde derives. | US-001, US-003 |
| F-003 | `crates/ecc-domain/src/cartography/slug.rs` | Create | Pure slug derivation: lowercase, replace non-alphanumeric with hyphens, collapse multiples, truncate at 60 chars. | Decision #11, #13 |
| F-004 | `crates/ecc-domain/src/cartography/merge.rs` | Create | Delta merge algorithm: section-marker insertion, step append/replace, manual-content preservation. Pure string processing, zero I/O. | US-002: AC-002.1, AC-002.2, AC-002.5 |
| F-005 | `crates/ecc-domain/src/cartography/validation.rs` | Create | Schema rules for journey files (required sections: Overview, Mermaid Diagram, Steps, Related Flows) and flow files (Overview, Mermaid Diagram, Source-Destination, Transformation Steps, Error Paths). Returns structured errors listing missing section names. | US-001: AC-001.2, AC-001.3 |
| F-006 | `crates/ecc-domain/src/cartography/staleness.rs` | Create | Pure date-comparison: parse `CARTOGRAPHY-META` marker, compare `last_updated` vs `source_modified`, produce stale annotation string; `remove_stale_marker` strips it. | US-007: AC-007.1, AC-007.3 |
| F-007 | `crates/ecc-domain/src/cartography/coverage.rs` | Create | Pure coverage calculation: given source file list and files referenced in journey/flow content, compute ratio and top-10 priority-gaps list when below 50%. | US-008: AC-008.1, AC-008.2 |
| F-008 | `crates/ecc-domain/src/lib.rs` | Modify | Add `pub mod cartography;` after existing `pub mod backlog;`. | US-001 |
| F-009 | `crates/ecc-app/src/hook/handlers/tier3_session/cartography.rs` | Create | Two handlers: `stop_cartography` (Stop hook — writes `pending-delta-<session_id>.json`) and `start_cartography` (SessionStart hook — scaffolds directories if absent, acquires file lock, reads pending deltas in chronological order, invokes cartographer agent, archives processed deltas). Discard logic for interrupted sessions (dirty `docs/cartography/` via git status) lives in the start handler. | US-001: AC-001.1, AC-001.4; US-002: AC-002.3, AC-002.4; US-003: AC-003.1–003.9; US-006: AC-006.1–006.8 |
| F-010 | `crates/ecc-app/src/hook/handlers/tier3_session/mod.rs` | Modify | Add `mod cartography;` declaration and `pub use cartography::{start_cartography, stop_cartography};`. | US-003, US-006 |
| F-011 | `crates/ecc-app/src/hook/handlers/mod.rs` | Modify | Add `start_cartography, stop_cartography` to the `tier3_session` re-export list. | US-003, US-006 |
| F-012 | `crates/ecc-app/src/hook/mod.rs` | Modify | Add two dispatch arms: `"stop:cartography" => handlers::stop_cartography(stdin, ports)` and `"start:cartography" => handlers::start_cartography(stdin, ports)`. | US-003, US-006 |
| F-013 | `crates/ecc-app/src/validate_cartography.rs` | Create | `run_validate_cartography(fs, shell, terminal, project_root, coverage_flag)` use case: scans `docs/cartography/journeys/` and `docs/cartography/flows/`, validates schema (journey/flow), checks staleness via `git log`, optionally runs coverage dashboard. | US-001: AC-001.2, AC-001.3; US-007: AC-007.2; US-008: AC-008.1–008.3 |
| F-014 | `crates/ecc-app/src/lib.rs` | Modify | Add `pub mod validate_cartography;`. | US-001, US-007, US-008 |
| F-015 | `crates/ecc-cli/src/commands/validate.rs` | Modify | Add `Cartography { #[arg(long)] coverage: bool }` variant to `CliValidateTarget` enum and match arm calling `ecc_app::validate_cartography::run_validate_cartography`. | US-001: AC-001.2, AC-001.3; US-007: AC-007.2; US-008: AC-008.1 |
| F-016 | `crates/ecc-integration-tests/tests/cartography_validate.rs` | Create | Integration tests for `run_validate_cartography`: schema errors, stale entries, coverage dashboard, and 500-file performance bound. Uses `InMemoryFileSystem` + `MockExecutor`. | US-001, US-007, US-008 |
| F-017 | `crates/ecc-integration-tests/tests/cli_validate_cartography.rs` | Create | Integration tests for the CLI `ecc validate cartography` surface: exit codes, output format, `--coverage` flag. Calls `run_validate_cartography` directly via app crate (no subprocess). | US-001, US-007, US-008 |
| F-018 | `hooks/hooks.json` | Modify | Add `stop:cartography` to `Stop` array (async, `"timeout": 10`) for delta writing. Add `start:cartography` to `SessionStart` array for delta processing. | US-003: AC-003.1, AC-003.6; US-006: AC-006.1 |
| F-019 | `agents/cartographer.md` | Create | Orchestrator agent: reads pending delta JSON, decides journey/flow slugs to update, dispatches cartography-journey-generator and cartography-flow-generator as sub-Tasks. Handles commit (scoped to `docs/cartography/`), archive of processed deltas, and `git reset` on commit failure. | US-006: AC-006.1–006.8; US-004; US-005 |
| F-020 | `agents/cartography-journey-generator.md` | Create | Specialized agent: receives delta + existing journey content, generates/updates journey markdown with section markers, Mermaid diagram, Steps, GAP markers for unknown actors. | US-004: AC-004.1–004.6 |
| F-021 | `agents/cartography-flow-generator.md` | Create | Specialized agent: receives delta + existing flow content, generates/updates flow markdown with section markers, Mermaid diagram, Transformation Steps, GAP markers for unknown paths. | US-005: AC-005.1–005.5 |
| F-022 | `commands/spec-dev.md` | Modify | In the actor-identification step: read `docs/cartography/journeys/*.md` if directory exists, extract actor names from each file's `## Overview` or `# Actor:` field as suggestions. Fall through gracefully when directory is absent or files have no actor field. Add note to create a journey entry when a new actor is introduced. | US-009: AC-009.1–009.4 |

---

## Pass Conditions Table

| PC | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | build | All cartography domain types compile with serde derives; zero I/O imports in `ecc-domain` | AC-003.1 (types), Decision #4 | `cargo build -p ecc-domain` | PASS, exit 0 |
| PC-002 | unit | `SessionDelta` round-trips through JSON; `ProjectType` detection: Cargo.toml → `rust`, package.json → `javascript`/`typescript`, neither → `unknown`; `derive_slug` produces lowercase-hyphenated output, collapses hyphens, truncates at 60 chars | AC-003.1, AC-003.2, AC-003.3, AC-003.4; Decision #11 | `cargo test --lib -p ecc-domain cartography` | PASS |
| PC-003 | unit | Merge algorithm: new step appended inside marker preserving surrounding content; existing step (same ID) replaced in-place; manual content outside all markers byte-for-byte preserved | AC-002.1, AC-002.2, AC-002.5 | `cargo test --lib -p ecc-domain cartography::merge` | PASS |
| PC-004 | unit | Journey schema validation: passes all required sections; reports missing section names when any absent | AC-001.2 | `cargo test --lib -p ecc-domain cartography::validation::tests::journey` | PASS |
| PC-005 | unit | Flow schema validation: passes all required sections; reports missing section names when any absent | AC-001.3 | `cargo test --lib -p ecc-domain cartography::validation::tests::flow` | PASS |
| PC-006 | unit | Staleness: returns stale marker string when `last_updated` older than `source_modified`; returns `None` when up-to-date; `remove_stale_marker` strips existing stale annotation | AC-007.1, AC-007.3 | `cargo test --lib -p ecc-domain cartography::staleness` | PASS |
| PC-007 | unit | Coverage: 10 source files, 6 referenced → 60% ratio and correct count; below 50% returns top-10 undocumented files sorted by frequency | AC-008.1, AC-008.2 | `cargo test --lib -p ecc-domain cartography::coverage` | PASS |
| PC-008 | unit | `stop_cartography`: zero committed changes (empty git diff) → passthrough, no delta file written | AC-003.5 | `cargo test --lib -p ecc-app hook::handlers::tier3_session::cartography::tests::no_delta_when_no_changes` | PASS |
| PC-009 | unit | `stop_cartography`: Cargo.toml at root + changed files → delta JSON at `.claude/cartography/pending-delta-<id>.json` with `project_type = "rust"` and crate classification | AC-003.1, AC-003.2 | `cargo test --lib -p ecc-app hook::handlers::tier3_session::cartography::tests::writes_delta_rust_project` | PASS |
| PC-010 | unit | `stop_cartography`: project-type variants: package.json → typescript/javascript; no recognized build file → unknown, files by top-level directory; `CLAUDE_SESSION_ID` absent → fallback ID from timestamp+PID | AC-003.3, AC-003.4, AC-003.7 | `cargo test --lib -p ecc-app hook::handlers::tier3_session::cartography::tests::project_type_variants_and_fallback_id` | PASS |
| PC-011 | unit | `stop_cartography`: no git repo (git diff exits non-zero with "not a git repo") → passthrough + warning; corrupt JSON in `.claude/cartography/` → file deleted, warning logged, current delta written | AC-003.8, AC-003.9 | `cargo test --lib -p ecc-app hook::handlers::tier3_session::cartography::tests::edge_cases_no_git_and_corrupt_delta` | PASS |
| PC-012 | unit | `start_cartography`: no pending deltas → exits immediately, no shell commands invoked | AC-006.5 | `cargo test --lib -p ecc-app hook::handlers::tier3_session::cartography::tests::noop_when_no_pending_deltas` | PASS |
| PC-013 | unit | `start_cartography`: pending deltas + missing scaffold → `docs/cartography/journeys/`, `docs/cartography/flows/`, and README placeholder created; existing scaffold left untouched | AC-001.1, AC-001.4 | `cargo test --lib -p ecc-app hook::handlers::tier3_session::cartography::tests::creates_scaffold_when_missing` | PASS |
| PC-014 | unit | `start_cartography`: dirty `docs/cartography/` state (git status shows uncommitted changes from prior interrupted run) → `git checkout docs/cartography/` invoked before processing | AC-002.3 | `cargo test --lib -p ecc-app hook::handlers::tier3_session::cartography::tests::discards_uncommitted_changes_on_start` | PASS |
| PC-015 | unit | `start_cartography`: file lock already held → processing skipped, pending deltas remain; delta already in `processed/` → skipped (idempotent); deltas passed to agent in chronological order by timestamp field | AC-002.4, AC-006.1, AC-006.6, AC-006.7 | `cargo test --lib -p ecc-app hook::handlers::tier3_session::cartography::tests::lock_idempotency_and_ordering` | PASS |
| PC-016 | unit | `start_cartography`: successful agent run → processed deltas moved to `.claude/cartography/processed/` AFTER agent completes; agent failure → error logged to stderr, deltas NOT archived, `git reset HEAD docs/cartography/` invoked | AC-006.3, AC-006.4, AC-006.8 | `cargo test --lib -p ecc-app hook::handlers::tier3_session::cartography::tests::archive_on_success_and_reset_on_failure` | PASS |
| PC-017 | unit | `dispatch` routes `"stop:cartography"` and `"start:cartography"` to correct handlers (not the unknown-hook path) | AC-003.1, AC-006.1 | `cargo test --lib -p ecc-app hook::tests::dispatches_cartography_hooks` | PASS |
| PC-018 | integration | `run_validate_cartography` reports all journey/flow schema errors across multiple files; exit code non-zero on invalid files | AC-001.2, AC-001.3 | `cargo test --test cartography_validate schema_errors_reported_for_all_files` | PASS |
| PC-019 | integration | `run_validate_cartography` reports stale entries with staleness delta in days | AC-007.2 | `cargo test --test cartography_validate stale_entries_reported_with_delta_days` | PASS |
| PC-020 | integration | `run_validate_cartography` with coverage flag: outputs total source files, referenced files, percentage; below 50% includes "Priority gaps" section with top-10 undocumented files | AC-008.1, AC-008.2 | `cargo test --test cartography_validate coverage_flag_outputs_dashboard` | PASS |
| PC-021 | integration | `run_validate_cartography` with coverage flag on 500-file fixture completes in under 5 seconds | AC-008.3 | `cargo test --test cartography_validate coverage_completes_within_timeout` | PASS |
| PC-022 | integration | CLI `ecc validate cartography` on valid journey + flow files exits 0; on a file with missing sections exits 1 and prints section name in error | AC-001.2, AC-001.3 | `cargo test --test cli_validate_cartography exits_zero_and_one_for_valid_and_invalid` | PASS |
| PC-023 | integration | CLI `ecc validate cartography --coverage` prints coverage percentage line | AC-008.1 | `cargo test --test cli_validate_cartography coverage_flag_prints_percentage` | PASS |
| PC-024 | lint | `ecc validate agents` passes on all three new agent files (cartographer, cartography-journey-generator, cartography-flow-generator) — validates frontmatter and required fields | AC-004.1 (agent skeleton), AC-005.1 (agent skeleton) | `cargo run --bin ecc -- validate agents` | exit 0 |
| PC-025 | lint | `hooks/hooks.json` valid JSON contains `stop:cartography` in Stop section (async, timeout 10) and `start:cartography` in SessionStart section | AC-003.1, AC-003.6, AC-006.1 | `jq '.hooks.Stop[] | select(.hooks[0].command | contains("stop:cartography"))' hooks/hooks.json && jq '.hooks.SessionStart[] | select(.hooks[0].command | contains("start:cartography"))' hooks/hooks.json` | Non-empty output, exit 0 |
| PC-026 | lint | `commands/spec-dev.md` contains cartography actor-reading logic (journey file path reference) | AC-009.1, AC-009.2, AC-009.3, AC-009.4 | `grep -q 'cartography/journeys' commands/spec-dev.md` | exit 0 |
| PC-027 | lint | `cargo clippy -- -D warnings` across entire workspace | All — code quality | `cargo clippy -- -D warnings` | exit 0 |
| PC-028 | build | Release build succeeds for all workspace crates | All — code quality | `cargo build --release` | exit 0 |

| PC-029 | unit | `start_cartography`: agent dispatch passes delta context including existing journey content for delta-merge; mock agent returns updated content with new steps appended inside markers | AC-004.2 | `cargo test --lib -p ecc-app hook::handlers::tier3_session::cartography::tests::agent_receives_existing_content_for_merge` | PASS |
| PC-030 | unit | `start_cartography`: agent output journey file contains `## Mermaid Diagram` section and `## Steps` section (skeleton validation before write) | AC-004.3 | `cargo test --lib -p ecc-app hook::handlers::tier3_session::cartography::tests::agent_output_validates_journey_schema` | PASS |
| PC-031 | unit | `start_cartography`: agent output journey file contains relative path links to flow files when flows exist in `docs/cartography/flows/` | AC-004.4 | `cargo test --lib -p ecc-app hook::handlers::tier3_session::cartography::tests::journey_links_to_flows` | PASS |
| PC-032 | unit | `start_cartography`: on first run with no existing journeys, only delta-referenced files get journey entries — no full project scan | AC-004.5 | `cargo test --lib -p ecc-app hook::handlers::tier3_session::cartography::tests::no_backfill_on_first_run` | PASS |
| PC-033 | unit | `start_cartography`: agent output includes `<!-- GAP: ... -->` markers for unknown actors/triggers | AC-004.6 | `cargo test --lib -p ecc-app hook::handlers::tier3_session::cartography::tests::gap_markers_for_unknown_actors` | PASS |
| PC-034 | unit | `start_cartography`: agent dispatch for flows includes external I/O detection (file/HTTP/DB patterns in changed files) | AC-005.2 | `cargo test --lib -p ecc-app hook::handlers::tier3_session::cartography::tests::flow_captures_external_io` | PASS |
| PC-035 | unit | `start_cartography`: agent output flow file contains `## Mermaid Diagram` and `## Transformation Steps` sections | AC-005.3 | `cargo test --lib -p ecc-app hook::handlers::tier3_session::cartography::tests::agent_output_validates_flow_schema` | PASS |
| PC-036 | unit | `start_cartography`: flow delta-merge only updates changed steps inside markers; unchanged steps preserved | AC-005.4 | `cargo test --lib -p ecc-app hook::handlers::tier3_session::cartography::tests::flow_delta_merge_preserves_unchanged` | PASS |
| PC-037 | unit | `start_cartography`: commit command uses `git add docs/cartography/` specifically — never `git add .` or `git add -A` | AC-006.2 | `cargo test --lib -p ecc-app hook::handlers::tier3_session::cartography::tests::commit_stages_only_cartography_dir` | PASS |

**Total: 37 PCs** covering all 47 ACs

---

## TDD Order

### Phase 1 — Domain Types, Slug, and Serialization (PCs 001–002)

**First because**: `SessionDelta`, `ProjectType`, and `derive_slug` are imported by every other module. No upstream dependencies.

- PC-001: `cargo build -p ecc-domain`
- PC-002: `SessionDelta` JSON, `ProjectType` detection, `derive_slug`

**Files**: F-001, F-002, F-003, F-008

---

### Phase 2 — Merge Algorithm (PC-003)

**Second because**: self-contained pure algorithm. Stop and start handlers both call it. No I/O.

- PC-003: append/replace/preserve merge

**Files**: F-004

---

### Phase 3 — Validation, Staleness, Coverage (PCs 004–007)

**Third because**: pure domain logic called by the validate use case (Phase 5). No I/O dependencies.

- PC-004: journey schema
- PC-005: flow schema
- PC-006: staleness detection and marker removal
- PC-007: coverage ratio and gaps

**Files**: F-005, F-006, F-007

---

### Phase 4 — Stop Hook Handler (PCs 008–011)

**Fourth because**: writes delta files; the simplest of the two handlers. Proves the full delta-writing path works before building the start handler that reads those files.

- PC-008: no delta when no changes
- PC-009: rust project delta
- PC-010: project-type variants and fallback ID
- PC-011: no-git and corrupt-delta edge cases

**Files**: F-009 (stop_cartography function), F-010, F-011, F-012 (stop arm only)

---

### Phase 5 — Start Hook Handler and Dispatch (PCs 012–017)

**Fifth because**: start handler reads deltas written by Phase 4, calls domain types from Phases 1–3, and invokes agents via shell.

- PC-012: noop when no pending deltas
- PC-013: scaffold creation
- PC-014: discard uncommitted changes (AC-002.3)
- PC-015: lock, idempotency, and ordering
- PC-016: archive on success, reset on failure
- PC-017: dispatch routes both hooks

**Files**: F-009 (start_cartography function), F-012 (start arm)

---

### Phase 6 — Validate Use Case and CLI (PCs 018–023)

**Sixth because**: calls domain validation + staleness + coverage functions (all tested in Phase 3). CLI is a thin wiring layer.

- PC-018: schema errors in integration test
- PC-019: stale entries reported
- PC-020: coverage dashboard
- PC-021: coverage performance (500 files)
- PC-022: CLI exits 0/1
- PC-023: CLI --coverage flag

**Files**: F-013, F-014, F-015, F-016, F-017

---

### Phase 7 — Agents, Config, and Quality Gates (PCs 024–028)

**Last because**: agents are markdown (no Rust compile dependency); hooks.json and spec-dev.md are config/command files. Final lint and build are gates after all code is written.

- PC-024: validate agents
- PC-025: hooks.json has both hooks
- PC-026: spec-dev.md actor-reading logic present
- PC-027: clippy clean
- PC-028: release build

**Files**: F-018, F-019, F-020, F-021, F-022

---

## Layer Declaration per Phase

| Phase | Layers Touched |
|-------|---------------|
| Phase 1 (domain types + slug) | Entity |
| Phase 2 (merge algorithm) | Entity |
| Phase 3 (validation, staleness, coverage) | Entity |
| Phase 4 (stop hook handler) | UseCase, Adapter |
| Phase 5 (start hook handler + dispatch) | UseCase, Adapter |
| Phase 6 (validate use case + CLI) | UseCase, Framework |
| Phase 7 (agents, config, gates) | Framework |

No phase crosses more than 2 layer boundaries.

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | `docs/adr/NNN-two-phase-cartography.md` | ADR | Create | Decision 1: Stop hook delta + next session processing | Decision 1 |
| 2 | `docs/adr/NNN-session-scoped-deltas.md` | ADR | Create | Decision 2: One delta file per session | Decision 2 |
| 3 | `docs/adr/NNN-no-cartography-store-port.md` | ADR | Create | Decision 3: FileSystem + FileLock | Decision 3 |
| 4 | `docs/adr/NNN-cartography-bounded-context.md` | ADR | Create | Decision 4: Independent bounded context | Decision 4 |
| 5 | `docs/adr/NNN-cartography-agent-invocation.md` | ADR | Create | Decision 10: Blocking SessionStart + background Task | Decision 10 |
| 6 | `CLAUDE.md` | Root | Edit | Add `ecc validate cartography` to CLI commands | US-001 |
| 7 | `docs/domain/bounded-contexts.md` | Domain | Edit | Add cartography bounded context | Decision 4 |
| 8 | `CHANGELOG.md` | Root | Edit | Add cartography feature entry | All |
| 9 | `docs/ARCHITECTURE.md` | Root | Edit | Add cartography bounded context to domain module list | Decision 4 |

## SOLID Assessment

PASS with 2 clarifications:
1. SessionDelta independence: confirmed — `session_id` is a plain `String`, not a `workflow::WorkflowState` type. App layer bridges env vars to domain types.
2. JSON serialization: via serde derives on domain types (same pattern as `WorkflowState`, `BacklogEntry`). No port violation — serde is a data format library, not an I/O framework.

## Robert's Oath Check

CLEAN — pure additive feature. No harmful code, no mess. Test coverage planned (28 PCs). Small atomic releases (7 TDD phases). No existing behavior modified.

## Security Notes

CLEAR with 3 MEDIUM findings incorporated into design:
1. Path validation: normalize all file paths to relative, reject `..` and symlinks (in stop_cartography handler)
2. Shell injection: use `Command::new("git").args(["diff", "--name-only"])` — never shell string concatenation
3. Stale lock handling: 5-minute TTL on cartography-merge lock, auto-release via RAII drop

## Rollback Plan

Reverse dependency order:
1. Revert `commands/spec-dev.md` changes (US-009)
2. Remove agent files (cartographer.md, journey-generator.md, flow-generator.md)
3. Remove hooks.json entries (stop:cartography, start:cartography)
4. Revert `ecc-cli/src/commands/validate.rs` changes
5. Remove `ecc-app/src/validate_cartography.rs`
6. Revert `ecc-app/src/hook/mod.rs` dispatch arms
7. Remove `ecc-app/src/hook/handlers/tier3_session/cartography.rs`
8. Remove `ecc-domain/src/cartography/` module
9. Revert `ecc-domain/src/lib.rs` `pub mod cartography;`
10. Remove integration tests (cartography_validate.rs, cli_validate_cartography.rs)
11. Remove 5 ADRs (two-phase-cartography, session-scoped-deltas, no-cartography-store-port, cartography-bounded-context, cartography-agent-invocation)
12. Revert CLAUDE.md, ARCHITECTURE.md, bounded-contexts.md, CHANGELOG.md changes
13. Remove `.claude/cartography/` runtime artifacts (pending deltas, processed directory)
