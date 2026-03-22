# Spec: Comprehensive pipeline output summaries + DRY cleanup (BL-048)

## Problem Statement

The spec-driven pipeline commands (`/spec-*`, `/design`, `/implement`) produce sparse bullet-point summaries at completion that lack transparency — users cannot see grill-me decisions, per-dimension adversary verdicts, artifact file paths, or commit inventories without scrolling through conversation history. Additionally, the 3 spec commands diverge unnecessarily in their Present output and duplicate ~150 lines of shared logic (project detection, grill-me rules, adversarial+persist blocks, spec output schema). This refactoring enriches all commands with comprehensive table-based summaries, persists them to artifact files, and extracts shared content into a reusable skill.

## Research Summary

- **Markdown table best practices**: Keep cell content concise, avoid line breaks within cells, use pipe-delimited format consistently
- **DRY extraction for shared command logic**: The existing ECC skill pattern (passive knowledge docs) is the right vehicle for shared command sections
- **Idempotent artifact persistence**: Write-or-overwrite (not append) prevents duplicate sections on re-runs
- **Co-change coupling risk**: These 5 files have zero co-change history — BL-048 is the first cross-cutting change

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Carve out coverage delta, bounded contexts, per-test-name | Require logic changes, violate AC-5 | No |
| 2 | Unified core tables + variant-specific extras for spec commands | Consistency with flexibility per variant | No |
| 3 | Include DRY cleanup in scope | Prevents another pass through all 5 files | No |
| 4 | Extract shared sections to skills/spec-pipeline-shared/SKILL.md | Proper DRY for ~150 lines duplicated across 3 spec commands | No |
| 5 | Instruction-level accumulator additions compatible with AC-5 | Collecting data != changing phase logic | No |
| 6 | Phase Summary persisted to artifact files | Both conversation + file persistence, idempotent | No |
| 7 | ADR for Phase Summary convention | Documents the new ## Phase Summary artifact section | Yes |

## User Stories

### US-001: Spec Command Output Summaries

**As a** developer completing a `/spec-*` command, **I want** comprehensive table-based summaries displayed at completion, **so that** I can verify all decisions, ACs, and adversary findings at a glance.

#### Acceptance Criteria

- AC-001.1: Given `/spec-*` completes, when the Present phase runs, then the output includes a Grill-Me Decisions table with one row per question (question, answer, source: recommended/user).
- AC-001.2: Given `/spec-*` completes, when the Present phase runs, then the output includes a User Stories table with columns: ID, Title, AC Count, Dependencies.
- AC-001.3: Given `/spec-*` completes, when the Present phase runs, then the output includes an Acceptance Criteria table with columns: AC ID, Description, Source US.
- AC-001.4: Given `/spec-*` completes, when the Present phase runs, then the output includes an Adversary Findings table with columns: Dimension, Verdict, Key Rationale.
- AC-001.5: Given `/spec-*` completes, when the Present phase runs, then the output includes an Artifacts Persisted table with columns: File Path, Section Written.
- AC-001.6: Given the spec is persisted to docs/specs/{slug}/spec.md, when the Present phase runs, then a ## Phase Summary section containing all 5 tables is appended to the spec file. If ## Phase Summary already exists, it is overwritten (idempotent).
- AC-001.7: Given all 3 spec commands (spec-dev, spec-fix, spec-refactor), when their Present phases run, then they share the same 5 core summary tables. Each variant may add variant-specific rows (spec-fix: root cause; spec-refactor: smells addressed/deferred).
- AC-001.8: Given the grill-me interview phase, when each question is answered, then the question and answer are accumulated into a structured list for the summary table.

#### Dependencies

- Depends on: US-004

### US-002: Design Command Output Summaries

**As a** developer completing `/design`, **I want** comprehensive table-based summaries displayed at completion, **so that** I can verify all design decisions, reviews, and artifacts at a glance.

#### Acceptance Criteria

- AC-002.1: Given `/design` completes, when the Present phase runs, then the output includes a Design Reviews table with columns: Review Type (SOLID/Robert/Security), Verdict, Finding Count.
- AC-002.2: Given `/design` completes, when the Present phase runs, then the output includes an Adversary Findings table with columns: Dimension, Verdict, Key Rationale.
- AC-002.3: Given `/design` completes, when the Present phase runs, then the output includes a File Changes table with columns: File, Action, Spec Ref.
- AC-002.4: Given `/design` completes, when the Present phase runs, then the output includes an Artifacts Persisted table with columns: File Path, Section Written.
- AC-002.5: Given the design is persisted to docs/specs/{slug}/design.md, when the Present phase runs, then a ## Phase Summary section containing all 4 tables is appended to the design file. If ## Phase Summary already exists, it is overwritten (idempotent).

