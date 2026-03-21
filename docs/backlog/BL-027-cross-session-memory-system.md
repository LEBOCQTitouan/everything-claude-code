---
id: BL-027
title: Cross-session memory system for actions, plans, and implementations
status: implemented
scope: HIGH
target: /plan dev
created: 2026-03-21
tags: [memory, hooks, agents, context, logging, markdown, json]
---

## Summary

Implement a deterministic, file-based memory system built into the ECC agent/config layer. It persists two categories of information across sessions: (1) a structured action log (JSON) capturing what was done and when, and (2) grouped Markdown files capturing plan, solution, and implementation artifacts per work item. Only high-value consumers (specific agents or commands) read from memory — it is never injected broadly to avoid context pollution.

## Optimized Prompt

```
Implement a cross-session memory system for the ECC Claude setup (agent/config layer — NOT a new CLI command).

## Context

ECC currently has no cross-session persistence. Each session starts from scratch — no record of what was done, what was planned, or what was implemented. The memory system must be file-based, deterministic in structure, and scoped so it does not pollute the general context or workflows.

## Memory Categories

### 1. Action Log (JSON)

File: `docs/memory/action-log.json`

Schema (append-only, one entry per action):
```json
{
  "timestamp": "ISO-8601",
  "session_id": "string",
  "action_type": "plan | solution | implement | verify | fix | audit | review | other",
  "description": "string",
  "artifacts": ["relative/path/to/artifact", "..."],
  "outcome": "success | partial | failed | skipped",
  "tags": ["string"]
}
```

Rules:
- Append-only — never mutate existing entries
- Written by hooks at the end of plan/solution/implement/verify phases
- No free-form fields — every field is typed and bounded
- `session_id` is a UUID generated once per Claude session and injected via a context hook

### 2. Grouped Work Item Files (Markdown)

Directory: `docs/memory/work-items/`

One subdirectory per work item, named `YYYY-MM-DD-<slug>/`:
- `plan.md` — output of /plan or /plan-* phase
- `solution.md` — output of /solution phase
- `implementation.md` — summary of /implement phase (key decisions, files changed, test results)

Rules:
- Deterministic file paths — path derived from date + slug (no UUIDs in paths)
- Deterministic internal structure — each Markdown file has fixed H2 sections (## Context, ## Decisions, ## Outcome, ## Artifacts)
- Slug is derived from the plan title, lowercased, hyphenated, max 40 chars
- Files are written once per phase and never overwritten (append a ## Revision block if corrections are needed)

## Consumer Access Rules

Memory is NOT injected into all agents. Only designated high-value consumers read it:
- `drift-checker` agent — reads action log to detect plan drift over time
- `catchup` command (BL-017) — reads action log + work items to reconstruct session context
- `robert` agent (BL-004) — reads past implementation summaries for negative examples
- Any future `retrospective` or `audit-evolution` consumer

All other agents and commands have NO access to memory files. Do not add memory injection to orchestrators, planners, or general hooks.

## Implementation Approach

1. **Storage writer hook** — A `PostToolUse` hook (or end-of-phase hook) writes to `action-log.json` after each major phase completes. Hook triggers on: plan complete, solution complete, implement complete, verify complete.

2. **Work item writer** — Integrated into the /plan-*, /solution, and /implement commands. After each phase outputs its result, a small write step appends the structured Markdown artifact to the appropriate work item file.

3. **Session ID injection** — A `PreToolUse` hook or context hook generates and injects a `SESSION_ID` env var at session start. All log entries reference it.

4. **Consumer read pattern** — Consuming agents/commands receive memory file paths via their system prompt context, not via auto-injection. They explicitly read what they need.

## Acceptance Criteria

- [ ] `docs/memory/action-log.json` is created on first use and grows append-only
- [ ] Each completed plan/solution/implement phase writes a corresponding Markdown file under `docs/memory/work-items/YYYY-MM-DD-<slug>/`
- [ ] File paths and internal Markdown structure are fully deterministic — same input always produces the same path and section layout
- [ ] No memory is injected into agents outside the designated consumer list
- [ ] `drift-checker` can read the action log without modification to its core logic (path passed via context)
- [ ] Running two concurrent sessions does not corrupt the log (append-only + session_id isolation)
- [ ] All new files follow ECC file naming conventions (lowercase, hyphens)

## Scope Boundaries — Do NOT

- Do not implement a new `ecc` CLI subcommand for memory (this is agent/config layer only)
- Do not inject memory into the general orchestrator or planner system prompts
- Do not add a vector store, embeddings, or semantic search — file-based only
- Do not store secrets or API keys in memory files
- Do not modify existing hook behavior beyond adding the new write step

## Verification Steps

1. Run `/plan dev` on a small task → confirm `action-log.json` gains one entry and `docs/memory/work-items/YYYY-MM-DD-<slug>/plan.md` is created with correct H2 sections
2. Run `/solution` → confirm `solution.md` appears in the same work item directory
3. Run `/implement` → confirm `implementation.md` appears with ## Artifacts listing changed files
4. Open `action-log.json` — verify all entries are valid JSON, append-only, no mutations
5. Confirm `drift-checker` agent reads the log correctly when given the path
6. Confirm no other agent system prompts reference memory paths
7. Run `/verify` — confirm build passes and no regressions
```

## Original Input

"I want to implement a memory system that is preserved between sessions. It should record all actions done as log but also all plan / solution / implementation. If there are any more to memorize make possible upgrades."

## Challenge Log

**Q1: Storage format?**
Best format for its use. Action logs should be JSON-like format. Plan/solution/implement should go under Markdown files. They should be grouped together.

**Q2: Who reads the memory?**
Only high-value consumers get to read memory — not to pollute context/workflows.

**Q3: CLI vs. convention?**
This is NOT a new CLI feature. It is a built-in feature of the ECC Claude setup (agent/config layer).

**Additional constraint stated by user:** Memory must be deterministic in how it works. Content should have a deterministic structure.

## Related Backlog Items

- BL-025 — Add memory:project to adversarial agents (narrower: per-agent memory flag, not cross-session log)
- BL-017 — Create /catchup command (designated consumer of this memory system)
- BL-004 — robert: read-only + memory + negative examples (designated consumer — reads past implementation summaries)
- BL-023 — Clean up stale workflow state (related: this memory system creates persistent state that also needs lifecycle management)
