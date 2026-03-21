---
id: BL-033
title: Add /spec-quick for lightweight changes
status: open
created: 2026-03-21
scope: MEDIUM
target_command: /spec-quick (new command)
tags: [bmad, scale-adaptive, lightweight, quick]
---

## Optimized Prompt

Create `/spec-quick` — a lightweight spec command for small, well-understood changes. Skips: grill-me interview, adversarial review, web research, Plan Mode doc preview. Keeps: intent classification, basic scope confirmation via AskUserQuestion, minimal spec output (problem + AC list + affected files). Auto-triggered by /spec router when: description is < 50 words AND estimated file count < 3. User can always override to full /spec-dev. This addresses the BMAD principle of scale-adaptive planning depth — not every change needs the full pipeline.

## Framework Source

- **BMAD**: Level 0-1 = lightweight tech-spec, Level 2+ = full PRD + architecture + formal gates

## Related Backlog Items

- None
