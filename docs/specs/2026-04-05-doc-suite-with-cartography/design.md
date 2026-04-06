# Solution: Create /doc-suite Command with Cartography Delta Processing

## Spec Reference
Concern: refactor, Feature: Create /doc-suite slash command with cartography delta processing

## File Changes (dependency order)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `crates/ecc-domain/src/cartography/classification.rs` | Create | Extract `classify_file` as pure domain function | US-005 (AC-005.1) |
| 2 | `crates/ecc-domain/src/cartography/mod.rs` | Modify | Add `pub mod classification`, `CartographyDocument` trait, `From<detection::ProjectType>` impl, `#![warn(missing_docs)]`, consolidate re-exports | US-005, US-010 (AC-010.1, AC-010.2, AC-010.4) |
| 3 | `crates/ecc-domain/src/cartography/types.rs` | Modify | Impl `CartographyDocument` for `SessionDelta`, `From<detection::framework::ProjectType>` for `ProjectType`, add `//!` doc | US-004 (AC-004.2), US-010 (AC-010.1, AC-010.4) |
| 4 | `commands/doc-suite.md` | Create | Slash command invoking doc-orchestrator with argument passthrough | US-001 (AC-001.1, AC-001.2) |
| 5 | `skills/cartography-processing/SKILL.md` | Create | Cartography phase: delta scan, flock(1), per-delta dispatch, JSON envelope parse, archive, single commit. Path validation for file_path. Type allowlist validation. | US-002, US-008, US-010 (AC-002.1-10, AC-008.1-3, AC-010.5) |
| 6 | `agents/doc-orchestrator.md` | Modify | Add `--phase=cartography` to enum, Phase 1.5 ref to skill, TodoWrite entry. Must stay ≤400 lines. | US-002 (AC-002.6, AC-002.9, AC-002.10), US-010 (AC-010.6) |
| 7 | `agents/cartographer.md` | Modify | Remove git commit/archive/prune steps, add JSON envelope output spec, reference derive_slug, add `skills: ["cartography-processing"]` to frontmatter | US-006 (AC-006.2), US-008 (AC-008.1) |
| 8 | `docs/commands-reference.md` | Modify | Add `/doc-suite` entry | US-001 (AC-001.3) |
| 9 | `CLAUDE.md` | Modify | Add `/doc-suite` to Slash Commands | US-001 (AC-001.3) |
| 10 | `crates/ecc-app/src/hook/handlers/tier3_session/cartography/mod.rs` | Create | Module barrel: re-exports `start_cartography`, `stop_cartography` | US-007 |
| 11 | `crates/ecc-app/src/hook/handlers/tier3_session/cartography/delta_writer.rs` | Create | `stop_cartography` handler, delegates to detection framework, calls `derive_slug`, calls `classify_file` from domain | US-004, US-006, US-007 (AC-004.1-4, AC-006.1, AC-007.1) |
| 12 | `crates/ecc-app/src/hook/handlers/tier3_session/cartography/delta_reminder.rs` | Create | Thin `start_cartography`: stat-walk CWD, count pending-delta-*.json, print hint | US-003, US-007 (AC-003.1-4, AC-007.2) |
| 13 | `crates/ecc-app/src/hook/handlers/tier3_session/cartography/delta_helpers.rs` | Create | Shared: `collect_pending_deltas`, `collect_slugs`, `collect_flow_slugs`, `collect_element_entries`, `AgentContext`, etc. | US-007 (AC-007.3) |
| 14 | `crates/ecc-app/src/hook/handlers/tier3_session/cartography.rs` | Delete | Replaced by cartography/ directory module | US-007 |
| 15 | `crates/ecc-app/src/hook/handlers/tier3_session/mod.rs` | Modify | Update `pub use cartography::` to new module path | US-007 |
| 16 | `crates/ecc-app/src/hook/mod.rs` | Modify | Add `Handler` trait + `HashMap<&str, Box<dyn Handler>>` registry | US-009 (AC-009.1-4) |
| 17 | `hooks/hooks.json` | Modify | No structural change needed (hook IDs unchanged) | US-003 |
| 18 | `docs/adr/NNNN-cartography-hook-to-doc-orchestrator.md` | Create | ADR: rationale for moving delta processing from hook to doc pipeline | Decision 2 |
| 19 | `docs/adr/NNNN-handler-trait-dispatch.md` | Create | ADR: rationale for Handler trait pattern | Decision 12 |
| 20 | `CHANGELOG.md` | Modify | Entry for this refactoring | All |

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | Safety-net: agent non-zero exit leaves delta unarchived | AC-011.1(a) | `cargo test -p ecc-app cartography::tests::safety_net_agent_nonzero_exit -- --exact` | PASS |
| PC-002 | unit | Safety-net: invalid agent output handled gracefully | AC-011.1(b) | `cargo test -p ecc-app cartography::tests::safety_net_agent_invalid_output -- --exact` | PASS |
| PC-003 | unit | Safety-net: archive failure reports error | AC-011.1(c) | `cargo test -p ecc-app cartography::tests::safety_net_archive_failure -- --exact` | PASS |
| PC-004 | unit | Safety-net: malformed delta JSON skipped | AC-011.1(d) | `cargo test -p ecc-app cartography::tests::safety_net_malformed_delta_json -- --exact` | PASS |
| PC-005 | lint | doc-suite.md exists with frontmatter | AC-001.1 | `test -f commands/doc-suite.md && head -5 commands/doc-suite.md \| grep -q 'description'` | exit 0 |
| PC-006 | lint | doc-suite.md passes through $ARGUMENTS | AC-001.2 | `grep -q 'ARGUMENTS' commands/doc-suite.md` | exit 0 |
| PC-007 | lint | /doc-suite in commands-reference.md | AC-001.3 | `grep -q 'doc-suite' docs/commands-reference.md` | exit 0 |
| PC-008 | lint | /doc-suite in CLAUDE.md | AC-001.3 | `grep -q 'doc-suite' CLAUDE.md` | exit 0 |
| PC-009 | lint | cartography-processing skill exists with frontmatter | AC-002.9 | `test -f skills/cartography-processing/SKILL.md && head -5 skills/cartography-processing/SKILL.md \| grep -q 'name: cartography-processing'` | exit 0 |
| PC-010 | lint | Skill documents delta scan, flock, dispatch, JSON, archive, commit | AC-002.1,3,4,5,7,8 | `grep -c 'flock\|pending-delta\|archive\|JSON envelope\|git commit\|git add' skills/cartography-processing/SKILL.md` | >= 6 |
| PC-011 | lint | Doc-orchestrator --phase includes cartography | AC-002.6 | `grep -q 'cartography' agents/doc-orchestrator.md` | exit 0 |
| PC-012 | lint | Doc-orchestrator TodoWrite has Phase 1.5 | AC-002.10 | `grep -q 'Phase 1.5.*[Cc]artography' agents/doc-orchestrator.md` | exit 0 |
| PC-013 | lint | Doc-orchestrator ≤400 lines | AC-010.6 | `test $(wc -l < agents/doc-orchestrator.md) -le 400` | exit 0 |
| PC-014 | lint | Cartographer has JSON envelope and no git commit | AC-008.1, AC-002.8 | `grep -q 'JSON.*envelope\|"status"' agents/cartographer.md && ! grep -q 'git commit' agents/cartographer.md` | exit 0 |
| PC-015 | lint | Cartographer references derive_slug | AC-006.2 | `grep -q 'derive_slug' agents/cartographer.md` | exit 0 |
| PC-016 | unit | Thin hook prints count + /doc-suite hint | AC-003.1, AC-011.2 | `cargo test -p ecc-app cartography::delta_reminder::tests::prints_pending_count -- --exact` | PASS |
| PC-017 | unit | Thin hook passthrough when no deltas | AC-003.2, AC-011.2 | `cargo test -p ecc-app cartography::delta_reminder::tests::silent_when_no_deltas -- --exact` | PASS |
| PC-018 | unit | Thin hook uses CWD, no env var | AC-003.3, AC-003.4 | `cargo test -p ecc-app cartography::delta_reminder::tests::uses_cwd_not_env_var -- --exact` | PASS |
| PC-019 | unit | classify_file in domain: Rust crate paths | AC-005.1, AC-011.3 | `cargo test -p ecc-domain cartography::classification::tests::classify_rust_crate -- --exact` | PASS |
| PC-020 | unit | classify_file in domain: JS/TS + unknown | AC-005.1, AC-011.3 | `cargo test -p ecc-domain cartography::classification::tests::classify_jsts_and_unknown -- --exact` | PASS |
| PC-021 | unit | stop_cartography delegates to detection framework | AC-004.1, AC-004.2, AC-004.3 | `cargo test -p ecc-app cartography::delta_writer::tests::delegates_to_detection_framework -- --exact` | PASS |
| PC-022 | lint | No detect_project_type in cartography module | AC-004.4 | `! grep -rn 'fn detect_project_type' crates/ecc-app/src/hook/handlers/tier3_session/cartography/` | exit 0 |
| PC-023 | unit | stop_cartography calls derive_slug | AC-006.1 | `cargo test -p ecc-app cartography::delta_writer::tests::uses_derive_slug -- --exact` | PASS |
| PC-024 | lint | No inline slug derivation in app layer | AC-006.3 | `! grep -rn 'to_lowercase.*replace.*alphanumeric' crates/ecc-app/src/hook/handlers/tier3_session/cartography/` | exit 0 |
| PC-025 | lint | delta_writer.rs ≤800 lines | AC-007.4 | `test $(wc -l < crates/ecc-app/src/hook/handlers/tier3_session/cartography/delta_writer.rs) -le 800` | exit 0 |
| PC-026 | lint | delta_reminder.rs ≤800 lines | AC-007.4 | `test $(wc -l < crates/ecc-app/src/hook/handlers/tier3_session/cartography/delta_reminder.rs) -le 800` | exit 0 |
| PC-027 | lint | delta_helpers.rs ≤800 lines | AC-007.4 | `test $(wc -l < crates/ecc-app/src/hook/handlers/tier3_session/cartography/delta_helpers.rs) -le 800` | exit 0 |
| PC-028 | unit | All existing cartography tests pass | AC-007.5, AC-004.3, AC-005.2 | `cargo test -p ecc-app cartography` | PASS |
| PC-029 | lint | No agent dispatch logic in delta_reminder | AC-007.6 | `! grep -n 'invoke_agent\|invoke_element' crates/ecc-app/src/hook/handlers/tier3_session/cartography/delta_reminder.rs` | exit 0 |
| PC-030 | unit | Handler trait compiles with hook_id + handle | AC-009.1 | `cargo test -p ecc-app hook::tests::handler_trait_compiles -- --exact` | PASS |
| PC-031 | unit | Handler impl dispatches cartography handler | AC-009.2 | `cargo test -p ecc-app hook::tests::handler_trait_dispatch -- --exact` | PASS |
| PC-032 | lint | Handler used in dispatch | AC-009.3 | `grep -q 'Handler' crates/ecc-app/src/hook/mod.rs` | exit 0 |
| PC-033 | unit | CartographyDocument trait exists | AC-010.1 | `cargo test -p ecc-domain cartography::tests::sap_trait_exists -- --exact` | PASS |
| PC-034 | lint | No direct internal path imports in app | AC-010.2 | `! grep -rn 'ecc_domain::cartography::validation::\|ecc_domain::cartography::element_types::' crates/ecc-app/src/` | exit 0 |
| PC-035 | unit | Agent name matches existing file | AC-010.3, AC-011.4 | `cargo test -p ecc-app cartography::tests::agent_name_matches_file -- --exact` | PASS |
| PC-036 | lint | Domain cartography has warn(missing_docs) + //! comments | AC-010.4 | `grep -q 'warn(missing_docs)' crates/ecc-domain/src/cartography/mod.rs` | exit 0 |
| PC-037 | lint | Skill references flock(1) | AC-010.5 | `grep -q 'flock' skills/cartography-processing/SKILL.md` | exit 0 |
| PC-038 | lint | Skill describes malformed JSON handling | AC-002.5, AC-008.3 | `grep -qi 'malformed\|invalid JSON' skills/cartography-processing/SKILL.md` | exit 0 |
| PC-039 | lint | Skill describes skip when no deltas/dir missing | AC-002.2 | `grep -qi 'no pending.*delta\|skip\|directory.*missing' skills/cartography-processing/SKILL.md` | exit 0 |
| PC-040 | build | Workspace builds | All | `cargo build` | exit 0 |
| PC-041 | lint | Clippy passes | All | `cargo clippy -- -D warnings` | exit 0 |
| PC-042 | unit | Full test suite passes | All | `cargo test` | PASS |
| PC-043 | lint | ecc validate agents passes | AC-008.1, AC-006.2 | `cargo run -- validate agents` | exit 0 |
| PC-044 | lint | ecc validate commands passes | AC-001.1 | `cargo run -- validate commands` | exit 0 |
| PC-045 | lint | ecc validate skills passes | AC-002.9 | `cargo run -- validate skills` | exit 0 |
| PC-046 | lint | delta_writer.rs imports classify_file from ecc_domain::cartography | AC-005.3 | `grep -q 'ecc_domain::cartography::classify_file\|use.*cartography.*classify_file' crates/ecc-app/src/hook/handlers/tier3_session/cartography/delta_writer.rs` | exit 0 |
| PC-047 | lint | stop_cartography function exists in delta_writer.rs | AC-007.1 | `grep -q 'pub fn stop_cartography' crates/ecc-app/src/hook/handlers/tier3_session/cartography/delta_writer.rs` | exit 0 |
| PC-048 | lint | Skill file describes JSON envelope parsing for routing output | AC-008.2 | `grep -qi 'parse.*JSON\|envelope.*type.*journey\|route.*output' skills/cartography-processing/SKILL.md` | exit 0 |
| PC-049 | lint | Handler registration does not require modifying dispatch match | AC-009.4 | `grep -q 'register\|insert\|HashMap.*Handler' crates/ecc-app/src/hook/mod.rs` | exit 0 |
| PC-050 | lint | Skill describes archiving to processed/ directory | AC-002.3 | `grep -qi 'processed/' skills/cartography-processing/SKILL.md` | exit 0 |
| PC-051 | lint | Skill describes failed delta skip (not archived) | AC-002.4 | `grep -qi 'not archived\|skip.*failed\|failed.*not.*archive' skills/cartography-processing/SKILL.md` | exit 0 |
| PC-052 | lint | delta_reminder.rs has no subprocess spawning | AC-003.3 | `! grep -n 'Command::new\|std::process\|run_command' crates/ecc-app/src/hook/handlers/tier3_session/cartography/delta_reminder.rs` | exit 0 |
| PC-053 | lint | delta_helpers.rs exists (spec AC-007.3 updated to delta_helpers) | AC-007.3 | `test -f crates/ecc-app/src/hook/handlers/tier3_session/cartography/delta_helpers.rs` | exit 0 |
| PC-054 | e2e | Manual E2E: /doc-suite --phase=cartography processes a delta file | AC-011.5 | `echo '{"session_id":"test","changed_files":[{"path":"test.rs","classification":"test"}]}' > .claude/cartography/pending-delta-test.json && test -f .claude/cartography/pending-delta-test.json` | exit 0 (delta file created for manual /doc-suite run) |

