---
name: grill-me
description: Universal questioning protocol. Stage-by-stage structured interview across 5 canonical stages (Clarity, Assumptions, Edge Cases, Alternatives, Stress Test). Uses AskUserQuestion — one question per turn, never batched. Supports standalone mode (default), spec-mode, and backlog-mode.
origin: ECC
---

# Grill Me — Universal Stage-by-Stage Questioning Protocol

## When to Activate

- User says "grill me", "challenge my assumptions", "stress test this idea", "poke holes in this", "what am I missing"
- Invoked by `/spec`, `/spec-dev`, `/spec-fix`, `/spec-refactor` commands (spec-mode)
- Invoked by `/backlog` command (backlog-mode)
- User wants to validate a plan, design, or proposal before committing

---

## Modes

### Standalone Mode (Default)

Activated when invoked directly — without a mode parameter. All 5 stages, no recommended answers, no shortcuts. The default mode when the user says "grill me" or equivalent.

### Spec-Mode

Activated when invoked by `/spec` commands. Enables:
- Recommended answers as the first option in each AskUserQuestion (marked `(Recommended)`)
- `"spec it"` shortcut: user types "spec it" to accept all remaining recommended answers and skip to output
- All 5 stages active

### Backlog-Mode

Activated when invoked by `/backlog`. Enables:
- Max 3 stages: Clarity, Assumptions, Edge Cases (first 3 only)
- Max 2 questions per stage
- Claude can escalate to full 5 stages when scope is assessed as HIGH or EPIC

---

## 5 Canonical Stages

| # | Stage | Focus |
|---|-------|-------|
| 1 | Clarity | Pin down the problem statement — unambiguous, falsifiable |
| 2 | Assumptions | Surface hidden assumptions — what is taken for granted |
| 3 | Edge Cases | Boundary conditions, failure modes, degenerate inputs |
| 4 | Alternatives | Why this approach? What was rejected and why? |
| 5 | Stress Test | Push the proposal to breaking point — load, adversity, time |

---

## Question Statuses

Every question in the question list has one of 4 statuses:

- **pending** — not yet asked
- **open** — asked, awaiting answer
- **challenged** — answered, but a follow-up challenge is active
- **answered** — fully resolved, no active challenge

---

## Question Cap

Maximum 25 questions total across all stages. Once the cap is reached, no new questions are added (cross-stage mutations included). Claude MUST respect this cap.

---

## Questioning Protocol

### AskUserQuestion Enforcement

Use `AskUserQuestion` for every question — one question per turn, never batched. Wait for the user's answer before asking the next. If AskUserQuestion is unavailable, fall back to conversational questions (one at a time, wait for response).

One question per turn is absolute: never ask two questions in the same message, even if they seem related.

### Build Question List Upfront

Before asking any question, build the full question list grouped by stage. Show the list to the user at the start and after each answer.

Display format:

```
Stage 1: Clarity
  [1] What problem does this solve? — pending
  [2] Who is the primary user? — pending

Stage 2: Assumptions
  [3] What is assumed about the environment? — pending
```

### Stage Progression

When all questions in a stage are **answered** (status = answered), proceed to the next stage. Stage complete — proceed to the next stage only when every question in the current stage reaches answered status.

### Challenge Loop

After each answer, evaluate whether the answer is challengeable:
- Vague, hand-waving, or unsubstantiated answers → challengeable
- Clear, specific, measurable answers → not challengeable

If challengeable:
1. Add a follow-up challenge question under the same question (status: **challenged**)
2. Ask the follow-up via AskUserQuestion
3. Repeat for a maximum of **2 follow-ups** per question
4. After 2 follow-ups exhausted, OR when Claude judges the answer complete, mark **answered** and state termination reason

State termination reason explicitly: "Marking as answered — [reason: 2 follow-ups exhausted / answer is sufficiently specific]"

### Cross-Stage Mutation

Any answer can add questions to any stage, including already-completed stages. Add questions to any stage when the answer reveals a gap. Cross-stage mutations count against the 25-question cap.

### Stage Reopen Limit

A completed stage reopens exactly once for new questions added via cross-stage mutation. If further mutations would target an already-reopened stage, queue them as notes in the decision log instead.

---

## Skip and Exit

- User says "skip all" or `"done"` → end the interview immediately. All unanswered questions = status **skipped**.
- If more than 50% of questions are skipped → emit a degraded quality warning: "Warning: more than 50% of questions were skipped. Output quality is degraded — key areas remain unexamined."
- Individual "skip" on one question → mark that question **skipped**, continue.

---

## Decision Log Output

When the interview is complete (all stages done, or user exits), produce a **decision log**:

```markdown
# Decision Log: {topic}

Date: {date}
Mode: {standalone | spec-mode | backlog-mode}
Stages completed: {N}/5
Questions answered: {X}/{total}
Questions skipped: {Y}

## Questions, Answers, and Challenges

### Stage 1: Clarity
**Q1**: {question text}
**A**: {answer}
**Challenge**: {challenge text} (if any)
**A**: {challenge answer} (if any)
**Status**: answered / skipped

...

## Unresolved / Skipped Questions
{list}

## Notes from Cross-Stage Mutations
{list of notes for stages that hit the reopen limit}
```

---

## Negative Rules

- DO NOT answer your own questions
- DO NOT offer solutions or suggestions
- DO NOT validate prematurely
- DO NOT accept hand-waving — demand specifics
- DO NOT skip a stage unless the user explicitly requests it
- DO NOT batch multiple questions — one question per turn, one at a time
- DO NOT soften questions to be polite

---

## Adversary Mode

Say "adversary mode" or "hard mode" to activate `grill-me-adversary` — a companion skill that adds answer scoring, adaptive follow-up probing, and question-generation challenge. The five-stage flow stays the same; only question selection and answer evaluation change.
