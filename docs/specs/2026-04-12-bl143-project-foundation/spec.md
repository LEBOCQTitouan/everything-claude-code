# Spec: BL-143 /project-foundation Command

## Problem Statement

ECC has feature-level PRD creation (write-a-prd skill) and feature spec generation (/spec-dev), but no structured way to create project-wide foundational documents. Developers starting new projects or onboarding to existing ones lack a guided process for producing a coherent PRD, architecture overview, and initial ADR. The BMAD-METHOD demonstrates that PM + Architect agent personas with challenge loops produce higher-quality foundational docs than ad-hoc creation.

## Research Summary

- BMAD-METHOD provides 12+ agent personas with sequential structured doc creation — validates the multi-persona challenge approach
- ArchLens LangGraph pipeline does 3-pass automated analysis on Git repos producing C4 architecture models — validates codebase-analysis-first approach
- Human oversight is non-negotiable: AI drafts, human validates with domain expertise
- Multi-layer analysis (static + dynamic + code comments) enables richer context extraction
- Iterative workflows deliver better results than premature full automation
- Well-structured processes reduce doc creation overhead by ~50%

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Include CLAUDE.md generation for new repos | Full bootstrapping | No |
| 2 | Output to docs/foundation/ with optional merge into ARCHITECTURE.md | Avoids macOS case collision | No |
| 3 | Auto-number ADRs for existing repos | Detect next available number | No |
| 4 | Add foundation-mode to grill-me skill | Upgrade existing skill, compose with full interview-me | Yes |
| 5 | Reuse spec-adversary with modified prompt | No new agent needed | No |
| 6 | Full ecc-workflow state machine | Re-entry support + phase gates | No |
| 7 | Add glossary entries to CLAUDE.md | Foundation document + codebase-analysis phase | No |

## User Stories

### US-001: Command File
**As a** developer, **I want** a `/project-foundation` command at `commands/project-foundation.md`, **so that** it integrates with ECC's command system.

#### Acceptance Criteria
- AC-001.1: Given the command file, when `ecc validate commands` runs, then validation passes
- AC-001.2: Given the command, when inspected, then frontmatter has description, allowed-tools, references narrative-conventions
- AC-001.3: Given the command, when inspected, then it uses TodoWrite for phase tracking
- AC-001.4: Given the command, when inspected, then it integrates with ecc-workflow state machine

#### Dependencies
- Depends on: US-002

### US-002: grill-me Foundation-Mode
**As a** developer, **I want** foundation-mode in grill-me, **so that** challenges are tuned for project-level docs.

#### Acceptance Criteria
- AC-002.1: Given foundation-mode PRD creation, then Clarity + Assumptions stages fire (max 2 questions per stage)
- AC-002.2: Given foundation-mode architecture creation, then Clarity + Edge Cases stages fire
- AC-002.3: Given the skill file, then foundation-mode is documented alongside other modes
- AC-002.4: Given existing grill-me consumers (spec-dev, spec-fix, spec-refactor, backlog), when they invoke grill-me in their respective modes (spec-mode, backlog-mode), then behavior is identical to pre-change (foundation-mode only activates when explicitly requested)

#### Dependencies
- Depends on: none

### US-003: Codebase Detection
**As a** developer, **I want** automatic new vs existing repo detection, **so that** the workflow adapts.

#### Acceptance Criteria
- AC-003.1: Given an empty repo, then codebase analysis is skipped (pure interview)
- AC-003.2: Given an existing repo, then analysis reads CLAUDE.md, directory structure, tech stack manifests
- AC-003.3: Given analysis results, then agent presents structured understanding for user to confirm/override
- AC-003.4: Given a repo with only README, then classified as "new"
- AC-003.5: Given codebase analysis produces findings that contradict user's interview answers, then agent presents both perspectives and user's answer takes precedence

#### Dependencies
- Depends on: none

### US-004: PRD Generation
**As a** developer, **I want** a guided interview producing a project-level PRD at docs/foundation/prd.md.

#### Acceptance Criteria
- AC-004.1: Given the interview, then interview-me fires all 8 stages (user can skip)
- AC-004.2: Given PRD creation, then grill-me foundation-mode fires Clarity + Assumptions
- AC-004.3: Given completed interview, then PRD contains all 7 sections (Problem, Users, Goals, Non-Goals, Success Metrics, Key Features, Risks) each with at least one non-empty paragraph
- AC-004.4: Given an existing repo, then PRD sections pre-populated from analysis
- AC-004.5: Given the PRD, then written to docs/foundation/prd.md

