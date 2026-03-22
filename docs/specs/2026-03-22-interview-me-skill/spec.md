# Spec: Create interview-me skill + interviewer agent (BL-013)

## Problem Statement

Developers need a structured way to extract requirements collaboratively before starting a `/spec` workflow. The existing grill-me skill is adversarial (stress-testing ideas), but there's no complementary collaborative interview tool that helps users think through current state, desired end state, constraints, stakeholders, dependencies, and failure modes. Additionally, `ecc validate skills` only checks that SKILL.md exists and is non-empty — it doesn't validate required frontmatter fields, allowing malformed skills to ship undetected.

## Research Summary

- Structured elicitation with open-ended → specific → follow-up question progression captures 34% more requirements than unstructured approaches
- AI-assisted interviews should read context before asking to avoid redundant questions
- Security hard-gates between requirements gathering and design prevent late-discovery of critical gaps
- Collaborative tone (extract & help) is distinct from adversarial tone (challenge & stress-test)
- The ECC skill/agent split pattern (passive knowledge + behavioral orchestration) is well-established

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Split into skill + agent (interview-me + interviewer) | Follows design-an-interface / interface-designer precedent | No |
| 2 | Dedicated Phase 1 codebase exploration before interview | Cleaner than interleaved, follows interface-designer pattern | No |
| 3 | Security gate after constraints stage | Most precise placement — gather context first, then gate | No |
| 4 | Lean skill (~300 words), detailed agent | 8 stages + security checkpoint won't fit in 500 words | No |
| 5 | Add frontmatter validation to ecc validate skills | Closes real validation gap, catches malformed skills | Yes |
| 6 | Fix 4 existing malformed skills | Necessary to avoid false positives from new validation | No |
| 7 | Output to docs/interviews/ (existing convention) | Already used by grill-me, already in .gitignore | No |

## User Stories

### US-001: Interview-Me Skill (Methodology)

**As a** developer, **I want** a passive knowledge skill defining the collaborative requirements interview methodology, **so that** any agent referencing it follows a consistent, non-adversarial interview protocol.

#### Acceptance Criteria

- AC-001.1: Given `skills/interview-me/SKILL.md` exists, when inspected, then it has frontmatter with `name: interview-me`, `description`, `origin: ECC` and no `model` or `tools` fields.
- AC-001.2: Given the skill content, when measured, then it is under 500 words.
- AC-001.3: Given the skill content, when read, then it defines: trigger phrases ("interview me", "help me think through", "extract requirements", "what should I consider"), interview stages (current state, desired state, constraints, security checkpoint, stakeholders, dependencies, prior art, failure modes), output format, and at least 1 negative example.
- AC-001.4: Given the skill's interview stages, when compared to grill-me, then they are distinct — collaborative extraction vs adversarial stress-testing with no stage name overlap.
- AC-001.5: Given the skill, when `ecc validate skills` runs, then it passes with no errors.

#### Dependencies

- Depends on: none

### US-002: Interviewer Agent (Orchestration)

**As a** developer, **I want** an agent that orchestrates the interview-me methodology with codebase-aware questioning, **so that** I get structured requirements extraction without wasting time on questions the codebase can answer.

#### Acceptance Criteria

- AC-002.1: Given `agents/interviewer.md` exists, when inspected, then it has frontmatter with `name: interviewer`, `description`, `model: opus`, `tools: [Read, Grep, Glob, Agent, Write, AskUserQuestion, TodoWrite, TodoRead]`, `skills: ["interview-me"]`.
- AC-002.2: Given the agent is invoked, when it starts, then it reads the codebase (architecture, relevant modules, existing patterns) in a dedicated Phase 1 BEFORE asking the first question. If the codebase has no relevant code to explore (empty repo, new project), the agent skips Phase 1 exploration and states it found no existing context before proceeding to interview.
- AC-002.3: Given the agent reads the codebase, when it formulates questions, then it skips questions whose answers are already evident from code and tells the user what it already knows.
- AC-002.4: Given the agent is in the constraints stage, when it detects unaddressed security implications, then it hard-blocks: flags the gap immediately and refuses to proceed until the user addresses it.
- AC-002.5: Given the interview completes, when output is persisted, then it writes to `docs/interviews/{topic}-{date}.md`. If the file exists, append a numeric suffix (e.g., `{topic}-{date}-2.md`).
- AC-002.6: Given the agent, when it asks questions, then it uses AskUserQuestion, one question per turn (never batches).
- AC-002.7: Given the agent frontmatter, when inspected, then it includes TodoWrite with graceful degradation.
- AC-002.8: Given the user ends the interview early, when output is persisted, then partial notes are written with "Stages completed: N/M" indicator.

