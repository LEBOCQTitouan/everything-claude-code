# Spec: Multi-Agent Team Coordination (BL-104)

## Problem Statement

ECC supports parallel subagents (wave dispatch, audit orchestration) but lacks a declarative, reusable way to define agent teams. Team composition is hard-coded in command files. Coordination patterns (shared state via tasks.md, handoff via campaign.md) exist informally but are not codified as reusable contracts. Claude Code's native Agent Teams feature (Feb 2026) provides runtime primitives (shared task lists, mailbox messaging, teammate hooks) but ECC does not leverage them.

## Research Summary

- **Claude Code Agent Teams**: Native experimental feature with shared task lists, file-locking-based claiming, mailbox/messaging between teammates, and hooks (TeammateIdle, TaskCreated, TaskCompleted). Teams stored at ~/.claude/teams/{name}/config.json.
- **Orchestration patterns**: Sequential pipeline, concurrent fan-out, structured handoff, supervisor-worker hierarchy. Microsoft Azure and OpenAI Agents SDK codify these as first-class patterns.
- **ALMAS framework**: ACM TOSEM survey confirms viability of role-aligned agent teams with Sprint Agent decomposition, Supervisor monitoring, and context-aware cost routing (matching ECC's Haiku/Sonnet/Opus model routing).
- **Practical sweet spot**: 3-5 teammates with 5-6 tasks each. File ownership must be partitioned strictly to avoid conflicts.
- **Pitfalls**: File conflicts from concurrent edits, stale task status, lead premature shutdown, context loss on handoff, no nested teams.
- **AutoAgents (Rust)**: Type-safe multi-agent framework with MCP support; 25% lower latency than Python alternatives. Worth evaluating for patterns.

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Content-layer team coordination over Rust execution engine | Team coordination is a Claude Code runtime concern; ECC provides declarative manifests and validation, not a new orchestrator | Yes |
| 2 | Strict agent cross-referencing (fail on missing) | Prevents silent failures when manifests reference non-existent agents | No |
| 3 | Commands require team manifests | Makes team composition explicit and configurable rather than hard-coded | No |
| 4 | Tool permission validation (warn on escalation) | Prevents manifests from granting tools beyond what the referenced agent defines | No |

## User Stories

### US-001: Team Manifest Format

**As a** developer configuring ECC agent teams, **I want** to define team composition in declarative Markdown manifests with YAML frontmatter, **so that** teams are reusable, versionable, and customizable without editing command files.

#### Acceptance Criteria

- AC-001.1: Given a teams/ directory, when a .md file with valid frontmatter (name, description, agents list, coordination strategy) is present, then ecc validate teams parses it successfully
- AC-001.2: Given a team manifest with agents: [{name, role, allowed-tools}], when parsed, then each agent entry has name (required), role (required), and allowed-tools (optional, defaults to agent's own tools)
- AC-001.3: Given a team manifest with coordination: wave-dispatch, when parsed, then the coordination strategy is recognized as one of: sequential, parallel, wave-dispatch
- AC-001.4: Given pre-defined team files (implement-team.md, audit-team.md, review-team.md), when installed via ecc install, then they are placed in teams/ alongside agents/ and skills/

#### Dependencies

- Depends on: none

### US-002: Team Manifest Validation

**As a** developer, **I want** ecc validate teams to catch errors in team manifests before runtime, **so that** misconfigured teams fail fast with clear error messages.

#### Acceptance Criteria

- AC-002.1: Given a team manifest referencing an agent name not present in agents/, when ecc validate teams runs, then it reports an error: "Agent '<name>' not found in agents/"
- AC-002.2: Given a team manifest with an unknown coordination strategy, when validated, then it reports an error: "Unknown coordination strategy '<value>'"
- AC-002.3: Given a team manifest where an agent's allowed-tools includes tools not in the referenced agent's frontmatter tools list, when validated, then it reports a warning: "Tool '<tool>' in team manifest exceeds agent '<name>' allowed tools (privilege escalation)"
- AC-002.4: Given ecc validate teams with no teams/ directory, when run, then it reports "No teams directory found" and exits 0

#### Dependencies

- Depends on: US-001

### US-003: Shared State Protocol Skill

**As a** developer building agent teams, **I want** a codified shared state protocol, **so that** agents know exactly how to read/write shared state files (tasks.md, campaign.md, state.json).

#### Acceptance Criteria

- AC-003.1: Given the shared-state-protocol skill, when loaded, then it documents the read/write contract for tasks.md (format, status trail, claiming)
- AC-003.2: Given the skill, when loaded, then it documents the campaign.md artifact exchange format
- AC-003.3: Given the skill, when loaded, then it documents state.json as read-only for non-orchestrator agents

#### Dependencies

- Depends on: none

### US-004: Task Handoff Skill

**As a** developer building sequential or fan-out teams, **I want** a codified task handoff protocol, **so that** agents can pass work between each other with self-contained context briefs.

#### Acceptance Criteria

- AC-004.1: Given the task-handoff skill, when loaded, then it documents the context brief format (PC spec, files to modify, prior results, commit rules)
- AC-004.2: Given the skill, when loaded, then it documents handoff triggers (phase completion, quality gate pass, task status change)
- AC-004.3: Given the skill, when loaded, then it documents sequential (A->B) and fan-out (A->[B,C,D]) patterns with examples

#### Dependencies

- Depends on: US-003

### US-005: Command Integration

**As a** developer using /implement or /audit-full, **I want** these commands to read team manifests for agent configuration, **so that** I can customize team composition without editing command files.

#### Acceptance Criteria

- AC-005.1: Given /implement is invoked and teams/implement-team.md exists, when Phase 3 (TDD Loop) starts, then agents are configured from the manifest instead of hard-coded defaults
- AC-005.2: Given /audit-full is invoked and teams/audit-team.md exists, when parallel dispatch starts, then agents are configured from the manifest
- AC-005.3: Given a command is invoked and no team manifest exists, when dispatch starts, then it fails with: "Team manifest required: teams/<command>-team.md not found"
- AC-005.4: Given a team manifest with max-concurrent: N, when wave dispatch runs, then concurrency is capped at N (overriding the default 4)

#### Dependencies

- Depends on: US-001, US-002

### US-006: Pre-defined Team Templates

**As a** developer setting up ECC, **I want** pre-defined team templates installed by default, **so that** /implement and /audit-full work out of the box with sensible defaults.

#### Acceptance Criteria

- AC-006.1: Given ecc install, when teams are installed, then teams/implement-team.md, teams/audit-team.md, and teams/review-team.md are created
- AC-006.2: Given the implement-team.md template, then it defines: tdd-executor (implementer), code-reviewer (reviewer), module-summary-updater (documenter), diagram-updater (documenter) with coordination: wave-dispatch
- AC-006.3: Given the audit-team.md template, then it defines domain-specific audit agents with coordination: parallel
- AC-006.4: Given the review-team.md template, then it defines code-reviewer + security-reviewer + arch-reviewer with coordination: sequential

#### Dependencies

- Depends on: US-001

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| ecc-domain/src/config/team.rs | Domain | New: parse and validate team manifest frontmatter (pure) |
| ecc-app/src/validate/teams.rs | App | New: validate team manifests via FileSystem port |
| ecc-app/src/validate/mod.rs | App | Modify: add Teams variant to ValidateTarget |
| ecc-cli/src/commands/validate.rs | CLI | Modify: wire "teams" argument |
| teams/implement-team.md | Content | New: pre-defined implement team manifest |
| teams/audit-team.md | Content | New: pre-defined audit team manifest |
| teams/review-team.md | Content | New: pre-defined review team manifest |
| skills/shared-state-protocol/SKILL.md | Content | New: shared state contract documentation |
| skills/task-handoff/SKILL.md | Content | New: handoff protocol documentation |
| commands/implement.md | Content | Modify: read team manifest in Phase 3 |
| commands/audit-full.md | Content | Modify: read team manifest in Phase 2 |

## Constraints

- Team manifests follow the same Markdown + YAML frontmatter pattern as agents/skills/commands
- Validation uses existing extract_frontmatter utility from ecc-domain::config::validate
- No new port traits needed -- uses existing FileSystem and TerminalIO
- Must not modify the wave dispatch algorithm itself -- teams consume it, they don't modify it
- Commands require team manifests (breaking change per user decision)

## Non-Requirements

- **Not in scope**: Generic team execution engine (team coordination is Claude Code runtime)
- **Not in scope**: Claude Code Agent Teams native integration (leveraged indirectly via skills, not wired in Rust)
- **Not in scope**: Nested teams (teammates cannot spawn sub-teams per Claude Code limitation)
- **Not in scope**: Real-time progress dashboard (progress tracked via existing tasks.md + campaign.md)

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| FileSystem (teams/ directory) | Read | E2E tests need team manifest fixtures |
| TerminalIO (validation output) | Write | E2E tests verify validation error messages |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| New concept | CLAUDE.md | Architecture section | Add teams/ to artifact directory list |
| New concept | docs/domain/bounded-contexts.md | Glossary | Add 4 terms |
| ADR | docs/adr/ | New file | Content-layer team coordination |
| Changelog | CHANGELOG.md | Unreleased | Add team coordination entry |

## Open Questions

None -- all resolved in grill-me interview.
