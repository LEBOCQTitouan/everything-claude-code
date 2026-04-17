# Solution: Backlog Status Conformance Fix

## Spec Reference
Concern: fix, Feature: backlog-status-conformance-fix

## File Changes (dependency order)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `crates/ecc-domain/src/backlog/entry.rs` | Modify | Add `replace_frontmatter_status()` pure function, `BacklogStatus::from_kebab()`, `VALID_STATUSES` const | US-001: AC-001.1, AC-001.4, AC-001.5, AC-001.6, AC-001.7 |
| 2 | `crates/ecc-ports/src/backlog.rs` | Modify | Add `update_entry_status()` and `read_entry_content()` to `BacklogEntryStore` trait | US-001: AC-001.1 |
| 3 | `crates/ecc-test-support/src/in_memory_backlog.rs` | Modify | Implement new trait methods + `raw_contents` field for content-level testing | US-001: AC-001.1 |
| 4 | `crates/ecc-infra/src/fs_backlog.rs` | Modify | Implement `update_entry_status` (locate + read + replace + atomic write) and `read_entry_content` | US-001: AC-001.1 |
| 5 | `crates/ecc-app/src/backlog.rs` | Modify | Add `update_status()` use case, `migrate_statuses()`, `MigrationReport`, `parse_index_statuses()`, reindex safety check, ERR-004 fix | US-001-004: all ACs |
| 6 | `crates/ecc-cli/src/commands/backlog.rs` | Modify | Add `UpdateStatus`, `Migrate` variants + `--force` on `Reindex` + exit codes | US-001: AC-001.3, AC-001.4; US-003: AC-003.1, AC-003.2 |
| 7 | `agents/backlog-curator.md` | Modify | Replace manual status edits with `ecc backlog update-status` CLI reference | Doc impact |
| 8 | `skills/backlog-management/SKILL.md` | Modify | Document `implemented` transition + CLI command | Doc impact |
| 9 | `CLAUDE.md` | Modify | Add `ecc backlog update-status` and `ecc backlog migrate` to CLI commands | Doc impact |
| 10 | `CHANGELOG.md` | Modify | Add fix entry for backlog status sync | Doc impact |

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | replace_frontmatter_status updates status line, preserves body | AC-001.1 | `cargo test -p ecc-domain -- backlog::entry::tests::replace_frontmatter_status_updates_status` | PASS |
| PC-002 | unit | No-op when status already matches | AC-001.5 | `cargo test -p ecc-domain -- backlog::entry::tests::replace_frontmatter_status_noop_same_status` | PASS |
| PC-003 | unit | Error when no status: field in frontmatter | AC-001.6 | `cargo test -p ecc-domain -- backlog::entry::tests::replace_frontmatter_status_missing_status_field` | PASS |
| PC-004 | unit | Updates only first status: on duplicate keys | AC-001.7 | `cargo test -p ecc-domain -- backlog::entry::tests::replace_frontmatter_status_duplicate_keys` | PASS |
| PC-005 | unit | Strips YAML quotes from status value | AC-002.4 | `cargo test -p ecc-domain -- backlog::entry::tests::replace_frontmatter_status_strips_quotes` | PASS |
| PC-006 | unit | Does not modify status: lines in body after --- | AC-001.1 | `cargo test -p ecc-domain -- backlog::entry::tests::replace_frontmatter_status_ignores_body_status` | PASS |
| PC-007 | unit | from_kebab returns Some for 5 valid, None for unknown | AC-001.4 | `cargo test -p ecc-domain -- backlog::entry::tests::from_kebab_valid_and_invalid` | PASS |
| PC-008 | unit | update_status errors on invalid BL id | AC-001.3 | `cargo test -p ecc-app -- backlog::tests::update_status_invalid_id` | PASS |
| PC-009 | unit | update_status errors on invalid status string | AC-001.4 | `cargo test -p ecc-app -- backlog::tests::update_status_invalid_status` | PASS |
| PC-010 | unit | update_status succeeds and triggers reindex | AC-001.2 | `cargo test -p ecc-app -- backlog::tests::update_status_success_triggers_reindex` | PASS |
| PC-011 | unit | update_status no-op for same status | AC-001.5 | `cargo test -p ecc-app -- backlog::tests::update_status_noop_same_status` | PASS |
| PC-012 | unit | parse_index_statuses extracts id→status map | AC-002.1 | `cargo test -p ecc-app -- backlog::tests::parse_index_statuses_extracts_map` | PASS |
| PC-013 | unit | migrate_statuses computes dynamic divergence | AC-002.1, AC-002.2, AC-002.3 | `cargo test -p ecc-app -- backlog::tests::migrate_statuses_dynamic_divergence` | PASS |
| PC-014 | unit | Migration handles partial failure (best-effort) | AC-002.7 | `cargo test -p ecc-app -- backlog::tests::migrate_statuses_partial_failure` | PASS |
| PC-015 | unit | MigrationReport has updated/skipped/failed fields | AC-002.6, AC-002.7 | `cargo test -p ecc-app -- backlog::tests::migrate_statuses_report_structure` | PASS |
| PC-016 | unit | Reindex blocks >5 changes without force | AC-003.1 | `cargo test -p ecc-app -- backlog::tests::reindex_safety_blocks_without_force` | PASS |
| PC-017 | unit | Reindex allows with force despite >5 changes | AC-003.2 | `cargo test -p ecc-app -- backlog::tests::reindex_safety_allows_with_force` | PASS |
| PC-018 | unit | Reindex no warning when <=5 changes | AC-003.1 | `cargo test -p ecc-app -- backlog::tests::reindex_no_warning_under_threshold` | PASS |
| PC-019 | unit | Lock removal failure logged via tracing::warn | AC-004.1 | `cargo test -p ecc-app -- backlog::tests::lock_removal_failure_logged` | PASS |
| PC-020 | unit | InMemoryBacklog update_entry_status roundtrip | AC-001.1 | `cargo test -p ecc-test-support -- in_memory_backlog::tests::update_entry_status_roundtrip` | PASS |
| PC-021 | integration | FsBacklog update_entry_status atomic write | AC-001.1 | `cargo test -p ecc-infra -- fs_backlog::tests::update_entry_status_atomic_write` | PASS |
| PC-022 | integration | FsBacklog update_entry_status preserves body | AC-001.1 | `cargo test -p ecc-infra -- fs_backlog::tests::update_entry_status_preserves_body` | PASS |
| PC-023 | integration | CLI update-status valid args exit 0 | AC-001.2 | `cargo test -p ecc-integration-tests -- backlog_update_status_valid` | PASS |
| PC-024 | integration | CLI update-status invalid id exit 1 | AC-001.3 | `cargo test -p ecc-integration-tests -- backlog_update_status_invalid_id` | PASS |
| PC-025 | integration | CLI update-status invalid status exit 1 | AC-001.4 | `cargo test -p ecc-integration-tests -- backlog_update_status_invalid_status` | PASS |
| PC-026 | integration | CLI reindex safety exit 2 without force | AC-003.1 | `cargo test -p ecc-integration-tests -- backlog_reindex_safety_exit_2` | PASS |
| PC-027 | integration | CLI reindex --force proceeds | AC-003.2 | `cargo test -p ecc-integration-tests -- backlog_reindex_force_proceeds` | PASS |
| PC-028 | migration | After migration, reindex dry-run matches index | AC-002.5 | `cargo test -p ecc-app -- backlog::tests::migration_idempotent_proof` | PASS |
| PC-029 | migration | Quoting normalized to unquoted | AC-002.4 | `cargo test -p ecc-app -- backlog::tests::migration_normalizes_quoting` | PASS |
| PC-030 | lint | clippy zero warnings | Build | `cargo clippy -- -D warnings` | exit 0 |
| PC-031 | lint | fmt check passes | Build | `cargo fmt --check` | exit 0 |
| PC-032 | build | Full workspace tests pass | Build | `cargo test` | exit 0 |

