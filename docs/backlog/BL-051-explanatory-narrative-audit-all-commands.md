---
id: BL-051
title: Explanatory narrative audit — all commands and workflows
status: open
created: 2026-03-22
promoted_to: ""
tags: [ux, commands, output, narrative, audit]
scope: HIGH
target_command: /spec refactor
---

## Optimized Prompt

Perform a full audit of every command and workflow in the ECC project to add
explanatory narrative, making Claude communicate its decisions and reasoning
to the user as it works.

**Context:** Currently, commands execute phases silently or with minimal status
output. Users cannot easily understand what choices Claude is making, why a
phase was triggered, or what happened at each step. This audit adds
explanatory narrative — not structural changes — to every command.

**Tech stack:** ECC project (Rust CLI + Markdown commands/agents/skills in
`.claude/` and `commands/`).

**Scope:** All files under `commands/` and any workflow orchestration files
under `.claude/workflow/`. Agent files (`agents/`) are out of scope unless
they are invoked as primary entry points.

**Workflow:**

```
/spec refactor "Explanatory narrative audit of all ECC commands"
```

Use the following as the refactor brief:

1. Inventory every command file in `commands/` and classify by workflow phase
   (spec, design, implement, audit, verify, utility).
2. For each command, identify gaps where Claude takes a non-obvious action
   without explaining why (e.g., choosing a scope level, skipping a phase,
   delegating to a subagent, failing a gate).
3. Add inline narrative instructions at each gap point so the command
   explicitly tells Claude to surface its reasoning to the user in plain
   language before acting. Examples:
   - "Before delegating to the planner agent, tell the user which subagent
     you are spawning and why."
   - "After classifying intent, state the detected intent and ask the user
     to confirm before proceeding."
   - "When a phase gate blocks progress, explain which gate failed and what
     the user must do to unblock it."
4. Narrative must be explanatory, not prescriptive — it should help the user
   understand what Claude is doing, not dictate UI copy verbatim.
5. Apply consistently: use a shared pattern so all commands narrate at the
   same level of detail (avoid some commands being verbose while others stay
   silent).

**Acceptance criteria:**
- Every command that executes 2+ distinct phases has at least one narrative
  instruction telling Claude to explain its decision before proceeding.
- Phase gate blocks always surface the reason and the remediation step.
- Subagent delegation always names the agent and the reason it was chosen.
- No structural command logic is changed — narrative additions only.
- A brief summary document (`docs/narrative-audit.md`) lists every command
  touched and the narrative points added.

**Scope boundaries — do NOT:**
- Refactor command logic, phase order, or tool selections.
- Change agent frontmatter or skill files.
- Add new commands or remove existing ones.
- Hardcode example output strings — instructions must be directive, not
  prescriptive copy.

**Verification steps:**
1. Read each modified command and confirm at least one narrative instruction
   was added per multi-phase command.
2. Run `/verify` to confirm no lint regressions on Markdown files.
3. Spot-check 3 commands manually: confirm the narrative instructions are
   clear, actionable, and consistent in tone.

## Original Input

"I want every commands and action to be explicited to the user so that he can
understand what was done and what choices claude makes"

"I want to make a full audit of every commands and workflow to have output as
relevant to the user as possible"

"explanatory narrative" (as opposed to restructuring/trimming output)

## Challenge Log

Q1: Which commands are you most frustrated with?
A1: Full audit of every command and workflow — not limited to specific ones.

Q2: Adding explanatory narrative or restructuring/trimming output?
A2: Explanatory narrative — making Claude explain its decisions and actions.

## Related Backlog Items

- BL-048 (implemented): Comprehensive output summaries for spec → design →
  implement pipeline. That entry added summary tables as artifacts; this entry
  adds live narrative during execution, and covers all commands, not just the
  main pipeline.
