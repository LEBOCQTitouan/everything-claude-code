# Spec: /party Command — Multi-Agent Round-Table Discussion

Source: BL-144 | Scope: HIGH | Worktree: ecc-bl144-party-command

## Problem Statement

ECC has a mature single-agent pipeline (/spec → /design → /implement) but no mechanism for convening multiple agents to debate a topic, challenge design decisions, or produce cross-perspective synthesis. The BMAD "party" pattern fills this gap — cross-functional round-table analysis in a single command.

## Research Summary

- **BMAD Party Mode** (v6.3.0) simplified from multi-file to single SKILL.md — right-sized abstraction
- **Round-table debate pattern**: advocate agents argue, leader synthesizes. Sequential, shared context.
- **Key pitfall: context window bloat** — grows multiplicatively. Cap at 8 + sequential-only mitigates.
- **Synthesis is critical** — without explicit arbitration, output is opinions not decisions
- **Simplicity wins** — BMAD's trajectory and 2026 consensus: minimal orchestration layer
- **Subagents sufficient** for v1 — full team peer-to-peer deferred

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Content-only: zero Rust changes | Markdown files only. Agent tool handles dispatch. | Yes |
| 2 | BMAD agents flat with `bmad-` prefix | Validator reads flat directory only | Yes |
| 3 | Sequential-only for v1 | Each agent sees prior outputs. Parallel deferred. | Yes |
| 4 | Ephemeral panels, not persisted teams | Ad-hoc per-invocation. Different concept than `teams/`. | Yes |
| 5 | Auto-generation best-effort | Fails gracefully. Not offered if agents exist. | No |
| 6 | 2-8 manual, 3-6 recommended | Range prevents bloat | No |
| 7 | Party-coordinator opus/max | Synthesis needs highest reasoning | No |
| 8 | Output to `docs/party/<slug>-<YYYYMMDD>.md` | Collision: `-N` suffix | No |
| 9 | Slug derivation: lowercase, non-alphanumeric → hyphens, collapsed, max 50 chars | Deterministic filename generation | No |
| 10 | BMAD panelists: sonnet/medium | Analysis contributors, not synthesis leads. Opus reserved for coordinator. | No |
| 11 | BMAD agent tools: [Read, Grep, Glob] in frontmatter | Aligned with dispatch tools in AC-005.2. Read-only advisors. | No |

## User Stories

### US-001: BMAD Role Agents

**As an** ECC user, **I want** BMAD-style role agents, **so that** I can assemble cross-functional panels.

- AC-001.1: `agents/` contains `bmad-pm.md`, `bmad-architect.md`, `bmad-dev.md`, `bmad-qa.md`, `bmad-security.md`
- AC-001.2: All BMAD agents have valid frontmatter (name, description, tools, model, effort)
- AC-001.3: `ecc validate agents` passes for all BMAD agents
- AC-001.4: `ecc validate conventions` passes for naming consistency
- AC-001.5: Each BMAD agent's system prompt specifies: role name, expertise domain, output format, and at least 2 topic areas it addresses

**Dependencies:** none

### US-002: /party Command Skeleton

**As an** ECC user, **I want** `/party <topic>`, **so that** I can invoke multi-agent discussions.

- AC-002.1: `/party <topic>` recognized and executes
- AC-002.2: `commands/party.md` has valid frontmatter (description, allowed-tools)
- AC-002.3: `ecc validate commands` passes
- AC-002.4: No arguments → prompts for topic
- AC-002.5: With topic → proceeds to enumeration
- AC-002.6: Empty, whitespace-only, or absent topic after trimming → re-prompts user. Topic truncated to 200 chars for display. Slug derivation: lowercase, non-alphanumeric → hyphens, consecutive hyphens collapsed, max 50 chars, leading/trailing hyphens stripped. Fallback slug: `party-session`.

**Dependencies:** none

### US-003: Agent Enumeration and Selection UX

**As an** ECC user, **I want** to pick agents or accept a recommended panel.

