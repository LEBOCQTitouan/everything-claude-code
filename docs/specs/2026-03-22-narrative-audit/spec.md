# Spec: Explanatory narrative audit — all commands and workflows (BL-051)

## Problem Statement

The 22 ECC command files execute phases silently or with minimal status output. Users cannot understand what choices Claude is making, why a phase was triggered, or what happened at each step. Of ~35 agent delegation points, 0 explain which agent is being launched or why. Of ~12 gate/blocking points, only ~3 provide remediation guidance. The implement command's multi-PC TDD loop — the longest automated sequence in the pipeline — runs with zero conversational narration between PCs. This refactoring adds explanatory narrative instructions throughout, establishing a consistent "narrate before acting" convention.

## Research Summary

- AI agent transparency best practices emphasize observable operations: what instructions were given, which tools used, what outcomes produced
- CLI UX research shows progress communication is the single highest-impact UX improvement for long-running processes
- Three progress patterns (spinner, X of Y, progress bar) apply — "X of Y" maps to PC tracking; spinners map to agent delegation
- Narrative should be explanatory (help user understand) not prescriptive (dictate exact UI copy)
- The "narrate before acting" pattern aligns with the ECC project's existing grill-me and Plan Mode transparency conventions

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Create shared narrative-conventions skill | Reusable patterns for delegation + gate narration across all commands | No |
| 2 | Inline narrative additions per command | Each command controls its own tone and detail level | No |
| 3 | 4 atomic groups for implementation | Matches co-change coupling: skill → spec trio → design+implement → audits+utilities | No |
| 4 | ADR for narrative convention | Establishes project-wide pattern that future commands must follow | Yes |
| 5 | Grep test suite + manual checklist | Automated regression protection + quality spot-checking | No |
| 6 | Narrative only — no logic changes | Preserves all existing behavior; additions are directive instructions only | No |
| 7 | Explanatory vs prescriptive boundary | Instruct Claude *what to communicate* (fact + reasoning), never *how to word it* | No |
| 8 | Tone: neutral technical, active voice, present tense | Consistent across all commands | No |

## User Stories

### US-001: Shared Narrative Conventions Skill

**As a** command author, **I want** a shared skill defining narrative patterns for agent delegation and gate failure communication, **so that** all commands narrate consistently.

#### Acceptance Criteria