### Coverage Check

All 18 ACs covered:

| AC | Covering PCs |
|----|-------------|
| AC-001.1 | PC-001, PC-006, PC-020, PC-021, PC-022, PC-023 |
| AC-001.2 | PC-010, PC-023 |
| AC-001.3 | PC-008, PC-024 |
| AC-001.4 | PC-007, PC-009, PC-025 |
| AC-001.5 | PC-002, PC-011 |
| AC-001.6 | PC-003 |
| AC-001.7 | PC-004 |
| AC-002.1 | PC-012, PC-013 |
| AC-002.2 | PC-013 |
| AC-002.3 | PC-013 |
| AC-002.4 | PC-005, PC-029 |
| AC-002.5 | PC-028 |
| AC-002.6 | PC-015 |
| AC-002.7 | PC-014, PC-015 |
| AC-002.8 | Manual (atomic git commit) |
| AC-003.1 | PC-016, PC-018, PC-026 |
| AC-003.2 | PC-017, PC-027 |
| AC-004.1 | PC-019 |

**Zero uncovered ACs.**

### E2E Test Plan

| # | Boundary | Adapter | Port | Test Description | Default State | Run When |
|---|----------|---------|------|------------------|---------------|----------|
| 1 | update_entry_status | FsBacklogRepository | BacklogEntryStore | Atomic write + body preservation | ignored | fs_backlog.rs modified |
| 2 | read_entry_content | FsBacklogRepository | BacklogEntryStore | Raw content read by id prefix | ignored | fs_backlog.rs modified |
| 3 | CLI update-status | ecc-cli | BacklogAction | Exit codes 0/1 for valid/invalid | ignored | backlog CLI modified |
| 4 | CLI reindex --force | ecc-cli | BacklogAction | Exit code 2 vs 0 based on force | ignored | reindex logic modified |

