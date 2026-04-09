# Spec: Claude Code Agent Teams API Assessment (BL-139)

## Problem Statement

Claude Code shipped Agent Teams (parallel persistent agents with shared task lists). ECC uses a fire-and-return subagent model via the Agent tool. Research shows Agent Teams is experimental (env flag required), has 7x token cost, CLI-only (no SDK API), and doesn't support nested teams — making it unsuitable for ECC's current architecture. A formal assessment document is needed to close this evaluation and define re-evaluation triggers.

## Research Summary

- Agent Teams is experimental, requires CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1
- No programmatic API — fully prompt-driven at runtime
- 7x token cost compared to single-session subagent dispatch
- No nested teams — blocks hierarchical orchestration (orchestrator → wave → tdd-executor)
- Teammates are persistent sessions (not fire-and-return) — fundamentally different model
- Hooks available: TeammateIdle, TaskCreated, TaskCompleted (exit 2 to control)
- Claude Agent SDK has agents option but no team-level API
- Claude Managed Agents (public beta April 2026) is a separate cloud-hosted layer

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Close as "resolved: wait" | API experimental, 7x cost, no nested teams | No |
| 2 | Write assessment document | Formal record for future reference | No |
| 3 | Define 5 GA trigger conditions | Clear re-evaluation criteria | No |

## User Stories

### US-001: Write Agent Teams API Assessment Document

**As a** ECC maintainer, **I want** a structured assessment comparing ECC dispatch to Agent Teams, **so that** the evaluation is documented and re-evaluation criteria are clear.

#### Acceptance Criteria

- AC-001.1: Assessment document at docs/audits/agent-teams-api-assessment-2026-04-09.md
- AC-001.2: Mapping table: 5 ECC dispatch surfaces to Agent Teams equivalents
- AC-001.3: Wave dispatch compatibility analysis section
- AC-001.4: tdd-executor isolation evaluation section
- AC-001.5: Decision gate with 5 observable GA trigger conditions
- AC-001.6: Verdict: wait (not adopt/skip)
- AC-001.7: BL-139 status updated to implemented in BACKLOG.md

#### Dependencies

- Depends on: none

### US-002: CHANGELOG entry

**As a** maintainer, **I want** the assessment recorded in CHANGELOG, **so that** the evaluation is discoverable.

#### Acceptance Criteria

- AC-002.1: CHANGELOG records the assessment and wait verdict

#### Dependencies

- Depends on: US-001

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| docs/audits/ | Docs | New assessment document |
| docs/backlog/BACKLOG.md | Docs | Status update |

## Constraints

- No code changes — documentation only
- Must not re-litigate ADR-0040 (team coordination)
- Channels explicitly out of scope

## Non-Requirements

- Not implementing any Agent Teams integration
- Not prototyping Agent Teams usage
- Not evaluating Claude Managed Agents

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| None | Doc-only | No E2E impact |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| CHANGELOG | root | CHANGELOG.md | Add BL-139 assessment entry |

## Open Questions

None.
