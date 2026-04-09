# Spec: BL-016 prd-to-plan skill

## Problem Statement

The project needs a skill that decomposes PRDs into multi-phase implementation plans using tracer-bullet vertical slices. The existing `blueprint` skill (community origin) provides similar planning capability but only accepts one-line objectives, not structured PRD input. Rather than creating a duplicate planning skill, absorb `blueprint` into `prd-to-plan` with both input modes.

## Research Summary

- Tracer bullets are thin vertical slices through all architectural layers — validate architecture E2E before horizontal expansion
- PRD-to-plan decomposition with LLMs works best with 5-15 minute phases, explicit dependencies, and bounded scope
- Claude Code skills use two-part structure: YAML frontmatter (trigger matching) + markdown body (loaded on demand)
- Instruction budget: ~150-200 instructions before LLM degradation — phases should have bounded instruction counts
- Quality gates between phases are mandatory (Explore-Plan-Code-Commit pattern)
- Progressive disclosure: description triggers activation, body loaded only when relevant
- The existing `/design` pipeline's planner agent already produces similar plans — this skill is for pre-pipeline exploration

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Absorb blueprint into prd-to-plan | Avoids duplicate planning skills; user preference | Yes |
| 2 | Two input modes (one-liner + PRD file) | Preserves blueprint's one-liner capability; adds structured input | No |
| 3 | Graceful PRD validation | Hand-written PRDs may not follow BL-012 template; flag gaps, don't hard-fail | No |
| 4 | Output to docs/plans/ | Already in phase-gate allowlist; consistent with convention | No |
| 5 | Enforce vertical-slice phasing | Core BL-016 requirement; negative example for horizontal slices | No |
| 6 | Position as pre-pipeline exploration | Does NOT replace /spec → /design pipeline; produces draft plans only | No |
| 7 | Add "tracer bullet" to CLAUDE.md glossary | Used across multiple components | No |
| 8 | Selective blueprint absorption | Carry forward: cold-start briefs, dependency graph, parallel detection, vertical-slice enforcement. Drop: adversarial review gate, mutation protocol, branch/PR/CI workflow, git/gh detection (handled by pipeline) | No |

## User Stories

### US-001: Skill file with valid frontmatter and dual-mode content

**As a** Claude Code user, **I want** a `prd-to-plan` skill that activates on planning triggers, **so that** I can decompose objectives or PRDs into phased plans.

#### Acceptance Criteria

- AC-001.1: `skills/prd-to-plan/SKILL.md` exists with frontmatter: name=prd-to-plan, description (non-empty), origin=ECC
- AC-001.2: `ecc validate skills` passes with zero errors
- AC-001.3: Skill body under 500 words (excluding frontmatter)
- AC-001.4: Trigger phrases documented: "turn this prd into a plan", "implementation plan", "break this down into phases", "blueprint", "roadmap"
- AC-001.5: Negative trigger documented: does NOT activate for single-PR tasks or "just do it"
- AC-001.6: Skill documents two input modes: one-liner objective and PRD file path

#### Dependencies
- None

### US-002: PRD file reading and validation

**As a** user with a PRD file, **I want** the skill to read and validate the PRD structure, **so that** plan generation is grounded in structured input.

#### Acceptance Criteria

- AC-002.1: Skill specifies PRD file path input step (user provides path or skill asks)
- AC-002.2: Skill validates expected sections (Problem Statement, Target Users, User Stories, Non-Goals, Risks, Module Sketch, Success Metrics, Open Questions)
- AC-002.3: If PRD is missing sections, skill flags gaps and asks whether to proceed
- AC-002.4: If PRD file doesn't exist, skill reports error and asks for valid path

#### Dependencies
- Depends on: US-001

### US-003: Vertical-slice phase decomposition with blueprint features

**As a** user, **I want** the objective/PRD decomposed into vertical-slice phases with cold-start briefs, **so that** each phase is independently executable.

#### Acceptance Criteria

- AC-003.1: Each phase includes: description, affected modules, acceptance criteria (single-command verifiable where possible), dependencies, complexity (LOW/MEDIUM/HIGH), rollback strategy
- AC-003.2: Explicit negative instruction: no horizontal slices
- AC-003.3: Phases ordered by dependency (no forward references)
- AC-003.4: Skill body contains instructions for generating self-contained context briefs per phase (cold-start execution)
- AC-003.5: Skill body contains instructions for generating a dependency graph with parallel step detection
- AC-003.6: Skill body contains instructions for codebase exploration (Grep/Glob) to identify affected modules

#### Dependencies
- Depends on: US-001

### US-004: Plan output and lifecycle

**As a** user, **I want** the plan written to `docs/plans/{feature}-plan.md`, **so that** it's a durable artifact.

#### Acceptance Criteria

- AC-004.1: Output to `docs/plans/{feature}-plan.md` with kebab-case slug
- AC-004.2: `docs/plans/` created automatically if missing
- AC-004.3: Plan links back to source PRD path (if PRD mode) or records the one-liner
- AC-004.4: Plan ends with: "To execute, run `/spec` to formalize requirements, then `/design` for pass conditions."

#### Dependencies
- Depends on: US-003

### US-005: Blueprint removal and housekeeping

**As a** project maintainer, **I want** blueprint removed and BL-016 marked implemented, **so that** there's no duplicate skill.

#### Acceptance Criteria

- AC-005.1: `skills/blueprint/` directory deleted
- AC-005.2: `docs/backlog/BL-016-create-prd-to-plan-skill.md` status changed to `implemented`
- AC-005.3: CHANGELOG entry for BL-016 (includes blueprint absorption note)
- AC-005.4: ADR 0060 documents blueprint absorption decision
- AC-005.5: "tracer bullet" added to CLAUDE.md glossary

#### Dependencies
- Depends on: US-001

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `skills/prd-to-plan/SKILL.md` | Skill (new) | New unified planning skill |
| `skills/blueprint/SKILL.md` | Skill (deleted) | Absorbed into prd-to-plan |
| `docs/adr/0060-*.md` | Docs (new) | ADR for absorption |

## Constraints

- Skill body must be under 500 words
- No Rust code changes (pure Markdown skill)
- `docs/plans/` already in phase-gate allowlist
- Must not duplicate planner agent internals (no pass conditions, TDD ordering)

## Non-Requirements

- Pass condition generation (belongs to planner agent in /design)
- TDD ordering or file-change tables
- Replacing the /spec → /design pipeline
- Behavioral testing (validation-only testing)
- Blueprint features NOT carried forward: adversarial review gate (handled by /design pipeline), plan mutation protocol (over-engineered for a skill), branch/PR/CI workflow generation (handled by /implement), anti-pattern catalog (folded into negative examples), git/gh detection (not needed for Markdown output)

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| None | N/A | Pure Markdown skill — no runtime boundaries |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| New ADR | Decision | docs/adr/0060-blueprint-absorption.md | Create |
| Glossary | Onboarding | CLAUDE.md | Add "tracer bullet" definition |
| Changelog | Project | CHANGELOG.md | BL-016 entry + blueprint absorption note |
| Backlog | Project | docs/backlog/BL-016-*.md | Mark implemented |

## Open Questions

None — all resolved during grill-me interview.