### E2E Activation Rules

All 4 boundaries activated — PC-021 through PC-027 cover them.

## Test Strategy

TDD order (dependency-driven):

1. **PC-001 through PC-007** (domain) — Pure function tests first. No dependencies.
2. **PC-020** (test-support) — In-memory double. Depends on port trait.
3. **PC-021, PC-022** (infra) — FS adapter tests. Depends on domain + port.
4. **PC-008 through PC-011, PC-019** (app: update_status + ERR-004) — Use case tests. Depends on port + test double.
5. **PC-012 through PC-018, PC-028, PC-029** (app: migration + reindex safety) — Depends on update_status.
6. **PC-023 through PC-027** (integration: CLI) — End-to-end CLI tests. Depends on all layers.
7. **PC-030, PC-031, PC-032** (lint + build) — Final verification.

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | `CLAUDE.md` | Project | Modify | Add `ecc backlog update-status BL-NNN <status>` and `ecc backlog migrate` to CLI commands | US-001, US-002 |
| 2 | `CHANGELOG.md` | Project | Modify | Fix entry: backlog status sync, CLI commands, migration, reindex safety | All US |
| 3 | `skills/backlog-management/SKILL.md` | Skill | Modify | Add `implemented` to transitions, reference `ecc backlog update-status` | US-001 |
| 4 | `agents/backlog-curator.md` | Agent | Modify | Replace manual edits with CLI command reference | US-001 |

No ADRs needed.

## SOLID Assessment

**PASS** (2 findings, none blocking):

| ID | Severity | Principle | Finding |
|----|----------|-----------|---------|
| OCP-001 | MEDIUM | OCP | `reindex()` gains `force: bool` parameter (modification, not extension). Acceptable: few call sites, internal function. |
| ISP-001 | LOW | ISP | Two new methods on `BacklogEntryStore` force 3 implementors to update. Cohesive with aggregate. |

## Robert's Oath Check

**CLEAN**. No harmful code, no mess, proof via 32 PCs, small atomic releases per TDD phase.

## Security Notes

