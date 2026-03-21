---
id: BL-044
title: Add grill-me interview step to /backlog add workflow
status: open
created: 2026-03-21
promoted_to: ""
tags: [backlog, grill-me, curation, ux, workflow]
scope: LOW
target_command: direct edit
---

## Optimized Prompt

In the ECC project, the `/backlog add` workflow currently challenges the user with 1-3 lightweight questions via `AskUserQuestion` before persisting an entry. This is functional but informal — it lacks the structured adversarial framing that the `grill-me` skill brings (recommended answer per question, explicit rationale capture, decision log).

Add a formal grill-me step to the `/backlog add` workflow in `commands/backlog.md` and the `backlog-management` skill (`skills/backlog-management/skill.md`).

**Scope of change:**
- `commands/backlog.md`: insert a "Challenge" phase between idea intake and optimization that uses `AskUserQuestion` with recommended answers (matching grill-me style)
- `skills/backlog-management/skill.md`: update the `add` subcommand flow to document the grill-me pattern — each question must include a recommended answer, and answers are captured in the `## Challenge Log` section of the entry

**Acceptance criteria:**
- AC-1: When a user runs `/backlog add <idea>`, the curation flow asks 1-3 questions using `AskUserQuestion`, each with at least one option labeled "(Recommended)" based on codebase context
- AC-2: The final entry's `## Challenge Log` records each question, the recommended answer, and the user's actual answer
- AC-3: The grill-me step does not exceed 3 questions (lightweight-by-design constraint stays in place)
- AC-4: The entry format in `skills/backlog-management/skill.md` is updated to reflect the challenge log structure

**Out of scope:**
- Do NOT add adversarial review rounds (that is the spec-adversary pattern, not appropriate here)
- Do NOT change the entry YAML frontmatter schema
- Do NOT touch `/spec-*` grill-me flows

**Verification:**
1. Manually run `/backlog add` with a test idea and confirm recommended answers appear in `AskUserQuestion`
2. Inspect the created `.md` file and confirm `## Challenge Log` is populated with Q, recommended answer, and user answer
3. Confirm no more than 3 questions were asked

## Original Input

Add a "grill me" interview step to the `/backlog add` command workflow. Currently `/backlog add` challenges the idea with 1-3 questions but doesn't have a formal grill-me step like the spec-driven pipeline does. The user wants the backlog command to include a grill-me style challenge before persisting.

## Challenge Log

No questions asked — intent confirmed directly by user.

## Related Backlog Items

- BL-034: Capture grill-me decisions in work-item files (related context: both involve capturing grill-me Q&A, but BL-034 targets `/spec-*` work-item files, this targets `/backlog add`)
- BL-011: Create grill-me skill (implemented — the skill exists and can be referenced)
