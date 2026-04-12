# Spec: BL-149 Agentic Self-Evaluation in /implement

## Problem Statement

ECC's /implement TDD loop has implicit gates (tests green, clippy clean) but no explicit evaluation of whether each iteration advances the spec. Tests can pass while the AC isn't truly satisfied — a stub that makes assertions pass, a refactor that breaks an untested public API, or a constraint discovered mid-implementation that invalidates remaining PCs. Goose's agent loop explicitly evaluates after each iteration. ECC should do the same, grounded in its existing test-based verification rather than LLM-as-judge alone.

## Research Summary

- Feedback-driven loops with empirical verification show 17-53% performance gains over one-shot (ComPilot, ReVeal)
- Self-evaluation without external ground truth degrades quality — agents confabulate success
- Multi-step reflection effective when agents compare against test results and error logs
- Short memory windows cause repeated mistakes — historical feedback accumulation essential
- Static evaluation rubrics are brittle; dynamic criteria from test results adapt better
- Clean passing test suites massively amplify agent value — the test suite IS the evaluation signal
- Goose's evaluate step asks "did this move us toward the goal?"

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Lightweight read-only subagent | Keeps parent context clean, consistent with agent-per-concern | Yes |
| 2 | Conditional triggers | Saves tokens: only fix rounds > 0, integration/e2e, wave boundary | No |
| 3 | Achievability on every triggered eval | User preference over budget-exhaust-only | No |
| 4 | FAIL-only escalation + 3-WARN auto-escalate | Prevents user fatigue | No |
| 5 | Eval after fix-round budget resolves | PC must pass/skip first | No |
| 6 | New pc-evaluator agent + skill | Agent for dispatch, skill for rubric | No |
| 7 | eval@timestamp in tasks.md | New status between green and done | No |

## User Stories

### US-001: PC Evaluator Skill
**As a** developer, **I want** the evaluation rubric in a reusable skill at `skills/pc-evaluation/SKILL.md`.

#### Acceptance Criteria
- AC-001.1: Skill defines 3 dimensions: AC satisfaction, regression heuristics, spec achievability
- AC-001.2: Each dimension has PASS/WARN/FAIL criteria with concrete thresholds
- AC-001.3: `ecc validate skills` passes
- AC-001.4: Documents conditional trigger rules

#### Dependencies
- Depends on: none

### US-002: PC Evaluator Agent
**As a** developer, **I want** a read-only agent at `agents/pc-evaluator.md`.

#### Acceptance Criteria
- AC-002.1: Tools are [Read, Grep, Glob] only
- AC-002.2: Receives PC result, AC text, files changed, prior PC results
- AC-002.3: Returns structured output: ac_satisfied, regressions, achievability (each PASS/WARN/FAIL), rationale
- AC-002.4: `ecc validate agents` passes

#### Dependencies
- Depends on: US-001

### US-003: /implement Integration
**As a** developer, **I want** evaluation in Phase 3 TDD Loop.

#### Acceptance Criteria
- AC-003.1: PC with fix_round_count > 0 triggers evaluation
- AC-003.2: Integration/e2e PC triggers evaluation on first-try pass
- AC-003.3: Last PC in wave triggers evaluation regardless of type
- AC-003.4: Clean unit PC first-try pass skips evaluation
- AC-003.5: WARN logged to tasks.md, pipeline continues
- AC-003.6: FAIL triggers AskUserQuestion: Re-dispatch, Accept, Pause/revise spec, Abort
- AC-003.7: 3 consecutive WARNs auto-escalate to FAIL
- AC-003.8: "Pause and revise spec" preserves state, stops cleanly

#### Dependencies
- Depends on: US-001, US-002

### US-004: implement-done.md Self-Evaluation Log
**As a** developer, **I want** evaluation results in implement-done.md.

#### Acceptance Criteria
- AC-004.1: Schema includes `## Self-Evaluation Log` section
- AC-004.2: Columns: PC ID, AC Verdict, Regression Verdict, Achievability Verdict, User Decision
- AC-004.3: Skipped PCs show "SKIPPED (clean unit)"

#### Dependencies
- Depends on: US-003

### US-005: Documentation Updates
**As a** developer, **I want** CHANGELOG, CLAUDE.md, team manifest, and skill files updated.

#### Acceptance Criteria
- AC-005.1: CHANGELOG has BL-149 entry
- AC-005.2: CLAUDE.md glossary has "self-evaluation" definition
- AC-005.3: teams/implement-team.md lists pc-evaluator agent
- AC-005.4: skills/progress-tracking/SKILL.md documents eval@timestamp
- AC-005.5: skills/tasks-generation/SKILL.md documents eval status marker

#### Dependencies
- Depends on: US-003, US-004

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| skills/pc-evaluation/SKILL.md | Skill | Create |
| agents/pc-evaluator.md | Agent | Create |
| commands/implement.md | Command | Modify Phase 3 + Phase 7 |
| teams/implement-team.md | Team | Modify |
| skills/progress-tracking/SKILL.md | Skill | Modify |
| skills/tasks-generation/SKILL.md | Skill | Modify |
| CLAUDE.md | Docs | Modify glossary |
| CHANGELOG.md | Docs | Modify |
| docs/adr/ | ADR | Create evaluation architecture ADR |

## Constraints

- Pure command/skill/agent — NO Rust code
- tdd-executor NOT modified
- Evaluation is parent-owned
- Read-only agent (no Write/Edit)

## Non-Requirements

- Modifying tdd-executor behavior
- Running additional test suites
- Automated spec revision
- Measuring rework ratio improvement

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| None | None | Pure command/skill/agent |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| New skill | skills/ | pc-evaluation/SKILL.md | Create rubric |
| New agent | agents/ | pc-evaluator.md | Create agent |
| ADR | docs/adr/ | New ADR | Evaluation architecture |
| Command | commands/ | implement.md | Modify Phase 3+7 |
| Team | teams/ | implement-team.md | Add entry |
| Skills | skills/ | progress-tracking, tasks-generation | Add eval status |
| Glossary | CLAUDE.md | Glossary | Add self-evaluation |
| Entry | CHANGELOG.md | Unreleased | Add BL-149 |

## Open Questions

None.
