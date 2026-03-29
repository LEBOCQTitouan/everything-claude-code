---
name: grill-me
description: Universal questioning protocol with Socratic techniques (OARS, Laddering, MECE, 6-Type Rotation) and depth profiles (shallow/standard/deep). Stage-by-stage structured interview across 5 canonical stages. Uses AskUserQuestion — one question per turn, never batched. Supports standalone mode (default), spec-mode, and backlog-mode.
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

Activated when invoked directly — without a mode parameter. All 5 stages, no recommended answers, no shortcuts. The default mode when the user says "grill me" or equivalent. Depth profile default: **deep**.

### Spec-Mode

Activated when invoked by `/spec` commands. Enables:
- Recommended answers as the first option in each AskUserQuestion (marked `(Recommended)`)
- `"spec it"` shortcut: user types "spec it" to accept all remaining recommended answers and skip to output
- All 5 stages active
- Depth profile default: **deep**

### Backlog-Mode

Activated when invoked by `/backlog`. Enables:
- Max 3 stages: Clarity, Assumptions, Edge Cases (first 3 only)
- Max 2 questions per stage
- Claude can escalate to full 5 stages when scope is assessed as HIGH or EPIC
- Depth profile default: **standard**

### Profile Override

Consuming commands can override the mode default by passing `profile=shallow|standard|deep`. The override takes precedence over the mode default. Mode stage/question limits take precedence over profile technique intensity.

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

## OARS Protocol

Apply OARS after every user answer:

- **Open**: Ask open-ended follow-ups that invite narrative, not yes/no answers
- **Acknowledge**: Factual recognition of what the answer addressed ("That addresses the concurrency concern"). Never use praise — factual recognition only, never empty validation
- **Reflect**: MANDATORY — restate what was heard to confirm understanding before proceeding. Example: "So what I'm hearing is: [restatement]. Is that correct?" Reflect fires before challenge evaluation and before adversary scoring when adversary mode is active
- **Summarize**: At every stage transition, consolidate themes from the completed stage before moving to the next

OARS applies regardless of depth profile, though `shallow` profile uses Reflect only (no Open follow-ups that trigger laddering, Summarize still fires at stage transitions).

---

## Laddering

Progressive depth drilling for abstract or vague answers.

**When to ladder**: An answer lacks concrete nouns, measurable quantities, or falsifiable claims → trigger laddering with "Why is that important?" or "What specifically do you mean by [term]?"

**Depth control**:
- No fixed depth limit — stop when the answer is concrete and falsifiable
- Safety valve: max 7 ladder levels on a single question. After depth 7, note "depth limit reached — moving to next question" and proceed
- User can exit the ladder chain via skip/exit (behavior preserved from `## Skip and Exit`)

**Profile interaction**:
- `shallow`: no laddering
- `standard`: 1-2 ladder levels maximum
- `deep`: unlimited laddering up to the safety valve (7 levels)

---

## MECE Decomposition

Partition every requirement or design space into Mutually Exclusive, Collectively Exhaustive sub-questions.

**How to apply**:
- Use binary splits: "Is this driven by [A] or [B]?"
- Ensure sub-questions do not overlap (mutually exclusive)
- Ensure sub-questions together cover the full space (collectively exhaustive)
- Applied universally — not just for complex topics

**Atomic-topic exemption**: If a topic is non-decomposable (e.g., "What is the project name?"), MECE is skipped. Note the rationale: "Atomic topic — no decomposition needed."

**Profile interaction**:
- `shallow`: MECE limited to top-level decomposition, max 2 branches
- `standard`: MECE on requirement spaces
- `deep`: full MECE decomposition at all levels

---

## Socratic Type Rotation

Every question is annotated with one of 6 Socratic types. The type tag is visible to the user.

| Tag | Purpose | Example |
|-----|---------|---------|
| [Clarification] | Clarify meaning | "What do you mean by X?" |
| [Assumption] | Surface hidden premise | "Why do you assume Y?" |
| [Evidence] | Challenge factual basis | "What data supports this?" |
| [Viewpoint] | Introduce alternative perspective | "How would [stakeholder] see this differently?" |
| [Implication] | Explore consequences | "What follows if we accept this?" |
| [Meta] | Examine the question itself | "Why is this question important?" |

**Balance rule**: No single type exceeds 2x its fair share across a session. For example, with 12 questions, no type appears more than 4 times (2 × 12/6 = 4).

Annotate every question — initial questions, follow-ups, and challenge questions all carry a `[Type]` tag.

---

## Depth Profiles

