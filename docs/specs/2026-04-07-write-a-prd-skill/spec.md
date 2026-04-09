# Spec: BL-012 Write-a-PRD Skill

## Problem Statement

ECC lacks a structured PRD generation workflow. Users jump straight to `/spec` without a product-level exploration phase. A `write-a-prd` skill provides an interactive interview + codebase exploration flow that produces a PRD file at `docs/prds/{feature}-prd.md` — the input artifact for BL-016 (`prd-to-plan`).

## Research Summary

- No web research needed — internal ECC skill convention
- Existing interview skills: `interview-me` (8-stage requirements), `grill-me` (adversarial questioning)
- BL-016 (`prd-to-plan`) is downstream consumer — PRD template is the contract
- `docs/prds/` not in phase-gate allowlist — requires Rust change
- Architect confirmed: no hexagonal boundary crossings, no port/adapter changes needed beyond phase-gate

## Decisions Made

| # | Decision | Rationale | ADR? |
|---|----------|-----------|------|
| 1 | Add `docs/prds/` to phase-gate allowlist | PRDs are pre-spec artifacts used during early pipeline phases | No |
| 2 | Strictly under 500 words | ECC v1 skill convention; terse protocol outline, tables not prose | No |
| 3 | Self-contained, no grill-me reference | No coupling to other skills; avoids dependency for standalone use | No |
| 4 | Defer deep module analysis to /design | Too early at PRD stage; module sketch is sufficient | No |
| 5 | No adversarial review | PRDs are exploration artifacts, not pipeline gates | No |
| 6 | `docs/prds/` committed to git | Durable artifacts, input to BL-016. Not gitignored. | No |

## User Stories

### US-001: Skill File with Valid Frontmatter

**As a** Claude Code user, **I want** the `write-a-prd` skill to be discoverable and valid, **so that** I can invoke it naturally.

#### Acceptance Criteria

- AC-001.1: `skills/write-a-prd/SKILL.md` exists with `name: write-a-prd`, `description`, `origin: ECC`
- AC-001.2: `ecc validate skills` passes with zero errors
- AC-001.3: Skill body under 500 words (excluding frontmatter)
- AC-001.4: Trigger phrases documented: "write a prd", "product requirements", "feature spec", "define what we're building"

#### Dependencies

- Depends on: none

### US-002: Interactive PRD Generation Flow

**As a** user with a feature idea, **I want** the skill to interview me and explore the codebase, **so that** the PRD is grounded in real needs and verified assertions.

#### Acceptance Criteria

- AC-002.1: Skill defines 6-step flow: (1) problem interview, (2) codebase exploration, (3) alternatives + tradeoffs, (4) scope hammering, (5) module sketch, (6) write PRD
- AC-002.2: Each step uses AskUserQuestion for user input (one question at a time)
- AC-002.3: Codebase exploration step uses Read/Grep/Glob to verify user assertions
- AC-002.4: If AskUserQuestion unavailable, fall back to conversational questions

#### Dependencies

- Depends on: US-001

### US-003: PRD Output to Standard Template

**As a** user, **I want** the PRD written to a file using a consistent template, **so that** downstream tools (BL-016) can parse it reliably.

#### Acceptance Criteria

- AC-003.1: PRD written to `docs/prds/{feature}-prd.md` using kebab-case slug (max 40 chars)
- AC-003.2: Template has 8 sections: Problem Statement, Target Users, User Stories + ACs, Non-Goals, Risks & Mitigations, Module Sketch, Success Metrics, Open Questions
- AC-003.3: `docs/prds/` directory created automatically if missing
- AC-003.4: If PRD already exists at path, skill asks to overwrite or append revision
- AC-003.5: Empty sections written as "None identified" rather than omitted

#### Dependencies

- Depends on: US-002

### US-004: Phase-Gate Allowlist Update

**As a** developer using the spec pipeline, **I want** `docs/prds/` in the phase-gate allowlist, **so that** PRD writing is not blocked during spec/design phases.

#### Acceptance Criteria

- AC-004.1: `docs/prds/` added to `allowed_prefixes()` in `crates/ecc-workflow/src/commands/phase_gate.rs`
- AC-004.2: All existing phase-gate tests still pass
- AC-004.3: New test: Write to `docs/prds/test.md` during plan phase is allowed

#### Dependencies

- Depends on: none

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `skills/write-a-prd/SKILL.md` | Skill | New file — interactive PRD generation |
| `crates/ecc-workflow/src/commands/phase_gate.rs` | Adapter | Add `"docs/prds/"` to `allowed_prefixes()` |

## Constraints

- Skill must be under 500 words (v1 convention)
- No new agents, tools, or crate dependencies
- No tracker/workflow state integration
- PRD template sections must be stable for BL-016 consumption
- Self-contained — no skill-to-skill references

## Non-Requirements

- No adversarial review gate for PRDs
- No Ousterhout deep module analysis (deferred to /design)
- No grill-me dependency or reference
- No BL-016 (`prd-to-plan`) implementation
- No BL-015 (`request-refactor-plan`) implementation

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| `phase_gate.rs` | Add prefix to static allowlist | Existing gate tests + 1 new test |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| CHANGELOG | root | CHANGELOG.md | Add entry |
| Backlog | docs | BL-012 | Mark implemented |

## Open Questions

None — all resolved during grill-me interview.
