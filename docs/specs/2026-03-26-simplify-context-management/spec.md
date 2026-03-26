# Spec: Simplify Context Management — Remove Graceful Exit Infrastructure (BL-060)

## Problem Statement

The graceful exit system (BL-054, BL-055, ADR-0014) over-engineers context management with custom 85%/75% thresholds, Resumption Pointer state serialization, `read-context-percentage.sh` shell scripts, and context checkpoint boilerplate in every phase of `/implement` and `/audit-full`. Claude Code now has a native feature: accepting a plan automatically clears context for a fresh window. This native behavior replaces the custom infrastructure, which adds complexity, doesn't match user expectations (the exit behavior is surprising), and has fragile state serialization. BL-060 removes the graceful exit infrastructure and relies on native plan-accept context clear.

## Research Summary

Web research skipped — pure deletion refactoring with clear scope from backlog item. No external patterns needed.

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Remove graceful exit skill and scripts entirely | Replaced by native plan-accept context clear | No |
| 2 | Remove strategic compact skill | Tightly coupled to graceful exit (defers to it at 85%+) | No |
| 3 | Supersede ADR-0014 (not delete) | Preserve decision history; mark as Superseded | No |
| 4 | Remove Resumption Pointer from campaign manifest | tasks.md is the authoritative resume source; Resumption Pointer adds no value without graceful exit | No |
| 5 | Remove context checkpoints from all command phases | Boilerplate that called read-context-percentage.sh; no longer needed | No |
| 6 | Keep informational context warnings | Commands may still display "context at X%" as informational text (no behavioral impact) | No |
| 7 | Remove re-entry logic from audit-full | Skip-completed-domains and partial-results cleanup were part of graceful exit recovery | No |
| 8 | Delete BL-055 spec/design/tasks artifacts | Historical artifacts for the removed feature | No |
| 9 | Keep `suggest-compact` Rust hook | Independent feature (counts tool calls, suggests /compact). Not part of graceful exit. Works without graceful-exit skill. | No |
| 10 | Keep `statusline/context-persist.sh` | Writes context % to temp file for statusline display. Independent of graceful exit reader script. | No |
| 11 | Update campaign manifest Malformed Recovery to remove Resumption Pointer from required headers | Without the section in schema, validation should not require it | No |

## User Stories

### US-001: Delete graceful exit infrastructure files

**As a** maintainer, **I want** the graceful exit skill, scripts, and spec artifacts deleted, **so that** the codebase has no dead infrastructure.

#### Acceptance Criteria

- AC-001.1: Given `skills/graceful-exit/SKILL.md`, when deleted, then no file exists at that path
- AC-001.2: Given `skills/graceful-exit/read-context-percentage.sh`, when deleted, then no file exists
- AC-001.3: Given `skills/strategic-compact/SKILL.md`, when deleted, then no file exists
- AC-001.4: Given `docs/specs/2026-03-23-graceful-mid-session-exit/`, when deleted, then no directory exists
- AC-001.5: Given the deleted files, when `grep -rl 'graceful-exit/SKILL.md' commands/ skills/ agents/ docs/ hooks/` is run (excluding `docs/specs/` and `docs/audits/`), then 0 results
- AC-001.6: Given the deleted files, when `grep -rl 'read-context-percentage' commands/ skills/ agents/ docs/ hooks/` is run (excluding `docs/specs/` and `docs/audits/`), then 0 results
- AC-001.7: Given the deleted files, when `grep -rl 'strategic-compact/SKILL.md' commands/ skills/ agents/ docs/ hooks/` is run (excluding `docs/specs/` and `docs/audits/`), then 0 results

#### Dependencies

- Depends on: none

### US-002: Remove context checkpoints from /implement

**As a** developer running `/implement`, **I want** context checkpoints removed from all phases, **so that** the command is simpler and doesn't trigger surprising exit behavior.

#### Acceptance Criteria

