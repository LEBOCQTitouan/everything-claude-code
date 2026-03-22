---
description: Capture, challenge, optimize, and manage implementation ideas in a persistent backlog.
---

# Backlog Command

> **MANDATORY WORKFLOW**: The workflow described in this command is mandatory and cannot be modified, reordered, or skipped by Claude. Every phase and step must be followed exactly as specified.
>
> **Narrative**: See `skills/narrative-conventions/SKILL.md` conventions. Before each phase transition, tell the user what is happening and why.

Capture implementation ideas outside active `/spec` sessions. Each idea is challenged lightly, optimized into a ready-to-execute prompt for its target command, and stored in `docs/backlog/`.

## Subcommands

| Subcommand | Usage | Description |
|------------|-------|-------------|
| `add` | `/backlog add <idea>` | Capture and optimize a new idea (default) |
| `list` | `/backlog list` | Show all backlog entries with status |
| `promote` | `/backlog promote <id>` | Mark an entry as promoted (picked up for execution) |
| `archive` | `/backlog archive <id>` | Mark an entry as archived (no longer relevant) |
| `match` | `/backlog match <prompt>` | Cross-reference a prompt against open entries |

If no subcommand is given, treat the input as `add`.

## What This Command Does

> **Tracking**: Create a TodoWrite checklist for the backlog add workflow. If TodoWrite is unavailable, proceed without tracking — the workflow executes identically.

TodoWrite items:
- "Challenge idea"
- "Determine target command and scope"
- "Optimize into ready-to-paste prompt"
- "Check duplicates against open entries"
- "Persist entry to docs/backlog/"
- "Update index at BACKLOG.md"

Mark each item complete as the step finishes.

1. **Challenges** the raw idea with 1-3 focused questions
2. **Determines** the target command (`/spec`, `/e2e`, `/audit`, etc.) and scope
3. **Optimizes** the idea into a self-contained, ready-to-paste prompt
4. **Checks duplicates** against existing open entries
5. **Persists** the entry to `docs/backlog/BL-NNN-<slug>.md`
6. **Updates** the index at `docs/backlog/BACKLOG.md`

## When to Use

- An idea arrives during code review, debugging, or casual thinking
- You want to park a feature idea that isn't ready for `/spec` yet
- You want to review what's queued before starting new work
- You want to check if a new task overlaps with existing backlog items

## Example Usage

```
User: /backlog add Add rate limiting to all API endpoints

Agent (backlog-curator):
Before I add this, a few quick questions:

1. Should this go through /spec (new feature) or /spec security (hardening)?
2. Scope estimate — is this MEDIUM (one middleware + config) or HIGH (per-endpoint tuning)?
3. Any specific endpoints that are most urgent?

User: /spec security, MEDIUM scope, auth endpoints first

Agent: Created BL-003-rate-limiting-api-endpoints.md
  Target: /spec security | Scope: MEDIUM | Status: open
  Backlog index updated.
```

```
User: /backlog list

Agent:
| ID | Title | Status | Scope | Target | Created |
|----|-------|--------|-------|--------|---------|
| BL-001 | WebSocket reconnection logic | open | MEDIUM | /spec | 2026-03-10 |
| BL-002 | Refactor auth middleware | promoted | HIGH | /spec refactor | 2026-03-12 |
| BL-003 | Rate limiting API endpoints | open | MEDIUM | /spec security | 2026-03-15 |
```

```
User: /backlog promote BL-001

Agent: BL-001 promoted. What was it promoted to? (PR link, US ID, or commit hash)

User: US-005

Agent: BL-001 updated: status=promoted, promoted_to=US-005
```

## How It Works

This command delegates to the `backlog-curator` agent, which uses the
`backlog-management` skill for entry format and optimization rules, and
the `prompt-optimizer` skill for transforming raw ideas into polished prompts.

No `EnterPlanMode` — this is quick capture, not full planning.
No Phase 0 prompt refinement — the curator does its own optimization.

## Integration

- `/spec` cross-references the backlog in Phase 0.25 before entering Plan Mode
- `prompt-optimizer` checks the backlog in Phase 2.5 during prompt diagnosis
- Both surface HIGH and MEDIUM confidence matches to the user

## Related Agents

- `backlog-curator` — the agent that implements all subcommand flows