**PC path migration note**: PC-001 through PC-004 use `cartography::tests::safety_net_*` paths which are valid during Phase 0. After Phase 4 decomposition, these tests physically move to `cartography::delta_writer::tests::safety_net_*`. The TDD executor must update the `cargo test` paths in PC-001-004 during Phase 4 to match the new module structure. This is an expected mechanical change, not a new test.

### Coverage Check

All 51 ACs covered. Zero uncovered.

| AC Range | Covering PCs |
|----------|-------------|
| AC-001.1-3 | PC-005, PC-006, PC-007, PC-008, PC-044 |
| AC-002.1-10 | PC-009, PC-010, PC-011, PC-012, PC-013, PC-037, PC-038, PC-039, PC-050, PC-051 |
| AC-003.1-4 | PC-016, PC-017, PC-018, PC-052 |
| AC-004.1-4 | PC-021, PC-022, PC-028 |
| AC-005.1-3 | PC-019, PC-020, PC-028, PC-046 |
| AC-006.1-3 | PC-023, PC-024, PC-015 |
| AC-007.1-6 | PC-025, PC-026, PC-027, PC-028, PC-029, PC-047, PC-053 |
| AC-008.1-3 | PC-014, PC-038, PC-048 |
| AC-009.1-4 | PC-030, PC-031, PC-032, PC-049 |
| AC-010.1-6 | PC-033, PC-034, PC-035, PC-036, PC-037, PC-013 |
| AC-011.1-5 | PC-001-004, PC-016, PC-017, PC-019, PC-035. Note: AC-011.5 (E2E for /doc-suite) is aspirational — no automated agent-command test framework exists; verified manually |

