# Spec: Display Full Artifacts Inline in Terminal (BL-062)

## Problem Statement

The `/spec-*`, `/design`, and `/implement` pipeline commands persist artifacts to `docs/specs/` but only display summary tables in their final phases. Users must open files to read the full spec or design — defeating the purpose of a conversational pipeline. Additionally, the final phase naming is inconsistent across commands (Phase 10, Phase 9, Phase 11, Phase 8), making the pattern harder to follow.

## Research Summary

- The refactoring is a presentation-layer change to Markdown instruction files — no Rust code changes
- All 5 commands share the same pattern: summary tables only, no inline artifact display
- File size additions are ~10-15 lines per file, well within the 800-line limit
- BL-062 explicitly requires no truncation — full content is always shown
- The only existing "display content" pattern is design.md's Plan Mode preview (partial, not final phase)
- Prior audit [SELF-001] flagged implement.md approaching 800-line limit — additions are minimal

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Display full artifact before summary tables | BL-062 requirement — users should not need to open files | No |
| 2 | Read artifact from disk via state.json path | Reliable source — artifact is already persisted before final phase | No |
| 3 | No truncation for large artifacts | BL-062 explicitly requires full content for large specs (37+ ACs) | No |
| 4 | Normalize final phase structure across all 5 commands | Address inconsistent naming — all follow same display→tables→path→STOP pattern | No |

## User Stories

### US-001: Display full spec inline in /spec-* commands

**As a** spec pipeline user, **I want** the full spec.md content displayed inline in the terminal at the end of `/spec-dev`, `/spec-fix`, and `/spec-refactor`, **so that** I can review the complete spec without opening files.

#### Acceptance Criteria

- AC-001.1: Given `/spec-dev` completes, when the final phase executes, then the full spec.md content is displayed as a fenced Markdown block before the summary tables.
- AC-001.2: Given `/spec-fix` completes, when the final phase executes, then the full spec.md content is displayed before summary tables.
- AC-001.3: Given `/spec-refactor` completes, when the final phase executes, then the full spec.md content is displayed before summary tables.
- AC-001.4: Given the artifact is displayed inline, when summary tables follow, then existing table format is unchanged (BL-048 compatibility).
- AC-001.5: Given the artifact is displayed inline, when the file path reference is shown, then it appears after the summary tables for future access.
- AC-001.6: Given the spec artifact is read, when `artifacts.spec_path` is set in state.json, then the file is read from that path.
- AC-001.7: Given `artifacts.spec_path` is null or the referenced file does not exist on disk, when the final phase executes, then a warning is displayed ("Artifact file not found; skipping inline display") and the phase continues with summary tables only.

#### Dependencies

- Depends on: none

### US-002: Display full design inline in /design

**As a** design pipeline user, **I want** the full design.md content displayed inline in the terminal at the end of `/design`, **so that** I can review the complete design without opening files.

#### Acceptance Criteria

- AC-002.1: Given `/design` completes, when the final phase executes, then the full design.md content is displayed as a fenced Markdown block before the summary tables.
- AC-002.2: Given the design artifact read, when `artifacts.design_path` is set in state.json, then the file is read from that path.
- AC-002.3: Given `artifacts.design_path` is null or the file does not exist, when the final phase executes, then a warning is displayed and the phase continues with summary tables only.

#### Dependencies

- Depends on: none

### US-003: Display full tasks.md inline in /implement

**As an** implementation pipeline user, **I want** the full tasks.md content displayed inline in the terminal at the end of `/implement`, **so that** I can review the complete task status without opening files.

#### Acceptance Criteria

- AC-003.1: Given `/implement` completes, when the final phase executes, then the full tasks.md content is displayed as a fenced Markdown block before the summary tables.
- AC-003.2: Given the tasks artifact read, when `artifacts.tasks_path` is set in state.json, then the file is read from that path.
- AC-003.3: Given `artifacts.tasks_path` is null or the file does not exist, when the final phase executes, then a warning is displayed and the phase continues with summary tables only.

#### Dependencies

- Depends on: none

### US-004: Normalize final phase structure

**As a** command maintainer, **I want** consistent final phase structure across all 5 commands, **so that** the pattern is easier to follow and maintain.

#### Acceptance Criteria

- AC-004.1: Given any of the 5 command files, when the final phase is read, then it follows the pattern: display artifact → summary tables → file path → STOP.
- AC-004.2: Given all 5 commands, when their final phase instructions are compared, then all use this template: "Read the full artifact from `artifacts.<type>_path` in state.json using the Read tool. Display the complete file content inline in conversation. If the path is null or the file does not exist, emit a warning and skip to the summary tables."

