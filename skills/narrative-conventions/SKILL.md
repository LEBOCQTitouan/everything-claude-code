---
name: narrative-conventions
description: Shared narration patterns for ECC commands — agent delegation, gate failure, progress, and result communication conventions.
origin: ECC
---

# Narrative Conventions

Standardized narration patterns that every ECC command follows when communicating with the user during execution.

## When to Apply

Reference this skill from any command file. These conventions apply to all ECC commands — pipeline commands, audit commands, and utility commands alike.

## Agent Delegation Narration

Before dispatching any agent, tell the user:

- **Which agent** is being launched and its role
- **What it analyzes** — the specific scope or input it receives
- **What output to expect** — the artifact or verdict it produces

This narration appears before the action it describes, so the user understands what is happening and why.

## Gate Failure Narration

When a gate blocks progress (adversarial review, build check, test suite, lint), explain:

- **What blocked** — the specific gate and its verdict
- **Why it matters** — the quality or safety concern the gate protects
- **Remediation steps** — concrete, actionable steps to resolve the failure

Never silently retry or skip a gate. The user must understand the failure before the command proceeds.

## Progress Narration

At every phase transition, tell the user:

- **What phase begins** — its name and position in the workflow
- **What it accomplishes** — the goal of this phase
- **What comes next** — the subsequent phase or completion state

Progress narration keeps the user oriented in multi-phase workflows.

## Result Narration

After receiving agent output or completing a phase, summarize key findings conversationally before incorporating them into structured output (tables, artifacts, summaries). Highlight what was found, what changed, and any items requiring attention.

## Tone

- Neutral technical register, active voice, present tense
- Instruct what to communicate, never dictate exact wording
- Narrative appears before the action it describes
- Keep narration concise — one to three sentences per narration point

## Anti-Patterns

- **DO NOT** dictate exact output text or templated sentences in command files
- **DO NOT** skip narration for "simple" phases — every phase transition gets narration
- **DO NOT** narrate after the fact — narration precedes the action
- **DO NOT** combine multiple narration types into a single block
