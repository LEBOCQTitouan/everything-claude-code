---
id: BL-098
title: "Socratic grill-me upgrade — depth-first questioning with OARS, laddering, MECE, and reflective rephrasing"
scope: HIGH
target: "/spec-dev"
status: open
tags: [grill-me, socratic, questioning, OARS, laddering, MECE, depth]
created: 2026-03-29
related: [BL-011, BL-057, BL-061]
---

# BL-098: Socratic Grill-Me Upgrade

## Problem

The current grill-me skill produces shallow questioning:
- Questions don't drill deep enough — they stay surface-level instead of chaining into root causes
- Questions are sometimes unclear to the user — no comprehension check mechanism
- No reflective rephrasing — the LLM never restates the user's idea to confirm mutual understanding
- The 25-question cap artificially limits depth, forcing breadth over depth

## Proposed Solution

Upgrade grill-me with four SOTA techniques integrated into its stage-based protocol:

### 1. OARS Framework (Motivational Interviewing)
After every user answer, apply the OARS loop:
- **Open**: ask open-ended follow-ups that invite narrative
- **Affirm**: acknowledge the user's reasoning effort
- **Reflect**: restate what was heard to confirm understanding (mandatory after every answer)
- **Summarize**: consolidate themes at stage transitions

### 2. Laddering (Progressive Depth Drilling)
Replace the current 2-follow-up challenge loop with laddering chains:
- "Why is that important?" → build hierarchical tree of reasoning
- Continue drilling until the user reaches a concrete, falsifiable statement
- No fixed follow-up limit — depth is governed by answer specificity, not count

### 3. MECE Decomposition (McKinsey Issue Trees)
When exploring a requirement or design space:
- Partition the problem into mutually exclusive, collectively exhaustive sub-questions
- Ensure no overlap (no redundant questions) and no gap (nothing missed)
- Use binary splits to isolate root causes: "Is this driven by [A] or [B]?"

### 4. Socratic 6-Type Question Rotation
Cycle through the six Socratic question types across stages:
- Clarification: "What do you mean by X?"
- Assumption-probing: "Why do you assume Y?"
- Evidence-probing: "What data supports this?"
- Viewpoint: "How would [stakeholder] see this differently?"
- Implication: "What follows if we accept this?"
- Meta-questions: "Why is this question important?"

### Structural Changes
- **Remove the 25-question cap** — depth is unlimited, governed by stage completion
- **Reflective rephrasing after every answer** — LLM restates the user's response before asking the next question
- **Update grill-me-adversary** companion to use the same enhanced questioning patterns

## SOTA Sources

- OARS in LLM agents: arXiv:2407.08095 (Virtual Agents for Motivational Interviewing)
- Laddering: LadderBot (KIT, 2019) — automated requirements self-elicitation
- MECE: McKinsey/Bain/BCG issue tree methodology
- Socratic prompting: arXiv:2303.08769 (Prompting LLMs with the Socratic Method)
- SoDa framework: ACL 2025 Findings — multi-agent Socratic dialogue
- Reflective rephrasing: Modern Requirements (2025) — AI co-facilitator pattern

## Ready-to-Paste Prompt

```
/spec-dev Upgrade the grill-me skill (skills/grill-me/SKILL.md) and its companion
grill-me-adversary (skills/grill-me-adversary/SKILL.md) with four SOTA Socratic
questioning techniques:

1. OARS Framework — after every user answer, apply Open/Affirm/Reflect/Summarize.
   Reflect is mandatory (restate the user's response to confirm understanding).
   Summarize triggers at stage transitions.

2. Laddering — replace the fixed 2-follow-up challenge loop with progressive
   "why?" drilling that builds a reasoning tree. No fixed depth limit — continue
   until the answer is concrete and falsifiable.

3. MECE Decomposition — when exploring a requirement or design space, partition
   into mutually exclusive, collectively exhaustive sub-questions using binary
   splits. Ensure no overlap and no gaps.

4. Socratic 6-Type Rotation — cycle through clarification, assumption-probing,
   evidence-probing, viewpoint, implication, and meta-questions across stages.

Structural changes:
- Remove the 25-question cap entirely. Depth is governed by stage completion.
- Add mandatory reflective rephrasing after every answer (part of OARS Reflect).
- Update all three modes (standalone, spec-mode, backlog-mode) to use the
  enhanced techniques. Backlog-mode keeps its 3-stage/2-question-per-stage
  limits but applies OARS within those bounds.
- Update grill-me-adversary to leverage the new question types in its adaptive
  scoring and follow-up probing.

All consumers (/spec-*, /backlog, standalone) must benefit from the upgrade.
Preserve backward compatibility with existing modes and skip/exit behavior.
```

## Scope Justification

HIGH — touches:
- `skills/grill-me/SKILL.md` (core protocol rewrite)
- `skills/grill-me-adversary/SKILL.md` (companion update)
- All consumers: `/spec-dev`, `/spec-fix`, `/spec-refactor`, `/backlog`, standalone
- Removes a structural constraint (25-question cap)
- Introduces 4 new questioning frameworks that change interaction patterns