#### Dependencies

- Depends on: none

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `commands/spec-dev.md` | Content (outside Rust hexagon) | Add inline artifact display to Phase 10 |
| `commands/spec-fix.md` | Content | Add inline artifact display to Phase 9 |
| `commands/spec-refactor.md` | Content | Add inline artifact display to Phase 9 |
| `commands/design.md` | Content | Add inline artifact display to Phase 11 |
| `commands/implement.md` | Content | Add inline artifact display to Phase 8 |

## Constraints

- All refactoring steps must be behavior-preserving for existing summary tables
- No truncation — full artifact content always displayed
- Documents still persist to docs/specs/ (display is additive, not a replacement)
- Do not alter phase ordering or introduce new phases
- Do not change summary table format (BL-048 owns that)
- File size must remain under 800 lines per file
- Each command file change can be shipped independently

## Non-Requirements

- Collapsible/expandable sections (plain Markdown only)
- Artifact persistence logic changes
- Summary table format changes (BL-048)
- Phase reordering or renumbering
- Deferred smells: inconsistent phase numbering across commands (cosmetic)

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| None | N/A | Content layer only — no port/adapter surface changed |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| Content | Minor | CHANGELOG.md | Add entry for inline artifact display |

## Open Questions

None — all resolved during grill-me interview.

## Phase Summary

### Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|
| 1 | Smell triage — which to address? | All 3 smells: hidden output, inconsistent naming, file size awareness | User |
| 2 | Target architecture? | Consistent final phase: display artifact → tables → path → STOP | Recommended |
| 3 | Step independence? | Ship independently per file, spec-dev first as reference | Recommended |
| 4 | Downstream dependencies? | Validate via ecc validate + grep + cargo test | Recommended |
| 5 | Rename vs behavioral? | All behavioral mods + heading renames | Recommended |
| 6 | Performance? | No impact. Large output accepted per BL-062 | Recommended |
| 7 | ADR? | No ADR needed | Recommended |
| 8 | Test safety? | ecc validate + grep + cargo test + manual verification | Recommended |
| — | **Smells addressed** | #1 hidden output, #2 inconsistent naming | — |
| — | **Smells deferred** | #3 file size (monitor only) | — |

### User Stories

| ID | Title | AC Count | Dependencies |
|----|-------|----------|--------------|
| US-001 | Display full spec inline in /spec-* | 7 | none |
| US-002 | Display full design inline in /design | 3 | none |
| US-003 | Display full tasks.md inline in /implement | 3 | none |
| US-004 | Normalize final phase structure | 2 | none |

### Acceptance Criteria

| AC ID | Description | Source US |
|-------|-------------|----------|
| AC-001.1 | spec-dev full spec inline before tables | US-001 |
| AC-001.2 | spec-fix full spec inline before tables | US-001 |
| AC-001.3 | spec-refactor full spec inline before tables | US-001 |
| AC-001.4 | Summary table format unchanged (BL-048) | US-001 |
| AC-001.5 | File path after tables for reference | US-001 |
| AC-001.6 | Read from artifacts.spec_path in state.json | US-001 |
| AC-001.7 | Fallback when path null/missing: warn + skip | US-001 |
| AC-002.1 | design full design inline before tables | US-002 |
| AC-002.2 | Read from artifacts.design_path | US-002 |
| AC-002.3 | Fallback when path null/missing | US-002 |
| AC-003.1 | implement full tasks.md inline before tables | US-003 |
| AC-003.2 | Read from artifacts.tasks_path | US-003 |
| AC-003.3 | Fallback when path null/missing | US-003 |
| AC-004.1 | Consistent display→tables→path→STOP pattern | US-004 |
| AC-004.2 | Template instruction text across all 5 files | US-004 |

### Adversary Findings

| Dimension | Verdict | Key Rationale |
|-----------|---------|---------------|
| Ambiguity | PASS (round 2) | AC-001.6 added for spec_path read; AC-004.2 template specified |
| Edge cases | PASS (round 2) | Fallback ACs (001.7, 002.3, 003.3) added for null/missing paths |
| Scope | PASS | Appropriate for MEDIUM content refactoring |
| Dependencies | PASS | state.json fields exist; BL-048 compatibility stated |
| Testability | PASS (round 2) | Template text in AC-004.2 makes verification objective |
| Decisions | PASS | All 4 decisions justified, no ADRs needed |
| Rollback | PASS | Independent file commits, simple git revert |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-03-26-inline-artifact-display/spec.md | Full spec + phase summary |
