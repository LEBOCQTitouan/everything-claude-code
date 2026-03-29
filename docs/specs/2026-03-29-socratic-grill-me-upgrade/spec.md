# Spec: Socratic Grill-Me Upgrade (BL-098)

## Problem Statement

The current grill-me skill produces shallow questioning — questions stay surface-level, lack progressive depth drilling, and never confirm mutual understanding via reflective rephrasing. The 25-question cap forces breadth over depth. This limits the quality of specs produced by the /spec pipeline and other consumers.

## Research Summary

- **OARS for LLM agents**: SMART-DREAM system shows structured OARS sequencing improves agent-led conversations by guiding self-discovery rather than prescribing answers
- **LadderBot**: Validates laddering for automated requirement interviews — depth-first "why?" probes climb from attributes to abstract values
- **Socratic taxonomy**: 6 question types (clarification, elenchus, dialectic, maieutics, generalization, counterfactual) map to inductive, deductive, and abductive reasoning chains
- **Anti-patterns**: Infinite regress is real — Socratic prompting needs termination constraints via answer specificity, not fixed counts
- **Interaction contract**: Questions must be answerable by the user and must advance understanding
- **Dialog management**: Structured state machine beats raw LLM generation for question sequencing
- **Depth-first then breadth**: Drill one concern until stable understanding, then move to next topic

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | OARS with Acknowledge (not Affirm) | Avoid sycophantic patterns; factual recognition without praise | Yes (ADR-033) |
| 2 | Uncapped laddering depth | Governed by answer specificity (concrete/falsifiable), not fixed count | Yes (ADR-033) |
| 3 | Universal MECE decomposition | Always partition into ME/CE sub-questions for exhaustive coverage | Yes (ADR-033) |
| 4 | Visible Socratic type annotations | Tags like [Clarification] help users understand questioning strategy | Yes (ADR-033) |
| 5 | Depth profiles: shallow/standard/deep | Control technique intensity per context; modes set defaults | Yes (ADR-033) |
| 6 | Full treatment for backlog-mode | All 4 techniques within existing stage limits | No |
| 7 | Remove 25-question cap | Not breaking — internal to skill, skip/exit preserved | No |

## User Stories

### US-001: OARS Framework Integration

**As a** grill-me user, **I want** every answer followed by Open/Acknowledge/Reflect/Summarize, **so that** I receive confirmation of mutual understanding.

#### Acceptance Criteria

- AC-001.1: Given a user answer, when the skill processes it, then it applies Reflect (restates the answer to confirm understanding)
- AC-001.2: Given the Acknowledge step, when applied, then it uses factual recognition ("That addresses X") without praise
- AC-001.3: Given a stage transition, when moving between stages, then a Summarize is emitted (consolidates themes from completed stage)
- AC-001.4: Given a follow-up question, when generated, then it is Open (invites narrative, not yes/no)
- AC-001.5: Given adversary mode is active, when OARS and adversary both fire, then OARS Reflect fires before adversary scoring (ordering explicit)

#### Dependencies

- Depends on: none

### US-002: Laddering (Progressive Depth)

**As a** grill-me user, **I want** progressive "why?" drilling, **so that** questions reach concrete, falsifiable statements.

#### Acceptance Criteria

- AC-002.1: Given a user answer, when the answer is abstract or vague, then a laddering follow-up is generated ("Why is that important?")
- AC-002.2: Given laddering depth, when the user provides a concrete/falsifiable answer, then laddering stops and moves to next question
- AC-002.3: Given a ladder chain, when the user wants to exit, then skip/exit behavior is preserved
- AC-002.4: Given "abstract/vague" detection, then answers lacking concrete nouns, measurable quantities, or falsifiable claims trigger laddering
- AC-002.5: Given a safety-valve depth of 7 ladder levels on a single question, then laddering stops with an explicit "depth limit reached" note and moves to next question

#### Dependencies

- Depends on: US-001

### US-003: MECE Decomposition

**As a** grill-me user, **I want** every requirement space decomposed into ME/CE sub-questions, **so that** nothing is missed and nothing overlaps.

#### Acceptance Criteria

- AC-003.1: Given a requirement or design space, when decomposing, then sub-questions are mutually exclusive (no overlap)
- AC-003.2: Given decomposition, when complete, then sub-questions are collectively exhaustive (no gaps)
- AC-003.3: Given any topic, when generating questions, then MECE is applied universally (not only for complex topics)
- AC-003.4: Given an atomic, non-decomposable topic (e.g., "What is the project name?"), when MECE would produce nonsense sub-questions, then MECE is skipped with a rationale note

#### Dependencies

- Depends on: none

### US-004: Socratic 6-Type Rotation

**As a** grill-me user, **I want** questions to cycle through 6 Socratic types, **so that** reasoning is challenged from multiple angles.

#### Acceptance Criteria

- AC-004.1: Given each question, when generated, then it is annotated with a type tag: [Clarification], [Assumption], [Evidence], [Viewpoint], [Implication], or [Meta]
- AC-004.2: Given questions across stages, when counted by type, then no single type dominates (balanced rotation)
- AC-004.3: Given type annotations, when displayed, then they are visible to the user during the interview
- AC-004.4: Given balanced rotation, then no single type exceeds 2x its fair share across a session (e.g., for 12 questions, no type appears more than 4 times)

#### Dependencies

- Depends on: none

### US-005: Depth Profiles

**As a** grill-me consumer, **I want** depth profiles (shallow/standard/deep), **so that** technique intensity matches context.