#### Dependencies

- Depends on: none

### US-003: Implement Command Output Summaries

**As a** developer completing `/implement`, **I want** comprehensive table-based summaries displayed at completion, **so that** I can verify tasks, commits, and docs at a glance.

#### Acceptance Criteria

- AC-003.1: Given `/implement` completes, when the Final Verification phase runs, then the output includes a Tasks Executed table with columns: PC ID, Description, RED-GREEN Status, Commit Count.
- AC-003.2: Given `/implement` completes, when the Final Verification phase runs, then the output includes a Commits Made table with columns: Hash (short), Message.
- AC-003.3: Given `/implement` completes, when the Final Verification phase runs, then the output includes a Docs Updated table with columns: Doc File, Level, What Changed.
- AC-003.4: Given `/implement` completes, when the Final Verification phase runs, then the output includes an Artifacts Persisted table with columns: File Path, Section Written.
- AC-003.5: Given the TDD loop phase, when each subagent completes, then the parent accumulates commit SHAs and messages into a structured list for the summary table.
- AC-003.6: Given implement-done.md is written, when the Final Verification phase runs, then a ## Phase Summary section containing all 4 tables is appended to the tasks.md file. If ## Phase Summary already exists, it is overwritten (idempotent).

#### Dependencies

- Depends on: none

### US-004: DRY Extraction into Shared Skill

**As a** maintainer, **I want** duplicated sections extracted from the 3 spec commands into a shared skill, **so that** changes to shared logic need only be made once.

#### Acceptance Criteria

- AC-004.1: Given skills/spec-pipeline-shared/SKILL.md exists, when inspected, then it contains: Project Detection section, Grill-Me Interview Rules section, Adversarial Review + Verdict Handling section, Spec Output Schema section.
- AC-004.2: Given the skill frontmatter, when inspected, then it has name: spec-pipeline-shared, description, origin: ECC.
- AC-004.3: Given each spec command (spec-dev, spec-fix, spec-refactor), when read, then it references spec-pipeline-shared for shared sections instead of inlining them.
- AC-004.4: Given the DRY extraction, when each spec command runs, then behavior is identical to before — no phase logic changes.
- AC-004.5: Given ecc validate skills runs, when the shared skill is validated, then it passes with no errors.

#### Dependencies

- Depends on: none

### US-005: Documentation

**As a** developer, **I want** the changes documented, **so that** the Phase Summary convention and DRY extraction are discoverable.

#### Acceptance Criteria

- AC-005.1: Given docs/adr/0009-phase-summary-convention.md exists, when read, then it documents the ## Phase Summary section convention with Status/Context/Decision/Consequences.
- AC-005.2: Given CHANGELOG.md, when read, then it includes a BL-048 feature entry.
- AC-005.3: Given the backlog, when checked, then coverage delta, bounded contexts, and per-test-name are noted as deferred items.

#### Dependencies

- Depends on: US-001, US-002, US-003, US-004

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| commands/spec-dev.md | Driving adapter (command) | Modify — add summary tables, reference shared skill |
| commands/spec-fix.md | Driving adapter (command) | Modify — add summary tables, reference shared skill |
| commands/spec-refactor.md | Driving adapter (command) | Modify — add summary tables, reference shared skill |
| commands/design.md | Driving adapter (command) | Modify — add summary tables |
| commands/implement.md | Driving adapter (command) | Modify — add summary tables + commit accumulator |
| skills/spec-pipeline-shared/SKILL.md | Content (skill) | New file — extracted shared sections |
| docs/adr/0009-phase-summary-convention.md | Documentation | New ADR |
| CHANGELOG.md | Documentation | Modify — add BL-048 entry |

No Rust crate changes. No hook changes. No state schema changes.

## Constraints

- All changes behavior-preserving — instruction additions only, no new phases
- Accumulator instructions collect data during existing phases, don't alter phase logic
- Tables use pipe-delimited markdown format (existing ECC convention)
- Phase Summary is idempotent — overwrite if ## Phase Summary already exists
- Shared skill must pass ecc validate skills
- Each command file stays under 800 lines after changes

## Non-Requirements

- Coverage delta table (requires cargo tarpaulin — separate backlog item)
- Bounded contexts table in /design (requires new analysis step — separate backlog item)
- Per-test-name inventory in /implement (requires subagent return schema change — separate backlog item)
- Full DRY refactor of /design and /implement commands (only spec commands in scope)
- Diagrams produced table in /design (optional data, not reliably generated)

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| None | — | No E2E boundaries crossed — pure command file and skill changes |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| Convention | architecture | docs/adr/ | ADR 0009 for Phase Summary convention |
| Feature | project | CHANGELOG.md | Add BL-048 entry |

## Open Questions

None — all resolved during grill-me interview.
