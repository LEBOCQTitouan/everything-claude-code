# Spec: Grill-Me-Adversary Companion Skill with Adaptive Loop (BL-057)

## Problem Statement

The existing grill-me skill follows a fixed question bank per stage. While effective for structured interviews, it doesn't adapt to the quality of user answers — weak answers get the same follow-up treatment as strong ones. Users who want a harder, more rigorous interview have no way to escalate. A companion skill that dynamically generates harder questions, evaluates answer quality on two axes, and probes deeper on weak answers would make the interview significantly more effective at surfacing genuine weaknesses in proposals and designs.

## Research Summary

- **DEBATE framework** (arxiv 2405.09935): adversarial agent criticizes step-by-step, asks incisive follow-ups — validates the two-agent challenge pattern
- **Adaptive difficulty calibration**: mirrors Item Response Theory — branch harder/softer based on demonstrated response quality, not static sequences
- **Behavioral anchors in rubrics**: each score level needs concrete observable criteria, not vague labels — critical for consistent 0-3 scoring
- **"Firm but curious" stance**: research warns over-aggression causes defensive shutdown; optimal stance is persistent probing without hostility
- **Structured devil's advocate protocol**: opening challenge → rebuttal → escalation → verdict prevents unfocused nitpicking
- **Multi-agent devil's advocate architecture**: adversarial agent must have equal standing to be effective, not performative
- **Conversational flow**: "help me understand" framing extracts more information than "you're wrong" framing

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Opt-in companion skill, not in-place extension | Keeps grill-me core clean; adversary is a separate concern | No |
| 2 | Two scoring axes: completeness (0-3) + specificity (0-3) | Matches DEBATE framework; simple enough for inline display | No |
| 3 | Follow-up threshold: either axis < 2 | Conservative — ensures minimum quality before advancing | No |
| 4 | Three-attempt cap per branch | Prevents infinite loops; flags unresolved branches | No |
| 5 | Always show challenge result (kept/replaced) | User chose — makes adversary reasoning visible at every stage | No |
| 6 | "Firm but curious" tone guidance | Research shows over-aggression kills info extraction | No |
| 7 | Placement in grill-me: after Negative Examples, before Output | Clear separation from core flow | No |
| 8 | Add "Adversary Mode" glossary entry | User chose to document the pattern for discoverability | No |
| 9 | Single-sentence rubric anchors (max 12 words each) | Allows rubric to fit within 500-word budget; accepts reduced granularity as v1 tradeoff | No |

## Rubric Anchor Definitions

These anchors MUST appear in the skill (abbreviated to fit word budget):

**Completeness (0-3):**

| Score | Anchor |
|-------|--------|
| 0 | No relevant aspects addressed |
| 1 | One aspect addressed, major gaps remain |
| 2 | Most aspects addressed, minor gaps |
| 3 | All aspects addressed comprehensively |

**Specificity (0-3):**

| Score | Anchor |
|-------|--------|
| 0 | Entirely vague, no concrete details |
| 1 | Some detail but relies on hand-waving |
| 2 | Concrete examples or data for key claims |
| 3 | Specific, measurable, falsifiable throughout |

## User Stories

### US-001: Create grill-me-adversary skill directory and frontmatter

**As a** spec pipeline user, **I want** a new skill `skills/grill-me-adversary/SKILL.md` with valid YAML frontmatter, **so that** `ecc validate skills` recognizes it as a valid skill.

#### Acceptance Criteria

- AC-001.1: Given the skill directory `skills/grill-me-adversary/` exists, when `ecc validate skills` runs, then the directory is discovered without errors
- AC-001.2: Given `skills/grill-me-adversary/SKILL.md` exists, when its frontmatter is parsed, then it contains exactly `name: grill-me-adversary`, non-empty `description`, and `origin: ECC`
- AC-001.3: Given the frontmatter is parsed, when checking for advisory fields, then `model` and `tools` fields are absent (validator warns but does not fail — this is a convention, not a hard gate)
- AC-001.4: Given the directory name is `grill-me-adversary`, when compared to the `name` frontmatter field, then they match exactly (convention-enforced, not validator-enforced)

#### Dependencies

- Depends on: none

### US-002: Document adversarial question-generation logic

