# Spec: BL-064 Element Wiring — Connect Sub-Spec B to Sub-Spec A

## Problem Statement

Sub-Spec B's element domain types, agent, and ADRs are on main but not wired into Sub-Spec A's handler and validate infrastructure. The `start_cartography` handler processes journey and flow deltas but does not scaffold `elements/`, dispatch the element generator, or regenerate `INDEX.md`. The `validate_cartography` function scans journeys and flows but not elements.

## Research Summary

Web research skipped — this is an integration task using existing patterns from the codebase.

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Follow existing handler patterns exactly | Sub-Spec A handlers are tested and working; element wiring mirrors journey/flow patterns | No |
| 2 | INDEX.md written by handler, not agent | `build_cross_reference_matrix()` is a pure domain function; handler calls it and writes result via `ports.fs` | No |

## User Stories

### US-001: Element Scaffold in start_cartography

**As a** cartography system, **I want** `docs/cartography/elements/` scaffolded alongside journeys and flows, **so that** the element generator has a target directory.

#### Acceptance Criteria
- AC-001.1: Given start_cartography runs and elements/ missing, then directory + README created
- AC-001.2: Given elements/ exists, then left untouched (idempotent)

#### Dependencies
- Depends on: none

### US-002: Post-Loop Element Generator Dispatch

**As a** cartography system, **I want** the element generator dispatched AFTER journey/flow processing, **so that** elements see complete journey/flow state.

#### Acceptance Criteria
- AC-002.1: Given deltas processed, when cartographer runs, then element generator dispatched AFTER journey+flow generators
- AC-002.2: Given element generator succeeds, then git add docs/cartography/ stages element files
- AC-002.3: Given element generator fails, then git reset, no archive (same failure path)
- AC-002.4: Given no element targets in delta, then element generator not dispatched

#### Dependencies
- Depends on: US-001

### US-003: INDEX.md Regeneration

**As a** developer, **I want** INDEX.md regenerated after element generation, **so that** the cross-reference matrix is current.

#### Acceptance Criteria
- AC-003.1: Given element files exist, when INDEX regenerated, then full replacement (not delta-merged)
- AC-003.2: Given INDEX regeneration, then it runs AFTER element generators complete
- AC-003.3: Given INDEX written, then it's at docs/cartography/elements/INDEX.md

#### Dependencies
- Depends on: US-002

### US-004: Validate Cartography Element Extension

**As a** developer, **I want** `ecc validate cartography` to scan elements/, **so that** element files are validated.

#### Acceptance Criteria
- AC-004.1: Given invalid element file, then CLI prints error and exits 1
- AC-004.2: Given elements/ missing, then exits cleanly (no error)
- AC-004.3: Given INDEX.md absent, then warning (not error)
- AC-004.4: Given stale INDEX.md (missing slugs), then warning with list
- AC-004.5: Given element with CARTOGRAPHY-META, when source modified, then staleness reported
- AC-004.6: Given --coverage, then element-referenced sources included
- AC-004.7: Given coverage below 50%, then gaps include all source types

#### Dependencies
- Depends on: none

### US-005: Hooks and Config Verification

**As a** developer, **I want** hooks.json and spec-dev.md properly configured, **so that** cartography triggers work end-to-end.

#### Acceptance Criteria
- AC-005.1: Given hooks.json, then stop:cartography in Stop (async, timeout 10) and start:cartography in SessionStart
- AC-005.2: Given commands/spec-dev.md, then it contains cartography/journeys actor reading logic

#### Dependencies
- Depends on: none

### US-006: BL-064 Status Update

**As a** project maintainer, **I want** BL-064 marked implemented, **so that** backlog reflects reality.

#### Acceptance Criteria
- AC-006.1: Given BACKLOG.md, then BL-064 status is implemented
- AC-006.2: Given BL-064 entry file, then status field is implemented

#### Dependencies
- Depends on: US-001, US-002, US-003, US-004, US-005

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| crates/ecc-app/src/hook/handlers/tier3_session/cartography.rs | App | Modify: add scaffold_elements_dir, post-loop dispatch, INDEX regen |
| crates/ecc-app/src/validate_cartography.rs | App | Modify: add element scan, staleness, coverage, INDEX warnings |
| agents/cartographer.md | Agent | Modify: add element dispatch step + INDEX regen step |
| hooks/hooks.json | Config | Verify/add cartography entries |
| commands/spec-dev.md | Command | Verify actor registry integration |
| docs/backlog/BACKLOG.md | Docs | Edit: BL-064 status |
| docs/backlog/BL-064-*.md | Docs | Edit: status field |

## Constraints

- 100% test coverage on all changes
- Follow existing handler patterns from Sub-Spec A
- No new domain types (all in ecc-domain already)
- No new ADRs (0042-0044 already created)

## Non-Requirements

- New domain types or domain code changes
- New agent files (cartography-element-generator.md already exists)
- ADR creation
- Changes to ecc-ports, ecc-infra, ecc-cli, or ecc-domain

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| FileSystem | Used by element scaffold + INDEX write | E2E: files created at correct paths |
| ShellExecutor | Used for element generator dispatch | E2E: correct command sequence |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| Backlog status | BACKLOG.md | BL-064 → implemented | Edit |
| Test count | CLAUDE.md | Update test count | Edit |
| Feature entry | CHANGELOG.md | Not needed (already has Sub-Spec B entry) | None |

## Open Questions

None.
