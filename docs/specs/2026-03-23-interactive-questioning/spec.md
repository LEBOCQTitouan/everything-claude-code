# Spec: Interactive Stage-by-Stage Questioning via AskUserQuestion (BL-061)

## Problem Statement

The ECC codebase has three independent questioning systems that serve the same purpose (challenge an idea before proceeding) but share no common protocol: (1) standalone grill-me with 5-stage adversarial interviews, (2) spec-pipeline-shared with inline "Grill-Me Interview Rules" that are actually a different fixed-question clarification model, and (3) backlog-curator with ad-hoc 1-3 question batches. The "grill-me" name is overloaded to mean two different things, interview rules are duplicated across 4 files, and the backlog challenge is disconnected from the formal protocol. BL-061 unifies these into a single universal grill-me protocol with stage-by-stage AskUserQuestion interaction, challenge loops, cross-stage mutation, and hook-based enforcement.

## Research Summary

- **Stage-gated questioning with follow-ups**: Survey design patterns show follow-up questions should branch from answers, with loop-back capability. Questions stay "open" until resolved.
- **Interactive CLI prompt patterns**: CLI tools (Inquirer.js, promptui) support conditional `when` logic for dynamic questioning. AskUserQuestion provides similar capability natively.
- **Closed-loop feedback**: Best practice is inner loops (challenge until resolved) within an outer loop (stage progression).
- **Challenge difficulty matching**: Users stay engaged when challenge difficulty matches capability. Grill-me-adversary's scoring aligns with this.
- **Agent structured deliberation**: Multi-agent CoT stages with feedback loops map naturally to the stage-by-stage model.

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Grill-me becomes the universal questioning protocol for all contexts | Eliminates 3 independent systems, removes naming collision, enables enforcement | Yes |
| 2 | Canonical stage names: Clarity, Assumptions, Edge Cases, Alternatives, Stress Test | Aligns with BL-061 backlog item. Replaces current grill-me's (Problem, Edge Cases, Scope, Rollback, Success Criteria). Mapping: Problem→Clarity, Scope→Assumptions, Rollback→Alternatives, Success Criteria→Stress Test | No |
| 3 | Spec-pipeline-shared grill-me rules section replaced with grill-me skill reference | Spec commands invoke grill-me directly with spec-mode parameters (recommended answers, "spec it" shortcut). Rules section deleted from spec-pipeline-shared. | No |
| 4 | Backlog delegates to grill-me with lighter config (max 3 stages, 2 questions/stage) | Replaces ad-hoc 1-3 question batch. Claude can escalate to full 5 stages for HIGH/EPIC scope ideas. | No |
| 5 | Hook enforcement via Stop hook checking campaign.md/spec output | grill-me-gate.sh runs as a Stop hook, checks that grill-me decision table is present in spec output. NOT a runtime tool-invocation check — a content-presence check consistent with existing hooks. | No |
| 6 | Challenge loop exit: 2 follow-ups or Claude judges complete (stated explicitly) | Non-adversary mode has concrete exit criterion. Adversary mode keeps existing rubric (both axes >= 2, or 3 attempts). | No |
| 7 | Total question cap: 25. Stage reopen limit: once per stage. | Prevents infinite cross-stage mutation loops. | No |
| 8 | "Skip all" / early termination: all unanswered questions recorded as "skipped" | If >50% skipped, warn user about degraded output quality. | No |
| 9 | Rollback via git tag pre-bl-061 and feature branch | Before modifications, tag current state. Implementation in feature branch (or atomic commits on main with rollback tag). | No |

## User Stories

### US-001: Refactor grill-me to stage-by-stage AskUserQuestion model

**As a** developer using grill-me, **I want** questions asked one-at-a-time via AskUserQuestion with challenge loops and cross-stage mutation, **so that** the interview is interactive, adaptive, and produces thorough coverage.

#### Acceptance Criteria

- AC-001.1: Given grill-me is invoked, when it builds the question list, then questions are grouped by stage (Clarity, Assumptions, Edge Cases, Alternatives, Stress Test)
- AC-001.2: Given a stage, when questions are asked, then each is presented via AskUserQuestion one at a time
- AC-001.3: Given an answer, when Claude evaluates it as challengeable, then a follow-up challenge is added under the same question and re-asked via AskUserQuestion
- AC-001.4: Given a question with challenges in non-adversary mode, when the follow-up answer is specific and complete OR 2 follow-up attempts are exhausted, then the question is marked "answered" and Claude states the termination reason
- AC-001.5: Given any answer, when it reveals a new concern, then Claude can add new questions to any stage (including earlier completed stages)
- AC-001.6: Given stage progression, when all questions in a stage are answered, then the next stage begins
- AC-001.7: Given the question list, when displayed after each interaction, then it shows: stage grouping, question text, status (pending/open/challenged/answered), challenge thread
- AC-001.8: Given all stages exhausted, when grill-me completes, then it produces the final output (decision log)
- AC-001.9: Given the total question count, when it reaches 25, then no new questions are added and remaining stages proceed with existing questions only
- AC-001.10: Given a completed stage that receives new questions from cross-stage mutation, when it reopens, then it reopens exactly once. Further mutations are queued as notes in the output.
- AC-001.11: Given the user says "skip all" or "done", when grill-me processes this, then it ends immediately with all unanswered questions recorded as "skipped". If >50% skipped, a degraded-quality warning is displayed.
- AC-001.12: Given AskUserQuestion enforcement, when verifying compliance, then all questioning instructions in the skill use "AskUserQuestion" or "ask via AskUserQuestion" language. Manual review confirms this.