**As a** Claude agent loading grill-me-adversary, **I want** clear instructions on how to synthesize devil's-advocate angles at each stage, **so that** the adversary generates the most uncomfortable question the user has not yet been pushed on.

#### Acceptance Criteria

- AC-002.1: Given the SKILL.md, when an agent reads the adversarial question generation section, then it finds instructions to synthesize a devil's-advocate angle before picking the next question
- AC-002.2: Given the instructions, when identifying weakness, then the agent uses these heuristics: (a) lowest-scored axis from prior answers, (b) most hedging/vague language, (c) most critical to overall proposal viability
- AC-002.3: Given prior questions in the stage, when generating the next question, then the agent avoids angles the user has already been pushed on

#### Dependencies

- Depends on: US-001

### US-003: Document question-generation challenge mechanism

**As a** Claude agent running adversary mode, **I want** instructions to challenge grill-me's initial question selection at the start of each stage, **so that** the planned question is replaced with a harder one when possible.

#### Acceptance Criteria

- AC-003.1: Given the start of each stage, when the adversary evaluates the planned question, then it asks "is this the hardest possible question for this stage?"
- AC-003.2: Given the evaluation determines a harder question exists (one that targets a less obvious failure mode, requires more specific evidence, or challenges an unstated assumption), when the adversary acts, then it substitutes the harder question
- AC-003.3: Given the evaluation, when the result is determined (kept or replaced), then the result is always shown to the user
- AC-003.4: Given the challenge mechanism, when it operates, then grill-me's five-stage structure itself is not altered — only the question content within each stage

#### Dependencies

- Depends on: US-001

### US-004: Document answer evaluation rubric

**As a** Claude agent evaluating user answers, **I want** a rubric with two axes (completeness 0-3, specificity 0-3) and clear scoring definitions, **so that** I can consistently evaluate answers and decide whether to probe deeper.

#### Acceptance Criteria

- AC-004.1: Given the SKILL.md, when an agent reads the rubric section, then it finds two scoring axes: completeness (0-3) and specificity (0-3)
- AC-004.2: Given the rubric, when reading score level definitions, then each level (0-3) has a single-sentence behavioral anchor (max 12 words) matching the definitions in this spec's Rubric Anchor Definitions section
- AC-004.3: Given a user answer is scored, when either axis is below 2, then the adversary probes deeper with a follow-up before advancing
- AC-004.4: Given a user answer is scored, when the scores are determined, then they are shown inline to the user
- AC-004.5: Given a user deflects instead of answering (e.g., "what do you think?", "I'm not sure"), when the adversary evaluates, then the deflection is redirected (not scored) and does not consume a follow-up attempt

#### Dependencies

- Depends on: US-001

### US-005: Document adaptive loop exit conditions and branch status labels

**As a** Claude agent running the adversarial loop, **I want** clear exit conditions and status labels for each branch, **so that** I know when to stop probing and how to label branches in the transcript.

#### Acceptance Criteria

- AC-005.1: Given a branch is being probed, when both completeness and specificity score >= 2, then the branch exits the loop
- AC-005.2: Given a branch is being probed, when three follow-up attempts have been made without both axes reaching >= 2, then the branch exits the loop
- AC-005.3: Given a branch exits on three-attempt exhaustion, when it is labeled, then the label is "stress-tested but unresolved"
- AC-005.4: Given a branch exits with both axes >= 2, when it is labeled, then it uses grill-me's existing resolved branch format (checkmark)
- AC-005.5: Given the three-attempt cap, when reading the skill, then the cap is stated explicitly (not implicit)
- AC-005.6: Given the user says "skip" in adversary mode, when the branch is labeled, then it is labeled "skipped" (distinct from "stress-tested but unresolved") and does not consume follow-up attempts

#### Dependencies

- Depends on: US-004

### US-006: Enforce 500-word limit on grill-me-adversary skill

**As a** project maintainer, **I want** the skill file body to be under 500 words, **so that** it complies with the ECC skill convention for v1 skills.

#### Acceptance Criteria

- AC-006.1: Given `skills/grill-me-adversary/SKILL.md`, when `wc -w` is run on the body (excluding YAML frontmatter), then the count is <= 500 (manual verification — not enforced by validator)
- AC-006.2: Given all content from US-002 through US-005 including single-sentence rubric anchors, when combined in the skill file, then it fits within the 500-word budget

#### Dependencies

