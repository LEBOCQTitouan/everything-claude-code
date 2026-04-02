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
| 4 | Tool permission validation (warn on escalation) | Prevents manifests from granting tools beyond what the referenced agent defines. Warnings exit 0; errors exit non-zero. | No |
| 5 | Team manifests installed to ~/.claude/teams/ (global) with project-local teams/ override | Matches existing agent/skill resolution: global install, project override | No |
| 6 | No schema version field in v1; breaking format changes handled by ecc install --upgrade (future) | Keep v1 simple; versioning is premature for the initial release | No |
| 7 | ECC_LEGACY_DISPATCH=1 fallback for commands during migration | Provides rollback path for the breaking change; deprecated after 2 releases | No |
| 8 | Coordination strategy is informational in v1 — commands read it as a hint, not as executable dispatch logic | Commands decide dispatch mode based on their own logic; strategy field documents intent | No |
| 9 | Role field is freeform descriptive text (no validation) | Roles are semantic labels for humans, not machine-enforced contracts | No |

## User Stories

### US-001: Team Manifest Format

**As a** developer configuring ECC agent teams, **I want** to define team composition in declarative Markdown manifests with YAML frontmatter, **so that** teams are reusable, versionable, and customizable without editing command files.

#### Acceptance Criteria

- AC-001.1: Given a teams/ directory, when a .md file with valid frontmatter (name, description, agents list, coordination strategy) is present, then ecc validate teams parses it successfully
- AC-001.2: Given a team manifest with agents: [{name, role, allowed-tools}], when parsed, then each agent entry has name (required), role (required), and allowed-tools (optional, defaults to agent's own tools)
- AC-001.3: Given a team manifest with coordination: wave-dispatch, when parsed, then the coordination strategy is recognized as one of: sequential, parallel, wave-dispatch
- AC-001.4: Given pre-defined team files (implement-team.md, audit-team.md, review-team.md), when installed via ecc install, then they are placed in ~/.claude/teams/ (global) and optionally overridden by project-local teams/
- AC-001.5: Given a team manifest with invalid YAML frontmatter (syntax error, missing closing ---), when ecc validate teams runs, then it reports a parse error including the file path
- AC-001.6: Given a team manifest with an empty agents: [] list, when validated, then it reports an error: "Team manifest must define at least one agent"

#### Dependencies

- Depends on: none

### US-002: Team Manifest Validation

**As a** developer, **I want** ecc validate teams to catch errors in team manifests before runtime, **so that** misconfigured teams fail fast with clear error messages.

#### Acceptance Criteria

- AC-002.1: Given a team manifest referencing an agent name not present in agents/, when ecc validate teams runs, then it reports an error: "Agent '<name>' not found in agents/"
- AC-002.2: Given a team manifest with an unknown coordination strategy, when validated, then it reports an error: "Unknown coordination strategy '<value>'"
- AC-002.3: Given a team manifest where an agent's allowed-tools includes tools not in the referenced agent's frontmatter tools list, when validated, then it reports a warning: "Tool '<tool>' in team manifest exceeds agent '<name>' allowed tools (privilege escalation)"
- AC-002.4: Given ecc validate teams with no teams/ directory, when run, then it reports "No teams directory found" and exits 0
- AC-002.5: Given a team manifest with duplicate agent names, when validated, then it reports an error: "Duplicate agent '<name>' in team manifest"
- AC-002.6: Given max-concurrent set to 0 or a negative number, when validated, then it reports an error: "max-concurrent must be >= 1"

#### Dependencies

- Depends on: US-001

### US-003: Shared State Protocol Skill

**As a** developer building agent teams, **I want** a codified shared state protocol, **so that** agents know exactly how to read/write shared state files (tasks.md, campaign.md, state.json).

#### Acceptance Criteria

- AC-003.1: [manual review] Given the shared-state-protocol skill, when loaded, then it contains sections: "## tasks.md Contract", "## campaign.md Contract", "## state.json Contract"
- AC-003.2: [manual review] Given the skill, then the tasks.md section documents format, status trail values (pending/red/green/done/failed), and claiming semantics
- AC-003.3: [manual review] Given the skill, then the state.json section documents it as read-only for non-orchestrator agents

#### Dependencies

- Depends on: none

### US-004: Task Handoff Skill

**As a** developer building sequential or fan-out teams, **I want** a codified task handoff protocol, **so that** agents can pass work between each other with self-contained context briefs.

#### Acceptance Criteria

- AC-004.1: [manual review] Given the task-handoff skill, when loaded, then it contains sections: "## Context Brief Format", "## Handoff Triggers", "## Patterns"
- AC-004.2: [manual review] Given the skill, then the Handoff Triggers section documents: phase completion, quality gate pass, task status change
- AC-004.3: [manual review] Given the skill, then the Patterns section includes sequential (A->B) and fan-out (A->[B,C,D]) examples with Mermaid diagrams

#### Dependencies

- Depends on: US-003

### US-005: Command Integration

**As a** developer using /implement or /audit-full, **I want** these commands to read team manifests for agent configuration, **so that** I can customize team composition without editing command files.

#### Acceptance Criteria

- AC-005.1: Given /implement is invoked and teams/implement-team.md exists, when Phase 3 (TDD Loop) starts, then the command reads the manifest's agents list and uses each entry's name and allowed-tools to construct subagent spawn parameters (replacing hard-coded agent names in the command Markdown)
- AC-005.2: Given /audit-full is invoked and teams/audit-team.md exists, when parallel dispatch starts, then the command reads the manifest's agents list for agent names and concurrency settings
- AC-005.3: Given a command is invoked and no team manifest exists AND ECC_LEGACY_DISPATCH is not set, when dispatch starts, then it fails with: "Team manifest required: teams/<command>-team.md not found. Set ECC_LEGACY_DISPATCH=1 for legacy behavior."
- AC-005.4: Given a team manifest with max-concurrent: N (where N >= 1), when wave dispatch runs, then the command passes N as the concurrency cap parameter (the wave algorithm itself is not modified)
- AC-005.5: Given a command invoked without a team manifest AND ECC_LEGACY_DISPATCH=1 is set, then the command falls back to hard-coded agent configuration with a deprecation warning: "DEPRECATED: Using legacy dispatch. Create teams/<command>-team.md to use team manifests."

#### Dependencies

- Depends on: US-001, US-002, US-006

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
| ecc-app/src/install/ | App | Modify: add teams/ to install artifact directories |
| commands/implement.md | Content | Modify: read team manifest in Phase 3, add ECC_LEGACY_DISPATCH fallback |
| commands/audit-full.md | Content | Modify: read team manifest in Phase 2, add ECC_LEGACY_DISPATCH fallback |

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
- **Not in scope**: Runtime enforcement of shared-state or handoff protocols (skills are documentation only)
- **Not in scope**: Coordination strategy as executable dispatch logic (v1 is informational; commands decide dispatch mode independently)

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