Three intensity levels controlling how techniques are applied:

### shallow

- OARS: Reflect only (no Open follow-ups triggering laddering; Summarize still fires at stage transitions)
- Laddering: disabled
- MECE: top-level decomposition only, max 2 branches
- Socratic rotation: type tags applied but no aggressive re-probing

### standard

- OARS: full sequence (Open, Acknowledge, Reflect, Summarize)
- Laddering: 1-2 levels maximum
- MECE: applied on requirement spaces
- Socratic rotation: full 6-type rotation

### deep

- OARS: full sequence at full intensity
- Laddering: unlimited up to safety valve (7 levels)
- MECE: full decomposition at all levels
- Socratic rotation: full 6-type rotation with strict balance enforcement

### Mode defaults

| Mode | Default profile |
|------|----------------|
| standalone | deep |
| spec-mode | deep |
| backlog-mode | standard |

Mode stage/question limits take precedence over profile technique intensity (e.g., backlog-mode with deep profile still respects the 3-stage / 2-question-per-stage limits).

---

## Questioning Protocol

### AskUserQuestion Enforcement

Use `AskUserQuestion` for every question — one question per turn, never batched. Wait for the user's answer before asking the next. If AskUserQuestion is unavailable, fall back to conversational questions (one at a time, wait for response).

One question per turn is absolute: never ask two questions in the same message, even if they seem related.

### Preview for Visual Alternatives

When a question presents 2+ distinct approaches with structural differences (architecture diagrams, code snippets, file trees), use `preview` on each AskUserQuestion option. Each preview should contain a Markdown code block showing the approach — Mermaid diagram source, code snippet, or ASCII layout. Keep preview content concise (under 15 lines per option) for the 60-second AskUserQuestion timeout.

**When to use preview:**
- Stage 4 (Alternatives): comparing architecture approaches, data models, API shapes
- Any stage where the user must choose between visual alternatives with structural differences

**When NOT to use preview:**
- Purely textual questions with no visual alternatives (e.g., "What is out of scope?") MUST NOT include preview
- Questions with a single approach (no comparison needed) skip preview

**Fallback:** If AskUserQuestion is unavailable, show preview content inline as Markdown code blocks in the conversational question.

### Build Question List Upfront

Before asking any question, build the full question list grouped by stage. Show the list to the user at the start and after each answer.

Display format:

```
Stage 1: Clarity
  [1] [Clarification] What problem does this solve? — pending
  [2] [Assumption] Who is the primary user? — pending

Stage 2: Assumptions
  [3] [Assumption] What is assumed about the environment? — pending
```

### Stage Progression

When all questions in a stage are **answered** (status = answered), proceed to the next stage. Stage complete — proceed to the next stage only when every question in the current stage reaches answered status.

### Challenge Loop

After each user answer:

1. **Reflect** (OARS — mandatory): Restate what was heard to confirm understanding. Fires before challenge evaluation. Format: "So what I'm hearing is: [restatement]. Is that correct?"
2. **Acknowledge** (OARS): Factual recognition of what the answer addressed, without praise
3. **Evaluate**: Is the answer challengeable?
   - Vague, hand-waving, or unsubstantiated → challengeable
   - Clear, specific, measurable → not challengeable
4. **If challengeable and answer is abstract/vague**: Apply laddering ("Why is that important?" / "What specifically...") with a `[Type]` tag on the follow-up question
5. **If challengeable but not abstract**: Add a follow-up challenge question (status: **challenged**) with a `[Type]` tag. Maximum 2 follow-ups per question
6. After 2 follow-ups exhausted, OR when answer is complete: mark **answered** and state termination reason

State termination reason explicitly: "Marking as answered — [reason: 2 follow-ups exhausted / answer is sufficiently specific]"

### Cross-Stage Mutation

Any answer can add questions to any stage, including already-completed stages. Add questions to any stage when the answer reveals a gap.

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
Depth Profile: {shallow | standard | deep}
Stages completed: {N}/5
Questions answered: {X}/{total}
Questions skipped: {Y}

## Questions, Answers, and Challenges

### Stage 1: Clarity
**Q1**: [Clarification] {question text}
**A**: {answer}
**Challenge**: [Assumption] {challenge text} (if any)
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

Say "adversary mode" or "hard mode" to activate `grill-me-adversary` — a companion skill that adds answer scoring, adaptive follow-up probing, and question-generation challenge. The five-stage flow stays the same; only question selection and answer evaluation change. OARS Reflect fires before adversary scoring when adversary mode is active.