- Depends on: US-002, US-003, US-004, US-005

### US-007: Add opt-in "Adversary Mode" section to grill-me

**As a** user of the grill-me skill, **I want** a short section in `skills/grill-me/SKILL.md` explaining how to opt into adversary mode, **so that** I can activate the harder interview when I want it.

#### Acceptance Criteria

- AC-007.1: Given `skills/grill-me/SKILL.md`, when reading the file, then a new section titled "Adversary Mode" exists
- AC-007.2: Given the Adversary Mode section, when counting its lines, then it is <= 5 lines
- AC-007.3: Given the section content, when reading the activation instructions, then it explains the user says "adversary mode" or "hard mode"
- AC-007.4: Given the section content, when reading references, then it references `grill-me-adversary` by name
- AC-007.5: Given the edit, when comparing to the original file, then no other sections are modified — the five-stage flow, branch tracking, vocabulary detection, negative examples, and output format remain untouched
- AC-007.6: Given the base grill-me skill, when checking for scoring UI, then no numeric scoring is added
- AC-007.7: Given the section placement, when reading the file, then "Adversary Mode" appears after "Negative Examples" and before "Output"

#### Dependencies

- Depends on: US-001

### US-008: Validate both skills and regression-check build

**As a** project maintainer, **I want** both skills to pass validation and the build to remain green, **so that** the change does not break the skill ecosystem or Rust toolchain.

#### Acceptance Criteria

- AC-008.1: Given both skill files are complete, when `ecc validate skills` runs, then it exits 0 with no errors
- AC-008.2: Given no Rust code changes, when `cargo clippy -- -D warnings` runs, then it passes
- AC-008.3: Given no Rust code changes, when `cargo test` runs, then it passes

#### Dependencies

- Depends on: US-006, US-007

### US-009: Update glossary and CHANGELOG

**As a** project maintainer, **I want** the glossary and CHANGELOG updated, **so that** the new concept is discoverable and the change is tracked.

#### Acceptance Criteria

- AC-009.1: Given `docs/domain/glossary.md`, when reading the file, then an "Adversary Mode" entry exists defining it as an opt-in enhancement to grill-me that adds adaptive adversarial questioning with scoring
- AC-009.2: Given `CHANGELOG.md`, when reading the file, then a BL-057 entry exists describing the grill-me-adversary companion skill

#### Dependencies

- Depends on: US-006, US-007

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `skills/grill-me-adversary/` | Content (new) | New skill directory + SKILL.md |
| `skills/grill-me/SKILL.md` | Content | Add 5-line Adversary Mode section |
| `docs/domain/glossary.md` | Domain docs | Add "Adversary Mode" entry |
| `CHANGELOG.md` | Project docs | Add BL-057 entry |

No Rust code changes. No port/adapter/domain impact.

## Constraints

- Skill body must be under 500 words (ECC convention, manual verification)
- Rubric anchors use single-sentence format (max 12 words each) to fit word budget
- Must not alter grill-me's 5-stage flow, branch tracking, vocabulary detection, negative examples, or output format
- Must remain opt-in only — never activated by default
- Must pass `ecc validate skills` for both skill files
- Frontmatter must have exactly `name`, `description`, `origin` (no `model` or `tools` by convention)

## Non-Requirements

- No spec-pipeline integration — adversary mode is standalone grill-me only for now
- No new command or agent
- No numeric scoring UI in base grill-me skill
- No restructuring of grill-me's five stages
- No changes to grill-me's transcript output format
- No Rust validator enhancements (name-directory match and word count remain convention-only)

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| Content validation (`ecc validate skills`) | New skill directory | Auto-discovered by validator's `read_dir` scan — no code change needed |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| New concept | Domain | `docs/domain/glossary.md` | Add "Adversary Mode" entry |
| Feature | Project | `CHANGELOG.md` | Add BL-057 entry |

## Open Questions

None — all resolved during grill-me interview and adversarial review.

## Phase Summary

### Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|
| 1 | Scope boundaries | Backlog boundaries sufficient (5 exclusions) | Recommended |
| 2 | Edge case: no harder question | Always show challenge result (kept/replaced) | User |
| 3 | Test strategy | Structural grep + validate + word count | Recommended |
| 4 | UX risk: defensive users | Add "firm but curious" tone guidance | Recommended |
| 5 | Placement of opt-in section | After Negative Examples, before Output | Recommended |
| 6 | Glossary entries | Add "Adversary Mode" to glossary | User |
| 7 | ADR decisions | No ADR needed | Recommended |