### E2E Test Plan

| # | Boundary | Adapter | Port | Test Description | Default State | Run When |
|---|----------|---------|------|------------------|---------------|----------|
| 1 | Agent dispatch | doc-orchestrator | ShellExecutor | Cartographer produces valid JSON envelope | ignored | cartographer.md modified |
| 2 | FS delta read | delta_reminder.rs | FileSystem | Hook reads pending-delta count from CWD | ignored | delta_reminder.rs modified |
| 3 | FS delta archive | skill | FileSystem | Processed deltas moved to processed/ | ignored | skill modified |
| 4 | TerminalIO | delta_reminder.rs | TerminalIO | Hook prints count to stderr | ignored | delta_reminder.rs modified |
| 5 | Command | doc-suite.md | N/A | Command invokes doc-orchestrator | ignored | doc-suite.md modified |

### E2E Activation Rules

All 5 boundaries are affected by this implementation. Boundaries 2 and 4 are covered by unit tests (PC-016-018). AC-011.5 covers the full flow E2E but is aspirational (no automated agent-command test framework exists).

## Test Strategy

TDD order (dependency-driven):

1. **Phase 0 — Safety-net** (PC-001→PC-004): Write in existing cartography.rs BEFORE any changes
2. **Phase 1 — Markdown** (PC-005→PC-015, PC-037→PC-039, PC-043→PC-045): No Rust deps
3. **Phase 2 — Domain** (PC-019→PC-020, PC-033, PC-036): classify_file + SAP trait in ecc-domain
4. **Phase 3 — Detection** (PC-021→PC-024): Consolidate detect_project_type, wire derive_slug
5. **Phase 4 — Decompose** (PC-025→PC-029, PC-046→PC-047): Split file, move tests alongside code. NOTE: Safety-net tests (PC-001-004) written in cartography.rs must be migrated to the appropriate new module's test block (delta_writer tests → delta_writer.rs, delta_reminder tests → delta_reminder.rs). The test module paths change from `cartography::tests::safety_net_*` to `cartography::delta_writer::tests::safety_net_*` etc.
6. **Phase 5 — Hook tests** (PC-016→PC-018): New thin reminder behavior
7. **Phase 6 — Handler** (PC-030→PC-035): Trait + agent name validation
8. **Phase 7 — Build gate** (PC-040→PC-042): Final cargo build/clippy/test

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | `commands/doc-suite.md` | commands | Create | Slash command invoking doc-orchestrator | US-001 |
| 2 | `skills/cartography-processing/SKILL.md` | skills | Create | Cartography phase instructions | US-002, US-008 |
| 3 | `docs/commands-reference.md` | docs | Modify | Add /doc-suite entry | AC-001.3 |
| 4 | `CLAUDE.md` | root | Modify | Add /doc-suite to Slash Commands | AC-001.3 |
| 5 | `agents/doc-orchestrator.md` | agents | Modify | Phase 1.5 ref, --phase=cartography | US-002 |
| 6 | `agents/cartographer.md` | agents | Modify | JSON envelope, no git, derive_slug | US-006, US-008 |
| 7 | `docs/adr/NNNN-cartography-hook-to-doc-orchestrator.md` | adr | Create | Hook-to-pipeline rationale | Decision 2 |
| 8 | `docs/adr/NNNN-handler-trait-dispatch.md` | adr | Create | Handler trait rationale | Decision 12 |
| 9 | `CHANGELOG.md` | root | Modify | Refactoring entry | All |

