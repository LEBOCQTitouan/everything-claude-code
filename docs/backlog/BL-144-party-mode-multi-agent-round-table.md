---
id: BL-144
title: /party command — BMAD-style multi-agent round-table with auto-generated domain agents
status: open
created: 2026-04-12
promoted_to: ""
tags: [commands, multi-agent, orchestration, agents, domain-generation, bmad]
scope: HIGH
target_command: /spec-dev
---

## Optimized Prompt

```
/spec-dev

Implement a new `/party` command for ECC that orchestrates a BMAD-style multi-agent
round-table discussion. The command assembles a panel of agents from three sources,
presents a selection UX, runs the session, and persists the output.

Tech stack: Rust (ecc-cli, ecc-app, ecc-infra), Markdown agent frontmatter schema,
existing `agents/` roster, Claude Code command frontmatter conventions.

---

### Context

ECC has a mature single-agent pipeline (/spec → /design → /implement). There is no
mechanism for convening multiple agents simultaneously to debate a topic, challenge
design decisions, or produce cross-perspective synthesis output — the BMAD "party"
pattern. This command fills that gap.

BL-143 (/project-foundation) produces PRD + Architecture docs. Those docs SHOULD feed
the agent-recommendation heuristics in /party so that recommended panels are
context-aware.

---

### Three Agent Sources

1. **BMAD role agents** — PM, Architect, Dev, QA, UX, Data Engineer, Security.
   If these don't exist as ECC agent files, they must be authored as part of this
   feature under `agents/bmad/` with correct frontmatter (name, description, tools,
   model, effort). These are generic cross-project roles.

2. **ECC roster agents** — all agents under `agents/` (arch-reviewer, code-reviewer,
   security-reviewer, uncle-bob, robert, spec-adversary, solution-adversary, etc.).
   Enumerate dynamically by reading the directory at command invocation time.

3. **Repo-domain agents** — project-local agents in `.claude/agents/` of the current
   repo. If none exist, the command triggers auto-generation (see below).

---

### Selection UX

When `/party <topic>` is invoked:

1. Enumerate all agents across the three sources.
2. Present two options via AskUserQuestion:
   - **Manual selection**: display full agent list (name + one-line description),
     user picks 2–8 agents.
   - **Claude-recommended panel**: Claude assembles a panel of 3–6 agents based on
     the topic, repo context (CLAUDE.md, docs/specs/, docs/backlog/BACKLOG.md), and
     optionally the BL-143 PRD/Architecture docs if they exist.
3. User confirms final panel before session starts.

---

### Repo-Domain Agent Auto-Generation

If `.claude/agents/` is empty or absent:
- Run `ecc analyze` (or invoke doc-analyzer) to scan codebase structure.
- Derive 1–3 domain agents from the detected bounded contexts / modules
  (e.g., `agents/domain-workflow-engine.md`, `agents/domain-infra-layer.md`).
- Persist generated agents to `.claude/agents/` of the current project.
- Generated agents MUST conform to ECC agent frontmatter schema.
- Auto-generation is non-blocking: if it fails, proceed with BMAD + ECC agents only,
  emit a warning.

---

### Session Execution

- Invoke selected agents sequentially (round-table order) or in parallel waves
  (group independent agents into the same wave).
- Each agent receives: the topic, repo context summary, and prior agents' outputs
  (threaded context).
- The session coordinator (a new `party-coordinator` agent) synthesizes outputs into
  a decisions log.

---

### Output Persistence

After the session:
- Write `docs/party/<slug>-<YYYYMMDD>.md` with:
  - Panel composition (agent list + source)
  - Topic
  - Per-agent output (verbatim or summarized, user-configurable)
  - Synthesis / decisions log from party-coordinator

---

### Acceptance Criteria

- [ ] `/party <topic>` command exists in `commands/party.md` with correct frontmatter.
- [ ] `agents/bmad/` directory contains at minimum: pm, architect, dev, qa, security
      role agents with valid ECC frontmatter.
- [ ] Command enumerates agents from all three sources dynamically.
- [ ] Manual selection and Claude-recommended panel paths both work.
- [ ] Recommended panel uses repo context (CLAUDE.md, backlog, specs) for heuristics;
      integrates BL-143 docs when available.
- [ ] Auto-generation of domain agents triggers when `.claude/agents/` is empty;
      persists valid agent files; degrades gracefully on failure.
- [ ] `party-coordinator` agent synthesizes round-table output into a decisions log.
- [ ] Session output persisted to `docs/party/<slug>-<YYYYMMDD>.md`.
- [ ] All generated/authored agents pass `ecc validate agents`.
- [ ] Command passes `ecc validate commands`.

---

### Scope Boundaries — NOT in scope

- Real-time streaming between agents (sequential or wave execution only).
- GUI or interactive TUI for agent selection.
- Merging party output back into specs automatically (separate entry).
- Modifying existing ECC agents to participate differently in party vs. solo mode.
- Cross-session party state persistence beyond the output document.

---

### Verification Steps

1. `ecc validate commands party` — passes.
2. `ecc validate agents` — all new BMAD agents pass.
3. `/party "review the auth module"` → presents agent list → manual selection works.
4. `/party "review the auth module"` → Claude-recommended path → 3–6 agents assembled.
5. Run in a repo with empty `.claude/agents/` → domain agents generated and persisted.
6. `docs/party/` output file created with correct structure after session.
7. `cargo test` — all existing tests pass (no regressions).
```

## Original Input

Party mode command — BMAD-style multi-agent round-table, includes BMAD agents + ECC
agents + repo-domain-specific agents; auto-scans codebase to generate domain agents if
none exist.

User answered Stage 1 Q1: user can pick (so all available agents must be presented in
a selectable list), but can also opt for a Claude-recommended panel. Key requirement:
command must (a) enumerate all agents (BMAD-style + ECC roster + repo-domain agents),
(b) offer manual selection, (c) offer "let Claude recommend" option that auto-assembles
based on context.

## Challenge Log

Mode: backlog-mode (standard profile)
Stages completed: 1/3 (user skipped remaining stages — enough signal)
Questions answered: 1/1

### Stage 1: Clarity

**Q1**: [Clarification] How does a user decide which agents join the party — fixed
roster, user-selected, or dynamic based on context?
**A**: User can pick (all available agents presented in a selectable list), but can
also opt for a Claude-recommended panel that auto-assembles based on context.
**Status**: answered

## Related Backlog Items

- BL-143: /project-foundation command — PRD + Architecture docs creation
  (HIGH — BL-143 docs feed agent-recommendation heuristics in /party)
- BL-137: Apply difficulty-aware model routing from multi-agent research
  (HIGH — model routing strategy applies to party-mode agent invocation)