### User Stories

| ID | Title | AC Count | Dependencies |
|----|-------|----------|--------------|
| US-001 | Skill directory + frontmatter | 4 | none |
| US-002 | Adversarial question generation | 3 | US-001 |
| US-003 | Question-generation challenge | 4 | US-001 |
| US-004 | Answer evaluation rubric | 5 | US-001 |
| US-005 | Adaptive loop exit + labels | 6 | US-004 |
| US-006 | 500-word limit | 2 | US-002, US-003, US-004, US-005 |
| US-007 | Grill-me opt-in section | 7 | US-001 |
| US-008 | Validation gate | 3 | US-006, US-007 |
| US-009 | Glossary + CHANGELOG | 2 | US-006, US-007 |

### Acceptance Criteria

| AC ID | Description | Source US |
|-------|-------------|----------|
| AC-001.1 | Skill directory discovered by validator | US-001 |
| AC-001.2 | Frontmatter: name, description, origin | US-001 |
| AC-001.3 | No model/tools fields (convention) | US-001 |
| AC-001.4 | Directory name matches name field (convention) | US-001 |
| AC-002.1 | Devil's-advocate angle synthesis | US-002 |
| AC-002.2 | Weakness heuristics (lowest score, hedging, viability) | US-002 |
| AC-002.3 | Avoid repeated angles | US-002 |
| AC-003.1 | Challenge planned question at stage start | US-003 |
| AC-003.2 | Substitute harder question (operational definition) | US-003 |
| AC-003.3 | Always show challenge result | US-003 |
| AC-003.4 | Five-stage structure preserved | US-003 |
| AC-004.1 | Two scoring axes: completeness + specificity | US-004 |
| AC-004.2 | Single-sentence behavioral anchors | US-004 |
| AC-004.3 | Follow-up on score < 2 | US-004 |
| AC-004.4 | Inline score display | US-004 |
| AC-004.5 | Deflections redirected, not scored | US-004 |
| AC-005.1 | Exit when both axes >= 2 | US-005 |
| AC-005.2 | Exit after three attempts | US-005 |
| AC-005.3 | "Stress-tested but unresolved" label | US-005 |
| AC-005.4 | Checkmark for resolved branches | US-005 |
| AC-005.5 | Three-attempt cap explicit | US-005 |
| AC-005.6 | "Skipped" label distinct from exhaustion | US-005 |
| AC-006.1 | Body <= 500 words (manual check) | US-006 |
| AC-006.2 | All content fits budget | US-006 |
| AC-007.1 | Adversary Mode section exists | US-007 |
| AC-007.2 | Section <= 5 lines | US-007 |
| AC-007.3 | Activation: "adversary mode" or "hard mode" | US-007 |
| AC-007.4 | References grill-me-adversary | US-007 |
| AC-007.5 | No other sections modified | US-007 |
| AC-007.6 | No scoring UI in base skill | US-007 |
| AC-007.7 | Placed after Negative Examples, before Output | US-007 |
| AC-008.1 | ecc validate skills passes | US-008 |
| AC-008.2 | cargo clippy passes | US-008 |
| AC-008.3 | cargo test passes | US-008 |
| AC-009.1 | Glossary "Adversary Mode" entry | US-009 |
| AC-009.2 | CHANGELOG BL-057 entry | US-009 |

### Adversary Findings

| Dimension | Verdict | Key Rationale |
|-----------|---------|---------------|
| Ambiguity | PASS (R2) | Weakness heuristics, "harder" definition, rubric anchors added |
| Edge Cases | PASS (R2) | Deflection handling + skip behavior ACs added |
| Scope | PASS (R2) | Single-sentence anchor format resolves 500-word tension |
| Dependencies | PASS (R2) | US-009 covers glossary + CHANGELOG |
| Testability | PASS (R2) | Convention vs. validator distinction clarified |
| Decisions | PASS (R2) | Decision 9 (anchor tradeoff) closes rationale gap |
| Rollback | PASS | Trivial — delete directory + revert section |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-03-23-grill-me-adversary-companion/spec.md | Full spec |
