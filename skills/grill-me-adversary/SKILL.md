---
name: grill-me-adversary
description: Opt-in companion to grill-me that adds adaptive adversarial questioning with answer scoring, follow-up probing, and question-generation challenge.
origin: ECC
---

# Grill-Me Adversary — Adaptive Interview Enhancement

Opt-in companion skill for `grill-me`. Activates when the user says "adversary mode" or "hard mode". Does not alter grill-me's five-stage structure — only enhances question selection and adds answer evaluation.

## Tone

Be firm but curious. Challenge the idea, not the person. Frame probes as "help me understand" rather than "you're wrong." Persistent probing without hostility extracts the most information.

## Question-Generation Challenge

At the start of each stage, evaluate grill-me's planned question: "Is this the hardest possible question for this stage?" A harder question targets a less obvious failure mode, requires more specific evidence, or challenges an unstated assumption. If a harder question exists, substitute it. Always show the challenge result to the user — whether the question was kept or replaced, and why.

## Adversarial Question Generation

Between questions within a stage, synthesize a devil's-advocate angle before picking the next question. Target the weakest point in the user's answers so far using these heuristics:

1. **lowest-scored axis** from prior answers in this stage
2. **most hedging or vague language** in the user's responses
3. **most critical to overall proposal viability** — the point whose failure would invalidate the entire idea

Avoid angles the user has already been pushed on. Never repeat a challenge already covered.

## Answer Evaluation Rubric

After each user answer, score on two axes and show scores inline:

**Completeness (0–3):**

| Score | Anchor |
|-------|--------|
| 0 | No relevant aspects addressed |
| 1 | One aspect addressed, major gaps remain |
| 2 | Most aspects addressed, minor gaps |
| 3 | All aspects addressed comprehensively |

**Specificity (0–3):**

| Score | Anchor |
|-------|--------|
| 0 | Entirely vague, no concrete details |
| 1 | Some detail but relies on hand-waving |
| 2 | Concrete examples or data for key claims |
| 3 | Specific, measurable, falsifiable throughout |

If either axis is below 2, probe deeper with a targeted follow-up before advancing to the next question.

## Deflections

If the user deflects instead of answering (e.g., "what do you think?", "I'm not sure"), redirect without scoring: "This interview is about YOUR thinking." Deflections do not consume follow-up attempts.

## Adaptive Loop Exit

Each branch terminates when:

- **Both axes score >= 2** → branch resolved (use grill-me's checkmark format)
- **Three follow-up attempts exhausted** without both axes reaching >= 2 → label as "stress-tested but unresolved"
- **User says "skip"** → label as "skipped" (distinct from exhaustion, does not consume attempts)

The three-attempt cap is absolute — after three follow-ups on a single branch, move on regardless of scores.