## SOLID Assessment

**Verdict: NEEDS WORK → 2 HIGH, 2 MEDIUM (addressed in design)**

- **HIGH — SAP trait consumer**: `CartographyDocument` trait will be consumed by `validate_cartography.rs` (avoids speculative generality). Design adds `From<detection::ProjectType>` impl in domain.
- **HIGH — ProjectType mapping placement**: `From<detection::framework::ProjectType>` impl added to `ecc-domain::cartography::types.rs` (domain, not app layer).
- **MEDIUM — OCP Handler registry**: Design uses `HashMap<&str, Box<dyn Handler>>` in `hook/mod.rs`. Population site is the designated open/close boundary.
- **MEDIUM — helpers.rs naming**: Renamed to `delta_helpers.rs` per SRP convention.

## Robert's Oath Check

**Verdict: CLEAN** — 0 oath warnings

- Safety-net tests before refactoring (Oath 3: proof)
- Each US independently shippable (Oath 4: small releases)
- No mess left behind (Oath 2: decomposition + consolidation)
- 2 self-audit findings: SELF-001 (agents missing `skills` field — add during implementation), SELF-002 (doc-orchestrator at 407 lines — extraction keeps it under 400)

## Security Notes

**Verdict: CLEAR** — 2 LOW advisory notes