- AC-003.1: Reads agents from `agents/bmad-*.md`, all `agents/*.md`, `.claude/agents/*.md`
- AC-003.2: Grouped by source: "BMAD Roles" = `agents/bmad-*.md`; "ECC Specialists" = `agents/*.md` excluding `bmad-*` and `party-coordinator`; "Project Domain" = `.claude/agents/*.md`
- AC-003.3: Manual selection via AskUserQuestion, 2-8 agents
- AC-003.4: Claude-recommended: 3-6 agents. Recommendation includes one-line rationale per agent citing which project context (CLAUDE.md, backlog, foundation docs) informed the choice. Panel presented for user confirmation.
- AC-003.5: <2 or >8 → re-prompts
- AC-003.8: Duplicate agent selections deduplicated with warning. If fewer than 2 agents available across all sources, command exits with error explaining minimum requirement.
- AC-003.6: `docs/foundation/` absent → uses CLAUDE.md + backlog only
- AC-003.7: User confirms panel before session starts

**Dependencies:** US-001, US-002

### US-004: Repo-Domain Agent Auto-Generation

**As an** ECC user with no `.claude/agents/`, **I want** auto-generated domain agents.

- AC-004.1: `.claude/agents/` absent or empty → offers auto-generation via AskUserQuestion
- AC-004.2: Uses doc-analyzer to scan codebase
- AC-004.3: 1-3 agents persisted with valid frontmatter
- AC-004.4: Failure → proceeds with BMAD + ECC agents + warning
- AC-004.5: `.claude/agents/` has files → auto-gen NOT offered

**Dependencies:** US-003

### US-005: Party Coordinator Agent

**As an** ECC user, **I want** a coordinator that synthesizes outputs into decisions.

- AC-005.1: `agents/party-coordinator.md` valid frontmatter: tools [Read, Grep, Glob, Agent, Write], model opus, effort max
- AC-005.2: Dispatches panelists via Agent tool with tools [Read, Grep, Glob]
- AC-005.3: Sequential: each agent receives topic + context + prior outputs
- AC-005.4: Synthesis output contains exactly 5 sections with these headings: "Per-Agent Summary", "Agreements", "Disagreements", "Recommendations", "Open Questions". Empty sections display "None identified." rather than being omitted.
- AC-005.5: `ecc validate agents` passes
- AC-005.6: Panelist failure → logs, continues with remaining, notes gap in synthesis
- AC-005.7: If ALL panelists fail, coordinator outputs an error report with failure details instead of synthesizing empty inputs

**Dependencies:** US-001

### US-006: Session Output Persistence

**As an** ECC user, **I want** output saved to `docs/party/`.

- AC-006.1: File at `docs/party/<slug>-<YYYYMMDD>.md`
- AC-006.2: Contains: YAML frontmatter, Panel Composition, Per-Agent Output, Synthesis, Open Questions
- AC-006.3: `docs/party/` created if absent
- AC-006.4: Collision → `-N` suffix
- AC-006.5: Write failure → output in conversation as fallback

**Dependencies:** US-005

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `agents/bmad-*.md` (5 new) | content | BMAD role agents |
| `agents/party-coordinator.md` (new) | content | Orchestrator |
| `commands/party.md` (new) | content | /party command |
| `docs/party/` (runtime) | content | Session output |

No Rust changes. No port/adapter changes.

## Constraints

- Zero Rust changes
- BMAD agents flat with `bmad-` prefix
- Sequential-only v1
- Panel cap: 8 manual, 6 recommended
- Auto-generation best-effort

## Non-Requirements

- Real-time streaming
- GUI/TUI
- Spec integration (BL-145)
- Agent dual-mode
- Cross-session state
- `agents/bmad/` subdirectory

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| None | Purely additive | No existing boundaries change |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| New command | Project | CLAUDE.md | Add /party to CLI table |
| ADR | Architecture | `docs/adr/0061-party-command.md` | 8 decisions |
| Glossary | Project | CLAUDE.md | 3 terms |
| CHANGELOG | Project | CHANGELOG.md | feat entry |

## Open Questions

None.