- AC-001.1: Given `skills/narrative-conventions/SKILL.md` exists, when inspected, then it has frontmatter with `name: narrative-conventions`, `description`, `origin: ECC` and no `model` or `tools` fields.
- AC-001.2: Given the skill content, when read, then it defines patterns for: agent delegation narration (which agent, why, what to expect), gate failure narration (what blocked, why, how to fix), and progress narration (what phase, what's next).
- AC-001.3: Given the skill, when measured, then it is under 500 words.
- AC-001.4: Given the skill content, when read, then it specifies: narrative instructs Claude what to communicate (fact + reasoning), never how to word it; tone is neutral technical, active voice, present tense; narrative appears before the action it describes.

#### Dependencies

- Depends on: none

### US-002: Pipeline Command Narrative (spec-dev, spec-fix, spec-refactor)

**As a** developer running `/spec-*`, **I want** Claude to explain each research phase, agent delegation, and adversarial review result in plain language, **so that** I understand what is happening and why at every step.

#### Acceptance Criteria

- AC-002.1: Given any spec-* command, when Phase 1-2 launch agents, then Claude tells the user which agent is being launched and what it will analyze before dispatching.
- AC-002.2: Given any spec-* command, when Phase 3 (Web Research) begins, then Claude tells the user it is searching the web and what queries it will run.
- AC-002.3: Given any spec-* command, when the adversarial review returns a verdict, then Claude translates the dimensional findings into plain language before presenting to the user.
- AC-002.4: Given any spec-* command, when a gate blocks (state validation, adversarial FAIL), then Claude explains what blocked and provides specific remediation steps.
- AC-002.5: Given all 3 spec commands, when narrative is added, then the same patterns are used consistently (same tone, same level of detail).

#### Dependencies

- Depends on: US-001

### US-003: Design + Implement Command Narrative

**As a** developer running `/design` or `/implement`, **I want** Claude to explain design reviews, TDD loop progress, and code review findings in plain language, **so that** I can follow the implementation without scrolling through conversation history.

#### Acceptance Criteria

- AC-003.1: Given `/design`, when Phases 2-4 (SOLID, Robert, Security) launch validation agents, then Claude tells the user which validation is running and what it checks.
- AC-003.2: Given `/design`, when Phase 7 (AC Coverage) completes, then Claude reports the coverage result conversationally before proceeding.
- AC-003.3: Given `/implement`, when each PC subagent is dispatched in the TDD loop, then Claude tells the user: which PC, what it implements, which AC it covers, and what to expect.
- AC-003.4: Given `/implement`, when regression verification runs after each PC, then Claude reports: how many prior PCs were re-verified and the result.
- AC-003.5: Given `/implement`, when code review findings are addressed, then Claude tells the user what was found and what was fixed.
- AC-003.6: Given `/implement` or `/design`, when a gate fails (state validation, regression), then Claude explains what blocked and suggests specific remediation.

#### Dependencies

- Depends on: US-001

### US-004: Audit + Utility Command Narrative

**As a** developer running an audit or utility command, **I want** Claude to explain which analysis agents are running and what each report section reveals, **so that** I understand the audit process and can act on findings.

#### Acceptance Criteria

- AC-004.1: Given any audit command, when the analysis phase launches agents, then Claude tells the user which domain is being analyzed and what the agent looks for.
- AC-004.2: Given `/audit-full`, when parallel domain agents complete, then Claude reports per-domain completion status as each finishes.
- AC-004.3: Given any audit command, when the report ends with "To act on findings, run /spec", then Claude explains how to reference the audit report in the /spec command.
- AC-004.4: Given `/verify`, when agents are invoked (code-reviewer, arch-reviewer), then Claude explains what each reviewer checks and why both are needed.
- AC-004.5: Given `/build-fix`, when errors are classified (Structural/Contractual/Incidental), then Claude explains the classification to the user before acting.
- AC-004.6: Given `/review`, when robert agent is invoked, then Claude explains what the Programmer's Oath evaluation means and what it checks.
- AC-004.7: Given `/catchup`, when stale workflow is detected, then Claude explains consequences of resetting before offering the option.

#### Dependencies

- Depends on: US-001

### US-005: Documentation

**As a** developer, **I want** the narrative convention documented, **so that** future command authors know to include narrative instructions.

#### Acceptance Criteria

- AC-005.1: Given `docs/adr/0011-command-narrative-convention.md` exists, when read, then it documents the narrative convention with Status/Context/Decision/Consequences.
- AC-005.2: Given `CHANGELOG.md`, when read, then it includes a BL-051 entry.
- AC-005.3: Given `docs/narrative-audit.md` exists, when read, then it lists every command touched and the narrative points added.

#### Dependencies

- Depends on: US-001, US-002, US-003, US-004

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| skills/narrative-conventions/SKILL.md | Content (skill) | New — shared narrative patterns |
| commands/spec-dev.md | Content (command) | Modify — add narrative at 5+ points |
| commands/spec-fix.md | Content (command) | Modify — add narrative at 5+ points |
| commands/spec-refactor.md | Content (command) | Modify — add narrative at 5+ points |
| commands/design.md | Content (command) | Modify — add narrative at 4+ points |
| commands/implement.md | Content (command) | Modify — add narrative at 6+ points |
| commands/audit-full.md | Content (command) | Modify — add narrative at 3+ points |
| 9 × commands/audit-*.md | Content (command) | Modify — add narrative at 2+ points each |
| commands/verify.md | Content (command) | Modify — add narrative at 3+ points |
| commands/build-fix.md | Content (command) | Modify — add narrative at 2+ points |
| commands/review.md | Content (command) | Modify — add narrative at 1+ points |
| commands/catchup.md | Content (command) | Modify — add narrative at 1 point |
| commands/backlog.md | Content (command) | Modify — add narrative at 1+ points |
| commands/spec.md | Content (command) | Modify — add narrative at 1 point |
| commands/ecc-test-mode.md | Content (command) | Modify — add narrative at 1 point |
| tests/test-narrative-audit.sh | Content (test) | New — bash grep test suite |
| docs/adr/0011-command-narrative-convention.md | Documentation | New — ADR |
| docs/narrative-audit.md | Documentation | New — audit summary |
| CHANGELOG.md | Documentation | Modify — BL-051 entry |

No Rust code changes. No agent/hook/skill frontmatter changes. Pure command content additions.

## Constraints

- All changes are narrative instruction additions only — no phase logic, tool selection, or agent configuration changes
- Each command file must stay under 800 lines after additions
- Narrative instructs Claude *what to communicate*, never *how to word it*
- Tone: neutral technical, active voice, present tense
- Narrative appears before the action it describes, not after
- Shared skill must be under 500 words
- Consistent narration level across all commands
- Manual verification checklist with 3 criteria: (a) narrative before action, (b) active voice naming agent/gate, (c) no duplication
- Existing partial narration is augmented for consistency, not rewritten

## Non-Requirements

- No structural command changes (phase order, tool lists, agent selection)
- No agent frontmatter modifications
- No hook or workflow state changes
- No Rust code changes
- No naming convention documentation (deferred — LOW severity smell)

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| None | — | No E2E boundaries crossed — pure command file content changes |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| Convention | architecture | docs/adr/ | ADR 0011 for command narrative convention |
| Feature | project | CHANGELOG.md | Add BL-051 entry |
| Audit | project | docs/narrative-audit.md | New — summary of all commands touched and narrative points added |

## Atomic Group Membership

- **Group 1**: `skills/narrative-conventions/SKILL.md` (new)
- **Group 2**: `commands/spec-dev.md`, `commands/spec-fix.md`, `commands/spec-refactor.md`
- **Group 3**: `commands/design.md`, `commands/implement.md`
- **Group 4**: `commands/audit-full.md`, 9× `commands/audit-*.md`, `commands/verify.md`, `commands/build-fix.md`, `commands/review.md`, `commands/catchup.md`, `commands/backlog.md`, `commands/spec.md`, `commands/ecc-test-mode.md`

Rollback ordering: group 4 → group 3 → group 2 → group 1 (skill last).

## Open Questions

None — all resolved during grill-me interview and adversarial review.