- AC-002.1: Given `commands/implement.md`, when checked for "Context Checkpoint" blockquotes, then 0 occurrences
- AC-002.2: Given `commands/implement.md`, when checked for `read-context-percentage`, then 0 occurrences
- AC-002.3: Given `commands/implement.md`, when checked for "85%" exit logic, then 0 occurrences
- AC-002.4: Given `commands/implement.md`, when checked for Phase 0 step 7 (Context Clear Gate / BL-054), then that step is removed
- AC-002.5: Given `commands/implement.md`, when checked for "Resumption Pointer", then 0 occurrences
- AC-002.6: Given `commands/implement.md Phase 0 step 6`, when checked, then campaign re-entry orientation still exists but without Resumption Pointer references

#### Dependencies

- Depends on: US-001

### US-003: Remove graceful exit from audit commands

**As a** developer running `/audit-full`, **I want** re-entry and cleanup logic removed, **so that** the audit flow is simpler.

#### Acceptance Criteria

- AC-003.1: Given `commands/audit-full.md`, when checked for "Graceful Exit Recovery" re-entry section, then removed
- AC-003.2: Given `commands/audit-full.md`, when checked for "Cleanup Partial Results" section, then removed
- AC-003.3: Given `agents/audit-orchestrator.md`, when checked for "graceful-exit" in skills frontmatter, then absent
- AC-003.4: Given `agents/audit-orchestrator.md`, when checked for "Context Checkpoint After Each Domain Agent" section, then removed
- AC-003.5: Given `agents/audit-orchestrator.md`, when checked for "Skip Completed Domains" section, then removed

#### Dependencies

- Depends on: US-001

### US-004: Remove Resumption Pointer from campaign manifest

**As a** maintainer, **I want** the Resumption Pointer removed from campaign manifest schema, **so that** the schema is simpler without dead sections.

#### Acceptance Criteria

- AC-004.1: Given `skills/campaign-manifest/SKILL.md`, when checked for "## Resumption Pointer", then absent
- AC-004.2: Given `skills/campaign-manifest/SKILL.md`, when checked for "Context checkpoint" in Incremental Updates, then absent
- AC-004.3: Given `commands/design.md`, when checked for "Resumption Pointer", then absent or updated
- AC-004.4: Given `skills/campaign-manifest/SKILL.md` Malformed Recovery section, when checked for required headers list, then "Resumption Pointer" is absent from the list

#### Dependencies

- Depends on: US-002

### US-005: Supersede ADR-0014 and update glossary

**As a** maintainer, **I want** ADR-0014 marked as Superseded and glossary entries cleaned up, **so that** documentation reflects current state.

#### Acceptance Criteria

- AC-005.1: Given `docs/adr/0014-context-aware-graceful-exit.md`, when checked, then Status is "Superseded by BL-060"
- AC-005.2: Given `docs/domain/glossary.md`, when checked for "### Graceful Exit", then absent
- AC-005.3: Given `docs/domain/glossary.md`, when checked for "### Context Checkpoint", then absent
- AC-005.4: Given `docs/domain/glossary.md`, when checked for "### Resumption Pointer", then absent
- AC-005.5: Given `docs/adr/README.md`, when checked, then ADR 0014 row shows "Superseded"

#### Dependencies

- Depends on: US-001

### US-006: Update backlog and add absence tests

**As a** maintainer, **I want** BL-054/BL-055 archived and explicit tests verifying graceful exit references are absent, **so that** the removal is verified and tracked.

#### Acceptance Criteria

- AC-006.1: Given `docs/backlog/BL-054-implement-compact-gate.md`, when checked, then status is `archived`
- AC-006.2: Given `docs/backlog/BL-055-graceful-mid-session-exit.md`, when checked, then status is `archived`
- AC-006.3: Given a grep test, when `grep -rl 'graceful-exit' commands/ skills/ agents/` is run, then 0 results
- AC-006.4: Given a grep test, when `grep -rl 'Resumption.*Pointer' commands/ skills/ agents/` is run, then 0 results
- AC-006.5: Given a grep test, when `grep -rl 'read-context-percentage' commands/ skills/ agents/ docs/ hooks/` is run (excluding `docs/specs/` and `docs/audits/`), then 0 results
- AC-006.6: Given `test-campaign-manifest.sh`, when Resumption Pointer assertions AND `test_strategic_compact_campaign` function are removed/updated, then test passes
- AC-006.7: Given additional reference files (`skills/prompt-optimizer/SKILL.md`, `skills/configure-ecc/SKILL.md`, `hooks/README.md`, `docs/longform-guide.md`, `docs/token-optimization.md`, `docs/DEPENDENCY-GRAPH.md`), when checked for `strategic-compact` or `graceful-exit` references, then references are removed or updated