#### Acceptance Criteria

- AC-005.1: Given `shallow` profile, when applied, then only OARS Reflect is used (no laddering, MECE limited to top-level decomposition with max 2 branches)
- AC-005.2: Given `standard` profile, when applied, then full OARS + 1-2 ladder levels + MECE on requirement spaces
- AC-005.3: Given `deep` profile, when applied, then all techniques at full intensity with unlimited laddering
- AC-005.4: Given mode defaults, then backlog=standard, spec=deep, standalone=deep
- AC-005.5: Given a consuming command, when it sets a profile, then the profile overrides the mode default
- AC-005.6: Given a depth profile that conflicts with mode stage/question limits (e.g., deep + backlog), then mode limits take precedence (profile controls technique intensity within those limits)
- AC-005.7: Given profile inference from mode, then it is documented in the skill file (spec=deep, backlog=standard, standalone=deep)

#### Dependencies

- Depends on: US-001, US-002, US-003, US-004

### US-006: Structural Updates

**As a** grill-me maintainer, **I want** the 25-question cap removed and adversary updated, **so that** the skill is structurally aligned with depth-first questioning.

#### Acceptance Criteria

- AC-006.1: Given the skill file, when the 25-question cap section is checked, then it is removed
- AC-006.2: Given grill-me-adversary, when its question generation is checked, then it uses enhanced question types (OARS, laddering, Socratic rotation)
- AC-006.3: Given any mode, when the user requests skip or exit, then the behavior is preserved
- AC-006.4: Given all three modes (standalone, spec-mode, backlog-mode), when checked, then all are updated with the new techniques
- AC-006.5: Given the implementation, then ADR-033 is created covering decisions 1-5, including the mapping from research taxonomy (elenchus, maieutics) to implementation types (Assumption, Evidence)
- AC-006.6: Given grill-me-adversary's question-generation challenge, then it uses Socratic type annotations when substituting questions
- AC-006.7: Given the skill file structure, then required section headers are present: OARS Protocol, Laddering, MECE Decomposition, Socratic Type Rotation, Depth Profiles

### Verification Strategy

Since this is a markdown-only change with no Rust code, verification uses three mechanisms:

1. **Structural validation (automated)**: grep-based checks that the skill file contains required section headers, tag formats (`[Type]`), profile names, and that the 25-question cap is removed
2. **Manual scenario verification**: Run grill-me with a test prompt ("build a REST API") and verify: OARS sequence appears, ladder follow-up on abstract answer, Socratic tags visible, Summarize at stage transition
3. **Code review**: `/verify` reviews the skill content for completeness against this spec

#### Dependencies

- Depends on: US-001, US-002, US-003, US-004, US-005

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `skills/grill-me/SKILL.md` | Skill | Core protocol rewrite with 4 techniques + depth profiles |
| `skills/grill-me-adversary/SKILL.md` | Skill | Companion update to use enhanced question types |

## Constraints

- Markdown-only change — no Rust code
- Skip/exit behavior must be preserved across all modes
- All existing consumers (/spec-*, /backlog, standalone) must work without modification
- Depth profiles are skill-internal — consumers set profile via existing mode parameter
- One question at a time via AskUserQuestion (existing constraint, per feedback memory)

## Rollback Plan

- **Git revert**: Revert the commits modifying `skills/grill-me/SKILL.md` and `skills/grill-me-adversary/SKILL.md` to restore pre-upgrade behavior
- **Behavioral fallback**: Set depth profile to `shallow` across all modes — approximates pre-upgrade behavior (OARS Reflect only, no laddering, minimal MECE)
- **ADR cleanup**: If rollback is permanent, supersede ADR-033 with rationale

## Non-Requirements

- Rust code changes
- New CLI subcommands
- Changes to consumer commands (/spec-*, /backlog)
- Training data or fine-tuning
- Automated testing of question quality (subjective)

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| Skills (markdown) | Content rewrite | All /spec-* consumers use updated skill automatically |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| Skill rewrite | skills/ | grill-me/SKILL.md | Full rewrite with 4 techniques + profiles |
| Companion update | skills/ | grill-me-adversary/SKILL.md | Update to match enhanced types |
| Architecture decision | docs/adr/ | New ADR-033 | Socratic questioning protocol |
| CHANGELOG | CHANGELOG.md | project | Add BL-098 entry |

## Open Questions

None — all resolved during grill-me interview.

## Phase Summary

### Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|
| 1 | Laddering cap | Truly uncapped, governed by answer specificity | User |
| 2 | OARS Affirm | Acknowledge (no praise) — factual recognition | User |
| 3 | MECE scope | Universal — always decompose | User |
| 4 | Socratic annotations | Visible during interview as tags | User |
| 5 | Backlog-mode | Full treatment within existing limits | User |
| 6 | Breaking changes | Not breaking — cap internal, skip/exit preserved | Recommended |
| 7 | Depth profile | Include in v1 — shallow/standard/deep | User |
| 8 | ADR | One ADR for Socratic questioning protocol | User |

### User Stories

| ID | Title | AC Count | Dependencies |
|----|-------|----------|--------------|
| US-001 | OARS Framework Integration | 4 | none |
| US-002 | Laddering | 3 | US-001 |
| US-003 | MECE Decomposition | 3 | none |
| US-004 | Socratic 6-Type Rotation | 3 | none |
| US-005 | Depth Profiles | 5 | US-001-004 |
| US-006 | Structural Updates | 4 | US-001-005 |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-03-29-socratic-grill-me-upgrade/spec.md | Full spec |
