# Spec: BL-015 Request-Refactor-Plan Skill

## Problem Statement

No structured way to decompose a refactoring into a sequence of tiny, green-state commits in ECC. Developers either refactor in one large commit (risky, hard to review) or plan ad-hoc. A skill that interviews the user, explores the codebase, and writes a structured refactoring plan to `docs/refactors/{name}-plan.md` fills this gap.

## Research Summary

- Same delivery pattern as BL-012 (write-a-prd): skill file + phase-gate allowlist
- Tiny-commit decomposition is a known refactoring best practice (Martin Fowler, "Refactoring" 2nd ed.)
- Each commit must leave the codebase in a green state (compiling + tests passing)
- ECC already has `docs/prds/` pattern from BL-012 — `docs/refactors/` follows same convention

## Decisions Made

| # | Decision | Rationale | ADR? |
|---|----------|-----------|------|
| 1 | Same pattern as BL-012 | Self-contained, under 500 words, AskUserQuestion, file output | No |
| 2 | `docs/refactors/` committed to git | Durable artifacts for code review and post-mortems | No |
| 3 | Add `docs/refactors/` to phase-gate | Allow writing during spec/design phases | No |
| 4 | No automated execution | Plan only — developer executes commits manually | No |

## User Stories

### US-001: Skill File with Valid Frontmatter

**As a** Claude Code user planning a refactoring, **I want** a skill that interviews me about the refactoring and writes a structured plan, **so that** I can decompose large refactorings into safe, tiny commits.

#### Acceptance Criteria

- AC-001.1: `skills/request-refactor-plan/SKILL.md` exists with `name: request-refactor-plan`, `description`, `origin: ECC`
- AC-001.2: Skill body under 500 words; `ecc validate skills` passes
- AC-001.3: Trigger phrases documented: "refactor plan", "plan a refactoring", "how should I restructure", "tiny commits for this change"

#### Dependencies

- Depends on: none

### US-002: Interactive Refactoring Flow

**As a** developer planning a refactoring, **I want** the skill to interview me and explore the codebase, **so that** the plan is grounded in real structure.

#### Acceptance Criteria

- AC-002.1: Skill defines 6-step flow: (1) interview about what/why, (2) explore codebase, (3) identify affected files, (4) decompose into tiny commits, (5) order commits by dependency, (6) write plan
- AC-002.2: Each step uses AskUserQuestion (one at a time); falls back to conversational if unavailable
- AC-002.3: Codebase exploration uses Read/Grep/Glob to verify assertions

#### Dependencies

- Depends on: US-001

### US-003: Plan Output Template

**As a** developer, **I want** the plan written to a file using a structured per-commit template, **so that** each commit is reviewable and safe.

#### Acceptance Criteria

- AC-003.1: Output to `docs/refactors/{name}-plan.md` (kebab-case slug, max 40 chars)
- AC-003.2: Each commit specifies: Change Description, Affected Files, Risk Level (LOW/MEDIUM/HIGH — LOW: no behavior change, MEDIUM: behavior change with test coverage, HIGH: behavior change in untested area), Rollback Instruction
- AC-003.3: Each commit must leave codebase green (compiling + tests passing) — noted explicitly in the plan
- AC-003.4: `docs/refactors/` directory created automatically if missing
- AC-003.5: If plan already exists at path, ask to overwrite or append revision

#### Dependencies

- Depends on: US-002

### US-004: Phase-Gate Allowlist Update

**As a** developer using the spec pipeline, **I want** `docs/refactors/` in the phase-gate allowlist, **so that** writing refactoring plans during spec/design phases is not blocked.

#### Acceptance Criteria

- AC-004.1: `docs/refactors/` added to `allowed_prefixes()` in `phase_gate.rs`
- AC-004.2: All existing phase-gate tests pass
- AC-004.3: New test: Write to `docs/refactors/test.md` during plan phase is allowed

#### Dependencies

- Depends on: none

### US-005: Backlog Status Update

**As a** project maintainer, **I want** BL-015 marked as implemented.

#### Acceptance Criteria

- AC-005.1: `docs/backlog/BL-015-*.md` has `status: implemented`

#### Dependencies

- Depends on: US-001

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `skills/request-refactor-plan/SKILL.md` | Skill | New file |
| `crates/ecc-workflow/src/commands/phase_gate.rs` | Adapter | Add `"docs/refactors/"` to allowlist |

## Constraints

- Skill must be under 500 words
- No new agents, tools, or crate dependencies
- No automated refactoring execution

## Non-Requirements

- No GitHub Issues integration
- No automated commit execution from the plan
- No Rust source code changes beyond phase-gate

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| None | Markdown files only | No E2E impact |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| CHANGELOG | root | CHANGELOG.md | Add entry |

## Open Questions

None.