#### Dependencies

- Depends on: US-001

### US-003: Skill Frontmatter Validation Enhancement

**As a** maintainer, **I want** `ecc validate skills` to check that skills have required frontmatter fields (name, description, origin), **so that** malformed skills are caught before they ship.

#### Acceptance Criteria

- AC-003.1: Given a skill with missing `name` field, when `ecc validate skills` runs, then it reports an error.
- AC-003.2: Given a skill with missing `description` field, when `ecc validate skills` runs, then it reports an error.
- AC-003.3: Given a skill with missing `origin` field, when `ecc validate skills` runs, then it reports an error.
- AC-003.4: Given a skill with all three required fields, when `ecc validate skills` runs, then it passes.
- AC-003.5: Given a skill with `model` or `tools` fields, when `ecc validate skills` runs, then it reports a warning.
- AC-003.6: Given existing skills with fixed frontmatter, when validation runs, then all pass with no false positives.

#### Dependencies

- Depends on: US-004

### US-004: Fix Existing Malformed Skills

**As a** maintainer, **I want** the 4 skills with missing frontmatter fields fixed, **so that** the new validation doesn't produce false positives.

#### Acceptance Criteria

- AC-004.1: Given `skills/foundation-models-on-device/SKILL.md`, when inspected, then it has `origin: ECC`.
- AC-004.2: Given `skills/skill-stocktake/SKILL.md`, when inspected, then it has `name: skill-stocktake`.
- AC-004.3: Given `skills/swift-concurrency-6-2/SKILL.md`, when inspected, then it has `origin: ECC`.
- AC-004.4: Given `skills/swiftui-patterns/SKILL.md`, when inspected, then it has `origin: ECC`.
- AC-004.5: Given all existing skills, when `ecc validate skills` runs with enhanced validation, then all pass.

#### Dependencies

- Depends on: none

### US-005: Documentation

**As a** developer, **I want** the changes documented, **so that** the interview-me skill, interviewer agent, and validation enhancement are discoverable.

#### Acceptance Criteria

- AC-005.1: Given `docs/domain/glossary.md`, when read, then it includes "Interview Me" and "Interviewer" definitions.
- AC-005.2: Given `CHANGELOG.md`, when read, then it includes a BL-013 feature entry.
- AC-005.3: Given `docs/adr/0010-skill-frontmatter-validation.md`, when read, then it documents the validation enhancement with Status/Context/Decision/Consequences.

#### Dependencies

- Depends on: US-001, US-002, US-003, US-004

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| skills/interview-me/SKILL.md | Content (skill) | New — collaborative interview methodology |
| agents/interviewer.md | Content (agent) | New — orchestration with codebase exploration |
| crates/ecc-app/src/validate.rs | Application | Modify — add frontmatter field validation to validate_skills |
| skills/foundation-models-on-device/SKILL.md | Content (skill) | Fix — add missing origin field |
| skills/skill-stocktake/SKILL.md | Content (skill) | Fix — add missing name field |
| skills/swift-concurrency-6-2/SKILL.md | Content (skill) | Fix — add missing origin field |
| skills/swiftui-patterns/SKILL.md | Content (skill) | Fix — add missing origin field |
| docs/domain/glossary.md | Documentation | Modify — add terms |
| docs/adr/0010-skill-frontmatter-validation.md | Documentation | New — ADR |
| CHANGELOG.md | Documentation | Modify — add entry |

No hexagonal boundary changes. Only `ecc-app` crate modified (validate_skills function).

## Constraints

- Skill must be under 500 words
- Agent must include TodoWrite with graceful degradation
- Agent must use AskUserQuestion one question per turn
- Validation enhancement must not break existing passing skills
- All changes must be behavior-preserving for existing workflows
- `docs/interviews/` directory convention already exists (reuse, don't create)
- AC-002.2 through AC-002.8 describe agent behaviors verified by content validation tests (grep-based assertions on the agent file) and exploratory testing during implementation, not automated behavioral tests

## Non-Requirements

- No /spec pipeline integration (interview-me is standalone)
- No modification of grill-me skill
- No /spec integration for interview output

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| ecc validate skills (FileSystem port) | Enhanced validation logic | Existing integration tests cover generically; new unit tests needed for frontmatter checks |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| Feature | project | CHANGELOG.md | Add BL-013 entry |
| Term | domain | docs/domain/glossary.md | Add Interview Me + Interviewer definitions |
| Decision | architecture | docs/adr/ | ADR 0010 for skill frontmatter validation |

## Open Questions

None — all resolved during grill-me interview.