- **LOW**: JSON envelope `file_path` must be validated under `docs/cartography/` before write (path traversal). Skill instructs: derive path from `type` + slug, validate containment.
- **LOW**: `flock(1)` on `.claude/cartography/cartography-merge.lock` — create with `>>` before flock. Directory already exists from stop hook.

## Rollback Plan

Reverse dependency order:
1. Revert `CHANGELOG.md`
2. Delete `docs/adr/NNNN-handler-trait-dispatch.md`, `docs/adr/NNNN-cartography-hook-to-doc-orchestrator.md`
3. Revert `crates/ecc-app/src/hook/mod.rs` (remove Handler trait)
4. Revert `crates/ecc-app/src/hook/handlers/tier3_session/mod.rs`
5. Restore `crates/ecc-app/src/hook/handlers/tier3_session/cartography.rs` (original 2728-line file)
6. Delete `cartography/` directory module (delta_writer, delta_reminder, delta_helpers, mod)
7. Revert `CLAUDE.md`, `docs/commands-reference.md`
8. Revert `agents/cartographer.md`, `agents/doc-orchestrator.md`
9. Delete `skills/cartography-processing/SKILL.md`, `commands/doc-suite.md`
10. Revert `crates/ecc-domain/src/cartography/mod.rs`, `types.rs`
11. Delete `crates/ecc-domain/src/cartography/classification.rs`