#### Dependencies

- Depends on: none

### US-002: Integrate grill-me into spec pipeline

**As a** developer running `/spec-*`, **I want** the spec pipeline to invoke the grill-me skill directly instead of using inline rules, **so that** there is one canonical questioning protocol.

#### Acceptance Criteria

- AC-002.1: Given spec-pipeline-shared, when the "Grill-Me Interview Rules" section is removed, then it is replaced with a reference: "Use the grill-me skill with spec-mode parameters"
- AC-002.2: Given spec-dev/fix/refactor, when the grill-me phase runs, then it invokes the grill-me skill with domain-specific mandatory questions passed as context and spec-mode enabled
- AC-002.3: Given spec-mode, when enabled, then the "spec it" shortcut accepts all remaining recommended answers and ends the interview
- AC-002.4: Given spec-mode, when enabled, then recommended answers are presented as the first AskUserQuestion option with "(Recommended)" label
- AC-002.5: Given spec commands, when they currently inline interview rules after `> Shared: See...`, then the inlined copy is removed
- AC-002.6: Given grill-me in spec context, when it produces output, then the output is persisted to campaign.md's `## Grill-Me Decisions` table

#### Dependencies

- Depends on: US-001

### US-003: Integrate grill-me into backlog

**As a** developer adding a backlog item, **I want** `/backlog add` to use grill-me as its challenge mechanism, **so that** ideas are rigorously challenged before being added.

#### Acceptance Criteria

- AC-003.1: Given `/backlog add`, when the challenge phase starts, then it invokes the grill-me skill
- AC-003.2: Given grill-me in backlog context, when the idea scope is LOW or MEDIUM, then grill-me runs with max 3 stages (Clarity, Assumptions, Edge Cases) and max 2 questions per stage
- AC-003.3: Given grill-me in backlog context, when the idea scope is HIGH or EPIC, then grill-me runs with all 5 stages
- AC-003.4: Given grill-me in backlog context, when it completes, then the output feeds into prompt optimization and entry creation
- AC-003.5: Given backlog-curator, when it previously asked 1-3 batched questions, then it now delegates to grill-me for one-at-a-time questioning
- AC-003.6: Given backlog-management skill, when the Challenge Log is populated, then it contains the grill-me output (stages, questions, answers, challenges)

#### Dependencies

- Depends on: US-001

### US-004: Align grill-me-adversary with stage-by-stage model

**As a** developer using adversary mode, **I want** grill-me-adversary to work within the stage-by-stage protocol, **so that** scoring and probing happen within the stage progression.

#### Acceptance Criteria

- AC-004.1: Given adversary mode is activated, when a question is asked, then the adversary evaluates whether it's the hardest possible question for that stage
- AC-004.2: Given an answer in adversary mode, when scored, then completeness and specificity scores (0-3) are displayed alongside the challenge decision
- AC-004.3: Given the stage-by-stage model, when adversary mode adds follow-ups, then they are added as challenges under the current question
- AC-004.4: Given grill-me-adversary, when it references grill-me, then it uses explicit skill reference (not informal cross-references)
- AC-004.5: Given adversary mode stage names, when referenced, then they use the canonical names (Clarity, Assumptions, Edge Cases, Alternatives, Stress Test)

#### Dependencies

- Depends on: US-001

### US-005: Decompose spec-pipeline-shared

**As a** maintainer, **I want** spec-pipeline-shared to have reduced scope after grill-me rules are extracted, **so that** it's no longer a grab-bag.

#### Acceptance Criteria

- AC-005.1: Given spec-pipeline-shared, when grill-me interview rules are removed, then only these sections remain: project detection, adversarial review + verdict handling, spec output schema
- AC-005.2: Given spec commands, when they reference spec-pipeline-shared, then they find the reduced-scope content
- AC-005.3: Given the removed grill-me rules, when a user reads the grill-me skill, then all interaction rules (including spec-mode and backlog-mode parameters) are in one place

#### Dependencies

- Depends on: US-002

### US-006: Hook-based enforcement of grill-me completion

**As a** workflow enforcer, **I want** a Stop hook that checks grill-me output completeness before the session ends, **so that** the interview cannot be skipped.

#### Acceptance Criteria