**CLEAR** (1 LOW finding):

| ID | Severity | Finding |
|----|----------|---------|
| SEC-001 | LOW | `replace_frontmatter_status` receives `&str` — callers pre-validate via `from_kebab()`. Defense-in-depth: function should assert alphanumeric+hyphen input. |

## Rollback Plan

Reverse dependency order:
1. Revert `CHANGELOG.md`, `CLAUDE.md`, skill, agent doc changes
2. Revert `crates/ecc-cli/src/commands/backlog.rs` (remove UpdateStatus, Migrate, --force)
3. Revert `crates/ecc-app/src/backlog.rs` (remove update_status, migrate_statuses, safety check, ERR-004 fix)
4. Revert `crates/ecc-infra/src/fs_backlog.rs` (remove update_entry_status, read_entry_content)
5. Revert `crates/ecc-test-support/src/in_memory_backlog.rs` (remove raw_contents, new methods)
6. Revert `crates/ecc-ports/src/backlog.rs` (remove trait methods)
7. Revert `crates/ecc-domain/src/backlog/entry.rs` (remove replace_frontmatter_status, from_kebab, VALID_STATUSES)
8. If migration was applied: `git revert <migration-commit>` restores all 147 files

## Bounded Contexts Affected

| Context | Role | Files Modified |
|---------|------|----------------|
| backlog | Entity + Service | `crates/ecc-domain/src/backlog/entry.rs` |

Other domain modules (not registered as bounded contexts):
- None — all changes outside domain are in ports/infra/app/CLI layers

## Phase Summary

### Design Reviews

| Review Type | Verdict | Finding Count |
|-------------|---------|---------------|
| SOLID | PASS | 2 (OCP-001 MEDIUM, ISP-001 LOW) |
| Robert | CLEAN | 0 |
| Security | CLEAR | 1 (SEC-001 LOW) |

### Adversary Findings

| Dimension | Score | Verdict | Key Rationale |
|-----------|-------|---------|---------------|
| AC Coverage | 92 | PASS | All 18 ACs covered, zero gaps |
| Execution Order | 95 | PASS | Dependency-driven TDD order correct |
| Fragility | 75 | PASS | PC-013 overloaded (non-blocking), tracing assertion noted |
| Rollback | 88 | PASS | Reverse dependency order documented, migration revertible |
| Architecture | 95 | PASS | Hexagonal boundaries respected, domain pure |
| Blast Radius | 78 | PASS | 6 crates touched but contained, StubEntries noted |
| Missing PCs | 90 | PASS | read_entry_content missing dedicated PC (non-blocking) |
| Doc Plan | 85 | PASS | CHANGELOG + CLAUDE.md + skill + agent all covered |

### File Changes Summary

| # | File | Action | Spec Ref |
|---|------|--------|----------|
| 1 | crates/ecc-domain/src/backlog/entry.rs | modify | US-001 |
| 2 | crates/ecc-ports/src/backlog.rs | modify | US-001 |
| 3 | crates/ecc-test-support/src/in_memory_backlog.rs | modify | US-001 |
| 4 | crates/ecc-infra/src/fs_backlog.rs | modify | US-001 |
| 5 | crates/ecc-app/src/backlog.rs | modify | US-001-004 |
| 6 | crates/ecc-cli/src/commands/backlog.rs | modify | US-001, US-003 |
| 7 | agents/backlog-curator.md | modify | Doc impact |
| 8 | skills/backlog-management/SKILL.md | modify | Doc impact |
| 9 | CLAUDE.md | modify | Doc impact |
| 10 | CHANGELOG.md | modify | Doc impact |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-04-17-backlog-status-conformance-fix/spec.md | Full spec |
| docs/specs/2026-04-17-backlog-status-conformance-fix/design.md | Full design |
| docs/specs/2026-04-17-backlog-status-conformance-fix/campaign.md | Campaign manifest |
| docs/audits/backlog-conformance-2026-04-17.md | Conformance audit report |