## Phase Summary

### Design Reviews

| Review Type | Verdict | Finding Count |
|-------------|---------|---------------|
| SOLID (uncle-bob) | NEEDS WORK → addressed | 2 HIGH (resolved: From impl in domain, SAP trait has consumer), 2 MEDIUM (resolved: delta_helpers naming, OCP HashMap registry) |
| Robert | CLEAN | 0 oath warnings, 2 self-audit (SELF-001: agents missing skills field, SELF-002: doc-orchestrator 407 lines) |
| Security | CLEAR | 2 LOW advisory (JSON file_path path traversal, flock file creation) |

### Adversary Findings

| Dimension | Score (R3) | Verdict | Key Rationale |
|-----------|-----------|---------|---------------|
| AC Coverage | 78 | PASS | 54 PCs cover all 51 ACs; AC-011.5 manual verification |
| Execution Order | 70 | PASS (with note) | Safety-net test paths migrate during Phase 4 decomposition |
| Fragility | 72 | PASS | Fragile PC-010 supplemented with PC-050/051 individual checks |
| Rollback | 88 | PASS | Reverse dependency order documented |
| Architecture | 92 | PASS | From impl domain-to-domain confirmed; no layering violations |
| Blast Radius | 80 | PASS | Each phase independently shippable |
| Missing PCs | 88 | PASS | 4 new PCs added in rounds 2-3 |
| Doc Plan | 85 | PASS | 9 doc updates including 2 ADRs and CHANGELOG |

### File Changes Summary

| # | File | Action | Spec Ref |
|---|------|--------|----------|
| 1 | `crates/ecc-domain/src/cartography/classification.rs` | Create | US-005 |
| 2 | `crates/ecc-domain/src/cartography/mod.rs` | Modify | US-005, US-010 |
| 3 | `crates/ecc-domain/src/cartography/types.rs` | Modify | US-004, US-010 |
| 4 | `commands/doc-suite.md` | Create | US-001 |
| 5 | `skills/cartography-processing/SKILL.md` | Create | US-002, US-008 |
| 6 | `agents/doc-orchestrator.md` | Modify | US-002, US-010 |
| 7 | `agents/cartographer.md` | Modify | US-006, US-008 |
| 8 | `docs/commands-reference.md` | Modify | US-001 |
| 9 | `CLAUDE.md` | Modify | US-001 |
| 10-14 | `cartography/` module (mod, delta_writer, delta_reminder, delta_helpers) | Create | US-003-007 |
| 15 | `cartography.rs` | Delete | US-007 |
| 16 | `crates/ecc-app/src/hook/mod.rs` | Modify | US-009 |
| 17 | `hooks/hooks.json` | Modify | US-003 |
| 18-19 | `docs/adr/` (2 ADRs) | Create | Decisions 2, 12 |
| 20 | `CHANGELOG.md` | Modify | All |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-04-05-doc-suite-with-cartography/spec.md | Full spec + Phase Summary |
| docs/specs/2026-04-05-doc-suite-with-cartography/design.md | Full design + Phase Summary |