#### Dependencies
- Depends on: US-002, US-003

### US-005: Architecture Document
**As a** developer, **I want** a guided interview producing an architecture overview at docs/foundation/architecture.md.

#### Acceptance Criteria
- AC-005.1: Given architecture interview, then grill-me fires Clarity + Edge Cases
- AC-005.2: Given completed interview, then output follows template: System Overview, Bounded Contexts, Data Flow, Tech Stack Rationale, ADRs list, Quality Attributes
- AC-005.3: Given an existing repo, then bounded contexts and tech stack pre-populated
- AC-005.4: Given the doc, then written to docs/foundation/architecture.md (does NOT touch docs/ARCHITECTURE.md — merge is a v2 concern)

#### Dependencies
- Depends on: US-002, US-003

### US-006: ADR Generation
**As a** developer, **I want** an initial ADR from interview decisions.

#### Acceptance Criteria
- AC-006.1: Given tech stack decisions, then ADR follows Status/Context/Decision/Consequences format
- AC-006.2: Given existing ADRs, then auto-detects next available number
- AC-006.3: Given new repo, then uses ADR-0001
- AC-006.4: Given the ADR, then the Context section references the specific tech stack discussed and the Decision section includes at least one named technology with reasoning

#### Dependencies
- Depends on: US-005

### US-007: CLAUDE.md Bootstrapping
**As a** developer starting a new project, **I want** initial CLAUDE.md generated.

#### Acceptance Criteria
- AC-007.1: Given new repo (no CLAUDE.md), then initial CLAUDE.md generated
- AC-007.2: Given generated CLAUDE.md, then includes overview, tech stack, commands, references to PRD/arch
- AC-007.3: Given existing repo with CLAUDE.md, then only command reference entry added

#### Dependencies
- Depends on: US-004, US-005

### US-008: Plan Mode Gate
**As a** developer, **I want** to review docs in Plan Mode before writing.

#### Acceptance Criteria
- AC-008.1: Given interview completion, then EnterPlanMode called
- AC-008.2: Given Plan Mode, then PRD + architecture + ADR outlines displayed
- AC-008.3: Given approval, then files written
- AC-008.4: Given rejection, then no files written

#### Dependencies
- Depends on: US-004, US-005, US-006

### US-009: Adversarial Review
**As a** developer, **I want** documents to pass adversarial review.

#### Acceptance Criteria
- AC-009.1: Given drafts, then spec-adversary reviews with dimensions: completeness, consistency, feasibility, ambiguity, scope
- AC-009.2: Given FAIL, then re-enters interview
- AC-009.3: Given PASS, then files written to docs/foundation/
- AC-009.4: Given 3 FAILs, then user can override or abandon
- AC-009.5: Given docs/foundation/ already exists from a prior run, when command runs again, then existing files are versioned (append revision block) not silently overwritten
- AC-009.6: Given interview abandonment mid-way (user quits), when session resumes, then ecc-workflow state machine allows re-entry from the last completed phase

#### Dependencies
- Depends on: US-008

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| commands/project-foundation.md | Command | Create — new command file |
| skills/grill-me/SKILL.md | Skill | Modify — add foundation-mode (new section, existing modes untouched) |
| skills/interview-me/SKILL.md | Skill | Consumed — all 8 stages used, no modification needed |
| CLAUDE.md | Docs | Modify — command ref + glossary entries |
| docs/commands-reference.md | Docs | Modify — add command entry |

## Constraints

- Pure command/skill work — NO new Rust CLI commands
- Compose grill-me and interview-me — do NOT reimplement
- ecc-domain must remain zero-I/O

## Non-Requirements

- Not a feature spec (that's /spec-dev)
- Not a full architecture design session (that's /design)
- Not a replacement for write-a-prd (that remains feature-level)
- Not implementing merge into ARCHITECTURE.md in v1 (v2 concern)
- Not modifying spec-adversary agent file — custom dimensions passed via Task prompt at invocation time

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| None | None | Pure command/skill — no hexagonal boundary changes |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| New command | CLAUDE.md | Slash Commands | Add /project-foundation |
| New command | commands-reference.md | Full reference | Add documentation |
| Glossary | CLAUDE.md | Glossary | Add foundation document + codebase-analysis phase |
| New mode | CHANGELOG.md | Unreleased | Add foundation-mode entry |
| ADR | docs/adr/ | New ADR | grill-me foundation-mode decision |

## Open Questions

None — all resolved during grill-me interview.
