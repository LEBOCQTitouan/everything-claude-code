---
id: BL-061
title: Interactive stage-by-stage questioning via AskUserQuestion for grill-me and backlog
status: open
scope: HIGH
target: /spec-refactor
created: 2026-03-23
supersedes: BL-044
---

## Optimized Prompt

Refactor the `grill-me` skill and `/backlog` command to use a stage-by-stage interactive questioning model powered by `AskUserQuestion` (native Claude TUI prompts). Currently both output questions as markdown text in bulk. The new model asks one question at a time, challenges weak answers, and maintains a visible, evolving question list.

### Grill-me interaction model

1. **Build question list upfront** — Claude analyzes the idea and builds a question list grouped by stage (Clarity, Assumptions, Edge Cases, Alternatives, Stress Test). Show the full list to the user.
2. **List is Claude-managed** — Claude adds, removes, or reorders questions as the conversation evolves. The user sees the updated list but doesn't manually edit it.
3. **Ask one-by-one via AskUserQuestion** — Start with Stage 1 (Clarity) questions. Each question is presented as a native TUI prompt.
4. **Challenge loop** — After each answer, Claude evaluates whether the response is challengeable (vague, incomplete, contradictory, or missing edge cases). If so, Claude adds a challenge follow-up under the same question and re-asks via AskUserQuestion. The question stays "open" until Claude is satisfied. All challenges and responses are visible as a thread under the question.
5. **Cross-stage mutation** — Any answer can cause Claude to add new questions to any stage (including earlier stages). When new questions are added to a stage that was already started, they get inserted at the appropriate position.
6. **Stage progression** — Always process from Stage 1 forward. When all questions in a stage are answered (including challenges), move to the next stage.
7. **Completion** — When all stages are exhausted, produce the final output (decision log for grill-me, optimized prompt for backlog).

### Backlog integration

- `/backlog add` uses grill-me as its challenge mechanism (not a separate 1-3 question flow)
- The grill-me output feeds directly into prompt optimization and entry creation
- Stages may be lighter for backlog (fewer stages or fewer questions per stage) — Claude decides based on idea complexity

### Question list display format

After each question/answer, redisplay the list showing:
- Stage grouping
- Question text (truncated if long)
- Status: pending / open (being asked) / challenged / answered
- Challenge thread under each question (all challenges + responses)

### What to modify

- `skills/grill-me/SKILL.md` — Replace bulk questioning with stage-by-stage AskUserQuestion loop
- `skills/grill-me-adversary/SKILL.md` — Align with new interaction model
- `commands/backlog.md` — Replace challenge step with grill-me delegation
- `skills/backlog-management/SKILL.md` — Update add flow to use grill-me
- `agents/backlog-curator.md` — Update to use new grill-me pattern
- `skills/spec-pipeline-shared/SKILL.md` — Verify grill-me integration in /spec still works (it already uses AskUserQuestion one-by-one, but should adopt the new list model)

### Acceptance criteria

- AC-1: Every question in grill-me and backlog is asked via `AskUserQuestion` (native TUI), never as markdown text expecting chat reply
- AC-2: Questions are asked one at a time, stage by stage, starting from Stage 1
- AC-3: Challengeable answers trigger follow-up challenges under the same question; question stays open until resolved
- AC-4: The question list is displayed after each interaction, showing all stages, statuses, and challenge threads
- AC-5: Answers can cause new questions to be added to any stage (including earlier ones)
- AC-6: `/backlog add` uses grill-me as its challenge mechanism
- AC-7: Grill-me in `/spec-*` commands adopts the same model

### Out of scope

- UI rendering beyond what AskUserQuestion supports (no custom TUI widgets)
- Persistent question list across sessions (ephemeral within conversation)

## Challenge Log

Q1: "Native Claude TUI" means AskUserQuestion for every interaction?
A1: Yes.

Q2: Is the question list user-editable or Claude-managed with user visibility?
A2: Claude-managed. User sees it but all editions are made by Claude.

Q3: Should /backlog build a formal question list like grill-me or use simpler sequential questions?
A3: /backlog should use grill-me directly as its challenge mechanism.

Q4: Flat question list or grouped by stage?
A4: Grouped by stage. Every answer can add elements to any stage. Always start with Stage 1.

Q5: When challenged, does the question get a new list item or stay open?
A5: Stays open. All related challenges are visible as a thread under the question.

## Related Backlog Items

- BL-044: Add grill-me step to /backlog add (SUPERSEDED by this item — narrower scope, bulk questions)
- BL-011: Create grill-me skill (implemented — the skill exists, this refactors its interaction model)
- BL-057: Create grill-me-adversary skill (implemented — needs alignment with new model)