- AC-006.1: Given `grill-me-gate.sh` as a Stop hook, when the session ends during a spec workflow, then it checks for grill-me decision table presence in spec output or campaign.md
- AC-006.2: Given incomplete grill-me output (no decision table found), when the hook runs, then it emits a WARNING with "Grill-me interview not completed or not found in output"
- AC-006.3: Given complete grill-me output, when the hook runs, then it passes silently
- AC-006.4: Given `ECC_WORKFLOW_BYPASS=1`, when the hook runs, then it exits 0
- AC-006.5: Given the hook script, when it starts, then it uses `set -uo pipefail`
- AC-006.6: Given the hook mechanism, when it validates, then it performs a content-presence check (grep for grill-me decision markers), NOT a runtime tool-invocation check. Consistent with existing hooks like doc-enforcement.sh.

#### Dependencies

- Depends on: US-002

### US-007: Fix backlog.md frontmatter

**As a** maintainer, **I want** `backlog.md` to have proper `allowed-tools` frontmatter, **so that** it complies with ECC conventions.

#### Acceptance Criteria

- AC-007.1: Given `commands/backlog.md`, when checked, then `allowed-tools` is present in frontmatter
- AC-007.2: Given existing backlog functionality, when allowed-tools is added, then no behavior changes

#### Dependencies

- Depends on: none

### US-008: Documentation updates

**As a** user, **I want** the glossary, CHANGELOG, and ADR updated, **so that** the change is discoverable.

#### Acceptance Criteria

- AC-008.1: Given `docs/domain/glossary.md`, when complete, then grill-me entry updated to "universal questioning protocol"
- AC-008.2: Given `CHANGELOG.md`, when complete, then BL-061 entry exists under Refactoring
- AC-008.3: Given ADR 0017, when created, then it documents the universal protocol decision
- AC-008.4: Given rollback preparation, when starting implementation, then current file versions are tagged as `pre-bl-061` in git

#### Dependencies

- Depends on: US-001, US-002, US-003

## Affected Modules

| Module | Layer | Nature of Change |
|--------|-------|-----------------|
| `skills/grill-me/SKILL.md` | Skill | Modified: stage-by-stage model, challenge loops, spec-mode/backlog-mode params, question cap |
| `skills/grill-me-adversary/SKILL.md` | Skill | Modified: align stages, explicit composition |
| `skills/spec-pipeline-shared/SKILL.md` | Skill | Modified: remove grill-me rules, reduced scope |
| `commands/spec-dev.md` | Command | Modified: invoke grill-me skill, remove inlined rules |
| `commands/spec-fix.md` | Command | Modified: invoke grill-me skill, remove inlined rules |
| `commands/spec-refactor.md` | Command | Modified: invoke grill-me skill, remove inlined rules |
| `commands/backlog.md` | Command | Modified: delegate to grill-me, add allowed-tools |
| `skills/backlog-management/SKILL.md` | Skill | Modified: update challenge log format |
| `agents/backlog-curator.md` | Agent | Modified: delegate to grill-me |
| `.claude/hooks/grill-me-gate.sh` | Hook (new) | Created: Stop hook for grill-me completion check |
| `docs/domain/glossary.md` | Docs | Modified: update grill-me entry |
| `CHANGELOG.md` | Docs | Modified: BL-061 entry |
| `docs/adr/0017-grill-me-universal-protocol.md` | Docs (new) | Created: universal protocol ADR |

## Constraints

- All refactoring steps must be behavior-preserving for existing users
- `cargo test` must pass (all ~1224 tests)
- `cargo clippy -- -D warnings` must pass
- Hook follows existing conventions: `ECC_WORKFLOW_BYPASS`, `set -uo pipefail`
- Grill-me skill must remain usable standalone
- "spec it" shortcut must continue working in spec contexts
- Campaign.md persistence must continue working
- Total question cap: 25. Stage reopen limit: once per completed stage.
- Before modifications, tag current state as `pre-bl-061` for rollback

## Non-Requirements

- Custom TUI widgets beyond AskUserQuestion
- Persistent question list across sessions
- Changes to spec-adversary or solution-adversary
- Modifying the spec output schema
- Any Rust CLI code changes (hook is shell-based)

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| None | No port/adapter changes | Pure skill/command/hook changes |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| Updated concept | Domain | `docs/domain/glossary.md` | Update grill-me entry |
| Feature entry | Project | `CHANGELOG.md` | Add BL-061 refactoring entry |
| New ADR | Architecture | `docs/adr/0017-grill-me-universal-protocol.md` | Universal protocol decision |

## Open Questions

None — all resolved during grill-me interview and adversarial review.

## Dependency Graph

```
US-001 (grill-me core refactor)
  |
  +--------+--------+
  |        |        |
  v        v        v
US-002   US-003   US-004
(spec)   (backlog) (adversary)
  |        |
  v        |
US-005     |
(decompose)|
  |        |
  +--------+
  |
  v
US-006 (hook enforcement)
  |
  v
US-007 (frontmatter — independent)
  |
  v
US-008 (docs — depends on US-001, US-002, US-003)
```