#### Dependencies

- Depends on: US-001, US-002, US-003, US-004

### US-007: Documentation and quality gate

**As a** maintainer, **I want** CHANGELOG updated and all tests passing, **so that** the refactoring is complete and clean.

#### Acceptance Criteria

- AC-007.1: Given `CHANGELOG.md`, when checked, then BL-060 entry exists
- AC-007.2: Given `cargo test`, when run, then all tests pass
- AC-007.3: Given `cargo clippy -- -D warnings`, when run, then zero warnings
- AC-007.4: Given `cargo build`, when run, then succeeds

#### Dependencies

- Depends on: all

## Affected Modules

| Module | Layer | Nature of Change |
|--------|-------|-----------------|
| `skills/graceful-exit/` | Skill (delete) | Delete entire directory |
| `skills/strategic-compact/` | Skill (delete) | Delete SKILL.md |
| `commands/implement.md` | Command | Remove context checkpoints, Phase 0 gate, Resumption Pointer |
| `commands/audit-full.md` | Command | Remove re-entry and cleanup sections |
| `commands/design.md` | Command | Remove Resumption Pointer reference |
| `agents/audit-orchestrator.md` | Agent | Remove graceful-exit skill ref, checkpoint, skip-domains |
| `skills/campaign-manifest/SKILL.md` | Skill | Remove Resumption Pointer schema section |
| `docs/adr/0014-*.md` | Docs | Supersede |
| `docs/domain/glossary.md` | Docs | Remove 3 entries |
| `CHANGELOG.md` | Docs | Add BL-060 entry |
| `skills/prompt-optimizer/SKILL.md` | Skill | Remove strategic-compact reference |
| `skills/configure-ecc/SKILL.md` | Skill | Remove strategic-compact from bundles |
| `hooks/README.md` | Docs | Remove strategic-compact link |
| `docs/longform-guide.md` | Docs | Update strategic compact mentions |
| `docs/token-optimization.md` | Docs | Remove graceful-exit references |
| `docs/DEPENDENCY-GRAPH.md` | Docs | Remove suggest-compact if referencing deleted skill |
| `tests/test-campaign-manifest.sh` | Test | Remove Resumption Pointer + strategic-compact test assertions |

## Constraints

- All refactoring steps are pure deletions/removals — no new behavior
- `cargo test` must pass (no Rust changes expected)
- `cargo clippy -- -D warnings` must pass
- No dangling references to deleted files
- Existing workflow functionality (tasks.md resume, campaign tracking) must continue working
- BL-054/BL-055 backlog entries archived, not deleted

## Non-Requirements

- Adding new context management features
- Modifying statusline context display
- Changing campaign manifest skill beyond Resumption Pointer removal
- Modifying any Rust code (the `suggest-compact` Rust hook handler is kept — independent feature)
- Deleting `statusline/context-persist.sh` (kept — writes context % for statusline display, independent of graceful exit)
- Creating a new ADR (ADR-0014 is superseded in place)

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| None | No port/adapter changes | Pure markdown/shell deletion |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| Removed concept | Domain | `docs/domain/glossary.md` | Remove Graceful Exit, Context Checkpoint, Resumption Pointer entries |
| Feature entry | Project | `CHANGELOG.md` | Add BL-060 removal entry |
| Superseded ADR | Architecture | `docs/adr/0014-*.md` | Mark as Superseded |

## Open Questions

None — all resolved during grill-me interview.
